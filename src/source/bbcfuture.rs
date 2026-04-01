use std::{collections::{HashMap, HashSet}, path::Path, vec};

use chrono::NaiveDate;
use reqwest;
use scraper::{Html, Selector, CaseSensitivity::AsciiCaseInsensitive};

use crate::source::{Entry, Source, add_entry};

pub struct  BBCFuture {
    remote_entries: HashMap<NaiveDate, Vec<Entry>>,
    entries: HashMap<NaiveDate, Vec<Entry>>
}

impl BBCFuture {
    pub fn new(path: &Path) -> BBCFuture {
        let mut source = BBCFuture { remote_entries: HashMap::new(), entries: HashMap::new() };

        source.entries = source.load(path);
        source
    }
}

impl Source for BBCFuture {

    fn name(&self) -> String { String::from("BBCFuture") }

    fn base_url(&self) -> String { String::from("https://www.bbc.co.uk/future") }

    fn get_remote(&self) -> HashMap<NaiveDate, Vec<Entry>> {
        self.remote_entries.clone()
    }

    fn entries(&self) -> HashMap<NaiveDate, Vec<Entry>> {
        self.entries.clone()
    }

    async fn update_remote(&mut self, document: Html) {
        let div_selector = Selector::parse("div").unwrap();
        let link_selector = Selector::parse("a").unwrap();
        let mut urls: HashSet<String> = HashSet::new();

        for div in document.select(&div_selector) {
            for link in div.select(&link_selector) {
                match link.attr("href") {
                    Some(url) => {
                        if url.starts_with("/future/article/") {
                            urls.insert(format!("https://www.bbc.co.uk{}", url));
                            break;
                        }
                    },
                    None => {continue;}
                }
            }
        }

        let h1_selector = Selector::parse("h1").unwrap();
        let img_selector = Selector::parse("img").unwrap();
        let span_selector = Selector::parse("span").unwrap();
        for url in &urls {
            let page = Html::parse_document(&reqwest::get(url)
                    .await.unwrap()
                    .text()
                    .await.unwrap());
            let titles: Vec<_> = page.select(&h1_selector).collect();

            let title = match titles.first() {
                Some(t) => {
                    match t.text().collect::<Vec<_>>().first() {
                        Some(t) => t.to_string(),
                        None => continue
                    }
                },
                None => { continue; }
            };

            let mut picture_url = String::new();
            let mut date: Option<NaiveDate> = None;

            for div in page.select(&div_selector) {
                if date.is_some() && picture_url != String::new() { break; }

                if picture_url == String::new() && div.value().has_class("hero-image", AsciiCaseInsensitive) {
                    let imgs: Vec<_> = div.select(&img_selector).collect();
                    match imgs.first() {
                        Some(img) => {
                            match img.attr("src") {
                                Some(src) => { picture_url = src.to_string(); },
                                None => {}
                            }
                        },
                        None => {}
                    }
                }

                if date.is_none() && div.value().has_class("author-unit", AsciiCaseInsensitive) {
                    let spans: Vec<_> = div.select(&span_selector).collect();
                    match spans.first() {
                        Some(span) => {
                            let text: Vec<_> = span.text().collect();
                            match text.first() {
                                Some(t) => {
                                    let dmy: Vec<_> = t.split_whitespace().collect();
                                    if dmy.len() == 3 {
                                        let d = dmy[0].replace("rd", "").replace("st", "").replace("nd", "").replace("th", "");
                                        let sdate = format!("{} {} {}", d, dmy[1], dmy[2]);
                                        date = match NaiveDate::parse_from_str(&sdate, "%d %B %Y") {
                                            Ok(d) => Some(d),
                                            Err(_) => continue
                                        };
                                    }
                                    else {
                                        continue;
                                    }
                                },
                                None => { continue; }
                            }
                        },
                        None => {}
                    }
                }
            }

            if date.is_some() {
                let d = date.unwrap();
                add_entry(&mut self.remote_entries, d, title, url.to_string(), Some(picture_url));
            }

        }
    }
}