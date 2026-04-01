use std::{collections::HashMap, path::Path};

use chrono::NaiveDate;
use scraper::{Html, Selector};

use crate::source::{Entry, Source, add_entry};

pub struct  NatureNews {
    remote_entries: HashMap<NaiveDate, Vec<Entry>>,
    entries: HashMap<NaiveDate, Vec<Entry>>
}

impl NatureNews {
    pub fn new(path: &Path) -> NatureNews {
        let mut source = NatureNews { remote_entries: HashMap::new(), entries: HashMap::new() };

        source.entries = source.load(path);
        source
    }
}

impl Source for NatureNews {

    fn name(&self) -> String { String::from("NatureNews") }

    fn base_url(&self) -> String { String::from("https://www.nature.com/nature/articles?type=news") }

    fn get_remote(&self) -> HashMap<NaiveDate, Vec<Entry>> {
        self.remote_entries.clone()
    }

    fn entries(&self) -> HashMap<NaiveDate, Vec<Entry>> {
        self.entries.clone()
    }

    async fn update_remote(&mut self, document: Html) {
        let article_selector = Selector::parse("article").unwrap();
        let time_selector = Selector::parse("time").unwrap();
        let link_selector = Selector::parse("a").unwrap();
        let picture_selector = Selector::parse("picture").unwrap();
        let img_selector = Selector::parse("img").unwrap();

        for article in document.select(&article_selector) {
            let sdate = match article.select(&time_selector).into_iter().next() {
                Some(t) => {
                    match t.text().collect::<Vec<_>>().first() {
                        Some(t) => t.to_string(),
                        None => continue
                    }
                },
                None => continue
            };
            let date = match NaiveDate::parse_from_str(&sdate, "%d %b %Y") {
                Ok(d) => d,
                Err(_) => continue
            };

            let (title, link) = match article.select(&link_selector).into_iter().next() {
                Some(a) => {
                    match a.attr("href") {
                        Some(href) => (a.text().collect::<Vec<_>>().join(""), href),
                        None => continue
                    }
                },
                None => continue
            };

            let mut picture_url = String::new();
            match article.select(&picture_selector).into_iter().next() {
                Some(pic) => {
                    match pic.select(&img_selector).into_iter().next() {
                        Some(img) => {
                            match img.attr("src") {
                                Some(src) => picture_url = src.to_string(),
                                None => {}
                            }
                        },
                        None => {}
                    }
                },
                None => {}
            }

            let url = format!("https://www.nature.com{}", link);

            add_entry(&mut self.remote_entries, date, title, url, Some(picture_url));

        }

    }
}