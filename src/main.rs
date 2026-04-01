use std::{fs::create_dir, path::Path, process::exit};

use feed_rs::{integrations::discord::post_summary, source::{
    Source, bbcfuture::BBCFuture, bbcinpictures::BBCInPictures, naturenews::NatureNews, photosoftheday::PhotosOfTheDay, subreddit::Subreddit, weekinwildlife::WeekInWildlife
}};

#[tokio::main]
async fn main(){

    let args: Vec<String> = std::env::args().collect();

    let mut path = Path::new("./data/");
    let mut webhook: Option<String> = None;

    if args.iter().any(|x| x == "path") {
        let index = args.iter().position(|x| x == "path").unwrap();
        if index+1 < args.len() {
            path = Path::new(&args[index+1]);
        }
    }

    if args.iter().any(|x| x == "webhook") {
        let index = args.iter().position(|x| x == "webhook").unwrap();
        if index+1 < args.len() {
            webhook = Some(args[index+1].clone());
        }
    }

    if args.iter().any(|x| x == "update")
    {
        check_path(path).await;
        update(&path, webhook).await;
        std::process::exit(0);
    }

    if args.iter().any(|x| x == "update-subreddit")
    {
        let index = args.iter().position(|x| x == "update-subreddit").unwrap();
        if index+1 < args.len() {
            let subreddit = args[index+1].clone();
            check_path(path).await;
            update_subreddit(&path, subreddit, webhook).await;
            std::process::exit(0);
        }

    }

}

async fn check_path(path: &Path) {

    if !path.exists() {
        match create_dir(path) {
            Ok(_) => {println!("Created new data store: {:?}", path);},
            Err(why) => {println!("Could not create new data store: {}", why); exit(1)}
        }
    }

}

async fn update(path: &Path, webhook: Option<String>) {

    let mut week_in_wild_life = WeekInWildlife::new(path);
    let f_week_in_wild_life = week_in_wild_life.get();

    let mut photos_of_the_day = PhotosOfTheDay::new(path);
    let f_photos_of_the_day = photos_of_the_day.get();

    let mut bbc_in_pictures = BBCInPictures::new(path);
    let f_bbc_in_pictures = bbc_in_pictures.get();

    let mut bbc_future = BBCFuture::new(path);
    let f_bbc_future = bbc_future.get();

    let mut nature_news = NatureNews::new(path);
    let f_nature_news = nature_news.get();

    f_week_in_wild_life.await;
    f_photos_of_the_day.await;
    f_bbc_future.await;
    f_bbc_in_pictures.await;
    f_nature_news.await;

    week_in_wild_life.commit(path, webhook.clone()).await;
    photos_of_the_day.commit(path, webhook.clone()).await;
    bbc_in_pictures.commit(path, webhook.clone()).await;
    bbc_future.commit(path, webhook.clone()).await;
    nature_news.commit(path, webhook.clone()).await;

}

async fn update_subreddit(path: &Path, subreddit: String, webhook: Option<String>) {

    let mut subreddit = Subreddit::new(path, subreddit);
    subreddit.get().await;
    let mut new_posts = Vec::new();

    for dentry in subreddit.new_entries() {
        new_posts.push(dentry.entry);
    }

    match webhook {
        Some(s) => post_summary(&s, new_posts, &format!("## Top {} articles of the last 24 hours.", subreddit.name())).await,
        None => {}
    }

    subreddit.commit(path, None).await;

}