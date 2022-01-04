use eframe::egui;
use eframe::egui::{Response, Ui, Widget};

use bin2json::ty::Length;

pub struct LengthUi<'a>(pub &'a mut Option<Length>);


impl Widget for LengthUi<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let len = self.0;

        ui.vertical(|ui| {
            if ui.radio(len.is_none(), "无限制").clicked() {
                *len = None;
            }

            ui.horizontal(|ui| {
                if let Some(Length::Fixed(size)) = len {
                    let _ = ui.radio(true, "固定大小");
                    ui.add(egui::DragValue::new(size).suffix("字节"));
                } else {
                    if ui.radio(false, "固定大小").clicked() {
                        *len = Some(Length::Fixed(0));
                    }
                };
            });

            ui.horizontal(|ui| {
                if let Some(Length::By(by)) = len {
                    let _ = ui.radio(true, "指定字段的值  字段名称");
                    ui.text_edit_singleline(by);
                } else {
                    if ui.radio(false, "指定字段的值").clicked() {
                        *len = Some(Length::by_field(""));
                    }
                };
            });
        })
            .response
    }
}
