use crate::values::{NitsRelativeCarCount, Values};
use egui::{vec2, Context, Id, Layout, RichText, Ui};
use egui_extras::{Column, TableBuilder, TableRow};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

enum TimelineRow {
    Command(NitsRelativeCarCount, u32),
    CommonLine(u32),
    Separator,
}

impl TimelineRow {
    fn get_height(&self) -> f32 {
        match self {
            TimelineRow::Command(_, _) => 20.0,
            TimelineRow::CommonLine(_) => 20.0,
            TimelineRow::Separator => 4.0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct NitsTimelineWindow {
    id: Id,
}

impl NitsTimelineWindow {
    pub fn new(id: impl Hash) -> Self {
        Self { id: Id::new(id) }
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
        let nits_timeline = &values.nits_timeline;
        let mut timeline_rows: Vec<TimelineRow> = Vec::new();
        for nits_tick in nits_timeline {
            timeline_rows.extend(
                nits_tick
                    .commands
                    .iter()
                    .map(|(sender, value)| TimelineRow::Command(sender.clone(), *value)),
            );
            timeline_rows.push(TimelineRow::CommonLine(nits_tick.commonline));
            timeline_rows.push(TimelineRow::Separator);
        }
        timeline_rows.pop();

        ui.style_mut().spacing.item_spacing = vec2(0.0, 2.0);
        TableBuilder::new(ui)
            .cell_layout(Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(100.0))
            .column(Column::auto().at_least(30.0))
            .columns(Column::exact(20.0), 24)
            .stick_to_bottom(true)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Sender");
                });
                header.col(|_| {});
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
                    timeline_rows.iter().map(|r| { r.get_height() }),
                    |row| {
                        let index = row.index();
                        let timeline_row = &timeline_rows[index];

                        match timeline_row {
                            TimelineRow::Command(sender, value) => {
                                self.command_row(row, &sender.to_string(), *value);
                            }
                            TimelineRow::CommonLine(value) => {
                                self.command_row(row, "Common Line", *value);
                            }
                            TimelineRow::Separator => {
                                self.separator_row(row);
                            }
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

    fn command_row(&self, mut row: TableRow<'_, '_>, sender_label: &str, value: u32) {
        row.col(|ui| {
            ui.label(sender_label);
        });
        let command_type = value >> 24 & 0xFF;
        let command_payload = value & 0xFFFFFF;
        row.col(|ui| {
            ui.label(format!("0x{:02x}", command_type));
        });
        for i in (0..24).rev() {
            row.col(|ui| {
                let bit = command_payload >> i & 1;
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
}
