// Shortwave - cover_loader.rs
// Copyright (C) 2024-2025  Felix Häcker <haeckerfelix@gnome.org>
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

use std::sync::LazyLock;
use std::time::{Duration, SystemTime};

use anyhow::{Error, Result};
use async_channel::Sender;
use async_compat::CompatExt;
use futures_util::StreamExt;
use gdk::RGBA;
use gtk::gio::File;
use gtk::graphene::Rect;
use gtk::prelude::TextureExt;
use gtk::prelude::*;
use gtk::{gdk, gio, glib, gsk};
use url::Url;

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::ClientBuilder::new()
        .user_agent(USER_AGENT.as_str())
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
});

/// How long a station whose homepage had no usable logo is left alone.
const NO_LOGO_TTL: Duration = Duration::from_secs(86400 * 7);
/// How much of a homepage is scanned for icon links.
const HOMEPAGE_READ_LIMIT: usize = 512 * 1024;
/// How many icons found on a homepage are tried before /favicon.ico.
const MAX_HOMEPAGE_CANDIDATES: usize = 4;

use super::client::USER_AGENT;
use super::logo_discovery::{icon_candidates, root_favicon};
use crate::{config, path, platform};

fn no_logo_marker_fresh(age: Duration) -> bool {
    age < NO_LOGO_TTL
}

/// URLs to try after the favicon: the best few homepage icons, then the
/// site's /favicon.ico. Nothing if the homepage couldn't be reached, as
/// its /favicon.ico is on the same unreachable host.
fn fallback_urls(favicon: Option<&Url>, found: Option<Vec<Url>>, homepage: &Url) -> Vec<Url> {
    let Some(found) = found else {
        return Vec::new();
    };

    let mut urls: Vec<Url> = found
        .into_iter()
        .filter(|u| Some(u) != favicon)
        .take(MAX_HOMEPAGE_CANDIDATES)
        .collect();
    if let Some(root) = root_favicon(homepage) {
        if Some(&root) != favicon && !urls.contains(&root) {
            urls.push(root);
        }
    }
    urls
}

/// Only conclude "no logo" when the homepage answered and nothing failed
/// for a reason that may go away (timeouts, 5xx, rate limits).
fn should_write_no_logo_marker(reachable: bool, any_transient: bool) -> bool {
    reachable && !any_transient
}

fn is_transient(err: &Error) -> bool {
    err.downcast_ref::<reqwest::Error>().is_some_and(|e| {
        e.status().is_none_or(|s| {
            s.is_server_error() || s == reqwest::StatusCode::TOO_MANY_REQUESTS
        })
    })
}

/// Deletes a temp file when dropped, so a cancelled load doesn't leave it behind.
struct TmpFileGuard(std::path::PathBuf);

impl Drop for TmpFileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[derive(Debug, Clone)]
struct CoverRequest {
    favicon: Option<Url>,
    homepage: Option<Url>,
    size: i32,
    sender: Sender<Result<gdk::Texture>>,
    cancellable: gio::Cancellable,
}

impl CoverRequest {
    pub async fn handle_request(self) {
        let res = gio::CancellableFuture::new(self.cover_texture(), self.cancellable.clone()).await;
        let msg = match res {
            Ok(res) => res,
            Err(_) => Err(Error::msg("cancelled")),
        };

        self.sender.send(msg).await.unwrap();
    }

    /// Cache key; the favicon wins so covers cached before homepage
    /// fallback existed stay valid.
    fn key(&self) -> String {
        let url = self.favicon.as_ref().or(self.homepage.as_ref());
        format!("{}@{}", url.map(Url::as_str).unwrap_or_default(), self.size)
    }

    fn no_logo_key(&self) -> String {
        format!("nologo:{}", self.key())
    }

    async fn cover_texture(&self) -> Result<gdk::Texture> {
        if let Ok(texture) = self.cached_texture().await {
            return Ok(texture);
        }

        if let Ok(Some(md)) = cacache::metadata(&*path::CACHE, self.no_logo_key()).await {
            let now = SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let age = Duration::from_millis(now.saturating_sub(md.time).try_into().unwrap_or(0));
            if no_logo_marker_fresh(age) {
                return Err(Error::msg("no logo"));
            }
        }

        self.compute_texture().await
    }

    async fn cached_texture(&self) -> Result<gdk::Texture> {
        let data = cacache::read(&*path::CACHE, self.key()).await?;
        let bytes = glib::Bytes::from_owned(data);

        Ok(gdk::Texture::from_bytes(&bytes)?)
    }

    async fn compute_texture(&self) -> Result<gdk::Texture> {
        let mut any_transient = false;

        if let Some(favicon) = &self.favicon {
            match self.download_texture(favicon).compat().await {
                Ok(texture) => return self.store(texture).await,
                Err(e) => {
                    debug!("Favicon {favicon} unusable: {}", e.root_cause());
                    any_transient |= is_transient(&e);
                }
            }
        }

        let Some(homepage) = &self.homepage else {
            return Err(Error::msg("No usable favicon"));
        };

        // None: the homepage couldn't be reached at all (e.g. offline), so
        // don't conclude that the station has no logo.
        let candidates = self.homepage_candidates(homepage).compat().await;
        let reachable = candidates.is_some();

        for url in fallback_urls(self.favicon.as_ref(), candidates, homepage) {
            match self.download_texture(&url).compat().await {
                Ok(texture) => {
                    debug!("Using logo {url} from homepage {homepage}");
                    return self.store(texture).await;
                }
                Err(e) => {
                    debug!("Logo candidate {url} unusable: {}", e.root_cause());
                    any_transient |= is_transient(&e);
                }
            }
        }

        if should_write_no_logo_marker(reachable, any_transient) {
            cacache::write(&*path::CACHE, self.no_logo_key(), b"").await?;
        }
        Err(Error::msg(format!("No usable logo found on {homepage}")))
    }

    /// Icon URLs found on the homepage; `None` if it couldn't be fetched.
    async fn homepage_candidates(&self, homepage: &Url) -> Option<Vec<Url>> {
        let mut response = HTTP_CLIENT
            .get(homepage.as_str())
            .send()
            .await
            .ok()?;
        if !response.status().is_success() {
            return Some(Vec::new());
        }

        let page_url = response.url().clone();
        let mut html = Vec::new();
        while html.len() < HOMEPAGE_READ_LIMIT {
            match response.chunk().await {
                Ok(Some(chunk)) => html.extend_from_slice(&chunk),
                Ok(None) => break,
                // Use whatever arrived before the error
                Err(_) => break,
            }
        }
        html.truncate(HOMEPAGE_READ_LIMIT);

        Some(icon_candidates(&String::from_utf8_lossy(&html), &page_url))
    }

    /// Downloads and decodes an image, rendered as a `size`² thumbnail.
    async fn download_texture(&self, url: &Url) -> Result<gdk::Texture> {
        let body = HTTP_CLIENT
            .get(url.as_str())
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        // We have to write the data to the disk in order to be able to load them using Glycin
        // TODO: Load bytes directly
        // TODO: https://gitlab.gnome.org/GNOME/glycin/-/issues/98
        let (tmp_file, tmp_stream) = File::new_tmp_future(
            Some(&format!("{}-Cover-XXXXXX", config::NAME)),
            glib::Priority::LOW,
        )
        .await?;
        let _guard = tmp_file.path().map(TmpFileGuard);
        // Rebound after the guard so the stream is dropped (closed) first;
        // Windows can't delete a file that is still open.
        let tmp_stream = tmp_stream;
        let written = tmp_stream
            .output_stream()
            .write_bytes_future(&glib::Bytes::from_owned(body), glib::Priority::LOW)
            .await;
        let _ = tmp_stream.close_future(glib::Priority::LOW).await;

        let texture = match written {
            Ok(_) => platform::load_texture(&tmp_file).await,
            Err(e) => Err(e.into()),
        };

        self.thumbnail(texture?)
    }

    fn thumbnail(&self, texture: gdk::Texture) -> Result<gdk::Texture> {
        let snapshot = gtk::Snapshot::new();
        snapshot_thumbnail(&snapshot, texture, self.size as f32);

        let node = snapshot
            .to_node()
            .ok_or_else(|| Error::msg("Empty thumbnail"))?;
        let renderer = gsk::CairoRenderer::new();
        let display = gdk::Display::default().expect("No default display available");
        renderer.realize_for_display(&display)?;

        let rect = Rect::new(0.0, 0.0, self.size as f32, self.size as f32);
        let texture = renderer.render_texture(node, Some(&rect));
        renderer.unrealize();

        Ok(texture)
    }

    async fn store(&self, texture: gdk::Texture) -> Result<gdk::Texture> {
        let png_bytes = texture.save_to_png_bytes().to_vec();
        cacache::write_with_algo(cacache::Algorithm::Xxh3, &*path::CACHE, self.key(), &png_bytes)
            .await?;

        Ok(texture)
    }
}

#[derive(Debug, Clone)]
pub struct CoverLoader {
    request_sender: Sender<CoverRequest>,
}

impl CoverLoader {
    pub fn new() -> Self {
        let (request_sender, request_receiver) = async_channel::unbounded::<CoverRequest>();
        let request_stream = request_receiver
            .map(|r| r.handle_request())
            .buffer_unordered(usize::max(glib::num_processors() as usize / 2, 2));

        glib::spawn_future_local(async move {
            request_stream.collect::<Vec<_>>().await;
        });

        Self { request_sender }
    }

    pub fn prune_cache(&self) {
        // Remove old Shortwave pre v4.0 cache
        let mut path = path::CACHE.clone();
        path.push("favicons");
        let _ = std::fs::remove_dir_all(&path);

        // Remove cached covers which are older > 30 days
        let ttl = Duration::from_secs(86400 * 30);
        for md in cacache::list_sync(&*path::CACHE).flatten() {
            let now = SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();

            let age = Duration::from_millis((now - md.time).try_into().unwrap_or_default());
            if age > ttl {
                let _ = cacache::remove_hash_sync(&*path::CACHE, &md.integrity);
                let _ = cacache::remove_sync(&*path::CACHE, &md.key);
            }
        }
    }

    pub async fn load_cover(
        &mut self,
        favicon_url: &Url,
        size: i32,
        cancellable: gio::Cancellable,
    ) -> Result<gdk::Texture> {
        self.load_station_cover(Some(favicon_url), None, size, cancellable)
            .await
    }

    /// Loads a station's cover from its favicon, falling back to icons found
    /// on its homepage.
    pub async fn load_station_cover(
        &mut self,
        favicon: Option<&Url>,
        homepage: Option<&Url>,
        size: i32,
        cancellable: gio::Cancellable,
    ) -> Result<gdk::Texture> {
        if favicon.is_none() && homepage.is_none() {
            return Err(Error::msg("Station has neither favicon nor homepage"));
        }

        let (sender, receiver) = async_channel::bounded(1);

        let request = CoverRequest {
            favicon: favicon.cloned(),
            homepage: homepage.cloned(),
            size,
            sender,
            cancellable: cancellable.clone(),
        };
        self.request_sender
            .send(request)
            .await
            .map_err(|_| Error::msg("Unable to send cover request"))?;

        receiver.recv().await?
    }
}

impl Default for CoverLoader {
    fn default() -> Self {
        Self::new()
    }
}

// Ported from Highscore (Alice Mikhaylenko)
// https://gitlab.gnome.org/World/highscore/-/blob/b07460f0c1475269381902c6305e4d91e55b61f5/src/library/cover-loader.vala#L124
fn snapshot_thumbnail(snapshot: &gtk::Snapshot, cover: gdk::Texture, size: f32) {
    let aspect_ratio = cover.width() as f32 / cover.height() as f32;
    let mut width = size;
    let mut height = size;

    if aspect_ratio < 1.0 {
        width = aspect_ratio * size;
    } else {
        height = size / aspect_ratio;
    }

    if width >= size - 2.0 {
        width = size;
    }

    if height >= size - 2.0 {
        height = size;
    }

    let cover_rect = Rect::new((size - width) / 2.0, (size - height) / 2.0, width, height);

    snapshot.push_clip(&Rect::new(0.0, 0.0, size, size));

    snapshot.append_color(
        &RGBA::new(0.96, 0.96, 0.96, 1.0),
        &Rect::new(0.0, 0.0, size, size),
    );

    if width < size || height < size {
        let blur_radius = size / 4.0;

        let outer_rect_width;
        let outer_rect_height;
        if aspect_ratio < 1.0 {
            outer_rect_width = size + blur_radius * 2.0;
            outer_rect_height = outer_rect_width / aspect_ratio;
        } else {
            outer_rect_height = size + blur_radius * 2.0;
            outer_rect_width = aspect_ratio * outer_rect_height;
        }

        let outer_rect = Rect::new(
            (size - outer_rect_width) / 2.0,
            (size - outer_rect_height) / 2.0,
            outer_rect_width,
            outer_rect_height,
        );

        snapshot.push_blur(blur_radius as f64);
        snapshot.append_texture(&cover, &outer_rect);
        snapshot.pop();
        snapshot.append_color(&RGBA::new(0.0, 0.0, 0.0, 0.2), &outer_rect);
    }

    snapshot.append_scaled_texture(&cover, gsk::ScalingFilter::Trilinear, &cover_rect);
    snapshot.pop();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_logo_marker_ttl() {
        assert!(no_logo_marker_fresh(Duration::from_secs(86400 * 6)));
        assert!(!no_logo_marker_fresh(Duration::from_secs(86400 * 7)));
    }
}

#[cfg(test)]
mod fallback_tests {
    use super::*;

    fn u(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn fallback_caps_homepage_candidates() {
        let home = u("https://r.example/");
        let found: Vec<Url> = (0..10).map(|i| u(&format!("https://r.example/{i}.png"))).collect();
        let plan = fallback_urls(None, Some(found), &home);
        assert_eq!(plan.len(), MAX_HOMEPAGE_CANDIDATES + 1);
        assert_eq!(plan.last().unwrap().as_str(), "https://r.example/favicon.ico");
    }

    #[test]
    fn fallback_unreachable_homepage_skips_root_favicon() {
        assert!(fallback_urls(None, None, &u("https://dead.example/")).is_empty());
    }

    #[test]
    fn fallback_skips_already_tried_favicon() {
        let home = u("https://r.example/");
        let fav = u("https://r.example/favicon.ico");
        let plan = fallback_urls(Some(&fav), Some(vec![u("https://r.example/a.png")]), &home);
        assert_eq!(plan, vec![u("https://r.example/a.png")]);
    }

    #[test]
    fn marker_only_when_reachable_and_no_transient_failure() {
        assert!(should_write_no_logo_marker(true, false));
        assert!(!should_write_no_logo_marker(true, true));
        assert!(!should_write_no_logo_marker(false, false));
    }

    #[test]
    fn tmp_file_guard_removes_file_on_drop() {
        let path = std::env::temp_dir().join("shortwave-guard-test.bin");
        std::fs::write(&path, b"x").unwrap();
        drop(TmpFileGuard(path.clone()));
        assert!(!path.exists());
    }
}
