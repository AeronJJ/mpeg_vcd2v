// use std::io::BufReader;
use std::fs::File;
use std::process;
use std::str::FromStr;

use clap::{App, Arg};

use std::io::{self, BufRead, BufReader};

fn main() {
    let (selections, start_time, end_time, scale, file, signal_map_file) = parse_args();

    // let signal_map_entries: Vec<String> = if let Some(path) = signal_map_path {
    // 	read_signal_map(path).expect("Failed to read signal map file")
    // } else {
    // 	Vec::new()
    // };

    let mut combined: Vec<String> = Vec::new();

    // selections
    // if let Some(values) = selections {
    // 	combined.extend(values.map(|v| v.to_string()));
    // }
    combined.extend(selections.iter().cloned()); 

    // signal map (vec<String> form)
    // let signal_map_entries: Vec<String> = if let Some(path) = signal_map_file {
    // 	read_signal_map(path).expect("Failed to read signal map file")
    // };
    if let Some(path) = signal_map_file {
	let smap = read_signal_map(path).expect("Failed to read signal map file");  
	combined.extend(smap); // Doesn't protect against duplicates
    }

    // let mut input = BufReader::new(io::stdin());
    let file = File::open(&file)
	.expect("failed to open input VCD file");
    
    let mut input = BufReader::new(file);


    if let Err(e) = vcd2v::run(&mut input, &combined, start_time, end_time, scale) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn parse_args() -> (Vec<String>, Option<u64>, Option<u64>, Option<f32>, String, Option<String>) {
    let matches = App::new("vcd2v")
	.arg(
            Arg::with_name("input")
		.long("input")
		.short("i")
		.takes_value(true)
		.value_name("INPUT")
                .required(true)
                .help("Input VCD file to parse")
        )
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
	.arg(
	    Arg::with_name("signal_map")
		.long("signal_map")
		.short("m")
		.takes_value(true)
		.value_name("SIGNAL_MAP")
		.help("Signal map file to map VCD signals to testbench signals")
	)
        .arg(Arg::with_name("selection").multiple(true))
        .get_matches();

    let selections: Vec<String> = matches
        .values_of("selection")
        .unwrap_or_default()
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

    let input_file = matches
        .value_of("input")
        .unwrap() // safe because required
        .to_string();

    // let signal_map_file = matches
    // 	.value_of("signal_map")
    // 	.map(|sm| String::from_str(sm))
    // 	.expect("Invalid signal map argument")
    // 	.unwrap_or(None);
    let signal_map_file: Option<String> = matches
	.values_of("signal_map")
	.and_then(|mut v| v.next().map(|s| s.to_string()));



    (selections, time_range.0, time_range.1, scale, input_file, signal_map_file)
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

fn read_signal_map(path: String) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut results = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        // Skip empty or comment lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Validate format: must contain '='
        if !trimmed.contains('=') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid line (missing '='): {}", trimmed),
            ));
        }

        // Store as a String, same as selection argument
        results.push(trimmed.to_string());
    }

    Ok(results)
}
