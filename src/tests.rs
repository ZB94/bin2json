use deku::ctx::Endian;

use crate::{Array, BinToJson, BytesSize, Type, Value};
use crate::_struct::{Field, Struct};
use crate::ty::Unit;

#[test]
pub fn test_struct_read() {
    let message = Struct {
        fields: vec![
            Field::new("head", Type::magic(b"##")),
            Field::new("cmd", Type::Uint8(Default::default())),
            Field::new("device_id", Type::String(BytesSize::Fixed(17))),
            Field::new("version", Type::Uint8(Default::default())),
            Field::new("crypto_type", Type::Uint8(Default::default())),
            Field::new("data_len", Type::Uint16(Unit::big_endian())),
            Field::new("data", Type::Bin(BytesSize::by_field("data_len"))),
            Field::new("check", Type::Uint8(Default::default())),
        ],
        size: None,
    };
    let data = b"##\x0212345678901234567\x01\x01\x00\x1f\x15\x08\x1e\x0b\x05\x0c\x00\x01\x00\x06\xc9MH\x01X\xf4\xd8\x80\x002\x00d\x00\x96\x00\x01\x00\x02\x00\x00\x1e\xce";
    let (de, d) = message.read(data).unwrap();
    assert_eq!(0, d.len());
    assert_eq!(de["head"], b"##".into());
    assert_eq!(de["cmd"], 2u8.into());
    assert_eq!(de["device_id"], "12345678901234567".into());
    assert_eq!(de["data_len"], 31u16.into());
    assert_eq!(de["check"], 0xce_u8.into());

    let body = Struct {
        fields: vec![
            Field::new("datetime", Type::Bin(BytesSize::Fixed(6))),
            Field::new("number", Type::Uint16(Unit::big_endian())),
            Field::new("list", Type::Bin(BytesSize::All)),
        ],
        size: None,
    };
    let data = if let Value::Bin(data) = &de["data"] {
        data
    } else {
        panic!()
    };
    assert_eq!(data.len(), 31);
    let (de2, d) = body.read(data).unwrap();
    assert_eq!(de2["datetime"], vec![0x15u8, 0x08, 0x1E, 0x0B, 0x05, 0x0C].into());
    assert_eq!(de2["number"], 1u16.into());
    assert_eq!(d.len(), 0);
}

#[test]
fn test_array_read() {
    let mut array = Array {
        ty: Box::new(Type::Struct(Struct {
            fields: vec![
                Field::new("id", Type::Uint8(Default::default())),
                Field::new("data", Type::String(BytesSize::by_enum("id", vec![
                    ('1' as i64, 1),
                    ('2' as i64, 2),
                    ('3' as i64, 3),
                ].into_iter().collect()))),
            ],
            size: None,
        })),
        length: None,
        size: None,
    };
    let (a, d) = array.read_to_json(b"333322211").unwrap();
    assert_eq!(d.len(), 0);
    assert_eq!(a, serde_json::json!([
        { "id": '3' as u8, "data": "333" },
        { "id": '2' as u8, "data": "22" },
        { "id": '1' as u8, "data": "1" }
    ]));
    assert_eq!(array.read_to_json(b"").unwrap().0, serde_json::json!([]));

    array.length = Some(1);
    assert_eq!(array.read_to_json(b"333322211").unwrap(), (serde_json::json!([{ "id": '3' as u8, "data": "333" }]), b"22211".as_slice()));

    array.length = Some(4);
    assert!(array.read_to_json(b"333322211").is_err());
}


#[test]
fn test_read_enum() {
    let _enum = Struct {
        fields: vec![
            Field::new("key", Type::Uint8(Default::default())),
            Field::new("value", Type::Enum {
                by: "key".to_string(),
                map: [
                    (1, Type::String(BytesSize::Fixed(5))),
                    (2, Type::Bin(BytesSize::Fixed(5))),
                    (3, Type::Uint32(Unit::big_endian())),
                ].into_iter().collect(),
                size: None,
            }),
        ],
        size: None,
    };
    let array = Array {
        ty: Box::new(Type::Struct(_enum)),
        length: None,
        size: None
    };

    let data = b"\x01hello\x02world\x03\x00\x00\x00\xff";
    assert_eq!(array.read_to_json(data).unwrap(), (serde_json::json!([
        {
            "key": 1,
            "value": "hello"
        },
        {
            "key": 2,
            "value": b"world"
        },
        {
            "key": 3,
            "value": u32::from_be_bytes([0, 0,0, 0xff])
        }
    ]), [0u8; 0].as_slice()));
}
