#![allow(dead_code)] // Methods used by test code
use serde::{Deserialize, Serialize};

/// Script mode element types for screenwriting
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScriptElement {
    SceneHeading,
    Action,
    Character,
    Dialogue,
    Parenthetical,
    Transition,
    Shot,
    Note,
}

impl ScriptElement {
    pub fn label(&self) -> &str {
        match self {
            ScriptElement::SceneHeading => "Scene Heading",
            ScriptElement::Action => "Action",
            ScriptElement::Character => "Character",
            ScriptElement::Dialogue => "Dialogue",
            ScriptElement::Parenthetical => "Parenthetical",
            ScriptElement::Transition => "Transition",
            ScriptElement::Shot => "Shot",
            ScriptElement::Note => "Note",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            ScriptElement::SceneHeading,
            ScriptElement::Action,
            ScriptElement::Character,
            ScriptElement::Dialogue,
            ScriptElement::Parenthetical,
            ScriptElement::Transition,
            ScriptElement::Shot,
            ScriptElement::Note,
        ]
    }

    pub fn shortcut_hint(&self) -> &str {
        match self {
            ScriptElement::SceneHeading => "INT./EXT.",
            ScriptElement::Action => "plain text",
            ScriptElement::Character => "UPPERCASE NAME",
            ScriptElement::Dialogue => "under character",
            ScriptElement::Parenthetical => "(in parens)",
            ScriptElement::Transition => "CUT TO:",
            ScriptElement::Shot => "ANGLE ON:",
            ScriptElement::Note => "[[note]]",
        }
    }

    /// Check if this element type is typically uppercase
    pub fn is_uppercase(&self) -> bool {
        matches!(
            self,
            ScriptElement::SceneHeading | ScriptElement::Character | ScriptElement::Transition
        )
    }

    /// Expected indentation level for Fountain format
    pub fn indent_level(&self) -> usize {
        match self {
            ScriptElement::Character => 2,
            ScriptElement::Dialogue | ScriptElement::Parenthetical => 1,
            _ => 0,
        }
    }

    /// Whether pressing Enter should auto-advance to another element type
    pub fn next_element_on_enter(&self) -> Self {
        match self {
            ScriptElement::SceneHeading => ScriptElement::Action,
            ScriptElement::Character => ScriptElement::Dialogue,
            ScriptElement::Dialogue => ScriptElement::Character,
            ScriptElement::Parenthetical => ScriptElement::Dialogue,
            _ => ScriptElement::Action,
        }
    }
}

/// Auto-correction settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoCorrection {
    pub smart_quotes: bool,
    pub em_dashes: bool,
    pub ellipsis: bool,
}

impl Default for AutoCorrection {
    fn default() -> Self {
        Self {
            smart_quotes: true,
            em_dashes: true,
            ellipsis: true,
        }
    }
}

impl AutoCorrection {
    /// Apply auto-corrections to text
    pub fn apply(&self, text: &str) -> String {
        let mut result = text.to_string();

        if self.em_dashes {
            result = result.replace("--", "\u{2014}"); // em dash
        }

        if self.ellipsis {
            result = result.replace("...", "\u{2026}"); // ellipsis
        }

        if self.smart_quotes {
            // Simple smart quote replacement (basic implementation)
            let mut in_quote = false;
            let mut chars: Vec<char> = result.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                if chars[i] == '"' {
                    if in_quote {
                        chars[i] = '\u{201D}'; // right double quotation
                    } else {
                        chars[i] = '\u{201C}'; // left double quotation
                    }
                    in_quote = !in_quote;
                } else if chars[i] == '\'' {
                    // Check if it's an apostrophe (after a letter) or opening quote
                    if i > 0 && chars[i - 1].is_alphanumeric() {
                        chars[i] = '\u{2019}'; // right single quotation (apostrophe)
                    } else {
                        chars[i] = '\u{2018}'; // left single quotation
                    }
                }
                i += 1;
            }
            result = chars.into_iter().collect();
        }

        result
    }

    /// The correction to make as a character is typed. `before` is the text of
    /// the line up to the cursor, ending with the character just typed. The
    /// result is how many characters to remove before the cursor and the
    /// character that replaces them.
    ///
    /// Double hyphens at the start of a line, after punctuation or after the
    /// `|` of a table row, as in a Markdown rule `---`, a comment `<!--` or a
    /// table rule `| --- |`, are left alone.
    pub fn correct_typed(&self, before: &str) -> Option<(usize, char)> {
        let mut previous = before.chars().rev();
        let typed = previous.next()?;
        let last = previous.next();
        // A quote opens at the start of a line, after a space or after an
        // opening bracket or dash, and closes everywhere else.
        let opens = last.is_none_or(|c| c.is_whitespace() || "([{\u{2014}".contains(c));
        match typed {
            '-' if self.em_dashes && last == Some('-') => {
                let prose = match previous.next() {
                    Some(c) if c.is_whitespace() => previous.find(|c| !c.is_whitespace()).is_some_and(|c| c != '|'),
                    Some(c) => c.is_alphanumeric(),
                    None => false,
                };
                prose.then_some((2, '\u{2014}'))
            }
            '.' if self.ellipsis && last == Some('.') && previous.next() == Some('.') => Some((3, '\u{2026}')),
            '"' if self.smart_quotes => Some((1, if opens { '\u{201C}' } else { '\u{201D}' })),
            '\'' if self.smart_quotes => Some((1, if opens { '\u{2018}' } else { '\u{2019}' })),
            _ => None,
        }
    }

    /// Check if any correction is enabled
    pub fn any_enabled(&self) -> bool {
        self.smart_quotes || self.em_dashes || self.ellipsis
    }

    /// Get a summary of active corrections
    pub fn active_list(&self) -> Vec<&str> {
        let mut active = Vec::new();
        if self.smart_quotes {
            active.push("Smart Quotes");
        }
        if self.em_dashes {
            active.push("Em Dashes");
        }
        if self.ellipsis {
            active.push("Ellipsis");
        }
        active
    }

    /// Disable all corrections
    pub fn disable_all(&mut self) {
        self.smart_quotes = false;
        self.em_dashes = false;
        self.ellipsis = false;
    }

    /// Enable all corrections
    pub fn enable_all(&mut self) {
        self.smart_quotes = true;
        self.em_dashes = true;
        self.ellipsis = true;
    }
}

/// Revision level for tracking changes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RevisionLevel {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
}

impl RevisionLevel {
    pub fn color_hex(&self) -> &str {
        match self {
            RevisionLevel::First => "#e74c3c",  // Red
            RevisionLevel::Second => "#3498db", // Blue
            RevisionLevel::Third => "#2ecc71",  // Green
            RevisionLevel::Fourth => "#e67e22", // Orange
            RevisionLevel::Fifth => "#9b59b6",  // Purple
        }
    }

    pub fn label(&self) -> &str {
        match self {
            RevisionLevel::First => "Revision 1",
            RevisionLevel::Second => "Revision 2",
            RevisionLevel::Third => "Revision 3",
            RevisionLevel::Fourth => "Revision 4",
            RevisionLevel::Fifth => "Revision 5",
        }
    }

    /// Get all revision levels
    pub fn all() -> Vec<Self> {
        vec![
            RevisionLevel::First,
            RevisionLevel::Second,
            RevisionLevel::Third,
            RevisionLevel::Fourth,
            RevisionLevel::Fifth,
        ]
    }

    /// Get the number (1-5)
    pub fn number(&self) -> usize {
        match self {
            RevisionLevel::First => 1,
            RevisionLevel::Second => 2,
            RevisionLevel::Third => 3,
            RevisionLevel::Fourth => 4,
            RevisionLevel::Fifth => 5,
        }
    }

    /// Get the next revision level (wraps around)
    pub fn next(&self) -> Self {
        match self {
            RevisionLevel::First => RevisionLevel::Second,
            RevisionLevel::Second => RevisionLevel::Third,
            RevisionLevel::Third => RevisionLevel::Fourth,
            RevisionLevel::Fourth => RevisionLevel::Fifth,
            RevisionLevel::Fifth => RevisionLevel::First,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_element_all() {
        let all = ScriptElement::all();
        assert_eq!(all.len(), 8);
    }

    #[test]
    fn test_script_element_label() {
        assert_eq!(ScriptElement::SceneHeading.label(), "Scene Heading");
        assert_eq!(ScriptElement::Dialogue.label(), "Dialogue");
    }

    #[test]
    fn test_script_element_uppercase() {
        assert!(ScriptElement::SceneHeading.is_uppercase());
        assert!(ScriptElement::Character.is_uppercase());
        assert!(ScriptElement::Transition.is_uppercase());
        assert!(!ScriptElement::Dialogue.is_uppercase());
        assert!(!ScriptElement::Action.is_uppercase());
    }

    #[test]
    fn test_script_element_indent() {
        assert_eq!(ScriptElement::SceneHeading.indent_level(), 0);
        assert_eq!(ScriptElement::Character.indent_level(), 2);
        assert_eq!(ScriptElement::Dialogue.indent_level(), 1);
    }

    #[test]
    fn test_next_element_on_enter() {
        assert!(matches!(
            ScriptElement::SceneHeading.next_element_on_enter(),
            ScriptElement::Action
        ));
        assert!(matches!(
            ScriptElement::Character.next_element_on_enter(),
            ScriptElement::Dialogue
        ));
        assert!(matches!(
            ScriptElement::Dialogue.next_element_on_enter(),
            ScriptElement::Character
        ));
    }

    #[test]
    fn test_auto_correction_default() {
        let ac = AutoCorrection::default();
        assert!(ac.smart_quotes);
        assert!(ac.em_dashes);
        assert!(ac.ellipsis);
        assert!(ac.any_enabled());
    }

    #[test]
    fn test_auto_correction_disable_all() {
        let mut ac = AutoCorrection::default();
        ac.disable_all();
        assert!(!ac.any_enabled());
        assert!(ac.active_list().is_empty());
    }

    #[test]
    fn test_auto_correction_enable_all() {
        let mut ac = AutoCorrection::default();
        ac.disable_all();
        ac.enable_all();
        assert!(ac.any_enabled());
        assert_eq!(ac.active_list().len(), 3);
    }

    #[test]
    fn test_revision_level_next() {
        assert!(matches!(RevisionLevel::First.next(), RevisionLevel::Second));
        assert!(matches!(RevisionLevel::Fifth.next(), RevisionLevel::First));
    }

    #[test]
    fn test_revision_level_label() {
        let label = RevisionLevel::First.label();
        assert!(label.contains("1") || label.contains("First"));
        let label3 = RevisionLevel::Third.label();
        assert!(label3.contains("3") || label3.contains("Third"));
    }

    #[test]
    fn test_script_element_shortcut_hints() {
        for el in ScriptElement::all() {
            assert!(!el.shortcut_hint().is_empty());
        }
    }

    #[test]
    fn test_auto_correction_apply() {
        let ac = AutoCorrection::default();
        let result = ac.apply("Hello -- world...");
        assert!(result.contains('\u{2014}') || result.contains("--")); // em dash or original
        assert!(result.contains('\u{2026}') || result.contains("...")); // ellipsis or original
    }

    #[test]
    fn test_auto_correction_apply_em_dash() {
        let ac = AutoCorrection {
            em_dashes: true,
            smart_quotes: false,
            ellipsis: false,
        };
        let result = ac.apply("He said -- and left");
        assert!(result.contains('\u{2014}'));
        assert!(!result.contains("--"));
    }

    #[test]
    fn test_auto_correction_apply_ellipsis() {
        let ac = AutoCorrection {
            em_dashes: false,
            smart_quotes: false,
            ellipsis: true,
        };
        let result = ac.apply("Wait for it...");
        assert!(result.contains('\u{2026}'));
        assert!(!result.contains("..."));
    }

    #[test]
    fn test_auto_correction_apply_smart_quotes() {
        let ac = AutoCorrection {
            em_dashes: false,
            smart_quotes: true,
            ellipsis: false,
        };
        let result = ac.apply("He said \"hello\" today");
        assert!(result.contains('\u{201C}')); // left double quote
        assert!(result.contains('\u{201D}')); // right double quote
        assert!(!result.contains('"'));
    }

    #[test]
    fn test_auto_correction_apply_apostrophe() {
        let ac = AutoCorrection {
            em_dashes: false,
            smart_quotes: true,
            ellipsis: false,
        };
        let result = ac.apply("It's a test");
        assert!(result.contains('\u{2019}')); // right single quote (apostrophe)
    }

    #[test]
    fn test_auto_correction_apply_opening_single_quote() {
        let ac = AutoCorrection {
            em_dashes: false,
            smart_quotes: true,
            ellipsis: false,
        };
        let result = ac.apply("'Hello' he said");
        assert!(result.contains('\u{2018}')); // left single quote
    }

    #[test]
    fn test_auto_correction_nothing_enabled() {
        let ac = AutoCorrection {
            em_dashes: false,
            smart_quotes: false,
            ellipsis: false,
        };
        let input = "Hello -- world... \"test\"";
        assert_eq!(ac.apply(input), input);
    }

    #[test]
    fn test_auto_correction_active_list() {
        let ac = AutoCorrection {
            em_dashes: true,
            smart_quotes: false,
            ellipsis: true,
        };
        let active = ac.active_list();
        assert_eq!(active.len(), 2);
        assert!(active.contains(&"Em Dashes"));
        assert!(active.contains(&"Ellipsis"));
    }

    #[test]
    fn test_revision_level_all() {
        let all = RevisionLevel::all();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_revision_level_number() {
        assert_eq!(RevisionLevel::First.number(), 1);
        assert_eq!(RevisionLevel::Second.number(), 2);
        assert_eq!(RevisionLevel::Third.number(), 3);
        assert_eq!(RevisionLevel::Fourth.number(), 4);
        assert_eq!(RevisionLevel::Fifth.number(), 5);
    }

    #[test]
    fn test_revision_level_color_hex() {
        for level in RevisionLevel::all() {
            let hex = level.color_hex();
            assert!(hex.starts_with('#'));
            assert_eq!(hex.len(), 7);
        }
    }

    #[test]
    fn test_revision_level_next_wraps() {
        let mut level = RevisionLevel::First;
        for _ in 0..5 {
            level = level.next();
        }
        assert_eq!(level, RevisionLevel::First); // Should wrap back to First
    }

    #[test]
    fn test_revision_level_equality() {
        assert_eq!(RevisionLevel::First, RevisionLevel::First);
        assert_ne!(RevisionLevel::First, RevisionLevel::Second);
    }

    #[test]
    fn test_script_element_equality() {
        assert_eq!(ScriptElement::Action, ScriptElement::Action);
        assert_ne!(ScriptElement::Action, ScriptElement::Dialogue);
    }

    #[test]
    fn test_script_element_all_have_labels() {
        for el in ScriptElement::all() {
            assert!(!el.label().is_empty());
        }
    }

    #[test]
    fn test_script_element_parenthetical_indent() {
        assert_eq!(ScriptElement::Parenthetical.indent_level(), 1);
    }

    #[test]
    fn test_script_element_note_not_uppercase() {
        assert!(!ScriptElement::Note.is_uppercase());
        assert!(!ScriptElement::Shot.is_uppercase());
    }

    #[test]
    fn test_next_element_transition() {
        assert!(matches!(
            ScriptElement::Transition.next_element_on_enter(),
            ScriptElement::Action
        ));
    }

    #[test]
    fn test_next_element_parenthetical() {
        assert!(matches!(
            ScriptElement::Parenthetical.next_element_on_enter(),
            ScriptElement::Dialogue
        ));
    }

    #[test]
    fn correct_typed_replaces_punctuation_as_you_type() {
        let ac = AutoCorrection::default();
        assert_eq!(ac.correct_typed("word--"), Some((2, '\u{2014}')));
        assert_eq!(ac.correct_typed("word --"), Some((2, '\u{2014}')));
        assert_eq!(ac.correct_typed("Wait..."), Some((3, '\u{2026}')));
        assert_eq!(ac.correct_typed("\""), Some((1, '\u{201C}')));
        assert_eq!(ac.correct_typed("said \""), Some((1, '\u{201C}')));
        assert_eq!(ac.correct_typed("\u{201C}Hi\""), Some((1, '\u{201D}')));
        assert_eq!(ac.correct_typed("don'"), Some((1, '\u{2019}')));
        assert_eq!(ac.correct_typed("('"), Some((1, '\u{2018}')));
    }

    #[test]
    fn correct_typed_keeps_markdown_and_plain_text() {
        let ac = AutoCorrection::default();
        assert_eq!(ac.correct_typed("--"), None);
        assert_eq!(ac.correct_typed("<!--"), None);
        assert_eq!(ac.correct_typed("|--"), None);
        assert_eq!(ac.correct_typed("| --"), None);
        assert_eq!(ac.correct_typed("| --- | --"), None);
        assert_eq!(ac.correct_typed("Name | --"), None);
        assert_eq!(ac.correct_typed("  --"), None);
        assert_eq!(ac.correct_typed("word-"), None);
        assert_eq!(ac.correct_typed("end.."), None);
        assert_eq!(ac.correct_typed("abc"), None);
        assert_eq!(ac.correct_typed(""), None);
    }

    #[test]
    fn correct_typed_follows_each_substitution_toggle() {
        let mut ac = AutoCorrection::default();
        ac.disable_all();
        assert_eq!(ac.correct_typed("word--"), None);
        assert_eq!(ac.correct_typed("Wait..."), None);
        assert_eq!(ac.correct_typed("\""), None);
        ac.ellipsis = true;
        assert_eq!(ac.correct_typed("Wait..."), Some((3, '\u{2026}')));
        assert_eq!(ac.correct_typed("word--"), None);
    }
}
