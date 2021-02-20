create table if not exists podcasts
(
    id           integer primary key not null,
    title        text                not null,
    url          text                not null unique,
    description  text,
    author       text,
    last_checked datetime
);

create table if not exists episodes
(
    id          integer primary key not null,
    podcast_id  integer             not null,
    title       text                not null,
    url         text                not null,
    description text,
    pubdate     integer,
    duration    integer,
    played      integer,
    foreign key (podcast_id) references podcasts (id) on delete cascade
);

create table if not exists files
(
    id         integer primary key not null,
    episode_id integer             not null,
    path       text                not null unique,
    foreign key (episode_id) references episodes (id) on delete cascade
);
