#![allow(dead_code)]

use crate::fingerprint::SongMetadata;
use std::collections::VecDeque;

/**
 * The global song queue, which will pull songs from each user into the global
 * queue to be played
 * */
pub struct GlobalQueue {
    /**
     * The global queue, containing all songs currently queued to be played
     * */
    pub q: VecDeque<Song>,

    /**
     * The Users who have submitted songs, in the order they will be pulled
     * from next.
     * */
    pub users: VecDeque<UserQueue>,
}

/**
 * A user's song queue, to be filtered into the global queue in a manner tbd
 * */
pub struct UserQueue {
    /**
     * The user's ID that owns this queue.
     * */
    pub user_id: String,

    /**
     * The list of songs currently in the user's queue.
     * */
    pub q: VecDeque<Song>,

    /**
     * Whether the user's songs should be selected in a random order
     * */
    pub shuffle: bool,
}

impl Default for UserQueue {
    fn default() -> Self {
        UserQueue {
            user_id: "".to_string(),
            q: vec![].into(),
            shuffle: false,
        }
    }
}

/**
 * A Song, with information about how to retrieve it, as well as
 * assocated metadata such as artist, title, album cover, etc.
 * */
pub struct Song {
    /**
     * The origin of the song, necessary for getting the audio data
     * */
    pub origin: SongOrigin,

    /**
     * The user who submitted the song to the queue
     * */
    pub submitter: String,

    pub metadata: Option<SongMetadata>,

    pub path: Option<String>,
}

impl Default for Song {
    fn default() -> Self {
        Song {
            origin: SongOrigin::Youtube("https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string()),
            submitter: "joeneil".to_string(),
            metadata: None,
            path: None,
        }
    }
}

/**
 * Enum for all supported and planned audio sources
 * */
pub enum SongOrigin {
    /**
     * Song originates from youtube; contained value is the full url
     * */
    Youtube(String),

    /**
     * Song originates from Spotify; contained value is the full url
     *
     * Spotify support is not currently implemented, and will be implemented
     * at a later date.
     * */
    Spotify(String),

    /**
     * Song originates from Soundcloud; contained value is the full url
     *
     * Soundcloud support is not currently implemented, and is postponed
     * indefinitely as Soundcloud has stopped allowing new 3rd party
     * applications API access since 2019.
     * */
    Soundcloud(String),

    /**
     * Song is a user submitted file; contained value if the full local path.
     *
     * Supports several common filetypes transparently. Current supported list
     * is:
     *  - FLAC
     *  - MP3
     *  - Vorbis
     *  - WAV
     * */
    FileUpload(String),
}
