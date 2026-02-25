use std::fmt::Write as _;
use chrono::Utc;

/// Replace placeholders in compiled text with actual values.
/// Scrivener-compatible placeholder syntax: <$placeholder>
pub fn replace_placeholders(text: &str, context: &PlaceholderContext) -> String {
    let mut result = text.to_string();

    // Date/time placeholders
    let now = Utc::now();
    result = result.replace("<$date>", &now.format(crate::core::DATE_FORMAT).to_string());
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
