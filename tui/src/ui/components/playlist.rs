use crate::ui::model::TermusicLayout;
use crate::ui::Model;
use anyhow::{anyhow, bail, Result};
use rand::seq::SliceRandom;
use std::borrow::Cow;
use std::path::Path;
use termusiclib::config::SharedTuiSettings;
use termusiclib::library_db::SearchCriteria;
use termusiclib::library_db::TrackDB;
use termusiclib::track::Track;
use termusiclib::types::{GSMsg, Id, Msg, PLMsg};
use termusiclib::utils::{filetype_supported, get_parent_folder, is_playlist, playlist_get_vec};
use termusicplayback::PlayerCmd;

use tui_realm_stdlib::Table;
use tuirealm::props::Borders;
use tuirealm::props::{Alignment, BorderType, PropPayload, PropValue, TableBuilder, TextSpan};
use tuirealm::{
    command::{Cmd, CmdResult, Direction, Position},
    event::KeyModifiers,
};
use tuirealm::{
    event::{Key, KeyEvent, NoUserEvent},
    AttrValue, Attribute, Component, Event, MockComponent, State, StateValue,
};

#[derive(MockComponent)]
pub struct Playlist {
    component: Table,
    config: SharedTuiSettings,
}

impl Playlist {
    pub fn new(config: SharedTuiSettings) -> Self {
        let component = {
            let config = config.read();
            Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(config.settings.theme.playlist_border()),
                )
                .background(config.settings.theme.playlist_background())
                .foreground(config.settings.theme.playlist_foreground())
                .title(" Playlist ", Alignment::Left)
                .scroll(true)
                .highlighted_color(config.settings.theme.playlist_highlight())
                .highlighted_str(&config.settings.theme.style.playlist.highlight_symbol)
                .rewind(false)
                .step(4)
                .row_height(1)
                .headers(&["Duration", "Artist", "Title", "Album"])
                .column_spacing(2)
                .widths(&[12, 20, 25, 43])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Empty"))
                        .add_col(TextSpan::from("Empty Queue"))
                        .add_col(TextSpan::from("Empty"))
                        .build(),
                )
        };

        Self { component, config }
    }
}

impl Component<Msg, NoUserEvent> for Playlist {
    #[allow(clippy::too_many_lines)]
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(key) if key == keys.navigation_keys.down.get() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.up.get() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(key) if key == keys.navigation_keys.goto_top.get() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.goto_bottom.get() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent {
                code: Key::End,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::End)),
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::Playlist(PLMsg::PlaylistTableBlurDown)),
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::Playlist(PLMsg::PlaylistTableBlurUp)),
            Event::Keyboard(key) if key == keys.playlist_keys.delete.get() => {
                match self.component.state() {
                    State::One(StateValue::Usize(index_selected)) => {
                        return Some(Msg::Playlist(PLMsg::Delete(index_selected)))
                    }
                    _ => return Some(Msg::None),
                }
            }
            Event::Keyboard(key) if key == keys.playlist_keys.delete_all.get() => {
                return Some(Msg::Playlist(PLMsg::DeleteAll))
            }
            Event::Keyboard(key) if key == keys.playlist_keys.shuffle.get() => {
                return Some(Msg::Playlist(PLMsg::Shuffle))
            }
            Event::Keyboard(key) if key == keys.playlist_keys.cycle_loop_mode.get() => {
                return Some(Msg::Playlist(PLMsg::LoopModeCycle))
            }
            Event::Keyboard(key) if key == keys.playlist_keys.play_selected.get() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Playlist(PLMsg::PlaySelected(index)));
                }
                CmdResult::None
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Playlist(PLMsg::PlaySelected(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(key) if key == keys.playlist_keys.search.get() => {
                return Some(Msg::GeneralSearch(GSMsg::PopupShowPlaylist))
            }
            Event::Keyboard(key) if key == keys.playlist_keys.swap_down.get() => {
                match self.component.state() {
                    State::One(StateValue::Usize(index_selected)) => {
                        self.perform(Cmd::Move(Direction::Down));
                        return Some(Msg::Playlist(PLMsg::SwapDown(index_selected)));
                    }
                    _ => return Some(Msg::None),
                }
            }
            Event::Keyboard(key) if key == keys.playlist_keys.swap_up.get() => {
                match self.component.state() {
                    State::One(StateValue::Usize(index_selected)) => {
                        self.perform(Cmd::Move(Direction::Up));
                        return Some(Msg::Playlist(PLMsg::SwapUp(index_selected)));
                    }
                    _ => return Some(Msg::None),
                }
            }
            Event::Keyboard(key) if key == keys.playlist_keys.add_random_album.get() => {
                return Some(Msg::Playlist(PLMsg::AddRandomAlbum));
            }
            Event::Keyboard(key) if key == keys.playlist_keys.add_random_songs.get() => {
                return Some(Msg::Playlist(PLMsg::AddRandomTracks));
            }
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

impl Model {
    pub fn playlist_reload(&mut self) {
        assert!(self
            .app
            .remount(
                Id::Playlist,
                Box::new(Playlist::new(self.config_tui.clone())),
                Vec::new()
            )
            .is_ok());
        self.playlist_switch_layout();
        self.playlist_sync();
    }

    pub fn playlist_switch_layout(&mut self) {
        if self.layout == TermusicLayout::Podcast {
            let headers = &["Duration", "Episodes"];
            self.app
                .attr(
                    &Id::Playlist,
                    Attribute::Text,
                    AttrValue::Payload(PropPayload::Vec(
                        headers
                            .iter()
                            .map(|x| PropValue::Str((*x).to_string()))
                            // .map(|x| PropValue::Str(x.as_ref().to_string()))
                            .collect(),
                    )),
                )
                .ok();

            let widths = &[12, 88];
            self.app
                .attr(
                    &Id::Playlist,
                    Attribute::Width,
                    AttrValue::Payload(PropPayload::Vec(
                        widths.iter().map(|x| PropValue::U16(*x)).collect(),
                    )),
                )
                .ok();
            self.playlist_sync();
            return;
        }

        let headers = &["Duration", "Artist", "Title", "Album"];
        self.app
            .attr(
                &Id::Playlist,
                Attribute::Text,
                AttrValue::Payload(PropPayload::Vec(
                    headers
                        .iter()
                        .map(|x| PropValue::Str((*x).to_string()))
                        // .map(|x| PropValue::Str(x.as_ref().to_string()))
                        .collect(),
                )),
            )
            .ok();

        let widths = &[12, 20, 25, 43];
        self.app
            .attr(
                &Id::Playlist,
                Attribute::Width,
                AttrValue::Payload(PropPayload::Vec(
                    widths.iter().map(|x| PropValue::U16(*x)).collect(),
                )),
            )
            .ok();
        self.playlist_sync();
    }

    fn playlist_add_playlist(&mut self, current_node: &str) -> Result<()> {
        let vec = playlist_get_vec(current_node)?;
        self.playlist.add_playlist(&vec)?;
        self.player_sync_playlist()?;
        self.playlist_sync();
        Ok(())
    }

    pub fn playlist_add_episode(&mut self, episode_index: usize) -> Result<()> {
        if self.podcast.podcasts.is_empty() {
            return Ok(());
        }
        let podcast_selected = self
            .podcast
            .podcasts
            .get(self.podcast.podcasts_index)
            .ok_or_else(|| anyhow!("get podcast selected failed."))?;
        let episode_selected = podcast_selected
            .episodes
            .get(episode_index)
            .ok_or_else(|| anyhow!("get episode selected failed."))?;
        self.playlist.add_episode(episode_selected);
        self.player_sync_playlist()?;
        self.playlist_sync();
        Ok(())
    }

    pub fn playlist_add(&mut self, current_node: &str) -> Result<()> {
        let p: &Path = Path::new(&current_node);
        if !p.exists() {
            return Ok(());
        }
        if p.is_dir() {
            let new_items_vec = Self::library_dir_children(p);
            self.playlist.add_playlist(&new_items_vec)?;
            self.player_sync_playlist()?;
            self.playlist_sync();
            return Ok(());
        }
        self.playlist_add_item(current_node)?;
        self.playlist_sync();
        Ok(())
    }

    fn playlist_add_item(&mut self, current_node: &str) -> Result<()> {
        if is_playlist(current_node) {
            self.playlist_add_playlist(current_node)?;
            return Ok(());
        }
        self.playlist.add_playlist(&[current_node])?;
        self.player_sync_playlist()?;
        Ok(())
    }

    pub fn playlist_add_all_from_db(&mut self, vec: &[TrackDB]) {
        let vec2: Vec<&str> = vec.iter().map(|f| f.file.as_str()).collect();
        if let Err(e) = self.playlist.add_playlist(&vec2) {
            self.mount_error_popup(e.context("add all to playlist from database"));
        }
        if let Err(e) = self.player_sync_playlist() {
            self.mount_error_popup(e.context("player sync playlist"));
        }
        self.playlist_sync();
    }

    pub fn playlist_add_random_album(&mut self) {
        let playlist_select_random_album_quantity = self
            .config_server
            .read()
            .settings
            .player
            .random_album_min_quantity
            .get();
        let vec = self.playlist_get_random_album_tracks(playlist_select_random_album_quantity);
        self.playlist_add_all_from_db(&vec);
    }

    pub fn playlist_add_random_tracks(&mut self) {
        let playlist_select_random_track_quantity = self
            .config_server
            .read()
            .settings
            .player
            .random_track_quantity
            .get();
        let vec = self.playlist_get_random_tracks(playlist_select_random_track_quantity);
        self.playlist_add_all_from_db(&vec);
    }

    fn playlist_sync_podcasts(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.playlist.tracks().iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let duration = record.duration_formatted().to_string();
            let duration_string = format!("[{duration:^7.7}]");

            let mut title = record.title().unwrap_or("Unknown Title").to_string();
            if record.podcast_localfile.is_some() {
                title = format!("[D] {title}");
            }
            if idx == self.playlist.get_current_track_index() {
                title = format!(
                    "{}{title}",
                    self.config_tui
                        .read()
                        .settings
                        .theme
                        .style
                        .playlist
                        .current_track_symbol
                );
            };
            table
                .add_col(TextSpan::new(duration_string.as_str()))
                .add_col(TextSpan::new(title).bold());
        }
        if self.playlist.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty playlist"));
        }

        let table = table.build();
        self.app
            .attr(
                &Id::Playlist,
                tuirealm::Attribute::Content,
                tuirealm::AttrValue::Table(table),
            )
            .ok();

        self.playlist_update_title();
    }

    pub fn playlist_sync(&mut self) {
        if self.layout == TermusicLayout::Podcast {
            self.playlist_sync_podcasts();
            return;
        }

        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.playlist.tracks().iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let duration = record.duration_formatted().to_string();
            let duration_string = format!("[{duration:^7.7}]");

            let noname_string = "No Name".to_string();
            let name = record.name().unwrap_or(&noname_string);
            let artist = record.artist().unwrap_or(name);
            let mut title: Cow<'_, str> = record.title().unwrap_or("Unknown Title").into();
            let album = record.album().unwrap_or("Unknown Album");

            // TODO: is there maybe a better option to do this on-demand instead of the whole playlist; like on draw-time?
            if idx == self.playlist.get_current_track_index() {
                title = format!(
                    "{}{title}",
                    self.config_tui
                        .read()
                        .settings
                        .theme
                        .style
                        .playlist
                        .current_track_symbol
                )
                .into();
            };

            table
                .add_col(TextSpan::new(duration_string.as_str()))
                .add_col(TextSpan::new(artist).fg(tuirealm::ratatui::style::Color::LightYellow))
                .add_col(TextSpan::new(title).bold())
                .add_col(TextSpan::new(album));
        }
        if self.playlist.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty playlist"));
            table.add_col(TextSpan::from(""));
            table.add_col(TextSpan::from(""));
        }

        let table = table.build();
        self.app
            .attr(
                &Id::Playlist,
                tuirealm::Attribute::Content,
                tuirealm::AttrValue::Table(table),
            )
            .ok();

        self.playlist_update_title();
    }

    pub fn playlist_delete_item(&mut self, index: usize) {
        if self.playlist.is_empty() {
            return;
        }
        self.playlist.remove(index);
        if let Err(e) = self.player_sync_playlist() {
            self.mount_error_popup(e.context("player sync playlist"));
        }
        self.playlist_sync();
    }

    pub fn playlist_clear(&mut self) {
        self.playlist.clear();
        if let Err(e) = self.player_sync_playlist() {
            self.mount_error_popup(e.context("player sync playlist"));
        }
        self.playlist_sync();
    }

    pub fn playlist_shuffle(&mut self) {
        self.playlist.shuffle();
        if let Err(e) = self.player_sync_playlist() {
            self.mount_error_popup(e.context("player sync playlist"));
        }
        self.playlist_sync();
    }

    pub fn playlist_update_library_delete(&mut self) {
        self.playlist.remove_deleted_items();
        if let Err(e) = self.player_sync_playlist() {
            self.mount_error_popup(e.context("player sync playlist"));
        }
        self.playlist_sync();
    }

    pub fn playlist_update_title(&mut self) {
        let duration = self.playlist.tracks().iter().map(Track::duration).sum();
        let display_symbol = self
            .config_tui
            .read()
            .settings
            .theme
            .style
            .playlist
            .use_loop_mode_symbol;
        let loop_mode = self.config_server.read().settings.player.loop_mode;
        let title = format!(
            "\u{2500} Playlist \u{2500}\u{2500}\u{2524} Total {} tracks | {} | Mode: {} \u{251c}\u{2500}",
            self.playlist.len(),
            Track::duration_formatted_short(&duration),
            loop_mode.display(display_symbol),
        );
        self.app
            .attr(
                &Id::Playlist,
                tuirealm::Attribute::Title,
                tuirealm::AttrValue::Title((title, Alignment::Left)),
            )
            .ok();
    }
    pub fn playlist_play_selected(&mut self, index: usize) {
        self.playlist.set_current_track_index(index);
        if let Err(e) = self.player_sync_playlist() {
            self.mount_error_popup(e.context("player sync playlist"));
        }
        self.command(&PlayerCmd::PlaySelected);
    }

    pub fn playlist_update_search(&mut self, input: &str) {
        let mut table: TableBuilder = TableBuilder::default();
        let mut idx = 0;
        let search = format!("*{}*", input.to_lowercase());
        for record in self.playlist.tracks() {
            let artist = record.artist().unwrap_or("Unknown artist");
            let title = record.title().unwrap_or("Unknown title");
            if wildmatch::WildMatch::new(&search).matches(&artist.to_lowercase())
                | wildmatch::WildMatch::new(&search).matches(&title.to_lowercase())
            {
                if idx > 0 {
                    table.add_row();
                }

                let duration = record.duration_formatted().to_string();
                let duration_string = format!("[{duration:^6.6}]");

                let noname_string = "No Name".to_string();
                let name = record.name().unwrap_or(&noname_string);
                let artist = record.artist().unwrap_or(name);
                let title = record.title().unwrap_or("Unknown Title");
                let file_name = record.file().unwrap_or("no file");

                table
                    .add_col(TextSpan::new(duration_string.as_str()))
                    .add_col(TextSpan::new(artist).fg(tuirealm::ratatui::style::Color::LightYellow))
                    .add_col(TextSpan::new(title).bold())
                    .add_col(TextSpan::new(file_name));
                // .add_col(TextSpan::new(record.album().unwrap_or("Unknown Album")));
                idx += 1;
            }
        }
        if self.playlist.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty playlist"));
            table.add_col(TextSpan::from(""));
        }
        let table = table.build();

        self.general_search_update_show(table);
    }

    pub fn playlist_locate(&mut self, index: usize) {
        assert!(self
            .app
            .attr(
                &Id::Playlist,
                Attribute::Value,
                AttrValue::Payload(PropPayload::One(PropValue::Usize(index))),
            )
            .is_ok());
    }

    pub fn playlist_get_random_tracks(&mut self, quantity: u32) -> Vec<TrackDB> {
        let mut result = vec![];
        if let Ok(vec) = self.db.get_all_records() {
            let mut i = 0;
            loop {
                if let Some(record) = vec.choose(&mut rand::thread_rng()) {
                    if record.title.contains("Unknown Title") {
                        continue;
                    }
                    if filetype_supported(&record.file) {
                        result.push(record.clone());
                        i += 1;
                        if i > quantity - 1 {
                            break;
                        }
                    }
                }
            }
        }
        result
    }

    pub fn playlist_get_random_album_tracks(&mut self, quantity: u32) -> Vec<TrackDB> {
        let mut result = vec![];
        if let Ok(vec) = self.db.get_all_records() {
            loop {
                if let Some(v) = vec.choose(&mut rand::thread_rng()) {
                    if v.album.contains("empty") {
                        continue;
                    }
                    if let Ok(mut vec2) = self
                        .db
                        .get_record_by_criteria(&v.album, &SearchCriteria::Album)
                    {
                        if vec2.len() < quantity as usize {
                            continue;
                        }
                        result.append(&mut vec2);
                        break;
                    }
                }
            }
        }
        result
    }

    pub fn playlist_save_m3u_before(&mut self, filename: &str) -> Result<()> {
        let current_node: String = match self.app.state(&Id::Library).ok().unwrap() {
            State::One(StateValue::String(id)) => id,
            _ => bail!("Invalid node selected in library"),
        };

        let parent_folder = get_parent_folder(&current_node);

        let full_filename = format!("{parent_folder}/{filename}.m3u");

        let path_m3u = Path::new(&full_filename);

        if path_m3u.exists() {
            self.mount_save_playlist_confirm(&full_filename);
            return Ok(());
        }

        self.playlist_save_m3u(&full_filename)
    }

    pub fn playlist_save_m3u(&mut self, filename: &str) -> Result<()> {
        self.playlist.save_m3u(filename)?;

        self.library_reload_with_node_focus(Some(filename));

        Ok(())
    }
}
