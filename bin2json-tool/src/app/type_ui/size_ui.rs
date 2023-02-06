use bin2json::ty::BitSize;
use eframe::egui;
use eframe::egui::{Response, Ui, Widget};

pub struct SizeUi<'a>(pub &'a mut Option<BitSize>);

impl Widget for SizeUi<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = self.0;

        ui.vertical(|ui| {
            if ui.radio(size.is_none(), "按类型大小").clicked() {
                *size = None;
            }

            ui.horizontal(|ui| {
                if let Some(BitSize(size)) = size {
                    let _ = ui.radio(true, "按指定比特数");
                    ui.add(egui::DragValue::new(size).suffix("比特"));
                } else {
                    if ui.radio(false, "按指定比特数").clicked() {
                        *size = Some(BitSize(0));
                    }
                };
            });
        })
        .response
    }
}
