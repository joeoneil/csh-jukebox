use anyhow::{anyhow, Error};
use lazy_static::lazy_static;
use log::{log, Level};
use musicbrainz_rs::entity::{recording::Recording, release_group::{ReleaseGroup, ReleaseGroupPrimaryType}};
use musicbrainz_rs::prelude::*;
use reqwest::Client;
use serde::{Serialize, Deserialize};

/**
 * Internal module for using chromaprint to generate fingerprints from audio files
 * */
pub mod chromaprint;

/**
 * Internal module for interfacing with the acoustID api
 * */
pub mod acoustid;

pub struct SongMetadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_art: Option<String>,
    pub duration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CoverArtArchiveResponse {
    images: Vec<CoverArtImage>,
    release: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CoverArtImage {
    types: Vec<String>,
    front: bool,
    back: bool,
    edit: usize,
    image: String,
    comment: String,
    approved: bool,
    id: String,
    thumbnails: CoverArtThumbnails
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CoverArtThumbnails {
    #[serde(rename = "250")]
    size_250: String,
    #[serde(rename = "500")]
    size_500: String,
    #[serde(rename = "1200")]
    size_1200: String,
    small: String,
    large: String,
}

lazy_static!(
    static ref CLIENT: Client = Client::new();
            );

/**
 * Spawn a thread for this bitch cause there's a lot of blocking requests in here
 * */
pub async fn lookup_song(path: &str) -> Result<SongMetadata, Error> {
    let mut out = SongMetadata {
        title:String::from("Not Found"),
        artist:String::from("Not Found"),
        album:String::from("Not Found"),
        album_art: None,
        duration: 0.0,
    };

    let fp = chromaprint::calculate_fingerprint(path)?;
    out.duration = fp.duration;

    log!(Level::Trace, "Audio fingerprint for {}: {}", path, fp.fp);

    let aid_result = acoustid::lookup_by_fingerprint(&fp).await?;

    log!(
        Level::Debug,
        "found {} results for path lookup {}",
        aid_result.results.len(),
        path
    );

    log!(Level::Trace, "response from AcoustID API: {:?}", aid_result);

    aid_result
        .results
        .iter()
        .for_each(|e| log!(Level::Trace, "id: {} | score: {}", e.id, e.score));

    let best_result = match aid_result
        .results
        .into_iter()
        .max_by(|a, b| a.score.total_cmp(&b.score))
    {
        Some(res) => Ok(res),
        None => Err(anyhow!(
            "AcoustID Lookup was successful but contained no results (should NEVER happen)"
        )),
    }?;

    log!(Level::Debug, "found best result with acoustID {} and score {}", best_result.id, best_result.score);

    match best_result.recordings {
        Some(recs) => {
            log!(Level::Debug, "found {} recordings from acoustID {}", recs.len(), best_result.id);
            recs.iter().for_each(|e| log!(Level::Trace, "id: {} | aid: {}", e.id, best_result.id));
            
            let r = recs.get(0).ok_or_else(|| anyhow!("AcoustID Result contained no recordings"))?;

            let rec: Recording = Recording::fetch()
                .id(r.id.as_str())
                .with_artists()
                .with_releases()
                .execute()
                .await?;

            log!(Level::Trace, "Musicbrainz recording from id {}: {:?}", r.id, rec);

            // Man this finding out metadata shit is easy
            out.title = rec.title;
            
            let mut album_mbid: Option<String> = None;

            // and then you get to the album title
            // there's gotta be a way to make this shit more concise
            // in short: find all releases of a song, map to release groups, and find the title of
            // that group, prioritizing albums, then EPs, then single releases.
            out.album = match rec.releases
                .and_then(|rels| Some(rels.into_iter()
                          .filter_map(|rel| rel.release_group)
                          .filter(|rg| rg.primary_type.is_some())
                          .collect::<Vec<ReleaseGroup>>())) {
                    Some(rgs) => {
                        if rgs.len() == 0 {
                            String::from("None")
                        } else {
                            // I feel like there should be a better way to do this but I don't know
                            // how to do a 'priority match' over enum variants like this other than
                            // implementing a sort over ReleaseGroupPrimaryType.
                            // Maybe min_by/max_by and match rg.primary_type?
                            if let Some(album) = rgs.iter().find(|rg| rg.primary_type.as_ref().unwrap() == &ReleaseGroupPrimaryType::Album) {
                                let _  = album_mbid.insert(album.id.clone());
                                album.title.clone()
                            } else if let Some(ep) = rgs.iter().find(|rg| rg.primary_type.as_ref().unwrap() == &ReleaseGroupPrimaryType::Ep) {
                                let _ = album_mbid.insert(ep.id.clone());
                                ep.title.clone()
                            } else if let Some(single) = rgs.iter().find(|rg| rg.primary_type.as_ref().unwrap() == &ReleaseGroupPrimaryType::Single) {
                                let _ = album_mbid.insert(single.id.clone());
                                String::from("Single")
                            } else {
                                String::from("Unrecognized Release Type")
                            }
                        }
                    },
                    None => String::from("Not Found"),
            };

            // nice break after that one up there
            out.artist = match rec.artist_credit {
                Some(artists) => {
                    // Assumes first credit is primary artist. TODO: Possibly check if returned
                    // order is by relevance or arbitrary?
                    artists.get(0).map_or("Not Found".to_string(), |a| a.name.clone())
                },
                None => "Not Found".to_string(),
            };

            // getting album art is non-trivial but is 'critical' apparently (why do I do this to
            // myself)
            // Fuck now I have to refactor the album thing up there to also get the mbid of that
            // release.
            out.album_art = match album_mbid {
                None => None,
                Some(mbid) => {
                    Some(CLIENT
                        .get(format!("https://coverartarchive.org/release/{}", mbid))
                        .send()
                        .await?
                        .json::<CoverArtArchiveResponse>()
                        .await?
                        .images[0]
                        .image
                        .clone()
                        )
                }
            };

            Ok(out)
        },
        None => Err(anyhow!(
                "AcoustID Result contained no recording information (was meta=recordingids included in fingerprint query?)"
                           ))
    }
}
