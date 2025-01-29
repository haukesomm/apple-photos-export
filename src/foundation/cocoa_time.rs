use std::ops::Add;
use chrono::{DateTime, Local, Offset, TimeDelta};


/// Delta in seconds between two Unix and Cocoa dates.
/// Roughly equals 31 years.
const UNIX_COCOA_DELTA_MILLIS: i64 = 978307200000;


/// Trait to parse a Cocoa timestamp into a `NaiveDateTime`.
pub trait ParseCocoaTimestamp {
    
    fn from_cocoa_timestamp(cocoa_timestamp: f32) -> Result<Self, String>
    where
        Self: Sized;
}

impl ParseCocoaTimestamp for chrono::NaiveDateTime {
    
    fn from_cocoa_timestamp(timestamp: f32) -> Result<Self, String> {
        let timestamp_secs = {
            let f = (timestamp as f64) / 100_000.0;
            (f as i64) * 100_000
        };

        let timestamp_millis = timestamp_secs * 1000;

        let datetime = DateTime::from_timestamp_millis(timestamp_millis)
            .ok_or("Could not convert timestamp to NaiveDateTime")?;

        let cocoa_unix_delta = TimeDelta::milliseconds(UNIX_COCOA_DELTA_MILLIS);
        let utc_offset = Local::now().offset().fix();

        Ok(datetime.add(cocoa_unix_delta).add(utc_offset).naive_local())
    }
}
