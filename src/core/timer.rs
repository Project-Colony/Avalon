use std::time::{Duration, Instant};

/// A writing focus timer for timed writing sessions
/// Supports Pomodoro-style intervals and sprint timers
#[derive(Debug)]
pub struct WritingTimer {
    /// Current timer state
    state: TimerState,
    /// Duration of the current timer (in seconds)
    duration_secs: u64,
    /// Start time of the current timer
    start_time: Option<Instant>,
    /// Words written at start of session
    start_word_count: usize,
    /// Completed sessions
    completed_sessions: Vec<TimerSession>,
    /// Timer preset being used
    pub preset: TimerPreset,
}

/// Current state of the timer
#[derive(Debug, Clone, PartialEq)]
pub enum TimerState {
    Idle,
    Running,
    Paused(Duration), // elapsed before pause
    Completed,
}

/// A preset timer configuration
#[derive(Debug, Clone, PartialEq)]
pub enum TimerPreset {
    /// Short sprint (10 minutes)
    Sprint,
    /// Standard Pomodoro (25 minutes)
    Pomodoro,
    /// Long session (45 minutes)
    LongSession,
    /// Hour-long session
    HourSession,
    /// Custom duration in seconds
    Custom(u64),
}

impl TimerPreset {
    pub fn duration_secs(&self) -> u64 {
        match self {
            TimerPreset::Sprint => 10 * 60,
            TimerPreset::Pomodoro => 25 * 60,
            TimerPreset::LongSession => 45 * 60,
            TimerPreset::HourSession => 60 * 60,
            TimerPreset::Custom(secs) => *secs,
        }
    }

    pub fn label(&self) -> String {
        match self {
            TimerPreset::Sprint => "Sprint (10 min)".to_string(),
            TimerPreset::Pomodoro => "Pomodoro (25 min)".to_string(),
            TimerPreset::LongSession => "Long (45 min)".to_string(),
            TimerPreset::HourSession => "Hour (60 min)".to_string(),
            TimerPreset::Custom(secs) => {
                let mins = secs / 60;
                if mins > 0 { format!("Custom ({} min)", mins) }
                else { format!("Custom ({} sec)", secs) }
            }
        }
    }

    pub fn all() -> Vec<TimerPreset> {
        vec![
            TimerPreset::Sprint,
            TimerPreset::Pomodoro,
            TimerPreset::LongSession,
            TimerPreset::HourSession,
        ]
    }
}

/// A completed timer session
#[derive(Debug, Clone)]
pub struct TimerSession {
    pub duration: Duration,
    pub words_written: usize,
    pub preset: TimerPreset,
    pub completed: bool,
}

impl TimerSession {
    pub fn words_per_minute(&self) -> f64 {
        let minutes = self.duration.as_secs_f64() / 60.0;
        if minutes > 0.0 { self.words_written as f64 / minutes } else { 0.0 }
    }
}

impl WritingTimer {
    pub fn new() -> Self {
        Self {
            state: TimerState::Idle,
            duration_secs: TimerPreset::Pomodoro.duration_secs(),
            start_time: None,
            start_word_count: 0,
            completed_sessions: Vec::new(),
            preset: TimerPreset::Pomodoro,
        }
    }

    /// Start the timer with the current preset
    pub fn start(&mut self, current_word_count: usize) {
        self.duration_secs = self.preset.duration_secs();
        self.state = TimerState::Running;
        self.start_time = Some(Instant::now());
        self.start_word_count = current_word_count;
    }

    /// Pause the running timer
    pub fn pause(&mut self) {
        if self.state == TimerState::Running {
            if let Some(start) = self.start_time {
                self.state = TimerState::Paused(start.elapsed());
            }
        }
    }

    /// Resume a paused timer
    pub fn resume(&mut self) {
        if let TimerState::Paused(elapsed) = self.state {
            // Set start_time so that elapsed() returns the correct value
            self.start_time = Some(Instant::now() - elapsed);
            self.state = TimerState::Running;
        }
    }

    /// Stop the timer and record the session
    pub fn stop(&mut self, current_word_count: usize) {
        let elapsed = self.elapsed();
        let words = current_word_count.saturating_sub(self.start_word_count);

        self.completed_sessions.push(TimerSession {
            duration: elapsed,
            words_written: words,
            preset: self.preset.clone(),
            completed: self.is_complete(),
        });

        self.state = TimerState::Idle;
        self.start_time = None;
    }

    /// Check the timer (call on tick) — returns true if timer just completed
    pub fn tick(&mut self) -> bool {
        if self.state == TimerState::Running {
            if self.elapsed().as_secs() >= self.duration_secs {
                self.state = TimerState::Completed;
                return true;
            }
        }
        false
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.state = TimerState::Idle;
        self.start_time = None;
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        match &self.state {
            TimerState::Running => {
                self.start_time.map(|t| t.elapsed()).unwrap_or_default()
            }
            TimerState::Paused(d) => *d,
            TimerState::Completed => Duration::from_secs(self.duration_secs),
            TimerState::Idle => Duration::ZERO,
        }
    }

    /// Get remaining time
    pub fn remaining(&self) -> Duration {
        if self.state == TimerState::Idle {
            return Duration::ZERO;
        }
        let elapsed = self.elapsed();
        let total = Duration::from_secs(self.duration_secs);
        if elapsed >= total { Duration::ZERO } else { total - elapsed }
    }

    /// Format remaining time as MM:SS
    pub fn remaining_display(&self) -> String {
        let remaining = self.remaining();
        let mins = remaining.as_secs() / 60;
        let secs = remaining.as_secs() % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    /// Format elapsed time as MM:SS
    pub fn elapsed_display(&self) -> String {
        let elapsed = self.elapsed();
        let mins = elapsed.as_secs() / 60;
        let secs = elapsed.as_secs() % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    /// Whether the timer is currently active (running or paused)
    pub fn is_active(&self) -> bool {
        matches!(self.state, TimerState::Running | TimerState::Paused(_))
    }

    /// Whether the timer is running
    pub fn is_running(&self) -> bool {
        self.state == TimerState::Running
    }

    /// Whether the timer has completed
    pub fn is_complete(&self) -> bool {
        self.state == TimerState::Completed
    }

    /// Get the current state
    pub fn state(&self) -> &TimerState {
        &self.state
    }

    /// Set the preset and update duration
    pub fn set_preset(&mut self, preset: TimerPreset) {
        if self.state == TimerState::Idle {
            self.duration_secs = preset.duration_secs();
            self.preset = preset;
        }
    }

    /// Get completed sessions
    pub fn sessions(&self) -> &[TimerSession] {
        &self.completed_sessions
    }

    /// Get the total number of completed sessions
    pub fn completed_count(&self) -> usize {
        self.completed_sessions.iter().filter(|s| s.completed).count()
    }

    /// Get the total words written across all sessions
    pub fn total_words(&self) -> usize {
        self.completed_sessions.iter().map(|s| s.words_written).sum()
    }

    /// Get the total time spent across all sessions
    pub fn total_time(&self) -> Duration {
        self.completed_sessions.iter().map(|s| s.duration).sum()
    }

    /// Get average words per minute across all sessions
    pub fn avg_wpm(&self) -> f64 {
        let total_mins = self.total_time().as_secs_f64() / 60.0;
        if total_mins > 0.0 {
            self.total_words() as f64 / total_mins
        } else {
            0.0
        }
    }

    /// Get a summary for display
    pub fn summary(&self) -> String {
        let sessions = self.completed_count();
        if sessions == 0 {
            return "No sessions yet".to_string();
        }
        format!(
            "{} session{}, {} words, {:.1} avg WPM",
            sessions,
            if sessions == 1 { "" } else { "s" },
            self.total_words(),
            self.avg_wpm(),
        )
    }

    /// Progress as a fraction (0.0 - 1.0)
    pub fn progress(&self) -> f64 {
        if self.duration_secs == 0 { return 0.0; }
        (self.elapsed().as_secs_f64() / self.duration_secs as f64).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_timer_new() {
        let timer = WritingTimer::new();
        assert!(!timer.is_active());
        assert!(!timer.is_running());
        assert_eq!(timer.state(), &TimerState::Idle);
    }

    #[test]
    fn test_timer_start() {
        let mut timer = WritingTimer::new();
        timer.start(100);
        assert!(timer.is_active());
        assert!(timer.is_running());
        assert_eq!(timer.state(), &TimerState::Running);
    }

    #[test]
    fn test_timer_pause_resume() {
        let mut timer = WritingTimer::new();
        timer.start(0);
        sleep(Duration::from_millis(50));
        timer.pause();
        assert!(timer.is_active());
        assert!(!timer.is_running());

        let paused_elapsed = timer.elapsed();
        sleep(Duration::from_millis(50));
        // Time shouldn't advance while paused
        assert_eq!(timer.elapsed(), paused_elapsed);

        timer.resume();
        assert!(timer.is_running());
    }

    #[test]
    fn test_timer_stop() {
        let mut timer = WritingTimer::new();
        timer.start(100);
        timer.stop(150);
        assert!(!timer.is_active());
        assert_eq!(timer.sessions().len(), 1);
        assert_eq!(timer.sessions()[0].words_written, 50);
    }

    #[test]
    fn test_timer_reset() {
        let mut timer = WritingTimer::new();
        timer.start(0);
        timer.reset();
        assert!(!timer.is_active());
        assert_eq!(timer.state(), &TimerState::Idle);
    }

    #[test]
    fn test_timer_preset_durations() {
        assert_eq!(TimerPreset::Sprint.duration_secs(), 600);
        assert_eq!(TimerPreset::Pomodoro.duration_secs(), 1500);
        assert_eq!(TimerPreset::LongSession.duration_secs(), 2700);
        assert_eq!(TimerPreset::HourSession.duration_secs(), 3600);
        assert_eq!(TimerPreset::Custom(120).duration_secs(), 120);
    }

    #[test]
    fn test_timer_preset_labels() {
        assert!(TimerPreset::Sprint.label().contains("10"));
        assert!(TimerPreset::Pomodoro.label().contains("25"));
        assert!(TimerPreset::Custom(120).label().contains("2 min"));
    }

    #[test]
    fn test_timer_preset_all() {
        let all = TimerPreset::all();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_timer_remaining_display() {
        let mut timer = WritingTimer::new();
        timer.set_preset(TimerPreset::Sprint);
        assert_eq!(timer.remaining_display(), "00:00"); // Idle state
        timer.start(0);
        let display = timer.remaining_display();
        // Should be close to 10:00 (600 seconds)
        assert!(display == "10:00" || display.starts_with("09:5"),
            "Expected ~10:00 but got {}", display);
    }

    #[test]
    fn test_timer_set_preset() {
        let mut timer = WritingTimer::new();
        timer.set_preset(TimerPreset::Sprint);
        assert_eq!(timer.preset, TimerPreset::Sprint);
        assert_eq!(timer.duration_secs, 600);
    }

    #[test]
    fn test_timer_set_preset_while_running() {
        let mut timer = WritingTimer::new();
        timer.start(0);
        timer.set_preset(TimerPreset::Sprint);
        // Should NOT change while running
        assert_eq!(timer.preset, TimerPreset::Pomodoro);
    }

    #[test]
    fn test_timer_completed_count() {
        let mut timer = WritingTimer::new();
        timer.set_preset(TimerPreset::Custom(0)); // 0 second timer
        timer.start(0);
        timer.tick(); // Should complete immediately
        timer.stop(10);
        assert_eq!(timer.completed_count(), 1);
    }

    #[test]
    fn test_timer_total_words() {
        let mut timer = WritingTimer::new();
        timer.start(0);
        timer.stop(100);
        timer.start(100);
        timer.stop(250);
        assert_eq!(timer.total_words(), 250);
    }

    #[test]
    fn test_timer_summary_no_sessions() {
        let timer = WritingTimer::new();
        assert_eq!(timer.summary(), "No sessions yet");
    }

    #[test]
    fn test_timer_progress() {
        let mut timer = WritingTimer::new();
        assert_eq!(timer.progress(), 0.0);
        timer.start(0);
        assert!(timer.progress() < 0.01); // Just started, nearly 0
    }

    #[test]
    fn test_session_words_per_minute() {
        let session = TimerSession {
            duration: Duration::from_secs(60),
            words_written: 30,
            preset: TimerPreset::Sprint,
            completed: true,
        };
        assert!((session.words_per_minute() - 30.0).abs() < 0.1);
    }

    #[test]
    fn test_elapsed_display() {
        let mut timer = WritingTimer::new();
        timer.start(0);
        sleep(Duration::from_millis(50));
        let display = timer.elapsed_display();
        assert!(display.starts_with("00:0")); // Should be < 1 sec
    }

    #[test]
    fn test_timer_custom_preset() {
        let mut timer = WritingTimer::new();
        timer.set_preset(TimerPreset::Custom(300));
        assert_eq!(timer.preset, TimerPreset::Custom(300));
        assert_eq!(timer.duration_secs, 300);
    }

    #[test]
    fn test_timer_custom_short_label() {
        let preset = TimerPreset::Custom(30);
        assert!(preset.label().contains("30 sec"));
    }

    #[test]
    fn test_timer_progress_capped_at_one() {
        let mut timer = WritingTimer::new();
        timer.set_preset(TimerPreset::Custom(0));
        timer.start(0);
        // 0 second timer — should immediately be complete
        assert!(timer.progress() <= 1.0);
    }

    #[test]
    fn test_timer_remaining_zero_when_idle() {
        let timer = WritingTimer::new();
        assert_eq!(timer.remaining(), Duration::ZERO);
    }

    #[test]
    fn test_timer_elapsed_idle() {
        let timer = WritingTimer::new();
        assert_eq!(timer.elapsed(), Duration::ZERO);
    }

    #[test]
    fn test_timer_elapsed_completed() {
        let mut timer = WritingTimer::new();
        timer.set_preset(TimerPreset::Custom(60));
        timer.start(0);
        // Manually set state to Completed
        timer.state = TimerState::Completed;
        assert_eq!(timer.elapsed(), Duration::from_secs(60));
    }

    #[test]
    fn test_timer_multiple_sessions() {
        let mut timer = WritingTimer::new();
        timer.start(0);
        timer.stop(100);
        timer.start(100);
        timer.stop(200);
        timer.start(200);
        timer.stop(350);

        assert_eq!(timer.sessions().len(), 3);
        assert_eq!(timer.total_words(), 350);
    }

    #[test]
    fn test_timer_avg_wpm_no_sessions() {
        let timer = WritingTimer::new();
        assert_eq!(timer.avg_wpm(), 0.0);
    }

    #[test]
    fn test_timer_summary_with_sessions() {
        let mut timer = WritingTimer::new();
        timer.set_preset(TimerPreset::Custom(0));
        timer.start(0);
        timer.tick(); // Complete immediately
        timer.stop(100);

        let summary = timer.summary();
        assert!(summary.contains("1 session"));
        assert!(summary.contains("100 words"));
    }

    #[test]
    fn test_timer_pause_while_idle() {
        let mut timer = WritingTimer::new();
        timer.pause(); // Should be a no-op
        assert_eq!(timer.state(), &TimerState::Idle);
    }

    #[test]
    fn test_timer_resume_while_idle() {
        let mut timer = WritingTimer::new();
        timer.resume(); // Should be a no-op
        assert_eq!(timer.state(), &TimerState::Idle);
    }

    #[test]
    fn test_timer_tick_while_idle() {
        let mut timer = WritingTimer::new();
        assert!(!timer.tick()); // Should return false
    }

    #[test]
    fn test_timer_total_time() {
        let mut timer = WritingTimer::new();
        timer.start(0);
        sleep(Duration::from_millis(50));
        timer.stop(10);
        timer.start(10);
        sleep(Duration::from_millis(50));
        timer.stop(20);

        let total = timer.total_time();
        assert!(total.as_millis() >= 80); // At least some time
    }

    #[test]
    fn test_timer_is_complete_false() {
        let timer = WritingTimer::new();
        assert!(!timer.is_complete());
    }

    #[test]
    fn test_session_wpm_zero_duration() {
        let session = TimerSession {
            duration: Duration::ZERO,
            words_written: 100,
            preset: TimerPreset::Sprint,
            completed: true,
        };
        assert_eq!(session.words_per_minute(), 0.0);
    }
}
