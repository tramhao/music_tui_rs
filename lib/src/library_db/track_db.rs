use std::time::{Duration, UNIX_EPOCH};

use rusqlite::{named_params, Connection, Row};

use crate::track::Track;

/// A struct representing a [`Track`](Track) in the database
#[derive(Clone, Debug)]
pub struct TrackDB {
    pub id: u64,
    pub artist: String,
    pub title: String,
    pub album: String,
    pub genre: String,
    pub file: String,
    pub duration: Duration,
    pub name: String,
    pub ext: String,
    pub directory: String,
    pub last_modified: String,
    pub last_position: Duration,
}

impl TrackDB {
    /// Try to convert a given row to a [`TrackDB`] instance, expecting correct row order.
    ///
    /// Use [`Self::try_from_row_named`] if possible.
    pub fn try_from_row_id(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        let d_u64: u64 = row.get(6)?;
        let last_position_u64: u64 = row.get(11)?;
        Ok(TrackDB {
            id: row.get(0)?,
            artist: row.get(1)?,
            title: row.get(2)?,
            album: row.get(3)?,
            genre: row.get(4)?,
            file: row.get(5)?,
            duration: Duration::from_secs(d_u64),
            name: row.get(7)?,
            ext: row.get(8)?,
            directory: row.get(9)?,
            last_modified: row.get(10)?,
            last_position: Duration::from_secs(last_position_u64),
        })
    }

    /// Try to convert a given row to a [`TrackDB`] instance, using column names to resolve the values
    pub fn try_from_row_named(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        // NOTE: all the names in "get" below are the *column names* as defined in migrations/002.sql#table_tracks (pseudo link)
        let d_u64: u64 = row.get("duration")?;
        let last_position_u64: u64 = row.get("last_position")?;
        Ok(TrackDB {
            id: row.get("id")?,
            artist: row.get("artist")?,
            title: row.get("title")?,
            album: row.get("album")?,
            genre: row.get("genre")?,
            file: row.get("file")?,
            duration: Duration::from_secs(d_u64),
            name: row.get("name")?,
            ext: row.get("ext")?,
            directory: row.get("directory")?,
            last_modified: row.get("last_modified")?,
            last_position: Duration::from_secs(last_position_u64),
        })
    }
}

/// A struct representing a [`Track`](Track) in the database to be inserted
///
/// This is required as some fields are auto-generated by the database compared to [`TrackDB`]
#[derive(Clone, Debug)]
pub struct TrackDBInsertable<'a> {
    // generated by the database
    // pub id: u64,
    pub artist: &'a str,
    pub title: &'a str,
    pub album: &'a str,
    pub genre: &'a str,
    pub file: &'a str,
    pub duration: Duration,
    pub name: &'a str,
    pub ext: &'a str,
    pub directory: &'a str,
    pub last_modified: String,
    pub last_position: Duration,
}

/// Constant strings for Unknown values
pub mod const_unknown {
    use crate::const_str;

    const_str! {
        UNKNOWN_ARTIST "Unknown Artist",
        UNKNOWN_TITLE "Unknown Title",
        UNKNOWN_ALBUM "empty",
        UNKNOWN_GENRE "no type",
        UNKNOWN_FILE "Unknown File",
    }
}
use const_unknown::{UNKNOWN_ALBUM, UNKNOWN_ARTIST, UNKNOWN_FILE, UNKNOWN_GENRE, UNKNOWN_TITLE};

impl<'a> From<&'a Track> for TrackDBInsertable<'a> {
    fn from(value: &'a Track) -> Self {
        Self {
            artist: value.artist().unwrap_or(UNKNOWN_ARTIST),
            title: value.title().unwrap_or(UNKNOWN_TITLE),
            album: value.album().unwrap_or(UNKNOWN_ALBUM),
            genre: value.genre().unwrap_or(UNKNOWN_GENRE),
            file: value.file().unwrap_or(UNKNOWN_FILE),
            duration: value.duration(),
            name: value.name().unwrap_or_default(),
            ext: value.ext().unwrap_or_default(),
            directory: value.directory().unwrap_or_default(),
            last_modified: value
                .last_modified
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
            last_position: Duration::default(),
        }
    }
}

impl TrackDBInsertable<'_> {
    /// Insert the current [`TrackDBInsertable`] into the `tracks` table
    #[inline]
    pub fn insert_track(&self, con: &Connection) -> Result<usize, rusqlite::Error> {
        con.execute(
            "INSERT INTO tracks (artist, title, album, genre, file, duration, name, ext, directory, last_modified, last_position) 
            values (:artist, :title, :album, :genre, :file, :duration, :name, :ext, :directory, :last_modified, :last_position)",
            named_params![
                ":artist": &self.artist,
                ":title": &self.title,
                ":album": &self.album,
                ":genre": &self.genre,
                ":file": &self.file,
                ":duration": &self.duration.as_secs(),
                ":name": &self.name,
                ":ext": &self.ext,
                ":directory": &self.directory,
                ":last_modified": &self.last_modified,
                ":last_position": &self.last_position.as_secs().to_string(),
            ],
        )
    }
}

/// Defined for types which could be indexed.
/// Was made to allow generalization of indexing/search functions.
///
/// the required functions are generally the metadata you would find in an mp3 file.
pub trait Indexable {
    fn meta_file(&self) -> Option<&str>;
    fn meta_title(&self) -> Option<&str>;
    fn meta_album(&self) -> Option<&str>;
    fn meta_artist(&self) -> Option<&str>;
    fn meta_genre(&self) -> Option<&str>;
    fn duration(&self) -> Duration;
}

impl Indexable for Track {
    fn meta_file(&self) -> Option<&str> {
        self.file()
    }
    fn meta_title(&self) -> Option<&str> {
        self.title()
    }
    fn meta_album(&self) -> Option<&str> {
        self.album()
    }
    fn meta_artist(&self) -> Option<&str> {
        self.artist()
    }
    fn meta_genre(&self) -> Option<&str> {
        self.genre()
    }
    fn duration(&self) -> Duration {
        self.duration()
    }
}

impl Indexable for TrackDB {
    fn meta_file(&self) -> Option<&str> {
        if self.file == UNKNOWN_FILE {
            return None;
        }
        Some(&self.file)
    }
    fn meta_title(&self) -> Option<&str> {
        if self.title == UNKNOWN_TITLE {
            return None;
        }
        Some(&self.title)
    }
    fn meta_album(&self) -> Option<&str> {
        if self.album == UNKNOWN_ALBUM {
            return None;
        }
        Some(&self.album)
    }
    fn meta_artist(&self) -> Option<&str> {
        if self.artist == UNKNOWN_ARTIST {
            return None;
        }
        Some(&self.artist)
    }
    fn meta_genre(&self) -> Option<&str> {
        if self.genre == UNKNOWN_GENRE {
            return None;
        }
        Some(&self.genre)
    }

    fn duration(&self) -> Duration {
        self.duration
    }
}
