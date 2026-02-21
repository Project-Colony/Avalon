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
}
