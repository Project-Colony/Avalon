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
                            if chars[i] == '{' {
                                depth += 1;
                            } else if chars[i] == '}' {
                                depth -= 1;
                            }
                            if depth > 0 {
                                result.push(chars[i]);
                            }
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
                            if chars[i] == '{' {
                                depth += 1;
                            } else if chars[i] == '}' {
                                depth -= 1;
                            }
                            if depth > 0 {
                                result.push(chars[i]);
                            }
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
                        if i < chars.len() {
                            i += 1;
                        } // skip }
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
                            if chars[i] == '{' {
                                depth += 1;
                            } else if chars[i] == '}' {
                                depth -= 1;
                            }
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
                if i + 1 < chars.len() {
                    i += 2;
                }
            } else {
                // Inline math $...$
                while i < chars.len() && chars[i] != '$' {
                    result.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- title_case tests ---

    #[test]
    fn test_title_case_basic() {
        assert_eq!(title_case("hello world"), "Hello World");
    }

    #[test]
    fn test_title_case_empty() {
        assert_eq!(title_case(""), "");
    }

    #[test]
    fn test_title_case_single_word() {
        assert_eq!(title_case("rust"), "Rust");
    }

    #[test]
    fn test_title_case_already_cased() {
        assert_eq!(title_case("HELLO WORLD"), "Hello World");
    }

    #[test]
    fn test_title_case_mixed() {
        assert_eq!(title_case("the QUICK brown FOX"), "The Quick Brown Fox");
    }

    // --- strip_latex_commands tests ---

    #[test]
    fn test_strip_latex_textbf() {
        let result = strip_latex_commands("\\textbf{bold}");
        assert_eq!(result.trim(), "bold");
    }

    #[test]
    fn test_strip_latex_section() {
        let result = strip_latex_commands("\\section{My Section}");
        assert!(result.contains("My Section"));
    }

    #[test]
    fn test_strip_latex_item() {
        let result = strip_latex_commands("\\begin{itemize}\n\\item First\n\\item Second\n\\end{itemize}");
        assert!(result.contains("First"));
        assert!(result.contains("Second"));
    }

    #[test]
    fn test_strip_latex_comment() {
        let result = strip_latex_commands("text % comment\nnext line");
        assert!(result.contains("text"));
        assert!(result.contains("next line"));
        assert!(!result.contains("comment"));
    }

    #[test]
    fn test_strip_latex_empty() {
        assert_eq!(strip_latex_commands("").trim(), "");
    }

    #[test]
    fn test_strip_latex_plain_text() {
        assert_eq!(strip_latex_commands("just text").trim(), "just text");
    }

    #[test]
    fn test_strip_latex_math() {
        let result = strip_latex_commands("$x + y$");
        assert!(result.contains("x + y"));
    }
}
