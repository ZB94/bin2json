use eframe::egui;
use eframe::egui::{Color32, Response, RichText, Sense, Ui, Widget};

use bin2json::range::KeyRange;
use bin2json::range_map;
use bin2json::ty::BytesSize;

use crate::app::type_ui::{KEY_RANGE_FORMAT, RawEditUi};

pub struct BytesSizeUi<'a> {
    pub bs: &'a mut Option<BytesSize>,
    pub temp_kr: &'a mut String,
    pub temp_v: &'a mut usize,
    pub error: &'a mut String,
}

impl<'a> BytesSizeUi<'a> {
    pub fn new(
        bs: &'a mut Option<BytesSize>,
        temp_kr: &'a mut String,
        temp_v: &'a mut usize,
        error: &'a mut String,
    ) -> Self {
        Self {
            bs,
            temp_kr,
            temp_v,
            error,
        }
    }
}

impl Widget for BytesSizeUi<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let bs = self.bs;
        let temp_kr = self.temp_kr;
        let temp_v = self.temp_v;
        let error = self.error;

        ui.vertical(|ui| {
            if ui.radio(bs.is_none(), "无限制").clicked() {
                *bs = None;
            }

            ui.horizontal(|ui| {
                if let Some(BytesSize::Fixed(size)) = bs {
                    let _ = ui.radio(true, "固定大小");
                    ui.add(egui::DragValue::new(size).suffix("字节"));
                } else {
                    if ui.radio(false, "固定大小").clicked() {
                        *bs = Some(BytesSize::new(0));
                    }
                };
            });

            ui.horizontal(|ui| {
                if let Some(BytesSize::EndWith(bytes)) = bs {
                    let _ = ui.radio(true, "以指定字节结尾");
                    ui.add(RawEditUi::new(bytes, false));
                } else {
                    if ui.radio(false, "以指定字节结尾").clicked() {
                        *bs = Some(BytesSize::new(vec![]));
                    }
                };
            });

            ui.horizontal(|ui| {
                if let Some(BytesSize::By(by)) = bs {
                    let _ = ui.radio(true, "指定字段的值  字段名称");
                    ui.text_edit_singleline(by);
                } else {
                    if ui.radio(false, "指定字段的值").clicked() {
                        *bs = Some(BytesSize::new(""));
                    }
                };
            });

            ui.vertical(|ui| {
                if let Some(BytesSize::Enum { by, map }) = bs {
                    let _ = ui.radio(true, "以指定字段的枚举值");

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("引用字段名称");
                        ui.text_edit_singleline(by);
                    });

                    ui.separator();
                    egui::Grid::new("bytes size enum")
                        .show(ui, |ui| {
                            ui.label("引用字段值");
                            ui.label("大小");
                            ui.label("操作");
                            ui.end_row();

                            map.retain(|k, v| {
                                ui.label(k.to_string());
                                ui.add(egui::DragValue::new(v).suffix("字节"));
                                let r = !ui.button("删除").clicked();
                                ui.end_row();
                                r
                            });

                            let resp = ui.text_edit_singleline(temp_kr)
                                .on_hover_text(KEY_RANGE_FORMAT);
                            if error.len() > 0 {
                                let pid = ui.make_persistent_id("error");
                                ui.memory().open_popup(pid);
                                egui::popup_below_widget(ui, pid, &resp, |ui| {
                                    if ui.add(
                                        egui::Label::new(RichText::new(error.as_str()).color(Color32::RED))
                                            .wrap(false)
                                            .sense(Sense::click())
                                    ).clicked() {
                                        *error = Default::default();
                                    }
                                });
                            }

                            ui.add(egui::DragValue::new(temp_v).suffix("字节"));
                            if ui.button("添加/修改").clicked() {
                                match temp_kr.parse::<KeyRange>() {
                                    Ok(kr) => {
                                        map.insert(kr, *temp_v);
                                        *temp_kr = Default::default();
                                        *temp_v = 0;
                                        *error = Default::default();
                                    }
                                    Err(_) => {
                                        *error = format!("输入格式错误\n{}", KEY_RANGE_FORMAT);
                                    }
                                }
                            }
                        });
                } else {
                    if ui.radio(false, "以指定字段的枚举值").clicked() {
                        *bs = Some(BytesSize::Enum {
                            by: "".to_string(),
                            map: range_map! {},
                        });
                    }
                };
            })
        })
            .response
    }
}
