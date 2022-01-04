use eframe::egui::{Response, Ui, Widget};

use bin2json::ty::Endian;

pub struct EndianUi<'a>(pub &'a mut Endian);

impl Widget for EndianUi<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let endian = self.0;
        ui.horizontal(|ui| {
            ui.radio_value(endian, Endian::Big, "大端");
            ui.radio_value(endian, Endian::Little, "小端");
        })
            .response
    }
}
