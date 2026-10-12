//! RTF import utilities for extracting plain text from RTF content.

/// Extract plain text from RTF content by stripping all RTF control words and formatting.
/// This is a best-effort parser that handles common RTF constructs.
pub fn extract_text_from_rtf(rtf: &str) -> String {
    if !rtf.trim_start().starts_with('{') {
        return rtf.to_string();
    }

    let mut result = String::new();
    let chars: Vec<char> = rtf.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut depth = 0;
    // Track whether we are inside a group we should skip (e.g., fonttbl, colortbl, stylesheet, etc.)
    let mut skip_depth: Option<usize> = None;
    // Number of fallback characters written after each \uN character (set by \ucN).
    let mut unicode_fallback_len = 1;

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
                    "\\fonttbl"
                        | "\\colortbl"
                        | "\\stylesheet"
                        | "\\info"
                        | "\\*"
                        | "\\pict"
                        | "\\header"
                        | "\\footer"
                        | "\\headerl"
                        | "\\headerr"
                        | "\\footerl"
                        | "\\footerr"
                ) && skip_depth.is_none()
                {
                    skip_depth = Some(depth);
                }
            }
            '}' => {
                if skip_depth == Some(depth) {
                    skip_depth = None;
                }
                // A stray closing brace must not underflow the depth.
                depth = depth.saturating_sub(1);
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
                            result.push(cp1252_char(byte));
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
                    // The space after the number delimits the control word.
                    if i < len && chars[i] == ' ' {
                        i += 1;
                    }
                    // Skip the fallback for readers without Unicode support:
                    // usually '?' or a \'xx escape, one per \ucN count.
                    for _ in 0..unicode_fallback_len {
                        if chars.get(i) == Some(&'\\') && chars.get(i + 1) == Some(&'\'') {
                            i += 4;
                        } else if i < len && !matches!(chars[i], '\\' | '{' | '}') {
                            i += 1;
                        } else {
                            break;
                        }
                    }
                } else if next == '\n' || next == '\r' {
                    // A backslash before a line break is a paragraph break,
                    // which is how TextEdit writes every new line.
                    result.push('\n');
                    i += 1;
                } else if !next.is_ascii_alphabetic() {
                    // Control symbol: \~ is a non-breaking space, \_ a
                    // non-breaking hyphen; the others (\- optional hyphen,
                    // \* and so on) add no text.
                    match next {
                        '~' => result.push('\u{a0}'),
                        '_' => result.push('\u{2011}'),
                        _ => {}
                    }
                    i += 1;
                } else {
                    // Control word: \wordN or \word-N or \word followed by space/delimiter
                    let mut word = String::new();
                    while i < len && chars[i].is_ascii_alphabetic() {
                        word.push(chars[i]);
                        i += 1;
                    }
                    // Optional numeric parameter
                    let param_start = i;
                    if i < len && (chars[i].is_ascii_digit() || chars[i] == '-') {
                        while i < len && (chars[i].is_ascii_digit() || chars[i] == '-') {
                            i += 1;
                        }
                    }
                    let param: String = chars[param_start..i].iter().collect();
                    // A space delimiter is consumed as part of the control word
                    if i < len && chars[i] == ' ' {
                        i += 1;
                    }

                    // Translate known control words
                    match word.as_str() {
                        "par" | "line" | "row" => result.push('\n'),
                        "tab" | "cell" => result.push('\t'),
                        "uc" => unicode_fallback_len = param.parse().unwrap_or(1),
                        "page" | "sect" => {
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

/// Decode a byte of Windows-1252, the code page of `\ansi` RTF. It matches
/// Latin-1 except for the punctuation stored in 0x80-0x9F.
// ponytail: \ansicpgN is not read, so `\'xx` escapes in other code pages
// (Cyrillic, Greek, CJK) decode wrongly; decode with encoding_rs if needed.
fn cp1252_char(byte: u8) -> char {
    const HIGH: [char; 32] = [
        '\u{20ac}', '\u{81}', '\u{201a}', '\u{192}', '\u{201e}', '\u{2026}', '\u{2020}', '\u{2021}', '\u{2c6}',
        '\u{2030}', '\u{160}', '\u{2039}', '\u{152}', '\u{8d}', '\u{17d}', '\u{8f}', '\u{90}', '\u{2018}', '\u{2019}',
        '\u{201c}', '\u{201d}', '\u{2022}', '\u{2013}', '\u{2014}', '\u{2dc}', '\u{2122}', '\u{161}', '\u{203a}',
        '\u{153}', '\u{9d}', '\u{17e}', '\u{178}',
    ];
    match byte {
        0x80..=0x9f => HIGH[usize::from(byte - 0x80)],
        _ => char::from(byte),
    }
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

    #[test]
    fn test_extract_cp1252_punctuation() {
        let rtf = r"{\rtf1\ansi it\'92s \'93fine\'94}";
        assert_eq!(extract_text_from_rtf(rtf), "it\u{2019}s \u{201C}fine\u{201D}");
    }

    #[test]
    fn test_extract_word_unicode_with_hex_fallback() {
        // Word and LibreOffice follow each \uN with a \'xx fallback.
        let rtf = "{\\rtf1\\ansi\\ansicpg1252\\deff0{\\fonttbl{\\f0\\froman Times New Roman;}}\
                   {\\*\\generator Riched20 10.0.19041}\\viewkind4\\uc1 \r\n\
                   \\pard\\sa200\\f0\\fs22\\lang9 Chapter One\\par\r\n\
                   She said \\ldblquote hi\\rdblquote  and it\\u8217\\'92s fine.\\par\r\n}";
        assert_eq!(
            extract_text_from_rtf(rtf),
            "Chapter One\nShe said \u{201C}hi\u{201D} and it\u{2019}s fine."
        );
    }

    #[test]
    fn test_extract_textedit_line_breaks_and_uc0() {
        // TextEdit ends each line with a backslash and writes \uc0 before \uN.
        let rtf = "{\\rtf1\\ansi\\ansicpg1252\\cocoartf2761\n\
                   \\cocoatextscaling0\\cocoaplatform0{\\fonttbl\\f0\\fswiss\\fcharset0 Helvetica;}\n\
                   {\\colortbl;\\red255\\green255\\blue255;}\n\
                   {\\*\\expandedcolortbl;;}\n\
                   \\pard\\tx566\\pardirnatural\\partightenfactor0\n\
                   \n\
                   \\f0\\fs24 \\cf0 First line\\\n\
                   Price: \\uc0\\u8364 5}";
        assert_eq!(extract_text_from_rtf(rtf), "First line\nPrice: \u{20AC}5");
    }

    #[test]
    fn test_extract_control_symbols() {
        let rtf = r"{\rtf1\ansi Mr.\~Smith, co\-operate, well\_known}";
        assert_eq!(
            extract_text_from_rtf(rtf),
            "Mr.\u{a0}Smith, cooperate, well\u{2011}known"
        );
    }

    #[test]
    fn test_extract_stray_closing_brace() {
        assert_eq!(extract_text_from_rtf(r"{\rtf1 a}}b"), "ab");
    }
}
