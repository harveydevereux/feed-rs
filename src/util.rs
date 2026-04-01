use std::{fs::File, io::Read, path::Path};

use scraper::{ElementRef, Selector};

pub fn read_file_utf8(path: &Path) -> Option<String>
{
    let mut file = match File::open(path) {
        Err(why) =>
        {
            return None
        },
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) =>
        {
            None
        },
        Ok(_) => Some(s)
    }
}