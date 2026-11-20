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
use std::path::PathBuf;
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

/// Default for the `recording-track-directory` setting: `<Music>\Shortwave`.
pub fn default_recording_dir() -> PathBuf {
    let mut path = glib::user_special_dir(glib::UserDirectory::Music)
        .unwrap_or_else(|| glib::home_dir().join("Music"));
    path.push(config::NAME);
    path
}

/// Starts the System Media Transport Controls integration.
// TODO(win): SMTC (Phase 4, workstream 7).
pub async fn start_media_controls() -> Result<()> {
    debug!("Media controls are not implemented on Windows yet.");
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
    Ok(gdk::Texture::from_bytes(&bytes)?)
}
