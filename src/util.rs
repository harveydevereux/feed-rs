use std::{fs::File, io::Read, path::Path};

pub fn read_file_utf8(path: &Path) -> Option<String> {
    let mut file = match File::open(path) {
        Err(_) => return None,
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(_) => None,
        Ok(_) => Some(s),
    }
}