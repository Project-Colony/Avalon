use std::fmt::Write;
use anyhow::Result;

use super::compiler::{self, CompileContent, CompileOptions};

/// FinalDraft paragraph element types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FdxParagraphType {
    SceneHeading,
    Action,
    Character,
    Dialogue,
    Parenthetical,
    Transition,
    Shot,
    General,
}

impl FdxParagraphType {
    /// Human-readable label for this paragraph type
    pub fn label(&self) -> &str {
        match self {
            FdxParagraphType::SceneHeading => "Scene Heading",
            FdxParagraphType::Action => "Action",
            FdxParagraphType::Character => "Character",
            FdxParagraphType::Dialogue => "Dialogue",
            FdxParagraphType::Parenthetical => "Parenthetical",
            FdxParagraphType::Transition => "Transition",
            FdxParagraphType::Shot => "Shot",
            FdxParagraphType::General => "General",
        }
    }

    /// FDX XML Type attribute value for this paragraph type
    pub fn to_fdx_type(&self) -> &str {
        match self {
            FdxParagraphType::SceneHeading => "Scene Heading",
            FdxParagraphType::Action => "Action",
            FdxParagraphType::Character => "Character",
            FdxParagraphType::Dialogue => "Dialogue",
            FdxParagraphType::Parenthetical => "Parenthetical",
            FdxParagraphType::Transition => "Transition",
            FdxParagraphType::Shot => "Shot",
            FdxParagraphType::General => "General",
        }
    }
}

/// A single paragraph element in the FDX document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FdxParagraph {
    pub paragraph_type: FdxParagraphType,
    pub text: String,
}

impl FdxParagraph {
    /// Create a new FDX paragraph
    pub fn new(paragraph_type: FdxParagraphType, text: &str) -> Self {
        Self {
            paragraph_type,
            text: text.to_string(),
        }
    }

    /// Render this paragraph as an FDX XML element
    pub fn to_xml(&self) -> String {
        format!(
            "    <Paragraph Type=\"{}\">\n      <Text>{}</Text>\n    </Paragraph>",
            self.paragraph_type.to_fdx_type(),
            escape_xml(&self.text),
        )
    }
}

pub fn escape_xml(text: &str) -> String { compiler::escape_xml(text) }

/// Compile content to FinalDraft (.fdx) XML format
pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let mut output = String::new();

    // XML declaration and root element
    output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    output.push_str("<FinalDraft DocumentType=\"Script\" Template=\"No\" Version=\"1\">\n");

    // Content section
    output.push_str("  <Content>\n");

    for content in contents {
        if content.is_folder {
            // Folders become scene heading paragraphs
            let para = FdxParagraph::new(
                FdxParagraphType::SceneHeading,
                &content.title.to_uppercase(),
            );
            output.push_str(&para.to_xml());
            output.push('\n');
        } else if !content.text.is_empty() {
            let paragraphs = parse_text_to_fdx_paragraphs(&content.text);
            for para in &paragraphs {
                output.push_str(&para.to_xml());
                output.push('\n');
            }
        }
    }

    output.push_str("  </Content>\n");

    // Title page section
    if options.include_front_matter {
        output.push_str("  <TitlePage>\n");
        output.push_str("    <Content>\n");

        if !options.title.is_empty() {
            let _ = write!(
                output,
                "      <Paragraph Type=\"Title\">\n        <Text>{}</Text>\n      </Paragraph>\n",
                escape_xml(&options.title),
            );
        }

        if !options.author.is_empty() {
            let _ = write!(
                output,
                "      <Paragraph Type=\"Author\">\n        <Text>Written by {}</Text>\n      </Paragraph>\n",
                escape_xml(&options.author),
            );
        }

        output.push_str("    </Content>\n");
        output.push_str("  </TitlePage>\n");
    }

    output.push_str("</FinalDraft>\n");

    Ok(output)
}

/// Parse prose text into FDX paragraph elements by detecting screenplay formatting.
///
/// Detection rules:
/// - Lines starting with INT., EXT., EST., I/E., or a forced . prefix are Scene Headings
/// - Lines ending with TO: or matching common transitions (FADE OUT., FADE IN:) are Transitions
/// - Lines starting with > are Transitions (Fountain convention)
/// - Lines in ALL CAPS (with at least 2 alpha characters) followed by dialogue are Characters
/// - Lines in parentheses following a character cue are Parentheticals
/// - Lines following a character cue (that are not parentheticals) are Dialogue
/// - Lines matching SHOT patterns (e.g., "ANGLE ON", "CLOSE ON") are Shots
/// - Everything else is Action
pub fn parse_text_to_fdx_paragraphs(text: &str) -> Vec<FdxParagraph> {
    let lines: Vec<&str> = text.lines().collect();
    let mut paragraphs = Vec::new();
    let mut i = 0;
    let mut in_dialogue_block = false;

    while i < lines.len() {
        let line = lines[i].trim();

        // Skip empty lines and reset dialogue state
        if line.is_empty() {
            in_dialogue_block = false;
            i += 1;
            continue;
        }

        // Scene heading detection
        if is_scene_heading(line) {
            in_dialogue_block = false;
            paragraphs.push(FdxParagraph::new(FdxParagraphType::SceneHeading, line));
            i += 1;
            continue;
        }

        // Transition detection
        if is_transition(line) {
            in_dialogue_block = false;
            paragraphs.push(FdxParagraph::new(FdxParagraphType::Transition, line));
            i += 1;
            continue;
        }

        // Shot detection
        if is_shot(line) {
            in_dialogue_block = false;
            paragraphs.push(FdxParagraph::new(FdxParagraphType::Shot, line));
            i += 1;
            continue;
        }

        // If we are inside a dialogue block (after a Character element)
        if in_dialogue_block {
            if is_parenthetical(line) {
                paragraphs.push(FdxParagraph::new(FdxParagraphType::Parenthetical, line));
            } else {
                paragraphs.push(FdxParagraph::new(FdxParagraphType::Dialogue, line));
            }
            i += 1;
            continue;
        }

        // Character cue detection: ALL CAPS line followed by non-empty line
        if is_character_cue(line) && i + 1 < lines.len() && !lines[i + 1].trim().is_empty() {
            paragraphs.push(FdxParagraph::new(FdxParagraphType::Character, line));
            in_dialogue_block = true;
            i += 1;
            continue;
        }

        // Default to Action
        paragraphs.push(FdxParagraph::new(FdxParagraphType::Action, line));
        i += 1;
    }

    paragraphs
}

/// Check if a line is a scene heading (INT., EXT., EST., I/E., or forced with .)
fn is_scene_heading(line: &str) -> bool {
    let upper = line.to_uppercase();
    upper.starts_with("INT.")
        || upper.starts_with("INT ")
        || upper.starts_with("EXT.")
        || upper.starts_with("EXT ")
        || upper.starts_with("EST.")
        || upper.starts_with("I/E.")
        || upper.starts_with("I/E ")
        || (line.starts_with('.') && line.len() > 1 && !line.starts_with(".."))
}

/// Check if a line is a transition (ends with TO:, or common transitions)
fn is_transition(line: &str) -> bool {
    let upper = line.trim().to_uppercase();
    upper.ends_with("TO:")
        || upper == "FADE OUT."
        || upper == "FADE IN:"
        || upper == "SMASH CUT:"
        || (line.starts_with('>') && !line.ends_with('<'))
}

/// Check if a line looks like a character cue (ALL CAPS, not a heading/transition/shot)
fn is_character_cue(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.len() < 2 {
        return false;
    }
    if is_scene_heading(trimmed) || is_transition(trimmed) || is_shot(trimmed) {
        return false;
    }
    let alpha_chars: Vec<char> = trimmed.chars().filter(|c| c.is_alphabetic()).collect();
    !alpha_chars.is_empty() && alpha_chars.iter().all(|c| c.is_uppercase())
}

/// Check if a line is a parenthetical
fn is_parenthetical(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('(') && trimmed.ends_with(')')
}

/// Check if a line is a shot description (ANGLE ON, CLOSE ON, etc.)
fn is_shot(line: &str) -> bool {
    let upper = line.trim().to_uppercase();
    upper.starts_with("ANGLE ON")
        || upper.starts_with("CLOSE ON")
        || upper.starts_with("WIDE ON")
        || upper.starts_with("POV")
        || upper.starts_with("EXTREME CLOSE")
        || upper.starts_with("INSERT ")
        || upper.starts_with("INTERCUT")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::compiler::{OutputFormat, SeparatorType};

    fn make_opts() -> CompileOptions {
        CompileOptions {
            format: OutputFormat::Fountain,
            title: "Test Screenplay".to_string(),
            author: "Test Writer".to_string(),
            include_front_matter: false,
            separator: SeparatorType::EmptyLine,
            page_break_between_folders: false,
            compile_marked_only: false,
            font_size: 12.0,
            font_family: "Courier".to_string(),
            include_toc: false,
            replace_placeholders: false,
        }
    }

    fn make_content(title: &str, text: &str, is_folder: bool) -> CompileContent {
        CompileContent {
            title: title.to_string(),
            text: text.to_string(),
            depth: 0,
            is_folder,
        }
    }

    // ---- escape_xml tests ----

    #[test]
    fn test_escape_xml_ampersand() {
        assert_eq!(escape_xml("Tom & Jerry"), "Tom &amp; Jerry");
    }

    #[test]
    fn test_escape_xml_less_than() {
        assert_eq!(escape_xml("a < b"), "a &lt; b");
    }

    #[test]
    fn test_escape_xml_greater_than() {
        assert_eq!(escape_xml("a > b"), "a &gt; b");
    }

    #[test]
    fn test_escape_xml_double_quote() {
        assert_eq!(escape_xml("say \"hello\""), "say &quot;hello&quot;");
    }

    #[test]
    fn test_escape_xml_single_quote() {
        assert_eq!(escape_xml("it's"), "it&apos;s");
    }

    #[test]
    fn test_escape_xml_all_entities() {
        let result = escape_xml("A & B < C > D \"E\" 'F'");
        assert_eq!(result, "A &amp; B &lt; C &gt; D &quot;E&quot; &apos;F&apos;");
    }

    #[test]
    fn test_escape_xml_no_special_chars() {
        assert_eq!(escape_xml("plain text"), "plain text");
    }

    #[test]
    fn test_escape_xml_empty_string() {
        assert_eq!(escape_xml(""), "");
    }

    // ---- FdxParagraphType tests ----

    #[test]
    fn test_paragraph_type_labels() {
        assert_eq!(FdxParagraphType::SceneHeading.label(), "Scene Heading");
        assert_eq!(FdxParagraphType::Action.label(), "Action");
        assert_eq!(FdxParagraphType::Character.label(), "Character");
        assert_eq!(FdxParagraphType::Dialogue.label(), "Dialogue");
        assert_eq!(FdxParagraphType::Parenthetical.label(), "Parenthetical");
        assert_eq!(FdxParagraphType::Transition.label(), "Transition");
        assert_eq!(FdxParagraphType::Shot.label(), "Shot");
        assert_eq!(FdxParagraphType::General.label(), "General");
    }

    #[test]
    fn test_paragraph_type_fdx_type() {
        assert_eq!(FdxParagraphType::SceneHeading.to_fdx_type(), "Scene Heading");
        assert_eq!(FdxParagraphType::Action.to_fdx_type(), "Action");
        assert_eq!(FdxParagraphType::Character.to_fdx_type(), "Character");
        assert_eq!(FdxParagraphType::Dialogue.to_fdx_type(), "Dialogue");
        assert_eq!(FdxParagraphType::Parenthetical.to_fdx_type(), "Parenthetical");
        assert_eq!(FdxParagraphType::Transition.to_fdx_type(), "Transition");
        assert_eq!(FdxParagraphType::Shot.to_fdx_type(), "Shot");
        assert_eq!(FdxParagraphType::General.to_fdx_type(), "General");
    }

    #[test]
    fn test_paragraph_type_equality() {
        assert_eq!(FdxParagraphType::Action, FdxParagraphType::Action);
        assert_ne!(FdxParagraphType::Action, FdxParagraphType::Dialogue);
    }

    // ---- FdxParagraph tests ----

    #[test]
    fn test_fdx_paragraph_new() {
        let p = FdxParagraph::new(FdxParagraphType::Action, "John walks in.");
        assert_eq!(p.paragraph_type, FdxParagraphType::Action);
        assert_eq!(p.text, "John walks in.");
    }

    #[test]
    fn test_fdx_paragraph_to_xml_action() {
        let p = FdxParagraph::new(FdxParagraphType::Action, "John walks in.");
        let xml = p.to_xml();
        assert!(xml.contains("Type=\"Action\""));
        assert!(xml.contains("<Text>John walks in.</Text>"));
    }

    #[test]
    fn test_fdx_paragraph_to_xml_scene_heading() {
        let p = FdxParagraph::new(FdxParagraphType::SceneHeading, "INT. OFFICE - DAY");
        let xml = p.to_xml();
        assert!(xml.contains("Type=\"Scene Heading\""));
        assert!(xml.contains("<Text>INT. OFFICE - DAY</Text>"));
    }

    #[test]
    fn test_fdx_paragraph_to_xml_escapes_entities() {
        let p = FdxParagraph::new(FdxParagraphType::Dialogue, "Tom & Jerry say \"hello\"");
        let xml = p.to_xml();
        assert!(xml.contains("Tom &amp; Jerry say &quot;hello&quot;"));
    }

    #[test]
    fn test_fdx_paragraph_to_xml_character() {
        let p = FdxParagraph::new(FdxParagraphType::Character, "JOHN");
        let xml = p.to_xml();
        assert!(xml.contains("Type=\"Character\""));
        assert!(xml.contains("<Text>JOHN</Text>"));
    }

    #[test]
    fn test_fdx_paragraph_to_xml_dialogue() {
        let p = FdxParagraph::new(FdxParagraphType::Dialogue, "Hello, world.");
        let xml = p.to_xml();
        assert!(xml.contains("Type=\"Dialogue\""));
        assert!(xml.contains("<Text>Hello, world.</Text>"));
    }

    #[test]
    fn test_fdx_paragraph_to_xml_parenthetical() {
        let p = FdxParagraph::new(FdxParagraphType::Parenthetical, "(whispering)");
        let xml = p.to_xml();
        assert!(xml.contains("Type=\"Parenthetical\""));
        assert!(xml.contains("<Text>(whispering)</Text>"));
    }

    #[test]
    fn test_fdx_paragraph_to_xml_transition() {
        let p = FdxParagraph::new(FdxParagraphType::Transition, "CUT TO:");
        let xml = p.to_xml();
        assert!(xml.contains("Type=\"Transition\""));
        assert!(xml.contains("<Text>CUT TO:</Text>"));
    }

    #[test]
    fn test_fdx_paragraph_to_xml_shot() {
        let p = FdxParagraph::new(FdxParagraphType::Shot, "ANGLE ON the door");
        let xml = p.to_xml();
        assert!(xml.contains("Type=\"Shot\""));
        assert!(xml.contains("<Text>ANGLE ON the door</Text>"));
    }

    // ---- parse_text_to_fdx_paragraphs tests ----

    #[test]
    fn test_parse_scene_heading() {
        let paras = parse_text_to_fdx_paragraphs("INT. OFFICE - DAY");
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].paragraph_type, FdxParagraphType::SceneHeading);
        assert_eq!(paras[0].text, "INT. OFFICE - DAY");
    }

    #[test]
    fn test_parse_ext_scene_heading() {
        let paras = parse_text_to_fdx_paragraphs("EXT. PARK - NIGHT");
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].paragraph_type, FdxParagraphType::SceneHeading);
    }

    #[test]
    fn test_parse_forced_scene_heading() {
        let paras = parse_text_to_fdx_paragraphs(".FLASHBACK");
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].paragraph_type, FdxParagraphType::SceneHeading);
    }

    #[test]
    fn test_parse_action_text() {
        let paras = parse_text_to_fdx_paragraphs("John walks into the room.");
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].paragraph_type, FdxParagraphType::Action);
        assert_eq!(paras[0].text, "John walks into the room.");
    }

    #[test]
    fn test_parse_character_and_dialogue() {
        let text = "JOHN\nHello, world.";
        let paras = parse_text_to_fdx_paragraphs(text);
        assert_eq!(paras.len(), 2);
        assert_eq!(paras[0].paragraph_type, FdxParagraphType::Character);
        assert_eq!(paras[0].text, "JOHN");
        assert_eq!(paras[1].paragraph_type, FdxParagraphType::Dialogue);
        assert_eq!(paras[1].text, "Hello, world.");
    }

    #[test]
    fn test_parse_character_with_parenthetical() {
        let text = "MARY\n(softly)\nI love you.";
        let paras = parse_text_to_fdx_paragraphs(text);
        assert_eq!(paras.len(), 3);
        assert_eq!(paras[0].paragraph_type, FdxParagraphType::Character);
        assert_eq!(paras[1].paragraph_type, FdxParagraphType::Parenthetical);
        assert_eq!(paras[1].text, "(softly)");
        assert_eq!(paras[2].paragraph_type, FdxParagraphType::Dialogue);
        assert_eq!(paras[2].text, "I love you.");
    }

    #[test]
    fn test_parse_transition_cut_to() {
        let paras = parse_text_to_fdx_paragraphs("CUT TO:");
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].paragraph_type, FdxParagraphType::Transition);
    }

    #[test]
    fn test_parse_transition_fade_out() {
        let paras = parse_text_to_fdx_paragraphs("FADE OUT.");
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].paragraph_type, FdxParagraphType::Transition);
    }

    #[test]
    fn test_parse_transition_fade_in() {
        let paras = parse_text_to_fdx_paragraphs("FADE IN:");
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].paragraph_type, FdxParagraphType::Transition);
    }

    #[test]
    fn test_parse_shot_angle_on() {
        let paras = parse_text_to_fdx_paragraphs("ANGLE ON the broken window");
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].paragraph_type, FdxParagraphType::Shot);
    }

    #[test]
    fn test_parse_shot_close_on() {
        let paras = parse_text_to_fdx_paragraphs("CLOSE ON John's face");
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].paragraph_type, FdxParagraphType::Shot);
    }

    #[test]
    fn test_parse_shot_intercut() {
        let paras = parse_text_to_fdx_paragraphs("INTERCUT phone conversation");
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].paragraph_type, FdxParagraphType::Shot);
    }

    #[test]
    fn test_parse_empty_lines_skipped() {
        let text = "Action line.\n\n\n\nAnother action.";
        let paras = parse_text_to_fdx_paragraphs(text);
        assert_eq!(paras.len(), 2);
        assert_eq!(paras[0].paragraph_type, FdxParagraphType::Action);
        assert_eq!(paras[1].paragraph_type, FdxParagraphType::Action);
    }

    #[test]
    fn test_parse_dialogue_block_reset_by_empty_line() {
        let text = "JOHN\nHello.\n\nSome action here.";
        let paras = parse_text_to_fdx_paragraphs(text);
        assert_eq!(paras.len(), 3);
        assert_eq!(paras[0].paragraph_type, FdxParagraphType::Character);
        assert_eq!(paras[1].paragraph_type, FdxParagraphType::Dialogue);
        assert_eq!(paras[2].paragraph_type, FdxParagraphType::Action);
    }

    #[test]
    fn test_parse_full_screenplay_snippet() {
        let text = "\
INT. OFFICE - DAY

John walks into the room.

JOHN
(nervously)
Is anyone here?

CUT TO:

EXT. PARK - NIGHT

MARY
Yes, I am.";
        let paras = parse_text_to_fdx_paragraphs(text);

        // INT. OFFICE - DAY -> SceneHeading
        assert_eq!(paras[0].paragraph_type, FdxParagraphType::SceneHeading);
        // John walks into the room. -> Action
        assert_eq!(paras[1].paragraph_type, FdxParagraphType::Action);
        // JOHN -> Character
        assert_eq!(paras[2].paragraph_type, FdxParagraphType::Character);
        // (nervously) -> Parenthetical
        assert_eq!(paras[3].paragraph_type, FdxParagraphType::Parenthetical);
        // Is anyone here? -> Dialogue
        assert_eq!(paras[4].paragraph_type, FdxParagraphType::Dialogue);
        // CUT TO: -> Transition
        assert_eq!(paras[5].paragraph_type, FdxParagraphType::Transition);
        // EXT. PARK - NIGHT -> SceneHeading
        assert_eq!(paras[6].paragraph_type, FdxParagraphType::SceneHeading);
        // MARY -> Character
        assert_eq!(paras[7].paragraph_type, FdxParagraphType::Character);
        // Yes, I am. -> Dialogue
        assert_eq!(paras[8].paragraph_type, FdxParagraphType::Dialogue);
    }

    #[test]
    fn test_parse_empty_input() {
        let paras = parse_text_to_fdx_paragraphs("");
        assert!(paras.is_empty());
    }

    #[test]
    fn test_parse_only_blank_lines() {
        let paras = parse_text_to_fdx_paragraphs("\n\n\n");
        assert!(paras.is_empty());
    }

    // ---- compile tests ----

    #[test]
    fn test_compile_xml_declaration() {
        let result = compile(&[], &make_opts()).unwrap();
        assert!(result.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    }

    #[test]
    fn test_compile_root_element() {
        let result = compile(&[], &make_opts()).unwrap();
        assert!(result.contains("<FinalDraft DocumentType=\"Script\" Template=\"No\" Version=\"1\">"));
        assert!(result.contains("</FinalDraft>"));
    }

    #[test]
    fn test_compile_content_section() {
        let result = compile(&[], &make_opts()).unwrap();
        assert!(result.contains("<Content>"));
        assert!(result.contains("</Content>"));
    }

    #[test]
    fn test_compile_no_title_page_when_disabled() {
        let opts = make_opts();
        let result = compile(&[], &opts).unwrap();
        assert!(!result.contains("<TitlePage>"));
    }

    #[test]
    fn test_compile_title_page_when_enabled() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        let result = compile(&[], &opts).unwrap();
        assert!(result.contains("<TitlePage>"));
        assert!(result.contains("</TitlePage>"));
        assert!(result.contains("Test Screenplay"));
        assert!(result.contains("Written by Test Writer"));
    }

    #[test]
    fn test_compile_title_page_with_empty_author() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        opts.author = String::new();
        let result = compile(&[], &opts).unwrap();
        assert!(result.contains("<TitlePage>"));
        assert!(!result.contains("Written by"));
    }

    #[test]
    fn test_compile_title_page_with_empty_title() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        opts.title = String::new();
        let result = compile(&[], &opts).unwrap();
        assert!(result.contains("<TitlePage>"));
        assert!(!result.contains("Type=\"Title\""));
    }

    #[test]
    fn test_compile_with_action_content() {
        let contents = vec![make_content("Scene 1", "John walks in.", false)];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("Type=\"Action\""));
        assert!(result.contains("<Text>John walks in.</Text>"));
    }

    #[test]
    fn test_compile_with_scene_heading_content() {
        let contents = vec![make_content("Scene 1", "INT. OFFICE - DAY", false)];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("Type=\"Scene Heading\""));
        assert!(result.contains("<Text>INT. OFFICE - DAY</Text>"));
    }

    #[test]
    fn test_compile_folder_as_scene_heading() {
        let contents = vec![make_content("ACT ONE", "", true)];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("Type=\"Scene Heading\""));
        assert!(result.contains("ACT ONE"));
    }

    #[test]
    fn test_compile_with_dialogue() {
        let contents = vec![make_content("Scene", "JOHN\nHello, world.", false)];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("Type=\"Character\""));
        assert!(result.contains("<Text>JOHN</Text>"));
        assert!(result.contains("Type=\"Dialogue\""));
        assert!(result.contains("<Text>Hello, world.</Text>"));
    }

    #[test]
    fn test_compile_escapes_xml_in_content() {
        let contents = vec![make_content("Scene", "Tom & Jerry <run>", false)];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("Tom &amp; Jerry &lt;run&gt;"));
    }

    #[test]
    fn test_compile_escapes_xml_in_title_page() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        opts.title = "A & B".to_string();
        opts.author = "C <D>".to_string();
        let result = compile(&[], &opts).unwrap();
        assert!(result.contains("A &amp; B"));
        assert!(result.contains("C &lt;D&gt;"));
    }

    #[test]
    fn test_compile_empty_content_text_skipped() {
        let contents = vec![make_content("Empty Scene", "", false)];
        let result = compile(&contents, &make_opts()).unwrap();
        // Should have Content tags but no Paragraph elements for empty text
        assert!(result.contains("<Content>"));
        assert!(result.contains("</Content>"));
        // The content section should be empty (no Paragraph inside it beyond the tags)
        let content_section = &result[result.find("<Content>").unwrap()..result.find("</Content>").unwrap()];
        assert!(!content_section.contains("<Paragraph"));
    }

    #[test]
    fn test_compile_multiple_contents() {
        let contents = vec![
            make_content("Act I", "", true),
            make_content("Scene 1", "INT. OFFICE - DAY\n\nJOHN\nHello.", false),
            make_content("Scene 2", "EXT. PARK - NIGHT\n\nAction text.", false),
        ];
        let result = compile(&contents, &make_opts()).unwrap();
        // Should contain both scenes
        assert!(result.contains("INT. OFFICE - DAY"));
        assert!(result.contains("EXT. PARK - NIGHT"));
        assert!(result.contains("ACT I")); // folder uppercased
    }

    #[test]
    fn test_compile_full_document_structure() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        let contents = vec![
            make_content("Scene 1", "INT. OFFICE - DAY\n\nJOHN\nHello, world.", false),
        ];
        let result = compile(&contents, &opts).unwrap();

        // Verify overall document structure
        assert!(result.starts_with("<?xml"));
        assert!(result.contains("<FinalDraft"));
        assert!(result.contains("<Content>"));
        assert!(result.contains("</Content>"));
        assert!(result.contains("<TitlePage>"));
        assert!(result.contains("</TitlePage>"));
        assert!(result.contains("</FinalDraft>"));

        // Verify content appears in right order: FinalDraft > Content > paragraphs
        let content_start = result.find("<Content>").unwrap();
        let content_end = result.find("</Content>").unwrap();
        let title_start = result.find("<TitlePage>").unwrap();

        // Content section comes before TitlePage section
        assert!(content_start < title_start);
        // Paragraph elements are inside Content section
        let content_block = &result[content_start..content_end];
        assert!(content_block.contains("<Paragraph"));
    }

    // ---- Detection helper tests ----

    #[test]
    fn test_is_scene_heading_int_dot() {
        assert!(is_scene_heading("INT. OFFICE - DAY"));
    }

    #[test]
    fn test_is_scene_heading_int_space() {
        assert!(is_scene_heading("INT OFFICE - DAY"));
    }

    #[test]
    fn test_is_scene_heading_ext() {
        assert!(is_scene_heading("EXT. PARK - NIGHT"));
    }

    #[test]
    fn test_is_scene_heading_est() {
        assert!(is_scene_heading("EST. CITY SKYLINE"));
    }

    #[test]
    fn test_is_scene_heading_ie() {
        assert!(is_scene_heading("I/E. CAR - MOVING"));
        assert!(is_scene_heading("I/E CAR"));
    }

    #[test]
    fn test_is_scene_heading_forced() {
        assert!(is_scene_heading(".FLASHBACK"));
    }

    #[test]
    fn test_is_scene_heading_not_ellipsis() {
        assert!(!is_scene_heading("..trailing off"));
    }

    #[test]
    fn test_is_scene_heading_false() {
        assert!(!is_scene_heading("Regular text"));
        assert!(!is_scene_heading(""));
    }

    #[test]
    fn test_is_transition_true() {
        assert!(is_transition("CUT TO:"));
        assert!(is_transition("FADE OUT."));
        assert!(is_transition("FADE IN:"));
        assert!(is_transition("SMASH CUT:"));
        assert!(is_transition("DISSOLVE TO:"));
    }

    #[test]
    fn test_is_transition_false() {
        assert!(!is_transition("Regular text"));
        assert!(!is_transition("JOHN")); // Character, not transition
    }

    #[test]
    fn test_is_character_cue_true() {
        assert!(is_character_cue("JOHN"));
        assert!(is_character_cue("MARY JANE"));
        assert!(is_character_cue("JOHN (V.O.)"));
    }

    #[test]
    fn test_is_character_cue_false() {
        assert!(!is_character_cue("hello"));
        assert!(!is_character_cue("INT. OFFICE"));
        assert!(!is_character_cue(""));
        assert!(!is_character_cue("A")); // Too short
        assert!(!is_character_cue("CUT TO:")); // This is a transition
    }

    #[test]
    fn test_is_parenthetical_true() {
        assert!(is_parenthetical("(whispering)"));
        assert!(is_parenthetical("(to John)"));
        assert!(is_parenthetical("(beat)"));
    }

    #[test]
    fn test_is_parenthetical_false() {
        assert!(!is_parenthetical("not parenthetical"));
        assert!(!is_parenthetical("(starts but doesn't end"));
        assert!(!is_parenthetical(""));
    }

    #[test]
    fn test_is_shot_true() {
        assert!(is_shot("ANGLE ON the door"));
        assert!(is_shot("CLOSE ON her face"));
        assert!(is_shot("WIDE ON the landscape"));
        assert!(is_shot("POV through the windshield"));
        assert!(is_shot("EXTREME CLOSE UP on the key"));
        assert!(is_shot("INSERT the letter"));
        assert!(is_shot("INTERCUT phone conversation"));
    }

    #[test]
    fn test_is_shot_false() {
        assert!(!is_shot("Regular text"));
        assert!(!is_shot("JOHN")); // Character, not shot
    }
}
