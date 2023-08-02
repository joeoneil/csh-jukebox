use anyhow::Error;
use lazy_static::lazy_static;
use log::{log, Level};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

use super::chromaprint::FingerprintData;

lazy_static! {
    static ref CLIENT_ID: String =
        env::var("ACOUSTID_CLIENT_ID").expect("ACOUSTID_CLIENT_ID Not Defined");
    static ref CLIENT: Client = reqwest::Client::new();
    static ref ACOUSTID_API_STRING: &'static str = "https://api.acoustid.org/v2/lookup";
}

/**
 * Response from the AcoustID API. Indicates success / failure as well as all tracks that
 * matched a specific fingerprint
 * */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcoustIDResponse {
    /**
     * indicates success or failure when fetching by fingerprint
     * */
    pub status: String,
    /**
     * All fingerprint matches. Contains at least one element for successful matches
     * */
    pub results: Vec<AcoustIDResult>,
}

/**
 * Individual acoustID result. A request may return multiple matches
 * */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcoustIDResult {
    /**
     * ID of this result
     * */
    pub id: String,
    /**
     * How closely this result matched the provided fingerprint from 0.0 - 1.0
     * */
    pub score: f64,
    /**
     * Musicbrainz recordings matched to this AcoustID track. If field is present contains at
     * least one recording.
     * */
    pub recordings: Option<Vec<Recording>>,
}

/**
 * Musicbrainz recording data
 * */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    /**
     * Musicbrainz ID of this recording
     * */
    pub id: String,
    /**
     * Duration of the recording in seconds
     * */
    pub duration: Option<usize>,
    /**
     * All released versions of this recording, if re-released (e.g. Single, EP, album). If field is present
     * contains at least one release group
     * */
    #[serde(rename = "releasegroups")]
    pub release_groups: Option<Vec<ReleaseGroup>>,
    /**
     * Artists present on this track. If field is present contains at least one artist.
     * */
    pub artists: Option<Vec<Artist>>,
}

/**
 * Indicates a specific release of a MusicBrainz recording
 * */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseGroup {
    /**
     * MusicBrainz ID of this release
     * */
    pub id: String,
    /**
     * Type of release (e.g. Single, EP, Album)
     * */
    #[serde(rename = "type")]
    pub release_type: Option<musicbrainz_rs::entity::release_group::ReleaseGroupPrimaryType>,
    /**
     * Title of this release
     * */
    pub title: String,
}

/**
 * MusicBrainz Artist information
 * */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    /**
     * MusicBrainz ID of this artist
     * */
    pub id: String,
    /**
     * Artist name
     * */
    pub name: String,
}

pub async fn lookup_by_fingerprint(fp: &FingerprintData) -> Result<AcoustIDResponse, Error> {
    let res = CLIENT
        .get(format!(
            "{}?meta=recordings+releasegroups+compress",
            ACOUSTID_API_STRING.to_string()
        ))
        .query(&[
            ("format", "json"),
            ("client", CLIENT_ID.as_str()),
            ("duration", (fp.duration as usize).to_string().as_str()),
            ("fingerprint", fp.fp.as_str()),
        ])
        .send()
        .await?;
    log!(Level::Trace, "Fingerprint query final url: {}", res.url());
    let t = res.text().await?;
    log!(Level::Trace, "Fingerprint returned text: {}", t);
    Ok(serde_json::from_str(t.as_str())?)
}
