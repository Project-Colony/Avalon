use chrono::Utc;
use std::fmt::Write as _;

/// Replace placeholders in compiled text with actual values.
/// Scrivener-compatible placeholder syntax: <$placeholder>
pub fn replace_placeholders(text: &str, context: &PlaceholderContext) -> String {
    // Early exit: skip all work if no placeholders present
    if !text.contains("<$") {
        return text.to_string();
    }

    // Build replacement table once (avoids repeated formatting)
    let now = Utc::now();
    let wc = context.word_count.to_string();
    let cc = context.char_count.to_string();
    let pc = context.page_count.to_string();

    let simple_replacements: &[(&str, &str)] = &[
        ("<$date>", &now.format(crate::core::DATE_FORMAT).to_string()),
        ("<$longdate>", &now.format("%B %d, %Y").to_string()),
        ("<$shortdate>", &now.format("%m/%d/%y").to_string()),
        ("<$time>", &now.format("%H:%M").to_string()),
        ("<$year>", &now.format("%Y").to_string()),
        ("<$month>", &now.format("%B").to_string()),
        ("<$day>", &now.format("%d").to_string()),
        ("<$projecttitle>", &context.project_title),
        ("<$author>", &context.author),
        ("<$surname>", &context.surname()),
        ("<$forename>", &context.forename()),
        ("<$wc>", &wc),
        ("<$wordcount>", &wc),
        ("<$cc>", &cc),
        ("<$charcount>", &cc),
        ("<$pagecount>", &pc),
    ];

    // Single pass for simple (non-counting) placeholders
    let mut result = text.to_string();
    for &(placeholder, value) in simple_replacements {
        if result.contains(placeholder) {
            result = result.replace(placeholder, value);
        }
    }

    // Auto-numbering requires per-line stateful processing
    let has_numbering = result.contains("<$n>")
        || result.contains("<$N>")
        || result.contains("<$W>")
        || result.contains("<$sn>")
        || result.contains("<$fn>")
        || result.contains("<$pagebreak>");

    if !has_numbering {
        return result;
    }

    let mut chapter_num = 0;
    let mut section_num = 0;
    let mut figure_num = 0;

    let lines: Vec<String> = result
        .lines()
        .map(|l| {
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
        })
        .collect();

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
        writeln!(toc, "{}{}.  {}", indent, num, title).unwrap();
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
        let slug = title
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();
        writeln!(toc, "{}- [{}](#{})", indent, title, slug).unwrap();
    }
    toc.push('\n');
    toc
}

/// Generate a table of contents in HTML format
pub fn generate_toc_html(sections: &[(String, usize)]) -> String {
    let mut toc = String::new();
    toc.push_str("<nav class=\"toc\">\n<h2>Table of Contents</h2>\n<ul>\n");

    for (title, _depth) in sections {
        let slug = title
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();
        writeln!(toc, "  <li><a href=\"#{}\">{}</a></li>", slug, title).unwrap();
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
}

/// Convert a number to Roman numerals
fn to_roman(num: usize) -> String {
    let values = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
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

    fn test_context() -> PlaceholderContext {
        PlaceholderContext {
            project_title: "My Novel".to_string(),
            author: "Jane Smith".to_string(),
            word_count: 50000,
            char_count: 250000,
            page_count: 200,
        }
    }

    #[test]
    fn test_surname() {
        assert_eq!(test_context().surname(), "Smith");
    }

    #[test]
    fn test_forename() {
        assert_eq!(test_context().forename(), "Jane");
    }

    #[test]
    fn test_no_placeholders() {
        let result = replace_placeholders("Hello world", &test_context());
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_project_title_placeholder() {
        let result = replace_placeholders("Title: <$projecttitle>", &test_context());
        assert_eq!(result, "Title: My Novel");
    }

    #[test]
    fn test_author_placeholder() {
        let result = replace_placeholders("By <$author>", &test_context());
        assert_eq!(result, "By Jane Smith");
    }

    #[test]
    fn test_name_placeholders() {
        let result = replace_placeholders("<$forename> <$surname>", &test_context());
        assert_eq!(result, "Jane Smith");
    }

    #[test]
    fn test_word_count_placeholders() {
        let result = replace_placeholders("<$wc> / <$wordcount>", &test_context());
        assert_eq!(result, "50000 / 50000");
    }

    #[test]
    fn test_auto_numbering_n() {
        let text = "Ch <$n>\nCh <$n>\nCh <$n>";
        let result = replace_placeholders(text, &test_context());
        assert!(result.contains("Ch 1"));
        assert!(result.contains("Ch 2"));
        assert!(result.contains("Ch 3"));
    }

    #[test]
    fn test_auto_numbering_roman() {
        let text = "Part <$N>\nPart <$N>";
        let result = replace_placeholders(text, &test_context());
        assert!(result.contains("Part I"));
        assert!(result.contains("Part II"));
    }

    #[test]
    fn test_auto_numbering_word() {
        let text = "Ch <$W>\nCh <$W>";
        let result = replace_placeholders(text, &test_context());
        assert!(result.contains("Ch One"));
        assert!(result.contains("Ch Two"));
    }

    #[test]
    fn test_section_numbering() {
        let text = "Ch <$n>\nSec <$sn>\nSec <$sn>\nCh <$n>\nSec <$sn>";
        let result = replace_placeholders(text, &test_context());
        assert!(result.contains("Sec 1.1"));
        assert!(result.contains("Sec 1.2"));
        assert!(result.contains("Sec 2.1"));
    }

    #[test]
    fn test_generate_toc() {
        let sections = vec![("Chapter 1".to_string(), 0), ("Scene 1".to_string(), 1)];
        let toc = generate_toc(&sections);
        assert!(toc.contains("Table of Contents"));
        assert!(toc.contains("1.  Chapter 1"));
    }

    #[test]
    fn test_generate_toc_markdown() {
        let sections = vec![("Intro".to_string(), 0)];
        let toc = generate_toc_markdown(&sections);
        assert!(toc.contains("[Intro]"));
        assert!(toc.contains("#intro"));
    }

    #[test]
    fn test_generate_toc_html() {
        let sections = vec![("Intro".to_string(), 0)];
        let toc = generate_toc_html(&sections);
        assert!(toc.contains("<nav"));
        assert!(toc.contains("Intro"));
    }

    #[test]
    fn test_to_roman() {
        assert_eq!(to_roman(1), "I");
        assert_eq!(to_roman(4), "IV");
        assert_eq!(to_roman(9), "IX");
        assert_eq!(to_roman(42), "XLII");
    }

    #[test]
    fn test_to_word() {
        assert_eq!(to_word(1), "One");
        assert_eq!(to_word(10), "Ten");
        assert_eq!(to_word(21), "Twenty-one");
        assert_eq!(to_word(50), "Fifty");
        assert_eq!(to_word(99), "99");
    }
}
