use askama::Template;

#[derive(Template)]
#[template(path = "upload.html")]
pub struct Upload {}

#[derive(Template)]
#[template(path = "image.html")]
pub struct Image {
    pub url: String,
    pub id: u64,
}

#[derive(Template)]
#[template(path = "no_images.html")]
pub struct NoImages {}

#[derive(Template)]
#[template(path = "help.html")]
pub struct Help {}
