use std::io;
use std::io::BufReader;
use std::process;
use std::str::FromStr;

use clap::{App, Arg};

fn main() {
    let (selections, start_time, end_time, scale) = parse_args();

    let mut input = BufReader::new(io::stdin());

    if let Err(e) = vcd2v::run(&mut input, &selections, start_time, end_time, scale) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn parse_args() -> (Vec<String>, Option<u64>, Option<u64>, Option<f32>) {
    let matches = App::new("vcd2v")
        .arg(
            Arg::with_name("time")
                .long("time")
                .short("t")
                .takes_value(true)
                .value_name("[START][:END]"),
        )
        .arg(
            Arg::with_name("scale")
                .long("scale")
                .short("s")
                .takes_value(true)
                .value_name("SCALE"),
        )
        .arg(Arg::with_name("selection").multiple(true).required(true))
        .get_matches();

    let selections: Vec<String> = matches
        .values_of("selection")
        .unwrap()
        .map(|v| v.to_string())
        .collect();

    let time_range = matches
        .value_of("time")
        .map(|s| parse_time_range(s))
        .transpose()
        .expect("invalid time argument")
        .unwrap_or((None, None));

    let scale = matches
        .value_of("scale")
        .map(|s| f32::from_str(s))
        .transpose()
        .expect("invalid scale argument");

    (selections, time_range.0, time_range.1, scale)
}

fn parse_time_range(value: &str) -> Result<(Option<u64>, Option<u64>), String> {
    if value.is_empty() {
        return Err("empty time range".into());
    }

    let elements: Vec<&str> = value.split(':').collect();

    let (start_time, end_time) = match elements.len() {
        1 => (Some(elements[0]), None),
        2 => (Some(elements[0]), Some(elements[1])),
        _ => return Err("invalid time range format".into()),
    };

    let parse_optional_time = |s: Option<&str>| {
        s.filter(|s| !s.is_empty())
            .map(|s| u64::from_str(s))
            .transpose()
            .map_err(|e| e.to_string())
    };

    let start_time = parse_optional_time(start_time)?;
    let end_time = parse_optional_time(end_time)?;

    if let (Some(s), Some(e)) = (start_time, end_time) {
        if s >= e {
            return Err("invalid time range: start time must be less than end time".into());
        }
    }

    Ok((start_time, end_time))
}

#[cfg(test)]
mod parse_time_range_tests {
    use super::*;

    #[test]
    fn test_start_time_only() {
        assert_eq!(parse_time_range("1"), Ok((Some(1), None)));
        assert_eq!(parse_time_range("1:"), Ok((Some(1), None)));
    }

    #[test]
    fn test_end_time_only() {
        assert_eq!(parse_time_range(":9"), Ok((None, Some(9))));
    }

    #[test]
    fn test_start_and_end_time() {
        assert_eq!(parse_time_range("1:9"), Ok((Some(1), Some(9))));
    }

    #[test]
    fn test_invalid_range() {
        assert!(parse_time_range("").is_err());
        assert!(parse_time_range("a").is_err());
        assert!(parse_time_range("1:a").is_err());
        assert!(parse_time_range("9:1").is_err());
    }
}
