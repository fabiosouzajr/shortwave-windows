// Shortwave - platform/windows/toast.rs
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

//! Track notifications as Windows toasts.
//!
//! GLib's Win32 notification backend can't show actions, so the
//! "Save Track" / "Don't Record" buttons of upstream's GNotification would be
//! lost. Toasts can, but Windows only shows them for an AppUserModelID that
//! belongs to a Start Menu shortcut, which the installer creates. When toasts
//! aren't available the caller falls back to GNotification.

use std::cell::RefCell;
use std::sync::OnceLock;

use anyhow::{anyhow, Result};
use gtk::prelude::*;
use gtk::glib;
use windows::core::{Interface, HSTRING, IInspectable};
use windows::Data::Xml::Dom::XmlDocument;
use windows::Foundation::TypedEventHandler;
use windows::UI::Notifications::{
    NotificationSetting, ToastActivatedEventArgs, ToastNotification, ToastNotificationManager,
};
use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

use crate::app::SwApplication;
use crate::audio::{SwRecordingMode, SwRecordingState, SwTrack};
use crate::config;
use crate::i18n::i18n;

/// Activation arguments (`action;uuid`) from the WinRT thread to the main loop.
static ACTIVATIONS: OnceLock<async_channel::Sender<String>> = OnceLock::new();

thread_local! {
    /// The toast on screen; replaced by the next one (like GNotification ids).
    static CURRENT: RefCell<Option<ToastNotification>> = const { RefCell::new(None) };
}

/// Identifies the process with the AppUserModelID of the installer's Start
/// Menu shortcut, so the taskbar, SMTC and toasts all attribute to it.
pub fn set_app_user_model_id() {
    let id = HSTRING::from(config::APP_ID);
    if let Err(err) = unsafe { SetCurrentProcessExplicitAppUserModelID(&id) } {
        warn!("Unable to set AppUserModelID: {err}");
    }
}

/// Shows the track notification as a toast. Returns false if toasts can't be
/// used, in which case the caller should send a GNotification instead.
pub fn show_track(track: &SwTrack, mode: SwRecordingMode) -> bool {
    match show(track, mode) {
        Ok(()) => true,
        Err(err) => {
            debug!("Toast notifications unavailable, using GNotification: {err:#}");
            false
        }
    }
}

fn show(track: &SwTrack, mode: SwRecordingMode) -> Result<()> {
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(config::APP_ID))?;
    let setting = notifier.Setting()?;
    if setting != NotificationSetting::Enabled {
        return Err(anyhow!("notifications not enabled for {} ({setting:?})", config::APP_ID));
    }

    // Same content and buttons as SwPlayer::track_notification (audio/player.rs)
    let uuid = track.uuid();
    let mut actions = String::new();
    if track.state() == SwRecordingState::Recording {
        if mode == SwRecordingMode::Decide {
            actions += &action_xml(&i18n("Save Track"), "save-track", &uuid);
        }
        if mode == SwRecordingMode::Everything || mode == SwRecordingMode::Decide {
            actions += &action_xml(&i18n("Don't Record"), "cancel-recording", &uuid);
        }
    }
    let xml = format!(
        r#"<toast launch="{launch}"><visual><binding template="ToastGeneric"><text>{title}</text><text>{body}</text></binding></visual><actions>{actions}</actions></toast>"#,
        launch = escape(&format!("show-track;{uuid}")),
        title = escape(&track.title()),
        body = escape(&track.station().title()),
    );

    let doc = XmlDocument::new()?;
    doc.LoadXml(&HSTRING::from(xml))?;
    let toast = ToastNotification::CreateToastNotification(&doc)?;
    toast.SetTag(&HSTRING::from("track"))?;

    let sender = activation_sender().clone();
    toast.Activated(&TypedEventHandler::<ToastNotification, IInspectable>::new(
        move |_, args| {
            let args = args.ok()?.cast::<ToastActivatedEventArgs>()?;
            let _ = sender.send_blocking(args.Arguments()?.to_string());
            Ok(())
        },
    ))?;

    notifier.Show(&toast)?;
    CURRENT.with(|current| current.replace(Some(toast)));
    Ok(())
}

fn action_xml(label: &str, action: &str, uuid: &str) -> String {
    format!(
        r#"<action content="{}" arguments="{}" activationType="foreground"/>"#,
        escape(label),
        escape(&format!("{action};{uuid}"))
    )
}

fn escape(text: &str) -> String {
    glib::markup_escape_text(text).to_string()
}

/// Runs the app action for a clicked toast or button on the main loop.
fn activation_sender() -> &'static async_channel::Sender<String> {
    ACTIVATIONS.get_or_init(|| {
        let (sender, receiver) = async_channel::unbounded::<String>();
        glib::spawn_future_local(async move {
            while let Ok(arguments) = receiver.recv().await {
                let Some((action, uuid)) = arguments.split_once(';') else {
                    continue;
                };
                debug!("Toast activated: app.{action} ({uuid})");
                SwApplication::default().activate_action(action, Some(&uuid.to_variant()));
            }
        });
        sender
    })
}
