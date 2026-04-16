use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::source::Entry;
use std::cmp::min;

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Embed {
    pub url: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<Image>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    pub content: String,
    pub embeds: Vec<Embed>,
}

pub async fn post_discord(webhook: &str, entry: &Entry, name: &str) {
    let client = reqwest::Client::new();

    let post = Post {
        embeds: vec![entry.as_discord_post(Some(name))],
        content: "".to_string(),
    };
    match client.post(webhook).json(&post).send().await {
        Ok(_) => {}
        Err(e) => {
            println!("{:?}", e)
        }
    }
}

pub async fn post_summary(webhook: &str, mut entries: Vec<Entry>, header: &str) {
    if entries.len() < 1 {
        return;
    }

    entries.sort_by(|a, b| a.score.cmp(&b.score));
    entries.reverse();

    for (i, batch) in entries.chunks(5).enumerate() {
        let client = reqwest::Client::new();

        let mut embeds = Vec::new();
        for entry in batch {
            embeds.push(entry.as_discord_post(None));
        }

        let post = Post {
            embeds: embeds,
            content: if i == 0 {
                header.to_string()
            } else {
                "".to_string()
            },
        };

        match client.post(webhook).json(&post).send().await {
            Ok(response) => {
                if response.status() != StatusCode::OK {
                    println!("{:?}", response);
                    println!("{}", serde_json::to_string_pretty(&post).unwrap());
                }
            }
            Err(e) => {
                println!("{:?}", e)
            }
        }
    }
}
