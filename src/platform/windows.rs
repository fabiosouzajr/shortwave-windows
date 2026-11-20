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

use anyhow::Result;
use gtk::prelude::*;
use gtk::{gdk, gio};

use crate::app::SwApplication;

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
