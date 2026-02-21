use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// An annotation (inline comment) attached to a text range in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: Uuid,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub color: AnnotationColor,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationColor {
    Yellow,
    Blue,
    Green,
    Red,
    Purple,
}

impl Annotation {
    pub fn new(start: usize, end: usize, text: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            start,
            end,
            text: text.to_string(),
            author: String::new(),
            created_at: Utc::now(),
            color: AnnotationColor::Yellow,
            resolved: false,
        }
    }
}

impl AnnotationColor {
    pub fn to_hex(&self) -> &str {
        match self {
            AnnotationColor::Yellow => "#f1c40f",
            AnnotationColor::Blue => "#3498db",
            AnnotationColor::Green => "#2ecc71",
            AnnotationColor::Red => "#e74c3c",
            AnnotationColor::Purple => "#9b59b6",
        }
    }

    pub fn to_iced_color(&self) -> iced::Color {
        let hex = self.to_hex().trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
        iced::Color::from_rgb(r, g, b)
    }
}
