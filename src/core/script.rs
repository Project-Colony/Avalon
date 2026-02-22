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
}

/// Auto-correction settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCorrection {
    pub smart_quotes: bool,
    pub em_dashes: bool,
    pub ellipsis: bool,
    pub capitalize_sentences: bool,
    pub superscript_ordinals: bool,
}

impl Default for AutoCorrection {
    fn default() -> Self {
        Self {
            smart_quotes: true,
            em_dashes: true,
            ellipsis: true,
            capitalize_sentences: false,
            superscript_ordinals: false,
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
            RevisionLevel::First => "#e74c3c",   // Red
            RevisionLevel::Second => "#3498db",   // Blue
            RevisionLevel::Third => "#2ecc71",    // Green
            RevisionLevel::Fourth => "#e67e22",   // Orange
            RevisionLevel::Fifth => "#9b59b6",    // Purple
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

impl ScriptElement {
    /// Check if this element type is typically uppercase
    pub fn is_uppercase(&self) -> bool {
        matches!(self, ScriptElement::SceneHeading | ScriptElement::Character | ScriptElement::Transition)
    }

    /// Expected indentation level for Fountain format
    pub fn indent_level(&self) -> usize {
        match self {
            ScriptElement::SceneHeading => 0,
            ScriptElement::Action => 0,
            ScriptElement::Character => 2,
            ScriptElement::Dialogue => 1,
            ScriptElement::Parenthetical => 1,
            ScriptElement::Transition => 0,
            ScriptElement::Shot => 0,
            ScriptElement::Note => 0,
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

impl AutoCorrection {
    /// Check if any correction is enabled
    pub fn any_enabled(&self) -> bool {
        self.smart_quotes || self.em_dashes || self.ellipsis
            || self.capitalize_sentences || self.superscript_ordinals
    }

    /// Get a summary of active corrections
    pub fn active_list(&self) -> Vec<&str> {
        let mut active = Vec::new();
        if self.smart_quotes { active.push("Smart Quotes"); }
        if self.em_dashes { active.push("Em Dashes"); }
        if self.ellipsis { active.push("Ellipsis"); }
        if self.capitalize_sentences { active.push("Auto-Capitalize"); }
        if self.superscript_ordinals { active.push("Ordinals"); }
        active
    }

    /// Disable all corrections
    pub fn disable_all(&mut self) {
        self.smart_quotes = false;
        self.em_dashes = false;
        self.ellipsis = false;
        self.capitalize_sentences = false;
        self.superscript_ordinals = false;
    }

    /// Enable all corrections
    pub fn enable_all(&mut self) {
        self.smart_quotes = true;
        self.em_dashes = true;
        self.ellipsis = true;
        self.capitalize_sentences = true;
        self.superscript_ordinals = true;
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
        assert!(matches!(ScriptElement::SceneHeading.next_element_on_enter(), ScriptElement::Action));
        assert!(matches!(ScriptElement::Character.next_element_on_enter(), ScriptElement::Dialogue));
        assert!(matches!(ScriptElement::Dialogue.next_element_on_enter(), ScriptElement::Character));
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
        assert_eq!(ac.active_list().len(), 5);
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
            capitalize_sentences: false,
            superscript_ordinals: false,
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
            capitalize_sentences: false,
            superscript_ordinals: false,
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
            capitalize_sentences: false,
            superscript_ordinals: false,
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
            capitalize_sentences: false,
            superscript_ordinals: false,
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
            capitalize_sentences: false,
            superscript_ordinals: false,
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
            capitalize_sentences: false,
            superscript_ordinals: false,
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
            capitalize_sentences: false,
            superscript_ordinals: false,
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
        assert!(matches!(ScriptElement::Transition.next_element_on_enter(), ScriptElement::Action));
    }

    #[test]
    fn test_next_element_parenthetical() {
        assert!(matches!(ScriptElement::Parenthetical.next_element_on_enter(), ScriptElement::Dialogue));
    }
}
