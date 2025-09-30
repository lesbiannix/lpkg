use console::style;
use md5;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

pub fn get_md5sums() -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let res = client
        .get("https://www.linuxfromscratch.org/~thomas/multilib-m32/md5sums")
        .send()?
        .text()?;
    Ok(res)
}


