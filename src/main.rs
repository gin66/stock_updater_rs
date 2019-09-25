use std::fs::File;
use std::io::Write;
use std::collections::HashMap;
use std::path::Path;

//use log::*;
use error_chain::*;
use chrono::NaiveDate;
use scraper::{Html, Selector};

mod ohlc;

use ohlc::OHLC;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        Reqwest(reqwest::Error);
    }

    errors { RandomResponseError(t: String) }
}


fn update_isin(isin: String) -> Result<()>  {
    let fname = format!("stock/{}/ohlc.csv",isin);
    let known_ohlc = match File::open(&fname) {
        Ok(f) => OHLC::load_file(f).expect(&format!("Read error on {}",fname)),
        _ => vec![]
    };

    let range = match known_ohlc.last() {
        Some((ref d,_)) => {
            let today = chrono::Utc::today().naive_local();
            let days = NaiveDate::signed_duration_since(today,*d).num_days();
            assert!(days >= 0);
            println!("{}",days);
            let months = days / 30 + 1;
            let months = months.min(120);
            format!("{}M",months)
        },
        _ => "120M".to_string()
    };

    let url = format!("https://www.onvista.de/aktien/kurshistorie.html?ISIN={}&RANGE={}",isin,range);
    println!("{}",url);
    let data = reqwest::get(&url)?.text()?;

    let mut all_ohlc = known_ohlc.into_iter().collect::<HashMap<_,_>>();

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

            let d_ohlc = OHLC { open, high, low, close };
            println!("{} {}", day, d_ohlc);
            all_ohlc.insert(day, d_ohlc);
        }
    }
    

    let mut all_ohlc = all_ohlc.into_iter().collect::<Vec<_>>();
    all_ohlc.sort_by_key(|e| e.0);

    if let Ok(mut f) = File::create(fname) {
        for (day,e) in all_ohlc.into_iter() {
            writeln!(f,"{} {:.5} {:.5} {:.5} {:.5}",
                     day,e.open,e.high,e.low,e.close)?;
        }
    }

    Ok(())
}


fn run() -> Result<()>  {
    println!("Hello, world!");

    let path = Path::new("stock/");
    for entry in path.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            println!("{:?}", entry.file_name());
            let isin = entry.file_name().into_string().unwrap();
            update_isin(isin)?;
        }
    }
    Ok(())
}
fn main() {
    simple_logger::init_with_level(log::Level::Warn).unwrap();

    if let Err(error) = run() {
        match *error.kind() {
            ErrorKind::Io(_) => println!("Standard IO error: {:?}", error),
            ErrorKind::Reqwest(_) => println!("Reqwest error: {:?}", error),
            _ => println!("Other error: {:?}", error),
        }
    }
}
