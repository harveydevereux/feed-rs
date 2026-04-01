use std::{collections::HashMap, path::Path};

use chrono::NaiveDate;
use scraper::{Html, Selector};

use crate::source::{Entry, Source, add_entry};

pub struct  WeekInWildlife {
    remote_entries: HashMap<NaiveDate, Vec<Entry>>,
    entries: HashMap<NaiveDate, Vec<Entry>>
}

impl WeekInWildlife {
    pub fn new(path: &Path) -> WeekInWildlife {
        let mut source = WeekInWildlife { remote_entries: HashMap::new(), entries: HashMap::new() };

        source.entries = source.load(path);
        source
    }
}

impl Source for WeekInWildlife {

    fn name(&self) -> String { String::from("WeekInWildlife") }

    fn base_url(&self) -> String { String::from("https://www.theguardian.com/environment/series/weekinwildlife") }

    fn get_remote(&self) -> HashMap<NaiveDate, Vec<Entry>> {
        self.remote_entries.clone()
    }

    fn entries(&self) -> HashMap<NaiveDate, Vec<Entry>> {
        self.entries.clone()
    }

    async fn update_remote(&mut self, document: Html) {
        let section_selector = Selector::parse("section").unwrap();
        let link_selector = Selector::parse("a").unwrap();
        let picture_selector = Selector::parse("picture").unwrap();
        let img_selector = Selector::parse("img").unwrap();
        for section in document.select(&section_selector) {

            let date = match section.value().id() {
                Some(t) => {
                    match NaiveDate::parse_from_str(t, "%d-%B-%Y") {
                        Ok(d) => d,
                        Err(_) => continue
                    }
                },
                None => continue
            };

            let picture_url: String = match section.select(&picture_selector).into_iter().next() {
                Some(pic) => {
                    match pic.select(&img_selector).into_iter().next() {
                        Some(img) => {
                            match img.attr("src") {
                                Some(src) => src.to_string(),
                                None => String::new()
                            }
                        },
                        None => String::new()
                    }
                },
                None => String::new()
            };

            for link in section.select(&link_selector) {
                match link.attr("aria-label") {
                    Some(title) => {
                        let url = match link.attr("href") {
                            Some(href) => format!("https://www.theguardian.com{}", href),
                            None => continue
                        };
                        add_entry(&mut self.remote_entries, date, title.to_string(), url, Some(picture_url));
                        break
                    },
                    None => continue
                }
            }

        }

    }
}