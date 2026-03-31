use std::process::ExitStatus;

const MAX_SUMMARY_CHARS: usize = 160;

pub fn summarize_command_output(output: &[u8]) -> String {
    let text = String::from_utf8_lossy(output);
    let last_line = text
        .lines()
        .rev()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .unwrap_or("no output");

    truncate_for_log(last_line, MAX_SUMMARY_CHARS)
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let units = ["KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = "B";
    for next_unit in units {
        value /= 1024.0;
        unit = next_unit;
        if value < 1024.0 {
            break;
        }
    }

    format!("{value:.1} {unit}")
}

pub fn exit_status_label(status: &ExitStatus) -> String {
    status
        .code()
        .map(|code| code.to_string())
        .unwrap_or_else(|| "signal".to_string())
}

fn truncate_for_log(input: &str, max_chars: usize) -> String {
    let char_count = input.chars().count();
    if char_count <= max_chars {
        return input.to_string();
    }

    let truncated = input.chars().take(max_chars).collect::<String>();
    format!("{truncated}...")
}

#[cfg(test)]
mod tests {
    use super::{format_bytes, summarize_command_output};

    #[test]
    fn summarize_command_output_uses_last_non_empty_line() {
        let output = b"warning: first\n\nerror: final line\n";
        assert_eq!(summarize_command_output(output), "error: final line");
    }

    #[test]
    fn summarize_command_output_truncates_long_lines() {
        let output = format!("noise\n{}\n", "x".repeat(220));
        let summary = summarize_command_output(output.as_bytes());
        assert!(summary.ends_with("..."));
        assert!(summary.len() <= 163);
    }

    #[test]
    fn format_bytes_uses_binary_units() {
        assert_eq!(format_bytes(3_333_824), "3.2 MiB");
    }
}
