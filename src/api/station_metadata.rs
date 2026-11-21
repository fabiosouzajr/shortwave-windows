// Shortwave - station_metadata.rs
// Copyright (C) 2021-2022  Felix Häcker <haeckerfelix@gnome.org>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fmt::Write;
use std::str::FromStr;

use gtk::glib;
use inflector::Inflector;
use serde::{Deserialize, Deserializer, Serializer};
use url::Url;


#[derive(glib::Boxed, Default, Debug, Clone, Serialize, Deserialize)]
#[boxed_type(name = "SwStationMetadata")]
pub struct StationMetadata {
    pub changeuuid: String,
    pub stationuuid: String,
    pub name: String,
    #[serde(serialize_with = "url_to_str")]
    #[serde(deserialize_with = "str_to_url")]
    pub url: Option<Url>,
    #[serde(serialize_with = "url_to_str")]
    #[serde(deserialize_with = "str_to_url")]
    pub url_resolved: Option<Url>,
    #[serde(serialize_with = "url_to_str")]
    #[serde(deserialize_with = "str_to_url")]
    pub homepage: Option<Url>,
    #[serde(serialize_with = "url_to_str")]
    #[serde(deserialize_with = "str_to_url")]
    pub favicon: Option<Url>,
    pub tags: String,
    pub country: String,
    pub countrycode: String,
    pub state: String,
    pub language: String,
    pub languagecodes: String,
    pub votes: i32,
    pub lastchangetime: Option<String>,
    pub lastchangetime_iso8601: Option<String>,
    pub codec: String,
    pub bitrate: i32,
    pub hls: i32,
    pub lastcheckok: i32,
    pub lastchecktime: Option<String>,
    pub lastchecktime_iso8601: Option<String>,
    pub lastcheckoktime: Option<String>,
    pub lastcheckoktime_iso8601: Option<String>,
    pub lastlocalchecktime: Option<String>,
    pub clicktimestamp: Option<String>,
    pub clicktimestamp_iso8601: Option<String>,
    pub clickcount: i32,
    pub clicktrend: i32,
    pub ssl_error: i32,
    pub geo_lat: Option<f32>,
    pub geo_long: Option<f32>,
    pub has_extended_info: bool,
}

impl StationMetadata {
    pub fn formatted_tags(&self) -> String {
        let tags = self.tags.split(',');
        let mut formatted = String::new();
        for tag in tags {
            write!(formatted, ", {}", tag.to_title_case()).unwrap_or_default();
        }
        formatted.split_at(2).1.to_string()
    }

    /// Codec name for display, e.g. `AAC+`; `None` if unknown.
    pub fn display_codec(&self) -> Option<String> {
        let codec = self.codec.trim().to_uppercase();
        (!codec.is_empty() && codec != "UNKNOWN").then_some(codec)
    }

    pub fn is_lossless(&self) -> bool {
        ["FLAC", "ALAC", "WAV", "PCM"].contains(&self.codec.trim().to_uppercase().as_str())
    }

    /// Number of lit bars (0–4) in the card's bitrate meter.
    pub fn bitrate_level(&self) -> u8 {
        [64, 128, 192, 320]
            .iter()
            .filter(|&&threshold| self.bitrate >= threshold)
            .count() as u8
    }
}

impl StationMetadata {
    /// Create metadata for a new local station.
    pub fn new(name: String, url: Url) -> Self {
        Self {
            name,
            url: Some(url),
            ..Default::default()
        }
    }
}

fn url_to_str<S>(url: &Option<Url>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value = if let Some(url) = url {
        url.as_str()
    } else {
        ""
    };
    serializer.serialize_str(value)
}

fn str_to_url<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Url::from_str(&s).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quality(codec: &str, bitrate: i32) -> StationMetadata {
        StationMetadata {
            codec: codec.to_string(),
            bitrate,
            ..Default::default()
        }
    }

    #[test]
    fn codec_display() {
        assert_eq!(quality(" aac+ ", 0).display_codec().as_deref(), Some("AAC+"));
        assert_eq!(quality("UNKNOWN", 0).display_codec(), None);
        assert_eq!(quality("  ", 0).display_codec(), None);
    }

    #[test]
    fn codec_lossless() {
        assert!(quality("flac", 0).is_lossless());
        assert!(!quality("MP3", 0).is_lossless());
    }

    #[test]
    fn bitrate_levels() {
        for (b, l) in [
            (-1, 0),
            (0, 0),
            (63, 0),
            (64, 1),
            (128, 2),
            (191, 2),
            (192, 3),
            (320, 4),
            (1411, 4),
        ] {
            assert_eq!(quality("", b).bitrate_level(), l, "bitrate {b}");
        }
    }
}
