use crate::values::Values;
use egui::{vec2, Context, Id, ScrollArea, Ui};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
enum Corner {
    LeftTop,
    RightTop,
    LeftBottom,
    RightBottom,
}

impl From<Corner> for egui_plot::Corner {
    fn from(c: Corner) -> Self {
        match c {
            Corner::LeftTop => egui_plot::Corner::LeftTop,
            Corner::RightTop => egui_plot::Corner::RightTop,
            Corner::LeftBottom => egui_plot::Corner::LeftBottom,
            Corner::RightBottom => egui_plot::Corner::RightBottom,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
enum VPlacement {
    Top,
    Bottom,
}

impl From<VPlacement> for egui_plot::VPlacement {
    fn from(p: VPlacement) -> Self {
        match p {
            VPlacement::Top => egui_plot::VPlacement::Top,
            VPlacement::Bottom => egui_plot::VPlacement::Bottom,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
enum HPlacement {
    Left,
    Right,
}

impl From<HPlacement> for egui_plot::HPlacement {
    fn from(p: HPlacement) -> Self {
        match p {
            HPlacement::Left => egui_plot::HPlacement::Left,
            HPlacement::Right => egui_plot::HPlacement::Right,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LineGraph {
    id: Id,
    title: String,
    keys: Vec<String>,
    legend_position: Corner,
    x_axis_position: VPlacement,
    y_axis_position: HPlacement,
    period: usize,
}

impl LineGraph {
    pub fn new(id: impl Hash, key: String) -> Self {
        let id = Id::new(id);
        Self {
            id,
            title: key.clone(),
            keys: vec![key],
            legend_position: Corner::LeftTop,
            x_axis_position: VPlacement::Bottom,
            y_axis_position: HPlacement::Right,
            period: 3600,
        }
    }

    pub fn show(&mut self, ctx: &Context, open: &mut bool, values: &Values) {
        egui::Window::new(&self.title)
            .id(self.id)
            .default_size(vec2(400.0, 600.0))
            .vscroll(false)
            .open(open)
            .show(ctx, |ui| self.ui(ui, values));
    }

    pub fn ui(&mut self, ui: &mut Ui, values: &Values) {
        ScrollArea::horizontal()
            .id_salt(self.id.with("header"))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for key in values.keys() {
                        if ui.selectable_label(self.keys.contains(key), key).clicked() {
                            if let Some(index) = self.keys.iter().position(|k| k == key) {
                                self.keys.remove(index);
                            } else {
                                self.keys.push(key.to_owned());
                            }
                            self.title = self.keys.join(", ");
                        }
                    }
                });
            });
        ui.separator();
        Plot::new(self.id.with("plot"))
            .legend(Legend::default().position(self.legend_position.into()))
            .x_axis_position(self.x_axis_position.into())
            .y_axis_position(self.y_axis_position.into())
            .y_axis_min_width(5.0)
            .show_axes(true)
            .show_grid(true)
            .show(ui, |ui| {
                for k in &self.keys {
                    if let Some(iter) = values.iter_for_key(k) {
                        let skip = iter.len().saturating_sub(self.period);
                        let iter = iter.skip(skip);
                        let len = iter.len();
                        let line = Line::new(PlotPoints::from_iter(
                            iter.enumerate()
                                .map(|(c, v)| [(c as f64 - len as f64) / 60.0, *v as f64]),
                        ))
                        .name(k);
                        ui.line(line);
                    }
                }
            })
            .response
            .context_menu(|ui| {
                graph_context_menu(
                    ui,
                    &mut self.legend_position,
                    &mut self.x_axis_position,
                    &mut self.y_axis_position,
                    &mut self.period,
                )
            });
    }
}

#[derive(Serialize, Deserialize)]
pub struct XYGraph {
    id: Id,
    selector: (String, String),
    keys: Vec<(String, String)>,
    legend_position: Corner,
    x_axis_position: VPlacement,
    y_axis_position: HPlacement,
    period: usize,
}

impl XYGraph {
    pub fn new(id: impl Hash) -> Self {
        let id = Id::new(id);
        Self {
            id,
            selector: Default::default(),
            keys: vec![],
            legend_position: Corner::LeftTop,
            x_axis_position: VPlacement::Bottom,
            y_axis_position: HPlacement::Left,
            period: 3600,
        }
    }

    pub fn show(&mut self, ctx: &Context, open: &mut bool, values: &Values) {
        egui::Window::new("XY Graph")
            .id(self.id)
            .default_size(vec2(400.0, 600.0))
            .vscroll(false)
            .open(open)
            .show(ctx, |ui| self.ui(ui, values));
    }

    pub fn ui(&mut self, ui: &mut Ui, values: &Values) {
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt(self.id.with("x_selector"))
                .selected_text(&self.selector.0)
                .show_ui(ui, |ui| {
                    for key in values.keys() {
                        ui.selectable_value(&mut self.selector.0, key.to_owned(), key);
                    }
                });
            egui::ComboBox::from_id_salt(self.id.with("y_selector"))
                .selected_text(&self.selector.1)
                .show_ui(ui, |ui| {
                    for key in values.keys() {
                        ui.selectable_value(&mut self.selector.1, key.to_owned(), key);
                    }
                });
            if ui.button("Add").clicked()
                && values.contains_key(&self.selector.0)
                && values.contains_key(&self.selector.1)
            {
                self.keys.push(std::mem::take(&mut self.selector));
            }
        });
        ui.separator();
        {
            let mut delete = None;
            for (index, keys) in self.keys.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{:5} {:5}", keys.0, keys.1));
                    if ui.button("Remove").clicked() {
                        delete = Some(index);
                    }
                });
            }
            if let Some(index) = delete {
                self.keys.remove(index);
            }
        }
        ui.separator();
        Plot::new(self.id.with("plot"))
            .legend(Legend::default().position(self.legend_position.into()))
            .x_axis_position(self.x_axis_position.into())
            .y_axis_position(self.y_axis_position.into())
            .y_axis_min_width(5.0)
            .show_axes(true)
            .show_grid(true)
            .data_aspect(1.0)
            .show(ui, |ui| {
                for (x_key, y_key) in &self.keys {
                    if let (Some(x_iter), Some(y_iter)) =
                        (values.iter_for_key(x_key), values.iter_for_key(y_key))
                    {
                        ui.line(
                            Line::new(PlotPoints::from_iter(
                                x_iter
                                    .rev()
                                    .zip(y_iter.rev())
                                    .take(self.period)
                                    .rev()
                                    .map(|(x, y)| [*x as f64, *y as f64]),
                            ))
                            .name(format!("{} {}", x_key, y_key)),
                        );
                    }
                }
            })
            .response
            .context_menu(|ui| {
                graph_context_menu(
                    ui,
                    &mut self.legend_position,
                    &mut self.x_axis_position,
                    &mut self.y_axis_position,
                    &mut self.period,
                )
            });
    }
}

fn graph_context_menu(
    ui: &mut Ui,
    legend_position: &mut Corner,
    x_axis_position: &mut VPlacement,
    y_axis_position: &mut HPlacement,
    period: &mut usize,
) {
    ui.menu_button("Legend", |ui| {
        let mut clicked = false;
        for (label, corner) in [
            ("Left Top", Corner::LeftTop),
            ("Left Bottom", Corner::LeftBottom),
            ("Right Top", Corner::RightTop),
            ("Right Bottom", Corner::RightBottom),
        ] {
            clicked |= ui.radio_value(legend_position, corner, label).clicked();
        }
        if clicked {
            ui.close_menu();
        }
    });
    ui.menu_button("X Axis", |ui| {
        let mut clicked = false;
        for (label, position) in [("Top", VPlacement::Top), ("Bottom", VPlacement::Bottom)] {
            clicked |= ui.radio_value(x_axis_position, position, label).clicked();
        }
        if clicked {
            ui.close_menu();
        }
    });
    ui.menu_button("Y Axis", |ui| {
        let mut clicked = false;
        for (label, position) in [("Left", HPlacement::Left), ("Right", HPlacement::Right)] {
            clicked |= ui.radio_value(y_axis_position, position, label).clicked();
        }
        if clicked {
            ui.close_menu();
        }
    });
    ui.menu_button("Period", |ui| {
        let mut clicked = false;
        for (label, p) in [
            ("10sec", 60 * 10),
            ("1min", 60 * 60),
            ("5min", 60 * 60 * 5),
            ("10min", 60 * 60 * 10),
            ("15min", 60 * 60 * 15),
            ("30min", 60 * 60 * 30),
        ] {
            clicked |= ui.radio_value(period, p, label).clicked();
        }
        if clicked {
            ui.close_menu();
        }
    });
}
