use eframe::egui;
use eframe::egui::{Response, Ui};

use bin2json::{range_map, Type};
use bin2json::secure::SecureKey;
use bin2json::ty::{Checksum, Endian};
pub use bytes_size_ui::BytesSizeUi;
pub use endian_ui::EndianUi;
pub use raw_edit_ui::RawEditUi;
pub use size_ui::SizeUi;

mod size_ui;
mod endian_ui;
mod bytes_size_ui;
mod raw_edit_ui;

pub struct TypeUi {
    ty: Type,
    temp_string: String,
    temp_usize: usize,
    error: String,
}

impl TypeUi {
    pub fn new() -> Self {
        Self {
            ty: Type::uint8(),
            temp_string: "".to_string(),
            temp_usize: 0,
            error: "".to_string(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Response {
        let ty = &mut self.ty;
        egui::Grid::new("root")
            .spacing([5.0, 10.0])
            .show(ui, |ui| {
                ui.label("类型");
                egui::ComboBox::from_id_source("type select")
                    .selected_text(ty.type_name())
                    .show_ui(ui, |ui| {
                        let old_ty = ty.type_name();
                        for t in default_types() {
                            let name = t.type_name();
                            ui.selectable_value(ty, t, name);
                        }
                        if ty.type_name() != old_ty {
                            self.temp_string = Default::default();
                            self.temp_usize = 0;
                            self.error = Default::default();
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
                            &mut self.temp_string,
                            &mut self.temp_usize,
                            &mut self.error,
                        ));
                        ui.end_row();
                    }

                    Type::Struct { .. } => {}
                    Type::Array { .. } => {}
                    Type::Enum { .. } => {}
                    Type::Converter { .. } => {}
                    Type::Checksum { .. } => {}
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
