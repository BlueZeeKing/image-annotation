begin;

create table if not exists images (id integer primary key autoincrement);

create table if not exists annotations (
    id integer primary key autoincrement,
    image integer not null references images(id),
    x1 integer not null,
    y1 integer not null,
    x2 integer not null,
    y2 integer not null
);

create table if not exists dimensions (
    id integer primary key autoincrement,
    image integer not null references images(id),
    width integer not null,
    height integer not null
);

end;
