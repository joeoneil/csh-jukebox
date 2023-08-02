#![allow(dead_code)]

pub mod fingerprint;
pub mod types;

use crate::fingerprint::SongMetadata;

use std::fs;
use std::io::BufReader;
use std::{fs::File, path::Path};

use anyhow::{anyhow, Error};
use fingerprint::lookup_song;
use log::{log, Level};
use rodio::Source;
use youtube_dl::{SearchOptions, YoutubeDl};

use types::*;

pub fn search(query: &str, count: usize) -> Result<Vec<Song>, Error> {
    let opts = SearchOptions::youtube(query).with_count(count);

    log!(Level::Debug, "Starting search for {}", query);

    let search = YoutubeDl::search_for(&opts).run()?;

    log!(Level::Debug, "{:?}", search);

    Err(anyhow!("Not Yet Implemented"))
}

impl Song {
    /**
     * create a new song with the specified origin and submitter
     * */
    pub fn new(origin: SongOrigin, submitter: String) -> Self {
        let path = match &origin {
            SongOrigin::FileUpload(path) => Some(path.clone()),
            _ => None,
        };
        Song {
            origin,
            submitter,
            metadata: None,
            path,
        }
    }

    /**
     * Gets the audio stream for a song
     * */
    pub fn as_stream(&mut self) -> Result<Box<dyn Source<Item = f32> + Send>, Error> {
        let start = std::time::Instant::now();
        let out: Result<Box<dyn Source<Item = f32> + Send>, Error> = match &self.origin {
            SongOrigin::FileUpload(path) => {
                let file = File::open(path)?;
                log!(Level::Debug, "Converting file to stream");
                Ok(Box::new(
                    rodio::Decoder::new(BufReader::new(file))?.convert_samples(),
                ))
            }
            SongOrigin::Youtube(url) => {
                if !Path::new("/tmp/jukebox").exists() {
                    log!(Level::Debug, "output dir does not exist, creating");
                    fs::create_dir_all("/tmp/jukebox")?;
                }

                log!(Level::Debug, "Downloading Youtube video from {url}");
                let song_dl = YoutubeDl::new(url.as_str())
                    .socket_timeout("15")
                    .format("bestaudio")
                    .output_directory("/tmp/jukebox")
                    .output_template("%(id)s")
                    .download(true)
                    .extra_arg("-x")
                    .extra_arg("--audio-format")
                    .extra_arg("mp3")
                    .run()?;

                log!(Level::Trace, "Getting id from data download");
                let filename = match song_dl.into_single_video().and_then(|v| Some(v.id)) {
                    Some(id) => id,
                    None => url.split("?v=").skip(1).next().unwrap().to_string(),
                };

                log!(Level::Debug, "Converting output video to stream");
                let file = File::open(format!("/tmp/jukebox/{filename}.mp3"))?;
                let _ = self.path.insert(format!("/tmp/jukebox/{filename}.mp3"));
                Ok(Box::new(
                    rodio::Decoder::new(BufReader::new(file))?.convert_samples(),
                ))
            }
            _ => todo!(),
        };

        log!(
            Level::Debug,
            "Converted song to source. took {} ms",
            start.elapsed().as_millis()
        );

        out
    }

    /**
     * Gets the song's metadata or, if the field is None, attempts to fetch
     * the metadata from the song's origin.
     * */
    pub async fn fetch_metadata(&mut self) -> Result<&SongMetadata, Error> {
        if self.metadata.is_some() {
            self.metadata.as_ref().ok_or_else(|| unreachable!())
        } else {
            if self.path.is_some() {
                let meta = lookup_song(self.path.as_ref().unwrap()).await?;
                let _ = self.metadata.insert(meta);
                self.metadata.as_ref().ok_or_else(|| unreachable!())
            } else {
                Err(anyhow!("Song has not been fetched yet"))
            }
        }
    }
}

impl UserQueue {
    /**
     * Construct a new user with the specified id
     * */
    pub fn new(user_id: String) -> Self {
        UserQueue {
            user_id,
            q: vec![].into(),
            shuffle: false,
        }
    }

    /**
     * gets the next song in the user's queue as long as it is not empty
     * */
    fn get_next(&mut self) -> Option<Song> {
        self.q.pop_front()
    }

    fn has_songs(&self) -> bool {
        self.q.len() > 0
    }
}

impl GlobalQueue {
    /**
     * Creates a new global queue with an empty queue and no users
     * */
    pub fn new() -> Self {
        GlobalQueue {
            q: vec![].into(),
            users: vec![].into(),
        }
    }

    /**
     * Gets the next song and adds a new song into the queue from the next user
     * as long as the current number of songs is less than or equal to the
     * target_count
     * */
    pub fn next(&mut self, target_count: usize) -> Option<Song> {
        if self.q.len() <= target_count {
            if let Some(song) = loop {
                if let Some(mut user) = self.users.pop_front() {
                    if let Some(song) = user.get_next() {
                        self.users.push_back(user);
                        break Some(song);
                    }
                    // Users with no songs in queue are dropped
                } else {
                    break None;
                }
            } {
                self.q.push_back(song);
            }
        }
        self.q.pop_front()
    }

    /**
     * moves songs from user queues into the global queue until there are at least
     * {count} songs in the global queue
     * */
    pub fn flush_songs(&mut self, count: usize) {
        while self.q.len() < count && self.users.len() > 0 {
            let mut user = self.users.pop_front().unwrap(); // Guarenteed safe by loop condition
            if let Some(song) = user.get_next() {
                self.q.push_back(song);
                self.users.push_back(user);
            }
            // Users with no songs in queue are dropped
        }
    }

    /**
     * Registers a new user, optionally adding a song to the queue immediately
     * */
    pub fn register_user(&mut self, user_id: String) {
        self.users.push_back(UserQueue::new(user_id))
    }

    pub fn preview(&self, count: usize) -> Vec<&Song> {
        if self.q.len() >= count {
            self.q.range(0..count).collect::<Vec<&Song>>()
        } else {
            self.q.range(0..self.q.len() - 1).collect::<Vec<&Song>>()
        }
    }
}
