use std::fs::create_dir_all;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use log::debug;
use structopt::StructOpt;

mod db;
mod fetch;

#[derive(StructOpt)]
struct Args {
    /// Port to listen on
    #[structopt(short, long, default_value = "5000")]
    port: u16,

    /// Path to database
    #[structopt(short, long, parse(from_os_str))]
    database: Option<PathBuf>,
}

impl Args {
    fn database_file(&self) -> Result<String> {
        let file_path = match &self.database {
            None => {
                let dir = directories::ProjectDirs::from("org", "rustypod", "rustypod")
                    .map(|d| d.data_dir().to_path_buf())
                    .ok_or(anyhow!(
                        "failed to get default user data directory. please specify database -d/--database"
                    ))?;
                dir.join("database")
            }
            Some(p) => p.to_path_buf(),
        };

        if let Some(parent) = file_path.parent() {
            create_dir_all(parent).with_context(|| {
                format!("failed to create directory path: {:?}", parent.as_os_str())
            })?;
        }

        Ok(String::from(file_path.to_string_lossy()))
    }
}

struct App {
    db: db::Database,
}

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::from_args();
    let dbpath: String = args.database_file()?;

    let app = App {
        db: db::Database::new(db::Connection::File(dbpath)).await?,
    };

    debug!("oh hai");

    Ok(())
}
