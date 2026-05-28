use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use chrono::{Datelike, NaiveDate};

use scraper::{ElementRef, Html, Selector};

use crate::source::{Entry, Source, add_entry};

pub struct ArchaeologyNews {
    remote_entries: HashMap<NaiveDate, Vec<Entry>>,
    entries: HashMap<NaiveDate, Vec<Entry>>,
}

impl ArchaeologyNews {
    pub fn new(path: &Path) -> ArchaeologyNews {
        let mut source = ArchaeologyNews {
            remote_entries: HashMap::new(),
            entries: HashMap::new(),
        };

        source.entries = source.load(path);
        source
    }
}

impl Source for ArchaeologyNews {
    fn name(&self) -> String {
        String::from("ArchaeologyNews")
    }

    fn base_url(&self) -> String {
        String::from("https://archaeology.org/news/")
    }

    fn get_remote(&self) -> HashMap<NaiveDate, Vec<Entry>> {
        self.remote_entries.clone()
    }

    fn entries(&self) -> HashMap<NaiveDate, Vec<Entry>> {
        self.entries.clone()
    }

    async fn update_remote(&mut self, document: Html) {
        let div_selector = Selector::parse("div").unwrap();
        let link_selector = Selector::parse("a").unwrap();
        let img_selector = Selector::parse("img").unwrap();
        let url_pattern = format!("https://archaeology.org/news/{}", chrono::Utc::now().year());

        let mut urls = HashSet::<String>::new();

        for div in document.select(&div_selector) {
            for link in div.select(&link_selector) {
                match link.attr("href") {
                    Some(url) => {

                        if urls.contains(url) {
                            continue;
                        }

                        urls.insert(url.to_string());

                        if url.starts_with(&url_pattern) {
                            let title = link.text().collect::<Vec<_>>().join("").replace("\n", "").replace("\t", "");
                            let article: ElementRef<'_> = match link.parent() {
                                Some(p) => match ElementRef::wrap(p) {
                                    Some(p) =>
                                        match p.parent() {
                                            Some(p) => match ElementRef::wrap(p) {
                                                Some(p) => match p.parent() {
                                                    Some(p) => match ElementRef::wrap(p) {
                                                        Some(p) => p,
                                                        None => continue,
                                                    },
                                                    None => continue,
                                                },
                                                None => continue,
                                            },
                                        None => continue,
                                    },
                                    None => continue,
                                },
                                None => continue,
                            };


                            let date = match extract_date(article) {
                                Some(d) => d,
                                None => continue
                            };

                            let picture_url = match article.select(&img_selector).into_iter().next() {
                                Some(img) => {
                                    match img.attr("src") {
                                        Some(src) => {
                                            if !src.contains("Archaeology-Magazine-Logo-Square") {
                                                Some(src.to_string())
                                            }
                                            else {
                                                None
                                            }
                                        }
                                        None => None
                                    }
                                },
                                None => None
                            };

                            add_entry(
                                &mut self.remote_entries,
                                date,
                                title,
                                url.to_string(),
                                picture_url,
                                None,
                            );
                        }
                    }
                    None => {
                        continue;
                    }
                }
            }
        }


    }
}

fn extract_date(article: ElementRef<'_>) -> Option<NaiveDate> {
    let span_selector = Selector::parse("span").unwrap();
    for span in article.select(&span_selector) {
        let text = span.text().collect::<Vec<_>>().join("");
        for line in text.split("\n") {
            match NaiveDate::parse_from_str(
                &line,
                "%B %d, %Y",
            ) {
                Ok(d) => {
                    return Some(d)
                },
                Err(_) => continue,
            };
        }
    }
    None
}
