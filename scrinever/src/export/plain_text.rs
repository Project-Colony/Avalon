use anyhow::Result;
use super::compiler::{CompileContent, CompileOptions};

pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let mut output = String::new();

    if options.include_front_matter && !options.title.is_empty() {
        let title = options.title.to_uppercase();
        output.push_str(&title);
        output.push('\n');
        output.push_str(&"=".repeat(title.len()));
        output.push('\n');

        if !options.author.is_empty() {
            output.push_str(&format!("by {}", options.author));
            output.push('\n');
        }

        output.push('\n');
        output.push_str(&"-".repeat(40));
        output.push_str("\n\n");
    }

    for content in contents {
        if content.is_folder {
            output.push('\n');
            output.push_str(&content.title.to_uppercase());
            output.push('\n');
            output.push_str(&"-".repeat(content.title.len()));
            output.push_str("\n\n");
        } else if !content.text.is_empty() {
            output.push_str(&content.text);
            output.push_str("\n\n");
        }
    }

    Ok(output.trim_end().to_string())
}
