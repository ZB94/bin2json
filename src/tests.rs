use deku::bitvec::BitView;
use deku::ctx::{Endian, Size};
use serde_json::json;

use crate::{Array, BinToJson, BytesSize, Length, Type, Value};
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
    let (de, d) = message.read(data.view_bits()).unwrap();
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
    let (de2, d) = body.read(data.view_bits()).unwrap();
    assert_eq!(de2["datetime"], vec![0x15u8, 0x08, 0x1E, 0x0B, 0x05, 0x0C].into());
    assert_eq!(de2["number"], 1u16.into());
    assert_eq!(d.len(), 0);
}

#[test]
fn test_array_read() {
    let mut array = Array::new(Type::Struct(Struct::new(vec![
        Field::new("id", Type::Uint8(Default::default())),
        Field::new("data", Type::String(BytesSize::by_enum("id", vec![
            ('1' as i64, 1),
            ('2' as i64, 2),
            ('3' as i64, 3),
        ].into_iter().collect()))),
    ])));
    let (a, d) = array.read_to_json(b"333322211".view_bits()).unwrap();
    assert_eq!(d.len(), 0);
    assert_eq!(a, serde_json::json!([
        { "id": '3' as u8, "data": "333" },
        { "id": '2' as u8, "data": "22" },
        { "id": '1' as u8, "data": "1" }
    ]));
    assert_eq!(array.read_to_json(b"".view_bits()).unwrap().0, json!([]));

    array.length = Length::Fixed(1);
    assert_eq!(array.read_to_json(b"333322211".view_bits()).unwrap(), (json!([{ "id": '3' as u8, "data": "333" }]), b"22211".view_bits()));

    array.length = Length::Fixed(4);
    assert!(array.read_to_json(b"333322211".view_bits()).is_err());
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
    let array = Array::new(Type::Struct(_enum));

    let data = b"\x01hello\x02world\x03\x00\x00\x00\xff";
    assert_eq!(array.read_to_json(data.view_bits()).unwrap(), (json!([
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
    ]), [0u8; 0].as_slice().view_bits()));
}


#[test]
fn test_read() {
    let message = Type::new_struct(vec![
        Field::new("head", Type::magic(b"##")),
        Field::new("command", Type::uint8()),
        Field::new("device_id", Type::String(BytesSize::Fixed(17))),
        Field::new("version", Type::uint8()),
        Field::new("crypto_type", Type::uint8()),
        Field::new("data_len", Type::Uint16(Unit::big_endian())),
        Field::new("data", Type::Enum {
            by: "command".to_string(),
            map: [
                (1, Type::new_struct(vec![
                    Field::new("datetime", Type::Bin(BytesSize::Fixed(6))),
                    Field::new("number", Type::Uint16(Unit::big_endian())),
                    Field::new("sim_id", Type::String(BytesSize::Fixed(20))),
                ])),
                (2, Type::new_struct(vec![
                    Field::new("datetime", Type::Bin(BytesSize::Fixed(6))),
                    Field::new("number", Type::Uint16(Unit::big_endian())),
                    Field::new("infos", Type::new_array(Type::new_struct(vec![
                        Field::new("info_type", Type::uint8()),
                        Field::new("info", Type::new_enum("info_type", [
                            (1, Type::new_struct(vec![
                                Field::new("protocol", Type::uint8()),
                                Field::new("mil_status", Type::uint8()),
                                Field::new("support_status", Type::Uint16(Unit::big_endian())),
                                Field::new("ready_status", Type::Uint16(Unit::big_endian())),
                                Field::new("vin", Type::String(BytesSize::Fixed(17))),
                                Field::new("scin", Type::String(BytesSize::Fixed(18))),
                                Field::new("cvn", Type::String(BytesSize::Fixed(18))),
                                Field::new("iupr", Type::String(BytesSize::Fixed(36))),
                                Field::new("code_len", Type::uint8()),
                                Field::new("code_list", Type::Array(Array {
                                    ty: Box::new(Type::Uint32(Unit::big_endian())),
                                    length: Length::by_field("code_len"),
                                    size: None,
                                })),
                            ])),
                            (2, Type::new_struct(vec![
                                Field::new("speed", Type::Uint16(Unit::big_endian())),
                                Field::new("atmospheric_pressure", Type::uint8()),
                                Field::new("torque", Type::uint8()),
                                Field::new("friction_torque", Type::uint8()),
                                Field::new("engine_speed", Type::Uint16(Unit::big_endian())),
                                Field::new("engine_fuel_flow", Type::Uint16(Unit::big_endian())),
                                Field::new("scr_nox_up", Type::Uint16(Unit::big_endian())),
                                Field::new("scr_nox_down", Type::Uint16(Unit::big_endian())),
                                Field::new("reactant", Type::uint8()),
                                Field::new("air_intake", Type::Uint16(Unit::big_endian())),
                                Field::new("scr_temp_in", Type::Uint16(Unit::big_endian())),
                                Field::new("scr_temp_out", Type::Uint16(Unit::big_endian())),
                                Field::new("dpf_pressure", Type::Uint16(Unit::big_endian())),
                                Field::new("engine_coolant_temp", Type::uint8()),
                                Field::new("oil_volume", Type::uint8()),
                                Field::new("pos_invalid", Type::BOOL_BIT),
                                Field::new("pos_south", Type::BOOL_BIT),
                                Field::new("pos_east", Type::BOOL_BIT),
                                Field::new("skip", Type::Uint8(Unit::new(Endian::Big, Some(Size::Bits(5))))),
                                Field::new("longitude", Type::Uint32(Unit::big_endian())),
                                Field::new("latitude", Type::Uint32(Unit::big_endian())),
                                Field::new("mileage", Type::Uint32(Unit::big_endian())),
                            ])),
                            (130, Type::new_struct(vec![
                                Field::new("absorption_coefficient", Type::Uint16(Unit::big_endian())),
                                Field::new("opaque", Type::Uint16(Unit::big_endian())),
                                Field::new("pm", Type::Uint16(Unit::big_endian())),
                            ]))
                        ].into_iter().collect())),
                    ]))),
                ]))
            ].into_iter().collect(),
            size: Some(BytesSize::by_field("data_len")),
        }),
        Field::new("check", Type::uint8()),
    ]);

    let login = [
        35, 35,
        1,
        49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 48, 49,
        1,
        1,
        0, 28,
        21, 12, 17, 14, 54, 1,
        0, 3,
        49, 50, 51, 52, 53, 54, 55, 56, 57, 48,
        49, 50, 51, 52, 53, 54, 55, 57, 56, 48,
        30
    ];
    assert_eq!(message.read_to_json(login.view_bits()).unwrap(), (json!({
        "head": b"##",
        "command": 1,
        "device_id": "12345678901234501",
        "version": 1,
        "crypto_type": 1,
        "data_len": 28,
        "data": {
            "datetime": [21, 12, 17, 14, 54, 1],
            "number": 3,
            "sim_id": "12345678901234567980"
        },
        "check": 30
    }), login[login.len()..].view_bits()));

    let info = [
        35, 35,
        2,
        49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 48, 49,
        1,
        1,
        0, 162,
        21, 12, 17, 15, 2, 35,
        0, 4,
        2,
        1, 0,
        4,
        128,
        129,
        0, 40,
        0, 120,
        16, 44,
        16, 64,
        22,
        0, 200,
        35, 128,
        35, 160,
        0, 130,
        54,
        37,
        32,
        0, 25, 240, 160,
        0, 27, 119, 64,
        0, 0, 0, 190,
        130,
        7, 208,
        0, 210,
        0, 220,
        1,
        1,
        1,
        23, 0,
        24, 0,
        49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 48, 49,
        50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57,
        48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55,
        56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53,
        54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51,
        3,
        0, 0, 0, 25,
        0, 0, 0, 26,
        0, 0, 0, 27,
        98
    ];

    assert_eq!(message.read_to_json(info.view_bits()).unwrap(), (json!({
        "head": b"##",
        "command": 2,
        "device_id": "12345678901234501",
        "version": 1,
        "crypto_type": 1,
        "data_len": 162,
        "data": {
            "datetime": [21, 12, 17, 15, 2, 35],
            "number": 4,
            "infos": [
                {
                    "info_type": 2,
                    "info": {
                        "speed": u16::from_be_bytes([1, 0]),
                        "atmospheric_pressure": 4,
                        "torque": 128,
                        "friction_torque": 129,
                        "engine_speed": 40,
                        "engine_fuel_flow": 120,
                        "scr_nox_up": u16::from_be_bytes([16, 44]),
                        "scr_nox_down": u16::from_be_bytes([16, 64]),
                        "reactant": 22,
                        "air_intake": 200,
                        "scr_temp_in": u16::from_be_bytes([35, 128]),
                        "scr_temp_out": u16::from_be_bytes([35, 160]),
                        "dpf_pressure": 130,
                        "engine_coolant_temp": 54,
                        "oil_volume": 37,
                        "pos_invalid": false,
                        "pos_south": false,
                        "pos_east": true,
                        "skip": 0,
                        "longitude": u32::from_be_bytes([0, 25, 240, 160]),
                        "latitude": u32::from_be_bytes([0, 27, 119, 64]),
                        "mileage": u32::from_be_bytes([0, 0, 0, 190]),
                    }
                },
                {
                    "info_type": 130,
                    "info": {
                        "absorption_coefficient": u16::from_be_bytes([7, 208]),
                        "opaque": 210,
                        "pm": 220,
                    }
                },
                {
                    "info_type": 1,
                    "info": {
                        "protocol": 1,
                        "mil_status": 1,
                        "support_status": u16::from_be_bytes([23, 0]),
                        "ready_status": u16::from_be_bytes([24, 0]),
                        "vin": "12345678901234501",
                        "scin": "234567890123456789",
                        "cvn": "012345678901234567",
                        "iupr": "890123456789012345678901234567890123",
                        "code_len": 3,
                        "code_list": [25, 26, 27]
                    }
                }
            ]
        },
        "check": 98
    }), info[info.len()..].view_bits()));
}
