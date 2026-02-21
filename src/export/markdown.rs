use anyhow::Result;
use super::compiler::{CompileContent, CompileOptions};

pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let mut output = String::new();

    // Front matter
    if options.include_front_matter && !options.title.is_empty() {
        output.push_str(&format!("# {}\n\n", options.title));
        if !options.author.is_empty() {
            output.push_str(&format!("*{}*\n\n", options.author));
        }
        output.push_str("---\n\n");
    }

    for (i, content) in contents.iter().enumerate() {
        if content.is_folder {
            // Folder becomes a heading (depth 0 = ##, depth 1 = ###, etc.)
            let level = (content.depth + 2).min(6);
            let hashes = "#".repeat(level);
            output.push_str(&format!("{} {}\n\n", hashes, content.title));
        } else {
            // Document content
            if !content.text.is_empty() {
                output.push_str(&content.text);
                output.push_str("\n\n");
            }
        }

        // Separator between sections
        if i < contents.len() - 1 && !content.is_folder {
            // Already added double newline above
        }
    }

    Ok(output.trim_end().to_string())
}
