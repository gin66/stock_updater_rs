use std::fs::File;
use std::io::BufReader;
use std::io::Read;

use error_chain::*;
use chrono::NaiveDate;
use scraper::{Html, Selector};

mod ohlc;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        Reqwest(reqwest::Error);
    }

    errors { RandomResponseError(t: String) }
}


fn run() -> Result<()>  {
    println!("Hello, world!");

    let data = reqwest::get("https://www.onvista.de/aktien/kurshistorie.html?ISIN=US2605661048&RANGE=120M")?.text()?;
    println!("{}",data);

    if false {
        let file = File::open("../dow").unwrap();
        let mut file = BufReader::new(file);

        let mut doc = String::new();
        file.read_to_string(&mut doc).unwrap();
    }

    if true {
        let doc = data;
        let selector = Selector::parse("tr").unwrap();
        let doc = Html::parse_document(&doc);
        for line in doc.select(&selector) {
            if line.value().classes().count() == 1 {
                let mut fields = line.text();
                let day = NaiveDate::parse_from_str(fields.next().unwrap(), "%d.%m.%y").unwrap();
                let open: f32 = fields
                    .next()
                    .unwrap()
                    .replace(".", "")
                    .replace(',', ".")
                    .parse()
                    .unwrap();
                let low: f32 = fields
                    .next()
                    .unwrap()
                    .replace(".", "")
                    .replace(',', ".")
                    .parse()
                    .unwrap();
                let high: f32 = fields
                    .next()
                    .unwrap()
                    .replace(".", "")
                    .replace(',', ".")
                    .parse()
                    .unwrap();
                let close: f32 = fields
                    .next()
                    .unwrap()
                    .replace(".", "")
                    .replace(',', ".")
                    .parse()
                    .unwrap();

                let d_ohlc = ohlc::OHLC { open, high, low, close };
                println!("{} {}", day, d_ohlc);
            }
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        match *error.kind() {
            ErrorKind::Io(_) => println!("Standard IO error: {:?}", error),
            ErrorKind::Reqwest(_) => println!("Reqwest error: {:?}", error),
            _ => println!("Other error: {:?}", error),
        }
    }
}
