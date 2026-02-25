/// Convert text to Title Case
pub fn title_case(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    let rest: String = chars.collect();
                    format!("{}{}", upper, rest.to_lowercase())
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Strip HTML tags from content for plain text import
pub fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script = false;

    let lower = html.to_lowercase();
    let chars: Vec<char> = html.chars().collect();
    let lower_chars: Vec<char> = lower.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        if !in_tag && i + 7 < lower_chars.len() {
            let slice: String = lower_chars[i..i + 7].iter().collect();
            if slice == "<script" {
                in_script = true;
            }
        }
        if in_script && i + 8 < lower_chars.len() {
            let slice: String = lower_chars[i..i + 9].iter().collect();
            if slice == "</script>" {
                in_script = false;
                i += 9;
                continue;
            }
        }

        if in_script {
            i += 1;
            continue;
        }

        if chars[i] == '<' {
            in_tag = true;
            // Convert block elements to newlines
            if i + 2 < lower_chars.len() {
                let next_two: String = lower_chars[i + 1..i + 3.min(lower_chars.len())].iter().collect();
                if next_two.starts_with('p') || next_two.starts_with('b') || next_two.starts_with('h')
                    || next_two.starts_with('l') || next_two.starts_with('d')
                    || next_two.starts_with('t')
                {
                    result.push('\n');
                }
            }
        } else if chars[i] == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(chars[i]);
        }
        i += 1;
    }

    // Clean up excessive newlines
    let mut cleaned = String::new();
    let mut prev_was_newline = false;
    for ch in result.chars() {
        if ch == '\n' {
            if !prev_was_newline {
                cleaned.push('\n');
            }
            prev_was_newline = true;
        } else {
            prev_was_newline = false;
            cleaned.push(ch);
        }
    }

    // Unescape common HTML entities
    cleaned = cleaned.replace("&amp;", "&");
    cleaned = cleaned.replace("&lt;", "<");
    cleaned = cleaned.replace("&gt;", ">");
    cleaned = cleaned.replace("&quot;", "\"");
    cleaned = cleaned.replace("&#39;", "'");
    cleaned = cleaned.replace("&nbsp;", " ");

    cleaned.trim().to_string()
}

/// Strip LaTeX commands from content for plain text import
pub fn strip_latex_commands(latex: &str) -> String {
    let mut result = String::new();
    let mut i = 0;
    let chars: Vec<char> = latex.chars().collect();

    while i < chars.len() {
        if chars[i] == '\\' {
            // Skip the command name
            i += 1;
            // Check for \begin{...} and \end{...} — skip the whole thing
            let mut cmd = String::new();
            while i < chars.len() && chars[i].is_alphanumeric() {
                cmd.push(chars[i]);
                i += 1;
            }
            // Convert some commands to their content
            match cmd.as_str() {
                "section" | "subsection" | "subsubsection" | "chapter" | "part" => {
                    result.push('\n');
                    result.push('\n');
                    // Skip the {title} — extract the text inside
                    if i < chars.len() && chars[i] == '{' {
                        i += 1; // skip {
                        let mut depth = 1;
                        while i < chars.len() && depth > 0 {
                            if chars[i] == '{' { depth += 1; }
                            else if chars[i] == '}' { depth -= 1; }
                            if depth > 0 { result.push(chars[i]); }
                            i += 1;
                        }
                    }
                    result.push('\n');
                }
                "textbf" | "textit" | "emph" | "underline" | "textsf" | "texttt" => {
                    // Extract content from braces
                    if i < chars.len() && chars[i] == '{' {
                        i += 1;
                        let mut depth = 1;
                        while i < chars.len() && depth > 0 {
                            if chars[i] == '{' { depth += 1; }
                            else if chars[i] == '}' { depth -= 1; }
                            if depth > 0 { result.push(chars[i]); }
                            i += 1;
                        }
                    }
                }
                "begin" | "end" => {
                    // Skip {environment}
                    if i < chars.len() && chars[i] == '{' {
                        i += 1;
                        let mut env = String::new();
                        while i < chars.len() && chars[i] != '}' {
                            env.push(chars[i]);
                            i += 1;
                        }
                        if i < chars.len() { i += 1; } // skip }
                        if env == "itemize" || env == "enumerate" || env == "description" {
                            result.push('\n');
                        }
                    }
                }
                "item" => {
                    result.push('\n');
                    result.push_str("  - ");
                }
                "par" | "newline" | "linebreak" => {
                    result.push('\n');
                }
                _ => {
                    // Skip unknown commands and their optional/required args
                    if i < chars.len() && chars[i] == '{' {
                        let mut depth = 1;
                        i += 1;
                        while i < chars.len() && depth > 0 {
                            if chars[i] == '{' { depth += 1; }
                            else if chars[i] == '}' { depth -= 1; }
                            i += 1;
                        }
                    }
                }
            }
        } else if chars[i] == '{' || chars[i] == '}' {
            // Skip bare braces
            i += 1;
        } else if chars[i] == '%' {
            // Skip LaTeX comments
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if chars[i] == '$' {
            // Skip math mode
            i += 1;
            if i < chars.len() && chars[i] == '$' {
                // Display math $$...$$
                i += 1;
                while i + 1 < chars.len() && !(chars[i] == '$' && chars[i + 1] == '$') {
                    result.push(chars[i]);
                    i += 1;
                }
                if i + 1 < chars.len() { i += 2; }
            } else {
                // Inline math $...$
                while i < chars.len() && chars[i] != '$' {
                    result.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() { i += 1; }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    // Clean up multiple blank lines
    let mut cleaned = String::new();
    let mut blank_count = 0;
    for line in result.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                cleaned.push('\n');
            }
        } else {
            blank_count = 0;
            cleaned.push_str(line);
            cleaned.push('\n');
        }
    }

    cleaned.trim().to_string()
}
