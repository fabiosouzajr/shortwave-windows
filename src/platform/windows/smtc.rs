// Shortwave - platform/windows/smtc.rs
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

//! Windows System Media Transport Controls (media overlay, hardware media
//! keys, lock screen), the counterpart of `audio/mpris.rs`.
//!
//! SMTC is bound to a top-level window (HWND). `SwPlayer` starts the media
//! controls before any window exists, so the controls are created when a
//! window is realized, and recreated if the window is ever replaced.

use std::cell::RefCell;
use std::ffi::c_void;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use glib::translate::ToGlibPtr;
use gtk::prelude::*;
use gtk::{gdk, gio, glib, Window};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};

use crate::api::SwStation;
use crate::app::SwApplication;
use crate::audio::{SwPlaybackState, SwPlayer};
use crate::config;

/// Size of the thumbnail handed to Windows, in pixels.
const COVER_SIZE: i32 = 256;

#[derive(Default)]
struct State {
    controls: Option<MediaControls>,
    /// (station uuid, `file://` URL of its rendered cover)
    cover: Option<(String, String)>,
}

type Shared = Rc<RefCell<State>>;

/// Hooks SMTC up to every application window as soon as it is realized.
pub fn start() {
    let app = SwApplication::default();
    let shared: Shared = Rc::default();

    // Events from SMTC arrive on a WinRT thread; handle them on the main loop.
    let (sender, receiver) = async_channel::unbounded::<MediaControlEvent>();
    glib::spawn_future_local(async move {
        while let Ok(event) = receiver.recv().await {
            handle_event(event).await;
        }
    });

    let bind = {
        let shared = shared.clone();
        move |window: &Window| {
            let shared = shared.clone();
            let sender = sender.clone();
            let attach = move |window: &Window| {
                if let Err(err) = attach(window, &shared, sender.clone()) {
                    warn!("Unable to set up media controls: {err:#}");
                }
            };

            if window.is_realized() {
                attach(window);
            }
            window.connect_realize(move |window| attach(window));
        }
    };

    for window in app.windows() {
        bind(&window);
    }
    app.connect_window_added(move |_, window| bind(window));

    // Keep SMTC in sync with the player
    let player = app.player();
    for property in ["state", "station", "playing-track"] {
        let shared = shared.clone();
        player.connect_notify_local(Some(property), move |player, _| {
            update(&shared, player);
            refresh_cover(&shared, player);
        });
    }
}

fn attach(
    window: &Window,
    shared: &Shared,
    sender: async_channel::Sender<MediaControlEvent>,
) -> Result<()> {
    let surface = window
        .surface()
        .ok_or_else(|| anyhow!("window has no surface"))?;
    let hwnd = win32_handle(&surface)?;

    let config = PlatformConfig {
        display_name: config::NAME,
        dbus_name: config::APP_ID,
        hwnd: Some(hwnd),
    };
    let mut media_controls =
        MediaControls::new(config).map_err(|err| anyhow!("SMTC: {err:?}"))?;
    media_controls
        .attach(move |event| {
            let _ = sender.send_blocking(event);
        })
        .map_err(|err| anyhow!("SMTC: {err:?}"))?;

    debug!("Media controls (SMTC) attached to window {hwnd:?}");
    shared.borrow_mut().controls = Some(media_controls);

    let player = SwApplication::default().player();
    update(shared, &player);
    refresh_cover(shared, &player);
    Ok(())
}

/// The HWND behind a GDK surface.
fn win32_handle(surface: &gdk::Surface) -> Result<*mut c_void> {
    // Declared here instead of using the gdk4-win32 crate (see Cargo.toml).
    extern "C" {
        fn gdk_win32_surface_get_handle(surface: *mut gdk::ffi::GdkSurface) -> *mut c_void;
    }

    // Top-level windows are GdkWin32Toplevel, a GdkWin32Surface subclass
    let is_win32 = glib::Type::from_name("GdkWin32Surface")
        .is_some_and(|win32_surface| surface.type_().is_a(win32_surface));
    if !is_win32 {
        return Err(anyhow!("{} is not a Win32 surface", surface.type_().name()));
    }

    let hwnd = unsafe { gdk_win32_surface_get_handle(surface.to_glib_none().0) };
    if hwnd.is_null() {
        return Err(anyhow!("surface has no HWND yet"));
    }
    Ok(hwnd)
}

/// Pushes the player's state and metadata to SMTC.
fn update(shared: &Shared, player: &SwPlayer) {
    let mut state = shared.borrow_mut();
    let State { controls, cover } = &mut *state;
    let Some(controls) = controls.as_mut() else {
        return;
    };

    let station = player.station();
    let playback = match (player.state(), &station) {
        (SwPlaybackState::Playing | SwPlaybackState::Loading, _) => {
            MediaPlayback::Playing { progress: None }
        }
        // Keep the overlay around (with a play button) while a station is selected
        (_, Some(_)) => MediaPlayback::Paused { progress: None },
        (_, None) => MediaPlayback::Stopped,
    };
    if let Err(err) = controls.set_playback(playback) {
        warn!("Unable to update media controls playback state: {err:?}");
    }

    let title = station.as_ref().map(|s| s.title());
    let track = player.playing_track().map(|t| t.title());
    let cover_url = station.as_ref().and_then(|station| {
        cover
            .as_ref()
            .filter(|(uuid, _)| *uuid == station.uuid())
            .map(|(_, url)| url.clone())
    });

    let metadata = MediaMetadata {
        title: title.as_deref(),
        artist: track.as_deref(),
        album: None,
        cover_url: cover_url.as_deref(),
        duration: None,
    };
    if let Err(err) = controls.set_metadata(metadata) {
        warn!("Unable to update media controls metadata: {err:?}");
    }
}

/// Renders the current station's cover to a PNG for the SMTC thumbnail.
///
/// Windows can't decode every favicon format (e.g. ICO), so we hand it the
/// same square PNG the app displays, from the cover cache.
fn refresh_cover(shared: &Shared, player: &SwPlayer) {
    let Some(station) = player.station() else {
        return;
    };
    let uuid = station.uuid();
    if shared
        .borrow()
        .cover
        .as_ref()
        .is_some_and(|(cover_uuid, _)| *cover_uuid == uuid)
    {
        return;
    }

    let shared = shared.clone();
    let player = player.clone();
    glib::spawn_future_local(async move {
        match write_cover(&station).await {
            Ok(path) => {
                // souvlaki loads `file://` URLs with StorageFile (plain Windows path)
                let url = format!("file://{}", path.display());
                shared.borrow_mut().cover = Some((uuid, url));
                update(&shared, &player);
            }
            Err(err) => debug!("No media controls thumbnail for {}: {err:#}", station.title()),
        }
    });
}

async fn write_cover(station: &SwStation) -> Result<PathBuf> {
    let texture = match station.custom_cover() {
        Some(texture) => texture,
        None => {
            let url = station
                .metadata()
                .favicon
                .ok_or_else(|| anyhow!("station has no cover"))?;
            SwApplication::default()
                .cover_loader()
                .load_cover(&url, COVER_SIZE, gio::Cancellable::new())
                .await?
        }
    };

    // One file per station, replacing the previous one
    let dir = super::cache_dir().join("smtc");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.png", station.uuid()));
    texture.save_to_png(&path)?;
    Ok(path)
}

async fn handle_event(event: MediaControlEvent) {
    debug!("Media control event: {event:?}");
    let app = SwApplication::default();
    let player = app.player();

    match event {
        MediaControlEvent::Play => player.start_playback().await,
        // Live radio has no pause: stopping is what upstream does for MPRIS too
        MediaControlEvent::Pause | MediaControlEvent::Stop => player.stop_playback().await,
        MediaControlEvent::Toggle => player.toggle_playback().await,
        MediaControlEvent::SetVolume(volume) => player.set_volume(volume.clamp(0.0, 1.0)),
        MediaControlEvent::Raise => app.activate(),
        MediaControlEvent::Quit => app.quit(),
        _ => (),
    }
}
