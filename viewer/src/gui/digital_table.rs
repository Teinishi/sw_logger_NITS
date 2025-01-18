use crate::{range_check::range_check, values::Values};
use egui::{vec2, Color32, Context, Id, Layout, Ui};
use egui_extras::{Column, TableBuilder};
//use egui_file::FileDialog;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Serialize, Deserialize, PartialEq)]
enum DecodeType {
    Float32,
    Int24,
    RealNumber,
}

impl std::fmt::Display for DecodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeType::Float32 => write!(f, "32bit (float)"),
            DecodeType::Int24 => write!(f, "24bit (integer)"),
            DecodeType::RealNumber => write!(f, "Real Number"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
enum BinaryDisplayStyle {
    Hex,
    Dec,
    Oct,
    Bin,
}

impl std::fmt::Display for BinaryDisplayStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryDisplayStyle::Hex => write!(f, "Hex"),
            BinaryDisplayStyle::Dec => write!(f, "Dec"),
            BinaryDisplayStyle::Oct => write!(f, "Oct"),
            BinaryDisplayStyle::Bin => write!(f, "Bin"),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ColumnProperty {
    key: String,
    decode_type: DecodeType,
    display_style: BinaryDisplayStyle,
    title: Option<String>,
    width: Option<u32>,
}

impl ColumnProperty {
    fn added(&mut self) {
        self.title = Some(self.get_title("\n"));
        self.width = Some(self.get_width());
    }

    fn get_title(&self, separator: &str) -> String {
        match self.decode_type {
            DecodeType::Float32 => format!("{}{}32bit {}", self.key, separator, self.display_style),
            DecodeType::Int24 => format!("{}{}24bit {}", self.key, separator, self.display_style),
            DecodeType::RealNumber => format!("{}{}Real Number", self.key, separator),
        }
    }

    fn get_width(&self) -> u32 {
        match self.decode_type {
            DecodeType::Float32 => match self.display_style {
                BinaryDisplayStyle::Hex => 8,
                BinaryDisplayStyle::Dec => 10,
                BinaryDisplayStyle::Oct => 11,
                BinaryDisplayStyle::Bin => 32,
            },
            DecodeType::Int24 => match self.display_style {
                BinaryDisplayStyle::Hex => 6,
                BinaryDisplayStyle::Dec => 8,
                BinaryDisplayStyle::Oct => 8,
                BinaryDisplayStyle::Bin => 24,
            },
            DecodeType::RealNumber => 10,
        }
    }

    fn format(&self, value: f32) -> (String, Option<String>) {
        match self.decode_type {
            DecodeType::Float32 => {
                let bits = f32::to_bits(value);
                (
                    match self.display_style {
                        BinaryDisplayStyle::Hex => format!("{:08x}", bits),
                        BinaryDisplayStyle::Dec => format!("{:10}", bits),
                        BinaryDisplayStyle::Oct => format!("{:011o}", bits),
                        BinaryDisplayStyle::Bin => format!("{:032b}", bits),
                    },
                    None,
                )
            }
            DecodeType::Int24 => {
                let bits = value.trunc() as u32;
                (
                    match self.display_style {
                        BinaryDisplayStyle::Hex => format!("{:06x}", bits),
                        BinaryDisplayStyle::Dec => format!("{:8}", bits),
                        BinaryDisplayStyle::Oct => format!("{:08o}", bits),
                        BinaryDisplayStyle::Bin => format!("{:024b}", bits),
                    },
                    if value.fract() != 0.0 {
                        Some(format!("Not integer ({:.4})", value))
                    } else if let Err(_) = range_check(&(0.0..((1 << 24) as f32)), value) {
                        Some(format!("Not within 24bit range ({:.4})", value))
                    } else {
                        None
                    },
                )
            }
            DecodeType::RealNumber => (value.to_string(), None),
        }
    }
}

impl Default for ColumnProperty {
    fn default() -> ColumnProperty {
        ColumnProperty {
            key: Default::default(),
            decode_type: DecodeType::Float32,
            display_style: BinaryDisplayStyle::Hex,
            title: None,
            width: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DigitalTableWindow {
    id: Id,
    selector: ColumnProperty,
    columns: Vec<ColumnProperty>,
    /*#[serde(skip, default)]
    save_dialog: Option<FileDialog>,*/
}

impl DigitalTableWindow {
    pub fn new(id: impl Hash) -> Self {
        Self {
            id: Id::new(id),
            selector: Default::default(),
            columns: vec![],
            //save_dialog: None,
        }
    }

    /*pub fn title(&self) -> String {
        self.columns
            .iter()
            .map(|c| c.get_title(" "))
            .collect::<Vec<_>>()
            .join(",")
    }*/

    pub fn show(&mut self, ctx: &Context, open: &mut bool, values: &Values) {
        egui::Window::new("Digital Table")
            .id(self.id)
            .default_size(vec2(100.0, 200.0))
            .vscroll(true)
            .open(open)
            .show(ctx, |ui| self.ui(ui, values));
    }
    pub fn ui(&mut self, ui: &mut Ui, values: &Values) {
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt(self.id.with("key_selector"))
                .selected_text(&self.selector.key)
                .show_ui(ui, |ui| {
                    for key in values.keys() {
                        ui.selectable_value(&mut self.selector.key, key.to_owned(), key);
                    }
                });
            egui::ComboBox::from_id_salt(self.id.with("decode_type_selector"))
                .selected_text(self.selector.decode_type.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.selector.decode_type,
                        DecodeType::Float32,
                        "32bit (float)",
                    );
                    ui.selectable_value(
                        &mut self.selector.decode_type,
                        DecodeType::Int24,
                        "24bit (integer)",
                    );
                    ui.selectable_value(
                        &mut self.selector.decode_type,
                        DecodeType::RealNumber,
                        "Real Number",
                    );
                });
            if self.selector.decode_type != DecodeType::RealNumber {
                egui::ComboBox::from_id_salt(self.id.with("display_style_selector"))
                    .selected_text(self.selector.display_style.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.selector.display_style,
                            BinaryDisplayStyle::Hex,
                            "Hex",
                        );
                        ui.selectable_value(
                            &mut self.selector.display_style,
                            BinaryDisplayStyle::Oct,
                            "Oct",
                        );
                        ui.selectable_value(
                            &mut self.selector.display_style,
                            BinaryDisplayStyle::Dec,
                            "Dec",
                        );
                        ui.selectable_value(
                            &mut self.selector.display_style,
                            BinaryDisplayStyle::Bin,
                            "Bin",
                        );
                    });
            }
            if ui.button("Add").clicked() && values.contains_key(&self.selector.key) {
                let mut column = std::mem::take(&mut self.selector);
                column.added();
                self.columns.push(column);
            }
        });

        /*#[cfg(not(target_arch = "wasm32"))]
        if ui.button("Save CSV").clicked() {
            let mut fd = FileDialog::save_file(None)
                .default_filename(format!("{}.csv", self.title()))
                .title("Save as CSV");
            fd.open();
            self.save_dialog = Some(fd);
        }*/
        ui.separator();

        let mut delete_column = None;

        let table = TableBuilder::new(ui)
            .cell_layout(Layout::left_to_right(egui::Align::Center))
            .columns(Column::auto(), self.columns.len())
            .stick_to_bottom(true);

        table
            .header(20.0, |mut header| {
                for (i, column) in self.columns.iter().enumerate() {
                    header.col(|ui| {
                        if let Some(title) = &column.title {
                            ui.strong(title);
                        }
                        if ui.button("X").clicked() {
                            delete_column = Some(i);
                        }
                    });
                }
            })
            .body(|body| {
                let mut table_values: Vec<_> = self
                    .columns
                    .iter()
                    .map(|column| (values.values_for_key(&column.key), column))
                    .collect();
                let max_len = table_values
                    .iter()
                    .map(|v| v.0.as_ref().map(|v| v.len()).unwrap_or_default())
                    .max()
                    .unwrap_or_default();
                body.rows(20.0, max_len, |mut row| {
                    let index = row.index();
                    for (iter, column) in table_values.iter_mut() {
                        row.col(|ui| {
                            if let Some(it) = iter.as_mut() {
                                let offset = max_len - it.len();
                                if offset <= index {
                                    if let Some(v) = it.get(index - offset) {
                                        let (label_text, tooltip) = column.format(*v);
                                        if let Some(tooltip_text) = tooltip {
                                            ui.colored_label(
                                                Color32::from_rgb(255, 0, 0),
                                                label_text,
                                            )
                                            .on_hover_text(tooltip_text);
                                        } else {
                                            ui.label(label_text);
                                        }
                                    } else {
                                        *iter = None;
                                    }
                                }
                            }
                        });
                    }
                });
            });

        if let Some(i) = delete_column {
            self.columns.remove(i);
        }

        /*if let Some(save_dialog) = self.save_dialog.as_mut() {
            if save_dialog.show(ui.ctx()).selected() {
                if let Some(path) = save_dialog.path() {
                    let _ = values.save_csv(path, self.keys.iter());
                }
                self.save_dialog = None;
            }
        }*/
    }
}
