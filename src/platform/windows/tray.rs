// Shortwave - platform/windows/tray.rs
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

//! Notification area (tray) icon: the way back to the app while it plays in
//! the background with no window. Replaces the background portal of the
//! Flatpak build.
//!
//! The icon is shown while the `background-playback` setting is on. Left click
//! shows or hides the window; the menu has Show/Hide, Play/Stop and Quit.

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use gtk::prelude::*;
use gtk::{gdk, glib};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::app::SwApplication;
use crate::audio::SwPlaybackState;
use crate::config;
use crate::i18n::{i18n, i18n_f};

enum Event {
    Menu(MenuEvent),
    Tray(TrayIconEvent),
}

struct Tray {
    icon: TrayIcon,
    show_hide: MenuItem,
    play_stop: MenuItem,
    quit: MenuItem,
}

#[derive(Default)]
pub struct TrayIntegration {
    tray: Rc<RefCell<Option<Tray>>>,
}

impl TrayIntegration {
    pub fn setup(&self, app: &SwApplication) {
        // Tray and menu events arrive outside of GTK; handle them on the main loop
        let (sender, receiver) = async_channel::unbounded::<Event>();
        let menu_sender = sender.clone();
        MenuEvent::set_event_handler(Some(move |event| {
            let _ = menu_sender.send_blocking(Event::Menu(event));
        }));
        TrayIconEvent::set_event_handler(Some(move |event| {
            let _ = sender.send_blocking(Event::Tray(event));
        }));

        let tray = self.tray.clone();
        glib::spawn_future_local(async move {
            while let Ok(event) = receiver.recv().await {
                handle_event(&tray, event).await;
            }
        });

        // Show the icon only while background playback is enabled
        let tray = self.tray.clone();
        let sync_visibility = move |app: &SwApplication| {
            let enabled = app.background_playback();
            let mut tray = tray.borrow_mut();
            if enabled && tray.is_none() {
                match create_tray() {
                    Ok(t) => *tray = Some(t),
                    Err(err) => warn!("Unable to create tray icon: {err:#}"),
                }
            } else if !enabled {
                // Dropping the TrayIcon removes it from the notification area
                *tray = None;
            }
        };
        sync_visibility(app);
        app.connect_notify_local(Some("background-playback"), move |app, _| sync_visibility(app));

        // Keep labels and tooltip in sync
        let tray = self.tray.clone();
        let update = move || update(&tray);
        let player = app.player();
        for property in ["state", "station"] {
            let update = update.clone();
            player.connect_notify_local(Some(property), move |_, _| update());
        }
        for window in app.windows() {
            let update = update.clone();
            window.connect_visible_notify(move |_| update());
        }
        app.connect_window_added(move |_, window| {
            let update = update.clone();
            update();
            window.connect_visible_notify(move |_| update());
        });
    }
}

fn create_tray() -> Result<Tray> {
    let show_hide = MenuItem::new(i18n("Show Window"), true, None);
    let play_stop = MenuItem::new(i18n("Play"), true, None);
    let quit = MenuItem::new(i18n("Quit"), true, None);
    let menu = Menu::with_items(&[
        &show_hide,
        &PredefinedMenuItem::separator(),
        &play_stop,
        &PredefinedMenuItem::separator(),
        &quit,
    ])?;

    let icon = TrayIconBuilder::new()
        .with_icon(app_icon()?)
        .with_tooltip(config::NAME)
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .build()?;

    let tray = Tray {
        icon,
        show_hide,
        play_stop,
        quit,
    };
    debug!("Tray icon created");
    Ok(tray)
}

/// The app icon (hicolor SVG) as RGBA pixels.
fn app_icon() -> Result<Icon> {
    let path = super::datadir()
        .join("icons/hicolor/scalable/apps")
        .join(format!("{}.svg", config::APP_ID));
    let texture = gdk::Texture::from_filename(&path)
        .map_err(|err| anyhow!("{}: {err}", path.display()))?;

    let mut downloader = gdk::TextureDownloader::new(&texture);
    downloader.set_format(gdk::MemoryFormat::R8g8b8a8);
    let (bytes, stride) = downloader.download_bytes();

    let (width, height) = (texture.width() as usize, texture.height() as usize);
    let mut rgba = Vec::with_capacity(width * height * 4);
    for row in bytes.chunks(stride).take(height) {
        rgba.extend_from_slice(&row[..width * 4]);
    }

    Ok(Icon::from_rgba(rgba, width as u32, height as u32)?)
}

fn main_window_visible() -> bool {
    SwApplication::default()
        .windows()
        .iter()
        .any(|window| window.is_visible())
}

fn update(tray: &Rc<RefCell<Option<Tray>>>) {
    let tray = tray.borrow();
    let Some(tray) = tray.as_ref() else {
        return;
    };

    let player = SwApplication::default().player();
    let playing = matches!(
        player.state(),
        SwPlaybackState::Playing | SwPlaybackState::Loading
    );

    tray.show_hide.set_text(if main_window_visible() {
        i18n("Hide Window")
    } else {
        i18n("Show Window")
    });
    tray.play_stop.set_text(if playing { i18n("Stop") } else { i18n("Play") });
    tray.play_stop.set_enabled(player.station().is_some());

    let tooltip = match player.station() {
        Some(station) if playing => format!(
            "{} – {}",
            config::NAME,
            i18n_f("Playing “{}”", &[&station.title()])
        ),
        _ => config::NAME.to_string(),
    };
    // The notification area limits tooltips to 127 characters
    let tooltip: String = tooltip.chars().take(127).collect();
    if let Err(err) = tray.icon.set_tooltip(Some(tooltip)) {
        warn!("Unable to update tray tooltip: {err}");
    }
}

fn toggle_window() {
    let app = SwApplication::default();
    let visible: Vec<gtk::Window> = app.windows().into_iter().filter(|w| w.is_visible()).collect();
    if visible.is_empty() {
        app.activate();
    } else {
        for window in visible {
            window.set_visible(false);
        }
    }
}

async fn handle_event(tray: &Rc<RefCell<Option<Tray>>>, event: Event) {
    let app = SwApplication::default();

    match event {
        Event::Tray(TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        }) => toggle_window(),
        Event::Menu(event) => {
            // Find out which item was activated without holding the borrow across awaits
            let (show_hide, play_stop, quit) = match tray.borrow().as_ref() {
                Some(t) => (
                    t.show_hide.id().clone(),
                    t.play_stop.id().clone(),
                    t.quit.id().clone(),
                ),
                None => return,
            };

            if event.id == show_hide {
                toggle_window();
            } else if event.id == play_stop {
                app.player().toggle_playback().await;
            } else if event.id == quit {
                app.quit();
            }
        }
        _ => (),
    }

    update(tray);
}
