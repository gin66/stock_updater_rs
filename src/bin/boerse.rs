use std::collections::HashMap;
use std::fs::File;
use std::fmt;

use log::*;
use chrono::{Datelike, NaiveDate};
use ndarray::s;
use ndarray::Array2;

struct OHLC {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    last_close: f64,
}
impl OHLC {
    fn as_f64_vec(&self) -> Vec<f64> {
        let lc = self.last_close;
        vec![   (self.open / lc - 1.0) * 20.0,
                (self.high / lc - 1.0) * 20.0,
                (self.low / lc - 1.0) * 20.0,
                (self.close / lc - 1.0) * 20.0]
    }
}
impl fmt::Debug for OHLC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OHLC({}, {}, {}, {})", self.open, self.high, self.low, self.close)
    }
}

fn as_f64_vec(day: &NaiveDate) -> Vec<f64> {
    match day.weekday() {
        chrono::Weekday::Mon => vec![1.0, 0.0, 0.0, 0.0, 0.0],
        chrono::Weekday::Tue => vec![0.0, 1.0, 0.0, 0.0, 0.0],
        chrono::Weekday::Wed => vec![0.0, 0.0, 1.0, 0.0, 0.0],
        chrono::Weekday::Thu => vec![0.0, 0.0, 0.0, 1.0, 0.0],
        chrono::Weekday::Fri => vec![0.0, 0.0, 0.0, 0.0, 1.0],
        chrono::Weekday::Sat => vec![0.0, 0.0, 0.0, 0.0, 0.0],
        chrono::Weekday::Sun => vec![0.0, 0.0, 0.0, 0.0, 0.0],
    }
}

fn load_file(fname: &std::ffi::OsString) -> Result<Vec<(NaiveDate, OHLC)>, std::io::Error> {
    let f = File::open(fname)?;
    let mut rdr = csv::ReaderBuilder::new().delimiter(b' ').from_reader(f);

    let mut ohlc_data = vec![];
    let mut opt_last_close = None;
    for result in rdr.records() {
        if let Ok(record) = result {
            let day = NaiveDate::parse_from_str(&record[0], "%Y-%m-%d").unwrap();
            let open: f64 = record[1].parse().unwrap();
            let high: f64 = record[2].parse().unwrap();
            let low: f64 = record[3].parse().unwrap();
            let close: f64 = record[4].parse().unwrap();
            if let Some(last_close) = opt_last_close {
                let ohlc = OHLC {
                    open,
                    high,
                    low,
                    close,
                    last_close,
                };
                ohlc_data.push((day, ohlc));
            }
            opt_last_close = Some(close);
            //println!("{:?} {} {} {} {}", day, open, high, low, close);
        }
        else {
            warn!("Parse error: {:?}",result);
        }
    }
    Ok(ohlc_data)
}

fn main() -> Result<(), std::io::Error> {
    simple_logger::init().unwrap();

    let home_path = dirs::home_dir().unwrap();

    let mut fname = home_path.clone();
    fname.push("data");
    fname.push("stock");
    fname.push("DE0008469008");
    fname.push("ohlc.csv");
    let dax = load_file(&fname.into_os_string()).unwrap();

    let mut fname = home_path.clone();
    fname.push("data");
    fname.push("stock");
    fname.push("US2605661048");
    fname.push("ohlc.csv");
    let dow = load_file(&fname.into_os_string()).unwrap();

    println!("dax=#{} dow=#{}", dax.len(), dow.len());

    println!("Last= {:?}",dax.last());
    println!("Last= {:?}",dow.last());

    let mut combined = HashMap::new();
    for (day,ohlc) in dax.into_iter() {
        let entry = combined.entry(day);
        let entry = entry.or_insert(vec![None,None]);
        entry[0] = Some(ohlc);
    }
    for (day,ohlc) in dow.into_iter() {
        let entry = combined.entry(day);
        let entry = entry.or_insert(vec![None,None]);
        entry[1] = Some(ohlc);
    }

    combined.retain(|_,entry| {
        entry.iter().filter(|e| e.is_some()).count() == entry.len()
    });

    let mut combined = combined.into_iter().collect::<Vec<_>>();
    combined.sort_by_key(|(day,_)| *day);

    println!("combined=#{}",combined.len());
    println!("Last= {:?}",combined.last());

    let mut rows = 0;
    let mut data = vec![];
    for (day,entries) in &combined {
        rows += 1;
        data.extend(as_f64_vec(day));
        for ohlc in entries {
            data.extend(ohlc.as_ref().unwrap().as_f64_vec());
        }
    }

    let inputs = data.len() / rows;
    let data = Array2::<f64>::from_shape_vec((rows, inputs), data).unwrap();

    let mut som = rusticsom::SOM::create(15, 15, inputs, true, None, None, None, None);
    som.train_random(data.clone(), 2000);

    let mut winners = vec![];
    let mut avg = HashMap::new();
    for v in data.outer_iter() {
        let winner = som.winner(v.to_owned());
        //println!("{:?} {:?}",v,winner);
        winners.push(winner);

        let e0 = avg.entry(winner);
        let e0 = e0.or_insert(vec![]);
        e0.push(v);
    }

    println!("{:?}", som.activation_response());

    let mut markov = HashMap::new();
    for i in 1..winners.len() {
        let w0 = &winners[i - 1];
        let w1 = &winners[i];

        let e0 = markov.entry(w0);
        let e0 = e0.or_insert(HashMap::new());
        let e1 = e0.entry(w1);
        let e1 = e1.or_insert(0);
        *e1 += 1;
    }

    let w_last = winners.last().unwrap();
    if let Some(hm) = markov.get(w_last) {
        let mut est = ndarray::Array1::<f64>::zeros(inputs);
        let mut est_cnt = 0;
        for (wp, cnt) in hm.iter() {
            print!("{:?} {}", wp, cnt);
            if let Some(avg_vec) = avg.get(wp) {
                let mut av = ndarray::Array1::<f64>::zeros(inputs);
                for ax in avg_vec.iter() {
                    av = av + ax;
                }
                av /= avg_vec.len() as f64;
                let mut avx = av.slice_mut(s![5..]);
                avx /= 20.0;
                avx += 1.0;
                //avx *= last_close;
                println!(" => {}", av);
                if *cnt > 1 {
                    av *= *cnt as f64;
                    est = est + av;
                    est_cnt += *cnt;
                }
            } else {
                println!(" => None");
            }
        }
        println!("estimate {}", est / est_cnt as f64);
    }

    //if let Some(tv) = opt_tv {
    //    let winner = som.winner(tv);
    //    println!("From last: {:?}", winner);
    //}

    Ok(())
}
