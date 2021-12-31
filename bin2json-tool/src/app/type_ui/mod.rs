use eframe::egui;
use eframe::egui::{Response, Ui};

use bin2json::{range_map, Type};
use bin2json::range::KeyRange;
use bin2json::secure::SecureKey;
use bin2json::ty::{Checksum, Endian, Field};
pub use bytes_size_ui::BytesSizeUi;
pub use converter_ui::ConverterUi;
pub use endian_ui::EndianUi;
pub use length_ui::LengthUi;
pub use raw_edit_ui::RawEditUi;
pub use size_ui::SizeUi;

mod size_ui;
mod endian_ui;
mod bytes_size_ui;
mod raw_edit_ui;
mod length_ui;
mod converter_ui;

#[derive(Clone)]
pub struct TypeUi {
    pub ty: Type,
    ident: String,
    ident_counter: usize,

    temp_bs_enum_key: String,
    temp_bs_enum_value: usize,
    temp_bs_error: String,

    temp_fields: Vec<(String, TypeUi)>,

    temp_enum_error: String,
}

impl TypeUi {
    pub fn new<S: Into<String>>(id: S) -> Self {
        let ident = id.into();
        Self {
            ty: Type::uint8(),
            temp_bs_enum_key: "".to_string(),
            temp_bs_enum_value: 0,
            temp_fields: vec![],
            temp_bs_error: "".to_string(),
            ident,
            temp_enum_error: "".to_string(),
            ident_counter: 0,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Response {
        let ty = &mut self.ty;
        egui::Grid::new(&self.ident)
            .spacing([5.0, 20.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("类型");
                egui::ComboBox::from_id_source(format!("{} > type combox", &self.ident))
                    .selected_text(ty.type_name())
                    .show_ui(ui, |ui| {
                        let old_ty = ty.type_name();
                        for t in default_types() {
                            let name = t.type_name();
                            ui.selectable_value(ty, t, name);
                        }
                        if ty.type_name() != old_ty {
                            self.temp_bs_enum_key = Default::default();
                            self.temp_bs_enum_value = 0;
                            self.temp_bs_error = Default::default();

                            self.temp_fields.clear();
                            self.ident_counter = 0;

                            self.temp_enum_error = Default::default();
                        }
                    });
                ui.end_row();

                match ty {
                    Type::Magic { magic } => {
                        ui.label("魔法值");
                        ui.add(RawEditUi::new(magic, false));
                    }
                    Type::Boolean { bit } => {
                        ui.label("大小");
                        ui.radio_value(bit, true, "1 比特位");
                        ui.radio_value(bit, false, "1 字节");
                        ui.end_row();
                    }

                    | Type::Int8 { unit }
                    | Type::Int16 { unit }
                    | Type::Int32 { unit }
                    | Type::Int64 { unit }
                    | Type::Uint8 { unit }
                    | Type::Uint16 { unit }
                    | Type::Uint32 { unit }
                    | Type::Uint64 { unit }
                    => {
                        ui.label("字节顺序");
                        ui.add(EndianUi(&mut unit.endian));
                        ui.end_row();

                        ui.label("总大小");
                        ui.add(SizeUi(&mut unit.size));
                        ui.end_row();
                    }

                    | Type::Float32 { endian }
                    | Type::Float64 { endian }
                    => {
                        ui.label("字节顺序");
                        ui.add(EndianUi(endian));
                        ui.end_row();
                    }

                    | Type::String { size }
                    | Type::Bin { size }
                    => {
                        ui.label("大小");
                        ui.add(BytesSizeUi::new(
                            size,
                            &mut self.temp_bs_enum_key,
                            &mut self.temp_bs_enum_value,
                            &mut self.temp_bs_error,
                        ));
                        ui.end_row();
                    }

                    Type::Struct { size, fields } => {
                        ui.label("大小");
                        ui.add(BytesSizeUi::new(
                            size,
                            &mut self.temp_bs_enum_key,
                            &mut self.temp_bs_enum_value,
                            &mut self.temp_bs_error,
                        ));
                        ui.end_row();

                        ui.horizontal(|ui| {
                            ui.label("字段列表");
                            if ui.button("+").on_hover_text("添加字段").clicked() {
                                self.temp_fields.push((
                                    Default::default(),
                                    TypeUi::new(format!("{} > fields[{}]", &self.ident, self.ident_counter)),
                                ));
                                self.ident_counter += 1;
                            }
                        });
                        egui::Grid::new("fields")
                            .show(ui, |ui| {
                                ui.label("字段名称");
                                ui.label("类型定义");
                                ui.label("操作");
                                ui.end_row();

                                let mut remove_list = vec![];
                                for (idx, (name, ty)) in self.temp_fields.iter_mut().enumerate() {
                                    ui.separator();
                                    ui.separator();
                                    ui.separator();
                                    ui.end_row();

                                    ui.text_edit_singleline(name);
                                    ty.ui(ui);
                                    if ui.button("删除").clicked() {
                                        remove_list.push(idx);
                                    };
                                    ui.end_row();
                                };
                                remove_list.reverse();
                                for idx in remove_list {
                                    self.temp_fields.remove(idx);
                                }

                                fields.clear();
                                fields.extend(self.temp_fields.iter()
                                    .map(|(name, ty)| {
                                        Field::new(name, ty.ty.clone())
                                    }));
                            });
                        ui.end_row();
                    }

                    Type::Array { size, length, element_type } => {
                        ui.label("大小");
                        ui.add(BytesSizeUi::new(size, &mut self.temp_bs_enum_key, &mut self.temp_bs_enum_value, &mut self.temp_bs_error));
                        ui.end_row();

                        ui.label("长度");
                        ui.add(LengthUi(length));
                        ui.end_row();

                        if self.temp_fields.is_empty() {
                            self.temp_fields.push((Default::default(), TypeUi::new(format!("{} > ArrayType", &self.ident))));
                        }
                        ui.label("成员类型");
                        ui.vertical(|ui| {
                            let ty_ui = &mut self.temp_fields[0].1;
                            ty_ui.ui(ui);
                            *element_type = Box::new(ty_ui.ty.clone());
                        });
                        ui.end_row();
                    }

                    Type::Enum { by, map, size } => {
                        ui.label("大小");
                        ui.add(BytesSizeUi::new(size, &mut self.temp_bs_enum_key, &mut self.temp_bs_enum_value, &mut self.temp_bs_error));
                        ui.end_row();

                        ui.label("引用字段名称");
                        ui.text_edit_singleline(by);
                        ui.end_row();

                        ui.label("枚举值");
                        ui.vertical(|ui| {
                            egui::Grid::new("Enum")
                                .show(ui, |ui| {
                                    ui.label("引用字段值");
                                    ui.label("类型");
                                    ui.label("操作");
                                    ui.end_row();

                                    if self.temp_fields.is_empty() {
                                        self.temp_fields.push((Default::default(), TypeUi::new(format!("{} > add enum", &self.ident))));
                                    }

                                    let last_idx = self.temp_fields.len() - 1;
                                    let mut remove_list = vec![];
                                    for (idx, (k, v)) in self.temp_fields
                                        .iter_mut()
                                        .take(last_idx)
                                        .enumerate()
                                    {
                                        for _ in 0..3 {
                                            ui.separator();
                                        }
                                        ui.end_row();

                                        ui.label(k.as_str());
                                        ui.horizontal_top(|ui| v.ui(ui));
                                        if ui.button("删除").clicked() {
                                            remove_list.push(idx);
                                        }
                                        ui.end_row();
                                    }
                                    remove_list.reverse();
                                    for idx in remove_list {
                                        self.temp_fields.remove(idx);
                                    }

                                    for _ in 0..3 {
                                        ui.separator();
                                    }
                                    ui.end_row();

                                    let last_idx = self.temp_fields.len() - 1;
                                    let (temp_kr, temp_ty) = &mut self.temp_fields[last_idx];

                                    let resp = ui.text_edit_singleline(temp_kr)
                                        .on_hover_text(KEY_RANGE_FORMAT);
                                    if self.temp_enum_error.len() > 0 {
                                        let pid = ui.make_persistent_id("enum error");
                                        ui.memory().open_popup(pid);
                                        egui::popup_below_widget(ui, pid, &resp, |ui| {
                                            if ui.add(
                                                egui::Label::new(egui::RichText::new(self.temp_enum_error.as_str()).color(egui::Color32::RED))
                                                    .wrap(false)
                                                    .sense(egui::Sense::click())
                                            ).clicked() {
                                                self.temp_enum_error = Default::default();
                                            }
                                        });
                                    }

                                    ui.horizontal_top(|ui| temp_ty.ui(ui));
                                    if ui.button("添加/修改").clicked() {
                                        if temp_kr.parse::<KeyRange>().is_ok() {
                                            let kr = temp_kr.clone();
                                            temp_ty.ident = format!("{} > Enum[{}]", &self.ident, self.ident_counter);
                                            self.ident_counter += 1;
                                            self.temp_fields.push((
                                                Default::default(),
                                                TypeUi::new(format!("{} > add enum", &self.ident)),
                                            ));
                                            self.temp_enum_error = Default::default();

                                            self.temp_fields.iter()
                                                .position(|(k, _)| k == &kr)
                                                .map(|idx| {
                                                    if idx != last_idx {
                                                        self.temp_fields.remove(idx);
                                                    }
                                                });
                                        } else {
                                            self.temp_enum_error = format!("输入格式错误\n{}", KEY_RANGE_FORMAT);
                                        }
                                    }

                                    map.clear();
                                    for (k, ty) in &self.temp_fields[..self.temp_fields.len() - 1] {
                                        let kr: KeyRange = k.parse().unwrap();
                                        map.insert(kr, ty.ty.clone());
                                    }
                                });
                        });
                        ui.end_row();
                    }

                    Type::Converter {
                        original_type,
                        on_read,
                        on_write,
                    } => {
                        ui.label("读取转换方式");
                        ui.add(ConverterUi(on_read));
                        ui.end_row();

                        ui.label("写入转换方式");
                        ui.add(ConverterUi(on_write));
                        ui.end_row();

                        if self.temp_fields.is_empty() {
                            self.temp_fields.push((
                                Default::default(),
                                TypeUi::new(format!("{} > Converter", &self.ident))
                            ));
                        }
                        ui.label("原始类型");
                        ui.horizontal_top(|ui| self.temp_fields[0].1.ui(ui));
                        ui.end_row();
                        *original_type = Box::new(self.temp_fields[0].1.ty.clone());
                    }

                    Type::Checksum {
                        method,
                        start_key,
                        end_key,
                    } => {
                        ui.label("计算方法");
                        egui::ComboBox::from_id_source(format!("{} > CheckSum ComboBox", &self.ident))
                            .selected_text("异或")
                            .show_ui(ui, |ui| {
                                ui.selectable_value(method, Checksum::Xor, "异或");
                            });
                        ui.end_row();

                        ui.label("开始字段");
                        ui.text_edit_singleline(start_key);
                        ui.end_row();

                        ui.label("停止字段").on_hover_text("未设置则为该类型对应的字段");
                        ui.horizontal_top(|ui| {
                            let mut checked = end_key.is_some();
                            ui.checkbox(&mut checked, "");
                            if checked != end_key.is_some() {
                                *end_key = if checked { Some(Default::default()) } else { None };
                            }
                            if let Some(s) = end_key {
                                ui.text_edit_singleline(s);
                            }
                        });
                        ui.end_row();
                    }

                    Type::Encrypt { .. } => {}
                    Type::Sign { .. } => {}
                }
            })
            .response
    }
}


fn default_types() -> Vec<Type> {
    vec![
        Type::magic(b""),
        Type::BOOL,
        Type::int8(),
        Type::int16(Endian::Big),
        Type::int32(Endian::Big),
        Type::int64(Endian::Big),
        Type::uint8(),
        Type::uint16(Endian::Big),
        Type::uint32(Endian::Big),
        Type::uint64(Endian::Big),
        Type::float32(Endian::Big),
        Type::float64(Endian::Big),
        Type::String { size: None },
        Type::Bin { size: None },
        Type::new_struct(vec![]),
        Type::new_array(Type::uint8()),
        Type::new_enum("", range_map! {}),
        Type::converter(Type::uint8(), "self", "self"),
        Type::checksum(Checksum::Xor, ""),
        Type::encrypt(Type::uint8(), SecureKey::None, SecureKey::None),
        Type::sign("", SecureKey::None, SecureKey::None),
    ]
}

pub const KEY_RANGE_FORMAT: &'static str = r#"接收以下格式数据：
- num: 指定值
- num..: 大于或等于指定值
- num1..num2: num1到num2之间的值（不包括num2)
- num1..=num2: num1到num2之间的值（包括num2)
- ..num: 小于指定值
- ..=num: 小于等于指定值
- ..: 所有值
注意: 输入的数值必须是整数"#;
