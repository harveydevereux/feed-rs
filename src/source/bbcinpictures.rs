use std::{collections::HashMap, path::Path};

use chrono::{Datelike, Local, NaiveDate, TimeDelta};
use scraper::{ElementRef, Html, Selector};

use crate::source::{Entry, Source, add_entry};

pub struct BBCInPictures {
    remote_entries: HashMap<NaiveDate, Vec<Entry>>,
    entries: HashMap<NaiveDate, Vec<Entry>>,
}

impl BBCInPictures {
    pub fn new(path: &Path) -> BBCInPictures {
        let mut source = BBCInPictures {
            remote_entries: HashMap::new(),
            entries: HashMap::new(),
        };

        source.entries = source.load(path);
        source
    }
}

impl Source for BBCInPictures {
    fn name(&self) -> String {
        String::from("BBCInPictures")
    }

    fn base_url(&self) -> String {
        String::from("https://www.bbc.co.uk/news/in_pictures")
    }

    fn get_remote(&self) -> HashMap<NaiveDate, Vec<Entry>> {
        self.remote_entries.clone()
    }

    fn entries(&self) -> HashMap<NaiveDate, Vec<Entry>> {
        self.entries.clone()
    }

    async fn update_remote(&mut self, document: Html) {
        let div_selector = Selector::parse("div").unwrap();
        let li_selector = Selector::parse("li").unwrap();
        let link_selector = Selector::parse("a").unwrap();
        let picture_selector = Selector::parse("picture").unwrap();
        let span_selector = Selector::parse("span").unwrap();
        let img_selector = Selector::parse("img").unwrap();

        let now = Local::now();

        for listitem in document.select(&li_selector) {
            let link = match listitem.select(&link_selector).into_iter().next() {
                Some(a) => a,
                None => continue,
            };

            let url = match link.attr("href") {
                Some(href) => format!("https://www.bbc.co.uk{}", href),
                None => continue,
            };

            let picture = match listitem.select(&picture_selector).into_iter().next() {
                Some(p) => p,
                None => continue,
            };

            for div in listitem.select(&div_selector) {
                let found = false;

                match div.text().collect::<Vec<_>>().first() {
                    Some(t) => {
                        if *t != "Posted" {
                            continue;
                        }
                    }
                    None => continue,
                };

                let parent = match div.parent() {
                    Some(p) => match ElementRef::wrap(p) {
                        Some(p) => p,
                        None => continue,
                    },
                    None => continue,
                };

                for span in parent.select(&span_selector) {
                    if span.attr("aria-hidden").is_some() {
                        let sdate = span.text().collect::<Vec<_>>().join(" ");

                        let date: NaiveDate = if sdate.contains(" ") {
                            match NaiveDate::parse_from_str(
                                &format!("{} {}", sdate, now.year()),
                                "%d %b %Y",
                            ) {
                                Ok(d) => d,
                                Err(_) => continue,
                            }
                        } else {
                            let delta: TimeDelta = if sdate.contains("min") {
                                match sdate.split("min").collect::<Vec<_>>().first() {
                                    Some(i) => match i.to_string().parse::<i64>() {
                                        Ok(i) => TimeDelta::minutes(i),
                                        Err(_) => continue,
                                    },
                                    None => continue,
                                }
                            } else if sdate.contains("h") {
                                match sdate.split("h").collect::<Vec<_>>().first() {
                                    Some(i) => match i.to_string().parse::<i64>() {
                                        Ok(i) => TimeDelta::hours(i),
                                        Err(_) => continue,
                                    },
                                    None => continue,
                                }
                            } else if sdate.contains("d") {
                                match sdate.split("d").collect::<Vec<_>>().first() {
                                    Some(i) => match i.to_string().parse::<i64>() {
                                        Ok(i) => TimeDelta::days(i),
                                        Err(_) => continue,
                                    },
                                    None => continue,
                                }
                            } else {
                                continue;
                            };
                            match now.checked_sub_signed(delta) {
                                Some(time) => {
                                    match NaiveDate::from_ymd_opt(
                                        time.year(),
                                        time.month(),
                                        time.day(),
                                    ) {
                                        Some(date) => date,
                                        None => continue,
                                    }
                                }
                                None => continue,
                            }
                        };
                        let title = link.text().collect::<Vec<_>>().join("");
                        let mut picture_url = String::new();
                        for img in picture.select(&img_selector) {
                            match img.attr("src") {
                                Some(src) => {
                                    picture_url = src.to_string();
                                    break;
                                }
                                None => {}
                            }
                        }
                        add_entry(
                            &mut self.remote_entries,
                            date,
                            title,
                            url.to_string(),
                            Some(picture_url),
                            None,
                        );
                    }
                }
                if found {
                    break;
                }
            }
        }
    }
}
