use chrono::Utc;

/// Replace placeholders in compiled text with actual values.
/// Scrivener-compatible placeholder syntax: <$placeholder>
pub fn replace_placeholders(text: &str, context: &PlaceholderContext) -> String {
    let mut result = text.to_string();

    // Date/time placeholders
    let now = Utc::now();
    result = result.replace("<$date>", &now.format("%Y-%m-%d").to_string());
    result = result.replace("<$longdate>", &now.format("%B %d, %Y").to_string());
    result = result.replace("<$shortdate>", &now.format("%m/%d/%y").to_string());
    result = result.replace("<$time>", &now.format("%H:%M").to_string());
    result = result.replace("<$year>", &now.format("%Y").to_string());
    result = result.replace("<$month>", &now.format("%B").to_string());
    result = result.replace("<$day>", &now.format("%d").to_string());

    // Project placeholders
    result = result.replace("<$projecttitle>", &context.project_title);
    result = result.replace("<$author>", &context.author);
    result = result.replace("<$surname>", &context.surname());
    result = result.replace("<$forename>", &context.forename());

    // Statistics placeholders
    result = result.replace("<$wc>", &context.word_count.to_string());
    result = result.replace("<$wordcount>", &context.word_count.to_string());
    result = result.replace("<$cc>", &context.char_count.to_string());
    result = result.replace("<$charcount>", &context.char_count.to_string());
    result = result.replace("<$pagecount>", &context.page_count.to_string());

    // Auto-numbering (chapter/section numbers)
    // These are replaced with incrementing counters
    let mut chapter_num = 0;
    let mut section_num = 0;
    let mut figure_num = 0;

    // Process line by line for auto-numbering
    let lines: Vec<String> = result.lines().map(|l| {
        let mut line = l.to_string();
        if line.contains("<$n>") {
            chapter_num += 1;
            section_num = 0;
            line = line.replace("<$n>", &chapter_num.to_string());
        }
        if line.contains("<$N>") {
            chapter_num += 1;
            section_num = 0;
            line = line.replace("<$N>", &to_roman(chapter_num));
        }
        if line.contains("<$W>") {
            chapter_num += 1;
            section_num = 0;
            line = line.replace("<$W>", &to_word(chapter_num));
        }
        if line.contains("<$sn>") {
            section_num += 1;
            line = line.replace("<$sn>", &format!("{}.{}", chapter_num, section_num));
        }
        if line.contains("<$fn>") {
            figure_num += 1;
            line = line.replace("<$fn>", &figure_num.to_string());
        }
        if line.contains("<$pagebreak>") {
            line = line.replace("<$pagebreak>", "\n---\n");
        }
        line
    }).collect();

    lines.join("\n")
}

/// Generate a table of contents from the compiled content
pub fn generate_toc(sections: &[(String, usize)]) -> String {
    let mut toc = String::new();
    toc.push_str("Table of Contents\n");
    toc.push_str("=================\n\n");

    for (i, (title, depth)) in sections.iter().enumerate() {
        let indent = "  ".repeat(*depth);
        let num = i + 1;
        toc.push_str(&format!("{}{}.  {}\n", indent, num, title));
    }
    toc.push('\n');
    toc
}

/// Generate a table of contents in Markdown format
pub fn generate_toc_markdown(sections: &[(String, usize)]) -> String {
    let mut toc = String::new();
    toc.push_str("# Table of Contents\n\n");

    for (title, depth) in sections {
        let indent = "  ".repeat(*depth);
        let slug = title.to_lowercase().replace(' ', "-")
            .chars().filter(|c| c.is_alphanumeric() || *c == '-').collect::<String>();
        toc.push_str(&format!("{}- [{}](#{})\n", indent, title, slug));
    }
    toc.push('\n');
    toc
}

/// Generate a table of contents in HTML format
pub fn generate_toc_html(sections: &[(String, usize)]) -> String {
    let mut toc = String::new();
    toc.push_str("<nav class=\"toc\">\n<h2>Table of Contents</h2>\n<ul>\n");

    for (title, _depth) in sections {
        let slug = title.to_lowercase().replace(' ', "-")
            .chars().filter(|c| c.is_alphanumeric() || *c == '-').collect::<String>();
        toc.push_str(&format!("  <li><a href=\"#{}\">{}</a></li>\n", slug, title));
    }

    toc.push_str("</ul>\n</nav>\n");
    toc
}

/// Context for placeholder replacement
pub struct PlaceholderContext {
    pub project_title: String,
    pub author: String,
    pub word_count: usize,
    pub char_count: usize,
    pub page_count: usize,
}

impl PlaceholderContext {
    pub fn surname(&self) -> String {
        self.author.split_whitespace().last().unwrap_or("").to_string()
    }

    pub fn forename(&self) -> String {
        self.author.split_whitespace().next().unwrap_or("").to_string()
    }

    /// Create a context with default values
    pub fn default_with_title(title: &str) -> Self {
        Self {
            project_title: title.to_string(),
            author: String::new(),
            word_count: 0,
            char_count: 0,
            page_count: 0,
        }
    }

    /// Author initials (e.g., "J.R.R." from "John Ronald Reuel")
    pub fn author_initials(&self) -> String {
        self.author.split_whitespace()
            .filter_map(|w| w.chars().next())
            .map(|c| format!("{}.", c.to_uppercase()))
            .collect::<Vec<_>>()
            .join("")
    }
}

/// List all supported placeholders for documentation
pub fn supported_placeholders() -> Vec<(&'static str, &'static str)> {
    vec![
        ("<$date>", "Current date (YYYY-MM-DD)"),
        ("<$longdate>", "Current date (Month Day, Year)"),
        ("<$shortdate>", "Current date (MM/DD/YY)"),
        ("<$time>", "Current time (HH:MM)"),
        ("<$year>", "Current year"),
        ("<$month>", "Current month name"),
        ("<$day>", "Current day of month"),
        ("<$projecttitle>", "Project title"),
        ("<$author>", "Author full name"),
        ("<$surname>", "Author surname"),
        ("<$forename>", "Author forename"),
        ("<$wc>", "Word count"),
        ("<$wordcount>", "Word count (alias)"),
        ("<$cc>", "Character count"),
        ("<$charcount>", "Character count (alias)"),
        ("<$pagecount>", "Estimated page count"),
        ("<$n>", "Auto-number (chapter)"),
        ("<$N>", "Auto-number (Roman numeral)"),
        ("<$W>", "Auto-number (English word)"),
        ("<$sn>", "Section number (chapter.section)"),
        ("<$fn>", "Figure number"),
        ("<$pagebreak>", "Page break marker"),
    ]
}

/// Count how many placeholders appear in a text
pub fn count_placeholders(text: &str) -> usize {
    let mut count = 0;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '<' && chars.peek() == Some(&'$') {
            // Found a potential placeholder start
            let rest: String = chars.clone().take_while(|&ch| ch != '>').collect();
            if !rest.is_empty() {
                count += 1;
            }
        }
    }
    count
}

/// Convert a number to Roman numerals
fn to_roman(num: usize) -> String {
    let values = [
        (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
        (100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
        (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I"),
    ];
    let mut result = String::new();
    let mut n = num;
    for &(value, numeral) in &values {
        while n >= value {
            result.push_str(numeral);
            n -= value;
        }
    }
    result
}

/// Convert a number to its English word
fn to_word(num: usize) -> String {
    match num {
        1 => "One".to_string(),
        2 => "Two".to_string(),
        3 => "Three".to_string(),
        4 => "Four".to_string(),
        5 => "Five".to_string(),
        6 => "Six".to_string(),
        7 => "Seven".to_string(),
        8 => "Eight".to_string(),
        9 => "Nine".to_string(),
        10 => "Ten".to_string(),
        11 => "Eleven".to_string(),
        12 => "Twelve".to_string(),
        13 => "Thirteen".to_string(),
        14 => "Fourteen".to_string(),
        15 => "Fifteen".to_string(),
        16 => "Sixteen".to_string(),
        17 => "Seventeen".to_string(),
        18 => "Eighteen".to_string(),
        19 => "Nineteen".to_string(),
        20 => "Twenty".to_string(),
        21..=29 => format!("Twenty-{}", to_word(num - 20).to_lowercase()),
        30 => "Thirty".to_string(),
        31..=39 => format!("Thirty-{}", to_word(num - 30).to_lowercase()),
        40 => "Forty".to_string(),
        41..=49 => format!("Forty-{}", to_word(num - 40).to_lowercase()),
        50 => "Fifty".to_string(),
        _ => num.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context() -> PlaceholderContext {
        PlaceholderContext {
            project_title: "My Novel".to_string(),
            author: "John Doe".to_string(),
            word_count: 50000,
            char_count: 250000,
            page_count: 200,
        }
    }

    #[test]
    fn test_replace_project_placeholders() {
        let ctx = make_context();
        let result = replace_placeholders("Title: <$projecttitle> by <$author>", &ctx);
        assert_eq!(result, "Title: My Novel by John Doe");
    }

    #[test]
    fn test_replace_stats_placeholders() {
        let ctx = make_context();
        let result = replace_placeholders("Words: <$wc>, Chars: <$cc>, Pages: <$pagecount>", &ctx);
        assert_eq!(result, "Words: 50000, Chars: 250000, Pages: 200");
    }

    #[test]
    fn test_replace_name_parts() {
        let ctx = make_context();
        let result = replace_placeholders("<$surname>, <$forename>", &ctx);
        assert_eq!(result, "Doe, John");
    }

    #[test]
    fn test_auto_numbering() {
        let ctx = make_context();
        let text = "Chapter <$n>\nScene 1\nChapter <$n>\nScene 2";
        let result = replace_placeholders(text, &ctx);
        assert!(result.contains("Chapter 1"));
        assert!(result.contains("Chapter 2"));
    }

    #[test]
    fn test_roman_numeral_numbering() {
        let ctx = make_context();
        let text = "Part <$N>\nContent\nPart <$N>";
        let result = replace_placeholders(text, &ctx);
        assert!(result.contains("Part I"));
        assert!(result.contains("Part II"));
    }

    #[test]
    fn test_word_numbering() {
        let ctx = make_context();
        let text = "Chapter <$W>\nChapter <$W>";
        let result = replace_placeholders(text, &ctx);
        assert!(result.contains("Chapter One"));
        assert!(result.contains("Chapter Two"));
    }

    #[test]
    fn test_section_numbering() {
        let ctx = make_context();
        let text = "Chapter <$n>\nSection <$sn>\nSection <$sn>";
        let result = replace_placeholders(text, &ctx);
        assert!(result.contains("Chapter 1"));
        assert!(result.contains("Section 1.1"));
        assert!(result.contains("Section 1.2"));
    }

    #[test]
    fn test_to_roman() {
        assert_eq!(to_roman(1), "I");
        assert_eq!(to_roman(4), "IV");
        assert_eq!(to_roman(9), "IX");
        assert_eq!(to_roman(14), "XIV");
        assert_eq!(to_roman(42), "XLII");
        assert_eq!(to_roman(1999), "MCMXCIX");
    }

    #[test]
    fn test_to_word() {
        assert_eq!(to_word(1), "One");
        assert_eq!(to_word(10), "Ten");
        assert_eq!(to_word(15), "Fifteen");
        assert_eq!(to_word(21), "Twenty-one");
        assert_eq!(to_word(42), "Forty-two");
    }

    #[test]
    fn test_generate_toc() {
        let sections = vec![
            ("Chapter 1".to_string(), 0),
            ("Scene 1".to_string(), 1),
            ("Chapter 2".to_string(), 0),
        ];
        let toc = generate_toc(&sections);
        assert!(toc.contains("Table of Contents"));
        assert!(toc.contains("Chapter 1"));
        assert!(toc.contains("Scene 1"));
    }

    #[test]
    fn test_generate_toc_markdown() {
        let sections = vec![
            ("Chapter One".to_string(), 0),
            ("Scene A".to_string(), 1),
        ];
        let toc = generate_toc_markdown(&sections);
        assert!(toc.contains("# Table of Contents"));
        assert!(toc.contains("[Chapter One]"));
        assert!(toc.contains("#chapter-one"));
    }

    #[test]
    fn test_generate_toc_html() {
        let sections = vec![("Intro".to_string(), 0)];
        let toc = generate_toc_html(&sections);
        assert!(toc.contains("<nav"));
        assert!(toc.contains("Intro"));
        assert!(toc.contains("</ul>"));
    }

    #[test]
    fn test_count_placeholders() {
        assert_eq!(count_placeholders("Hello <$name> and <$date>"), 2);
        assert_eq!(count_placeholders("No placeholders here"), 0);
        assert_eq!(count_placeholders("<$n> <$N> <$W>"), 3);
    }

    #[test]
    fn test_supported_placeholders() {
        let ph = supported_placeholders();
        assert!(ph.len() >= 20);
        assert!(ph.iter().any(|(k, _)| *k == "<$date>"));
        assert!(ph.iter().any(|(k, _)| *k == "<$author>"));
    }

    #[test]
    fn test_placeholder_context() {
        let ctx = make_context();
        assert_eq!(ctx.surname(), "Doe");
        assert_eq!(ctx.forename(), "John");
        assert_eq!(ctx.author_initials(), "J.D.");
    }

    #[test]
    fn test_default_context() {
        let ctx = PlaceholderContext::default_with_title("Test");
        assert_eq!(ctx.project_title, "Test");
        assert_eq!(ctx.word_count, 0);
    }

    #[test]
    fn test_pagebreak_placeholder() {
        let ctx = make_context();
        let result = replace_placeholders("Before<$pagebreak>After", &ctx);
        assert!(result.contains("---"));
    }
}
