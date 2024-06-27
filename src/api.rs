use std::sync::Arc;

use anyhow::anyhow;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{primitives::ByteStream, Client as S3Client};
use axum::{
    extract::{Path, Request, State},
    http::{header::CONTENT_TYPE, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use libsql::{Builder, Connection as DbConnection, Database};
use multer::parse_boundary;
use serde::Deserialize;
use tracing::{error, warn};

use crate::templates::Image;

const BUCKET: &str = "image-annotation";

#[derive(Clone)]
pub struct ImageState {
    database: Arc<Database>,
    s3: S3Client,
}

impl ImageState {
    pub fn database(&self) -> libsql::Result<DbConnection> {
        self.database.connect()
    }

    pub fn s3(&self) -> S3Client {
        self.s3.clone()
    }
}

impl ImageState {
    pub async fn new() -> anyhow::Result<ImageState> {
        let db = if let Ok(url) = std::env::var("LIBSQL_URL") {
            let token = std::env::var("LIBSQL_AUTH_TOKEN").unwrap_or_else(|_| "".to_string());

            Builder::new_remote(url, token).build().await?
        } else {
            Builder::new_local("data.db").build().await?
        };

        let connection = db.connect()?;

        connection
            .execute_batch(include_str!("../initial.sql"))
            .await?;

        let client = S3Client::new(&aws_config::load_defaults(BehaviorVersion::latest()).await);

        Ok(Self {
            s3: client,
            database: Arc::new(db),
        })
    }
}

pub async fn get_router() -> Router<ImageState> {
    Router::new()
        .route("/upload_file", post(upload_file))
        .route("/get_image_data/:id", get(get_image_data))
        .route("/add_annotations/:id", post(add_annotations))
}

pub async fn upload_file(State(state): State<ImageState>, req: Request) -> Response {
    let Some(content_type_header) = req
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|val| val.to_str().ok())
    else {
        warn!("Bad content type header");

        return StatusCode::BAD_REQUEST.into_response();
    };

    let Ok(boundary) = parse_boundary(content_type_header) else {
        warn!("Couldn't parse form data boundary");

        return StatusCode::BAD_REQUEST.into_response();
    };

    let mut data = multer::Multipart::new(req.into_body().into_data_stream(), boundary);

    loop {
        let file = match data.next_field().await {
            Ok(Some(value)) => value,
            Ok(None) => break,
            _ => {
                warn!("No field in request");

                return StatusCode::BAD_REQUEST.into_response();
            }
        };

        let Some(file_type) = file.content_type().map(|val| val.to_owned()) else {
            warn!("Failed to get content type");

            return StatusCode::BAD_REQUEST.into_response();
        };

        let Ok(file) = file.bytes().await else {
            warn!("Failed to get file bytes");

            return StatusCode::BAD_REQUEST.into_response();
        };

        let database = match state.database.connect() {
            Ok(connection) => connection,
            Err(err) => {
                warn!(?err, "Couldn't connect to db");

                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        let s3 = state.s3();

        tokio::spawn(async move {
            let id: anyhow::Result<_> = try {
                let mut rows = database
                    .query("insert into images default values returning id", ())
                    .await?;

                let row = rows
                    .next()
                    .await?
                    .ok_or_else(|| anyhow!("Could not create row"))?;

                row.get::<u64>(0)?
            };

            let id = match id {
                Ok(id) => id,
                Err(err) => {
                    error!(?err, "couldn't create new image in db");

                    return;
                }
            };

            let res = s3
                .put_object()
                .key(&format!("image-{}", id))
                .bucket(BUCKET)
                .body(ByteStream::from(file))
                .content_type(file_type.to_string())
                .send()
                .await;

            if let Err(err) = res {
                if let Err(err) = database
                    .execute("delete from images where id = (?1)", [id])
                    .await
                {
                    error!(?err, "couldn't delete image from db when upload failed");
                }

                error!(?err, "couldn't upload image");
            }
        });
    }

    Redirect::to("/").into_response()
}

pub async fn get_random_image(State(state): State<ImageState>) -> Response {
    let id: anyhow::Result<_> = try {
        let mut rows = state
            .database.connect()?
            .query("select id from images where not exists (select * from annotations where image = images.id) order by random() limit 1;", ())
            .await?;

        let Some(row) = rows.next().await? else {
            return Redirect::to("/no_images").into_response();
        };

        row.get::<u64>(0)?
    };

    let id = match id {
        Ok(id) => id,
        Err(err) => {
            error!(?err, "couldn't create new image in db");

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    Image {
        url: format!("/api/get_image_data/{id}"),
        id,
    }
    .into_response()
}

#[derive(Deserialize)]
pub struct ImageId {
    id: u64,
}

pub async fn get_image_data(State(state): State<ImageState>, Path(id): Path<ImageId>) -> Response {
    let file = match state
        .s3
        .get_object()
        .bucket(BUCKET)
        .key(&format!("image-{}", id.id))
        .send()
        .await
    {
        Ok(val) => val,
        Err(err) => {
            error!(?err, "couldn't get image from s3");

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let Some(content_type) = file.content_type else {
        error!("Couldn't get content type");

        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let body = match file.body.collect().await {
        Ok(val) => val,
        Err(err) => {
            error!(?err, "couldn't get image from s3");

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    ([(CONTENT_TYPE, content_type)], body.into_bytes()).into_response()
}

#[derive(Deserialize)]
pub struct Annotation {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

#[derive(Deserialize)]
pub struct AnnotationGroup {
    annotations: Vec<Annotation>,
    width: u64,
    height: u64,
}

pub async fn add_annotations(
    State(state): State<ImageState>,
    Path(id): Path<ImageId>,
    Json(annotations): Json<AnnotationGroup>,
) -> impl IntoResponse {
    let mut statements = format!(
        "begin transaction;delete from annotations where image = {};",
        id.id
    );
    for annotation in annotations.annotations {
        statements.push_str(&format!(
            "insert into annotations (image, x1, y1, x2, y2) values ({}, {}, {}, {}, {});",
            id.id, annotation.x1, annotation.y1, annotation.x2, annotation.y2
        ));
    }
    statements.push_str(&format!(
        "insert into dimensions (image, width, height) values ({}, {}, {});",
        id.id, annotations.width, annotations.height
    ));
    statements.push_str("commit transaction;");

    let connection = match state.database.connect() {
        Ok(connection) => connection,
        Err(err) => {
            error!(?err, "Failed to connect to db");

            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    match connection.execute_batch(&statements).await {
        Ok(()) => StatusCode::OK,
        Err(err) => {
            error!(?err, "couldn't upload annotations");

            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
