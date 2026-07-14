//! Output formatting for CLI commands.

use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Json,
}

/// Print a success message in human format
pub fn print_success(msg: &str) {
    println!("\x1b[32m✓\x1b[0m {}", msg);
}

/// Print an error message in human format
pub fn print_error(msg: &str) {
    eprintln!("\x1b[31m✗\x1b[0m {}", msg);
}

/// Print a warning message
pub fn print_warning(msg: &str) {
    eprintln!("\x1b[33m⚠\x1b[0m {}", msg);
}

/// Print an info message
pub fn print_info(msg: &str) {
    println!("\x1b[34mℹ\x1b[0m {}", msg);
}

/// Print JSON output
pub fn print_json(value: &Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}

/// Print JSON error
pub fn print_error_json(error: &str) {
    let value = serde_json::json!({ "error": error });
    print_json(&value);
}

/// Print a progress message
pub fn print_progress(msg: &str) {
    println!("  {}", msg);
}

/// Print a key-value pair
pub fn print_kv(key: &str, value: &str) {
    println!("  \x1b[1m{}\x1b[0m: {}", key, value);
}
