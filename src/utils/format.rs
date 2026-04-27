use std::time::{SystemTime, UNIX_EPOCH};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:>5.1} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:>5.1} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:>5.1} KB", size as f64 / KB as f64)
    } else {
        format!("{:>6} B", size)
    }
}

pub fn format_time_absolute(time: SystemTime) -> String {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let total_secs = duration.as_secs();
            let mut days = total_secs / 86400;
            let secs_in_day = total_secs % 86400;
            let hours = secs_in_day / 3600;
            let mins = (secs_in_day % 3600) / 60;
            let secs = secs_in_day % 60;

            let mut year = 1970;
            while days >= if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                366
            } else {
                365
            } {
                days -= if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                    366
                } else {
                    365
                };
                year += 1;
            }

            let mut month = 1;
            let month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
            for &md in month_days.iter() {
                let adjust = if month == 2 && (year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)) {
                    1
                } else {
                    0
                };
                if days < md + adjust {
                    break;
                }
                days -= md + adjust;
                month += 1;
            }

            let day = days + 1;

            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                year, month, day, hours, mins, secs
            )
        }
        Err(_) => "                   N/A".to_string(),
    }
}

pub fn truncate_string(s: &str, max_width: usize) -> String {
    let width = s.width();
    if width <= max_width {
        return s.to_string();
    }

    let available_width = max_width.saturating_sub(3);
    if available_width == 0 {
        return "...".to_string();
    }

    let mut result = String::new();
    let mut current_width = 0;

    for c in s.chars() {
        let c_width = c.width().unwrap_or(1);
        if current_width + c_width > available_width {
            break;
        }
        result.push(c);
        current_width += c_width;
    }

    result + "..."
}

pub fn center_text(text: &str, width: usize) -> String {
    let text_width = text.width();
    if text_width >= width {
        return truncate_string(text, width);
    }
    let left = (width - text_width) / 2;
    let right = width - text_width - left;
    format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
}

pub fn pad_to_width(s: &str, width: usize) -> String {
    let s_width = s.width();
    if s_width >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s_width))
    }
}