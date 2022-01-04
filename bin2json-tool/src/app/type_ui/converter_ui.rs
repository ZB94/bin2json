use eframe::egui::{Response, Ui, Widget};

use bin2json::ty::Converter;

pub struct ConverterUi<'a>(pub &'a mut Converter);

const URL: &'static str = "https://docs.rs/evalexpr/latest/evalexpr/";
const TOOLTIP: &'static str = r#"执行表达式时有以下变量：
- 对于Struct: self.field_name，其中field_name为字段列表中的字段名称
- 对于Array: self[idx]，其中idx为数组成员的下标
- 对于其他类型: self"#;

impl Widget for ConverterUi<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            for (label, value) in [
                ("转换前验证", &mut self.0.before_valid),
                ("转换", &mut self.0.convert),
                ("转换后验证", &mut self.0.after_valid),
            ] {
                ui.horizontal(|ui| {
                    let mut checked = value.is_some();
                    ui.checkbox(&mut checked, label)
                        .on_hover_ui(|ui| {
                            ui.label("表达式说明地址: ");
                            ui.add(eframe::egui::Hyperlink::new(URL));
                            ui.label(TOOLTIP);
                        });
                    if checked != value.is_some() {
                        *value = if checked { Some(Default::default()) } else { None };
                    }
                    if let Some(expr) = value {
                        ui.text_edit_singleline(expr);
                    }
                });
            }
        }).response
    }
}
