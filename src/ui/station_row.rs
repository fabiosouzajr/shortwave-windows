// Shortwave - station_row.rs
// Copyright (C) 2021-2024  Felix Häcker <haeckerfelix@gnome.org>
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

use std::cell::RefCell;

use adw::subclass::prelude::*;
use glib::clone;
use glib::subclass;
use glib::Properties;
use gtk::prelude::*;
use gtk::{glib, CompositeTemplate};
use inflector::Inflector;

use crate::api::StationMetadata;
use crate::i18n::i18n;
use crate::api::SwStation;
use crate::ui::SwStationCover;
use crate::SwApplication;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate, Properties)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/station_row.ui")]
    #[properties(wrapper_type = super::SwStationRow)]
    pub struct SwStationRow {
        #[template_child]
        station_label: TemplateChild<gtk::Label>,
        #[template_child]
        subtitle_label: TemplateChild<gtk::Label>,
        #[template_child]
        quality_box: TemplateChild<gtk::Box>,
        #[template_child]
        codec_image: TemplateChild<gtk::Image>,
        #[template_child]
        codec_label: TemplateChild<gtk::Label>,
        #[template_child]
        quality_separator: TemplateChild<gtk::Label>,
        #[template_child]
        bars_box: TemplateChild<gtk::Box>,
        #[template_child]
        bitrate_label: TemplateChild<gtk::Label>,
        #[template_child]
        station_cover: TemplateChild<SwStationCover>,
        #[template_child]
        local_image: TemplateChild<gtk::Image>,
        #[template_child]
        orphaned_image: TemplateChild<gtk::Image>,
        #[template_child]
        play_button: TemplateChild<gtk::Button>,

        #[property(get, set=Self::set_station)]
        station: RefCell<Option<SwStation>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwStationRow {
        const NAME: &'static str = "SwStationRow";
        type ParentType = adw::Bin;
        type Type = super::SwStationRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SwStationRow {
        fn constructed(&self) {
            self.parent_constructed();

            self.play_button.connect_clicked(clone!(
                #[weak(rename_to = obj)]
                self.obj(),
                move |_| {
                    glib::spawn_future_local(clone!(
                        #[weak]
                        obj,
                        async move {
                            if let Some(station) = obj.station() {
                                let player = SwApplication::default().player();
                                player.set_station(station).await;
                                player.start_playback().await;
                            }
                        }
                    ));
                }
            ));
        }
    }

    impl WidgetImpl for SwStationRow {}

    impl BinImpl for SwStationRow {}

    impl SwStationRow {
        fn set_station(&self, station: Option<&SwStation>) {
            if let Some(station) = station {
                station.connect_metadata_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |s| {
                        imp.set_metadata(s.metadata());
                    }
                ));
                self.set_metadata(station.metadata());
            }

            *self.station.borrow_mut() = station.cloned();
        }

        fn set_metadata(&self, metadata: StationMetadata) {
            self.station_label.set_text(&metadata.name);

            let codec = metadata.display_codec();
            let has_codec = codec.is_some();
            let has_bitrate = metadata.bitrate > 0;

            if let Some(codec) = codec {
                let icon = if metadata.is_lossless() {
                    "codec-lossless-symbolic"
                } else {
                    "codec-lossy-symbolic"
                };
                self.codec_image.set_icon_name(Some(icon));
                self.codec_label.set_text(&codec);
            }
            self.codec_image.set_visible(has_codec);
            self.codec_label.set_visible(has_codec);

            if has_bitrate {
                let level = metadata.bitrate_level();
                let mut bar = self.bars_box.first_child();
                let mut i = 0;
                while let Some(widget) = bar {
                    if i < level {
                        widget.add_css_class("lit");
                    } else {
                        widget.remove_css_class("lit");
                    }
                    bar = widget.next_sibling();
                    i += 1;
                }
                self.bitrate_label
                    .set_markup(&bitrate_markup(&i18n("{} kbit/s"), metadata.bitrate));
            }
            self.bars_box.set_visible(has_bitrate);
            self.bitrate_label.set_visible(has_bitrate);

            self.quality_separator.set_visible(has_codec && has_bitrate);
            self.quality_box.set_visible(has_codec || has_bitrate);

            let mut subtitle = metadata.country.to_title_case();

            if subtitle.is_empty() {
                subtitle = metadata.tags;
            } else if !metadata.tags.is_empty() {
                subtitle = format!("{} · {}", subtitle, metadata.formatted_tags());
            }

            self.subtitle_label.set_text(&subtitle);
            self.subtitle_label.set_visible(!subtitle.is_empty());
        }
    }
}

glib::wrapper! {
    pub struct SwStationRow(ObjectSubclass<imp::SwStationRow>)
        @extends gtk::Widget, adw::Bin;
}

impl SwStationRow {
    pub fn new(station: &SwStation) -> Self {
        glib::Object::builder().property("station", station).build()
    }
}

/// Pango markup for a translated bitrate template such as `{} kbit/s`: the
/// number in a monospace font, the surrounding unit text smaller.
fn bitrate_markup(template: &str, bitrate: i32) -> String {
    let unit = |text: &str| {
        if text.is_empty() {
            String::new()
        } else {
            format!("<span size=\"smaller\">{}</span>", glib::markup_escape_text(text))
        }
    };
    let (before, after) = template.split_once("{}").unwrap_or(("", template));

    format!(
        "{}<span font_family=\"monospace\">{bitrate}</span>{}",
        unit(before),
        unit(after)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitrate_markup_monospace_number_small_unit() {
        assert_eq!(
            bitrate_markup("{} kbit/s", 128),
            "<span font_family=\"monospace\">128</span><span size=\"smaller\"> kbit/s</span>"
        );
    }

    #[test]
    fn bitrate_markup_escapes_translation() {
        assert_eq!(
            bitrate_markup("<{}> & kb", 64),
            "<span size=\"smaller\">&lt;</span><span font_family=\"monospace\">64</span><span size=\"smaller\">&gt; &amp; kb</span>"
        );
    }
}
