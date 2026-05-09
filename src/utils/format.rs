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

    let (name_part, ext_part) = split_filename(s);
    let ext_width = ext_part.width();

    if ext_width >= available_width {
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
        return result + "...";
    }

    let name_available_width = available_width.saturating_sub(ext_width);
    if name_available_width == 0 {
        return "...".to_string() + ext_part;
    }

    let mut truncated_name = String::new();
    let mut current_width = 0;

    for c in name_part.chars() {
        let c_width = c.width().unwrap_or(1);
        if current_width + c_width > name_available_width {
            break;
        }
        truncated_name.push(c);
        current_width += c_width;
    }

    truncated_name + "..." + ext_part
}

fn split_filename(s: &str) -> (&str, &str) {
    if let Some(last_dot) = s.rfind('.') {
        if last_dot == 0 {
            return (s, "");
        }
        let (name, ext) = s.split_at(last_dot);
        (name, ext)
    } else {
        (s, "")
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_filename() {
        assert_eq!(split_filename("file.txt"), ("file", ".txt"));
        assert_eq!(split_filename("document.pdf"), ("document", ".pdf"));
        assert_eq!(split_filename("long_file_name_with_many_dots.tar.gz"), ("long_file_name_with_many_dots.tar", ".gz"));
        assert_eq!(split_filename("no_extension"), ("no_extension", ""));
        assert_eq!(split_filename(".hiddenfile"), (".hiddenfile", ""));
        assert_eq!(split_filename(""), ("", ""));
        assert_eq!(split_filename("a"), ("a", ""));
        assert_eq!(split_filename(".a"), (".a", ""));
        assert_eq!(split_filename("a."), ("a", "."));
    }

    #[test]
    fn test_truncate_string_no_extension() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("this is a very long string", 10), "this is...");
        assert_eq!(truncate_string("abcdefghijklmnop", 8), "abcde...");
    }

    #[test]
    fn test_truncate_string_with_extension() {
        assert_eq!(truncate_string("very_long_file_name_that_needs_truncating.docx", 20), "very_long_fi....docx");
        assert_eq!(truncate_string("document.pdf", 10), "doc....pdf");
        assert_eq!(truncate_string("a_very_long_filename_that_needs_to_be_truncated.rs", 30), "a_very_long_filename_tha....rs");
    }

    #[test]
    fn test_truncate_string_extension_too_long() {
        assert_eq!(truncate_string("file.verylongextension", 15), "file.verylon...");
        assert_eq!(truncate_string("file.ext", 3), "...");
    }

    #[test]
    fn test_truncate_string_exact_length() {
        assert_eq!(truncate_string("exact", 5), "exact");
        assert_eq!(truncate_string("exact.txt", 9), "exact.txt");
    }

    #[test]
    fn test_truncate_string_just_over() {
        assert_eq!(truncate_string("just_over", 8), "just_...");
        assert_eq!(truncate_string("just_over.txt", 12), "just_....txt");
    }

    #[test]
    fn test_truncate_string_unicode() {
        assert_eq!(truncate_string("文件名称很长.txt", 12), "文件....txt");
    }
}