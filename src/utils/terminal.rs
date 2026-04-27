pub fn get_terminal_width() -> usize {
    crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(120)
}

pub fn make_separator(widths: &[usize]) -> String {
    let mut result = String::new();
    for &w in widths {
        result.push('+');
        result.push_str(&"-".repeat(w + 2));
    }
    result.push('+');
    result
}

pub fn calculate_column_widths(term_width: usize, show_tags: bool) -> (usize, usize, usize, usize, usize) {
    const MIN_NAME: usize = 10;
    const MIN_DATE: usize = 10;
    const MIN_SIZE: usize = 6;
    const MIN_TAGS: usize = 5;

    let border_chars = if show_tags { 16 } else { 13 };
    let content_total = term_width.saturating_sub(border_chars);
    let min_total = MIN_NAME + MIN_DATE * 2 + MIN_SIZE + if show_tags { MIN_TAGS } else { 0 };

    if content_total < min_total {
        let tags_w = if show_tags { MIN_TAGS } else { 0 };
        return (MIN_NAME, MIN_DATE, MIN_DATE, MIN_SIZE, tags_w);
    }

    if show_tags {
        const TOTAL_PARTS: usize = 118;
        let name = (content_total * 41 / TOTAL_PARTS).max(MIN_NAME);
        let created = (content_total * 21 / TOTAL_PARTS).max(MIN_DATE);
        let modified = (content_total * 21 / TOTAL_PARTS).max(MIN_DATE);
        let size = (content_total * 12 / TOTAL_PARTS).max(MIN_SIZE);
        let tags = (content_total * 23 / TOTAL_PARTS).max(MIN_TAGS);

        let total = name + created + modified + size + tags;
        if total > content_total {
            let excess = total - content_total;
            let name = name.saturating_sub(excess).max(MIN_NAME);
            (name, created, modified, size, tags)
        } else {
            (name, created, modified, size, tags)
        }
    } else {
        const TOTAL_PARTS: usize = 95;
        let name = (content_total * 41 / TOTAL_PARTS).max(MIN_NAME);
        let created = (content_total * 21 / TOTAL_PARTS).max(MIN_DATE);
        let modified = (content_total * 21 / TOTAL_PARTS).max(MIN_DATE);
        let size = (content_total * 12 / TOTAL_PARTS).max(MIN_SIZE);

        let total = name + created + modified + size;
        if total > content_total {
            let excess = total - content_total;
            let name = name.saturating_sub(excess).max(MIN_NAME);
            (name, created, modified, size, 0)
        } else {
            (name, created, modified, size, 0)
        }
    }
}