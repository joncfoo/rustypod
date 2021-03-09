create table if not exists podcasts (
    id integer primary key not null,
    title text not null,
    url text not null unique,
    description text not null,
    enabled boolean not null default true,
    last_checked datetime not null,
    image_url text,
    cache_key text
);

create table if not exists episodes (
    id integer primary key not null,
    podcast_id integer not null,
    title text not null,
    url text not null,
    description text,
    pubdate datetime,
    duration integer,
    played integer,
    path text,
    foreign key (podcast_id) references podcasts (id) on delete cascade
);
