# bin2json

## 目标功能

- [x] 将二进制数据转化为指定格式的数据或JSON
- [x] 数据格式定义可以从文本反序列化或序列化为指定格式的文本
- [x] 将指定数据或JSON按指定格式转为二进制数据
- [ ] 支持数据加/解密、签名/验证
- [ ] 数据验证
- [ ] 数据简单计算和格式转换

## 使用示例

```rust
use bin2json::{Type, ReadBin, WriteBin};
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
    						"3": { "type": "Float32" }
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
    0, 14,
    0, 3,
    1, 0, 100,
    2, 104, 101, 108, 108, 111,
    3, 4, 3, 2, 1,
    3, 2, 1,
];

let (msg, d) = message.read_to_json(data.view_bits()).unwrap();
assert_eq!(
	serde_json::json!({
        "head": [1, 2, 3],
        "field": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        "array_size": 14,
        "array_len": 3,
        "array": [
            { "ty": 1, "value": 100u16 },
            { "ty": 2, "value": b"hello" },
            { "ty": 3, "value": f32::from_be_bytes([4, 3, 2, 1]) }
        ],
        "tail": [3, 2, 1]
    }),
    msg
);
assert_eq!([0u8; 0].view_bits::<Msb0>(), d);
assert_eq!(data, message.write_json(&msg).unwrap().as_raw_slice());

let msg = serde_json::json!({
    "head": [1, 2, 3],
    "field": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    "array": [
        { "ty": 1, "value": 100u16 },
        { "ty": 2, "value": b"hello" },
        { "ty": 3, "value": f32::from_be_bytes([4, 3, 2, 1]) }
    ],
    "tail": [3, 2, 1]
});
assert_eq!(data.view_bits::<Msb0>(), message.write_json(&msg).unwrap());
```

