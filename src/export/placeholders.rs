use std::fmt::Write as _;
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
        let _ = writeln!(toc,"{}{}.  {}", indent, num, title);
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
        let _ = writeln!(toc,"{}- [{}](#{})", indent, title, slug);
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
        let _ = writeln!(toc,"  <li><a href=\"#{}\">{}</a></li>", slug, title);
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

    /// Estimated reading time in minutes (assuming 200 words/minute)
    pub fn estimated_reading_time(&self) -> usize {
        (self.word_count / 200).max(if self.word_count > 0 { 1 } else { 0 })
    }

    /// Build context from compile contents
    pub fn from_contents(
        contents: &[super::compiler::CompileContent],
        title: &str,
        author: &str,
    ) -> Self {
        let word_count: usize = contents.iter().map(|c| c.text.split_whitespace().count()).sum();
        let char_count: usize = contents.iter().map(|c| c.text.len()).sum();
        Self {
            project_title: title.to_string(),
            author: author.to_string(),
            word_count,
            char_count,
            page_count: (word_count / 250).max(if word_count > 0 { 1 } else { 0 }),
        }
    }

    /// Format a compact credit line like "Title by Author (N words)"
    pub fn credit_line(&self) -> String {
        if self.author.is_empty() {
            format!("{} ({} words)", self.project_title, self.word_count)
        } else {
            format!(
                "{} by {} ({} words)",
                self.project_title, self.author, self.word_count,
            )
        }
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

    #[test]
    fn test_figure_numbering() {
        let ctx = make_context();
        let text = "Figure <$fn>: Diagram\nFigure <$fn>: Photo\nFigure <$fn>: Chart";
        let result = replace_placeholders(text, &ctx);
        assert!(result.contains("Figure 1"));
        assert!(result.contains("Figure 2"));
        assert!(result.contains("Figure 3"));
    }

    #[test]
    fn test_section_numbering_resets_per_chapter() {
        let ctx = make_context();
        let text = "Chapter <$n>\nSection <$sn>\nSection <$sn>\nChapter <$n>\nSection <$sn>";
        let result = replace_placeholders(text, &ctx);
        assert!(result.contains("Chapter 1"));
        assert!(result.contains("Section 1.1"));
        assert!(result.contains("Section 1.2"));
        assert!(result.contains("Chapter 2"));
        assert!(result.contains("Section 2.1"));
    }

    #[test]
    fn test_no_placeholders() {
        let ctx = make_context();
        let result = replace_placeholders("Just plain text, no placeholders.", &ctx);
        assert_eq!(result, "Just plain text, no placeholders.");
    }

    #[test]
    fn test_multiple_placeholders_same_line() {
        let ctx = make_context();
        let result = replace_placeholders("<$projecttitle> by <$author> (<$wc> words)", &ctx);
        assert_eq!(result, "My Novel by John Doe (50000 words)");
    }

    #[test]
    fn test_alias_placeholders() {
        let ctx = make_context();
        let wc = replace_placeholders("<$wc>", &ctx);
        let wordcount = replace_placeholders("<$wordcount>", &ctx);
        assert_eq!(wc, wordcount);

        let cc = replace_placeholders("<$cc>", &ctx);
        let charcount = replace_placeholders("<$charcount>", &ctx);
        assert_eq!(cc, charcount);
    }

    #[test]
    fn test_date_placeholders_format() {
        let ctx = make_context();
        let date = replace_placeholders("<$date>", &ctx);
        // YYYY-MM-DD format
        assert_eq!(date.len(), 10);
        assert_eq!(date.chars().nth(4), Some('-'));
        assert_eq!(date.chars().nth(7), Some('-'));
    }

    #[test]
    fn test_empty_author() {
        let ctx = PlaceholderContext {
            project_title: "Test".to_string(),
            author: String::new(),
            word_count: 0,
            char_count: 0,
            page_count: 0,
        };
        assert_eq!(ctx.surname(), "");
        assert_eq!(ctx.forename(), "");
        assert_eq!(ctx.author_initials(), "");
    }

    #[test]
    fn test_multi_word_author() {
        let ctx = PlaceholderContext {
            project_title: "Test".to_string(),
            author: "John Ronald Reuel Tolkien".to_string(),
            word_count: 0,
            char_count: 0,
            page_count: 0,
        };
        assert_eq!(ctx.surname(), "Tolkien");
        assert_eq!(ctx.forename(), "John");
        assert_eq!(ctx.author_initials(), "J.R.R.T.");
    }

    #[test]
    fn test_to_roman_edge_cases() {
        assert_eq!(to_roman(0), "");
        assert_eq!(to_roman(3), "III");
        assert_eq!(to_roman(50), "L");
        assert_eq!(to_roman(100), "C");
        assert_eq!(to_roman(500), "D");
        assert_eq!(to_roman(1000), "M");
    }

    #[test]
    fn test_to_word_edge_cases() {
        assert_eq!(to_word(11), "Eleven");
        assert_eq!(to_word(19), "Nineteen");
        assert_eq!(to_word(20), "Twenty");
        assert_eq!(to_word(25), "Twenty-five");
        assert_eq!(to_word(30), "Thirty");
        assert_eq!(to_word(35), "Thirty-five");
        assert_eq!(to_word(40), "Forty");
        assert_eq!(to_word(45), "Forty-five");
        assert_eq!(to_word(50), "Fifty");
        assert_eq!(to_word(99), "99"); // Falls through to default
    }

    #[test]
    fn test_generate_toc_empty() {
        let sections: Vec<(String, usize)> = vec![];
        let toc = generate_toc(&sections);
        assert!(toc.contains("Table of Contents"));
        // No entries
        assert!(!toc.contains("1."));
    }

    #[test]
    fn test_generate_toc_indentation() {
        let sections = vec![
            ("Part I".to_string(), 0),
            ("Chapter 1".to_string(), 1),
            ("Scene 1".to_string(), 2),
        ];
        let toc = generate_toc(&sections);
        assert!(toc.contains("Part I"));
        assert!(toc.contains("  2.  Chapter 1"));
        assert!(toc.contains("    3.  Scene 1"));
    }

    #[test]
    fn test_generate_toc_markdown_slugs() {
        let sections = vec![
            ("My Great Chapter".to_string(), 0),
            ("Sub Section Here".to_string(), 1),
        ];
        let toc = generate_toc_markdown(&sections);
        assert!(toc.contains("#my-great-chapter"));
        assert!(toc.contains("#sub-section-here"));
        assert!(toc.contains("[My Great Chapter]"));
    }

    #[test]
    fn test_generate_toc_html_structure() {
        let sections = vec![
            ("Intro".to_string(), 0),
            ("Body".to_string(), 0),
            ("Conclusion".to_string(), 0),
        ];
        let toc = generate_toc_html(&sections);
        assert!(toc.starts_with("<nav"));
        assert!(toc.contains("<li>"));
        assert!(toc.contains("</nav>"));
        // Count list items
        let li_count = toc.matches("<li>").count();
        assert_eq!(li_count, 3);
    }

    #[test]
    fn test_count_placeholders_none() {
        assert_eq!(count_placeholders(""), 0);
        assert_eq!(count_placeholders("no placeholders"), 0);
        assert_eq!(count_placeholders("angle < bracket > but not placeholder"), 0);
    }

    #[test]
    fn test_count_placeholders_mixed() {
        assert_eq!(count_placeholders("Title: <$projecttitle> by <$author>, <$date>"), 3);
        assert_eq!(count_placeholders("Ch <$n> Sec <$sn>"), 2);
    }

    #[test]
    fn test_supported_placeholders_all_documented() {
        let ph = supported_placeholders();
        // Verify each documented placeholder actually has a description
        for (placeholder, desc) in &ph {
            assert!(placeholder.starts_with("<$"), "Placeholder should start with <$: {}", placeholder);
            assert!(placeholder.ends_with(">"), "Placeholder should end with >: {}", placeholder);
            assert!(!desc.is_empty(), "Description should not be empty for {}", placeholder);
        }
    }

    #[test]
    fn test_replace_year_placeholder() {
        let ctx = make_context();
        let result = replace_placeholders("<$year>", &ctx);
        // Should be a 4-digit year
        assert_eq!(result.len(), 4);
        assert!(result.parse::<u32>().is_ok());
    }

    // ---- New placeholder context helpers ----

    #[test]
    fn test_estimated_reading_time() {
        let ctx = PlaceholderContext {
            project_title: "Test".to_string(),
            author: "Author".to_string(),
            word_count: 1000,
            char_count: 5000,
            page_count: 4,
        };
        assert_eq!(ctx.estimated_reading_time(), 5); // 1000/200 = 5
    }

    #[test]
    fn test_estimated_reading_time_short() {
        let ctx = PlaceholderContext {
            project_title: "T".to_string(),
            author: "A".to_string(),
            word_count: 50,
            char_count: 250,
            page_count: 1,
        };
        assert_eq!(ctx.estimated_reading_time(), 1); // min 1
    }

    #[test]
    fn test_estimated_reading_time_zero() {
        let ctx = PlaceholderContext::default_with_title("Empty");
        assert_eq!(ctx.estimated_reading_time(), 0);
    }

    #[test]
    fn test_from_contents() {
        use crate::export::compiler::CompileContent;
        let contents = vec![
            CompileContent { title: "Ch1".into(), text: "one two three four five".into(), depth: 0, is_folder: false },
            CompileContent { title: "Ch2".into(), text: "six seven eight".into(), depth: 0, is_folder: false },
        ];
        let ctx = PlaceholderContext::from_contents(&contents, "My Book", "Jane Smith");
        assert_eq!(ctx.project_title, "My Book");
        assert_eq!(ctx.author, "Jane Smith");
        assert_eq!(ctx.word_count, 8);
        assert_eq!(ctx.page_count, 1);
    }

    #[test]
    fn test_from_contents_empty() {
        use crate::export::compiler::CompileContent;
        let contents: Vec<CompileContent> = vec![];
        let ctx = PlaceholderContext::from_contents(&contents, "Empty", "");
        assert_eq!(ctx.word_count, 0);
        assert_eq!(ctx.page_count, 0);
    }

    #[test]
    fn test_credit_line() {
        let ctx = make_context();
        let line = ctx.credit_line();
        assert!(line.contains("My Novel"));
        assert!(line.contains("John Doe"));
        assert!(line.contains("50000"));
    }

    #[test]
    fn test_credit_line_no_author() {
        let ctx = PlaceholderContext {
            project_title: "Solo".to_string(),
            author: String::new(),
            word_count: 100,
            char_count: 500,
            page_count: 1,
        };
        let line = ctx.credit_line();
        assert!(line.contains("Solo"));
        assert!(!line.contains("by"));
    }
}
