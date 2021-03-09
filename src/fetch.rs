use anyhow::{anyhow, bail, Context, Result};
use log::debug;
use rss::Channel;

use crate::db;
use crate::db::Podcast;
use chrono::Utc;

fn fetch(url: String) -> Result<Podcast> {
    // TODO use ETag to cache

    let response: minreq::Response = minreq::get(&url).send()?;
    if response.status_code < 200 || response.status_code > 299 {
        // error
        bail!("Failed to fetch {}", &url)
    }
    //response.
    let channel = Channel::read_from(response.as_bytes())?;

    let podcast = db::Podcast {
        id: 0,
        title: channel.title,
        url: url.clone(),
        description: channel.description,
        enabled: true,
        last_checked: Utc::now().naive_utc(),
        image_url: channel.itunes_ext.and_then(|i| i.image),
        cache_key: response.headers.get("etag").map(|s| s.clone()),
    };

    Ok(podcast)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch() -> Result<()> {
        let podcast =
            fetch("https://static.anjunabeats.com/anjunabeats-worldwide/podcast.xml".to_string())?;

        Ok(())
    }
}
