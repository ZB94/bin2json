use eframe::egui;
use eframe::egui::{Response, Ui};

use bin2json::{range_map, Type};
use bin2json::range::{KeyRange, KeyRangeMap};
use bin2json::secure::SecureKey;
use bin2json::ty::{BytesSize, Checksum, Endian, Field};
pub use bytes_size_ui::BytesSizeUi;
pub use converter_ui::ConverterUi;
pub use endian_ui::EndianUi;
pub use length_ui::LengthUi;
pub use raw_edit_ui::RawEditUi;
pub use secure_key_ui::SecureKeyUi;
pub use size_ui::SizeUi;

mod size_ui;
mod endian_ui;
mod bytes_size_ui;
mod raw_edit_ui;
mod length_ui;
mod converter_ui;
mod secure_key_ui;

#[derive(Clone)]
pub struct TypeUi {
    pub ty: Type,
    pub(crate) ident: String,
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

    fn reset_state(&mut self) {
        self.temp_bs_enum_key.clear();
        self.temp_bs_enum_value = 0;
        self.temp_bs_error.clear();

        self.temp_fields.clear();
        self.ident_counter = 0;

        self.temp_enum_error.clear();

        if let
        | Type::Array { .. }
        | Type::Converter { .. }
        | Type::Encrypt { .. }
        | Type::Enum { .. }
        = self.ty {
            self.add_temp_field("", None);
        }
    }

    fn set_type(&mut self, ty: Type) {
        self.ty = ty;
        self.reset_state();
        let fields: Vec<_> = match &self.ty {
            Type::Struct { fields, .. } => {
                fields.iter()
                    .map(|Field { name, ty }| (name.clone(), Some(ty.clone())))
                    .collect()
            }
            Type::Enum { map, .. } => {
                let mut l = map.iter()
                    .map(|(k, v)| (k.to_string(), Some(v.clone())))
                    .collect::<Vec<_>>();
                l.push(("".to_string(), None));
                l
            }
            | Type::Array { element_type: ty, .. }
            | Type::Converter { original_type: ty, .. }
            | Type::Encrypt { inner_type: ty, .. }
            => vec![("".to_string(), Some(ty.as_ref().clone()))],
            _ => vec![],
        };
        if !fields.is_empty() {
            self.temp_fields.clear();
            self.ident_counter = 0;
            for (name, ty) in fields {
                self.add_temp_field(name, ty);
            }
        }
    }

    fn add_temp_field<S: Into<String>>(&mut self, name: S, ty: Option<Type>) -> &mut (String, TypeUi) {
        let last_idx = self.temp_fields.len();

        let name = name.into();
        let mut tui = TypeUi::new(format!("{} > {}[{}]", &self.ident, &name, self.ident_counter));
        if let Some(ty) = ty {
            tui.set_type(ty);
        }

        self.temp_fields.push((name, tui));
        self.ident_counter += 1;

        &mut self.temp_fields[last_idx]
    }

    pub fn from_type<S: Into<String>>(id: S, ty: Type) -> Self {
        let mut tui = Self::new(id);
        tui.set_type(ty);
        tui
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Response {
        egui::Grid::new(&self.ident)
            .spacing([5.0, 20.0])
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("类型");
                egui::ComboBox::from_id_source(format!("{} > type combox", &self.ident))
                    .selected_text(self.ty.type_name())
                    .show_ui(ui, |ui| {
                        let old_ty = self.ty.type_name();
                        for t in default_types() {
                            let name = t.type_name();
                            ui.selectable_value(&mut self.ty, t, name);
                        }
                        if self.ty.type_name() != old_ty {
                            self.reset_state()
                        }
                    });
                ui.end_row();

                let TypeUi {
                    ty,
                    ident,
                    ident_counter,
                    temp_bs_enum_key,
                    temp_bs_enum_value,
                    temp_bs_error,
                    temp_fields,
                    temp_enum_error,
                } = self;

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
                            temp_bs_enum_key,
                            temp_bs_enum_value,
                            temp_bs_error,
                            format!("{} > Bin/String", ident),
                        ));
                        ui.end_row();
                    }

                    Type::Struct { size, fields } => {
                        ui_struct(
                            ui,
                            ident,
                            ident_counter,
                            temp_bs_enum_key,
                            temp_bs_enum_value,
                            temp_bs_error,
                            temp_fields,
                            size,
                            fields,
                        );
                    }

                    Type::Array { size, length, element_type } => {
                        ui.label("大小");
                        ui.add(BytesSizeUi::new(
                            size,
                            temp_bs_enum_key,
                            temp_bs_enum_value,
                            temp_bs_error,
                            format!("{} > Array", ident),
                        ));
                        ui.end_row();

                        ui.label("长度");
                        ui.add(LengthUi(length));
                        ui.end_row();

                        ui.label("成员类型");
                        ui.vertical(|ui| {
                            let (_, ty_ui) = last_field(temp_fields);
                            ty_ui.ui(ui);
                            *element_type = Box::new(ty_ui.ty.clone());
                        });
                        ui.end_row();
                    }

                    Type::Enum { by, map, size } => {
                        if ui_enum(
                            ui,
                            ident,
                            temp_bs_enum_key,
                            temp_bs_enum_value,
                            temp_bs_error,
                            temp_fields,
                            temp_enum_error,
                            by,
                            map,
                            size,
                        ) {
                            self.add_temp_field("", None);
                        }
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

                        let (_, ty_ui) = last_field(temp_fields);
                        ui.label("原始类型");
                        ui.horizontal_top(|ui| ty_ui.ui(ui));
                        ui.end_row();
                        *original_type = Box::new(ty_ui.ty.clone());
                    }

                    Type::Checksum {
                        method,
                        start_key,
                        end_key,
                    } => {
                        ui_checksum(ui, ident, method, start_key, end_key);
                    }

                    Type::Encrypt {
                        inner_type,
                        on_read,
                        on_write,
                        size,
                    } => {
                        ui_encrypt(
                            ui,
                            ident,
                            temp_fields,
                            temp_bs_enum_key,
                            temp_bs_enum_value,
                            temp_bs_error,
                            inner_type,
                            on_read,
                            on_write,
                            size,
                        );
                    }
                    Type::Sign {
                        on_read,
                        on_write,
                        start_key,
                        end_key,
                        size
                    } => {
                        ui_sign(
                            ui,
                            ident,
                            temp_bs_enum_key,
                            temp_bs_enum_value,
                            temp_bs_error,
                            on_read,
                            on_write,
                            start_key,
                            end_key,
                            size,
                        );
                    }
                }
            })
            .response
    }
}

fn ui_struct(ui: &mut Ui, parent_id: &mut String, parent_id_counter: &mut usize, temp_bs_enum_key: &mut String, temp_bs_enum_value: &mut usize, temp_bs_error: &mut String, temp_fields: &mut Vec<(String, TypeUi)>, size: &mut Option<BytesSize>, fields: &mut Vec<Field>) {
    ui.label("大小");
    ui.add(BytesSizeUi::new(
        size,
        temp_bs_enum_key,
        temp_bs_enum_value,
        temp_bs_error,
        format!("{} > Struct", parent_id),
    ));
    ui.end_row();

    ui.horizontal(|ui| {
        ui.label("字段列表");
        if ui.button("+").on_hover_text("添加字段").clicked() {
            temp_fields.push((
                Default::default(),
                TypeUi::new(format!("{} > fields[{}]", parent_id, parent_id_counter)),
            ));
            *parent_id_counter += 1;
        }
    });
    egui::Grid::new("fields")
        .show(ui, |ui| {
            ui.label("字段名称");
            ui.label("类型定义");
            ui.label("操作");
            ui.end_row();

            let mut remove_list = vec![];
            for (idx, (name, ty)) in temp_fields.iter_mut().enumerate() {
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
                temp_fields.remove(idx);
            }

            fields.clear();
            fields.extend(temp_fields.iter()
                .map(|(name, ty)| {
                    Field::new(name, ty.ty.clone())
                }));
        });
    ui.end_row();
}


fn ui_enum(
    ui: &mut Ui,
    parent_id: &str,
    temp_bs_enum_key: &mut String,
    temp_bs_enum_value: &mut usize,
    temp_bs_error: &mut String,
    temp_fields: &mut Vec<(String, TypeUi)>,
    temp_enum_error: &mut String,
    by: &mut String,
    map: &mut KeyRangeMap<Type>,
    size: &mut Option<BytesSize>,
) -> bool {
    let mut add = false;
    ui.label("大小");
    ui.add(BytesSizeUi::new(
        size,
        temp_bs_enum_key,
        temp_bs_enum_value,
        temp_bs_error,
        format!("{} > Enum", parent_id),
    ));
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

                last_field(temp_fields);

                let last_idx = temp_fields.len() - 1;
                let mut remove_list = vec![];
                for (idx, (k, v)) in temp_fields
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
                    temp_fields.remove(idx);
                }

                for _ in 0..3 {
                    ui.separator();
                }
                ui.end_row();

                let last_idx = temp_fields.len() - 1;
                let (temp_kr, temp_ty) = last_field(temp_fields);

                let resp = ui.text_edit_singleline(temp_kr)
                    .on_hover_text(KEY_RANGE_FORMAT);
                if temp_enum_error.len() > 0 {
                    let pid = ui.make_persistent_id("enum error");
                    ui.memory().open_popup(pid);
                    egui::popup_below_widget(ui, pid, &resp, |ui| {
                        if ui.add(
                            egui::Label::new(egui::RichText::new(temp_enum_error.as_str()).color(egui::Color32::RED))
                                .wrap(false)
                                .sense(egui::Sense::click())
                        ).clicked() {
                            *temp_enum_error = Default::default();
                        }
                    });
                }

                ui.horizontal_top(|ui| temp_ty.ui(ui));
                if ui.button("添加/修改").clicked() {
                    if temp_kr.parse::<KeyRange>().is_ok() {
                        let kr = temp_kr.clone();
                        *temp_enum_error = Default::default();

                        temp_fields.iter()
                            .position(|(k, _)| k == &kr)
                            .map(|idx| {
                                if idx != last_idx {
                                    temp_fields.remove(idx);
                                }
                            });
                        add = true;
                    } else {
                        *temp_enum_error = format!("输入格式错误\n{}", KEY_RANGE_FORMAT);
                    }
                }

                map.clear();
                for (k, ty) in &temp_fields[..temp_fields.len() - 1] {
                    let kr: KeyRange = k.parse().unwrap();
                    map.insert(kr, ty.ty.clone());
                }
            });
    });
    ui.end_row();
    add
}

fn ui_checksum(
    ui: &mut Ui,
    parent_id: &str,
    method: &mut Checksum,
    start_key: &mut String,
    end_key: &mut Option<String>,
) {
    ui.label("计算方法");
    egui::ComboBox::from_id_source(format!("{} > CheckSum ComboBox", parent_id))
        .selected_text(method.name())
        .show_ui(ui, |ui| {
            ui.selectable_value(method, Checksum::Xor, Checksum::Xor.name());
            ui.selectable_value(method, Checksum::Complement, Checksum::Complement.name());
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

fn ui_encrypt(
    ui: &mut Ui,
    parent_id: &str,
    temp_fields: &mut Vec<(String, TypeUi)>,
    temp_bs_enum_key: &mut String,
    temp_bs_enum_value: &mut usize,
    temp_bs_error: &mut String,
    inner_type: &mut Box<Type>,
    on_read: &mut SecureKey,
    on_write: &mut SecureKey,
    size: &mut Option<BytesSize>,
) {
    ui.label("解密方式");
    ui.add(SecureKeyUi(on_read, format!("{} > Encrypt > on_read", parent_id), false));
    ui.end_row();

    ui.label("加密方式");
    ui.add(SecureKeyUi(on_write, format!("{} > Encrypt > on_write", parent_id), false));
    ui.end_row();

    ui.label("大小");
    ui.add(BytesSizeUi::new(
        size,
        temp_bs_enum_key,
        temp_bs_enum_value,
        temp_bs_error,
        format!("{} > Encrypt", parent_id),
    ));
    ui.end_row();

    let (_, ty_ui) = last_field(temp_fields);
    ui.label("内部数据类型");
    ui.horizontal_top(|ui| ty_ui.ui(ui));
    ui.end_row();
    *inner_type = Box::new(ty_ui.ty.clone());
}

fn ui_sign(
    ui: &mut Ui,
    parent_id: &str,
    temp_bs_enum_key: &mut String,
    temp_bs_enum_value: &mut usize,
    temp_bs_error: &mut String,
    on_read: &mut SecureKey,
    on_write: &mut SecureKey,
    start_key: &mut String,
    end_key: &mut Option<String>,
    size: &mut Option<BytesSize>,
) {
    ui.label("验证方式");
    ui.add(SecureKeyUi(on_read, format!("{} > Sign > on_read", parent_id), true));
    ui.end_row();

    ui.label("签名方式");
    ui.add(SecureKeyUi(on_write, format!("{} > Sign > on_write", parent_id), true));
    ui.end_row();

    ui.label("大小");
    ui.add(BytesSizeUi::new(
        size,
        temp_bs_enum_key,
        temp_bs_enum_value,
        temp_bs_error,
        format!("{} > sign", parent_id),
    ));
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

#[inline]
fn last_field(temp_fields: &mut Vec<(String, TypeUi)>) -> &mut (String, TypeUi) {
    let last_idx = temp_fields.len() - 1;
    &mut temp_fields[last_idx]
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
