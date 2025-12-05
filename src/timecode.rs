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

    pub fn from_frames(frames: u32, fps: f64) -> Self {
        assert!(fps > 0.0);
        let total_seconds = frames as f64 / fps;

        let hours = (total_seconds / 3600.0).floor() as u8;
        let minutes = ((total_seconds / 60.0) % 60.0).floor() as u8;
        let seconds = (total_seconds % 60.0).floor() as u8;
        let fps_u32 = fps.round() as u32;
        let frames = frames % fps_u32;

        Self {
            hours,
            minutes,
            seconds,
            frames,
        }
    }

    pub fn normalized(&self, fps: f64) -> Self {
        Timecode::from_frames(self.to_frames(fps), fps)
    }

    pub fn to_frames(&self, fps: f64) -> u32 {
        assert!(fps > 0.0);
        let seconds = total_seconds(self.hours as u32, self.minutes as u32, self.seconds as u32);
        (seconds as f64 * fps).round() as u32 + self.frames
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
        let mut parts: Vec<u32> = Vec::with_capacity(4);
        for item in s.split(':') {
            let parsed_item = item
                .parse::<u32>()
                .map_err(|_| format!("Invalid timecode item '{item}' "))?;
            parts.push(parsed_item);
        }

        let frames = if let Some(v) = parts.pop() { v } else { 0 };
        let seconds = if let Some(v) = parts.pop() { v } else { 0 };
        let minutes = if let Some(v) = parts.pop() { v } else { 0 };
        let hours = if let Some(v) = parts.pop() { v } else { 0 };

        Ok(Timecode {
            hours: hours as u8,
            minutes: minutes as u8,
            seconds: seconds as u8,
            frames,
        })
    }
}

pub fn num_digits(n: u32) -> u8 {
    if n == 0 {
        1 // Zero has one digit
    } else {
        (n as f64).log10().ceil() as u8
    }
}

pub fn total_seconds(hours: u32, minutes: u32, seconds: u32) -> u32 {
    hours
        .saturating_mul(3600)
        .saturating_add(minutes.saturating_mul(60).saturating_add(seconds))
}

pub fn hours_from_frames(frames: u32, fps: f64) -> u8 {
    assert!(fps > 0.0);
    let total_seconds = frames as f64 / fps;
    let hours = (total_seconds / 3600.0).floor() as u8;
    hours
}
