use std::collections::HashMap;
use std::io;
use std::io::{BufWriter, Read, Write};

use vcd::Command::{ChangeScalar, Timestamp};
use vcd::{Header, IdCode, Parser, Var, VarType};
use vcd::ScopeItem;

pub fn run<R: Read>(
    input: &mut R,
    selections: &[String],
    start_time: Option<u64>,
    end_time: Option<u64>,
    scale: Option<f32>,
) -> Result<(), String> {
    let mut parser = Parser::new(input);

    let header = parser.parse_header().map_err(|e| e.to_string())?;

    let signals = get_signal_map(&header, selections)?;

    let mut output = BufWriter::new(io::stdout());

    generate(
        &mut output,
        &mut parser,
        &signals,
        start_time,
        end_time,
        scale.unwrap_or(1.0),
    )
    .map_err(|e| e.to_string())
}

fn parse_selection(selection: &str) -> Result<(Option<&str>, &str), String> {
    if selection.is_empty() {
        return Err("empty signal selection".into());
    }

    let elements: Vec<&str> = selection.split('=').collect();

    match elements.len() {
        1 => Ok((None, elements[0])),
        2 => {
            if elements[0].is_empty() || elements[1].is_empty() {
                return Err(format!("invalid signal selection: {}", selection));
            }

            Ok((Some(elements[0]), elements[1]))
        }
        _ => Err(format!("invalid signal selection: {}", selection)),
    }
}

#[cfg(test)]
mod parse_selection_tests {
    use super::*;

    #[test]
    fn test_valid() {
        assert_eq!(parse_selection("a.b.x"), Ok((None, "a.b.x")));
    }

    #[test]
    fn test_valid_reassignment() {
        assert_eq!(parse_selection("q=a.b.x"), Ok((Some("q"), "a.b.x")));
    }

    #[test]
    fn test_invalid() {
        assert!(parse_selection("").is_err());
        assert!(parse_selection("=").is_err());
        assert!(parse_selection("q=").is_err());
        assert!(parse_selection("=a.b.x").is_err());
    }
}

fn find_vcd_var<'a>(header: &'a Header, path: &str) -> Option<&'a Var> {
    let path: Vec<&str> = path.split('.').collect();

    header.find_var(&path)
}

#[cfg(test)]
mod find_vcd_var_tests {
    use super::*;

    #[test]
    fn test_valid() {
        let mut parser = vcd::Parser::new(
            &b"
                $scope module a $end
                $scope module b $end
                $var wire 1 ! x $end
                $var wire 1 # y $end
                $var wire 8 $ z $end
                $upscope $end
                $upscope $end
                $enddefinitions $end
            "[..],
        );

        let header = parser.parse_header().unwrap();

        let vcd_var = find_vcd_var(&header, "a.b.x").unwrap();

        assert_eq!(vcd_var.reference, "x");
    }

    #[test]
    fn test_invalid() {
        let mut parser = vcd::Parser::new(
            &b"
                $scope module a $end
                $scope module b $end
                $var wire 1 ! x $end
                $var wire 1 # y $end
                $var wire 8 $ z $end
                $upscope $end
                $upscope $end
                $enddefinitions $end
            "[..],
        );

        let header = parser.parse_header().unwrap();

        assert!(find_vcd_var(&header, "a.b.c").is_none());
        assert!(find_vcd_var(&header, "a.c.z").is_none());
    }
}

fn collect_all_signals(header: &Header, out: &mut HashMap<IdCode, String>) {
    fn walk_items(items: &[ScopeItem], prefix: &str, out: &mut HashMap<IdCode, String>) {
        for item in items {
            match item {
                ScopeItem::Var(var) => {
                    if var.var_type == VarType::Wire && var.size == 1 {
                        let full_name = if prefix.is_empty() {
                            var.reference.clone()
                        } else {
                            format!("{}.{}", prefix, var.reference)
                        };

                        out.insert(var.code, full_name);
                    }
                }

                ScopeItem::Scope(scope) => {
                    let new_prefix = if prefix.is_empty() {
                        scope.identifier.clone()
                    } else {
                        format!("{}.{}", prefix, scope.identifier)
                    };

                    walk_items(&scope.children, &new_prefix, out);
                }
            }
        }
    }

    walk_items(&header.items, "", out);
}


fn get_signal_map(
    header: &Header,
    selections: &[String],
) -> Result<HashMap<IdCode, String>, String> {
    let mut signals = HashMap::new();

    if selections.is_empty() {
        collect_all_signals(header, &mut signals);
        return Ok(signals);
    }

    for selection in selections {
        let (verilog_name, vcd_path) = parse_selection(selection)?;

        let vcd_var = find_vcd_var(header, vcd_path)
            .ok_or_else(|| format!("invalid VCD path: {}", vcd_path))?;

        if vcd_var.var_type != VarType::Wire || vcd_var.size != 1 {
            return Err(format!("signal must be 1-bit wide wire: {}", vcd_path));
        }

        let verilog_name = verilog_name.unwrap_or(&vcd_var.reference);

        signals.insert(vcd_var.code, verilog_name.to_string());
    }

    Ok(signals)
}

#[cfg(test)]
mod get_signal_map_tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_valid() {
        let mut parser = vcd::Parser::new(
            &b"
                $scope module a $end
                $scope module b $end
                $var wire 1 ! x $end
                $var wire 1 # y $end
                $var wire 8 $ z $end
                $upscope $end
                $upscope $end
                $enddefinitions $end
            "[..],
        );

        let header = parser.parse_header().unwrap();

        let signals =
            get_signal_map(&header, &["q=a.b.x".to_string(), "a.b.y".to_string()]).unwrap();

        assert_eq!(signals.get(&IdCode::from_str("!").unwrap()).unwrap(), "q");
        assert_eq!(signals.get(&IdCode::from_str("#").unwrap()).unwrap(), "y");
    }

    #[test]
    fn test_invalid() {
        let mut parser = vcd::Parser::new(
            &b"
                $scope module a $end
                $scope module b $end
                $var wire 1 ! x $end
                $var wire 1 # y $end
                $var wire 8 $ z $end
                $upscope $end
                $upscope $end
                $enddefinitions $end
            "[..],
        );

        let header = parser.parse_header().unwrap();

        assert!(get_signal_map(&header, &["a.b.c".to_string()]).is_err());
        assert!(get_signal_map(&header, &["a.b.z".to_string()]).is_err());
    }
}

fn generate<R: Read, W: Write>(
    output: &mut BufWriter<W>,
    parser: &mut Parser<R>,
    signals: &HashMap<IdCode, String>,
    start_time: Option<u64>,
    end_time: Option<u64>,
    scale: f32,
) -> io::Result<()> {
    let mut time = 0;
    let mut last_time = start_time.unwrap_or(0);

    for name in signals.values() {
        // Convert hierarchical names "top.a.b.c" → "top_a_b_c"
        let safe_name = name.replace('.', "_");

        writeln!(output, "wire {};", safe_name)?;
    }
    writeln!(output)?; // blank line

    writeln!(output, "initial begin")?;
    for command_result in parser {
        let command = command_result?;

        match command {
            Timestamp(t) => time = t,
            ChangeScalar(i, v) if signals.contains_key(&i) => {
                if start_time.map_or(false, |start_time| time < start_time) {
                    continue;
                }

                if end_time.map_or(false, |end_time| time > end_time) {
                    break;
                }

                let delay = time - last_time;

                if delay > 0 {
                    let delay = delay as f32 * scale;

                    writeln!(output, "\t#{};", delay)?;
                }

                let verilog_name = signals.get(&i).unwrap();
                let safe_name = verilog_name.replace('.', "_");

                writeln!(output, "\t{} = {};", safe_name, v)?;

                last_time = time;
            }
            _ => (),
        }
    }
    writeln!(output, "end")?;

    Ok(())
}

#[cfg(test)]
mod generate_tests {
    use super::*;

    #[test]
    fn test_no_time_range() {
        let mut parser = vcd::Parser::new(
            &b"
                $scope module a $end
                $scope module b $end
                $var wire 1 ! x $end
                $var wire 1 # y $end
                $var wire 8 $ z $end
                $upscope $end
                $upscope $end
                $enddefinitions $end
                #100
                1!
                1#
                #110
                0!
                #120
                0#
                #130
                1!
                1#
            "[..],
        );

        let header = parser.parse_header().unwrap();

        let signals =
            get_signal_map(&header, &["q=a.b.x".to_string(), "a.b.y".to_string()]).unwrap();

        let mut output = BufWriter::new(Vec::new());

        assert!(generate(&mut output, &mut parser, &signals, None, None, 1.0).is_ok());

        let output = String::from_utf8(output.into_inner().unwrap()).unwrap();

        assert_eq!(
            output,
            "#100;\nq = 1;\ny = 1;\n#10;\nq = 0;\n#10;\ny = 0;\n#10;\nq = 1;\ny = 1;\n"
        );
    }

    #[test]
    fn test_with_time_range() {
        let mut parser = vcd::Parser::new(
            &b"
                $scope module a $end
                $scope module b $end
                $var wire 1 ! x $end
                $var wire 1 # y $end
                $var wire 8 $ z $end
                $upscope $end
                $upscope $end
                $enddefinitions $end
                #100
                1!
                1#
                #110
                0!
                #120
                0#
                #130
                1!
                1#
            "[..],
        );

        let header = parser.parse_header().unwrap();

        let signals =
            get_signal_map(&header, &["q=a.b.x".to_string(), "a.b.y".to_string()]).unwrap();

        let mut output = BufWriter::new(Vec::new());

        assert!(generate(
            &mut output,
            &mut parser,
            &signals,
            Some(105),
            Some(125),
            1.0
        )
        .is_ok());

        let output = String::from_utf8(output.into_inner().unwrap()).unwrap();

        assert_eq!(output, "#5;\nq = 0;\n#10;\ny = 0;\n");
    }
}
