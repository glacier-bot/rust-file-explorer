//! 命令渲染模块
//! 提供 ls 等命令的输出渲染功能

use crate::models::FileInfo;
use crate::utils::format::truncate_string;
use crate::utils::terminal::{calculate_column_widths, make_separator};
use colored::Colorize;

/// 渲染长格式列表
pub fn render_long_format(
    output: &mut String,
    all_items: &[FileInfo],
    show_tags: bool,
) {
    let term_width = crate::utils::terminal::get_terminal_width();
    let (name_width, created_width, modified_width, size_width, tags_width) =
        calculate_column_widths(term_width, show_tags);
    let truncate_name_width = name_width.saturating_sub(4);

    if show_tags {
        let widths = [3, name_width, created_width, modified_width, size_width, tags_width];
        let separator = make_separator(&widths).bright_black();

        output.push_str(&format!("{}\n", separator));
        output.push_str(&format!(
            "| {:^3} | {} | {} | {} | {} | {} |\n",
            "#".bright_white().bold(),
            crate::utils::format::center_text("Name", name_width)
                .bright_white()
                .bold(),
            crate::utils::format::center_text("Created Date", created_width)
                .bright_white()
                .bold(),
            crate::utils::format::center_text("Modified Date", modified_width)
                .bright_white()
                .bold(),
            crate::utils::format::center_text("Size", size_width)
                .bright_white()
                .bold(),
            crate::utils::format::center_text("Tags", tags_width)
                .bright_white()
                .bold(),
        ));
        output.push_str(&format!("{}\n", separator));

        for (idx, item) in all_items.iter().enumerate() {
            let line_num = idx + 1;
            let created_str = item
                .created
                .map_or_else(|| "N/A".to_string(), crate::utils::format::format_time_absolute);
            let modified_str = crate::utils::format::format_time_absolute(item.modified);
            let tags_str = if item.tags.is_empty() {
                String::new()
            } else {
                item.tags.join(", ")
            };

            let display_name = truncate_string(&item.name, truncate_name_width);
            let display_text = format!("{}  {}", item.icon, display_name);
            let display_text_width = unicode_width::UnicodeWidthStr::width(&*display_text);
            let padding = if display_text_width < name_width {
                " ".repeat(name_width - display_text_width)
            } else {
                String::new()
            };
            let padded_name = format!("{}{}", display_text, padding);
            let padded_tags = crate::utils::format::pad_to_width(
                &truncate_string(&tags_str, tags_width),
                tags_width,
            );

            output.push_str(&format!(
                "| {:3} | {} | {} | {} | {} | {} |\n",
                line_num,
                padded_name.color(item.color).bold(),
                crate::utils::format::pad_to_width(
                    &truncate_string(&created_str, created_width),
                    created_width
                )
                .bright_cyan(),
                crate::utils::format::pad_to_width(
                    &truncate_string(&modified_str, modified_width),
                    modified_width
                )
                .bright_magenta(),
                crate::utils::format::pad_to_width(
                    &crate::utils::format::format_size(item.size),
                    size_width
                )
                .bright_yellow()
                .bold(),
                padded_tags.bright_yellow()
            ));
        }

        output.push_str(&format!("{}\n", separator));
    } else {
        let widths = [3, name_width, created_width, modified_width, size_width];
        let separator = make_separator(&widths).bright_black();

        output.push_str(&format!("{}\n", separator));
        output.push_str(&format!(
            "| {:^3} | {} | {} | {} | {} |\n",
            "#".bright_white().bold(),
            crate::utils::format::center_text("Name", name_width)
                .bright_white()
                .bold(),
            crate::utils::format::center_text("Created Date", created_width)
                .bright_white()
                .bold(),
            crate::utils::format::center_text("Modified Date", modified_width)
                .bright_white()
                .bold(),
            crate::utils::format::center_text("Size", size_width)
                .bright_white()
                .bold(),
        ));
        output.push_str(&format!("{}\n", separator));

        for (idx, item) in all_items.iter().enumerate() {
            let line_num = idx + 1;
            let created_str = item
                .created
                .map_or_else(|| "N/A".to_string(), crate::utils::format::format_time_absolute);
            let modified_str = crate::utils::format::format_time_absolute(item.modified);

            let display_name = truncate_string(&item.name, truncate_name_width);
            let display_text = format!("{}  {}", item.icon, display_name);
            let display_text_width = unicode_width::UnicodeWidthStr::width(&*display_text);
            let padding = if display_text_width < name_width {
                " ".repeat(name_width - display_text_width)
            } else {
                String::new()
            };
            let padded_name = format!("{}{}", display_text, padding);

            output.push_str(&format!(
                "| {:3} | {} | {} | {} | {} |\n",
                line_num,
                padded_name.color(item.color).bold(),
                crate::utils::format::pad_to_width(
                    &truncate_string(&created_str, created_width),
                    created_width
                )
                .bright_cyan(),
                crate::utils::format::pad_to_width(
                    &truncate_string(&modified_str, modified_width),
                    modified_width
                )
                .bright_magenta(),
                crate::utils::format::pad_to_width(
                    &crate::utils::format::format_size(item.size),
                    size_width
                )
                .bright_yellow()
                .bold()
            ));
        }

        output.push_str(&format!("{}\n", separator));
    }
}

/// 渲染短格式列表
pub fn render_short_format(output: &mut String, all_items: &[FileInfo], show_tags: bool) {
    for (idx, item) in all_items.iter().enumerate() {
        let line_num = idx + 1;
        let display_name = truncate_string(&item.name, 50);
        if show_tags && !item.tags.is_empty() {
            let tags_str = format!(" [{}]", item.tags.join(", "));
            output.push_str(&format!(
                "{:3}. {} {}{}\n",
                line_num,
                item.icon,
                display_name.color(item.color).bold(),
                tags_str.bright_yellow()
            ));
        } else {
            output.push_str(&format!(
                "{:3}. {} {}\n",
                line_num,
                item.icon,
                display_name.color(item.color).bold()
            ));
        }
    }
}
