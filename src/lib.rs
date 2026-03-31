use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, Duration};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClockSessionError {
    #[error("Session is not paused")]
    NotPaused,
    #[error("Session is already paused")]
    Paused,
    #[error("Session is already finished")]
    AlreadyFinished,
    #[error("Effective duration is less than expected duration")]
    ExpectedDurationNotMet,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct TimeProvider {
    current_utc: OffsetDateTime
}

impl TimeProvider {
    pub fn new() -> Self {
        TimeProvider {
            current_utc: OffsetDateTime::now_utc() 
        }
    }

    pub fn update(&mut self) {
        self.current_utc = OffsetDateTime::now_utc();
    }

    pub fn current_utc(&self) -> OffsetDateTime {
        self.current_utc
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClockSessionSection {
    start: OffsetDateTime,
    duration: Option<Duration>,
    label: Option<String>
}

impl ClockSessionSection {
    pub fn new(time_provider: &TimeProvider, label: Option<String>) -> Self {
        ClockSessionSection {
            start: time_provider.current_utc(),
            duration: None,
            label: label,
        }
    }

    pub fn start(&self) -> OffsetDateTime {
        self.start
    }

    pub fn duration(&self, time_provider: &TimeProvider) -> Duration {
        if let Some(duration) = self.duration {
            duration
        } else {
            time_provider.current_utc() - self.start
        } 
    }

    pub fn is_finished(&self) -> bool {
        self.duration.is_some()
    }

    pub fn end(&mut self, time_provider: &TimeProvider) -> Result<(), ClockSessionError> {
        if self.is_finished() {
            return Err(ClockSessionError::AlreadyFinished)
        }
        self.duration = Some(time_provider.current_utc() - self.start);
        Ok(())
    }

    pub fn maybe_end(&mut self, time_provider: &TimeProvider) {
        if self.is_finished() {
            return;
        }
        self.duration = Some(time_provider.current_utc() - self.start)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClockSession {
    time_provider: TimeProvider,
    start: OffsetDateTime,
    expected_duration: Option<Duration>,
    effective_duration: Option<Duration>,
    pauses: Vec<ClockSessionSection>,
    sections: Vec<ClockSessionSection>,
}

impl ClockSession {
    pub fn new(time_provider: TimeProvider, expected_duration: Option<Duration>, start_label: Option<String>) -> Self {
        ClockSession {
            time_provider: time_provider,
            start: time_provider.current_utc(),
            expected_duration: expected_duration,
            effective_duration: None,
            pauses: vec![],
            sections: vec![ClockSessionSection::new(&time_provider, start_label)],
        }
    }

    pub fn is_finished(&self) -> bool {
        self.effective_duration.is_some()
    }

    pub fn end(&mut self, force: bool) -> Result<(), ClockSessionError> {
        if self.is_finished() {
            return Err(ClockSessionError::AlreadyFinished);
        }
        let effective_duration = self.effective_duration();
        if !force && effective_duration < self.expected_duration.unwrap_or(Duration::ZERO) {
            return Err(ClockSessionError::ExpectedDurationNotMet);
        }

        if let Some(section) = self.sections.last_mut() {
            section.maybe_end(&self.time_provider);
        }
        if self.is_paused() {
            self.resume()?;
        }
        
        self.effective_duration = Some(effective_duration);
        Ok(())
    }

    pub fn new_section(&mut self, label: Option<String>) {
        if let Some(section) = self.sections.last_mut() {
            section.maybe_end(&self.time_provider);
        }
        self.sections.push(ClockSessionSection::new(&self.time_provider, label));
    }

    pub fn is_paused(&self) -> bool {
        if let Some(pause) = self.pauses.last() {
            pause.duration.is_none()
        } else {
            false
        }
    }

    pub fn pause(&mut self) -> Result<(), ClockSessionError> {
        if self.is_paused() {
            return Err(ClockSessionError::Paused);
        }

        let pause = ClockSessionSection::new(&self.time_provider, None);
        self.pauses.push(pause);
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), ClockSessionError> {
        if !self.is_paused() {
            return Err(ClockSessionError::NotPaused);
        }

        if let Some(pause) = self.pauses.last_mut() {
            pause.end(&self.time_provider)?;
        }
        Ok(())
    }

    pub fn total_paused_duration(&self) -> Duration {
        self.pauses.iter().fold(Duration::ZERO, |acc, pause| {
            acc + pause.duration(&self.time_provider)
        })
    }

    pub fn effective_duration(&self) -> Duration {
        if let Some(effective_duration) = self.effective_duration {
            effective_duration
        } else {
            self.time_provider.current_utc() - self.start - self.total_paused_duration()
        }
    }

    pub fn update_time_provider(&mut self) {
        self.time_provider.update();
    }

    pub fn start(&self) -> OffsetDateTime {
        self.start
    }

    pub fn expected_duration(&self) -> Option<Duration> {
        self.expected_duration
    }

    pub fn pauses(&self) -> &Vec<ClockSessionSection> {
        &self.pauses
    }

    pub fn sections(&self) -> &Vec<ClockSessionSection> {
        &self.sections
    }

    pub fn time_provider(&self) -> &TimeProvider {
        &self.time_provider
    }
}
