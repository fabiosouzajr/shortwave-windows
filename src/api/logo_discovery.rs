// Shortwave - logo_discovery.rs
// Copyright (C) 2026  The Shortwave Windows port contributors
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

//! Finds station logos on a station's homepage, for stations whose
//! `favicon` is missing or broken.

use std::sync::LazyLock;

use regex::Regex;
use url::Url;

/// `<link ...>` and `<meta ...>` tags.
static TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<(link|meta)\b([^>]*)>").unwrap());

/// `name="value"`, `name='value'` or `name=value`.
static ATTRIBUTE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)([a-z_:-]+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'>]+))"#).unwrap()
});

/// Icon URLs found in a homepage's HTML, best first: apple-touch-icons, then
/// icons by declared size (largest first), then `og:image`. Relative URLs
/// are resolved against `page_url`; only http(s) URLs are returned.
pub fn icon_candidates(html: &str, page_url: &Url) -> Vec<Url> {
    // (group, size, url): lower group is better, larger size is better
    let mut found: Vec<(u8, u32, Url)> = Vec::new();

    for tag in TAG.captures_iter(html) {
        let attr = |name: &str| {
            ATTRIBUTE
                .captures_iter(&tag[2])
                .find(|a| a[1].eq_ignore_ascii_case(name))
                .and_then(|a| a.get(2).or(a.get(3)).or(a.get(4)))
                .map(|v| v.as_str().trim().replace("&amp;", "&"))
        };

        let (group, size, href) = if tag[1].eq_ignore_ascii_case("link") {
            let rel = attr("rel").unwrap_or_default().to_lowercase();
            let rels: Vec<&str> = rel.split_whitespace().collect();
            let group = if rels.iter().any(|r| r.starts_with("apple-touch-icon")) {
                0
            } else if rels.contains(&"icon") {
                1
            } else {
                continue;
            };
            (group, declared_size(&attr("sizes").unwrap_or_default()), attr("href"))
        } else {
            let property = attr("property").or_else(|| attr("name")).unwrap_or_default();
            if !property.eq_ignore_ascii_case("og:image") {
                continue;
            }
            (2, 0, attr("content"))
        };

        let Some(url) = href.and_then(|h| page_url.join(&h).ok()) else {
            continue;
        };
        if matches!(url.scheme(), "http" | "https") && !found.iter().any(|f| f.2 == url) {
            found.push((group, size, url));
        }
    }

    // Stable sort keeps document order among equally ranked icons
    found.sort_by_key(|(group, size, _)| (*group, std::cmp::Reverse(*size)));
    found.into_iter().map(|(_, _, url)| url).collect()
}

/// Largest width in a `sizes` attribute (`"16x16 32x32"`); `any` is largest.
fn declared_size(sizes: &str) -> u32 {
    sizes
        .split_whitespace()
        .map(|s| {
            if s.eq_ignore_ascii_case("any") {
                u32::MAX
            } else {
                s.split(['x', 'X']).next().and_then(|w| w.parse().ok()).unwrap_or(0)
            }
        })
        .max()
        .unwrap_or(0)
}

/// `/favicon.ico` at the root of the homepage's site.
pub fn root_favicon(page_url: &Url) -> Option<Url> {
    page_url.join("/favicon.ico").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page() -> Url {
        Url::parse("https://radio.example/live/index.html").unwrap()
    }

    #[test]
    fn candidates_order_touch_then_largest_icon_then_og() {
        let html = r#"<link rel="icon" sizes="32x32" href="/i32.png">
          <meta property="og:image" content="https://cdn.example/og.jpg">
          <link rel="icon" sizes="192x192" href="i192.png">
          <link rel="apple-touch-icon" href="/touch.png">"#;
        let got: Vec<String> = icon_candidates(html, &page())
            .iter()
            .map(|u| u.to_string())
            .collect();
        assert_eq!(
            got,
            vec![
                "https://radio.example/touch.png",
                "https://radio.example/live/i192.png",
                "https://radio.example/i32.png",
                "https://cdn.example/og.jpg"
            ]
        );
    }

    #[test]
    fn candidates_any_attribute_order_and_quotes() {
        let html = "<LINK HREF='//cdn.example/a.png?x=1&amp;y=2' REL='Shortcut Icon'>";
        assert_eq!(
            icon_candidates(html, &page())[0].as_str(),
            "https://cdn.example/a.png?x=1&y=2"
        );
    }

    #[test]
    fn candidates_skip_data_and_mask_icon() {
        let html = r#"<link rel="icon" href="data:image/png;base64,AAA"><link rel="mask-icon" href="/m.svg">"#;
        assert!(icon_candidates(html, &page()).is_empty());
    }

    #[test]
    fn candidates_deduplicated() {
        let html = r#"<link rel="icon" href="/a.png"><link rel="shortcut icon" href="/a.png">"#;
        assert_eq!(icon_candidates(html, &page()).len(), 1);
    }

    #[test]
    fn root_favicon_keeps_scheme_host_port() {
        let u = Url::parse("http://radio.example:8080/a/b?q").unwrap();
        assert_eq!(
            root_favicon(&u).unwrap().as_str(),
            "http://radio.example:8080/favicon.ico"
        );
    }
}
