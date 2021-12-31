use eframe::egui;
use eframe::egui::{CtxRef, FontData, FontDefinitions};
use eframe::epi::{App, Frame, Storage};

use crate::app::type_ui::TypeUi;

mod type_ui;

pub struct Application {
    ty: TypeUi,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            ty: TypeUi::new(),
        }
    }
}

impl App for Application {
    fn update(&mut self, ctx: &CtxRef, _frame: &Frame) {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                self.ty.ui(ui);
            });
    }

    fn setup(&mut self, ctx: &CtxRef, _frame: &Frame, _storage: Option<&dyn Storage>) {
        let mut fonts = FontDefinitions::default();

        let font_data = FontData::from_owned(std::fs::read("C:/Windows/Fonts/msyh.ttc").unwrap());
        fonts.font_data.insert(format!("msyh"), font_data);

        for (_, list) in &mut fonts.fonts_for_family {
            list.insert(0, format!("msyh"));
        }

        for (_, (_, s)) in &mut fonts.family_and_size {
            *s *= 1.4;
        }

        ctx.set_fonts(fonts);
    }


    fn name(&self) -> &str {
        "bin2json tool"
    }
}
