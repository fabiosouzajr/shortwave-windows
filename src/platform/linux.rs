// Shortwave - platform/linux.rs
// Copyright (C) 2021-2025  Felix Häcker <haeckerfelix@gnome.org>
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

//! Upstream (freedesktop) platform integration: MPRIS, the background portal
//! and glycin image loading. The code here was moved verbatim from `app.rs`,
//! `utils.rs`, `audio/player.rs` and `api/cover_loader.rs`.

use std::cell::OnceCell;
use std::rc::Rc;

use anyhow::Result;
use ashpd::desktop::background::{Background, BackgroundProxy};
use glycin::Loader;
use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use crate::app::SwApplication;
use crate::audio::{MprisServer, SwPlaybackState};
use crate::i18n::{i18n, i18n_f};
use crate::utils;

/// Starts the MPRIS media controls on the session bus.
pub async fn start_media_controls() -> Result<()> {
    MprisServer::start().await?;
    Ok(())
}

/// Whether the app may keep playing after its window has been closed.
pub async fn background_permissions() -> bool {
    if !ashpd::is_sandboxed().await {
        debug!("App is not sandboxed, background playback is allowed.");
        return true;
    }

    if let Ok(res) = Background::request()
        .reason("Play radio station in the background")
        .send()
        .await
    {
        match res.response() {
            Ok(response) => response.run_in_background(),
            Err(err) => {
                warn!("{}", err);
                false
            }
        }
    } else {
        warn!("Unable to check background permissions, falling back to true.");
        true
    }
}

/// Keeps the background portal's status message in sync with playback.
#[derive(Default)]
pub struct BackgroundIntegration {
    proxy: Rc<OnceCell<BackgroundProxy<'static>>>,
}

impl BackgroundIntegration {
    pub async fn setup(&self, app: &SwApplication) {
        if !ashpd::is_sandboxed().await {
            debug!("Not sandboxed, not setting up background portal proxy.");
            return;
        }

        match BackgroundProxy::new().await {
            Ok(proxy) => {
                let _ = self.proxy.set(proxy);

                let weak_app = app.downgrade();
                let proxy = self.proxy.clone();
                app.player().connect_state_notify(move |_| {
                    if let Some(app) = weak_app.upgrade() {
                        update_background_portal_status(&app, &proxy);
                    }
                });

                let weak_app = app.downgrade();
                let proxy = self.proxy.clone();
                app.player().connect_station_notify(move |_| {
                    if let Some(app) = weak_app.upgrade() {
                        update_background_portal_status(&app, &proxy);
                    }
                });

                update_background_portal_status(app, &self.proxy);
            }
            Err(err) => warn!("Unable to setup background portal proxy: {}", err),
        };
    }
}

fn update_background_portal_status(
    app: &SwApplication,
    proxy: &Rc<OnceCell<BackgroundProxy<'static>>>,
) {
    let mut message = i18n("No Playback");

    if let Some(station) = app.player().station() {
        if app.player().state() == SwPlaybackState::Playing {
            message = i18n_f("Playing “{}”", &[&station.title()]);
        }
    }

    let proxy = proxy.clone();
    glib::spawn_future_local(async move {
        let message = utils::ellipsize_end(message, 96);
        if let Some(proxy) = proxy.get() {
            if let Err(err) = proxy.set_status(&message).await {
                warn!("Unable to update background portal status message: {}", err);
            }
        }
    });
}

/// Decodes a downloaded station cover with glycin (sandboxed loaders).
pub async fn load_texture(file: &gio::File) -> Result<gdk::Texture> {
    let loader = Loader::new(file.clone());
    let image = loader.load().await?;
    Ok(image.next_frame().await?.texture())
}
