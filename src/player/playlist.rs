use crate::podcast::{db::Database as DBPod, Episode};
use crate::track::MediaType;
use crate::{
    config::Settings,
    track::Track,
    utils::{filetype_supported, get_app_config_path, get_parent_folder},
};
use anyhow::{bail, Result};
use pathdiff::diff_utf8_paths;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
#[cfg(not(any(feature = "mpv", feature = "gst")))]
use std::time::Duration;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Status {
    Running,
    #[default]
    Stopped,
    Paused,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "Running"),
            Self::Stopped => write!(f, "Stopped"),
            Self::Paused => write!(f, "Paused"),
        }
    }
}

#[derive(Clone, Copy, Default, Deserialize, Serialize)]
pub enum Loop {
    Single,
    #[default]
    Playlist,
    Queue,
}

#[allow(clippy::non_ascii_literal)]
impl Loop {
    pub fn display(self, display_symbol: bool) -> String {
        if display_symbol {
            match self {
                Self::Single => "🔂".to_string(),
                Self::Playlist => "🔁".to_string(),
                Self::Queue => "⬇".to_string(),
            }
        } else {
            match self {
                Self::Single => "single".to_string(),
                Self::Playlist => "playlist".to_string(),
                Self::Queue => "consume".to_string(),
            }
        }
    }
}

#[derive(Default)]
pub struct Playlist {
    tracks: VecDeque<Track>,
    current_track: Option<Track>,
    next_track: Option<Track>,
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    next_track_duration: Duration,
    // pub index: Option<usize>,
    status: Status,
    loop_mode: Loop,
    add_playlist_front: bool,
}

// #[allow(unused)]
impl Playlist {
    pub fn new(config: &Settings) -> Result<Self> {
        let mut tracks = Self::load()?;
        let mut current_track: Option<Track> = None;
        if let Some(track) = tracks.pop_front() {
            current_track = Some(track);
        }
        let loop_mode = config.loop_mode;
        let add_playlist_front = config.add_playlist_front;

        Ok(Self {
            tracks,
            current_track,
            next_track: None,
            #[cfg(not(any(feature = "mpv", feature = "gst")))]
            next_track_duration: Duration::from_secs(0),
            // index: Some(0),
            status: Status::Stopped,
            loop_mode,
            add_playlist_front,
        })
    }

    pub fn reload(&mut self) -> Result<()> {
        self.save()?;
        let mut tracks = Self::load()?;
        let mut current_track: Option<Track> = None;
        if let Some(track) = tracks.pop_front() {
            current_track = Some(track);
        }
        self.tracks = tracks;
        self.current_track = current_track;
        Ok(())
    }

    pub fn load() -> Result<VecDeque<Track>> {
        let mut path = get_app_config_path()?;
        path.push("playlist.log");

        let file = if let Ok(f) = File::open(path.as_path()) {
            f
        } else {
            File::create(path.as_path())?;
            File::open(path)?
        };
        let reader = BufReader::new(file);
        let lines: Vec<_> = reader
            .lines()
            .map(|line| line.unwrap_or_else(|_| "Error".to_string()))
            .collect();

        let mut playlist_items = VecDeque::new();
        let db_path = get_app_config_path()?;
        let db_podcast = DBPod::connect(&db_path)?;
        let podcasts = db_podcast
            .get_podcasts()
            .expect("failed to get podcasts from db.");
        for line in &lines {
            if let Ok(s) = Track::read_from_path(line, false) {
                playlist_items.push_back(s);
                continue;
            };
            if line.starts_with("http") {
                'outer: for pod in &podcasts {
                    for ep in &pod.episodes {
                        if &ep.url == line {
                            let track = Track::from_episode(ep);
                            playlist_items.push_back(track);
                            break 'outer;
                        }
                    }
                }
            }
        }

        Ok(playlist_items)
    }

    pub fn save(&mut self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("playlist.log");

        let file = File::create(path.as_path())?;
        let mut writer = BufWriter::new(file);
        let mut bytes = Vec::new();
        if let Some(track) = &self.current_track {
            if let Some(f) = track.file() {
                bytes.extend(f.as_bytes());
                bytes.extend("\n".as_bytes());
            }
        }
        for i in &self.tracks {
            if let Some(f) = i.file() {
                bytes.extend(f.as_bytes());
                bytes.extend("\n".as_bytes());
            }
        }

        writer.write_all(&bytes)?;
        writer.flush()?;

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn swap_down(&mut self, index: usize) {
        if index < self.len() - 1 {
            if let Some(track) = self.tracks.remove(index) {
                self.tracks.insert(index + 1, track);
            }
        }
    }

    pub fn swap_up(&mut self, index: usize) {
        if index > 0 {
            if let Some(track) = self.tracks.remove(index) {
                self.tracks.insert(index - 1, track);
            }
        }
    }

    pub fn get_current_track(&mut self) -> Option<String> {
        let mut result = None;
        if let Some(track) = &self.current_track {
            match track.media_type {
                Some(MediaType::Music) => {
                    if let Some(file) = track.file() {
                        result = Some(file.to_string());
                    }
                }
                Some(MediaType::Podcast) => {
                    if let Some(local_file) = &track.podcast_localfile {
                        let path = Path::new(&local_file);
                        if path.exists() {
                            return Some(local_file.clone());
                        }
                    }
                    if let Some(file) = track.file() {
                        result = Some(file.to_string());
                    }
                }
                None => {}
            }
        }
        result
    }

    pub fn fetch_next_track(&self) -> Option<&Track> {
        self.tracks.get(0)
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn is_stopped(&self) -> bool {
        self.status == Status::Stopped
    }

    pub fn is_paused(&self) -> bool {
        self.status == Status::Paused
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn handle_current_track(&mut self) {
        // eprintln!("handle current track");

        if let Some(song) = self.tracks.pop_front() {
            match self.loop_mode {
                Loop::Playlist => self.tracks.push_back(song.clone()),
                Loop::Single => self.tracks.push_front(song.clone()),
                Loop::Queue => {}
            }
            self.current_track = Some(song);
        } else {
            self.current_track = None;
            self.set_status(Status::Stopped);
        }
    }

    pub fn cycle_loop_mode(&mut self) -> Loop {
        match self.loop_mode {
            Loop::Queue => {
                self.loop_mode = Loop::Playlist;
            }
            Loop::Playlist => {
                self.loop_mode = Loop::Single;
                if let Some(song) = self.tracks.pop_back() {
                    self.tracks.push_front(song);
                }
            }
            Loop::Single => {
                self.loop_mode = Loop::Queue;
                if let Some(song) = self.tracks.pop_front() {
                    self.tracks.push_back(song);
                }
            }
        };
        self.loop_mode
    }

    #[cfg(feature = "webservice")]
    pub fn set_loop_mode(&mut self, loop_mode: Loop) -> Loop {
        match loop_mode {
            Loop::Single => {
                self.loop_mode = Loop::Playlist;
                self.cycle_loop_mode();
            }
            Loop::Playlist => {
                self.loop_mode = Loop::Queue;
                self.cycle_loop_mode();
            }
            Loop::Queue => {
                self.loop_mode = Loop::Single;
                self.cycle_loop_mode();
            }
        }
        self.loop_mode
    }

    #[cfg(feature = "webservice")]
    pub fn get_loop_mode(&self) -> Loop {
        self.loop_mode
    }

    // export to M3U
    pub fn save_m3u(&self, filename: &str) -> Result<()> {
        if self.tracks.is_empty() {
            bail!("No tracks in playlist, so no need to save.");
        }

        let parent_folder = get_parent_folder(filename);

        let m3u = self.get_m3u_file(&parent_folder);

        std::fs::write(filename, m3u)?;
        Ok(())
    }

    fn get_m3u_file(&self, parent_folder: &str) -> String {
        let mut m3u = String::from("#EXTM3U\n");
        for track in &self.tracks {
            if let Some(file) = track.file() {
                let path_relative = diff_utf8_paths(file, parent_folder);

                if let Some(p) = path_relative {
                    let path = format!("{p}\n");
                    m3u.push_str(&path);
                }
            }
        }
        m3u
    }

    pub fn toggle_add_front(&mut self) -> bool {
        self.add_playlist_front = !self.add_playlist_front;
        self.add_playlist_front
    }

    pub fn add_episode(&mut self, ep: &Episode) {
        let track = Track::from_episode(ep);

        if self.add_playlist_front {
            self.tracks.push_front(track);
            return;
        }
        self.tracks.push_back(track);
    }
    pub fn add_playlist(&mut self, mut vec: Vec<&str>) -> Result<()> {
        if self.add_playlist_front {
            vec.reverse();
            self.add_playlist_inside(vec)?;
            return Ok(());
        }

        self.add_playlist_inside(vec)?;
        Ok(())
    }

    fn add_playlist_inside(&mut self, vec: Vec<&str>) -> Result<()> {
        for item in vec {
            if !filetype_supported(item) {
                continue;
            }
            if !PathBuf::from(item).exists() {
                continue;
            }
            self.add_playlist_inside_inside(item)?;
        }
        Ok(())
    }

    fn add_playlist_inside_inside(&mut self, item: &str) -> Result<()> {
        let track = Track::read_from_path(item, false)?;
        if self.add_playlist_front {
            self.tracks.push_front(track);
            return Ok(());
        }
        self.tracks.push_back(track);
        Ok(())
    }

    pub fn tracks(&self) -> &VecDeque<Track> {
        &self.tracks
    }

    pub fn remove(&mut self, index: usize) -> Option<Track> {
        self.tracks.remove(index)
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.tracks.make_contiguous().shuffle(&mut rng);
    }

    pub fn remove_deleted_items(&mut self) {
        self.tracks
            .retain(|x| x.file().map_or(false, |p| Path::new(p).exists()));
    }

    pub fn push_front(&mut self, track: &Track) {
        self.tracks.push_front(track.clone());
    }

    pub fn handle_previous(&mut self) {
        if let Some(song) = self.tracks.pop_back() {
            self.tracks.push_front(song);
        }
        if let Some(song) = self.tracks.pop_back() {
            self.tracks.push_front(song);
        }
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.current_track.as_ref()
    }

    pub fn current_track_as_mut(&mut self) -> Option<&mut Track> {
        self.current_track.as_mut()
    }

    pub fn set_current_track(&mut self, track: Option<&Track>) {
        match track {
            Some(t) => self.current_track = Some(t.clone()),
            None => self.current_track = None,
        }
    }

    pub fn next_track(&self) -> Option<&Track> {
        self.next_track.as_ref()
    }

    pub fn set_next_track(&mut self, track: Option<&Track>) {
        match track {
            Some(t) => self.next_track = Some(t.clone()),
            None => self.next_track = None,
        }
    }

    pub fn has_next_track(&mut self) -> bool {
        self.next_track.is_some()
    }

    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    pub fn next_track_duration(&self) -> Duration {
        self.next_track_duration
    }

    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    pub fn set_next_track_duration(&mut self, d: Duration) {
        self.next_track_duration = d;
    }
}
