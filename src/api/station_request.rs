// Shortwave - station_request.rs
// Copyright (C) 2021-2023  Felix Häcker <haeckerfelix@gnome.org>
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StationRequest {
    pub name: Option<String>,
    pub name_exact: Option<bool>,
    pub country: Option<String>,
    pub country_exact: Option<bool>,
    pub countrycode: Option<String>,
    pub state: Option<String>,
    pub state_exact: Option<bool>,
    pub language: Option<String>,
    pub language_exact: Option<bool>,
    pub tag: Option<String>,
    pub tag_exact: Option<bool>,
    pub codec: Option<String>,
    pub bitrate_min: Option<u32>,
    pub bitrate_max: Option<u32>,
    pub has_geo_info: Option<bool>,
    pub has_extended_info: Option<bool>,
    pub is_https: Option<bool>,
    pub order: Option<String>,
    pub reverse: Option<bool>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub hidebroken: Option<bool>,
}

impl StationRequest {
    /// Search filters with the ordering every search uses (most votes first).
    pub fn filtered(
        countrycode: Option<String>,
        codec: Option<String>,
        bitrate_min: Option<u32>,
    ) -> Self {
        Self {
            countrycode,
            codec,
            bitrate_min,
            limit: Some(1000),
            order: Some(String::from("votes")),
            reverse: Some(true),
            ..Self::default()
        }
    }

    pub fn url_encode(&self) -> String {
        serde_urlencoded::to_string(self).unwrap()
    }
}

impl Default for StationRequest {
    fn default() -> Self {
        Self {
            name: None,
            name_exact: None,
            country: None,
            country_exact: None,
            countrycode: None,
            state: None,
            state_exact: None,
            language: None,
            language_exact: None,
            tag: None,
            tag_exact: None,
            codec: None,
            bitrate_min: None,
            bitrate_max: None,
            has_geo_info: None,
            has_extended_info: None,
            is_https: None,
            order: None,
            reverse: None,
            offset: None,
            limit: None,
            hidebroken: Some(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codec_is_url_encoded() {
        let r = StationRequest {
            codec: Some("AAC+".into()),
            ..Default::default()
        };
        assert!(r.url_encode().contains("codec=AAC%2B"));
    }

    #[test]
    fn filtered_sets_votes_order_and_limit() {
        let r = StationRequest::filtered(Some("BR".into()), Some("FLAC".into()), Some(128));
        let q = r.url_encode();
        for part in [
            "countrycode=BR",
            "codec=FLAC",
            "bitrate_min=128",
            "limit=1000",
            "order=votes",
            "reverse=true",
        ] {
            assert!(q.contains(part), "{q} missing {part}");
        }
    }
}
