use std::io::{self, IsTerminal};

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Table,
    Csv,
}

impl OutputFormat {
    pub fn from_str_or_auto(s: Option<&str>) -> Self {
        match s {
            Some("json") => Self::Json,
            Some("table") => Self::Table,
            Some("csv") => Self::Csv,
            _ => {
                if io::stdout().is_terminal() {
                    Self::Table
                } else {
                    Self::Json
                }
            }
        }
    }
}

/// Print a single serializable value.
pub fn print_value<T: Serialize>(value: &T, format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(value)?;
            println!("{json}");
        }
        OutputFormat::Table | OutputFormat::Csv => {
            // For single values, fall back to JSON
            let json = serde_json::to_string_pretty(value)?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Print a list of serializable items.
pub fn print_list<T: Serialize>(items: &[T], format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(items)?;
            println!("{json}");
        }
        OutputFormat::Table => {
            // Convert to JSON Value array, then render as table
            let values: Vec<serde_json::Value> = items
                .iter()
                .map(serde_json::to_value)
                .collect::<Result<_, _>>()?;
            print_value_table(&values);
        }
        OutputFormat::Csv => {
            print_csv(items)?;
        }
    }
    Ok(())
}

/// Render a JSON Value array as a table using tabled.
fn print_value_table(values: &[serde_json::Value]) {
    if values.is_empty() {
        println!("(no results)");
        return;
    }

    // Extract column headers from the first object
    let headers: Vec<String> = if let serde_json::Value::Object(map) = &values[0] {
        map.keys().cloned().collect()
    } else {
        // Not objects, just print as JSON lines
        for v in values {
            println!("{v}");
        }
        return;
    };

    let mut builder = tabled::builder::Builder::new();
    builder.push_record(&headers);

    for value in values {
        if let serde_json::Value::Object(map) = value {
            let row: Vec<String> = headers.iter().map(|h| format_cell(map.get(h))).collect();
            builder.push_record(row);
        }
    }

    let table = builder
        .build()
        .with(tabled::settings::Style::rounded())
        .to_string();
    println!("{table}");
}

fn format_cell(value: Option<&serde_json::Value>) -> String {
    match value {
        None | Some(serde_json::Value::Null) => String::new(),
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(serde_json::Value::Bool(b)) => b.to_string(),
        Some(v) => {
            // For nested objects/arrays, compact JSON
            let s = serde_json::to_string(v).unwrap_or_default();
            if s.len() > 60 {
                format!("{}...", &s[..s.floor_char_boundary(57)])
            } else {
                s
            }
        }
    }
}

/// Write items as CSV to stdout.
fn print_csv<T: Serialize>(items: &[T]) -> anyhow::Result<()> {
    if items.is_empty() {
        return Ok(());
    }

    let mut wtr = csv::Writer::from_writer(io::stdout().lock());
    for item in items {
        wtr.serialize(item)?;
    }
    wtr.flush()?;
    Ok(())
}

/// Print a simple scalar (like wallet balance).
pub fn print_scalar<T: std::fmt::Display>(value: T, label: &str, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::json!({ label: value.to_string() }));
        }
        OutputFormat::Table | OutputFormat::Csv => {
            println!("{label}: {value}");
        }
    }
}
