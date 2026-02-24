#![allow(dead_code)]
use std::path::Path;
use std::fs;

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::binder::{BinderItem, BinderItemKind};
use crate::core::document::Document;

/// Metadata about an imported media file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    /// Absolute or project-relative path to the imported file.
    pub file_path: String,
    /// Size of the file in bytes.
    pub file_size: u64,
    /// Detected format (e.g. "png", "pdf").
    pub format: String,
    /// The original filename including extension.
    pub original_filename: String,
}

/// Return the list of image file extensions that can be imported.
pub fn supported_image_extensions() -> Vec<&'static str> {
    vec!["png", "jpg", "jpeg", "gif", "bmp", "svg", "webp", "tiff"]
}

/// Return the list of all media file extensions that can be imported (images + PDF).
pub fn supported_media_extensions() -> Vec<&'static str> {
    vec!["png", "jpg", "jpeg", "gif", "bmp", "svg", "webp", "tiff", "pdf"]
}

/// Check whether `path` refers to a file whose extension is among the
/// supported media types.
pub fn is_supported_media(path: &Path) -> bool {
    let ext = match path.extension().and_then(|e| e.to_str()) {
        Some(e) => e.to_lowercase(),
        None => return false,
    };
    supported_media_extensions().iter().any(|&supported| supported == ext)
}

/// Import an image file, returning a `BinderItem` of kind `Image`.
///
/// The item's title is derived from the filename (without extension).
/// A `MediaMetadata` struct is serialised as JSON and stored in the
/// document's `notes` field so that other parts of the application can
/// retrieve the original file information.
pub fn import_image(path: &Path) -> Result<BinderItem> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or_else(|| anyhow!("File has no extension: {}", path.display()))?;

    if !supported_image_extensions().iter().any(|&s| s == ext) {
        return Err(anyhow!(
            "Unsupported image format '{}': {}",
            ext,
            path.display()
        ));
    }

    let fs_meta = fs::metadata(path)
        .map_err(|e| anyhow!("Cannot read file metadata for {}: {}", path.display(), e))?;

    let original_filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled Image")
        .to_string();

    let metadata = MediaMetadata {
        file_path: path.to_string_lossy().to_string(),
        file_size: fs_meta.len(),
        format: ext.clone(),
        original_filename,
    };

    let notes_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| anyhow!("Failed to serialize media metadata: {}", e))?;

    let now = Utc::now();
    let mut doc = Document::new();
    doc.notes = notes_json;
    doc.content = format!("[Image: {}]", metadata.file_path);

    let item = BinderItem {
        id: Uuid::new_v4(),
        title,
        kind: BinderItemKind::Image,
        created_at: now,
        modified_at: now,
        metadata: Default::default(),
        children: Vec::new(),
        document: Some(doc),
        snapshots: Vec::new(),
        expanded: false,
        include_in_compile: false,
        synopsis: format!("{} image ({} bytes)", ext.to_uppercase(), fs_meta.len()),
    };

    Ok(item)
}

/// Import a PDF file, returning a `BinderItem` of kind `Pdf`.
///
/// Behaves the same as [`import_image`] but validates that the file has
/// a `.pdf` extension and uses `BinderItemKind::Pdf`.
pub fn import_pdf(path: &Path) -> Result<BinderItem> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or_else(|| anyhow!("File has no extension: {}", path.display()))?;

    if ext != "pdf" {
        return Err(anyhow!(
            "Not a PDF file (extension is '{}'): {}",
            ext,
            path.display()
        ));
    }

    let fs_meta = fs::metadata(path)
        .map_err(|e| anyhow!("Cannot read file metadata for {}: {}", path.display(), e))?;

    let original_filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled PDF")
        .to_string();

    let metadata = MediaMetadata {
        file_path: path.to_string_lossy().to_string(),
        file_size: fs_meta.len(),
        format: "pdf".to_string(),
        original_filename,
    };

    let notes_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| anyhow!("Failed to serialize media metadata: {}", e))?;

    let now = Utc::now();
    let mut doc = Document::new();
    doc.notes = notes_json;
    doc.content = format!("[PDF: {}]", metadata.file_path);

    let item = BinderItem {
        id: Uuid::new_v4(),
        title,
        kind: BinderItemKind::Pdf,
        created_at: now,
        modified_at: now,
        metadata: Default::default(),
        children: Vec::new(),
        document: Some(doc),
        snapshots: Vec::new(),
        expanded: false,
        include_in_compile: false,
        synopsis: format!("PDF document ({} bytes)", fs_meta.len()),
    };

    Ok(item)
}

/// Auto-detect the media type from the file extension and delegate to the
/// appropriate import function ([`import_image`] or [`import_pdf`]).
pub fn import_media_file(path: &Path) -> Result<BinderItem> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or_else(|| anyhow!("File has no extension: {}", path.display()))?;

    if ext == "pdf" {
        import_pdf(path)
    } else if supported_image_extensions().iter().any(|&s| s == ext) {
        import_image(path)
    } else {
        Err(anyhow!(
            "Unsupported media format '{}': {}",
            ext,
            path.display()
        ))
    }
}

/// Batch-import several files.  Each path is processed independently; a
/// failure on one file does not prevent the others from being imported.
pub fn import_multiple(paths: &[&Path]) -> Vec<Result<BinderItem>> {
    paths.iter().map(|p| import_media_file(p)).collect()
}

/// Produce a short human-readable summary string for a media binder item.
///
/// If the item's document notes contain valid `MediaMetadata` JSON the
/// summary includes the format and file size.  Otherwise a generic
/// description is returned.
pub fn media_item_summary(item: &BinderItem) -> String {
    if let Some(ref doc) = item.document {
        if let Ok(meta) = serde_json::from_str::<MediaMetadata>(&doc.notes) {
            let size_display = super::format_bytes(meta.file_size);
            return format!(
                "{} ({}, {})",
                item.title,
                meta.format.to_uppercase(),
                size_display
            );
        }
    }

    match item.kind {
        BinderItemKind::Image => format!("{} (Image)", item.title),
        BinderItemKind::Pdf => format!("{} (PDF)", item.title),
        _ => format!("{} ({})", item.title, item.kind.label()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper: create a temporary file with the given extension and some
    /// dummy content, returning the `NamedTempFile` so it stays alive for
    /// the duration of the test.
    fn tmp_file(extension: &str, content: &[u8]) -> NamedTempFile {
        let suffix = format!(".{}", extension);
        let mut f = tempfile::Builder::new()
            .suffix(&suffix)
            .tempfile()
            .expect("failed to create temp file");
        f.write_all(content).expect("failed to write temp file");
        f.flush().expect("failed to flush temp file");
        f
    }

    // ---------------------------------------------------------------
    // 1. import_image -- basic success
    // ---------------------------------------------------------------
    #[test]
    fn test_import_image_png() {
        let f = tmp_file("png", b"fake png data");
        let item = import_image(f.path()).unwrap();
        assert_eq!(item.kind, BinderItemKind::Image);
        assert!(!item.title.is_empty());
        assert!(item.document.is_some());
        let doc = item.document.as_ref().unwrap();
        let meta: MediaMetadata = serde_json::from_str(&doc.notes).unwrap();
        assert_eq!(meta.format, "png");
        assert_eq!(meta.file_size, 13);
    }

    // ---------------------------------------------------------------
    // 2. import_image -- each supported extension
    // ---------------------------------------------------------------
    #[test]
    fn test_import_image_all_extensions() {
        for ext in supported_image_extensions() {
            let f = tmp_file(ext, b"data");
            let item = import_image(f.path()).unwrap();
            assert_eq!(item.kind, BinderItemKind::Image);
            let doc = item.document.as_ref().unwrap();
            let meta: MediaMetadata = serde_json::from_str(&doc.notes).unwrap();
            assert_eq!(meta.format, ext);
        }
    }

    // ---------------------------------------------------------------
    // 3. import_image -- unsupported extension is rejected
    // ---------------------------------------------------------------
    #[test]
    fn test_import_image_unsupported_extension() {
        let f = tmp_file("txt", b"hello");
        let result = import_image(f.path());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Unsupported image format"));
    }

    // ---------------------------------------------------------------
    // 4. import_image -- file without extension is rejected
    // ---------------------------------------------------------------
    #[test]
    fn test_import_image_no_extension() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("noext");
        fs::write(&path, b"data").unwrap();
        let result = import_image(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no extension"));
    }

    // ---------------------------------------------------------------
    // 5. import_image -- nonexistent file
    // ---------------------------------------------------------------
    #[test]
    fn test_import_image_nonexistent_file() {
        let result = import_image(Path::new("/tmp/does_not_exist_12345.png"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot read file metadata"));
    }

    // ---------------------------------------------------------------
    // 6. import_pdf -- basic success
    // ---------------------------------------------------------------
    #[test]
    fn test_import_pdf_success() {
        let f = tmp_file("pdf", b"%PDF-1.4 fake");
        let item = import_pdf(f.path()).unwrap();
        assert_eq!(item.kind, BinderItemKind::Pdf);
        let doc = item.document.as_ref().unwrap();
        let meta: MediaMetadata = serde_json::from_str(&doc.notes).unwrap();
        assert_eq!(meta.format, "pdf");
    }

    // ---------------------------------------------------------------
    // 7. import_pdf -- wrong extension is rejected
    // ---------------------------------------------------------------
    #[test]
    fn test_import_pdf_wrong_extension() {
        let f = tmp_file("png", b"not a pdf");
        let result = import_pdf(f.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not a PDF file"));
    }

    // ---------------------------------------------------------------
    // 8. import_pdf -- nonexistent file
    // ---------------------------------------------------------------
    #[test]
    fn test_import_pdf_nonexistent() {
        let result = import_pdf(Path::new("/tmp/no_such_file_98765.pdf"));
        assert!(result.is_err());
    }

    // ---------------------------------------------------------------
    // 9. import_media_file -- auto-detect image
    // ---------------------------------------------------------------
    #[test]
    fn test_import_media_file_image() {
        let f = tmp_file("jpg", b"jpeg bytes");
        let item = import_media_file(f.path()).unwrap();
        assert_eq!(item.kind, BinderItemKind::Image);
    }

    // ---------------------------------------------------------------
    // 10. import_media_file -- auto-detect pdf
    // ---------------------------------------------------------------
    #[test]
    fn test_import_media_file_pdf() {
        let f = tmp_file("pdf", b"pdf bytes");
        let item = import_media_file(f.path()).unwrap();
        assert_eq!(item.kind, BinderItemKind::Pdf);
    }

    // ---------------------------------------------------------------
    // 11. import_media_file -- unsupported format
    // ---------------------------------------------------------------
    #[test]
    fn test_import_media_file_unsupported() {
        let f = tmp_file("docx", b"word doc");
        let result = import_media_file(f.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported media format"));
    }

    // ---------------------------------------------------------------
    // 12. supported_image_extensions has the right count
    // ---------------------------------------------------------------
    #[test]
    fn test_supported_image_extensions_count() {
        let exts = supported_image_extensions();
        assert_eq!(exts.len(), 8);
        assert!(exts.contains(&"png"));
        assert!(exts.contains(&"jpg"));
        assert!(exts.contains(&"jpeg"));
        assert!(exts.contains(&"gif"));
        assert!(exts.contains(&"bmp"));
        assert!(exts.contains(&"svg"));
        assert!(exts.contains(&"webp"));
        assert!(exts.contains(&"tiff"));
    }

    // ---------------------------------------------------------------
    // 13. supported_media_extensions includes pdf
    // ---------------------------------------------------------------
    #[test]
    fn test_supported_media_extensions_includes_pdf() {
        let exts = supported_media_extensions();
        assert_eq!(exts.len(), 9); // 8 image + pdf
        assert!(exts.contains(&"pdf"));
    }

    // ---------------------------------------------------------------
    // 14. is_supported_media -- positive cases
    // ---------------------------------------------------------------
    #[test]
    fn test_is_supported_media_positive() {
        assert!(is_supported_media(Path::new("photo.png")));
        assert!(is_supported_media(Path::new("/home/user/scan.PDF"))); // case insensitive
        assert!(is_supported_media(Path::new("art.JPEG")));
        assert!(is_supported_media(Path::new("drawing.svg")));
    }

    // ---------------------------------------------------------------
    // 15. is_supported_media -- negative cases
    // ---------------------------------------------------------------
    #[test]
    fn test_is_supported_media_negative() {
        assert!(!is_supported_media(Path::new("readme.txt")));
        assert!(!is_supported_media(Path::new("archive.zip")));
        assert!(!is_supported_media(Path::new("noextension")));
        assert!(!is_supported_media(Path::new(".")));
    }

    // ---------------------------------------------------------------
    // 16. import_multiple -- mixed results
    // ---------------------------------------------------------------
    #[test]
    fn test_import_multiple_mixed() {
        let img = tmp_file("png", b"image data");
        let pdf = tmp_file("pdf", b"pdf data");
        let bad = Path::new("/tmp/nonexistent_file_54321.xyz");

        let results = import_multiple(&[img.path(), pdf.path(), bad]);
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_err());
    }

    // ---------------------------------------------------------------
    // 17. import_multiple -- empty slice
    // ---------------------------------------------------------------
    #[test]
    fn test_import_multiple_empty() {
        let results = import_multiple(&[]);
        assert!(results.is_empty());
    }

    // ---------------------------------------------------------------
    // 18. media_item_summary -- image with metadata
    // ---------------------------------------------------------------
    #[test]
    fn test_media_item_summary_image() {
        let f = tmp_file("png", &[0u8; 2048]);
        let item = import_image(f.path()).unwrap();
        let summary = media_item_summary(&item);
        assert!(summary.contains("PNG"));
        assert!(summary.contains("2.0 KB"));
    }

    // ---------------------------------------------------------------
    // 19. media_item_summary -- pdf with metadata
    // ---------------------------------------------------------------
    #[test]
    fn test_media_item_summary_pdf() {
        let f = tmp_file("pdf", &[0u8; 1_500_000]);
        let item = import_pdf(f.path()).unwrap();
        let summary = media_item_summary(&item);
        assert!(summary.contains("PDF"));
        assert!(summary.contains("MB"));
    }

    // ---------------------------------------------------------------
    // 20. media_item_summary -- item without metadata
    // ---------------------------------------------------------------
    #[test]
    fn test_media_item_summary_no_metadata() {
        let mut item = BinderItem::new_text("Plain Item");
        item.kind = BinderItemKind::Image;
        item.document = None;
        let summary = media_item_summary(&item);
        assert_eq!(summary, "Plain Item (Image)");
    }

    // ---------------------------------------------------------------
    // 21. media_item_summary -- fallback for non-media kind
    // ---------------------------------------------------------------
    #[test]
    fn test_media_item_summary_text_kind() {
        let item = BinderItem::new_text("Chapter 1");
        let summary = media_item_summary(&item);
        assert_eq!(summary, "Chapter 1 (Text)");
    }

    // ---------------------------------------------------------------
    // 22. title is derived from filename stem
    // ---------------------------------------------------------------
    #[test]
    fn test_title_is_filename_stem() {
        let f = tmp_file("gif", b"GIF89a");
        let item = import_image(f.path()).unwrap();
        let stem = f.path().file_stem().unwrap().to_str().unwrap();
        assert_eq!(item.title, stem);
    }

    // ---------------------------------------------------------------
    // 23. metadata round-trip through JSON
    // ---------------------------------------------------------------
    #[test]
    fn test_metadata_round_trip() {
        let meta = MediaMetadata {
            file_path: "/path/to/file.png".to_string(),
            file_size: 42,
            format: "png".to_string(),
            original_filename: "file.png".to_string(),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let parsed: MediaMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.file_path, meta.file_path);
        assert_eq!(parsed.file_size, meta.file_size);
        assert_eq!(parsed.format, meta.format);
        assert_eq!(parsed.original_filename, meta.original_filename);
    }

    // ---------------------------------------------------------------
    // 24. imported item has include_in_compile = false
    // ---------------------------------------------------------------
    #[test]
    fn test_imported_item_not_in_compile() {
        let f = tmp_file("webp", b"webp data");
        let item = import_image(f.path()).unwrap();
        assert!(!item.include_in_compile);
    }

    // ---------------------------------------------------------------
    // 25. imported item has a non-empty synopsis
    // ---------------------------------------------------------------
    #[test]
    fn test_imported_item_has_synopsis() {
        let f = tmp_file("bmp", b"BM bitmap data");
        let item = import_image(f.path()).unwrap();
        assert!(!item.synopsis.is_empty());
        assert!(item.synopsis.contains("BMP"));
    }

    // ---------------------------------------------------------------
    // 26. media_item_summary -- small file shown in bytes
    // ---------------------------------------------------------------
    #[test]
    fn test_media_item_summary_bytes_display() {
        let f = tmp_file("svg", b"<svg/>");
        let item = import_image(f.path()).unwrap();
        let summary = media_item_summary(&item);
        assert!(summary.contains("bytes"));
    }

    // ---------------------------------------------------------------
    // 27. import_media_file -- no extension
    // ---------------------------------------------------------------
    #[test]
    fn test_import_media_file_no_extension() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("plain");
        fs::write(&path, b"data").unwrap();
        let result = import_media_file(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no extension"));
    }
}
