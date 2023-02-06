use deku::bitvec::{BitSlice, BitVec, BitView, Msb0};
use deku::ctx::Limit;
use deku::{DekuError, DekuRead};
use evalexpr::ContextWithMutableVariables;
use serde_json::{Map, Value};

use crate::error::WriteBinError;
use crate::ty::BytesSize;
use crate::ReadBinError;

pub fn get_bin(
    list: &Vec<serde_json::Value>,
    type_name: &'static str,
) -> Result<Vec<u8>, WriteBinError> {
    list.iter()
        .map(|v| {
            let v = v.as_u64().ok_or(WriteBinError::TypeError(type_name))?;
            if v <= u8::MAX as u64 {
                Ok(v as u8)
            } else {
                Err(WriteBinError::TypeError(type_name))
            }
        })
        .collect()
}

/// 检查输入是否为指定长度或者以指定字节数组结束
pub fn check_size(size: &Option<BytesSize>, out: &BitVec<u8, Msb0>) -> Result<(), WriteBinError> {
    if let Some(BytesSize::Fixed(size)) = size {
        if *size * 8 != out.len() {
            return Err(WriteBinError::BytesSizeError);
        }
    } else if let Some(BytesSize::EndWith(end)) = size {
        if !out.ends_with(end.view_bits::<Msb0>()) {
            return Err(WriteBinError::BytesSizeError);
        }
    }
    Ok(())
}

pub fn to_json_value(value: evalexpr::Value) -> serde_json::Value {
    match value {
        evalexpr::Value::String(s) => s.into(),
        evalexpr::Value::Float(f) => f.into(),
        evalexpr::Value::Int(i) => i.into(),
        evalexpr::Value::Boolean(b) => b.into(),
        evalexpr::Value::Tuple(l) => {
            serde_json::Value::Array(l.into_iter().map(to_json_value).collect())
        }
        evalexpr::Value::Empty => serde_json::Value::Null,
    }
}

pub fn set_ctx(
    value: &serde_json::Value,
    prefix: Option<String>,
    ctx: &mut evalexpr::HashMapContext,
) -> evalexpr::EvalexprResult<()> {
    let ident = prefix.unwrap_or_else(|| "self".to_string());

    match value {
        serde_json::Value::Null => ctx.set_value(ident, evalexpr::Value::Empty)?,
        serde_json::Value::Bool(b) => ctx.set_value(ident, evalexpr::Value::Boolean(*b))?,
        serde_json::Value::Number(n) => {
            let v = evalexpr::Value::Float(n.as_f64().unwrap());
            ctx.set_value(ident, v)?;
        }
        serde_json::Value::String(s) => ctx.set_value(ident, evalexpr::Value::String(s.clone()))?,
        serde_json::Value::Array(a) => {
            ctx.set_value(
                format!("{}.len", a.len()),
                evalexpr::Value::Int(a.len() as i64),
            )?;
            for (idx, v) in a.iter().enumerate() {
                let ident = format!("{}[{}]", &ident, idx);
                set_ctx(v, Some(ident), ctx)?
            }
        }
        serde_json::Value::Object(m) => {
            for (k, v) in m {
                let ident = format!("{}.{}", ident, k);
                set_ctx(v, Some(ident), ctx)?;
            }
        }
    };
    Ok(())
}

pub fn get_data_by_size<'a>(
    data: &'a BitSlice<u8, Msb0>,
    size: &Option<BytesSize>,
    by_map: Option<&Map<String, Value>>,
) -> Result<&'a BitSlice<u8, Msb0>, ReadBinError> {
    let len = match size {
        None => return Ok(data),
        Some(BytesSize::Fixed(size)) => *size,
        Some(BytesSize::EndWith(with)) => {
            let with_end_error = |e: DekuError| -> ReadBinError {
                if let DekuError::Incomplete(_) = &e {
                    ReadBinError::EndNotFound(with.clone())
                } else {
                    e.into()
                }
            };

            let (mut d, mut v) =
                Vec::<u8>::read(data, Limit::new_count(with.len())).map_err(with_end_error)?;
            while !v.ends_with(with) {
                let (d2, b) = u8::read(d, ()).map_err(with_end_error)?;
                v.push(b);
                d = d2;
            }
            v.len()
        }
        Some(BytesSize::By(ref by) | BytesSize::Enum { ref by, .. }) => {
            if let Some(map) = by_map {
                let by_value = map.get(by).ok_or(ReadBinError::ByKeyNotFound(by.clone()))?;

                if let Some(BytesSize::Enum { map, .. }) = size {
                    as_i64(by_value).and_then(|key| map.get(&key).copied())
                } else {
                    as_u64(by_value).map(|s| s as usize)
                }
                .ok_or(ReadBinError::LengthTargetIsInvalid(by.clone()))?
            } else {
                return Err(ReadBinError::ByKeyNotFound(by.clone()));
            }
        }
    };

    let bits_len = len * 8;
    if data.len() >= bits_len {
        Ok(&data[..bits_len])
    } else {
        Err(ReadBinError::Incomplete)
    }
}

pub fn as_i64(num: &Value) -> Option<i64> {
    num.as_i64().or_else(|| {
        if let Some(f) = num.as_f64() {
            if f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
                return Some(f as i64);
            }
        }
        None
    })
}

pub fn as_u64(num: &Value) -> Option<u64> {
    num.as_u64().or_else(|| {
        if let Some(f) = num.as_f64() {
            if f.fract() == 0.0 && f >= 0.0 && f <= u64::MAX as f64 {
                return Some(f as u64);
            }
        }
        None
    })
}
