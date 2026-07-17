use std::{
    str::FromStr,
    fmt::Display,
    path,
};

use netcdf::AttributeValue;
use chrono::{
    DateTime,
    Utc,
    Duration,
    ParseError,
};


/// This struct represents the time unit of one hostrada file, e.g. 2005-01 respectively.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)] // funktioniert, weil es in deklarationsreihenfolge verglichen wird -> erst check ob jahr1>jahr2 und dann ob monat1>monat2
pub struct YearMonth {
    pub year: u32, 
    pub month: u8,
}

impl YearMonth {

    pub fn new(year: u32, month: u8) -> Result<Self, String> {
        if year > 9999 {
            Err("Year must be between 1 and 9999".to_owned())
        } else if !(1..=12).contains(&month) {
            Err(format!("Month must be between 0 and 12, is {month}"))
        } else {
            Ok( Self { year, month } )
        }
    }

    /// Take a self value and increase it by one month, consuming the old self value.
    pub fn next(self) -> Self {
        if self.month == 12 {
            Self {
                year: self.year +1,
                month: 1,
            }
        } else {
            Self {
                year: self.year,
                month: self.month +1,
            }
        }
    }

    /// Returns all YearMonth objects between 2 YearMonth objects, including self, excluding other (e.g. 2005-01, 2005-02, 2005-03, ...)
    pub fn range_to(&self, other: &YearMonth) -> Vec<YearMonth> {
        let mut current = *self; // works because YearMonth implements Clone
        let mut result: Vec<YearMonth> = Vec::new();

        while current < *other {
            result.push(current);

            current = current.next();
        } 
        
        result    
    }

    /// Returns the number of days in the self.month
    pub fn days_in_month(&self) -> u8 {
        match self.month {
            1|3|5|7|8|10|12 => 31,
            4|6|9|11 => 30,
            2 => if is_leap_year(self.year) {
                29
            } else {
                28
            },
            _ => unreachable!(),
        }
    }
}

impl FromStr for YearMonth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (year, month) = s.split_once("-").ok_or(&format!("{} is not a valid format! Must be YYYY-MM", s))?;

        let year: u32 = year.parse().map_err(|err| format!("Failed to parse {year}: {err}"))?;
        let month: u8 = month.parse().map_err(|err| format!("Failed to parse {month}: {err}"))?;

        if !(1..=12).contains(&month) {
            return Err(format!("Month must be between 0 and 12, is {month}"));
        }

        Ok( Self { year, month })
    }
}

impl Display for YearMonth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}-{:02}", self.year, self.month)
    }
}



pub fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Parses a days since rfc3339-origin input to a DateTime<Utc> object
pub fn parse_time(since: &str, days: f64) -> Result<DateTime<Utc>, ParseError> {
    let origin = DateTime::parse_from_rfc3339(since)?.with_timezone(&Utc);

    let datetime = origin + Duration::seconds((days * 86400.0) as i64);

    // println!("Parsed {days} to {datetime}");

    Ok(datetime)
}

/// Formats durations to human readable formats, e.g. 4h 33m 2s
pub fn readable_dur(duration: std::time::Duration) -> String {
    let total_secs = duration.as_secs_f32();

    if total_secs < 60.0 {
        return format!("{:.02}s", total_secs);
    }

    let total_secs = total_secs.round() as i32;
    let total_mins = total_secs / 60;
    let final_secs = total_secs % 60;

    if total_mins < 60 {
        return format!("{}m {}s", total_mins, final_secs);
    }

    let total_hours = total_mins / 60;
    let final_mins = total_mins % 60;

    format!("{}h {}m {}s", total_hours, final_mins, final_secs)
}

/// time unit of the file without building a whole HostradaDataset object
pub fn fast_origin(ref_file: path::PathBuf) -> Result<String, Box<dyn std::error::Error>> {

    let file = netcdf::open(ref_file)?;
    
    let attr_val = file
        .variable("time")
        .ok_or("Could not get time variable")?
        .attribute("units")
        .ok_or("Time variable does not have a units attribute")?
        .value()?;

    if let AttributeValue::Str(value) = attr_val {
        return Ok(value.split_whitespace().last().ok_or("time unit does not match the expected format")?.to_owned());
    }
    Err("Failed to get origin".into())
}