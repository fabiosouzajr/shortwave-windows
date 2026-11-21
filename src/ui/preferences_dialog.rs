// Shortwave - preferences_dialog.rs
// Copyright (C) 2021-2025  Felix Häcker <haeckerfelix@gnome.org>
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

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, subclass};
use gtk::{gio, glib, CompositeTemplate};

use crate::i18n::{i18n, ni18n_f};
use crate::settings::{settings_manager, Key};

/// Languages with a translation (see po/LINGUAS), shown by their own name.
const LANGUAGES: &[(&str, &str)] = &[
    ("be", "Беларуская"),
    ("bg", "Български"),
    ("ca", "Català"),
    ("cs", "Čeština"),
    ("da", "Dansk"),
    ("de", "Deutsch"),
    ("el", "Ελληνικά"),
    ("en", "English"),
    ("en_GB", "English (UK)"),
    ("es", "Español"),
    ("eu", "Euskara"),
    ("fa", "فارسی"),
    ("fi", "Suomi"),
    ("fr", "Français"),
    ("fur", "Furlan"),
    ("gl", "Galego"),
    ("he", "עברית"),
    ("hi", "हिन्दी"),
    ("hr", "Hrvatski"),
    ("hu", "Magyar"),
    ("ia", "Interlingua"),
    ("id", "Bahasa Indonesia"),
    ("is", "Íslenska"),
    ("it", "Italiano"),
    ("ka", "ქართული"),
    ("nl", "Nederlands"),
    ("oc", "Occitan"),
    ("pl", "Polski"),
    ("pt", "Português"),
    ("pt_BR", "Português (Brasil)"),
    ("ro", "Română"),
    ("ru", "Русский"),
    ("sk", "Slovenčina"),
    ("sl", "Slovenščina"),
    ("sr", "Српски"),
    ("sv", "Svenska"),
    ("tr", "Türkçe"),
    ("uk", "Українська"),
    ("vi", "Tiếng Việt"),
    ("zh_CN", "简体中文"),
    ("zh_TW", "繁體中文"),
];

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/preferences_dialog.ui")]
    pub struct SwPreferencesDialog {
        // General
        #[template_child]
        language_row: TemplateChild<adw::ComboRow>,

        // Playback
        #[template_child]
        background_playback_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        notifications_switch: TemplateChild<gtk::Switch>,

        // Recording
        #[template_child]
        recording_track_directory_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        recording_maximum_duration_row: TemplateChild<adw::SpinRow>,
        #[template_child]
        recording_minimum_duration_row: TemplateChild<adw::SpinRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwPreferencesDialog {
        const NAME: &'static str = "SwSettingsDialog";
        type ParentType = adw::PreferencesDialog;
        type Type = super::SwPreferencesDialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SwPreferencesDialog {
        fn constructed(&self) {
            // Language: first entry is the system language (empty setting)
            let names = std::iter::once(i18n("System Default"))
                .chain(LANGUAGES.iter().map(|(_, name)| name.to_string()))
                .collect::<Vec<_>>();
            let names = names.iter().map(String::as_str).collect::<Vec<_>>();
            self.language_row
                .set_model(Some(&gtk::StringList::new(&names)));

            let current = settings_manager::string(Key::Language);
            let selected = LANGUAGES
                .iter()
                .position(|(code, _)| *code == current)
                .map_or(0, |i| i as u32 + 1);
            self.language_row.set_selected(selected);

            self.language_row.connect_selected_notify(|row| {
                let code = match row.selected() {
                    0 => "",
                    i => LANGUAGES[i as usize - 1].0,
                };
                settings_manager::set_string(Key::Language, code.to_string());
            });

            // Playback
            settings_manager::bind_property(
                Key::BackgroundPlayback,
                &*self.background_playback_switch,
                "active",
            );

            settings_manager::bind_property(
                Key::Notifications,
                &*self.notifications_switch,
                "active",
            );

            // Recording
            let recording_mode_action = settings_manager::create_action(Key::RecordingMode);
            let group = gio::SimpleActionGroup::new();
            group.add_action(&recording_mode_action);
            self.obj().insert_action_group("player", Some(&group));

            settings_manager::bind_property(
                Key::RecordingTrackDirectory,
                &*self.recording_track_directory_row,
                "subtitle",
            );

            self.recording_track_directory_row.connect_activated(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.select_recording_save_directory();
                }
            ));

            settings_manager::bind_property(
                Key::RecordingMaximumDuration,
                &*self.recording_maximum_duration_row,
                "value",
            );

            self.recording_maximum_duration_row.connect_input(|row| {
                let mut text = row.text().to_string();
                text.retain(|c| c.is_numeric());

                if let Ok(value) = text.parse::<f64>() {
                    Some(Ok(value * 60.0))
                } else {
                    None
                }
            });

            settings_manager::bind_property(
                Key::RecordingMinimumDuration,
                &*self.recording_minimum_duration_row,
                "value",
            );
        }
    }

    impl WidgetImpl for SwPreferencesDialog {}

    impl AdwDialogImpl for SwPreferencesDialog {}

    impl PreferencesDialogImpl for SwPreferencesDialog {}

    #[gtk::template_callbacks]
    impl SwPreferencesDialog {
        pub fn select_recording_save_directory(&self) {
            let parent = self
                .obj()
                .root()
                .unwrap()
                .downcast::<gtk::Window>()
                .unwrap();

            let dialog = gtk::FileDialog::new();
            dialog.set_title(&i18n("Select Save Directory"));
            dialog.set_accept_label(Some(&i18n("_Select")));

            dialog.select_folder(
                Some(&parent),
                gio::Cancellable::NONE,
                move |result| match result {
                    Ok(folder) => {
                        debug!("Selected save directory: {:?}", folder.path());
                        settings_manager::set_string(
                            Key::RecordingTrackDirectory,
                            folder.parse_name().to_string(),
                        );
                    }
                    Err(err) => {
                        warn!("Selected directory could not be accessed {:?}", err);
                    }
                },
            );
        }

        #[template_callback]
        fn on_maximum_duration_output(row: &adw::SpinRow) -> bool {
            let value = (row.value() / 60.0) as u32;
            let text = ni18n_f("{} min", "{} min", value, &[&value.to_string()]);
            row.set_text(&text);
            row.set_width_chars(text.len() as i32);
            true
        }

        #[template_callback]
        fn on_minimum_duration_output(row: &adw::SpinRow) -> bool {
            let value = row.value() as u32;
            let text = ni18n_f("{} sec", "{} sec", value, &[&value.to_string()]);
            row.set_text(&text);
            row.set_width_chars(text.len() as i32);
            true
        }
    }
}

glib::wrapper! {
    pub struct SwPreferencesDialog(ObjectSubclass<imp::SwPreferencesDialog>)
        @extends gtk::Widget, adw::Dialog, adw::PreferencesDialog;
}

impl SwPreferencesDialog {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for SwPreferencesDialog {
    fn default() -> Self {
        Self::new()
    }
}
