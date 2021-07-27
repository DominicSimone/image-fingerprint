use clipboard_win::{get_clipboard, formats};
use serde;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    fp_list: Vec<String>,

    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    add_file_name_value: String,

    #[cfg_attr(feature = "persistence", serde(skip))]
    current_image: Option<Image>,

    #[cfg_attr(feature = "persistence", serde(skip))]
    texture_manager: TextureManager,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            fp_list: Default::default(),
            add_file_name_value: Default::default(),
            current_image: Default::default(),
            texture_manager: Default::default(),
        }
    }
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "Image Fingerprint + Search"
    }

    /// Called by the framework to load old app state (if any).
    #[cfg(feature = "persistence")]
    fn load(&mut self, storage: &dyn epi::Storage) {
        *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
    }

    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let Self {
            add_file_name_value,
            fp_list,
            current_image,
            texture_manager,
        } = self;
        
        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {

                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Open image").clicked() {
                    }
                    
                });
            });
        });

        

        // SidePanel contains the fingerprint storage list and an input/button pair to add file paths to the list
        egui::SidePanel::left("side_panel", 300.0).show(ctx, |ui| {
            ui.heading("Fingerprint lists");

            ui.horizontal_wrapped(|ui| {
                ui.label("Copy + paste file paths into this box. \
                On windows, shift+right click in the file explorer will give the option to copy a file path.");
            });

            // Add fingerprint filepath input + button
            ui.horizontal(|ui| {
                ui.text_edit_singleline(add_file_name_value);
                if ui.button("Add file path").clicked() {
                    fp_list.push(add_file_name_value.trim_matches('\"').to_string());
                    *add_file_name_value = "".to_string();
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

            ui.horizontal(|ui| {
                if ui.button("Paste image from clipboard").clicked() {
                    *current_image = Image::from_clipboard();
                    dbg!("Clipboard access attempt");
                }
    
                if ui.button("Clear image").clicked() {
                    *current_image = None;
                }
            });
            
            if let Some(current_image) = current_image {
                if let Some(texture_id) = texture_manager.texture(_frame, current_image) {
                    // TODO Render a thumbnail instead of the full image
                    let size = egui::Vec2::new(current_image.size.0 as f32, current_image.size.1 as f32);
                    ui.image(texture_id, size);
                }
            }
            egui::warn_if_debug_build(ui);
        });
    }
}


// ----------------------------------------------------------------------------
// Texture/image handling is very manual at the moment.

/// Immediate mode texture manager that supports at most one texture at the time :)
#[derive(Default)]
struct TextureManager {
    texture_id: Option<egui::TextureId>,
}

impl TextureManager {
    fn texture(
        &mut self,
        frame: &mut epi::Frame<'_>,
        image: &Image,
    ) -> Option<egui::TextureId> {

        if let Some(texture_id) = self.texture_id.take() {
            frame.tex_allocator().free(texture_id);
        }

        self.texture_id = Some(
            frame
                .tex_allocator()
                .alloc_srgba_premultiplied(image.size, &image.pixels),
        );
        self.texture_id
    }
        
}

struct Image {
    size: (usize, usize),
    pixels: Vec<egui::Color32>,
}

impl Clone for Image {
    fn clone(&self) -> Image {
        Image {
            size: self.size,
            pixels: self.pixels.clone(),
        }
    }
}

impl Image {
    fn decode(bytes: &[u8]) -> Option<Image> {
        use image::GenericImageView;
        let image = image::load_from_memory(bytes).ok()?;
        let image_buffer = image.to_rgba8();
        let size = (image.width() as usize, image.height() as usize);
        let pixels = image_buffer.into_vec();
        assert_eq!(size.0 * size.1 * 4, pixels.len());
        let pixels = pixels
            .chunks(4)
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();

        Some(Image { size, pixels })
    }

    fn from_clipboard() -> Option<Image> {
        if let Some(data) = get_clipboard(formats::Bitmap).ok() {
            return Image::decode(&data)
        } else {
            None
        }
    }
}