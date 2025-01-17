use crate::values::{NitsRelativeCarCount, NitsCommand, Values};
use egui::{vec2, Context, Id, Layout, RichText, Ui};
use egui_extras::{Column, TableBuilder, TableRow};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, hash::Hash};

#[derive(PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
enum NitsSender {
    Command(NitsRelativeCarCount),
    CommonLine,
}

impl std::fmt::Display for NitsSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command(sender) => write!(f, "{}", sender.to_string()),
            Self::CommonLine => write!(f, "Common Line"),
        }
    }
}

enum TimelineRow {
    Command(NitsSender, NitsCommand),
    Blank(usize),
    Separator,
}

impl TimelineRow {
    fn get_height(&self) -> f32 {
        match self {
            TimelineRow::Command(_, _) => 20.0,
            TimelineRow::Blank(_) => 20.0,
            TimelineRow::Separator => 4.0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct NitsTimelineWindow {
    id: Id,
    sender_filter: BTreeMap<NitsSender, bool>,
    command_type_filter: BTreeMap<u8, bool>,
}

impl NitsTimelineWindow {
    pub fn new(id: impl Hash) -> Self {
        Self {
            id: Id::new(id),
            sender_filter: BTreeMap::new(),
            command_type_filter: BTreeMap::new(),
        }
    }

    pub fn show(&mut self, ctx: &Context, open: &mut bool, values: &Values) {
        egui::Window::new("NITS Timeline")
            .id(self.id)
            .default_size(vec2(100.0, 200.0))
            .vscroll(true)
            .open(open)
            .show(ctx, |ui| self.ui(ui, values));
    }
    pub fn ui(&mut self, ui: &mut Ui, values: &Values) {
        let timeline_rows = self.get_timeline_rows(values);

        ui.style_mut().spacing.item_spacing = vec2(0.0, 2.0);
        TableBuilder::new(ui)
            .cell_layout(Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(100.0))
            .column(Column::auto().at_least(30.0))
            .columns(Column::exact(20.0), 24)
            .stick_to_bottom(true)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.style_mut().spacing.item_spacing = vec2(4.0, 0.0);
                    ui.strong("Sender");
                    ui.menu_button("⏷", |ui| {
                        for sender in values.get_nits_senders()
                            .iter()
                            .map(|s| NitsSender::Command(*s))
                            .chain([NitsSender::CommonLine]) {
                            let mut checked = true;
                            if let Some(c) = self.sender_filter.get(&sender) {
                                checked = *c;
                            }
                            ui.checkbox(&mut checked, sender.to_string());
                            self.sender_filter.insert(sender, checked);
                        }
                    });
                });
                header.col(|ui| {
                    if values.get_nits_command_types().len() > 0 {
                        ui.menu_button("⏷", |ui| {
                            for command_type in values.get_nits_command_types() {
                                let mut checked = true;
                                if let Some(c) = self.command_type_filter.get(command_type) {
                                    checked = *c;
                                }
                                ui.checkbox(&mut checked, format!("0x{:02x}", command_type));
                                self.command_type_filter.insert(*command_type, checked);
                            }
                        });
                    }
                });
                for i in 0..24 {
                    header.col(|ui| {
                        ui.centered_and_justified(|ui| {
                            ui.strong(RichText::new((23 - i).to_string()).size(10.0));
                        });
                    });
                }
            })
            .body(|body| {
                body.heterogeneous_rows(
                    timeline_rows.iter().map(|r| r.get_height()),
                    |row| {
                        let index = row.index();
                        let timeline_row = &timeline_rows[index];

                        match timeline_row {
                            TimelineRow::Command(sender, value) => {
                                self.command_row(row, &sender.to_string(), value);
                            },
                            TimelineRow::Blank(blank_count) => {
                                self.blank_row(row, *blank_count);
                            }
                            TimelineRow::Separator => {
                                self.separator_row(row);
                            },
                        }
                    }
                );
            });
    }

    fn separator_row(&self, mut row: TableRow<'_, '_>) {
        for _ in 0..26 {
            row.col(|ui| {
                ui.add(egui::Separator::default().horizontal());
            });
        }
    }

    fn blank_row(&self, mut row: TableRow<'_, '_>, blank_count: usize) {
        row.col(|ui| {
            ui.label(format!("{} Blank Ticks", blank_count));
        });
    }

    fn command_row(&self, mut row: TableRow<'_, '_>, sender_label: &str, command: &NitsCommand) {
        row.col(|ui| {
            ui.label(sender_label);
        });
        row.col(|ui| {
            ui.label(format!("0x{:02x}", command.get_command_type()));
        });
        for i in (0..24).rev() {
            row.col(|ui| {
                let bit = command.get_payload() >> i & 1;
                if bit != 0 {
                    ui.painter().rect_filled(
                        ui.available_rect_before_wrap(),
                        1.0,
                        ui.visuals().gray_out(ui.visuals().weak_text_color()),
                    );
                }
                ui.centered_and_justified(|ui| {
                    ui.label(format!("{:01b}", bit));
                });
            });
        }
    }

    fn get_timeline_rows(&self, values: &Values) -> Vec<TimelineRow> {
        let commonline_pass_sender_filter = *self.sender_filter.get(&NitsSender::CommonLine).unwrap_or(&true);

        let len = values.get_nits_timeline().len();
        let mut timeline_rows: Vec<TimelineRow> = Vec::new();
        let mut blank_count = 0;
        for (t, nits_tick) in values.get_nits_timeline().iter().enumerate() {
            let is_last = t + 1 >= len;
            let mut rows_tmp: Vec<TimelineRow> = Vec::new();

            for (c, value) in nits_tick.get_commands() {
                let sender = NitsSender::Command(*c);
                let pass_sender_filter = *self.sender_filter
                    .get(&sender)
                    .unwrap_or(&true);
                let pass_command_type_filter = *self.command_type_filter
                    .get(&value.get_command_type())
                    .unwrap_or(&true);
                if pass_sender_filter && pass_command_type_filter {
                    rows_tmp.push(TimelineRow::Command(sender, *value));
                }
            }

            let commonline_pass_command_type_filter = *self.command_type_filter
                .get(&nits_tick.get_commonline().get_command_type())
                .unwrap_or(&true);
            if commonline_pass_sender_filter && commonline_pass_command_type_filter {
                rows_tmp.push(TimelineRow::Command(NitsSender::CommonLine, *nits_tick.get_commonline()));
            }

            if blank_count > 0 {
                if rows_tmp.len() > 0 {
                    timeline_rows.push(TimelineRow::Blank(blank_count));
                    timeline_rows.push(TimelineRow::Separator);
                    blank_count = 0;
                } else if is_last {
                    timeline_rows.push(TimelineRow::Blank(blank_count + 1));
                    blank_count = 0;
                }
            }

            if rows_tmp.len() > 0 {
                timeline_rows.append(&mut rows_tmp);
                if !is_last {
                    timeline_rows.push(TimelineRow::Separator);
                }
            } else {
                blank_count += 1;
            }
        }

        return timeline_rows;
    }
}
