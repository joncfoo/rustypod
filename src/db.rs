use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::NaiveDateTime;
use log::debug;
use sqlx::migrate::Migrator;
use sqlx::{Pool, Sqlite, SqlitePool};

static MIGRATOR: Migrator = sqlx::migrate!();

pub enum Connection {
    Memory,
    File(String),
}

pub struct Database {
    pool: Pool<Sqlite>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Podcast {
    id: i64,
    title: String,
    url: String,
    description: Option<String>,
    author: Option<String>,
    enabled: bool,
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
                (title, url, description, author, enabled, last_checked)
            values
                (?, ?, ?, ?, ?, ?);

            select * from podcasts where id = last_insert_rowid();
            "#,
            p.title,
            p.url,
            p.description,
            p.author,
            p.enabled,
            p.last_checked
        )
        .fetch_one(&mut tx)
        .await?;
        tx.commit().await?;

        Ok(podcast)
    }

    pub async fn update_podcast(&self, p: &Podcast) -> Result<Podcast> {
        let mut tx = self.pool.begin().await?;
        let podcast = sqlx::query_as!(
            Podcast,
            r#"
            update podcasts
            set
                title = ?,
                url = ?,
                description = ?,
                author = ?,
                enabled = ?,
                last_checked = ?
            where
                id = ?;

            select * from podcasts where id = ?;
            "#,
            p.title,
            p.url,
            p.description,
            p.author,
            p.enabled,
            p.last_checked,
            p.id,
            p.id,
        )
        .fetch_one(&mut tx)
        .await?;
        tx.commit().await?;

        Ok(podcast)
    }

    pub async fn delete_podcast(&self, p: &Podcast) -> Result<()> {
        let _ = sqlx::query(r#"delete from podcasts where id = ?"#)
            .bind(p.id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_podcast(&self, id: &i64) -> Result<Option<Podcast>> {
        let p = sqlx::query_as!(Podcast, r#"select * from podcasts where id = ?"#, id,)
            .fetch_optional(&self.pool)
            .await?;

        Ok(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{Fake, Faker};

    fn podcast() -> Podcast {
        Podcast {
            id: 0,
            title: Faker.fake(),
            url: Faker.fake(),
            description: Faker.fake(),
            author: Faker.fake(),
            enabled: Faker.fake(),
            last_checked: Faker.fake(),
        }
    }

    #[async_std::test]
    async fn test_create_podcast() -> Result<()> {
        let d = Database::new(Connection::Memory).await?;
        let p = podcast();
        let podcast = d.create_podcast(&p).await?;

        assert_eq!(podcast.id > 0, true);
        assert_eq!(podcast.title, p.title);
        assert_eq!(podcast.url, p.url);
        assert_eq!(podcast.description, p.description);
        assert_eq!(podcast.author, p.author);
        assert_eq!(podcast.enabled, p.enabled);
        assert_eq!(podcast.last_checked, p.last_checked);

        Ok(())
    }

    #[async_std::test]
    async fn test_update_podcast() -> Result<()> {
        let d = Database::new(Connection::Memory).await?;
        let mut p = d.create_podcast(&podcast()).await?;
        p = Podcast {
            id: p.id,
            ..podcast()
        };
        let podcast = d.update_podcast(&p).await?;

        assert_eq!(podcast.id, p.id);
        assert_eq!(podcast.title, p.title);
        assert_eq!(podcast.url, p.url);
        assert_eq!(podcast.description, p.description);
        assert_eq!(podcast.author, p.author);
        assert_eq!(podcast.enabled, p.enabled);
        assert_eq!(podcast.last_checked, p.last_checked);

        Ok(())
    }

    #[async_std::test]
    async fn test_delete_podcast() -> Result<()> {
        let d = Database::new(Connection::Memory).await?;
        let p = d.create_podcast(&podcast()).await?;
        let p2 = d.get_podcast(&p.id).await?;
        assert_eq!(p2, Some(p.clone()));

        d.delete_podcast(&p).await?;
        let p2 = d.get_podcast(&p.id).await?;
        assert_eq!(p2, None);

        Ok(())
    }
}
