use std::ops::Add;

use chrono::{DateTime, Local, NaiveDateTime, Offset, TimeDelta};
use num_traits::cast::FromPrimitive;

/// Delta between the minimal unix and cocoa dates.
/// Roughly equals 31 years.
const UNIX_COCOA_DELTA_MILLIS: i64 = 978307200000;

// TODO: Respect the asset's timezone
pub fn parse_cocoa_timestamp(cocoa_seconds: f32) -> Result<NaiveDateTime, String> {
    let timestamp_secs = i64::from_f32(cocoa_seconds)
        .ok_or("Could not convert timestamp to i64")?;

    let timestamp_millis = timestamp_secs * 1000;

    let datetime = DateTime::from_timestamp_millis(timestamp_millis)
        .ok_or("Could not convert timestamp to NaiveDateTime")?;

    let cocoa_unix_delta = TimeDelta::milliseconds(UNIX_COCOA_DELTA_MILLIS);
    let utc_offset = Local::now().offset().fix();

    Ok(datetime.add(cocoa_unix_delta).add(utc_offset).naive_local())
}