use std::ops::Add;

use chrono::{Local, NaiveDateTime, Offset, TimeDelta};
use num_traits::cast::FromPrimitive;

/// Delta between the minimal unix and cocoa dates.
/// Roughly equals 31 years.
const UNIX_COCOA_DELTA_MILLIS: i64 = 978307200000;

// TODO: Respect the asset's timezone
pub fn parse_cocoa_timestamp(cocoa_seconds: f32) -> NaiveDateTime {
    let timestamp_millis = i64::from_f32(cocoa_seconds).unwrap() * 1000;

    let datetime = NaiveDateTime::from_timestamp_millis(timestamp_millis).unwrap();
    let cocoa_unix_delta = TimeDelta::milliseconds(UNIX_COCOA_DELTA_MILLIS);
    let utc_offset = Local::now().offset().fix();

    datetime.add(cocoa_unix_delta).add(utc_offset)
}