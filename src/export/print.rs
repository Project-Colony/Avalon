use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result};
use tempfile::TempDir;

use crate::core::binder::{Binder, BinderItem};
use crate::export::compiler::{CompileOptions, OutputFormat, Compiler};

/// Paper size options for printing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaperSize {
    Letter,
    A4,
    A5,
    Legal,
}

impl PaperSize {
    /// Width and height in millimeters (portrait orientation).
    pub fn dimensions_mm(&self) -> (f32, f32) {
        match self {
            PaperSize::Letter => (215.9, 279.4),
            PaperSize::A4 => (210.0, 297.0),
            PaperSize::A5 => (148.0, 210.0),
            PaperSize::Legal => (215.9, 355.6),
        }
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &str {
        match self {
            PaperSize::Letter => "US Letter",
            PaperSize::A4 => "A4",
            PaperSize::A5 => "A5",
            PaperSize::Legal => "US Legal",
        }
    }

    /// All available paper sizes.
    pub fn all() -> Vec<PaperSize> {
        vec![PaperSize::Letter, PaperSize::A4, PaperSize::A5, PaperSize::Legal]
    }
}

/// Page orientation for printing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Portrait,
    Landscape,
}

impl Orientation {
    /// Human-readable display name.
    pub fn display_name(&self) -> &str {
        match self {
            Orientation::Portrait => "Portrait",
            Orientation::Landscape => "Landscape",
        }
    }

    /// Apply orientation to paper dimensions, returning (width, height) in mm.
    pub fn apply(&self, paper_size: &PaperSize) -> (f32, f32) {
        let (w, h) = paper_size.dimensions_mm();
        match self {
            Orientation::Portrait => (w, h),
            Orientation::Landscape => (h, w),
        }
    }
}

/// Page margins in millimeters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Margins {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Margins {
    /// Create uniform margins (same value on all sides).
    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            bottom: value,
            left: value,
            right: value,
        }
    }

    /// Standard one-inch margins (25.4 mm).
    pub fn one_inch() -> Self {
        Self::uniform(25.4)
    }

    /// Check whether margin values are reasonable (non-negative and not excessively large).
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if self.top < 0.0 {
            issues.push("Top margin cannot be negative".to_string());
        }
        if self.bottom < 0.0 {
            issues.push("Bottom margin cannot be negative".to_string());
        }
        if self.left < 0.0 {
            issues.push("Left margin cannot be negative".to_string());
        }
        if self.right < 0.0 {
            issues.push("Right margin cannot be negative".to_string());
        }
        if self.top > 100.0 || self.bottom > 100.0 || self.left > 100.0 || self.right > 100.0 {
            issues.push("Margins exceeding 100mm are unusually large".to_string());
        }
        issues
    }

    /// Total horizontal margin (left + right).
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Total vertical margin (top + bottom).
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

impl Default for Margins {
    fn default() -> Self {
        Self::one_inch()
    }
}

/// Configuration options for printing a document.
#[derive(Debug, Clone)]
pub struct PrintOptions {
    pub paper_size: PaperSize,
    pub orientation: Orientation,
    pub margins: Margins,
    pub include_header: bool,
    pub include_footer: bool,
    pub include_page_numbers: bool,
    pub copies: usize,
}

impl PrintOptions {
    /// Validate the print options and return any issues found.
    pub fn validate(&self) -> Vec<String> {
        let mut issues = self.margins.validate();
        if self.copies == 0 {
            issues.push("Number of copies must be at least 1".to_string());
        }
        if self.copies > 100 {
            issues.push("Number of copies exceeds reasonable limit (100)".to_string());
        }
        let (page_w, page_h) = self.orientation.apply(&self.paper_size);
        if self.margins.horizontal() >= page_w {
            issues.push("Horizontal margins exceed page width".to_string());
        }
        if self.margins.vertical() >= page_h {
            issues.push("Vertical margins exceed page height".to_string());
        }
        issues
    }

    /// Compute the usable content area in millimeters (width, height) after
    /// accounting for orientation and margins.
    pub fn content_area_mm(&self) -> (f32, f32) {
        let (page_w, page_h) = self.orientation.apply(&self.paper_size);
        let content_w = (page_w - self.margins.horizontal()).max(0.0);
        let content_h = (page_h - self.margins.vertical()).max(0.0);
        (content_w, content_h)
    }

    /// Build a `CompileOptions` configured for PDF output that reflects these
    /// print settings.
    pub fn to_compile_options(&self, title: &str, author: &str) -> CompileOptions {
        CompileOptions {
            format: OutputFormat::Pdf,
            title: title.to_string(),
            author: author.to_string(),
            include_front_matter: true,
            include_toc: false,
            replace_placeholders: true,
            compile_marked_only: true,
            ..CompileOptions::default()
        }
    }

    /// Summary string describing the current print configuration.
    pub fn summary(&self) -> String {
        let (w, h) = self.orientation.apply(&self.paper_size);
        format!(
            "{} {}, {:.0}x{:.0}mm, margins T{:.1}/B{:.1}/L{:.1}/R{:.1}mm, {} copies{}{}{}",
            self.paper_size.display_name(),
            self.orientation.display_name(),
            w,
            h,
            self.margins.top,
            self.margins.bottom,
            self.margins.left,
            self.margins.right,
            self.copies,
            if self.include_header { ", header" } else { "" },
            if self.include_footer { ", footer" } else { "" },
            if self.include_page_numbers { ", page numbers" } else { "" },
        )
    }
}

impl Default for PrintOptions {
    fn default() -> Self {
        default_print_options()
    }
}

/// Return sensible default print options: Letter paper, portrait, one-inch
/// margins, page numbers enabled, single copy.
pub fn default_print_options() -> PrintOptions {
    PrintOptions {
        paper_size: PaperSize::Letter,
        orientation: Orientation::Portrait,
        margins: Margins::one_inch(),
        include_header: false,
        include_footer: false,
        include_page_numbers: true,
        copies: 1,
    }
}

/// Generate a PDF for the entire compiled document and return the path to
/// the temporary file. The caller is responsible for cleaning up the temp
/// directory once the PDF viewer has opened the file.
pub fn print_document(binder: &Binder, options: &PrintOptions) -> Result<PathBuf> {
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let pdf_path = temp_dir.path().join("avalon_print.pdf");

    print_to_pdf(binder, &pdf_path, options)?;

    // Persist the temp dir so the file is not deleted when TempDir drops.
    let pdf_path_owned = pdf_path.to_path_buf();
    let _ = temp_dir.keep();

    Ok(pdf_path_owned)
}

/// Print a single binder item by compiling it into a temporary PDF and
/// returning the path.
pub fn print_current_item(item: &BinderItem, options: &PrintOptions) -> Result<PathBuf> {
    // Wrap the single item in a minimal binder so the compiler can process it.
    let mut binder = Binder::default_structure();
    let mut clone = item.clone();
    clone.include_in_compile = true;
    binder.draft.add_child(clone);

    print_document(&binder, options)
}

/// Generate a PDF to a specific output path using the existing compiler
/// infrastructure.
pub fn print_to_pdf(binder: &Binder, output_path: &Path, options: &PrintOptions) -> Result<()> {
    let compile_opts = options.to_compile_options(
        &binder.draft.title,
        "",
    );

    Compiler::save_to_file(binder, &compile_opts, output_path)
        .context("Failed to compile PDF for printing")
}

/// Open the given PDF file with the system's default PDF viewer.
///
/// Platform detection:
/// - Linux: `xdg-open`
/// - macOS: `open`
/// - Windows: `cmd /C start`
pub fn open_pdf_viewer(path: &Path) -> Result<()> {
    let path_str = path
        .to_str()
        .context("PDF path contains invalid UTF-8")?;

    let mut cmd = if cfg!(target_os = "linux") {
        let mut c = Command::new("xdg-open");
        c.arg(path_str);
        c
    } else if cfg!(target_os = "macos") {
        let mut c = Command::new("open");
        c.arg(path_str);
        c
    } else if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", "start", "", path_str]);
        c
    } else {
        anyhow::bail!(
            "Unsupported platform: unable to detect a PDF viewer command"
        );
    };

    let mut child = cmd.spawn().context("Failed to launch PDF viewer")?;

    // Reap the child in a background thread to prevent zombie processes on Unix.
    // The viewer is expected to outlive our interest in it, so we just wait quietly.
    std::thread::spawn(move || {
        let _ = child.wait();
    });

    Ok(())
}

/// Return the name of the command that would be used to open a PDF on the
/// current platform, or `None` if the platform is not recognized.
pub fn pdf_viewer_command() -> Option<&'static str> {
    if cfg!(target_os = "linux") {
        Some("xdg-open")
    } else if cfg!(target_os = "macos") {
        Some("open")
    } else if cfg!(target_os = "windows") {
        Some("cmd /C start")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // PaperSize tests
    // ---------------------------------------------------------------

    #[test]
    fn test_paper_size_dimensions_letter() {
        let (w, h) = PaperSize::Letter.dimensions_mm();
        assert!((w - 215.9).abs() < 0.01);
        assert!((h - 279.4).abs() < 0.01);
    }

    #[test]
    fn test_paper_size_dimensions_a4() {
        let (w, h) = PaperSize::A4.dimensions_mm();
        assert!((w - 210.0).abs() < 0.01);
        assert!((h - 297.0).abs() < 0.01);
    }

    #[test]
    fn test_paper_size_dimensions_a5() {
        let (w, h) = PaperSize::A5.dimensions_mm();
        assert!((w - 148.0).abs() < 0.01);
        assert!((h - 210.0).abs() < 0.01);
    }

    #[test]
    fn test_paper_size_dimensions_legal() {
        let (w, h) = PaperSize::Legal.dimensions_mm();
        assert!((w - 215.9).abs() < 0.01);
        assert!((h - 355.6).abs() < 0.01);
    }

    #[test]
    fn test_paper_size_display_names() {
        assert_eq!(PaperSize::Letter.display_name(), "US Letter");
        assert_eq!(PaperSize::A4.display_name(), "A4");
        assert_eq!(PaperSize::A5.display_name(), "A5");
        assert_eq!(PaperSize::Legal.display_name(), "US Legal");
    }

    #[test]
    fn test_paper_size_all() {
        let all = PaperSize::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&PaperSize::Letter));
        assert!(all.contains(&PaperSize::A4));
        assert!(all.contains(&PaperSize::A5));
        assert!(all.contains(&PaperSize::Legal));
    }

    // ---------------------------------------------------------------
    // Orientation tests
    // ---------------------------------------------------------------

    #[test]
    fn test_orientation_display_names() {
        assert_eq!(Orientation::Portrait.display_name(), "Portrait");
        assert_eq!(Orientation::Landscape.display_name(), "Landscape");
    }

    #[test]
    fn test_orientation_portrait_preserves_dimensions() {
        let (w, h) = Orientation::Portrait.apply(&PaperSize::A4);
        assert!((w - 210.0).abs() < 0.01);
        assert!((h - 297.0).abs() < 0.01);
    }

    #[test]
    fn test_orientation_landscape_swaps_dimensions() {
        let (w, h) = Orientation::Landscape.apply(&PaperSize::A4);
        assert!((w - 297.0).abs() < 0.01);
        assert!((h - 210.0).abs() < 0.01);
    }

    #[test]
    fn test_orientation_landscape_letter() {
        let (w, h) = Orientation::Landscape.apply(&PaperSize::Letter);
        let (pw, ph) = PaperSize::Letter.dimensions_mm();
        assert!((w - ph).abs() < 0.01);
        assert!((h - pw).abs() < 0.01);
    }

    // ---------------------------------------------------------------
    // Margins tests
    // ---------------------------------------------------------------

    #[test]
    fn test_margins_uniform() {
        let m = Margins::uniform(10.0);
        assert_eq!(m.top, 10.0);
        assert_eq!(m.bottom, 10.0);
        assert_eq!(m.left, 10.0);
        assert_eq!(m.right, 10.0);
    }

    #[test]
    fn test_margins_one_inch() {
        let m = Margins::one_inch();
        assert!((m.top - 25.4).abs() < 0.01);
        assert!((m.bottom - 25.4).abs() < 0.01);
        assert!((m.left - 25.4).abs() < 0.01);
        assert!((m.right - 25.4).abs() < 0.01);
    }

    #[test]
    fn test_margins_default_is_one_inch() {
        let m = Margins::default();
        let expected = Margins::one_inch();
        assert_eq!(m, expected);
    }

    #[test]
    fn test_margins_horizontal_vertical() {
        let m = Margins {
            top: 10.0,
            bottom: 20.0,
            left: 15.0,
            right: 25.0,
        };
        assert!((m.horizontal() - 40.0).abs() < 0.01);
        assert!((m.vertical() - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_margins_validate_ok() {
        let m = Margins::one_inch();
        assert!(m.validate().is_empty());
    }

    #[test]
    fn test_margins_validate_negative() {
        let m = Margins {
            top: -1.0,
            bottom: 10.0,
            left: 10.0,
            right: 10.0,
        };
        let issues = m.validate();
        assert!(issues.iter().any(|i| i.contains("Top margin")));
    }

    #[test]
    fn test_margins_validate_too_large() {
        let m = Margins::uniform(150.0);
        let issues = m.validate();
        assert!(issues.iter().any(|i| i.contains("unusually large")));
    }

    // ---------------------------------------------------------------
    // PrintOptions construction and defaults tests
    // ---------------------------------------------------------------

    #[test]
    fn test_default_print_options_values() {
        let opts = default_print_options();
        assert_eq!(opts.paper_size, PaperSize::Letter);
        assert_eq!(opts.orientation, Orientation::Portrait);
        assert!(!opts.include_header);
        assert!(!opts.include_footer);
        assert!(opts.include_page_numbers);
        assert_eq!(opts.copies, 1);
    }

    #[test]
    fn test_print_options_default_trait() {
        let a = default_print_options();
        let b = PrintOptions::default();
        assert_eq!(a.paper_size, b.paper_size);
        assert_eq!(a.orientation, b.orientation);
        assert_eq!(a.copies, b.copies);
        assert_eq!(a.include_header, b.include_header);
        assert_eq!(a.include_footer, b.include_footer);
        assert_eq!(a.include_page_numbers, b.include_page_numbers);
    }

    #[test]
    fn test_print_options_validate_ok() {
        let opts = default_print_options();
        assert!(opts.validate().is_empty());
    }

    #[test]
    fn test_print_options_validate_zero_copies() {
        let mut opts = default_print_options();
        opts.copies = 0;
        let issues = opts.validate();
        assert!(issues.iter().any(|i| i.contains("at least 1")));
    }

    #[test]
    fn test_print_options_validate_too_many_copies() {
        let mut opts = default_print_options();
        opts.copies = 200;
        let issues = opts.validate();
        assert!(issues.iter().any(|i| i.contains("exceeds reasonable limit")));
    }

    #[test]
    fn test_print_options_validate_margins_exceed_width() {
        let mut opts = default_print_options();
        opts.margins = Margins {
            top: 10.0,
            bottom: 10.0,
            left: 150.0,
            right: 150.0,
        };
        let issues = opts.validate();
        assert!(issues.iter().any(|i| i.contains("Horizontal margins")));
    }

    #[test]
    fn test_print_options_validate_margins_exceed_height() {
        let mut opts = default_print_options();
        opts.margins = Margins {
            top: 200.0,
            bottom: 200.0,
            left: 10.0,
            right: 10.0,
        };
        let issues = opts.validate();
        assert!(issues.iter().any(|i| i.contains("Vertical margins")));
    }

    // ---------------------------------------------------------------
    // Content area and summary tests
    // ---------------------------------------------------------------

    #[test]
    fn test_content_area_portrait_a4() {
        let opts = PrintOptions {
            paper_size: PaperSize::A4,
            orientation: Orientation::Portrait,
            margins: Margins::uniform(20.0),
            include_header: false,
            include_footer: false,
            include_page_numbers: false,
            copies: 1,
        };
        let (cw, ch) = opts.content_area_mm();
        assert!((cw - 170.0).abs() < 0.01); // 210 - 40
        assert!((ch - 257.0).abs() < 0.01); // 297 - 40
    }

    #[test]
    fn test_content_area_landscape_letter() {
        let opts = PrintOptions {
            paper_size: PaperSize::Letter,
            orientation: Orientation::Landscape,
            margins: Margins::one_inch(),
            include_header: false,
            include_footer: false,
            include_page_numbers: false,
            copies: 1,
        };
        let (cw, ch) = opts.content_area_mm();
        // Landscape Letter: w=279.4, h=215.9
        let expected_w = 279.4 - 2.0 * 25.4;
        let expected_h = 215.9 - 2.0 * 25.4;
        assert!((cw - expected_w).abs() < 0.1);
        assert!((ch - expected_h).abs() < 0.1);
    }

    #[test]
    fn test_content_area_clamps_to_zero() {
        let opts = PrintOptions {
            paper_size: PaperSize::A5,
            orientation: Orientation::Portrait,
            margins: Margins::uniform(500.0), // absurdly large
            include_header: false,
            include_footer: false,
            include_page_numbers: false,
            copies: 1,
        };
        let (cw, ch) = opts.content_area_mm();
        assert_eq!(cw, 0.0);
        assert_eq!(ch, 0.0);
    }

    #[test]
    fn test_print_options_summary_contains_paper_size() {
        let opts = default_print_options();
        let summary = opts.summary();
        assert!(summary.contains("US Letter"));
        assert!(summary.contains("Portrait"));
        assert!(summary.contains("1 copies"));
    }

    #[test]
    fn test_print_options_summary_with_header_footer() {
        let mut opts = default_print_options();
        opts.include_header = true;
        opts.include_footer = true;
        opts.include_page_numbers = true;
        let summary = opts.summary();
        assert!(summary.contains("header"));
        assert!(summary.contains("footer"));
        assert!(summary.contains("page numbers"));
    }

    #[test]
    fn test_print_options_summary_without_extras() {
        let mut opts = default_print_options();
        opts.include_header = false;
        opts.include_footer = false;
        opts.include_page_numbers = false;
        let summary = opts.summary();
        assert!(!summary.contains("header"));
        assert!(!summary.contains("footer"));
        assert!(!summary.contains("page numbers"));
    }

    // ---------------------------------------------------------------
    // to_compile_options tests
    // ---------------------------------------------------------------

    #[test]
    fn test_to_compile_options_format() {
        let opts = default_print_options();
        let compile = opts.to_compile_options("My Book", "Jane Doe");
        assert_eq!(compile.format, OutputFormat::Pdf);
        assert_eq!(compile.title, "My Book");
        assert_eq!(compile.author, "Jane Doe");
    }

    #[test]
    fn test_to_compile_options_flags() {
        let opts = default_print_options();
        let compile = opts.to_compile_options("Title", "Author");
        assert!(compile.include_front_matter);
        assert!(!compile.include_toc);
        assert!(compile.replace_placeholders);
        assert!(compile.compile_marked_only);
    }

    // ---------------------------------------------------------------
    // pdf_viewer_command tests
    // ---------------------------------------------------------------

    #[test]
    fn test_pdf_viewer_command_returns_something() {
        // On any recognized platform this should return Some.
        // In CI (Linux) we expect "xdg-open".
        let cmd = pdf_viewer_command();
        if cfg!(target_os = "linux") {
            assert_eq!(cmd, Some("xdg-open"));
        } else if cfg!(target_os = "macos") {
            assert_eq!(cmd, Some("open"));
        } else if cfg!(target_os = "windows") {
            assert_eq!(cmd, Some("cmd /C start"));
        }
        // On an unknown platform it would be None, which is also acceptable.
    }

    // ---------------------------------------------------------------
    // Equality / Clone / Copy trait tests
    // ---------------------------------------------------------------

    #[test]
    fn test_paper_size_equality() {
        assert_eq!(PaperSize::A4, PaperSize::A4);
        assert_ne!(PaperSize::A4, PaperSize::Letter);
    }

    #[test]
    fn test_orientation_equality() {
        assert_eq!(Orientation::Portrait, Orientation::Portrait);
        assert_ne!(Orientation::Portrait, Orientation::Landscape);
    }

    #[test]
    fn test_paper_size_copy() {
        let a = PaperSize::A4;
        let b = a; // Copy
        assert_eq!(a, b);
    }

    #[test]
    fn test_margins_clone() {
        let m = Margins::uniform(15.0);
        let m2 = m;
        assert_eq!(m.top, m2.top);
        assert_eq!(m.left, m2.left);
    }

    #[test]
    fn test_print_options_clone() {
        let opts = default_print_options();
        let cloned = opts.clone();
        assert_eq!(cloned.paper_size, opts.paper_size);
        assert_eq!(cloned.orientation, opts.orientation);
        assert_eq!(cloned.copies, opts.copies);
    }

    // ---------------------------------------------------------------
    // Custom configuration combinations
    // ---------------------------------------------------------------

    #[test]
    fn test_custom_print_options() {
        let opts = PrintOptions {
            paper_size: PaperSize::A4,
            orientation: Orientation::Landscape,
            margins: Margins {
                top: 15.0,
                bottom: 15.0,
                left: 20.0,
                right: 20.0,
            },
            include_header: true,
            include_footer: true,
            include_page_numbers: true,
            copies: 3,
        };
        assert!(opts.validate().is_empty());
        let (cw, ch) = opts.content_area_mm();
        // Landscape A4: w=297, h=210
        assert!((cw - (297.0 - 40.0)).abs() < 0.01);
        assert!((ch - (210.0 - 30.0)).abs() < 0.01);
    }

    #[test]
    fn test_legal_landscape_content_area() {
        let opts = PrintOptions {
            paper_size: PaperSize::Legal,
            orientation: Orientation::Landscape,
            margins: Margins::one_inch(),
            include_header: false,
            include_footer: false,
            include_page_numbers: false,
            copies: 1,
        };
        let (cw, ch) = opts.content_area_mm();
        // Landscape Legal: w=355.6, h=215.9
        let expected_w = 355.6 - 2.0 * 25.4;
        let expected_h = 215.9 - 2.0 * 25.4;
        assert!((cw - expected_w).abs() < 0.1);
        assert!((ch - expected_h).abs() < 0.1);
    }

    #[test]
    fn test_a5_portrait_content_area() {
        let opts = PrintOptions {
            paper_size: PaperSize::A5,
            orientation: Orientation::Portrait,
            margins: Margins::uniform(10.0),
            include_header: false,
            include_footer: false,
            include_page_numbers: false,
            copies: 1,
        };
        let (cw, ch) = opts.content_area_mm();
        assert!((cw - 128.0).abs() < 0.01); // 148 - 20
        assert!((ch - 190.0).abs() < 0.01); // 210 - 20
    }

    // ---------------------------------------------------------------
    // Asymmetric margins
    // ---------------------------------------------------------------

    #[test]
    fn test_asymmetric_margins() {
        let m = Margins {
            top: 5.0,
            bottom: 10.0,
            left: 30.0,
            right: 15.0,
        };
        assert!((m.horizontal() - 45.0).abs() < 0.01);
        assert!((m.vertical() - 15.0).abs() < 0.01);
        assert!(m.validate().is_empty());
    }

    // ---------------------------------------------------------------
    // Multiple negative margins
    // ---------------------------------------------------------------

    #[test]
    fn test_multiple_negative_margins() {
        let m = Margins {
            top: -5.0,
            bottom: -3.0,
            left: -1.0,
            right: -2.0,
        };
        let issues = m.validate();
        assert!(issues.len() >= 4);
    }
}
