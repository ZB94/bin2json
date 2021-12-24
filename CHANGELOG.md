# 更新日志

## Unreleased

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


