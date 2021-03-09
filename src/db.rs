use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::migrate::Migrator;
use sqlx::{query, query_as, Pool, Sqlite, SqlitePool};

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
    pub id: i64,
    pub title: String,
    pub url: String,
    pub description: String,
    pub enabled: bool,
    pub last_checked: NaiveDateTime,
    pub image_url: Option<String>,
    pub cache_key: Option<String>,
}

impl Database {
    pub async fn new(connection: Connection) -> Result<Database> {
        let dbfile = match connection {
            Connection::Memory => String::from("sqlite::memory:"),
            Connection::File(path) => format!("file://{}?mode=rwc", path).to_string(),
        };

        let pool = SqlitePool::connect(&dbfile).await?;
        MIGRATOR.run(&pool).await?;

        Ok(Database { pool })
    }

    pub async fn create_podcast(&self, p: &Podcast) -> Result<Podcast> {
        let mut tx = self.pool.begin().await?;
        let podcast = query_as!(
            Podcast,
            r#"
            insert into podcasts
                (title, url, description, enabled, last_checked, image_url, cache_key)
            values
                (?, ?, ?, ?, ?, ?, ?)
            on conflict (url)
            do update set
                title = excluded.title,
                description = excluded.description,
                enabled = excluded.enabled,
                last_checked = excluded.last_checked,
                image_url = excluded.image_url,
                cache_key = excluded.cache_key;

            select * from podcasts where id = last_insert_rowid();
            "#,
            p.title,
            p.url,
            p.description,
            p.enabled,
            p.last_checked,
            p.image_url,
            p.cache_key,
        )
        .fetch_one(&mut tx)
        .await?;
        tx.commit().await?;

        Ok(podcast)
    }

    pub async fn update_podcast(&self, p: &Podcast) -> Result<Podcast> {
        let mut tx = self.pool.begin().await?;
        let podcast = query_as!(
            Podcast,
            r#"
            update podcasts
            set
                title = ?,
                url = ?,
                description = ?,
                enabled = ?,
                last_checked = ?,
                image_url = ?,
                cache_key = ?
            where
                id = ?;

            select * from podcasts where id = ?;
            "#,
            p.title,
            p.url,
            p.description,
            p.enabled,
            p.last_checked,
            p.image_url,
            p.cache_key,
            p.id,
            p.id,
        )
        .fetch_one(&mut tx)
        .await?;
        tx.commit().await?;

        Ok(podcast)
    }

    pub async fn delete_podcast(&self, p: &Podcast) -> Result<()> {
        let _ = query(r#"delete from podcasts where id = ?"#)
            .bind(p.id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_podcast(&self, id: &i64) -> Result<Option<Podcast>> {
        let p = query_as!(Podcast, r#"select * from podcasts where id = ?"#, id,)
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
            enabled: Faker.fake(),
            last_checked: Faker.fake(),
            image_url: Faker.fake(),
            cache_key: Faker.fake(),
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
        assert_eq!(podcast.enabled, p.enabled);
        assert_eq!(podcast.last_checked, p.last_checked);
        assert_eq!(podcast.image_url, p.image_url);
        assert_eq!(podcast.cache_key, p.cache_key);

        // inserting twice is ok
        let same_podcast = d.create_podcast(&podcast).await?;
        assert_eq!(&same_podcast, &podcast);

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
        assert_eq!(podcast.enabled, p.enabled);
        assert_eq!(podcast.last_checked, p.last_checked);
        assert_eq!(podcast.image_url, p.image_url);
        assert_eq!(podcast.cache_key, p.cache_key);

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
