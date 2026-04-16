use std::{collections::HashMap, path::Path, time::Duration};

use chrono::{Datelike, Local, NaiveDate};
use reqwest::{Client, StatusCode};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::source::{Entry, Source, add_entry};

pub struct Subreddit {
    remote_entries: HashMap<NaiveDate, Vec<Entry>>,
    entries: HashMap<NaiveDate, Vec<Entry>>,
    subreddit: String,
}

impl Subreddit {
    pub fn new(path: &Path, subreddit: String) -> Subreddit {
        let mut source = Subreddit {
            remote_entries: HashMap::new(),
            entries: HashMap::new(),
            subreddit: subreddit,
        };

        source.entries = source.load(path);
        source
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ChildData {
    permalink: String,
    thumbnail: String,
    title: String,
    created_utc: f64,
    ups: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Child {
    data: ChildData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Listings {
    children: Vec<Child>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Response {
    data: Listings,
}

impl Source for Subreddit {
    fn name(&self) -> String {
        self.subreddit.clone()
    }

    fn base_url(&self) -> String {
        format!("https://www.reddit.com/r/{}/.json", self.subreddit)
    }

    fn get_remote(&self) -> HashMap<NaiveDate, Vec<Entry>> {
        self.remote_entries.clone()
    }

    fn entries(&self) -> HashMap<NaiveDate, Vec<Entry>> {
        self.entries.clone()
    }

    async fn get(&mut self) {
        let client = Client::new();
        let mut resp = client
            .get(self.base_url())
            .header("Connection", "close")
            .header("User-Agent", "feed")
            .send()
            .await
            .unwrap();
        let mut tries = 0u8;

        while resp.status() != StatusCode::OK {
            resp = client
                .get(self.base_url())
                .header("Connection", "close")
                .header("User-Agent", "feed")
                .send()
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(1000)).await;
            tries += 1u8;
            if tries == 3u8 {
                break;
            }
        }

        let response: Response = resp.json().await.unwrap();

        for post in response.data.children {
            let time = match chrono::DateTime::from_timestamp(post.data.created_utc as i64, 0) {
                Some(t) => NaiveDate::from_ymd_opt(t.year(), t.month(), t.day()).unwrap(),
                None => continue,
            };
            add_entry(
                &mut self.remote_entries,
                time,
                post.data.title,
                format!("https://reddit.com/{}", post.data.permalink),
                None,
                Some(post.data.ups),
            );
        }
    }

    async fn update_remote(&mut self, _: Html) {}
}
