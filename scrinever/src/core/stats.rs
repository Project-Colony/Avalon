use super::binder::Binder;

/// Aggregated statistics for the project or a selection
#[derive(Debug, Clone, Default)]
pub struct Statistics {
    pub word_count: usize,
    pub char_count: usize,
    pub char_count_no_spaces: usize,
    pub paragraph_count: usize,
    pub sentence_count: usize,
    pub line_count: usize,
    pub page_count: f64,
    pub document_count: usize,
    pub folder_count: usize,
    pub average_words_per_document: f64,
}

impl Statistics {
    /// Compute statistics for the entire project
    pub fn from_binder(binder: &Binder) -> Self {
        use super::binder::BinderItemKind;

        let items = binder.all_items();
        let mut stats = Statistics::default();

        for item in &items {
            match item.kind {
                BinderItemKind::Text => stats.document_count += 1,
                BinderItemKind::Folder => stats.folder_count += 1,
                _ => {}
            }

            if let Some(ref doc) = item.document {
                stats.word_count += doc.word_count();
                stats.char_count += doc.char_count();
                stats.char_count_no_spaces += doc.char_count_no_spaces();
                stats.paragraph_count += doc.paragraph_count();
                stats.sentence_count += doc.sentence_count();
                stats.line_count += doc.line_count();
            }
        }

        stats.page_count = stats.word_count as f64 / 250.0;
        stats.average_words_per_document = if stats.document_count > 0 {
            stats.word_count as f64 / stats.document_count as f64
        } else {
            0.0
        };

        stats
    }

    /// Compute statistics for a single text content
    pub fn from_text(text: &str) -> Self {
        Statistics {
            word_count: text.split_whitespace().count(),
            char_count: text.len(),
            char_count_no_spaces: text.chars().filter(|c| !c.is_whitespace()).count(),
            paragraph_count: text.split("\n\n").filter(|p| !p.trim().is_empty()).count(),
            sentence_count: text.chars()
                .filter(|c| *c == '.' || *c == '!' || *c == '?')
                .count()
                .max(if text.is_empty() { 0 } else { 1 }),
            line_count: if text.is_empty() { 0 } else { text.lines().count() },
            page_count: text.split_whitespace().count() as f64 / 250.0,
            document_count: 1,
            folder_count: 0,
            average_words_per_document: text.split_whitespace().count() as f64,
        }
    }

    /// Format word count with target progress
    pub fn progress_string(&self, target: Option<usize>) -> String {
        match target {
            Some(target) => {
                let pct = (self.word_count as f64 / target as f64 * 100.0).min(100.0);
                format!("{} / {} words ({:.1}%)", self.word_count, target, pct)
            }
            None => format!("{} words", self.word_count),
        }
    }
}

/// Session statistics for tracking writing sessions
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub words_written: i64, // Can be negative if deleting
    pub time_elapsed_seconds: u64,
    pub words_per_minute: f64,
}

impl SessionStats {
    pub fn new() -> Self {
        Self {
            words_written: 0,
            time_elapsed_seconds: 0,
            words_per_minute: 0.0,
        }
    }

    pub fn update(&mut self, word_delta: i64, elapsed_seconds: u64) {
        self.words_written += word_delta;
        self.time_elapsed_seconds = elapsed_seconds;
        if elapsed_seconds > 0 {
            self.words_per_minute = self.words_written as f64 / (elapsed_seconds as f64 / 60.0);
        }
    }
}
