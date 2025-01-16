use crate::{
    digital_table::DigitalTableWindow,
    graph::{LineGraph, XYGraph},
    nits_timeline::NitsTimelineWindow,
    table::TableWindow,
    values::Values,
};
use egui::{ahash::HashMap, Context};
use egui_file::FileDialog;
use ewebsock::{WsMessage, WsReceiver, WsSender};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Window {
    LineGraph(Box<LineGraph>),
    XYGraph(Box<XYGraph>),
    Table(Box<TableWindow>),
    DigitalTable(Box<DigitalTableWindow>),
    NitsTimeline(Box<NitsTimelineWindow>),
}

impl Window {
    fn show(&mut self, ctx: &Context, open: &mut bool, values: &Values) {
        match self {
            Window::LineGraph(w) => w.show(ctx, open, values),
            Window::XYGraph(w) => w.show(ctx, open, values),
            Window::Table(w) => w.show(ctx, open, values),
            Window::DigitalTable(w) => w.show(ctx, open, values),
            Window::NitsTimeline(w) => w.show(ctx, open, values),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct App {
    id: usize,
    server: String,
    #[serde(skip, default)]
    ws: Option<(WsSender, WsReceiver)>,
    values: Values,
    windows: Vec<(Window, bool)>,
    #[serde(skip, default)]
    open_dialog: Option<FileDialog>,
    #[serde(skip, default)]
    save_dialog: Option<FileDialog>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        if let Some(storage) = cc.storage {
            if let Some(app) = eframe::get_value(storage, eframe::APP_KEY) {
                return app;
            }
        }
        #[cfg(target_arch = "wasm32")]
        let server = {
            let location = &cc.integration_info.web_info.location;
            format!("ws://{}/socket", location.host)
        };
        #[cfg(not(target_arch = "wasm32"))]
        let server = "ws://127.0.0.1:8080/socket".into();
        Self {
            id: 0,
            server,
            ws: None,
            values: Default::default(),
            windows: vec![],
            open_dialog: None,
            save_dialog: None,
        }
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self);
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if let Some((_, rx)) = self.ws.as_ref() {
            while let Some(e) = rx.try_recv() {
                match e {
                    ewebsock::WsEvent::Opened => {}
                    ewebsock::WsEvent::Message(WsMessage::Text(m)) => {
                        match serde_json::from_str::<HashMap<String, Vec<f32>>>(&m) {
                            Ok(v) => {
                                self.values.add_data(v);
                            }
                            Err(e) => {
                                log::error!("failed to parse: {}", e);
                            }
                        }
                    }
                    ewebsock::WsEvent::Message(_) => {}
                    ewebsock::WsEvent::Error(e) => log::error!("{}", e),
                    ewebsock::WsEvent::Closed => {
                        let ctx = ctx.clone();
                        let wakeup = move || ctx.request_repaint();
                        self.ws =
                            ewebsock::connect_with_wakeup(&self.server, Default::default(), wakeup)
                                .map_err(|e| log::error!("failed to init websocket {}", e))
                                .ok();
                        break;
                    }
                }
            }
        }
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_theme_preference_switch(ui);
                ui.separator();
                ui.menu_button("File", |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if ui.button("Open CSV").clicked() {
                            let mut fd = FileDialog::open_file(None).title("Open CSV");
                            fd.open();
                            self.open_dialog = Some(fd);
                        }
                        if ui.button("Save as CSV").clicked() {
                            let mut fd = FileDialog::save_file(None)
                                .default_filename("all.csv")
                                .title("Save as CSV");
                            fd.open();
                            self.save_dialog = Some(fd);
                        }
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
                ui.menu_button("Settings", |ui| {
                    ui.menu_button("Retention period", |ui| {
                        for (label, len) in [
                            ("10sec", 60 * 10),
                            ("1min", 60 * 60),
                            ("5min", 60 * 60 * 5),
                            ("10min", 60 * 60 * 10),
                            ("15min", 60 * 60 * 15),
                            ("30min", 60 * 60 * 30),
                        ] {
                            if ui.radio(self.values.max_len() == len, label).clicked() {
                                self.values.set_max_len(len);
                                ui.close_menu();
                            }
                        }
                    });
                });
                egui::widgets::reset_button(ui, &mut self.values, "Reset");
                ui.separator();
                if ui.button("XY Graph").clicked() {
                    self.windows.push((
                        Window::XYGraph(Box::new(XYGraph::new(format!("xy_graph_{}", self.id)))),
                        true,
                    ));
                    self.id += 1;
                }
                if ui.button("Digital Table").clicked() {
                    self.windows.push((
                        Window::DigitalTable(Box::new(DigitalTableWindow::new(format!(
                            "digital_table_{}",
                            self.id
                        )))),
                        true,
                    ));
                    self.id += 1;
                }
                if ui.button("NITS Timeline").clicked() {
                    self.windows.push((
                        Window::NitsTimeline(Box::new(NitsTimelineWindow::new(format!(
                            "nits_timeline_{}",
                            self.id
                        )))),
                        true,
                    ));
                    self.id += 1;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.server);
                if self.ws.is_none() {
                    if ui.button("connect").clicked() {
                        let ctx = ctx.clone();
                        let wakeup = move || ctx.request_repaint();
                        self.ws =
                            ewebsock::connect_with_wakeup(&self.server, Default::default(), wakeup)
                                .map_err(|e| log::error!("failed to init websocket {}", e))
                                .ok();
                    }
                } else if ui.button("disconnect").clicked() {
                    self.ws = None;
                }
            });
            ui.separator();
            self.table(ui);
        });

        for graph in &mut self.windows {
            graph.0.show(ctx, &mut graph.1, &self.values);
        }
        self.windows.retain(|g| g.1);

        if let Some(open_dialog) = self.open_dialog.as_mut() {
            if open_dialog.show(ctx).selected() {
                if let Some(path) = open_dialog.path() {
                    let _ = self.values.load_csv(path);
                }
                self.open_dialog = None;
            }
        }

        if let Some(save_dialog) = self.save_dialog.as_mut() {
            if save_dialog.show(ctx).selected() {
                if let Some(path) = save_dialog.path() {
                    let _ = self.values.save_csv(path, self.values.keys());
                }
                self.save_dialog = None;
            }
        }
    }
}

impl App {
    fn table(&mut self, ui: &mut egui::Ui) {
        let mut keys: Vec<_> = self.values.keys().collect();
        keys.sort();
        use egui_extras::{Column, TableBuilder};
        let table = TableBuilder::new(ui)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::exact(256.0))
            .column(Column::auto());

        table
            .header(20.0, |mut header| {
                header.col(|_| {});
                header.col(|ui| {
                    ui.strong("Key");
                });
                header.col(|ui| {
                    ui.strong("Last Value");
                });
            })
            .body(|body| {
                body.rows(20.0, keys.len(), |mut row| {
                    let index = row.index();
                    let key = keys[index];
                    row.col(|ui| {
                        if ui.button("G").clicked() {
                            self.windows.push((
                                Window::LineGraph(Box::new(LineGraph::new(
                                    self.id,
                                    key.to_owned(),
                                ))),
                                true,
                            ));
                            self.id += 1;
                        }
                        if ui.button("T").clicked() {
                            self.windows.push((
                                Window::Table(Box::new(TableWindow::new(self.id, key.to_owned()))),
                                true,
                            ));
                            self.id += 1;
                        }
                    });
                    row.col(|ui| {
                        ui.label(key);
                    });
                    row.col(|ui| {
                        if let Some(v) = self.values.get_last_value_for_key(key) {
                            ui.label(v.to_string());
                        }
                    });
                });
            });
    }
}
