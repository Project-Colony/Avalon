/// DateTime format for InsertDateTime action
#[derive(Debug, Clone)]
pub enum DateTimeFormat {
    DateOnly,
    TimeOnly,
    DateTime,
    Iso8601,
}

impl DateTimeFormat {
    /// Format the current date/time according to this format
    pub fn format_now(&self) -> String {
        let now = chrono::Local::now();
        match self {
            DateTimeFormat::DateOnly => now.format(crate::core::DATE_FORMAT).to_string(),
            DateTimeFormat::TimeOnly => now.format("%H:%M").to_string(),
            DateTimeFormat::DateTime => now.format("%Y-%m-%d %H:%M").to_string(),
            DateTimeFormat::Iso8601 => now.format("%Y-%m-%dT%H:%M:%S%z").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datetime_format_now() {
        let date = DateTimeFormat::DateOnly.format_now();
        assert!(date.contains("-"), "Date should contain dashes: {}", date);
        assert_eq!(date.len(), 10); // YYYY-MM-DD

        let time = DateTimeFormat::TimeOnly.format_now();
        assert!(time.contains(":"), "Time should contain colon: {}", time);

        let datetime = DateTimeFormat::DateTime.format_now();
        assert!(datetime.contains(" "), "DateTime should contain space: {}", datetime);

        let iso = DateTimeFormat::Iso8601.format_now();
        assert!(iso.contains("T"), "ISO should contain T: {}", iso);
    }

    #[test]
    fn test_datetime_format_debug_clone() {
        let fmt = DateTimeFormat::Iso8601;
        let _cloned = fmt.clone();
        let debug = format!("{:?}", fmt);
        assert!(debug.contains("Iso8601"));
    }
}
