# bin2json

## 更新日志

见[CHANGELOG](CHANGELOG.md)

## 目标功能

- [x] 将二进制数据转化为定义格式的SON值
- [x] 数据格式定义可以从文本反序列化或序列化为指定格式的文本
- [x] 将JSON值按照定义格式转为二进制数据
- [x] 数据验证
- [x] 数据简单计算和格式转换
- [x] 校验和
- [x] 支持数据加/解密、签名/验证

## 示例

### 示例列表

- [完整示例](#完整示例)
- [数值转换和校验示例](#数值转换和校验示例)

### 完整示例

```rust
use bin2json::Type;
use bin2json::bitvec::{BitView, Msb0};

let message: Type = serde_json::from_str(r#"{
    "type": "Struct",
    "fields": [
    	{ "name": "head", "type": "Magic", "magic": [1, 2, 3] },
    	{ "name": "field", "type": "Bin", "size": 10 },
    	{ "name": "array_size", "type": "Uint16" },
    	{ "name": "array_len", "type": "Uint16" },
    	{ 
    		"name": "array", 
    		"type": "Array",
    		"size": "array_size", 
    		"length": "array_len",
    		"element_type": {
    			"type": "Struct",
    			"fields": [
    				{ "name": "ty", "type": "Uint8" },
    				{ 
    					"name": "value", 
    					"type": "Enum", 
    					"by": "ty",
    					"map": {
    						"1": { "type": "Uint16" },
    						"2": { "type": "Bin", "size": 5 },
    						"3": { "type": "Float32" },
    						"4": { 
    							"type": "Converter",
    							"original_type": { "type": "Uint32" },
    							"on_read": {
    								"convert": "self / 100"
    							},
    							"on_write": {
	    							"convert": "self * 100"
    							}
    						}
    					}
    				}
    			]
		    }
    	},
    	{ "name": "tail", "type": "Magic", "magic": [3, 2, 1] }
    ]
}"#).unwrap();

let data = [
	1, 2, 3,
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    0, 19,
    0, 4,
    1, 0, 100,
    2, 104, 101, 108, 108, 111,
    3, 4, 3, 2, 1,
    4, 0, 0, 0, 200,
    3, 2, 1,
];

let (msg, d) = message.read_and_convert(data.view_bits()).unwrap();
assert_eq!(
	serde_json::json!({
        "head": [1, 2, 3],
        "field": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        "array_size": 19,
        "array_len": 4,
        "array": [
            { "ty": 1, "value": 100u16 },
            { "ty": 2, "value": b"hello" },
            { "ty": 3, "value": f32::from_be_bytes([4, 3, 2, 1]) },
            { "ty": 4, "value": 2 }
        ],
        "tail": [3, 2, 1]
    }),
    msg
);
assert_eq!([0u8; 0].view_bits::<Msb0>(), d);
assert_eq!(data, message.convert_and_write(msg).unwrap().as_raw_slice());

let msg = serde_json::json!({
    "field": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    "array": [
        { "ty": 1, "value": 100u16 },
        { "ty": 2, "value": b"hello" },
        { "ty": 3, "value": f32::from_be_bytes([4, 3, 2, 1]) },
        { "ty": 4, "value": 2 }
    ]
});
assert_eq!(data.view_bits::<Msb0>(), message.convert_and_write(msg).unwrap());
```

### 数值转换和校验示例

```rust
use bin2json::Type;
use bin2json::bitvec::{BitView, Msb0};

let ty: Type = serde_json::from_str(r#"{
	"type": "Converter",
    "original_type": { "type": "Uint32" },
    "on_read": {
    	"before_valid": "self > 100",
    	"convert": "self * 10",
    	"after_valid": "self < 5000"
    },
    "on_write": {
    	"before_valid": "self < 5000",
    	"convert": "self / 10",
    	"after_valid": "self > 100"
    }
}"#).unwrap();

// 读
assert_eq!(serde_json::json!(2000), ty.read_and_convert(200u32.to_be_bytes().view_bits::<Msb0>()).unwrap().0);
// before error
assert!(ty.read_and_convert(100u32.to_be_bytes().view_bits::<Msb0>()).is_err());
// after error
assert!(ty.read_and_convert(500u32.to_be_bytes().view_bits::<Msb0>()).is_err());

// 写
assert_eq!(200u32.to_be_bytes().view_bits::<Msb0>(), ty.convert_and_write(serde_json::json!(2000)).unwrap());
// before error
assert!(ty.convert_and_write(serde_json::json!(5000)).is_err());
// after error
assert!(ty.convert_and_write(serde_json::json!(1000)).is_err());
```

