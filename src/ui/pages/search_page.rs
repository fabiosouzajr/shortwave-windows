// Shortwave - search_page.rs
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

use std::cell::{Cell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, subclass};
use gtk::{glib, CompositeTemplate};
use rand::seq::SliceRandom;

use crate::api::{client, Country, Error, StationRequest, SwStation, SwStationModel};
use crate::i18n::{i18n, i18n_f};
use crate::ui::{DisplayError, SwStationDialog, SwStationRow};

/// Codec filter values, exactly as radio-browser stores them (exact match).
const CODECS: [&str; 5] = ["MP3", "AAC", "AAC+", "OGG", "FLAC"];
/// Minimum bitrate filter values in kbit/s.
const BITRATES: [u32; 4] = [64, 128, 192, 320];

/// The option picked in a filter dropdown whose first entry is "Any".
fn filter_choice<T: Copy>(options: &[T], selected: u32) -> Option<T> {
    let index = selected.checked_sub(1)? as usize;
    options.get(index).copied()
}

/// Dropdown model: the "no filter" entry (e.g. "Any Country") followed by `labels`.
fn any_and(any: &str, labels: impl IntoIterator<Item = String>) -> gtk::StringList {
    let list = gtk::StringList::new(&[any]);
    for label in labels {
        list.append(&label);
    }
    list
}

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/search_page.ui")]
    pub struct SwSearchPage {
        #[template_child]
        search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        popular_flowbox: TemplateChild<gtk::FlowBox>,
        #[template_child]
        random_flowbox: TemplateChild<gtk::FlowBox>,
        #[template_child]
        search_gridview: TemplateChild<gtk::GridView>,
        #[template_child]
        failure_statuspage: TemplateChild<adw::StatusPage>,
        #[template_child]
        filter_box: TemplateChild<gtk::Box>,
        #[template_child]
        country_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        codec_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        bitrate_dropdown: TemplateChild<gtk::DropDown>,

        popular_model: SwStationModel,
        random_model: SwStationModel,
        search_model: SwStationModel,

        countries: RefCell<Vec<Country>>,
        search_generation: Cell<u64>,
        loaded: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwSearchPage {
        const NAME: &'static str = "SwSearchPage";
        type ParentType = adw::NavigationPage;
        type Type = super::SwSearchPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SwSearchPage {
        fn constructed(&self) {
            // Discover view
            let flowbox_widget_func = |s: &glib::Object| {
                let station: &SwStation = s.downcast_ref().unwrap();
                let row = SwStationRow::new(station);
                let child = gtk::FlowBoxChild::new();
                child.set_child(Some(&row));
                child.into()
            };

            self.popular_flowbox
                .bind_model(Some(&self.popular_model), flowbox_widget_func);
            self.random_flowbox
                .bind_model(Some(&self.random_model), flowbox_widget_func);

            let child_activate_func = |flowbox: &gtk::FlowBox, child: &gtk::FlowBoxChild| {
                let row = child.child().unwrap().downcast::<SwStationRow>().unwrap();
                if let Some(station) = row.station() {
                    let station_dialog = SwStationDialog::new(&station);
                    station_dialog.present(Some(flowbox));
                }
            };

            self.popular_flowbox
                .connect_child_activated(child_activate_func);
            self.random_flowbox
                .connect_child_activated(child_activate_func);

            // Search grid view
            let model = gtk::NoSelection::new(Some(self.search_model.clone()));
            self.search_gridview.set_model(Some(&model));

            self.search_gridview
                .connect_activate(|gv: &gtk::GridView, pos| {
                    let model = gv.model().unwrap();
                    let station = model.item(pos).unwrap().downcast::<SwStation>().unwrap();
                    let station_dialog = SwStationDialog::new(&station);
                    station_dialog.present(Some(gv));
                });

            // Search filters
            let string_expr = gtk::PropertyExpression::new(
                gtk::StringObject::static_type(),
                None::<&gtk::Expression>,
                "string",
            );
            let bitrates = BITRATES.map(|b| i18n_f("≥ {} kbit/s", &[&b.to_string()]));
            let models = [
                (&*self.country_dropdown, any_and(&i18n("Any Country"), [])),
                (
                    &*self.codec_dropdown,
                    any_and(&i18n("Any Codec"), CODECS.map(String::from)),
                ),
                (&*self.bitrate_dropdown, any_and(&i18n("Any Bitrate"), bitrates)),
            ];
            for (dropdown, model) in models {
                dropdown.set_model(Some(&model));
                dropdown.set_expression(Some(&string_expr));
                // Formal names ("The United States Of America"): match anywhere
                dropdown.set_search_match_mode(gtk::StringFilterMatchMode::Substring);
                dropdown.set_factory(Some(&label_factory(true)));
                dropdown.set_list_factory(Some(&label_factory(false)));
                dropdown.connect_selected_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_| {
                        glib::spawn_future_local(clone!(
                            #[weak]
                            imp,
                            async move { imp.search_changed().await }
                        ));
                    }
                ));
            }

            self.stack.set_visible_child_name("spinner");
        }
    }

    /// Dropdown item factory showing the item's string; the closed button
    /// ellipsizes so long country names don't widen the filter row.
    fn label_factory(ellipsize: bool) -> gtk::SignalListItemFactory {
        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(move |_, item| {
            let label = gtk::Label::builder().xalign(0.0).build();
            if ellipsize {
                label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            }
            item.downcast_ref::<gtk::ListItem>()
                .unwrap()
                .set_child(Some(&label));
        });
        factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let text = item
                .item()
                .and_downcast::<gtk::StringObject>()
                .map(|s| s.string());
            let label = item.child().and_downcast::<gtk::Label>().unwrap();
            label.set_text(text.as_deref().unwrap_or_default());
        });
        factory
    }

    impl WidgetImpl for SwSearchPage {
        fn map(&self) {
            self.parent_map();

            if !self.loaded.get() {
                glib::spawn_future_local(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    async move {
                        imp.refresh_discover_page().await;
                    }
                ));
            }
        }
    }

    impl NavigationPageImpl for SwSearchPage {}

    #[gtk::template_callbacks]
    impl SwSearchPage {
        #[template_callback]
        async fn refresh_discover_page(&self) {
            self.stack.set_visible_child_name("spinner");

            match self.load_discover_stations().await {
                Ok(()) => {
                    if self.countries.borrow().is_empty() {
                        self.load_countries().await;
                    }
                    self.loaded.set(true);
                    self.search_entry.set_sensitive(true);
                    self.filter_box.set_visible(true);
                    self.stack.set_visible_child_name("discover");
                }
                Err(e) => {
                    self.stack.set_visible_child_name("failure");
                    self.failure_statuspage
                        .set_description(Some(&e.to_string()));
                }
            }
        }

        async fn load_discover_stations(&self) -> Result<(), Error> {
            debug!("Update discover stations...");
            let countrycode = Self::region_code().unwrap_or("GB".into());

            // Popular stations
            let request = StationRequest {
                limit: Some(100),
                order: Some("votes".into()),
                reverse: Some(true),
                countrycode: Some(countrycode.clone()),
                ..Default::default()
            };

            let mut stations = client::station_request(request).await?;

            // Anything more than 50k votes can be considered as botted spam
            stations.retain(|s| s.metadata().votes < 50_000);

            // Randomize the selection to avoid that always the same stations are visible
            let stations = stations
                .choose_multiple(&mut rand::thread_rng(), 12)
                .cloned()
                .collect::<Vec<SwStation>>();

            self.popular_model.clear();
            self.popular_model.add_stations(stations);

            // Random stations
            let request = StationRequest {
                limit: Some(18),
                order: Some("random".into()),
                countrycode: Some(countrycode),
                ..Default::default()
            };

            let stations = client::station_request(request).await?;
            self.random_model.clear();
            self.random_model.add_stations(stations);

            Ok(())
        }

        /// Fill the country filter; on failure it keeps only "Any Country".
        async fn load_countries(&self) {
            match client::countries().await {
                Ok(countries) => {
                    let model = any_and(
                        &i18n("Any Country"),
                        countries.iter().map(|c| c.name.clone()),
                    );
                    *self.countries.borrow_mut() = countries;
                    self.country_dropdown.set_model(Some(&model));
                }
                Err(e) => warn!("Unable to load country list: {}", e),
            }
        }

        #[template_callback]
        async fn search_changed(&self) {
            if !self.loaded.get() {
                return;
            }

            let text = self.search_entry.text().trim().to_string();
            let text = if text.is_empty() { None } else { Some(text) };

            let countrycode = self
                .country_dropdown
                .selected()
                .checked_sub(1)
                .and_then(|i| self.countries.borrow().get(i as usize).cloned())
                .map(|c| c.iso_3166_1);
            let codec = filter_choice(&CODECS, self.codec_dropdown.selected()).map(String::from);
            let bitrate_min = filter_choice(&BITRATES, self.bitrate_dropdown.selected());

            // Nothing to search for: back to the discover view
            if text.is_none() && countrycode.is_none() && codec.is_none() && bitrate_min.is_none() {
                self.search_generation.set(self.search_generation.get() + 1);
                self.stack.set_visible_child_name("discover");
                return;
            }

            let generation = self.search_generation.get() + 1;
            self.search_generation.set(generation);

            let filters = StationRequest::filtered(countrycode, codec, bitrate_min);
            self.stack.set_visible_child_name("spinner");

            debug!("Search for: {:?} {:?}", text, filters);
            let res = client::search_stations(text, filters).await;

            // A newer search started meanwhile: its result wins
            if generation != self.search_generation.get() {
                return;
            }
            res.handle_error("Unable to search for stations");

            if let Ok(stations) = res {
                if stations.is_empty() {
                    self.stack.set_visible_child_name("no-results");
                } else {
                    self.stack.set_visible_child_name("results");
                }

                self.search_model.clear();
                self.search_model.add_stations(stations);
            }
        }

        fn region_code() -> Option<String> {
            let locale = sys_locale::get_locale()?;
            let langtag = language_tags::LanguageTag::parse(&locale).ok()?;
            langtag.region().map(|s| s.to_string())
        }
    }
}

glib::wrapper! {
    pub struct SwSearchPage(ObjectSubclass<imp::SwSearchPage>)
        @extends gtk::Widget, adw::NavigationPage;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_choice_any_and_invalid_are_none() {
        assert_eq!(filter_choice(&BITRATES, 0), None);
        assert_eq!(filter_choice(&BITRATES, gtk::INVALID_LIST_POSITION), None);
        assert_eq!(filter_choice(&BITRATES, 99), None);
    }

    #[test]
    fn filter_choice_maps_past_any() {
        assert_eq!(filter_choice(&BITRATES, 1), Some(64));
        assert_eq!(filter_choice(&CODECS, 3), Some("AAC+"));
    }
}
