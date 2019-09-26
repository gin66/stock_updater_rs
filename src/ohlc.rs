use std::fmt;
use std::fs::File;

use chrono::NaiveDate;
use error_chain::*;
use crate::error_def::*;

pub struct OHLC {
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
}
impl fmt::Display for OHLC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OHLC({}, {}, {}, {})",
            self.open, self.high, self.low, self.close
        )
    }
}
impl fmt::Debug for OHLC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OHLC({}, {}, {}, {})",
            self.open, self.high, self.low, self.close
        )
    }
}

impl OHLC {
    pub fn load_file(f: File) -> Result<Vec<(NaiveDate, OHLC)>> {
        let mut rdr = csv::ReaderBuilder::new().delimiter(b' ').from_reader(f);

        let mut ohlc_data = vec![];
        for result in rdr.records() {
            if let Ok(record) = result {
                let day = NaiveDate::parse_from_str(&record[0], "%Y-%m-%d")?;
                let open: f32 = record[1].parse()?;
                let high: f32 = record[2].parse()?;
                let low: f32 = record[3].parse()?;
                let close: f32 = record[4].parse()?;
                let ohlc = OHLC {
                    open,
                    high,
                    low,
                    close,
                };
                ohlc_data.push((day, ohlc));
            } else {
                bail!("Parse error: {:?}", result);
            }
        }
        Ok(ohlc_data)
    }
}
