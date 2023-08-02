use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::process::Command;

/**
 * Data output by the fpcalc utility including file length and fingerprint string
 * */
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FingerprintData {
    pub duration: f64,
    #[serde(rename = "fingerprint")]
    pub fp: String,
}

/**
 * Calculates the fingerprint of an audio file using fpcalc and the underlying chromaprint
 * library. This fingerprint can be used to lookup song information using the acoustID API
 * and musicbrainz API.
 * */
pub fn calculate_fingerprint(filepath: &str) -> Result<FingerprintData, Error> {
    Ok(serde_json::from_str(
        String::from_utf8_lossy(
            Command::new("fpcalc")
                .arg("-json") // not --json
                .arg(filepath)
                .output()?
                .stdout
                .as_slice(),
        )
        .to_string()
        .as_str(),
    )?)
}
