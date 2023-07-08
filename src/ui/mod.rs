/*
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
pub mod components;
pub mod model;

use crate::config::{BindingForEvent, ColorTermusic, Settings};
use crate::podcast::{EpData, PodcastFeed, PodcastNoId};
use crate::songtag::SongTag;
#[cfg(feature = "webservice")]
use crate::webservice::run_music_web_cmd_process;
use components::ImageWrapper;
use model::YoutubeOptions;
use model::{Model, TermusicLayout};
use std::time::Duration;
use tuirealm::application::PollStrategy;
use tuirealm::{Application, Update};
// -- internal

const FORCED_REDRAW_INTERVAL: Duration = Duration::from_millis(1000);

// Let's define the messages handled by our app. NOTE: it must derive `PartialEq`
#[derive(Clone, PartialEq, Eq)]
pub enum Msg {
    // AppClose,
    ConfigEditor(ConfigEditorMsg),
    DataBase(DBMsg),
    DeleteConfirmCloseCancel,
    DeleteConfirmCloseOk,
    DeleteConfirmShow,
    Download(DLMsg),
    ErrorPopupClose,
    GeneralSearch(GSMsg),
    HelpPopupShow,
    HelpPopupClose,
    LayoutTreeView,
    LayoutDataBase,
    LayoutPodCast,
    Library(LIMsg),
    LyricMessage(LyricMsg),
    LyricCycle,
    LyricAdjustDelay(i64),
    PlayerToggleGapless,
    PlayerTogglePause,
    #[cfg(feature = "webservice")]
    AddTrack(String, bool),
    // webapi message: will be used by webservice process to retire current player's status
    #[cfg(feature = "webservice")]
    PlayerStatus(String),
    #[cfg(feature = "webservice")]
    PlayerCurrentTrack(String),
    #[cfg(feature = "webservice")]
    PlayerGapless(String),
    #[cfg(feature = "webservice")]
    NextTrack(String),
    #[cfg(feature = "webservice")]
    LoopMode(String),
    #[cfg(feature = "webservice")]
    PlaylistCurrnetAddFront(String),
    // end of webservice message
    // addtional player message: for new player operation
    #[cfg(feature = "webservice")]
    PlayerStart,
    #[cfg(feature = "webservice")]
    PlayerStop,
    #[cfg(feature = "webservice")]
    PlayerEnableGapless,
    #[cfg(feature = "webservice")]
    PlayerDisableGapless,
    #[cfg(feature = "webservice")]
    PlaylistEnableAddFront(bool),
    // end of addtional player message
    PlayerVolumeUp,
    PlayerVolumeDown,
    PlayerSpeedUp,
    PlayerSpeedDown,
    PlayerSeekForward,
    PlayerSeekBackward,
    Playlist(PLMsg),
    Podcast(PCMsg),
    QuitPopupCloseCancel,
    QuitPopupCloseOk,
    QuitPopupShow,
    SavePlaylistPopupShow,
    SavePlaylistPopupCloseCancel,
    SavePlaylistPopupUpdate(String),
    SavePlaylistPopupCloseOk(String),
    SavePlaylistConfirmCloseCancel,
    SavePlaylistConfirmCloseOk(String),
    TagEditor(TEMsg),
    UpdatePhoto,
    YoutubeSearch(YSMsg),
    Xywh(XYWHMsg),
    None,
}

#[derive(Clone, PartialEq, Eq)]
pub enum XYWHMsg {
    Hide,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    ZoomIn,
    ZoomOut,
}

#[derive(Clone, PartialEq, Eq)]
pub enum DLMsg {
    DownloadRunning(String, String), // indicates progress
    DownloadSuccess(String),
    DownloadCompleted(String, Option<String>),
    DownloadErrDownload(String, String, String),
    DownloadErrEmbedData(String, String),
    MessageShow((String, String)),
    MessageHide((String, String)),
    YoutubeSearchSuccess(YoutubeOptions),
    YoutubeSearchFail(String),
    FetchPhotoSuccess(ImageWrapper),
    FetchPhotoErr(String),
}

#[derive(Clone, PartialEq, Eq)]
pub enum LyricMsg {
    LyricTextAreaBlurUp,
    LyricTextAreaBlurDown,
}

#[derive(Clone, PartialEq, Eq)]
pub enum ConfigEditorMsg {
    PodcastDirBlurDown,
    PodcastDirBlurUp,
    PodcastSimulDownloadBlurDown,
    PodcastSimulDownloadBlurUp,
    PodcastMaxRetriesBlurDown,
    PodcastMaxRetriesBlurUp,
    AlbumPhotoAlignBlurDown,
    AlbumPhotoAlignBlurUp,
    ChangeLayout,
    CloseCancel,
    CloseOk,
    ColorChanged(IdConfigEditor, ColorTermusic),
    SymbolChanged(IdConfigEditor, String),
    ConfigChanged,
    ConfigSaveOk,
    ConfigSaveCancel,
    ExitConfirmationBlurDown,
    ExitConfirmationBlurUp,
    Open,
    KeyFocus(KFMsg),
    MusicDirBlurDown,
    MusicDirBlurUp,
    PlaylistDisplaySymbolBlurDown,
    PlaylistDisplaySymbolBlurUp,
    PlaylistRandomTrackBlurDown,
    PlaylistRandomTrackBlurUp,
    PlaylistRandomAlbumBlurDown,
    PlaylistRandomAlbumBlurUp,
    LibraryForegroundBlurDown,
    LibraryForegroundBlurUp,
    LibraryBackgroundBlurDown,
    LibraryBackgroundBlurUp,
    LibraryBorderBlurDown,
    LibraryBorderBlurUp,
    LibraryHighlightBlurDown,
    LibraryHighlightBlurUp,
    LibraryHighlightSymbolBlurDown,
    LibraryHighlightSymbolBlurUp,
    PlaylistForegroundBlurDown,
    PlaylistForegroundBlurUp,
    PlaylistBackgroundBlurDown,
    PlaylistBackgroundBlurUp,
    PlaylistBorderBlurDown,
    PlaylistBorderBlurUp,
    PlaylistHighlightBlurDown,
    PlaylistHighlightBlurUp,
    PlaylistHighlightSymbolBlurDown,
    PlaylistHighlightSymbolBlurUp,
    ProgressForegroundBlurDown,
    ProgressForegroundBlurUp,
    ProgressBackgroundBlurDown,
    ProgressBackgroundBlurUp,
    ProgressBorderBlurDown,
    ProgressBorderBlurUp,
    LyricForegroundBlurDown,
    LyricForegroundBlurUp,
    LyricBackgroundBlurDown,
    LyricBackgroundBlurUp,
    LyricBorderBlurDown,
    LyricBorderBlurUp,
    ThemeSelectBlurDown,
    ThemeSelectBlurUp,
    ThemeSelectLoad(usize),
    KeyChange(IdKey, BindingForEvent),
    SaveLastPositionBlurDown,
    SaveLastPosotionBlurUp,
    SeekStepBlurDown,
    SeekStepBlurUp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KFMsg {
    DatabaseAddAllBlurDown,
    DatabaseAddAllBlurUp,
    GlobalConfigBlurDown,
    GlobalConfigBlurUp,
    GlobalDownBlurDown,
    GlobalDownBlurUp,
    GlobalGotoBottomBlurDown,
    GlobalGotoBottomBlurUp,
    GlobalGotoTopBlurDown,
    GlobalGotoTopBlurUp,
    GlobalHelpBlurDown,
    GlobalHelpBlurUp,
    GlobalLayoutTreeviewBlurDown,
    GlobalLayoutTreeviewBlurUp,
    GlobalLayoutDatabaseBlurDown,
    GlobalLayoutDatabaseBlurUp,
    GlobalLeftBlurDown,
    GlobalLeftBlurUp,
    GlobalLyricAdjustForwardBlurDown,
    GlobalLyricAdjustForwardBlurUp,
    GlobalLyricAdjustBackwardBlurDown,
    GlobalLyricAdjustBackwardBlurUp,
    GlobalLyricCycleBlurDown,
    GlobalLyricCycleBlurUp,
    GlobalPlayerNextBlurDown,
    GlobalPlayerNextBlurUp,
    GlobalPlayerPreviousBlurDown,
    GlobalPlayerPreviousBlurUp,
    GlobalPlayerSeekForwardBlurDown,
    GlobalPlayerSeekForwardBlurUp,
    GlobalPlayerSeekBackwardBlurDown,
    GlobalPlayerSeekBackwardBlurUp,
    GlobalPlayerSpeedUpBlurDown,
    GlobalPlayerSpeedUpBlurUp,
    GlobalPlayerSpeedDownBlurDown,
    GlobalPlayerSpeedDownBlurUp,
    GlobalPlayerToggleGaplessBlurDown,
    GlobalPlayerToggleGaplessBlurUp,
    GlobalPlayerTogglePauseBlurDown,
    GlobalPlayerTogglePauseBlurUp,
    GlobalQuitBlurDown,
    GlobalQuitBlurUp,
    GlobalRightBlurDown,
    GlobalRightBlurUp,
    GlobalUpBlurDown,
    GlobalUpBlurUp,
    GlobalVolumeDownBlurDown,
    GlobalVolumeDownBlurUp,
    GlobalVolumeUpBlurDown,
    GlobalVolumeUpBlurUp,
    GlobalSavePlaylistBlurDown,
    GlobalSavePlaylistBlurUp,
    LibraryDeleteBlurDown,
    LibraryDeleteBlurUp,
    LibraryLoadDirBlurDown,
    LibraryLoadDirBlurUp,
    LibraryPasteBlurDown,
    LibraryPasteBlurUp,
    LibrarySearchBlurDown,
    LibrarySearchBlurUp,
    LibrarySearchYoutubeBlurDown,
    LibrarySearchYoutubeBlurUp,
    LibraryTagEditorBlurDown,
    LibraryTagEditorBlurUp,
    LibraryYankBlurDown,
    LibraryYankBlurUp,
    PlaylistDeleteBlurDown,
    PlaylistDeleteBlurUp,
    PlaylistDeleteAllBlurDown,
    PlaylistDeleteAllBlurUp,
    PlaylistShuffleBlurDown,
    PlaylistShuffleBlurUp,
    PlaylistModeCycleBlurDown,
    PlaylistModeCycleBlurUp,
    PlaylistPlaySelectedBlurDown,
    PlaylistPlaySelectedBlurUp,
    PlaylistAddFrontBlurDown,
    PlaylistAddFrontBlurUp,
    PlaylistSearchBlurDown,
    PlaylistSearchBlurUp,
    PlaylistSwapDownBlurDown,
    PlaylistSwapDownBlurUp,
    PlaylistSwapUpBlurDown,
    PlaylistSwapUpBlurUp,
    PlaylistLqueueBlurDown,
    PlaylistLqueueBlurUp,
    PlaylistTqueueBlurDown,
    PlaylistTqueueBlurUp,
    LibrarySwitchRootBlurDown,
    LibrarySwitchRootBlurUp,
    LibraryAddRootBlurDown,
    LibraryAddRootBlurUp,
    LibraryRemoveRootBlurDown,
    LibraryRemoveRootBlurUp,
    GlobalLayoutPodcastBlurDown,
    GlobalLayoutPodcastBlurUp,
    GlobalXywhMoveLeftBlurDown,
    GlobalXywhMoveLeftBlurUp,
    GlobalXywhMoveRightBlurDown,
    GlobalXywhMoveRightBlurUp,
    GlobalXywhMoveUpBlurDown,
    GlobalXywhMoveUpBlurUp,
    GlobalXywhMoveDownBlurDown,
    GlobalXywhMoveDownBlurUp,
    GlobalXywhZoomInBlurDown,
    GlobalXywhZoomInBlurUp,
    GlobalXywhZoomOutBlurDown,
    GlobalXywhZoomOutBlurUp,
    GlobalXywhHideBlurDown,
    GlobalXywhHideBlurUp,
    PodcastMarkPlayedBlurDown,
    PodcastMarkPlayedBlurUp,
    PodcastMarkAllPlayedBlurDown,
    PodcastMarkAllPlayedBlurUp,
    PodcastEpDownloadBlurDown,
    PodcastEpDownloadBlurUp,
    PodcastEpDeleteFileBlurDown,
    PodcastEpDeleteFileBlurUp,
    PodcastDeleteFeedBlurDown,
    PodcastDeleteFeedBlurUp,
    PodcastDeleteAllFeedsBlurDown,
    PodcastDeleteAllFeedsBlurUp,
    PodcastSearchAddFeedBlurDown,
    PodcastSearchAddFeedBlurUp,
    PodcastRefreshFeedBlurDown,
    PodcastRefreshFeedBlurUp,
    PodcastRefreshAllFeedsBlurDown,
    PodcastRefreshAllFeedsBlurUp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LIMsg {
    TreeExtendDir(String),
    TreeGoToUpperDir,
    TreeBlur,
    Yank,
    Paste,
    SwitchRoot,
    AddRoot,
    RemoveRoot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DBMsg {
    AddAllToPlaylist,
    AddPlaylist(usize),
    CriteriaBlurDown,
    CriteriaBlurUp,
    SearchResult(usize),
    SearchResultBlurDown,
    SearchResultBlurUp,
    SearchTrack(usize),
    SearchTracksBlurDown,
    SearchTracksBlurUp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PCMsg {
    PodcastBlurDown,
    PodcastBlurUp,
    EpisodeBlurDown,
    EpisodeBlurUp,
    PodcastAddPopupShow,
    PodcastAddPopupCloseOk(String),
    PodcastAddPopupCloseCancel,
    SyncData((i64, PodcastNoId)),
    NewData(PodcastNoId),
    Error(String, PodcastFeed),
    PodcastSelected(usize),
    DescriptionUpdate,
    EpisodeAdd(usize),
    EpisodeMarkPlayed(usize),
    EpisodeMarkAllPlayed,
    PodcastRefreshOne(usize),
    PodcastRefreshAll,
    FetchPodcastStart(String),
    EpisodeDownload(usize),
    DLStart(EpData),
    DLComplete(EpData),
    DLResponseError(EpData),
    DLFileCreateError(EpData),
    DLFileWriteError(EpData),
    EpisodeDeleteFile(usize),
    FeedDeleteShow,
    FeedDeleteCloseOk,
    FeedDeleteCloseCancel,
    FeedsDeleteShow,
    FeedsDeleteCloseOk,
    FeedsDeleteCloseCancel,
    SearchItunesCloseCancel,
    SearchItunesCloseOk(usize),
    SearchSuccess(Vec<PodcastFeed>),
    SearchError(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PLMsg {
    AddFront,
    NextSong,
    PrevSong,
    PlaylistTableBlurDown,
    PlaylistTableBlurUp,
    Add(String),
    Delete(usize),
    DeleteAll,
    LoopModeCycle,
    #[cfg(feature = "webservice")]
    LoopModeQueue,
    #[cfg(feature = "webservice")]
    LoopModePlaylist,
    #[cfg(feature = "webservice")]
    LoopModeSingle,
    PlaySelected(usize),
    Shuffle,
    SwapDown(usize),
    SwapUp(usize),
    CmusLQueue,
    CmusTQueue,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GSMsg {
    PopupShowDatabase,
    PopupShowLibrary,
    PopupShowPlaylist,
    PopupCloseCancel,
    InputBlur,
    PopupUpdateDatabase(String),
    PopupUpdateLibrary(String),
    PopupUpdatePlaylist(String),
    TableBlur,
    PopupCloseDatabaseAddPlaylist,
    PopupCloseLibraryAddPlaylist,
    PopupCloseOkLibraryLocate,
    PopupClosePlaylistPlaySelected,
    PopupCloseOkPlaylistLocate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum YSMsg {
    InputPopupShow,
    InputPopupCloseCancel,
    InputPopupCloseOk(String),
    TablePopupNext,
    TablePopupPrevious,
    TablePopupCloseCancel,
    TablePopupCloseOk(usize),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TEMsg {
    TagEditorRun(String),
    TagEditorClose(Option<String>),
    TECounterDeleteOk,
    TEDownload(usize),
    TEEmbed(usize),
    TEFocus(TFMsg),
    TERename,
    TESearch,
    TESelectLyricOk(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TFMsg {
    CounterDeleteBlurDown,
    CounterDeleteBlurUp,
    InputArtistBlurDown,
    InputArtistBlurUp,
    InputTitleBlurDown,
    InputTitleBlurUp,
    InputAlbumBlurDown,
    InputAlbumBlurUp,
    InputGenreBlurDown,
    InputGenreBlurUp,
    SelectLyricBlurDown,
    SelectLyricBlurUp,
    TableLyricOptionsBlurDown,
    TableLyricOptionsBlurUp,
    TextareaLyricBlurDown,
    TextareaLyricBlurUp,
}
// Let's define the component ids for our application
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    ConfigEditor(IdConfigEditor),
    DBListCriteria,
    DBListSearchResult,
    DBListSearchTracks,
    DeleteConfirmRadioPopup,
    DeleteConfirmInputPopup,
    DownloadSpinner,
    Episode,
    ErrorPopup,
    GeneralSearchInput,
    GeneralSearchTable,
    GlobalListener,
    HelpPopup,
    Label,
    Library,
    Lyric,
    MessagePopup,
    Playlist,
    Podcast,
    PodcastAddPopup,
    PodcastSearchTablePopup,
    FeedDeleteConfirmRadioPopup,
    FeedDeleteConfirmInputPopup,
    Progress,
    QuitPopup,
    SavePlaylistPopup,
    SavePlaylistLabel,
    SavePlaylistConfirm,
    TagEditor(IdTagEditor),
    YoutubeSearchInputPopup,
    YoutubeSearchTablePopup,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum IdTagEditor {
    CounterDelete,
    LabelHint,
    InputArtist,
    InputTitle,
    InputAlbum,
    InputGenre,
    SelectLyric,
    TableLyricOptions,
    TextareaLyric,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum IdConfigEditor {
    Key(IdKey),
    PodcastDir,
    PodcastSimulDownload,
    PodcastMaxRetries,
    AlbumPhotoAlign,
    CEThemeSelect,
    ConfigSavePopup,
    ExitConfirmation,
    Footer,
    Header,
    MusicDir,
    PlaylistDisplaySymbol,
    PlaylistRandomAlbum,
    PlaylistRandomTrack,
    LibraryLabel,
    LibraryForeground,
    LibraryBackground,
    LibraryBorder,
    LibraryHighlight,
    LibraryHighlightSymbol,
    PlaylistLabel,
    PlaylistForeground,
    PlaylistBackground,
    PlaylistBorder,
    PlaylistHighlight,
    PlaylistHighlightSymbol,
    ProgressLabel,
    ProgressForeground,
    ProgressBackground,
    ProgressBorder,
    LyricLabel,
    LyricForeground,
    LyricBackground,
    LyricBorder,
    SaveLastPosition,
    SeekStep,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum IdKey {
    DatabaseAddAll,
    GlobalConfig,
    GlobalDown,
    GlobalGotoBottom,
    GlobalGotoTop,
    GlobalHelp,
    GlobalLayoutTreeview,
    GlobalLayoutDatabase,
    GlobalLeft,
    GlobalLyricAdjustForward,
    GlobalLyricAdjustBackward,
    GlobalLyricCycle,
    GlobalPlayerToggleGapless,
    GlobalPlayerTogglePause,
    GlobalPlayerNext,
    GlobalPlayerPrevious,
    GlobalPlayerSeekForward,
    GlobalPlayerSeekBackward,
    GlobalPlayerSpeedUp,
    GlobalPlayerSpeedDown,
    GlobalQuit,
    GlobalRight,
    GlobalUp,
    GlobalVolumeDown,
    GlobalVolumeUp,
    GlobalSavePlaylist,
    LibraryDelete,
    LibraryLoadDir,
    LibraryPaste,
    LibrarySearch,
    LibrarySearchYoutube,
    LibraryTagEditor,
    LibraryYank,
    PlaylistDelete,
    PlaylistDeleteAll,
    PlaylistShuffle,
    PlaylistModeCycle,
    PlaylistPlaySelected,
    PlaylistAddFront,
    PlaylistSearch,
    PlaylistSwapDown,
    PlaylistSwapUp,
    PlaylistLqueue,
    PlaylistTqueue,
    LibrarySwitchRoot,
    LibraryAddRoot,
    LibraryRemoveRoot,
    GlobalLayoutPodcast,
    GlobalXywhMoveLeft,
    GlobalXywhMoveRight,
    GlobalXywhMoveUp,
    GlobalXywhMoveDown,
    GlobalXywhZoomIn,
    GlobalXywhZoomOut,
    GlobalXywhHide,
    PodcastMarkPlayed,
    PodcastMarkAllPlayed,
    PodcastEpDownload,
    PodcastEpDeleteFile,
    PodcastDeleteFeed,
    PodcastDeleteAllFeeds,
    PodcastSearchAddFeed,
    PodcastRefreshFeed,
    PodcastRefreshAllFeeds,
}
pub enum SearchLyricState {
    Finish(Vec<SongTag>),
}

pub struct UI {
    model: Model,
}

impl UI {
    /// Instantiates a new Ui
    pub fn new(config: &Settings) -> Self {
        let mut model = Model::new(config);
        model.init_config();
        Self { model }
    }
    /// ### run
    ///
    /// Main loop for Ui thread
    pub fn run(&mut self) {
        self.model.init_terminal();

        #[cfg(feature = "webservice")]
        if self.model.config.web_service_addr.is_some() {
            run_music_web_cmd_process(self.model.tx_to_main.clone(), &self.model.config);
        }
        // Main loop
        let mut progress_interval = 0;
        while !self.model.quit {
            #[cfg(feature = "mpris")]
            self.model.update_mpris();

            self.model.te_update_lyric_options();
            self.model.update_player_msg();
            self.model.update_outside_msg();
            if self.model.layout != TermusicLayout::Podcast {
                self.model.lyric_update();
            }
            if progress_interval == 0 {
                self.model.run();
            }
            progress_interval += 1;
            if progress_interval >= 80 {
                progress_interval = 0;
            }

            match self.model.app.tick(PollStrategy::Once) {
                Err(err) => {
                    self.model
                        .mount_error_popup(format!("Application error: {err}"));
                }
                Ok(messages) if !messages.is_empty() => {
                    // NOTE: redraw if at least one msg has been processed
                    self.model.redraw = true;
                    for msg in messages {
                        let mut msg = Some(msg);
                        while msg.is_some() {
                            msg = self.model.update(msg);
                        }
                    }
                }
                _ => {}
            }
            // Check whether to force redraw
            self.check_force_redraw();
            self.model.view();
            // sleep(Duration::from_millis(20));
        }
        self.model.player_save_last_position();
        assert!(self.model.player.playlist.save().is_ok());
        if let Err(e) = self.model.config.save() {
            eprintln!("{e}");
        };

        self.model.finalize_terminal();
    }

    fn check_force_redraw(&mut self) {
        // If source are loading and at least 100ms has elapsed since last redraw...
        // if self.model.status == Status::Running {
        if self.model.since_last_redraw() >= FORCED_REDRAW_INTERVAL {
            self.model.force_redraw();
        }
        // }
    }
}
