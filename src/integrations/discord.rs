use serde::{Deserialize, Serialize};

use crate::source::Entry;

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub url: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Embed {
    pub url: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<Image>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    pub content: Option<String>,
    pub embeds: Vec<Embed>
}

pub async fn post_discord(webhook: &str, entry: &Entry, name: &str) {

    let client = reqwest::Client::new();

    let post = Post { embeds: vec![entry.as_discord_post(Some(name))], content: None };
    match client.post(webhook)
        .json(&post)
        .send()
        .await
    {
        Ok(_) => {},
        Err(e) => {println!("{:?}", e)}
    }
}

pub async fn post_summary(webhook: &str, entries: Vec<Entry>, header: &str) {

    if entries.len() < 1 { return }

    let client = reqwest::Client::new();

    let mut embeds = Vec::new();
    for entry in entries {
        embeds.push(entry.as_discord_post(None));
    }
    let mut post = Post { embeds: embeds, content: Some(header.to_string()) };

    match client.post(webhook)
        .json(&post)
        .send()
        .await
    {
        Ok(_) => {},
        Err(e) => {println!("{:?}", e)}
    }
}
