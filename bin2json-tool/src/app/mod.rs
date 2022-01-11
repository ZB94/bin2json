use eframe::egui;
use eframe::egui::{Align, Color32, CtxRef, FontData, FontDefinitions, Label, Layout, RichText, Sense, TextEdit, Ui};
use eframe::epi::{App, Frame, Storage};

use bin2json::bitvec::BitView;

use crate::app::type_ui::{RawEditUi, TypeUi};

mod type_ui;

pub struct Application {
    ty: TypeUi,
    bin: Vec<u8>,
    text: String,
    ty_json: String,
    from_ty_json_error: String,
    b2j_error: String,
    b2j: bool,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            ty: TypeUi::new("application"),
            bin: vec![],
            text: "".to_string(),
            ty_json: "".to_string(),
            from_ty_json_error: "".to_string(),
            b2j_error: "".to_string(),
            b2j: false,
        }
    }
}

impl App for Application {
    fn update(&mut self, ctx: &CtxRef, _frame: &Frame) {
        egui::SidePanel::left("type def")
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("类型配置");
                ui.separator();
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.ty.ui(ui);
                    });
            });

        egui::CentralPanel::default()
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let width = ui.available_width();

                        self.ui_ty_json(ui, width);

                        ui.group(|ui| {
                            ui.label("数据");
                            let resp_bin = ui.add(RawEditUi::new(&mut self.bin, true)
                                .desired_width(width));
                            ui.separator();

                            ui.label("JSON");
                            let resp_text = ui.add(TextEdit::multiline(&mut self.text)
                                .desired_width(width));
                            ui.separator();

                            ui.with_layout(Layout::right_to_left().with_cross_align(Align::Min), |ui| {
                                if ui.button("数据转JSON").clicked() {
                                    match self.ty.ty.read(self.bin.view_bits()) {
                                        Ok((j, _)) => {
                                            self.text = bin2json::serde_json::to_string_pretty(&j).unwrap();
                                            self.b2j_error.clear();
                                        }
                                        Err(e) => {
                                            self.b2j_error = e.to_string();
                                            self.b2j = true;
                                        }
                                    }
                                }

                                if ui.button("JSON转数据").clicked() {
                                    let r = bin2json::serde_json::from_str(&self.text)
                                        .map_err(|e| format!("输入JSON无效: {}", e))
                                        .and_then(|j| {
                                            self.ty.ty.write(&j)
                                                .map_err(|e| e.to_string())
                                        });

                                    match r {
                                        Ok(d) => {
                                            self.bin = d.into_vec();
                                            self.b2j_error.clear();
                                        }
                                        Err(e) => {
                                            self.b2j_error = e;
                                            self.b2j = false;
                                        }
                                    }
                                }
                            });

                            if !self.b2j_error.is_empty() {
                                let resp = if self.b2j { &resp_bin } else { &resp_text };
                                let pid = ui.make_persistent_id("b2j error");
                                ctx.memory().open_popup(pid);
                                egui::popup_below_widget(ui, pid, resp, |ui| {
                                    let label = Label::new(
                                        RichText::new(&self.b2j_error)
                                            .color(Color32::RED)
                                    )
                                        .sense(Sense::click());
                                    if ui.add(label).clicked() {
                                        self.b2j_error.clear();
                                    }
                                });
                            }
                        });
                    });
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

impl Application {
    fn ui_ty_json(&mut self, ui: &mut Ui, width: f32) {
        ui.group(|ui| {
            ui.label("类型配置(JSON)");
            ui.separator();
            let resp = ui.add(TextEdit::multiline(&mut self.ty_json)
                .desired_width(width));
            ui.separator();
            ui.with_layout(Layout::right_to_left().with_cross_align(Align::Min), |ui| {
                if ui.button("类型配置=>JSON").clicked() {
                    self.ty_json = bin2json::serde_json::to_string_pretty(&self.ty.ty).unwrap();
                }
                if ui.button("JSON=>类型配置").clicked() {
                    match bin2json::serde_json::from_str(&self.ty_json) {
                        Ok(ty) => {
                            self.ty = TypeUi::from_type(&self.ty.ident, ty);
                            self.from_ty_json_error.clear();
                        }
                        Err(e) => self.from_ty_json_error = format!("数据格式错误: {}", e),
                    };
                }

                if !self.from_ty_json_error.is_empty() {
                    let pid = ui.make_persistent_id("from ty json error");
                    ui.memory().open_popup(pid);
                    egui::popup_below_widget(ui, pid, &resp, |ui| {
                        let label = Label::new(
                            RichText::new(&self.from_ty_json_error)
                                .color(Color32::RED)
                        )
                            .sense(Sense::click());
                        if ui.add(label).clicked() {
                            self.from_ty_json_error.clear();
                        }
                    });
                }
            });
        });
    }
}
