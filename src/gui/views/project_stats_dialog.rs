use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Comprehensive project statistics data
pub struct ProjectStatsData {
    pub title: String,
    pub word_count: usize,
    pub char_count: usize,
    pub char_no_spaces: usize,
    pub paragraph_count: usize,
    pub sentence_count: usize,
    pub page_count: f64,
    pub document_count: usize,
    pub folder_count: usize,
    pub avg_words_per_doc: f64,
    pub avg_words_per_day: f64,
    pub total_writing_days: usize,
    pub current_streak: usize,
    pub best_day_words: i64,
    pub total_time_hours: f64,
    pub reading_time_minutes: f64,
    pub speaking_time_minutes: f64,
    pub target_words: Option<usize>,
    pub deadline: Option<String>,
    pub days_remaining: Option<i64>,
    pub words_per_day_needed: Option<usize>,
}

/// Render the project statistics dialog
pub fn view(data: &ProjectStatsData) -> Element<'static, Message> {
    let header = text("Project Statistics")
        .size(20)
        .color(Theme::TEXT_PRIMARY);

    let subtitle = text(format!("\"{}\"", data.title))
        .size(14)
        .color(Theme::TEXT_SECONDARY);

    // Document metrics
    let doc_section = section("Document Metrics", vec![
        stat("Total Words", &format_number(data.word_count)),
        stat("Characters (with spaces)", &format_number(data.char_count)),
        stat("Characters (no spaces)", &format_number(data.char_no_spaces)),
        stat("Paragraphs", &format_number(data.paragraph_count)),
        stat("Sentences", &format_number(data.sentence_count)),
        stat("Pages (est.)", &format!("{:.1}", data.page_count)),
        stat("Documents", &data.document_count.to_string()),
        stat("Folders", &data.folder_count.to_string()),
        stat("Avg Words/Doc", &format!("{:.0}", data.avg_words_per_doc)),
    ]);

    // Time estimates
    let time_section = section("Time Estimates", vec![
        stat("Reading Time", &format_time(data.reading_time_minutes)),
        stat("Speaking Time", &format_time(data.speaking_time_minutes)),
    ]);

    // Writing habits
    let habits_section = section("Writing Habits", vec![
        stat("Avg Words/Day", &format!("{:.0}", data.avg_words_per_day)),
        stat("Writing Days", &data.total_writing_days.to_string()),
        stat("Current Streak", &format!("{} days", data.current_streak)),
        stat("Best Day", &format!("{} words", data.best_day_words)),
        stat("Total Time", &format!("{:.1} hours", data.total_time_hours)),
    ]);

    // Target progress (if target set)
    let target_section: Element<'static, Message> = if let Some(target) = data.target_words {
        let pct = (data.word_count as f64 / target as f64 * 100.0).min(100.0);
        let remaining = if target > data.word_count { target - data.word_count } else { 0 };

        let mut items = vec![
            stat("Target", &format_number(target)),
            stat("Progress", &format!("{:.1}%", pct)),
            stat("Remaining", &format_number(remaining)),
        ];

        if let Some(ref deadline) = data.deadline {
            items.push(stat("Deadline", deadline));
        }
        if let Some(days) = data.days_remaining {
            items.push(stat("Days Remaining", &days.to_string()));
        }
        if let Some(wpd) = data.words_per_day_needed {
            items.push(stat("Words/Day Needed", &format_number(wpd)));
        }

        // Progress bar
        let bar_width = 200.0;
        let filled = (pct / 100.0 * bar_width) as u16;
        let bar_color = if pct >= 100.0 { Theme::SUCCESS }
        else if pct >= 75.0 { Theme::WARNING }
        else { Theme::TEXT_ACCENT };

        let progress_bar = row![
            text(format!("[{}{}]",
                "=".repeat(filled as usize / 5),
                " ".repeat(((bar_width as u16 - filled) / 5) as usize),
            )).size(12).color(bar_color),
        ];

        let s = section("Target Progress", items);
        column![s, progress_bar].spacing(4).into()
    } else {
        Space::with_height(0).into()
    };

    let close_btn = button(
        text("  Close  ").size(14).color(Theme::TEXT_SECONDARY),
    )
    .on_press(Message::HideProjectStats)
    .padding(Padding::from([8, 20]));

    let content = column![
        header,
        subtitle,
        Space::with_height(12),
        doc_section,
        Space::with_height(8),
        time_section,
        Space::with_height(8),
        habits_section,
        Space::with_height(8),
        target_section,
        Space::with_height(16),
        close_btn,
    ]
    .padding(24)
    .max_width(500);

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn section(title: &str, items: Vec<Element<'static, Message>>) -> Element<'static, Message> {
    let header = text(title.to_string())
        .size(14)
        .color(Theme::TEXT_ACCENT);

    let mut col = column![header].spacing(2);
    for item in items {
        col = col.push(item);
    }
    col.into()
}

fn stat(label: &str, value: &str) -> Element<'static, Message> {
    row![
        text(label.to_string()).size(12).color(Theme::TEXT_MUTED).width(Length::FillPortion(2)),
        text(value.to_string()).size(12).color(Theme::TEXT_PRIMARY).width(Length::FillPortion(1)),
    ]
    .spacing(8)
    .into()
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{},{:03}", n / 1000, n % 1000)
    } else {
        n.to_string()
    }
}

fn format_time(minutes: f64) -> String {
    if minutes < 1.0 {
        format!("{:.0} sec", minutes * 60.0)
    } else if minutes < 60.0 {
        format!("{:.0} min", minutes)
    } else {
        let hours = (minutes / 60.0).floor();
        let mins = minutes - hours * 60.0;
        format!("{:.0}h {:.0}m", hours, mins)
    }
}
