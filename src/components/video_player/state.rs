use crate::timecode::*;
use crate::utils::*;
use leptos::prelude::*;
use smart_default::SmartDefault;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum PlayingState {
    #[default]
    StartPause,
    EndPause,
    Pause,
    PrecisePause,
    Play,
}

impl PlayingState {
    pub fn toggle_play(&mut self) {
        if *self == PlayingState::Play {
            *self = PlayingState::PrecisePause
        } else {
            *self = PlayingState::Play
        }
    }

    pub fn is_playing(&self) -> bool {
        *self == PlayingState::Play
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum WaitingState {
    #[default]
    Ready, // Not waiting
    Buffering, // Waiting for more data
    Loading,   // Initial load
    Seeking,   // Wait for seek to complete
    Stalled,   // Playback stalled
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VideoInfo {
    pub src: String,
    pub proxy: Option<String>,
    pub poster: Option<String>,
    pub fps: f64,
    pub aspect_ratio: f64,
    pub end_frame: u32,
}

impl VideoInfo {
    pub fn set_duration(&mut self, duration: f64) {
        let total_frames = frame_from_time(duration, self.fps);
        self.end_frame = total_frames.saturating_sub(1);
    }

    pub fn frame_from_time(&self, time: f64) -> u32 {
        frame_from_time(time, self.fps).min(self.end_frame)
    }

    pub fn frame_from_pos(&self, pos: f64) -> u32 {
        frame_from_pos(pos, self.end_frame)
    }

    pub fn time_from_frame(&self, frame: u32) -> f64 {
        time_from_frame(frame.min(self.end_frame), self.fps)
    }

    pub fn duration(&self) -> f64 {
        time_from_frame(self.end_frame, self.fps)
    }

    pub fn time_string(&self, frame: u32, time_format: TimeFormat) -> String {
        match time_format {
            TimeFormat::Frames => frame.to_string(),
            TimeFormat::Timecode => self.timecode_string(frame),
        }
    }

    pub fn frame_from_time_string(
        &self,
        time_string: &str,
        time_format: TimeFormat,
    ) -> Option<u32> {
        match time_format {
            TimeFormat::Frames => {
                if let Ok(f) = time_string.parse::<u32>() {
                    return Some(f.min(self.end_frame));
                }
            }
            TimeFormat::Timecode => {
                if let Ok(tc) = time_string.parse::<Timecode>() {
                    let f = tc.to_frames(self.fps);
                    return Some(f.min(self.end_frame));
                }
            }
        }
        None
    }

    pub fn end_time_string(&self, time_format: TimeFormat) -> String {
        match time_format {
            TimeFormat::Frames => self.end_frame.to_string(),
            TimeFormat::Timecode => self.end_timecode_string(),
        }
    }

    pub fn timecode_string(&self, frame: u32) -> String {
        let show_hours = hours_from_frames(self.end_frame, self.fps) != 0;
        let t = Timecode::from_frames(frame, self.fps);
        t.to_string_opt(show_hours, true)
    }

    pub fn end_timecode_string(&self) -> String {
        self.timecode_string(self.end_frame)
    }

    pub fn progress(&self, frame: u32) -> f64 {
        frame as f64 / (self.end_frame + 1) as f64
    }
}

#[derive(Clone, Copy, Debug, SmartDefault, PartialEq)]
pub struct AudioState {
    #[default = 1.0]
    pub volume: f64,
    pub is_muted: bool,
}

impl AudioState {
    pub fn set_volume(&mut self, volume: f64) {
        let volume = volume.clamp(0.0, 1.0);
        if volume > 0.0 {
            self.is_muted = false;
        }
        self.volume = volume;
    }

    pub fn set_muted(&mut self, mute: bool) {
        if !mute && self.volume == 0.0 {
            self.volume = 1.0;
        }
        self.is_muted = mute;
    }

    pub fn toggle_mute(&mut self) {
        self.set_muted(!self.is_muted());
    }

    pub fn is_muted(&self) -> bool {
        self.is_muted || self.volume <= 0.0
    }

    pub fn volume(&self) -> f64 {
        if self.is_muted { 0.0 } else { self.volume }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeFormat {
    #[default]
    Frames,
    Timecode,
}

pub fn frame_from_time(time: f64, fps: f64) -> u32 {
    (time * fps + 0.01) as u32
}

pub fn frame_from_pos(pos: f64, end_frame: u32) -> u32 {
    let pos = pos.max(0.0);
    let total_frames = end_frame + 1;
    ((pos * total_frames as f64 + 0.01) as u32).min(end_frame)
}

pub fn time_from_frame(frame: u32, fps: f64) -> f64 {
    (frame as f64 + 0.01) / fps
}

pub fn calc_video_box(
    container_width: f64,
    container_height: f64,
    video_aspect: f64,
) -> (f64, f64, f64, f64) {
    let container_aspect = container_width / container_height;
    if video_aspect < container_aspect {
        let w = video_aspect * container_height;
        let h = container_height;
        let x = (container_width - w) / 2.0;
        let y = 0.0;
        (w, h, x, y)
    } else {
        let w = container_width;
        let h = container_width / video_aspect;
        let x = 0.0;
        let y = (container_height - h) / 2.0;
        (w, h, x, y)
    }
}
