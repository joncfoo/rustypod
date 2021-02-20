use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::NaiveDateTime;
use log::debug;
use sqlx::migrate::Migrator;
use sqlx::{query_as, Pool, Sqlite, SqlitePool};

static MIGRATOR: Migrator = sqlx::migrate!();

pub enum Connection {
    Memory,
    File(String),
}

pub struct Database {
    pool: Pool<Sqlite>,
}

pub struct Podcast {
    id: i64,
    title: String,
    url: String,
    description: Option<String>,
    author: Option<String>,
    last_checked: Option<NaiveDateTime>,
}

impl Database {
    pub async fn new(connection: Connection) -> Result<Database> {
        let dbfile = match connection {
            Connection::Memory => String::from("sqlite::memory:"),
            Connection::File(path) => format!("file://{}?mode=rwc", path).to_string(),
        };
        debug!("database = {}", dbfile);

        let pool = SqlitePool::connect(&dbfile).await?;
        MIGRATOR.run(&pool).await?;

        Ok(Database { pool })
    }

    pub async fn create_podcast(&self, p: &Podcast) -> Result<Podcast> {
        let mut tx = self.pool.begin().await?;
        let podcast = sqlx::query_as!(
            Podcast,
            r#"
            insert into podcasts
                (title, url, description, author, last_checked)
            values
                (?, ?, ?, ?, ?);

            select * from podcasts where id = last_insert_rowid();
            "#,
            p.title,
            p.url,
            p.description,
            p.author,
            p.last_checked
        )
        .fetch_one(&mut tx)
        .await?;
        tx.commit().await?;

        Ok(podcast)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{Fake, Faker};

    #[async_std::test]
    async fn test_create_podcast() -> Result<()> {
        let d = Database::new(Connection::Memory).await?;
        let p = Podcast {
            id: 0,
            title: Faker.fake(),
            url: Faker.fake(),
            description: Faker.fake(),
            author: Faker.fake(),
            last_checked: Faker.fake(),
        };
        let podcast = d.create_podcast(&p).await?;

        assert_eq!(podcast.id > 0, true);
        assert_eq!(podcast.title, p.title);
        assert_eq!(podcast.url, p.url);
        assert_eq!(podcast.description, p.description);
        assert_eq!(podcast.author, p.author);
        assert_eq!(podcast.last_checked, p.last_checked);

        Ok(())
    }
}
