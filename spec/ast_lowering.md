
## 语义解析

符号 sym
同名符号的编号 tagid
区域 Scope（词法区域 Lexical Scope）

sym(Symbol) + tagid(usize) = Unique Identifier

根据符号, 按照当前scope及其上层scope的顺序，查询所属对应类型和tagid


## 函数符号查询

根据
+ 操作名: base
+ 参数类型列表组成： (ty1, ty2, ..., tyn)

函数名：

    fullname = base@ty1#ty2#...#tyn


| 内部实现的函数 | 外部函数导出的符号 |
|---|---|
| AFnDec | AnExtFnDec |
| 使用名字作为符号名 | 额外关联的导出符号名： sign_name |

举例：

```text
push([int], int) -> `push` + Arr(-4) + Int(-4) -> push@[i32]#i32
```

