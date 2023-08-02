#![allow(unused)]

use csh_jukebox::fingerprint;
use csh_jukebox::fingerprint::lookup_song;
use csh_jukebox::search;
use csh_jukebox::types::Song;
use csh_jukebox::types::SongOrigin;
use log::{log, Level};

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv().unwrap();

    env_logger::init();

    /*
    let f = Song::new(
        SongOrigin::FileUpload(
            "/home/joe/Music/FLAC/King Gizzard & The Lizard Wizard/Omnium Gatherum/1.01 - The Dripping Tap.flac"
                .to_string(),
        ),
        "me".to_string(),
    );

    let sc = Song::new(
        SongOrigin::Soundcloud("https://soundcloud.com/neilcic/crocodile-chop".to_string()),
        "me".to_string(),
    );

    let yt = Song::new(
        SongOrigin::Youtube("https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string()),
        "me".to_string(),
    );

    let def = Song::default();

    log!(Level::Info, "Opening Stream");
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();
    log!(Level::Info, "Stream Opened & Sink Created");

    let source_file = f.as_stream().unwrap();
    let source_yt = match yt.as_stream() {
        Ok(s) => s,
        Err(e) => {
            println!("{:?}", e);
            panic!();
        }
    };

    let source_def = def.as_stream().unwrap();

    // sink.append(source_file);
    // sink.append(source_yt);
    sink.append(source_def);

    // let _ = search("Rick Astley Never Gonna Give You Up", 1);

    let _ = sink.sleep_until_end();
    */

    //let filepath = "/home/joe/Music/FLAC/King Gizzard & The Lizard Wizard/Omnium Gatherum/1.03 - Kepler‐22b.flac";
    let mut song = Song::new(
        // Neco Arc AI Cover of 'Linkin Park - In The End'
        // Matched 'In the End', the original, with 0.5 score. Victory
        // SongOrigin::Youtube("https://www.youtube.com/watch?v=U1zDp9923PU".to_string()),

        // Touch Tone Telephone x Bubblegum mashup
        // No matches found. Guess it didn't match either well enough.
        // SongOrigin::Youtube("https://www.youtube.com/watch?v=WgPmQ84QE00".to_string()),

        // 10,000 Spoons - Mashup primarily consisting of Alanis Morissette - Ironic
        // Matched 10,000 Spoons with a score of 0.92. I guess you win this one AcoustID.
        SongOrigin::Youtube("https://www.youtube.com/watch?v=pEfr1eMCaPE".to_string()),
        "me".to_string(),
    );
    let _ = song.as_stream();
    let meta = song.fetch_metadata().await.unwrap();
    println!("title: {}", meta.title);
    println!("album: {}", meta.album);
    println!("artist: {}", meta.artist);
    println!(
        "album cover: {}",
        meta.album_art.clone().unwrap_or("Not found".to_string())
    );
}
