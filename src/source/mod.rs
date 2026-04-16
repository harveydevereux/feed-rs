pub mod bbcfuture;
pub mod bbcinpictures;
pub mod naturenews;
pub mod photosoftheday;
pub mod subreddit;
pub mod weekinwildlife;

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
    path::Path,
    time::Duration,
};

use chrono::NaiveDate;
use reqwest::{Client, StatusCode};
use scraper::Html;
use serde::{Deserialize, Serialize};

use crate::{
    integrations::discord::{Embed, Image, post_discord},
    util::read_file_utf8,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entry {
    title: String,
    url: String,
    preview_image_url: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub score: u64,
}

impl Entry {
    pub fn as_discord_post(&self, name: Option<&str>) -> Embed {
        let image = match self.preview_image_url != "" {
            true => Some(Image {
                url: self.preview_image_url.clone(),
            }),
            false => None,
        };

        let mut title = match name {
            Some(n) => format!("{} | {}", n, self.title),
            None => self.title.clone(),
        };

        if title.len() > 128 {
            title = title[0..125].to_string();
            for i in 0..2 {
                title.push_str(".");
            }
        }

        Embed {
            url: self.url.clone(),
            title: title,
            image: image,
        }
    }
}

pub struct DatedEntry {
    pub entry: Entry,
    pub date: NaiveDate,
}

pub fn add_entry(
    remote_entries: &mut HashMap<NaiveDate, Vec<Entry>>,
    date: NaiveDate,
    title: String,
    url: String,
    picture_url: Option<String>,
    score: Option<u64>,
) {
    let pic = match picture_url {
        Some(s) => s,
        None => String::new(),
    };

    let s = match score {
        Some(s) => s,
        None => 1,
    };

    if remote_entries.contains_key(&date) {
        remote_entries.get_mut(&date).unwrap().push(Entry {
            title,
            url: url,
            preview_image_url: pic,
            score: s,
        });
    } else {
        remote_entries.insert(
            date,
            vec![Entry {
                title,
                url: url,
                preview_image_url: pic,
                score: s,
            }],
        );
    }
}

pub trait Source {
    fn name(&self) -> String;

    fn id(&self) -> String {
        self.name().to_lowercase().trim().replace(" ", "-")
    }

    fn urls(&self) -> HashSet<String> {
        let mut urls = HashSet::<String>::new();
        for (_, entries) in self.entries() {
            for entry in entries {
                urls.insert(entry.url);
            }
        }
        urls
    }

    fn base_url(&self) -> String;

    fn get_remote(&self) -> HashMap<NaiveDate, Vec<Entry>>;

    async fn update_remote(&mut self, document: Html);

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

        let html = resp.text().await.unwrap();
        let document = Html::parse_document(&html);
        self.update_remote(document).await;
    }

    fn entries(&self) -> HashMap<NaiveDate, Vec<Entry>>;

    fn new_entries(&self) -> Vec<DatedEntry> {
        let mut new: Vec<DatedEntry> = Vec::new();
        let mut seen_urls = self.urls();
        let remote = self.get_remote();

        for (date, entries) in remote {
            for entry in entries {
                if !seen_urls.contains(&entry.url) {
                    seen_urls.insert(entry.url.to_string());
                    new.push(DatedEntry { entry, date });
                }
            }
        }

        new
    }

    async fn commit(&self, data: &Path, webhook: Option<String>) {
        match webhook {
            Some(s) => {
                for dentry in self.new_entries() {
                    post_discord(&s, &dentry.entry, &self.name()).await;
                }
            }
            None => {}
        }

        let mut entries = self.entries();
        for dentry in self.new_entries() {
            if entries.contains_key(&dentry.date) {
                entries.get_mut(&dentry.date).unwrap().push(dentry.entry);
            } else {
                entries.insert(dentry.date, vec![dentry.entry]);
            }
        }

        let yaml = serde_yaml::to_string(&entries).unwrap();

        let mut file = File::create(data.join(format!("{}.yml", self.id()))).unwrap();
        file.write_all(yaml.as_bytes()).unwrap();
    }

    fn load(&self, data: &Path) -> HashMap<NaiveDate, Vec<Entry>> {
        let mut entries = HashMap::<NaiveDate, Vec<Entry>>::new();

        let file = data.join(format!("{}.yml", self.id()));

        if file.exists() {
            match read_file_utf8(&file) {
                Some(data) => {
                    entries = match serde_yaml::from_str(&data) {
                        Ok(data) => data,
                        Err(why) => HashMap::new(),
                    };
                }
                None => {}
            }
        }

        entries
    }
}
