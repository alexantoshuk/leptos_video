use regex::Regex;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TimecodeItem {
    #[default]
    Hours = 0,
    Minutes = 1,
    Seconds = 2,
    Frames = 3,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Timecode {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u32,
}

impl Timecode {
    pub fn new(hours: u8, minutes: u8, seconds: u8, frames: u32) -> Option<Self> {
        if hours < 24 && minutes < 60 && seconds < 60 {
            Some(Self {
                hours,
                minutes,
                seconds,
                frames,
            })
        } else {
            None
        }
    }

    pub fn from_frame(frame: u32, fps: f64) -> Self {
        assert!(fps > 0.0);
        let total_seconds = frame as f64 / fps;

        let hours = (total_seconds / 3600.0).floor() as u8;
        let minutes = ((total_seconds / 60.0) % 60.0).floor() as u8;
        let seconds = (total_seconds % 60.0).floor() as u8;
        let fps_u32 = fps.round() as u32;
        let frames = frame % fps_u32;

        Self {
            hours,
            minutes,
            seconds,
            frames,
        }
    }

    pub fn hours(frame: u32, fps: f64) -> u8 {
        assert!(fps > 0.0);
        let total_seconds = frame as f64 / fps;
        let hours = (total_seconds / 3600.0).floor() as u8;
        hours
    }

    pub fn first_nonzero_item(&self) -> TimecodeItem {
        if self.hours != 0 {
            TimecodeItem::Hours
        } else if self.minutes != 0 {
            TimecodeItem::Minutes
        } else if self.seconds != 0 {
            TimecodeItem::Seconds
        } else {
            TimecodeItem::Frames
        }
    }

    pub fn to_string_opt(&self, show_hours: bool, show_frames: bool) -> String {
        format!(
            "{}{:02}:{:02}{}",
            if show_hours {
                format!("{:02}:", self.hours)
            } else {
                "".into()
            },
            self.minutes,
            self.seconds,
            if show_frames {
                format!(":{:02}", self.frames)
            } else {
                "".into()
            }
        )
    }
}

impl std::fmt::Display for Timecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}:{:02}",
            self.hours, self.minutes, self.seconds, self.frames
        )
    }
}

impl FromStr for Timecode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^(\d{2}):(\d{2}):(\d{2}):(\d{2})$").unwrap();
        if let Some(caps) = re.captures(s) {
            let hours = caps[1].parse().map_err(|_| "Invalid hours")?;
            let minutes = caps[2].parse().map_err(|_| "Invalid minutes")?;
            let seconds = caps[3].parse().map_err(|_| "Invalid seconds")?;
            let frames = caps[4].parse().map_err(|_| "Invalid frames")?;

            Self::new(hours, minutes, seconds, frames)
                .ok_or_else(|| "Timecode values out of range".to_string())
        } else {
            Err("Invalid timecode format".to_string())
        }
    }
}

pub fn num_digits(n: u32) -> u8 {
    if n == 0 {
        1 // Zero has one digit
    } else {
        (n as f64).log10().ceil() as u8
    }
}
