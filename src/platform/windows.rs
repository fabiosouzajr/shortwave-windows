// Shortwave - platform/windows.rs
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

//! Windows platform integration.

use std::env;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::Result;
use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use crate::app::SwApplication;
use crate::config;

/// `<prefix>\share` of the running installation, i.e. `..\share` relative to
/// `bin\shortwave.exe`, if it contains our GResource bundle. Falls back to the
/// build-time `DATADIR` (development runs from a Meson install prefix).
static DATADIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let relocated = env::current_exe().ok().and_then(|exe| {
        let share = exe.parent()?.parent()?.join("share");
        let gresource = share
            .join(config::PKGNAME)
            .join(format!("{}.gresource", config::APP_ID));
        gresource.is_file().then_some(share)
    });

    match relocated {
        Some(share) => share,
        None => {
            warn!(
                "No data directory next to the executable, using build-time path {}",
                config::DATADIR
            );
            PathBuf::from(config::DATADIR)
        }
    }
});

/// Early process setup, before GTK is initialised.
pub fn init() {
    // Make our compiled GSettings schema visible. GLib also searches
    // `<dir of libgio>\..\share`, which covers the bundled layout, but not a
    // development prefix. `gio::Settings::new()` aborts if the schema is
    // missing, so this has to happen before anything touches settings.
    if env::var_os("GSETTINGS_SCHEMA_DIR").is_none() {
        let schemas = datadir().join("glib-2.0").join("schemas");
        if schemas.join("gschemas.compiled").is_file() {
            debug!("Using GSettings schemas from {}", schemas.display());
            env::set_var("GSETTINGS_SCHEMA_DIR", &schemas);
        }
    }
}

/// Setup that needs an initialised GTK display.
pub fn init_gtk() {
    // The app icon is looked up by name (APP_ID) in the hicolor theme.
    if let Some(display) = gdk::Display::default() {
        gtk::IconTheme::for_display(&display).add_search_path(datadir().join("icons"));
    }
}

/// Installed data directory (GResource, schemas, icons).
pub fn datadir() -> PathBuf {
    DATADIR.clone()
}

/// Installed translations.
pub fn localedir() -> PathBuf {
    datadir().join("locale")
}

/// Cache directory: `%LOCALAPPDATA%\Shortwave\cache`.
///
/// GLib's `user_cache_dir()` on Windows is the browser cache
/// (`INetCache`), which Disk Cleanup may wipe.
pub fn cache_dir() -> PathBuf {
    let mut path = glib::user_data_dir();
    path.push(config::NAME);
    path.push("cache");
    path
}

/// GStreamer element factory for audio output: WASAPI (the modern
/// `wasapi2sink` first), falling back to `autoaudiosink`.
pub fn audio_sink() -> &'static str {
    for factory in ["wasapi2sink", "wasapisink"] {
        if gstreamer::ElementFactory::find(factory).is_some() {
            return factory;
        }
    }

    warn!("No WASAPI audio sink found, falling back to autoaudiosink (no volume control).");
    "autoaudiosink"
}

/// File name for a saved recording in `dir`.
///
/// Same sanitising as upstream (`sanitize-filename` applies the Windows rules
/// when built for Windows), plus:
/// - a fallback name when nothing is left (e.g. the title is `CON` or `???`);
/// - the name is shortened so that `dir\name` fits in `MAX_PATH`, since most
///   systems don't have long paths enabled.
pub fn recording_file_name(dir: &Path, title: &str) -> String {
    const EXTENSION: &str = ".ogg";
    const FALLBACK: &str = "Track";
    // MAX_PATH is 260 UTF-16 units including the terminating NUL
    const MAX_PATH_LEN: usize = 259;

    let sanitized = sanitize_filename::sanitize(title);

    let dir_len = dir.as_os_str().encode_wide().count();
    let budget = MAX_PATH_LEN
        .saturating_sub(dir_len + 1 + EXTENSION.len())
        .max(1);

    let mut used = 0;
    let mut end = 0;
    for (i, c) in sanitized.char_indices() {
        used += c.len_utf16();
        if used > budget {
            break;
        }
        end = i + c.len_utf8();
    }

    // Shortening may leave a trailing dot or space, which Windows strips
    let stem = sanitized[..end].trim_end_matches(['.', ' ']);
    let stem = if stem.trim().is_empty() { FALLBACK } else { stem };

    format!("{stem}{EXTENSION}")
}

/// Default for the `recording-track-directory` setting: `<Music>\Shortwave`.
pub fn default_recording_dir() -> PathBuf {
    let mut path = glib::user_special_dir(glib::UserDirectory::Music)
        .unwrap_or_else(|| glib::home_dir().join("Music"));
    path.push(config::NAME);
    path
}

#[path = "windows/smtc.rs"]
mod smtc;

/// Starts the System Media Transport Controls integration.
pub async fn start_media_controls() -> Result<()> {
    smtc::start();
    Ok(())
}

/// Windows has no sandbox / portal: background playback is always allowed.
pub async fn background_permissions() -> bool {
    true
}

/// Background playback integration.
// TODO(win): tray icon (Phase 4, workstream 8).
#[derive(Default)]
pub struct BackgroundIntegration;

impl BackgroundIntegration {
    pub async fn setup(&self, _app: &SwApplication) {}
}

/// Decodes a downloaded station cover with GTK's built-in decoders and the
/// gdk-pixbuf loaders (glycin's sandboxed loaders are Linux-only).
pub async fn load_texture(file: &gio::File) -> Result<gdk::Texture> {
    let (bytes, _etag) = file.load_bytes_future().await?;

    // Many favicons are ICO files that contain a PNG image. gdk-pixbuf's ICO
    // loader only handles BMP entries ("Compressed icons are not supported"),
    // while glycin on Linux handles both, so decode the PNG directly.
    if let Some(png) = png_from_ico(&bytes) {
        return Ok(gdk::Texture::from_bytes(&glib::Bytes::from(png))?);
    }

    Ok(gdk::Texture::from_bytes(&bytes)?)
}

/// Returns the PNG data of the largest PNG-compressed image in an ICO file,
/// or `None` if `data` isn't an ICO file or has no PNG entry.
fn png_from_ico(data: &[u8]) -> Option<&[u8]> {
    const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

    let u16_at = |pos: usize| Some(u16::from_le_bytes(data.get(pos..pos + 2)?.try_into().ok()?));
    let u32_at = |pos: usize| Some(u32::from_le_bytes(data.get(pos..pos + 4)?.try_into().ok()?));

    // ICONDIR: reserved = 0, type = 1 (icon), count
    if u16_at(0)? != 0 || u16_at(2)? != 1 {
        return None;
    }
    let count = usize::from(u16_at(4)?);

    let mut best: Option<(u32, &[u8])> = None;
    for i in 0..count {
        // ICONDIRENTRY (16 bytes): width, height (0 = 256), ..., size, offset
        let entry = 6 + 16 * i;
        let dim = |b: u8| if b == 0 { 256 } else { u32::from(b) };
        let width = dim(*data.get(entry)?);
        let height = dim(*data.get(entry + 1)?);
        let size = usize::try_from(u32_at(entry + 8)?).ok()?;
        let offset = usize::try_from(u32_at(entry + 12)?).ok()?;

        let Some(image) = data.get(offset..offset.checked_add(size)?) else {
            continue;
        };
        if !image.starts_with(PNG_SIGNATURE) {
            continue;
        }

        let area = width * height;
        if best.is_none_or(|(best_area, _)| area > best_area) {
            best = Some((area, image));
        }
    }

    best.map(|(_, image)| image)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG_SIG: &[u8] = b"\x89PNG\r\n\x1a\n";

    /// Builds an ICO file with the given (width, height, payload) entries.
    fn ico(entries: &[(u8, u8, &[u8])]) -> Vec<u8> {
        let mut out = vec![0, 0, 1, 0];
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        let mut offset = 6 + 16 * entries.len() as u32;
        for (w, h, data) in entries {
            out.extend_from_slice(&[*w, *h, 0, 0, 1, 0, 32, 0]);
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(&offset.to_le_bytes());
            offset += data.len() as u32;
        }
        for (_, _, data) in entries {
            out.extend_from_slice(data);
        }
        out
    }

    #[test]
    fn recording_name_is_sanitized_for_windows() {
        let dir = Path::new(r"C:\Users\me\Music\Shortwave");
        assert_eq!(recording_file_name(dir, "AC/DC: Back <in> Black?"), "ACDC Back in Black.ogg");
        assert_eq!(recording_file_name(dir, "Ends with dot."), "Ends with dot.ogg");
    }

    #[test]
    fn reserved_or_empty_title_gets_a_fallback_name() {
        let dir = Path::new(r"C:\Music");
        assert_eq!(recording_file_name(dir, "CON"), "Track.ogg");
        assert_eq!(recording_file_name(dir, "???"), "Track.ogg");
    }

    #[test]
    fn full_path_stays_within_max_path() {
        let dir = PathBuf::from(format!(r"C:\{}", "d".repeat(200)));
        let name = recording_file_name(&dir, &"é".repeat(300));
        let full = dir.join(&name);
        assert!(full.as_os_str().encode_wide().count() <= 259, "{}", full.display());
        assert!(name.ends_with(".ogg"));
        assert!(name.starts_with('é'));
    }

    #[test]
    fn short_names_are_untouched() {
        let dir = Path::new(r"C:\Music");
        assert_eq!(recording_file_name(dir, "Artist - Title"), "Artist - Title.ogg");
    }

    #[test]
    fn png_in_ico_is_extracted() {
        let png = [PNG_SIG, b"png-payload"].concat();
        let file = ico(&[(32, 32, &png)]);
        assert_eq!(png_from_ico(&file), Some(&png[..]));
    }

    #[test]
    fn largest_png_entry_wins() {
        let small = [PNG_SIG, b"small"].concat();
        let big = [PNG_SIG, b"big"].concat();
        // width/height 0 means 256 px
        let file = ico(&[(16, 16, &small), (0, 0, &big)]);
        assert_eq!(png_from_ico(&file), Some(&big[..]));
    }

    #[test]
    fn bmp_only_ico_is_left_to_gdk_pixbuf() {
        let file = ico(&[(32, 32, b"\x28\x00\x00\x00bmp-dib")]);
        assert_eq!(png_from_ico(&file), None);
    }

    #[test]
    fn non_ico_and_truncated_data_are_ignored() {
        assert_eq!(png_from_ico(PNG_SIG), None);
        assert_eq!(png_from_ico(&[0, 0, 1, 0, 5, 0]), None);
        let mut file = ico(&[(32, 32, &[PNG_SIG, b"x"].concat())]);
        file.truncate(file.len() - 3);
        assert_eq!(png_from_ico(&file), None);
    }
}
