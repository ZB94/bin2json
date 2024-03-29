# 更新日志

## Unreleased

## [0.7.0] 2023.02.06
### 修改
- 更新依赖版本

## [0.6.1] 2022.01.12

### 修复

1. 修复执行表达式时可能出现精度丢失的问题
2. 修复值为整数的浮点数（如：`1.0`）作为引用值时无法正常读写的问题

## [0.6.0] 2022.01.04

### 添加

- `Checksum`添加枚举值`Complement`

### 修改

- `KeyRangeMap`添加`into_iter`, `retain`, `remove`, `claer`方法

### 删除

- 现在`Type::read`方法与`Type::read_and_convert`结果一致，所以移除`Type::read_and_convert`方法
- 现在`Type::write`方法与`Type::convert_and_write`结果一致，所以移除`Type::convert_and_write`方法

## [0.5.0] 2021.12-30

### 添加

- 添加`secure::{SecureKey, Hasher, SecureError}`
- 添加`Type::Encrypt`，用于在读/写数据时对数据进行解/加密
- 添加`Type::Sign`，用于在读/写数据时对数据进行验证/签名

### 修改

- `SecureKey`现在会对数据长度超过加/解密长度的数据进行分块加/解密

## [0.4.0] 2021-12-27

### 添加

- 添加`Checksum`类型
- `Type`添加枚举值`Checksum`
- `ReadBinError`, `WriteBinError`添加枚举值`ChecksumError`

### 修改

- `Type::Magic`现在在写入时忽略输入值

### 删除

- `WriteBinError`移除枚举值`MagicError`

## [0.3.0] 2021-12-24

### 添加

- 添加`Converter`，负责数据校验和转换

### 修改

- `Type::Converter`的`on_read`和`on_write`的类型修改为`Converter`

## [0.2.0] 2021-12-24

### 添加

- 添加`Type::Convert`，用于执行额外的表达式。并添加`Type::converter`方法
- 添加`Type::convert`, `Type::read_and_convert`, `Type::convert_and_write`三个方法
- `ReadBinError`和`WriteBinError`添加`EvalExprError`

### 删除

- 删除`ReadBin`和`WriteBin`，将`read`, `write`方法移动到`Type`的实现中

## [0.1.0] 2021-12-23

- 实现二进制数据与JSON数据的互相转换
- 实现格式定义的序列化与反序列化

