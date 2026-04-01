use std::{collections::HashMap, path::Path};

use chrono::{Datelike, Local, NaiveDate};
use scraper::{Html, Selector};

use crate::source::{Entry, Source, add_entry};

pub struct  Subreddit {
    remote_entries: HashMap<NaiveDate, Vec<Entry>>,
    entries: HashMap<NaiveDate, Vec<Entry>>,
    subreddit: String
}

impl Subreddit {
    pub fn new(path: &Path, subreddit: String) -> Subreddit {
        let mut source = Subreddit { remote_entries: HashMap::new(), entries: HashMap::new(), subreddit: subreddit };

        source.entries = source.load(path);
        source
    }
}

impl Source for Subreddit {

    fn name(&self) -> String { self.subreddit.clone() }

    fn base_url(&self) -> String { format!("https://www.reddit.com/r/{}/top/?t=day&feedViewType=compactView", self.subreddit) }

    fn get_remote(&self) -> HashMap<NaiveDate, Vec<Entry>> {
        self.remote_entries.clone()
    }

    fn entries(&self) -> HashMap<NaiveDate, Vec<Entry>> {
        self.entries.clone()
    }

    async fn update_remote(&mut self, document: Html) {
        let article_selector = Selector::parse("article").unwrap();
        let link_selector = Selector::parse("a").unwrap();
        let img_selector = Selector::parse("img").unwrap();
        let shreddit_selector = Selector::parse("shreddit-post").unwrap();

        let now = Local::now();
        let today = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap();

        for article in document.select(&article_selector) {

            let title: String = match article.attr("aria-label") {
                Some(s) => s.to_string(),
                None => continue
            };

            let link: String = match article.select(&link_selector).into_iter().next() {
                Some(a) => {
                    match a.attr("href") {
                        Some(href) => format!("https://www.reddit.com{}", href),
                        None => continue
                    }
                },
                None => continue
            };

            let picture_url: String = match article.select(&img_selector).into_iter().next() {
                Some(pic) =>
                {
                    match pic.attr("alt") {
                        Some(alt) => {
                            if alt.to_lowercase().contains("avatar") {
                                String::new()
                            }
                            else {
                                match pic.attr("src") { Some(src) => src.to_string(), None => String::new()}
                            }
                        },
                        None => match pic.attr("src") { Some(src) => src.to_string(), None => String::new()}
                    }
                }
                None => String::new()
            };

            let date = match article.select(&shreddit_selector).into_iter().next() {
                Some(post) => {
                    match post.attr("created-timestamp") {
                        Some(t) => {
                            let ymd = t.split("T").collect::<Vec<_>>()[0];
                            match NaiveDate::parse_from_str(ymd, "%Y-%m-%d") {
                                Ok(date) => date,
                                Err(_) => today
                            }
                        },
                        None => today
                    }
                },
                None => today
            };

            add_entry(&mut self.remote_entries, date, title.to_string(), link, Some(picture_url));

        }

    }
}