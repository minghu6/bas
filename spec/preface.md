
## 总概

编译型的bash增强的语言

[Reference](../src/spec.rs)

## 数据结构(builtin)

### Dynamic Array: [T]

`[i32]/[u8]/[ptr]`


## 属性注解(attr)


**no_mangle:**

  1. 函数 fullname = basename,
  2. 暗示了 `unique`, 同名函数只能有一个，没有依据类型的分发（dispatch）
  3. 允许变长参数

**symbol_name**

  `@sym(xxx)`
