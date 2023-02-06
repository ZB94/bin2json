use deku::bitvec::{BitView, Msb0};
use deku::ctx::BitSize;
use rand::rngs::OsRng;
use rand::RngCore;
use rsa::pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey};
use rsa::pkcs8::LineEnding;
use rsa::{Pkcs1v15Encrypt, PublicKey, PublicKeyParts, RsaPrivateKey};
use serde_json::json;

use crate::secure::{Hasher, SecureKey};
use crate::ty::{BytesSize, Checksum, Endian, Field, Length, Unit};
use crate::{range_map, Type};

#[test]
fn test_convert_enum() {
    let ty = Type::new_struct(vec![
        Field::new("head", Type::magic(&[0x21])),
        Field::new("cmd", Type::uint8()),
        Field::new(
            "data_len",
            Type::converter(Type::uint8(), "self - 3", "self + 3"),
        ),
        Field::new(
            "data",
            Type::Enum {
                size: Some(BytesSize::new("data_len")),
                by: "cmd".to_string(),
                map: range_map! {
                    0x50 => Type::magic(&[])
                },
            },
        ),
        Field::new("checksum", Type::checksum(Checksum::Complement, "head")),
    ]);

    let data = [0x21u8, 0x50, 0x03, 0x8C];
    let r = ty.read(data.view_bits()).unwrap().0;
    assert_eq!(
        json!({
            "head": [0x21],
            "cmd": 0x50,
            "data_len": 0.0,
            "data": [],
            "checksum": [0x8c]
        }),
        r
    );
}

#[test]
fn test_sign() {
    let sk = rsa::RsaPrivateKey::new(&mut rand::rngs::OsRng, 1024).unwrap();
    let pk = sk.to_public_key();
    let sk_pem = sk.to_pkcs1_pem(LineEnding::LF).unwrap().to_string();
    let pk_pem = pk.to_pkcs1_pem(LineEnding::LF).unwrap();

    let message = Type::new_struct(vec![
        Field::new("head", Type::magic(b"##")),
        Field::new("command", Type::uint8()),
        Field::new("device_id", Type::string(BytesSize::Fixed(17))),
        Field::new("version", Type::uint8()),
        Field::new("crypto_type", Type::uint8()),
        Field::new("data_len", Type::uint16(Endian::Big)),
        Field::new(
            "data",
            Type::Enum {
                by: "command".to_string(),
                map: range_map! {
                    1 => Type::new_struct(vec![
                        Field::new("status", Type::uint8()),
                        Field::new("number", Type::uint16(Endian::Big)),
                        Field::new("result", Type::BOOL),
                        Field::new("sign_len", Type::uint16(Endian::Big)),
                        Field::new("sign", Type::Sign {
                            size: Some(BytesSize::new("sign_len")),
                            start_key: "status".to_string(),
                            end_key: Some("sign_len".to_string()),
                            on_read: SecureKey::rsa_pkcs1_pem(&pk_pem, false, Hasher::SHA3_256),
                            on_write: SecureKey::rsa_pkcs1_pem(&sk_pem, true, Hasher::SHA3_256),
                        })
                    ]),
                    2 => Type::new_struct(vec![
                        Field::new("status", Type::uint8()),
                        Field::new("number", Type::uint16(Endian::Big)),
                        Field::new("sign_len", Type::uint16(Endian::Big)),
                        Field::new("sign", Type::Sign {
                            size: Some(BytesSize::new("sign_len")),
                            start_key: "status".to_string(),
                            end_key: Some("sign_len".to_string()),
                            on_read: SecureKey::rsa_pkcs1_pem(&pk_pem, false, Hasher::SHA3_256),
                            on_write: SecureKey::rsa_pkcs1_pem(&sk_pem, true, Hasher::SHA3_256),
                        })
                    ]),
                    3 => Type::new_struct(vec![
                        Field::new("status", Type::uint8()),
                        Field::new("number", Type::uint16(Endian::Big)),
                        Field::new("sign_len", Type::uint16(Endian::Big)),
                        Field::new("sign", Type::Sign {
                            size: Some(BytesSize::new("sign_len")),
                            start_key: "status".to_string(),
                            end_key: Some("sign_len".to_string()),
                            on_read: SecureKey::rsa_pkcs1_pem(&pk_pem, false, Hasher::SHA3_256),
                            on_write: SecureKey::rsa_pkcs1_pem(&sk_pem, true, Hasher::SHA3_256),
                        })
                    ]),
                    4 => Type::new_struct(vec![
                        Field::new("status", Type::uint8()),
                        Field::new("number", Type::uint16(Endian::Big)),
                        Field::new("sign_len", Type::uint16(Endian::Big)),
                        Field::new("sign", Type::Sign {
                            size: Some(BytesSize::new("sign_len")),
                            start_key: "status".to_string(),
                            end_key: Some("sign_len".to_string()),
                            on_read: SecureKey::rsa_pkcs1_pem(&pk_pem, false, Hasher::SHA3_256),
                            on_write: SecureKey::rsa_pkcs1_pem(&sk_pem, true, Hasher::SHA3_256),
                        })
                    ]),
                    5 => Type::new_struct(vec![
                        Field::new("status", Type::uint8()),
                        Field::new("year", Type::converter(Type::uint8(), "self + 2000", "self - 2000")),
                        Field::new("month", Type::uint8()),
                        Field::new("day", Type::uint8()),
                        Field::new("hour", Type::uint8()),
                        Field::new("minute", Type::uint8()),
                        Field::new("second", Type::uint8()),
                        Field::new("sign_len", Type::uint16(Endian::Big)),
                        Field::new("sign", Type::Sign {
                            size: Some(BytesSize::new("sign_len")),
                            start_key: "status".to_string(),
                            end_key: Some("sign_len".to_string()),
                            on_read: SecureKey::rsa_pkcs1_pem(&pk_pem, false, Hasher::SHA3_256),
                            on_write: SecureKey::rsa_pkcs1_pem(&sk_pem, true, Hasher::SHA3_256),
                        })
                    ])
                },
                size: Some(BytesSize::new("data_len")),
            },
        ),
        Field::new("check", Type::checksum(Checksum::Xor, "command")),
    ]);

    let array = Type::new_array(message);
    let msg_list = json!([
        {
            "command": 1,
            "device_id": "12345678901234501",
            "version": 1,
            "crypto_type": 1,
            "data": {
                "status": 0,
                "number": 1,
                "result": true
            }
        },
        {
            "command": 2,
            "device_id": "12345678901234501",
            "version": 1,
            "crypto_type": 1,
            "data": {
                "status": 1,
                "number": 2
            }
        },
        {
            "command": 3,
            "device_id": "12345678901234501",
            "version": 1,
            "crypto_type": 1,
            "data": {
                "status": 0,
                "number": 3
            }
        },
        {
            "command": 4,
            "device_id": "12345678901234501",
            "version": 1,
            "crypto_type": 1,
            "data": {
                "status": 0,
                "number": 4
            }
        },
        {
            "command": 5,
            "device_id": "12345678901234501",
            "version": 1,
            "crypto_type": 1,
            "data": {
                "status": 0,
                "year": 2021,
                "month": 1,
                "day": 2,
                "hour": 3,
                "minute": 4,
                "second": 5
            }
        }
    ]);
    let raw = array.write(&msg_list).unwrap();
    array.read(&raw).unwrap();
}

#[test]
fn test_encrypt() {
    let sk = rsa::RsaPrivateKey::new(&mut rand::rngs::OsRng, 1024).unwrap();
    let pk = sk.to_public_key();
    let sk_pem = sk.to_pkcs1_pem(LineEnding::LF).unwrap().to_string();
    let pk_pem = pk.to_pkcs1_pem(LineEnding::LF).unwrap();

    eprintln!("sk:\n{}", &sk_pem);
    eprintln!("pk:\n{}", &pk_pem);

    let ty = Type::encrypt(
        Type::bin(BytesSize::new(50)),
        SecureKey::rsa_pkcs1_pem(sk_pem, true, Hasher::None),
        SecureKey::rsa_pkcs1_pem(pk_pem, false, Hasher::None),
    );

    let mut data: [u8; 50] = [0; 50];
    rand::thread_rng().fill_bytes(&mut data);

    let msg = ty.write(&json!(data.as_slice())).unwrap();
    let (out, _) = ty.read(&msg).unwrap();
    assert_eq!(json!(data.as_slice()), out);
}

#[test]
fn test_checksum() {
    let cs = Type::new_struct(vec![
        Field::new("data", Type::bin(BytesSize::new(10))),
        Field::new("checksum", Type::checksum(Checksum::Xor, "data")),
    ]);
    let mut data = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let checksum = {
        let mut b = data[0];
        for i in &data[1..] {
            b ^= *i;
        }
        b
    };
    data.push(checksum);
    assert_eq!(
        json!({ "data": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10], "checksum": [checksum] }),
        cs.read(data.view_bits()).unwrap().0
    );

    let d = cs
        .write(&json!({ "data": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]}))
        .unwrap();
    assert_eq!(data.view_bits::<Msb0>(), d);
}

#[test]
fn test_write_array() {
    let str_array = Type::Array {
        element_type: Box::new(Type::string(BytesSize::Fixed(5))),
        length: Some(Length::Fixed(2)),
        size: Some(BytesSize::new(10)),
    };
    let out = str_array.write(&json!(["hello", "world"])).unwrap();
    assert_eq!("helloworld".as_bytes().view_bits::<Msb0>(), out);
    assert!(str_array.write(&json!(["hello", "world", ".."])).is_err());
    assert!(str_array.write(&json!(["hello", "world1"])).is_err())
}

#[test]
fn test_write_str_or_bin() {
    let bin = Type::bin(BytesSize::Fixed(5));
    let out = bin.write(&json!([1, 2, 3, 4, 5])).unwrap();
    assert_eq!([1u8, 2, 3, 4, 5].view_bits::<Msb0>(), out);

    let s = Type::string(BytesSize::Fixed(10));
    let out = s.write(&json!("HelloWorld")).unwrap();
    assert_eq!("HelloWorld".as_bytes().view_bits::<Msb0>(), out);
}

#[test]
fn test_write_num() {
    let t_i8 = Type::int8();
    let out = t_i8.write(&json!(100)).unwrap();
    assert_eq!([100u8].view_bits::<Msb0>(), out);
    assert!(t_i8.write(&json!(256)).is_err());
    assert!(t_i8.write(&json!(-129)).is_err());

    let t_u64 = Type::uint64(Endian::Big);
    let out = t_u64.write(&json!(123456)).unwrap();
    assert_eq!(123456u64.to_be_bytes().view_bits::<Msb0>(), out);
    assert!(t_u64.write(&json!(-1)).is_err());
    assert!(t_u64.write(&json!(-5.0)).is_err());

    let t_f32 = Type::float32(Endian::Big);
    let out = t_f32.write(&json!(100.0)).unwrap();
    assert_eq!(100.0f32.to_be_bytes().view_bits::<Msb0>(), out);
    assert!(t_f32.write(&json!(f32::MAX as f64 * 2.0)).is_err());
}

#[test]
fn test_write_magic() {
    let magic = Type::magic(&[1, 2, 3]);
    let out = magic.write(&json!([1, 2, 3])).unwrap();
    assert_eq!([1u8, 2, 3].view_bits::<Msb0>(), out);
    assert_eq!(
        [1u8, 2, 3].view_bits::<Msb0>(),
        magic.write(&json!([1, 2, 3, 4])).unwrap()
    );
    assert_eq!(
        [1u8, 2, 3].view_bits::<Msb0>(),
        magic.write(&json!("test")).unwrap()
    );
}

#[test]
fn test_read_array() {
    let mut array = Type::new_array(Type::new_struct(vec![
        Field::new("id", Type::uint8()),
        Field::new(
            "data",
            Type::string(BytesSize::by_enum(
                "id",
                range_map! {
                    '1' as i64 => 1,
                    '2' as i64 => 2,
                    '3' as i64 => 3
                },
            )),
        ),
    ]));
    let (a, d) = array.read(b"333322211".view_bits()).unwrap();
    assert_eq!(d.len(), 0);
    assert_eq!(
        a,
        serde_json::json!([
            { "id": '3' as u8, "data": "333" },
            { "id": '2' as u8, "data": "22" },
            { "id": '1' as u8, "data": "1" }
        ])
    );
    assert_eq!(array.read(b"".view_bits()).unwrap().0, json!([]));

    if let Type::Array { length, .. } = &mut array {
        *length = Some(Length::Fixed(1));
    }
    assert_eq!(
        array.read(b"333322211".view_bits()).unwrap(),
        (
            json!([{ "id": '3' as u8, "data": "333" }]),
            b"22211".view_bits()
        )
    );

    if let Type::Array { length, .. } = &mut array {
        *length = Some(Length::Fixed(4));
    }
    assert!(array.read(b"333322211".view_bits()).is_err());
}

#[test]
fn test_read_enum() {
    let _enum = Type::new_struct(vec![
        Field::new("key", Type::uint8()),
        Field::new(
            "value",
            Type::new_enum(
                "key",
                range_map!(
                    1 => Type::string(BytesSize::Fixed(5)),
                    2 => Type::bin(BytesSize::Fixed(5)),
                    3 => Type::uint32(Endian::Big)
                ),
            ),
        ),
    ]);
    let array = Type::new_array(_enum);

    let data = b"\x01hello\x02world\x03\x00\x00\x00\xff";
    assert_eq!(
        array.read(data.view_bits()).unwrap(),
        (
            json!([
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
            ]),
            [0u8; 0].view_bits()
        )
    );
}

#[test]
fn test_read_write() {
    let message = Type::new_struct(vec![
        Field::new("head", Type::magic(b"##")),
        Field::new("command", Type::uint8()),
        Field::new("device_id", Type::string(BytesSize::Fixed(17))),
        Field::new("version", Type::uint8()),
        Field::new("crypto_type", Type::uint8()),
        Field::new("data_len", Type::uint16(Endian::Big)),
        Field::new(
            "data",
            Type::Enum {
                by: "command".to_string(),
                map: range_map! {
                    1 => Type::new_struct(vec![
                        Field::new("datetime", Type::bin(BytesSize::Fixed(6))),
                        Field::new("number", Type::uint16(Endian::Big)),
                        Field::new("sim_id", Type::string(BytesSize::Fixed(20))),
                    ]),
                    2 => Type::new_struct(vec![
                        Field::new("datetime", Type::bin(BytesSize::Fixed(6))),
                        Field::new("number", Type::uint16(Endian::Big)),
                        Field::new("infos", Type::new_array(Type::new_struct(vec![
                            Field::new("info_type", Type::uint8()),
                            Field::new("info", Type::new_enum("info_type", range_map! {
                                1 => Type::new_struct(vec![
                                    Field::new("protocol", Type::uint8()),
                                    Field::new("mil_status", Type::uint8()),
                                    Field::new("support_status", Type::uint16(Endian::Big)),
                                    Field::new("ready_status", Type::uint16(Endian::Big)),
                                    Field::new("vin", Type::string(BytesSize::Fixed(17))),
                                    Field::new("scin", Type::string(BytesSize::Fixed(18))),
                                    Field::new("cvn", Type::string(BytesSize::Fixed(18))),
                                    Field::new("iupr", Type::string(BytesSize::Fixed(36))),
                                    Field::new("code_len", Type::uint8()),
                                    Field::new("code_list",Type::new_array_with_length(Type::uint32(Endian::Big), Length::by_field("code_len")))
                                ]),
                                2 => Type::new_struct(vec![
                                    Field::new("speed", Type::uint16(Endian::Big)),
                                    Field::new("atmospheric_pressure", Type::uint8()),
                                    Field::new("torque", Type::uint8()),
                                    Field::new("friction_torque", Type::uint8()),
                                    Field::new("engine_speed", Type::uint16(Endian::Big)),
                                    Field::new("engine_fuel_flow", Type::uint16(Endian::Big)),
                                    Field::new("scr_nox_up", Type::uint16(Endian::Big)),
                                    Field::new("scr_nox_down", Type::uint16(Endian::Big)),
                                    Field::new("reactant", Type::uint8()),
                                    Field::new("air_intake", Type::uint16(Endian::Big)),
                                    Field::new("scr_temp_in", Type::uint16(Endian::Big)),
                                    Field::new("scr_temp_out", Type::uint16(Endian::Big)),
                                    Field::new("dpf_pressure", Type::uint16(Endian::Big)),
                                    Field::new("engine_coolant_temp", Type::uint8()),
                                    Field::new("oil_volume", Type::uint8()),
                                    Field::new("pos_invalid", Type::BOOL_BIT),
                                    Field::new("pos_south", Type::BOOL_BIT),
                                    Field::new("pos_east", Type::BOOL_BIT),
                                    Field::new("skip", Type::Uint8{ unit: Unit::new(Endian::Big, BitSize(5))}),
                                    Field::new("longitude", Type::uint32(Endian::Big)),
                                    Field::new("latitude", Type::uint32(Endian::Big)),
                                    Field::new("mileage", Type::converter(Type::uint32(Endian::Big), "self * 0.1", "floor(self * 10)")),
                                ]),
                                130 => Type::new_struct(vec![
                                    Field::new("absorption_coefficient", Type::uint16(Endian::Big)),
                                    Field::new("opaque", Type::uint16(Endian::Big)),
                                    Field::new("pm", Type::uint16(Endian::Big)),
                                ])
                            })),
                        ]))),
                    ])
                },
                size: Some(BytesSize::new("data_len")),
            },
        ),
        Field::new("check", Type::checksum(Checksum::Xor, "command")),
    ]);

    let login = [
        35, 35, 1, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 48, 49, 1, 1, 0, 28,
        21, 12, 17, 14, 54, 1, 0, 3, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53,
        54, 55, 57, 56, 48, 30,
    ];
    let (msg, _) = message.read(login.view_bits()).unwrap();
    assert_eq!(
        msg,
        json!({
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
            "check": [30]
        })
    );
    assert_eq!(message.write(&msg).unwrap().as_raw_slice(), login);
    assert_eq!(
        message
            .write(&json!({
                "command": 1,
                "device_id": "12345678901234501",
                "version": 1,
                "crypto_type": 1,
                "data": {
                    "datetime": [21, 12, 17, 14, 54, 1],
                    "number": 3,
                    "sim_id": "12345678901234567980"
                }
            }))
            .unwrap()
            .as_raw_slice(),
        login
    );

    let info = [
        35, 35, 2, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 48, 49, 1, 1, 0,
        162, 21, 12, 17, 15, 2, 35, 0, 4, 2, 1, 0, 4, 128, 129, 0, 40, 0, 120, 16, 44, 16, 64, 22,
        0, 200, 35, 128, 35, 160, 0, 130, 54, 37, 32, 0, 25, 240, 160, 0, 27, 119, 64, 0, 0, 0,
        100, 130, 7, 208, 0, 210, 0, 220, 1, 1, 1, 23, 0, 24, 0, 49, 50, 51, 52, 53, 54, 55, 56,
        57, 48, 49, 50, 51, 52, 53, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53,
        54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56,
        57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49,
        50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 3, 0, 0, 0, 25, 0, 0, 0, 26, 0, 0, 0, 27,
        184,
    ];

    let (msg, _) = message.read(info.view_bits()).unwrap();
    assert_eq!(
        msg,
        json!({
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
                            "mileage": 10.0,
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
            "check": [184]
        })
    );
    assert_eq!(message.write(&msg).unwrap().as_raw_slice(), info);
    assert_eq!(
        message
            .write(&json!({
                "command": 2,
                "device_id": "12345678901234501",
                "version": 1,
                "crypto_type": 1,
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
                                "mileage": 10.0,
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
                                "code_list": [25, 26, 27]
                            }
                        }
                    ]
                }
            }))
            .unwrap()
            .as_raw_slice(),
        info
    );
}

#[test]
fn test_read_write_encrypt() {
    let sk = RsaPrivateKey::new(&mut OsRng, 1024).unwrap();
    let pk = sk.to_public_key();
    let sk_pem = sk.to_pkcs1_pem(LineEnding::LF).unwrap().to_string();
    let pk_pem = pk.to_pkcs1_pem(LineEnding::LF).unwrap();

    let message = Type::new_struct(vec![
        Field::new("head", Type::magic(b"##")),
        Field::new("command", Type::uint8()),
        Field::new("device_id", Type::string(BytesSize::Fixed(17))),
        Field::new("version", Type::uint8()),
        Field::new("crypto_type", Type::uint8()),
        Field::new("data_len", Type::uint16(Endian::Big)),
        Field::new(
            "data",
            Type::Encrypt {
                size: Some(BytesSize::new("data_len")),
                on_read: SecureKey::rsa_pkcs1_pem(sk_pem, true, Hasher::None),
                on_write: SecureKey::rsa_pkcs1_pem(pk_pem, false, Hasher::None),
                inner_type: Box::new(Type::new_enum(
                    "command",
                    range_map! {
                        1 => Type::new_struct(vec![
                            Field::new("datetime", Type::bin(BytesSize::Fixed(6))),
                            Field::new("number", Type::uint16(Endian::Big)),
                            Field::new("sim_id", Type::string(BytesSize::Fixed(20))),
                        ]),
                        2 => Type::new_struct(vec![
                            Field::new("datetime", Type::bin(BytesSize::Fixed(6))),
                            Field::new("number", Type::uint16(Endian::Big)),
                            Field::new("infos", Type::new_array(Type::new_struct(vec![
                                Field::new("info_type", Type::uint8()),
                                Field::new("info", Type::new_enum("info_type", range_map! {
                                    1 => Type::new_struct(vec![
                                        Field::new("protocol", Type::uint8()),
                                        Field::new("mil_status", Type::uint8()),
                                        Field::new("support_status", Type::uint16(Endian::Big)),
                                        Field::new("ready_status", Type::uint16(Endian::Big)),
                                        Field::new("vin", Type::string(BytesSize::Fixed(17))),
                                        Field::new("scin", Type::string(BytesSize::Fixed(18))),
                                        Field::new("cvn", Type::string(BytesSize::Fixed(18))),
                                        Field::new("iupr", Type::string(BytesSize::Fixed(36))),
                                        Field::new("code_len", Type::uint8()),
                                        Field::new("code_list",Type::new_array_with_length(Type::uint32(Endian::Big), Length::by_field("code_len")))
                                    ]),
                                    2 => Type::new_struct(vec![
                                        Field::new("speed", Type::uint16(Endian::Big)),
                                        Field::new("atmospheric_pressure", Type::uint8()),
                                        Field::new("torque", Type::uint8()),
                                        Field::new("friction_torque", Type::uint8()),
                                        Field::new("engine_speed", Type::uint16(Endian::Big)),
                                        Field::new("engine_fuel_flow", Type::uint16(Endian::Big)),
                                        Field::new("scr_nox_up", Type::uint16(Endian::Big)),
                                        Field::new("scr_nox_down", Type::uint16(Endian::Big)),
                                        Field::new("reactant", Type::uint8()),
                                        Field::new("air_intake", Type::uint16(Endian::Big)),
                                        Field::new("scr_temp_in", Type::uint16(Endian::Big)),
                                        Field::new("scr_temp_out", Type::uint16(Endian::Big)),
                                        Field::new("dpf_pressure", Type::uint16(Endian::Big)),
                                        Field::new("engine_coolant_temp", Type::uint8()),
                                        Field::new("oil_volume", Type::uint8()),
                                        Field::new("pos_invalid", Type::BOOL_BIT),
                                        Field::new("pos_south", Type::BOOL_BIT),
                                        Field::new("pos_east", Type::BOOL_BIT),
                                        Field::new("skip", Type::Uint8{ unit: Unit::new(Endian::Big, BitSize(5))}),
                                        Field::new("longitude", Type::uint32(Endian::Big)),
                                        Field::new("latitude", Type::uint32(Endian::Big)),
                                        Field::new("mileage", Type::converter(Type::uint32(Endian::Big), "self * 0.1", "floor(self * 10)")),
                                    ]),
                                    130 => Type::new_struct(vec![
                                        Field::new("absorption_coefficient", Type::uint16(Endian::Big)),
                                        Field::new("opaque", Type::uint16(Endian::Big)),
                                        Field::new("pm", Type::uint16(Endian::Big)),
                                    ])
                                })),
                            ]))),
                        ])
                    },
                )),
            },
        ),
        Field::new("check", Type::checksum(Checksum::Xor, "command")),
    ]);

    let mut login = vec![
        35u8, 35, 1, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 48, 49, 1, 1,
    ];
    let data = [
        21, 12, 17, 14, 54, 1, 0, 3, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53,
        54, 55, 57, 56, 48,
    ];
    let mut data = pk
        .encrypt(&mut OsRng, Pkcs1v15Encrypt, data.as_slice())
        .unwrap();
    let data_len = data.len() as u16;
    login.extend_from_slice(data_len.to_be_bytes().as_slice());
    login.append(&mut data);
    let mut check = 1;
    for i in &login[3..] {
        check ^= *i;
    }
    login.push(check);

    let login_msg = json!({
        "head": b"##",
        "command": 1,
        "device_id": "12345678901234501",
        "version": 1,
        "crypto_type": 1,
        "data_len": data_len,
        "data": {
            "datetime": [21, 12, 17, 14, 54, 1],
            "number": 3,
            "sim_id": "12345678901234567980"
        },
        "check": [check]
    });

    let (msg, d) = message.read(login.view_bits()).unwrap();
    assert_eq!(d.len(), 0);
    assert_eq!(login_msg, msg);
    let data = message
        .write(&json!({
            "command": 1,
            "device_id": "12345678901234501",
            "version": 1,
            "crypto_type": 1,
            "data": {
                "datetime": [21, 12, 17, 14, 54, 1],
                "number": 3,
                "sim_id": "12345678901234567980"
            }
        }))
        .unwrap();
    let (msg, _) = message.read(&data).unwrap();
    assert_eq!(login_msg["command"], msg["command"]);
    assert_eq!(login_msg["device_id"], msg["device_id"]);
    assert_eq!(login_msg["version"], msg["version"]);
    assert_eq!(login_msg["crypto_type"], msg["crypto_type"]);
    assert_eq!(login_msg["data"], msg["data"]);

    let mut info = vec![
        35u8, 35, 2, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 48, 49, 1, 1,
    ];
    let de_data = [
        21, 12, 17, 15, 2, 35, 0, 4, 2, 1, 0, 4, 128, 129, 0, 40, 0, 120, 16, 44, 16, 64, 22, 0,
        200, 35, 128, 35, 160, 0, 130, 54, 37, 32, 0, 25, 240, 160, 0, 27, 119, 64, 0, 0, 0, 100,
        130, 7, 208, 0, 210, 0, 220, 1, 1, 1, 23, 0, 24, 0, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48,
        49, 50, 51, 52, 53, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55,
        56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48,
        49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51,
        52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 3, 0, 0, 0, 25, 0, 0, 0, 26, 0, 0, 0, 27,
    ];
    let mut data = Vec::with_capacity(pk.size() * 2);
    let mut data_len = 0;
    for chunk in de_data.chunks(pk.size() - 11) {
        let mut d = pk.encrypt(&mut OsRng, Pkcs1v15Encrypt, chunk).unwrap();
        data_len += d.len() as u16;
        data.append(&mut d);
    }
    info.extend_from_slice(data_len.to_be_bytes().as_slice());
    info.append(&mut data);
    let mut check = 2;
    for i in &info[3..] {
        check ^= *i;
    }
    info.push(check);

    let info_msg = json!({
        "head": b"##",
        "command": 2,
        "device_id": "12345678901234501",
        "version": 1,
        "crypto_type": 1,
        "data_len": data_len,
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
                        "mileage": 10.0,
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
        "check": [check]
    });
    let (msg, d) = message.read(info.view_bits()).unwrap();
    assert_eq!(d.len(), 0);
    assert_eq!(info_msg, msg);

    let msg = json!({
        "command": 2,
        "device_id": "12345678901234501",
        "version": 1,
        "crypto_type": 1,
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
                        "mileage": 10.0,
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
                        "code_list": [25, 26, 27]
                    }
                }
            ]
        }
    });

    let data = message.write(&msg).unwrap();
    let (msg, _) = message.read(&data).unwrap();
    assert_eq!(info_msg["command"], msg["command"]);
    assert_eq!(info_msg["device_id"], msg["device_id"]);
    assert_eq!(info_msg["version"], msg["version"]);
    assert_eq!(info_msg["crypto_type"], msg["crypto_type"]);
    assert_eq!(info_msg["data"], msg["data"]);
}
