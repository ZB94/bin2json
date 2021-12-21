# bin2json

## 目标功能

- [x] 将二进制数据转化为指定格式的数据或JSON
- [x] 数据格式定义可以从文本反序列化或序列化为指定格式的文本
- [ ] 将指定数据或JSON按指定格式转为二进制数据
- [ ] 支持数据加/解密、签名/验证
- [ ] 数据验证
- [ ] 数据简单计算和格式转换

## 读取数据

```rust
use bin2json::{Type, ReadBin};
use bin2json::bitvec::{BitView, Msb0};

let message: Type = serde_json::from_str(r#"{
    "type": "Struct",
    "fields": [
    	{ "name": "head", "type": "Magic", "magic": [1, 2, 3] },
    	{ "name": "field", "type": "Bin", "size": 10 },
    	{ "name": "data_len", "type": "Uint16" },
    	{ 
    		"name": "data", 
    		"type": "Array",
    		"size": "data_len", 
    		"element_type": {
    			"type": "Struct",
    			"fields": [
    				{ "name": "ty", "type": "Uint8" },
    				{ 
    					"name": "value", 
    					"type": "Enum", 
    					"by": "ty",
    					"map": {
    						"1": { "type": "Uint8" },
    						"2": { "type": "Int16" },
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
    0, 10,
    1, 100,
    2, 200, 100,
    3, 4, 3, 2, 1,
    3, 2, 1,
];

let (msg, d) = message.read_to_json(data.view_bits()).unwrap();
assert_eq!(
	serde_json::json!({
        "head": [1, 2, 3],
        "field": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        "data_len": 10,
        "data": [
            { "ty": 1, "value": 100u8 },
            { "ty": 2, "value": i16::from_be_bytes([200, 100]) },
            { "ty": 3, "value": f32::from_be_bytes([4, 3, 2, 1]) }
        ],
        "tail": [3, 2, 1]
    }),
    msg
);
assert_eq!([0u8; 0].view_bits::<Msb0>(), d);
```

