/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
mod key;
mod theme;

use crate::player::Loop;
use crate::ui::components::Xywh;
use crate::utils::get_app_config_path;
use anyhow::Result;
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
pub use key::{BindingForEvent, Keys, ALT_SHIFT, CONTROL_ALT, CONTROL_ALT_SHIFT, CONTROL_SHIFT};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
pub use theme::{load_alacritty, ColorTermusic, StyleColorSymbol};

// pub const MUSIC_DIR: [&str; 2] = ["~/Music/mp3", "~/Music"];
// pub const PODCAST_DIR: &str = "~/.cache/termusic/podcast";

lazy_static! {
    static ref MUSIC_DIR: Vec<String> = {
        let mut vec = Vec::new();
        let mut path = dirs::audio_dir()
            .unwrap_or_else(|| PathBuf::from(shellexpand::tilde("~/Music").to_string()));
        path.push("mp3");
        // if !path.exists() {
        //     std::fs::create_dir_all(path.as_path()).unwrap_or_else(|_| {
        //         panic!(
        //             "create music dir failed: {}",
        //             path.as_path().to_string_lossy()
        //         )
        //     });
        // }
        vec.push(path.as_path().to_string_lossy().to_string());
        path.pop();
        vec.push(path.as_path().to_string_lossy().to_string());
        vec
    };
    static ref PODCAST_DIR: String = {
        let mut path = dirs::audio_dir().unwrap_or_else(|| PathBuf::from(shellexpand::tilde("~/Music").to_string()));
        path.push(Path::new("podcast"));
        path.as_path().to_string_lossy().to_string()
    };
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeekStep {
    Short,
    Long,
    Auto,
}

impl std::fmt::Display for SeekStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let seek_step = match self {
            Self::Short => "short(5 seconds)",
            Self::Long => "long(30 seconds)",
            Self::Auto => "auto(depend on audio length)",
        };
        write!(f, "{seek_step}")
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LastPosition {
    Yes,
    No,
    Auto,
}

impl std::fmt::Display for LastPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let save_last_position = match self {
            Self::Yes => "yes",
            Self::No => "no",
            Self::Auto => "auto",
        };
        write!(f, "{save_last_position}")
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct Settings {
    pub music_dir: Vec<String>,
    #[serde(skip)]
    pub music_dir_from_cli: Option<String>,
    #[serde(skip)]
    pub disable_album_art_from_cli: bool,
    #[serde(skip)]
    pub disable_discord_rpc_from_cli: bool,
    #[serde(skip)]
    pub max_depth_cli: usize,
    pub loop_mode: Loop,
    pub volume: i32,
    pub speed: i32,
    pub add_playlist_front: bool,
    pub gapless: bool,
    pub podcast_simultanious_download: usize,
    pub podcast_max_retries: usize,
    pub podcast_dir: String,
    pub seek_step: SeekStep,
    pub remember_last_played_position: LastPosition,
    pub enable_exit_confirmation: bool,
    pub playlist_display_symbol: bool,
    pub playlist_select_random_track_quantity: u32,
    pub playlist_select_random_album_quantity: u32,
    pub theme_selected: String,
    pub album_photo_xywh: Xywh,
    pub style_color_symbol: StyleColorSymbol,
    pub keys: Keys,
    #[cfg(feature = "webservice")]
    #[serde(skip)]
    pub web_service_addr: Option<String>,
    #[cfg(feature = "webservice")]
    #[serde(skip)]
    pub web_service_token: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        // let absolute_dir = shellexpand::tilde(&MUSIC_DIR).to_string();
        // let path = Path::new(&dir);
        // if path.exists() {
        // }
        Self {
            music_dir: MUSIC_DIR.to_vec(),
            music_dir_from_cli: None,
            loop_mode: Loop::Queue,
            volume: 70,
            speed: 10,
            add_playlist_front: false,
            gapless: true,
            remember_last_played_position: LastPosition::Auto,
            enable_exit_confirmation: true,
            playlist_display_symbol: true,
            keys: Keys::default(),
            theme_selected: "default".to_string(),
            style_color_symbol: StyleColorSymbol::default(),
            album_photo_xywh: Xywh::default(),
            playlist_select_random_track_quantity: 20,
            playlist_select_random_album_quantity: 5,
            disable_album_art_from_cli: false,
            disable_discord_rpc_from_cli: false,
            max_depth_cli: 4,
            podcast_simultanious_download: 3,
            podcast_dir: PODCAST_DIR.to_string(),
            podcast_max_retries: 3,
            seek_step: SeekStep::Auto,
            #[cfg(feature = "webservice")]
            web_service_addr: None,
            #[cfg(feature = "webservice")]
            web_service_token: None,
        }
    }
}

impl Settings {
    pub fn save(&self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("config.toml");
        let string = toml::to_string(self)?;

        fs::write(path.to_string_lossy().as_ref(), string)?;

        Ok(())
    }

    pub fn load(&mut self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("config.toml");
        if !path.exists() {
            let config = Self::default();
            config.save()?;
        }

        let config: Settings = Figment::new()
            .merge(Serialized::defaults(Settings::default()))
            .merge(Toml::file(path))
            .extract()?;
        *self = config;
        Ok(())
    }
}
