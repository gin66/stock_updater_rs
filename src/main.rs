use std::fs::File;
use std::io::BufReader;
use std::io::Read;

use chrono::NaiveDate;
use curl::easy::Easy;
use scraper::{Html, Selector};

fn main() {
    println!("Hello, world!");
    // Write the contents of rust-lang.org to stdout
        let (tx,rx) = std::sync::mpsc::channel();
    if true {

        let mut easy = Easy::new();
        easy.url("https://www.onvista.de/aktien/kurshistorie.html?ISIN=US2605661048&RANGE=120M").unwrap();
        easy.write_function(move |part| {let pl = part.len();tx.send(part.to_vec()).unwrap();Ok(pl)})
            .unwrap();
        easy.perform().unwrap();
    }

    let mut data = String::new();
    for chunk in rx {
        let chunk_str = String::from_utf8(chunk).unwrap();
        //println!("{:?}",chunk_str);
        data.push_str(&chunk_str);
    }

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

                println!("{} {} {} {} {}", day, open, low, high, close);
            }
        }
    }
}