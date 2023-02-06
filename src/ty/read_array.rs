use deku::bitvec::{BitSlice, Msb0};

use crate::error::ReadBinError;
use crate::ty::utils::get_data_by_size;
use crate::ty::{BytesSize, Length};
use crate::Type;
use crate::Value;

pub fn read_array<'a>(
    ty: &Type,
    length: &Option<Length>,
    size: &Option<BytesSize>,
    data: &'a BitSlice<u8, Msb0>,
) -> Result<(Value, &'a BitSlice<u8, Msb0>), ReadBinError> {
    let src = data;
    let mut data = get_data_by_size(data, size, None)?;
    let data_len = data.len();

    let (mut ret, len) = match length {
        Some(Length::Fixed(size)) => (Vec::with_capacity(*size), *size),
        Some(Length::By(by)) => return Err(ReadBinError::ByKeyNotFound(by.clone())),
        None => (vec![], 0),
    };

    loop {
        match ty.read(data) {
            Ok((s, d)) => {
                data = d;
                ret.push(s);
                if len > 0 && ret.len() == len {
                    break;
                }
            }
            Err(_) => {
                if len == 0 {
                    break;
                } else {
                    return Err(ReadBinError::Incomplete);
                }
            }
        }
    }

    Ok((Value::Array(ret), &src[data_len - data.len()..]))
}
