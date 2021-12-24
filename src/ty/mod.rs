use deku::bitvec::{BitSlice, Msb0};
use deku::ctx::Limit;
pub use deku::ctx::Size;
use deku::prelude::*;
use serde_json::Map;

pub use array_length::Length;
pub use bytes_size::BytesSize;
pub use endian::Endian;
pub use field::Field;
use read_array::read_array;
use read_struct::read_struct;
pub use unit::Unit;
use utils::get_data_by_size;

use crate::bitvec::BitVec;
use crate::error::{ReadBinError, WriteBinError};
use crate::range::KeyRangeMap;
use crate::ty::utils::{set_ctx, to_json_value};
use crate::ty::write_struct::write_struct;
use crate::Value;

mod bytes_size;
mod unit;
mod read_struct;
mod field;
mod read_array;
mod array_length;
mod endian;
mod write_struct;
mod utils;

/// 数据类型
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Type {
    /// 魔法值。即一段固定数据值
    ///
    /// ```rust
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "Magic",
    ///     "magic": [1, 2, 3]
    /// }"#;
    /// assert_eq!(Type::magic(&[1, 2, 3]), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Magic {
        magic: Vec<u8>
    },

    /// 布尔型数据。
    ///
    /// ```rust
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "Boolean",
    ///     "bit": true
    /// }"#;
    /// assert_eq!(Type::BOOL_BIT, serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Boolean {
        /// 是否是位数据。如果是则读取1比特位的数据作为该值，否则读取1字节。
        bit: bool
    },

    /// 有符号8位整数
    ///
    /// ```rust
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "Int8"
    /// }"#;
    /// assert_eq!(Type::int8(), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Int8 {
        #[serde(default)]
        unit: Unit
    },

    /// 有符号16位整数
    ///
    /// ```rust
    /// use bin2json::ty::Endian;
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "Int16"
    /// }"#;
    /// assert_eq!(Type::int16(Endian::Big), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Int16 {
        #[serde(default)]
        unit: Unit
    },

    /// 有符号32位整数
    ///
    /// ```rust
    /// use bin2json::ty::Endian;
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "Int32"
    /// }"#;
    /// assert_eq!(Type::int32(Endian::Big), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Int32 {
        #[serde(default)]
        unit: Unit
    },

    /// 有符号64位整数
    ///
    /// ```rust
    /// use bin2json::ty::Endian;
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "Int64"
    /// }"#;
    /// assert_eq!(Type::int64(Endian::Big), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Int64 {
        #[serde(default)]
        unit: Unit
    },

    /// 无符号8位整数
    ///
    /// ```rust
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "Uint8"
    /// }"#;
    /// assert_eq!(Type::uint8(), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Uint8 {
        #[serde(default)]
        unit: Unit
    },

    /// 无符号16位整数
    ///
    /// ```rust
    /// use bin2json::ty::Endian;
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "Uint16"
    /// }"#;
    /// assert_eq!(Type::uint16(Endian::Big), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Uint16 {
        #[serde(default)]
        unit: Unit
    },

    /// 无符号32位整数
    ///
    /// ```rust
    /// use bin2json::ty::Endian;
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "Uint32"
    /// }"#;
    /// assert_eq!(Type::uint32(Endian::Big), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Uint32 {
        #[serde(default)]
        unit: Unit
    },

    /// 无符号64位整数
    ///
    /// ```rust
    /// use bin2json::ty::Endian;
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "Uint64"
    /// }"#;
    /// assert_eq!(Type::uint64(Endian::Big), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Uint64 {
        #[serde(default)]
        unit: Unit
    },

    /// 单精度浮点数
    ///
    /// ```rust
    /// use bin2json::ty::Endian;
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "Float32"
    /// }"#;
    /// assert_eq!(Type::float32(Endian::Big), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Float32 {
        #[serde(default)]
        endian: Endian
    },

    /// 双精度浮点数
    ///
    /// ```rust
    /// use bin2json::ty::Endian;
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "Float64"
    /// }"#;
    /// assert_eq!(Type::float64(Endian::Big), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Float64 {
        #[serde(default)]
        endian: Endian
    },

    /// UTF8字符串
    ///
    /// ```rust
    /// use bin2json::ty::BytesSize;
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "String",
    ///     "size": 64
    /// }"#;
    /// assert_eq!(Type::string(BytesSize::Fixed(64)), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    String {
        #[serde(default)]
        size: Option<BytesSize>,
    },

    /// 二进制数据
    ///
    /// ```rust
    /// use bin2json::ty::BytesSize;
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "Bin",
    ///     "size": "len by field"
    /// }"#;
    /// assert_eq!(Type::bin(BytesSize::new("len by field")), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Bin {
        #[serde(default)]
        size: Option<BytesSize>,
    },

    /// 结构体
    ///
    /// ```rust
    /// use bin2json::ty::Field;
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "Struct",
    ///     "fields": [
    ///         { "name": "f1", "type": "Uint8" }
    ///     ]
    /// }"#;
    /// assert_eq!(Type::new_struct(vec![Field::new("f1", Type::uint8())]), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Struct {
        /// 结构的字段列表
        fields: Vec<Field>,
        /// 手动指定结构的总字节大小
        #[serde(default)]
        size: Option<BytesSize>,
    },

    /// 数组
    ///
    /// ```rust
    /// use bin2json::ty::BytesSize;
    /// use bin2json::Type;
    /// let json = r#"{
    ///     "type": "Array",
    ///     "element_type": {
    ///         "type": "Uint8"
    ///     }
    /// }"#;
    /// assert_eq!(Type::new_array(Type::uint8()), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Array {
        element_type: Box<Type>,
        #[serde(default)]
        length: Option<Length>,
        #[serde(default)]
        size: Option<BytesSize>,
    },

    /// 枚举
    ///
    /// **注意:**
    /// - 枚举类型不能单独存在，必须位于[`Type::Struct`]的字段列表中
    ///
    /// ```rust
    /// use bin2json::ty::Endian;
    /// use bin2json::{Type, range_map};
    /// let json = r#"{
    ///     "type": "Enum",
    ///     "by": "field name",
    ///     "map": {
    ///         "1": { "type": "Uint8" },
    ///         "2..10": { "type": "Int16" }
    ///     }
    /// }"#;
    /// assert_eq!(Type::new_enum("field name", range_map!(1 => Type::uint8(), 2..10 => Type::int16(Endian::Big))), serde_json::from_str(json)?);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    Enum {
        /// 字段名称。*字段必须存在于与该枚举同级的结构体中，且指定的字段顺序应在该枚举之前*
        by: String,
        /// 枚举值对应的类型
        map: KeyRangeMap<Type>,
        /// 手动指定枚举的总字节大小
        #[serde(default)]
        size: Option<BytesSize>,
    },

    /// 转换
    ///
    /// 可以在执行[`Type::convert`]时对数据进行额外的计算，支持的表达式见[expreval](https://docs.rs/evalexpr/latest/evalexpr/)
    ///
    /// 不建议用在复杂类型上
    ///
    /// **执行表达式时有以下变量：**
    /// - 对于[`Type::Struct`]: `self.field_name`，其中`field_name`为字段列表中的字段名称
    /// - 对于[`Type::Array`]: `self[idx]`，其中`idx`为数组成员的下标
    /// - 对于其他类型: `self`
    Converter {
        #[serde(rename = "original_type")]
        ty: Box<Type>,
        on_read: String,
        on_write: String,
    },
}

/// Create
impl Type {
    /// 大小为1比特位的布尔值
    pub const BOOL_BIT: Type = Type::Boolean { bit: true };
    /// 大小为1字节的布尔值
    pub const BOOL: Type = Type::Boolean { bit: false };

    pub fn magic(magic: &[u8]) -> Self {
        Self::Magic { magic: magic.to_vec() }
    }

    pub fn int8() -> Self {
        Self::Int8 { unit: Default::default() }
    }

    pub fn int16(endian: Endian) -> Self {
        Self::Int16 { unit: endian.into() }
    }

    pub fn int32(endian: Endian) -> Self {
        Self::Int32 { unit: endian.into() }
    }

    pub fn int64(endian: Endian) -> Self {
        Self::Int64 { unit: endian.into() }
    }

    pub fn uint8() -> Self {
        Self::Uint8 { unit: Default::default() }
    }

    pub fn uint16(endian: Endian) -> Self {
        Self::Uint16 { unit: endian.into() }
    }

    pub fn uint32(endian: Endian) -> Self {
        Self::Uint32 { unit: endian.into() }
    }

    pub fn uint64(endian: Endian) -> Self {
        Self::Uint64 { unit: endian.into() }
    }

    pub fn float32(endian: Endian) -> Self {
        Self::Float32 { endian }
    }

    pub fn float64(endian: Endian) -> Self {
        Self::Float64 { endian }
    }

    pub fn string(size: BytesSize) -> Self {
        Self::String { size: Some(size) }
    }

    pub fn bin(size: BytesSize) -> Self {
        Self::Bin { size: Some(size) }
    }

    pub fn new_struct(fields: Vec<Field>) -> Self {
        Self::Struct { fields, size: None }
    }

    pub fn new_struct_with_size(fields: Vec<Field>, size: BytesSize) -> Self {
        Self::Struct { fields, size: Some(size) }
    }

    pub fn new_array(ty: Type) -> Self {
        Self::Array { element_type: Box::new(ty), size: None, length: None }
    }

    pub fn new_array_with_size(ty: Type, size: BytesSize) -> Self {
        Self::Array { element_type: Box::new(ty), size: Some(size), length: None }
    }

    pub fn new_array_with_length(ty: Type, length: Length) -> Self {
        Self::Array { element_type: Box::new(ty), size: None, length: Some(length) }
    }

    pub fn new_enum<S: Into<String>, M: Into<KeyRangeMap<Type>>>(by: S, map: M) -> Self {
        Self::Enum {
            by: by.into(),
            map: map.into(),
            size: None,
        }
    }

    pub fn converter<S: Into<String>>(ty: Type, on_read: S, on_write: S) -> Self {
        Self::Converter {
            ty: Box::new(ty),
            on_read: on_read.into(),
            on_write: on_write.into(),
        }
    }

    pub const fn type_name(&self) -> &'static str {
        match self {
            Type::Magic { .. } => "Magic",
            Type::Boolean { .. } => "Boolean",
            Type::Int8 { .. } => "Int8",
            Type::Int16 { .. } => "Int16",
            Type::Int32 { .. } => "Int32",
            Type::Int64 { .. } => "Int64",
            Type::Uint8 { .. } => "Uint8",
            Type::Uint16 { .. } => "Uint16",
            Type::Uint32 { .. } => "Uint32",
            Type::Uint64 { .. } => "Uint64",
            Type::Float32 { .. } => "Float32",
            Type::Float64 { .. } => "Float64",
            Type::String { .. } => "String",
            Type::Bin { .. } => "Bin",
            Type::Struct { .. } => "Struct",
            Type::Array { .. } => "Array",
            Type::Enum { .. } => "Enum",
            Type::Converter { .. } => "Converter",
        }
    }
}

macro_rules! parse_numeric_field {
    ($input: expr, $name: expr, $ty: ty, $unit: expr, $default_size: expr) => {{
        let size = $unit.size.unwrap_or($default_size);
        let (input, value) = <$ty>::read($input, (($unit.endian).into(), size))?;
        (value.into(), input)
    }};
}

/// Read
impl Type {
    /// 尝试从数据流中读取符合定义的JSON值
    ///
    /// **注意:** 本方法只读取原始数值，不对[`Type::Converter`]中的数据进行转化
    pub fn read<'a>(&self, data: &'a BitSlice<Msb0, u8>) -> Result<(Value, &'a BitSlice<Msb0, u8>), ReadBinError> {
        let (value, data): (Value, _) = match self {
            Self::Magic { ref magic } => {
                let (input, value): (_, Vec<u8>) = DekuRead::read(
                    data,
                    Limit::new_count(magic.len()),
                )?;

                if magic == &value {
                    (value.into(), input)
                } else {
                    return Err(ReadBinError::MagicError(magic.clone()));
                }
            }
            Self::Boolean { bit } => {
                let size = if *bit { Size::Bits(1) } else { Size::Bytes(1) };
                let (input, v) = bool::read(data, size)?;
                (v.into(), input)
            }
            Self::Int8 { unit } => {
                parse_numeric_field!(data, field.name, i8, unit, Size::Bytes(1))
            }
            Self::Int16 { unit } => {
                parse_numeric_field!(data, field.name, i16, unit, Size::Bytes(2))
            }
            Self::Int32 { unit } => {
                parse_numeric_field!(data, field.name, i32, unit, Size::Bytes(4))
            }
            Self::Int64 { unit } => {
                parse_numeric_field!(data, field.name, i64, unit, Size::Bytes(8))
            }
            Self::Uint8 { unit } => {
                parse_numeric_field!(data, field.name, u8, unit, Size::Bytes(1))
            }
            Self::Uint16 { unit } => {
                parse_numeric_field!(data, field.name, u16, unit, Size::Bytes(2))
            }
            Self::Uint32 { unit } => {
                parse_numeric_field!(data, field.name, u32, unit, Size::Bytes(4))
            }
            Self::Uint64 { unit } => {
                parse_numeric_field!(data, field.name, u64, unit, Size::Bytes(8))
            }
            Self::Float32 { endian } => {
                let (input, v): (_, f32) = DekuRead::<'_, deku::ctx::Endian>::read(data, (*endian).into())?;
                (v.into(), input)
            }
            Self::Float64 { endian } => {
                let (input, v): (_, f64) = DekuRead::<'_, deku::ctx::Endian>::read(data, (*endian).into())?;
                (v.into(), input)
            }
            Self::String { ref size } | Type::Bin { ref size } => {
                let d = get_data_by_size(data, size, None)?;
                let d_len = d.len();

                let (_, v) = Vec::<u8>::read(data, Limit::new_size(Size::Bits(d_len)))?;
                let v = if let Type::String { .. } = self {
                    String::from_utf8(v)?.into()
                } else {
                    v.into()
                };
                (v, &data[d_len..])
            }
            Self::Struct { fields, size } => {
                read_struct(fields, size, data)?
            }
            Self::Array { element_type: ty, size, length } => {
                read_array(ty, length, size, data)?
            }
            Self::Enum { by, .. } => return Err(ReadBinError::ByKeyNotFound(by.clone())),
            Self::Converter { ty, .. } => ty.read(data)?,
        };
        Ok((value, data))
    }

    /// 尝试从数据流中读取符合定义的JSON值并应用[`Type::Converter`]中的`on_read`表达式
    pub fn read_and_convert<'a>(&self, data: &'a BitSlice<Msb0, u8>) -> Result<(Value, &'a BitSlice<Msb0, u8>), ReadBinError> {
        let (v, d) = self.read(data)?;
        let v = self.convert(v, true)?;
        Ok((v, d))
    }
}

/// Write
impl Type {
    /// 尝试将指定的JSON值以定义的格式写到数据流中
    ///
    /// **注意:** 调用之前应对调用[`Type::convert`]方法转换数据，本方法不会对[`Type::Converter`]中的数据进行转化
    pub fn write(&self, value: &serde_json::Value) -> Result<BitVec<Msb0, u8>, WriteBinError> {
        let mut output = BitVec::new();

        macro_rules! v {
            ($conv: expr) => {{
                $conv.ok_or(WriteBinError::TypeError(self.type_name()))?
            }};
        }
        macro_rules! write_num {
            ($need_ty: ty, $unit: ident, $default_bytes: literal) => {
                let v = v!(value.as_f64());
                if v >= <$need_ty>::MIN as f64 && v <= <$need_ty>::MAX as f64 {
                    let ctx: (deku::ctx::Endian, Size) = ($unit.endian.into(), $unit.size.unwrap_or(Size::Bytes($default_bytes)));
                    (v as $need_ty).write(&mut output, ctx)?;
                } else {
                    return Err(WriteBinError::ValueOverflowOf(self.type_name()));
                }
            };
        }

        match self {
            | Type::String { size: Some(BytesSize::By(_) | BytesSize::Enum { .. }) }
            | Type::Bin { size: Some(BytesSize::By(_) | BytesSize::Enum { .. }) }
            | Type::Struct { size: Some(BytesSize::By(_) | BytesSize::Enum { .. }), .. }
            | Type::Array { size: Some(BytesSize::By(_) | BytesSize::Enum { .. }), .. }
            | Type::Array { length: Some(Length::By(_)), .. }
            | Type::Enum { .. }
            => return Err(WriteBinError::ByError),

            Type::Magic { magic } => {
                let m = utils::get_bin(v!(value.as_array()), self.type_name())?;
                if &m == magic {
                    m.write(&mut output, ())?;
                } else {
                    return Err(WriteBinError::MagicError { input: m, need: magic.clone() });
                }
            }
            Type::Boolean { bit } => {
                let b = v!(value.as_bool());
                let size = if *bit { Size::Bits(1) } else { Size::Bytes(1) };
                b.write(&mut output, size)?;
            }
            Type::Int8 { unit } => {
                write_num!(i8, unit, 1);
            }
            Type::Int16 { unit } => {
                write_num!(i16, unit, 2);
            }
            Type::Int32 { unit } => {
                write_num!(i32, unit, 4);
            }
            Type::Int64 { unit } => {
                write_num!(i64, unit, 8);
            }
            Type::Uint8 { unit } => {
                write_num!(u8, unit, 1);
            }
            Type::Uint16 { unit } => {
                write_num!(u16, unit, 2);
            }
            Type::Uint32 { unit } => {
                write_num!(u32, unit, 4);
            }
            Type::Uint64 { unit } => {
                write_num!(u64, unit, 8);
            }
            Type::Float32 { endian } => {
                let v = v!(value.as_f64());
                let f = v as f32;
                if f.is_infinite() && !v.is_infinite() {
                    return Err(WriteBinError::ValueOverflowOf(self.type_name()));
                } else {
                    let endian: deku::ctx::Endian = (*endian).into();
                    f.write(&mut output, endian)?;
                }
            }
            Type::Float64 { endian } => {
                let endian: deku::ctx::Endian = (*endian).into();
                v!(value.as_f64()).write(&mut output, endian)?;
            }
            Type::Bin { size } | Type::String { size } => {
                #[allow(unused_assignments)] let mut c = None;

                let b = if let Type::String { .. } = self {
                    v!(value.as_str()).as_bytes()
                } else {
                    c = Some(utils::get_bin(v!(value.as_array()), self.type_name())?);
                    c.as_ref()
                        .map(|v| v.as_slice())
                        .unwrap()
                };

                let e = match size {
                    Some(BytesSize::Fixed(size)) => *size == b.len(),
                    Some(BytesSize::EndWith(end)) => b.ends_with(end),
                    _ => true,
                };
                if e {
                    b.write(&mut output, ())?;
                } else {
                    return Err(WriteBinError::BytesSizeError);
                }
            }

            Type::Array { element_type, length, size } => {
                let mut out = BitVec::new();
                let mut len = 0;
                v!(value.as_array())
                    .iter()
                    .map(|v| -> Result<(), WriteBinError> {
                        out.append(&mut element_type.write(v)?);
                        len += 1;
                        Ok(())
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                if let Some(Length::Fixed(l)) = length {
                    if l != &len {
                        return Err(WriteBinError::LengthError { input: len, need: *l });
                    }
                }

                utils::check_size(size, &out)?;
                output = out;
            }
            Type::Struct { fields, size } => {
                let obj = v!(value.as_object());
                let out = write_struct(fields, obj)?;
                utils::check_size(size, &out)?;
                output = out;
            }
            Type::Converter { ty, .. } => {
                output = ty.write(value)?;
            }
        };

        Ok(output)
    }

    /// 对输入数据应用[`Type::Converter`]中的`on_write`表达式，并将应用后的数据写到数据流中
    pub fn convert_and_write(&self, value: serde_json::Value) -> Result<BitVec<Msb0, u8>, WriteBinError> {
        let value = self.convert(value, false)?;
        self.write(&value)
    }
}

/// Convert
impl Type {
    /// 如果类型为[`Type::Converter`]，则将输入值作为变量执行设置的表达式，并返回表达式执行的结果，否则返回输入值
    pub fn convert(&self, value: Value, is_read: bool) -> Result<Value, evalexpr::EvalexprError> {
        match (self, value) {
            (Type::Converter { on_read, on_write, .. }, value) => {
                let mut ctx = evalexpr::HashMapContext::new();
                set_ctx(&value, None, &mut ctx)?;

                let expr = if is_read { on_read } else { on_write };
                let v = evalexpr::eval_with_context(expr, &ctx)?;
                Ok(to_json_value(v))
            }
            (Type::Struct { fields, .. }, Value::Object(mut map)) => {
                let mut rm = Map::new();
                for Field { name, ty } in fields {
                    if let Some((k, v)) = map.remove_entry(name) {
                        let ty = if let Type::Enum { by, map, .. } = ty {
                            let k = rm.get(by)
                                .ok_or(evalexpr::EvalexprError::CustomMessage(format!("未找到引用键: {}", by)))?
                                .as_i64()
                                .ok_or(evalexpr::EvalexprError::CustomMessage(format!("引用键({})无法转化为整数", by)))?;
                            map.get(&k)
                                .ok_or(evalexpr::EvalexprError::CustomMessage(format!("未能找到引用键({})对应的类型({})", by, k)))?
                        } else {
                            ty
                        };
                        rm.insert(k, ty.convert(v, is_read)?);
                    }
                }
                rm.append(&mut map);
                Ok(Value::Object(rm))
            }
            (Type::Array { element_type, .. }, Value::Array(array)) => {
                let a = array.into_iter()
                    .map(|v| element_type.convert(v, is_read))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Value::Array(a))
            }
            (_, value) => Ok(value),
        }
    }
}
