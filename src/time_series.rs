// The point of the cache is to provide a subset of data from a given window of time.  The cache should be:
// * Fast on searching for specific indexes
// * Fast on write, need to add data quickly
// * Dump all relevant data from the time window
// * Periodically remove "old" data
// * Prefer timeliness of read to accuracy of the "time window"

use super::location::Location;

use std::sync::Arc;

use datetime::{DatePiece, Duration, LocalDateTime, TimePiece};

pub struct LocationTimeSeries {
    data: Vec<Vec<Location>>
}

impl LocationTimeSeries {
    pub fn new() -> LocationTimeSeries {
        let init_time = LocalDateTime::now();

        LocationTimeSeries {
            data: vec![]
        }
    }

    fn get_path(&self, datetime: LocalDateTime) -> Arc<str> {
        // Creating a path to determine location for storing location data to 
        // objectstore.  This will use something like the following as the 
        // directory structure:
        //    YYYY/MM/DD/hh/mm/ss/mmm/iii/nnn
        
        // This allows us to easily parse all files in a directory that is 
        // older than some number of seconds or milliseconds.
        format!("{:?}/{:?}/{:?}/{:?}/{:?}/{:?}/{:?}", datetime.year(), datetime.month(), datetime.day(), datetime.hour(), datetime.minute(), datetime.second(), datetime.millisecond()).into()
    }

    pub fn get(&self, start: LocalDateTime) {
        
    }
}