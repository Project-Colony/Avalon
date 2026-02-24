use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::writing_prompts::{
    self, CharacterProfile, Difficulty, NameSuggestion, PlotSeed, PromptCategory,
    PromptHistory, WritingExercise, WritingPrompt,
};
use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// Data for the writing prompts panel
pub struct WritingPromptsData {
    pub current_prompt: Option<WritingPrompt>,
    pub daily_prompt: WritingPrompt,
    pub character: Option<CharacterProfile>,
    pub plot_seed: Option<PlotSeed>,
    pub names: Vec<NameSuggestion>,
    pub exercises: Vec<WritingExercise>,
    pub history: PromptHistory,
    pub seed: u64,
}

impl WritingPromptsData {
    pub fn new() -> Self {
        let seed = chrono::Utc::now().timestamp() as u64;
        let daily = writing_prompts::daily_prompt(0);
        Self {
            current_prompt: None,
            daily_prompt: daily,
            character: None,
            plot_seed: None,
            names: Vec::new(),
            exercises: writing_prompts::writing_exercises(Difficulty::Beginner),
            history: PromptHistory::default(),
            seed,
        }
    }

    pub fn generate_prompt(&mut self, category: Option<PromptCategory>) {
        self.seed = self.seed.wrapping_add(1);
        self.current_prompt = Some(match category {
            Some(cat) => writing_prompts::generate_prompt(cat, self.seed),
            None => writing_prompts::generate_random_prompt(self.seed),
        });
    }

    pub fn generate_character(&mut self) {
        self.seed = self.seed.wrapping_add(1);
        self.character = Some(writing_prompts::generate_character(self.seed));
    }

    pub fn generate_plot(&mut self) {
        self.seed = self.seed.wrapping_add(1);
        self.plot_seed = Some(writing_prompts::generate_plot_seed(self.seed));
    }

    pub fn generate_names(&mut self) {
        self.seed = self.seed.wrapping_add(1);
        self.names = writing_prompts::generate_names(self.seed, 5);
    }

    pub fn streak(&self) -> usize {
        writing_prompts::prompt_streak(&self.history)
    }
}

/// Render the writing prompts panel (bottom panel)
pub fn view(data: &WritingPromptsData) -> Element<'static, Message> {
    let streak = data.streak();
    let header = row![
        text("WRITING PROMPTS").size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(8),
        if streak > 0 {
            text(format!("{} day streak!", streak)).size(10).color(Theme::SUCCESS)
        } else {
            text("Start your streak!").size(10).color(Theme::TEXT_MUTED)
        },
        Space::with_width(Length::Fill),
        button(
            text("Random Prompt").size(11).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::GenerateWritingPrompt)
        .padding(Padding::from([4, 12])),
        Space::with_width(4),
        button(
            text("Character").size(11).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::GenerateCharacter)
        .padding(Padding::from([4, 12])),
        Space::with_width(4),
        button(
            text("Plot Seed").size(11).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::GeneratePlotSeed)
        .padding(Padding::from([4, 12])),
        Space::with_width(4),
        button(
            text("Names").size(11).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::GenerateWritingNames)
        .padding(Padding::from([4, 12])),
    ];

    // Daily prompt
    let daily_section = column![
        text("Daily Prompt:").size(11).color(Theme::TEXT_MUTED),
        text(data.daily_prompt.prompt_text.clone()).size(12).color(Theme::TEXT_PRIMARY),
        row![
            text(format!("Category: {}", data.daily_prompt.category.label()))
                .size(9).color(Theme::TEXT_MUTED),
            Space::with_width(8),
            text(format!("~{} words", data.daily_prompt.estimated_words))
                .size(9).color(Theme::TEXT_MUTED),
        ],
    ].spacing(2);

    // Category buttons
    let mut cat_row = row![
        text("Categories: ").size(10).color(Theme::TEXT_MUTED),
    ].spacing(4);
    for cat in PromptCategory::all() {
        cat_row = cat_row.push(
            button(
                text(cat.label()).size(9).color(Theme::TEXT_SECONDARY),
            )
            .on_press(Message::GenerateWritingPromptCategory(cat.label().to_string()))
            .padding(Padding::from([2, 6])),
        );
    }

    // Current prompt display
    let prompt_section: Element<'static, Message> = if let Some(ref prompt) = data.current_prompt {
        column![
            row![
                text(format!("[{}]", prompt.category.label()))
                    .size(10).color(Theme::TEXT_ACCENT),
                Space::with_width(8),
                text(format!("~{} words", prompt.estimated_words))
                    .size(9).color(Theme::TEXT_MUTED),
            ],
            text(prompt.prompt_text.clone()).size(12).color(Theme::TEXT_PRIMARY),
            if !prompt.tags.is_empty() {
                text(format!("Tags: {}", prompt.tags.join(", ")))
                    .size(9).color(Theme::TEXT_MUTED)
            } else {
                text("").size(1)
            },
        ].spacing(2).into()
    } else {
        Space::with_height(0).into()
    };

    // Character display
    let char_section: Element<'static, Message> = if let Some(ref ch) = data.character {
        column![
            text("Generated Character:").size(11).color(Theme::TEXT_MUTED),
            text(format!("{} - {}", ch.name_suggestion, ch.occupation))
                .size(12).color(Theme::TEXT_PRIMARY),
            text(format!("Trait: {} | Flaw: {} | Age: {}",
                ch.trait_primary, ch.flaw, ch.age_range
            )).size(10).color(Theme::TEXT_SECONDARY),
            text(format!("Motivation: {}", ch.motivation))
                .size(10).color(Theme::TEXT_SECONDARY),
            text(format!("Background: {}", ch.background_hook))
                .size(10).color(Theme::TEXT_MUTED),
        ].spacing(1).into()
    } else {
        Space::with_height(0).into()
    };

    // Plot seed display
    let plot_section: Element<'static, Message> = if let Some(ref plot) = data.plot_seed {
        column![
            text("Generated Plot:").size(11).color(Theme::TEXT_MUTED),
            text(format!("Genre: {} | Theme: {}", plot.genre, plot.theme))
                .size(11).color(Theme::TEXT_PRIMARY),
            text(format!("Protagonist: {} vs Antagonist: {}",
                plot.protagonist_type, plot.antagonist_type
            )).size(10).color(Theme::TEXT_SECONDARY),
            text(format!("Setting: {} | Conflict: {}",
                plot.setting, plot.central_conflict
            )).size(10).color(Theme::TEXT_SECONDARY),
            text(format!("Hook: {}", plot.opening_hook))
                .size(10).color(Theme::TEXT_ACCENT),
        ].spacing(1).into()
    } else {
        Space::with_height(0).into()
    };

    // Names display
    let names_section: Element<'static, Message> = if !data.names.is_empty() {
        let mut names_col = column![
            text("Generated Names:").size(11).color(Theme::TEXT_MUTED),
        ].spacing(1);
        for name in &data.names {
            names_col = names_col.push(
                row![
                    text(format!("{} {}", name.first_name, name.last_name))
                        .size(11).color(Theme::TEXT_PRIMARY),
                    Space::with_width(8),
                    text(format!("Origin: {}", name.origin))
                        .size(9).color(Theme::TEXT_MUTED),
                    Space::with_width(4),
                    text(format!("({})", name.meaning))
                        .size(9).color(Theme::TEXT_SECONDARY),
                ]
            );
        }
        names_col.into()
    } else {
        Space::with_height(0).into()
    };

    // Exercises section
    let mut exercises_col = column![
        text("Writing Exercises:").size(11).color(Theme::TEXT_MUTED),
    ].spacing(1);
    for ex in data.exercises.iter().take(3) {
        exercises_col = exercises_col.push(
            row![
                text(format!("[{}]", ex.category.label()))
                    .size(9).color(Theme::TEXT_ACCENT),
                Space::with_width(4),
                text(ex.title.clone()).size(10).color(Theme::TEXT_PRIMARY),
                Space::with_width(4),
                text(format!("~{} min", ex.duration_minutes))
                    .size(9).color(Theme::TEXT_MUTED),
            ]
        );
    }

    // Search prompts example
    let search_results = writing_prompts::search_prompts("character");
    let search_hint = if !search_results.is_empty() {
        text(format!("{} prompts match 'character'", search_results.len()))
            .size(9).color(Theme::TEXT_MUTED)
    } else {
        text("").size(1)
    };

    // Exercise counts per difficulty
    let inter_exercises = writing_prompts::writing_exercises(Difficulty::Intermediate);
    let adv_exercises = writing_prompts::writing_exercises(Difficulty::Advanced);
    let exercise_summary = text(format!(
        "Exercises: {} beginner, {} intermediate, {} advanced",
        data.exercises.len(), inter_exercises.len(), adv_exercises.len()
    )).size(9).color(Theme::TEXT_MUTED);

    // Prompts by tag
    let tagged = writing_prompts::prompts_by_tag("emotion");
    let tag_hint = text(format!("{} prompts tagged 'emotion'", tagged.len()))
        .size(9).color(Theme::TEXT_MUTED);

    // All prompts for a category
    let all_freewrite = writing_prompts::all_prompts_for_category(PromptCategory::FreeWrite);
    let all_prompts_hint = text(format!("{} free-write prompts available", all_freewrite.len()))
        .size(9).color(Theme::TEXT_MUTED);

    let content = column![
        header,
        Space::with_height(4),
        daily_section,
        Space::with_height(4),
        scrollable(cat_row),
        Space::with_height(4),
        prompt_section,
        Space::with_height(4),
        char_section,
        plot_section,
        names_section,
        Space::with_height(4),
        exercises_col,
        exercise_summary,
        Space::with_height(2),
        search_hint,
        tag_hint,
        all_prompts_hint,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .style(theme::panel_style)
        .width(Length::Fill)
        .into()
}
