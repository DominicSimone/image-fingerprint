use eframe::{egui, epi};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    fp_list: Vec<String>,

    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    add_file_value: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            fp_list: vec![],
            add_file_value: "".to_string(),
        }
    }
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "Image Fingerprint + Search"
    }

    /// Called by the framework to load old app state (if any).
    #[cfg(feature = "persistence")]
    pub fn load(&mut self, storage: &dyn epi::Storage) {
        *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
        println!("Loaded!");
    }

    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    pub fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
        println!("Saved!");
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let Self {
            add_file_value,
            fp_list,
        } = self;

        // egui::TopPanel::top("top_panel").show(ctx, |ui| {
        //     // The top panel is often a good place for a menu bar:
        //     egui::menu::bar(ui, |ui| {
        //         egui::menu::menu(ui, "File", |ui| {
        //             if ui.button("Add fingerprint list").clicked() {
        //             }
        //         });
        //     });
        // });

        // SidePanel contains the fingerprint storage list and an input/button pair to add file paths to the list
        egui::SidePanel::left("side_panel", 300.0).show(ctx, |ui| {
            ui.heading("Fingerprint lists");

            ui.horizontal_wrapped(|ui| {
                ui.label("Copy + paste file paths into this box. \
                On windows, shift+right click in the file explorer will give the option to copy a file path.");
            });

            // Add fingerprint filepath input + button
            ui.horizontal(|ui| {
                ui.text_edit_singleline(add_file_value);
                if ui.button("Add file path").clicked() {
                    fp_list.push(add_file_value.trim_matches('\"').to_string());
                    *add_file_value = "".to_string();
                }
            });


            // List of fingerprint files to be scanned
            egui::ScrollArea::from_max_height(f32::INFINITY).show(ui, |ui| {
                ui.vertical(|ui| {
                    let mut to_remove = 0;
                    let mut remove = false;
                    for (i, item) in fp_list.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}", item));
                            if ui.button("X").clicked() {
                                to_remove = i;
                                remove = true;
                            }
                        });
                    }
                    if remove {
                        fp_list.remove(to_remove);
                    }
                });
            });

        });

        // Main panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Canvas");

            egui::warn_if_debug_build(ui);
        });
    }
}
