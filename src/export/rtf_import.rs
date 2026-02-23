//! RTF import utilities for extracting plain text from RTF content.

/// Extract plain text from RTF content by stripping all RTF control words and formatting.
/// This is a best-effort parser that handles common RTF constructs.
pub fn extract_text_from_rtf(rtf: &str) -> String {
    if !rtf.starts_with('{') {
        return rtf.to_string();
    }

    let mut result = String::new();
    let chars: Vec<char> = rtf.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut depth = 0;
    // Track whether we are inside a group we should skip (e.g., fonttbl, colortbl, stylesheet, etc.)
    let mut skip_depth: Option<usize> = None;

    while i < len {
        let ch = chars[i];

        match ch {
            '{' => {
                depth += 1;
                i += 1;
                // Check if this group starts with a known skip-group control word
                let rest = peek_control_word(&chars, i);
                if matches!(
                    rest.as_str(),
                    "\\fonttbl" | "\\colortbl" | "\\stylesheet" | "\\info"
                        | "\\*" | "\\pict" | "\\header" | "\\footer"
                        | "\\headerl" | "\\headerr" | "\\footerl" | "\\footerr"
                )
                    && skip_depth.is_none() {
                        skip_depth = Some(depth);
                    }
            }
            '}' => {
                if skip_depth == Some(depth) {
                    skip_depth = None;
                }
                depth -= 1;
                i += 1;
            }
            _ if skip_depth.is_some() => {
                i += 1;
            }
            '\\' => {
                i += 1;
                if i >= len {
                    break;
                }
                let next = chars[i];
                // Escaped literal characters
                if next == '\\' || next == '{' || next == '}' {
                    result.push(next);
                    i += 1;
                } else if next == '\'' {
                    // Hex-encoded character: \'xx
                    i += 1;
                    if i + 1 < len {
                        let hex: String = chars[i..i + 2].iter().collect();
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                            result.push(byte as char);
                        }
                        i += 2;
                    }
                } else if next == 'u' && i + 1 < len && (chars[i + 1].is_ascii_digit() || chars[i + 1] == '-') {
                    // Unicode character: \uNNNN followed by a replacement char
                    i += 1;
                    let start = i;
                    if chars[i] == '-' {
                        i += 1;
                    }
                    while i < len && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    let num_str: String = chars[start..i].iter().collect();
                    if let Ok(code) = num_str.parse::<i32>() {
                        let code = if code < 0 { (code + 65536) as u32 } else { code as u32 };
                        if let Some(c) = char::from_u32(code) {
                            result.push(c);
                        }
                    }
                    // Skip the replacement character (usually '?')
                    if i < len && chars[i] != '\\' && chars[i] != '{' && chars[i] != '}' {
                        i += 1;
                    }
                } else if next == '\n' || next == '\r' {
                    // Line break after backslash — ignore
                    i += 1;
                } else {
                    // Control word: \wordN or \word-N or \word followed by space/delimiter
                    let mut word = String::new();
                    while i < len && chars[i].is_ascii_alphabetic() {
                        word.push(chars[i]);
                        i += 1;
                    }
                    // Skip optional numeric parameter
                    if i < len && (chars[i].is_ascii_digit() || chars[i] == '-') {
                        while i < len && (chars[i].is_ascii_digit() || chars[i] == '-') {
                            i += 1;
                        }
                    }
                    // A space delimiter is consumed as part of the control word
                    if i < len && chars[i] == ' ' {
                        i += 1;
                    }

                    // Translate known control words
                    match word.as_str() {
                        "par" | "line" => result.push('\n'),
                        "tab" => result.push('\t'),
                        "page" => {
                            result.push('\n');
                            result.push('\n');
                        }
                        "lquote" => result.push('\u{2018}'),
                        "rquote" => result.push('\u{2019}'),
                        "ldblquote" => result.push('\u{201C}'),
                        "rdblquote" => result.push('\u{201D}'),
                        "emdash" => result.push('\u{2014}'),
                        "endash" => result.push('\u{2013}'),
                        "bullet" => result.push('\u{2022}'),
                        _ => {}
                    }
                }
            }
            '\n' | '\r' => {
                // Bare newlines in RTF are ignored
                i += 1;
            }
            _ => {
                result.push(ch);
                i += 1;
            }
        }
    }

    // Clean up: collapse excessive blank lines
    let mut cleaned = String::new();
    let mut blank_count = 0;
    for line in result.split('\n') {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                cleaned.push('\n');
            }
        } else {
            blank_count = 0;
            if !cleaned.is_empty() && !cleaned.ends_with('\n') {
                cleaned.push('\n');
            }
            cleaned.push_str(line);
        }
    }

    cleaned.trim().to_string()
}

/// Peek ahead to read a control word starting at position `i` in the char array.
/// Returns the control word including the leading backslash (if present) or empty string.
fn peek_control_word(chars: &[char], start: usize) -> String {
    let mut i = start;
    let len = chars.len();
    if i >= len || chars[i] != '\\' {
        return String::new();
    }
    let mut word = String::new();
    word.push('\\');
    i += 1;
    // Handle \* destination
    if i < len && chars[i] == '*' {
        word.push('*');
        return word;
    }
    while i < len && chars[i].is_ascii_alphabetic() {
        word.push(chars[i]);
        i += 1;
    }
    word
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_plain_text() {
        let rtf = r"{\rtf1\ansi Hello world}";
        let text = extract_text_from_rtf(rtf);
        assert_eq!(text, "Hello world");
    }

    #[test]
    fn test_extract_with_formatting() {
        let rtf = r"{\rtf1\ansi \b Bold\b0  and \i italic\i0  text}";
        let text = extract_text_from_rtf(rtf);
        assert_eq!(text, "Bold and italic text");
    }

    #[test]
    fn test_extract_paragraphs() {
        let rtf = r"{\rtf1\ansi First paragraph\par Second paragraph}";
        let text = extract_text_from_rtf(rtf);
        assert!(text.contains("First paragraph"));
        assert!(text.contains("Second paragraph"));
    }

    #[test]
    fn test_extract_escaped_braces() {
        let rtf = r"{\rtf1\ansi a\{b\}c}";
        let text = extract_text_from_rtf(rtf);
        assert_eq!(text, "a{b}c");
    }

    #[test]
    fn test_extract_unicode() {
        let rtf = r"{\rtf1\ansi caf\u233?}";
        let text = extract_text_from_rtf(rtf);
        assert!(text.contains("caf\u{00E9}"));
    }

    #[test]
    fn test_extract_non_rtf() {
        let text = "Just plain text, not RTF at all.";
        assert_eq!(extract_text_from_rtf(text), text);
    }

    #[test]
    fn test_extract_empty() {
        assert_eq!(extract_text_from_rtf(""), "");
    }

    #[test]
    fn test_extract_font_table_skipped() {
        let rtf = r"{\rtf1{\fonttbl{\f0 Times New Roman;}}Hello}";
        let text = extract_text_from_rtf(rtf);
        assert_eq!(text, "Hello");
        assert!(!text.contains("Times"));
    }

    #[test]
    fn test_extract_smart_quotes() {
        let rtf = r"{\rtf1\ansi \ldblquote Hello\rdblquote }";
        let text = extract_text_from_rtf(rtf);
        assert!(text.contains('\u{201C}'));
        assert!(text.contains('\u{201D}'));
    }

    #[test]
    fn test_extract_hex_chars() {
        let rtf = r"{\rtf1\ansi caf\'e9}";
        let text = extract_text_from_rtf(rtf);
        assert!(text.contains("caf\u{00E9}"));
    }
}
