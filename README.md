# cbml

我的配置文件标记语言.

string // "string"
number // -3425345.2345
boolean // true false
array<T> // array<numbber> [1, 2, 3, 4, 5] array<String> ["hello", "world"]
struct
union // union(string) "a" | "b" | "hello" union(number) 1 | 2 | 3 | 4 | 99 union(any) 123213 | "hello" | true | 123.456
// character // "a" 用 长度为 1 的 string 来代替.
any

option // name: ?string = none

key word
any struct string number boolean true false none union todo use
: = ( ) [ ] { } , \n  
todo 是写给 代码编辑器的语法检查工具看的, 代码编辑器的语法检查工具 在遇到 todo 的时候要暂时忽略这个 error.
在尝试将一个 todo 序列化为某个编程语言的具体类型是 panic: not yet implemented.
use <string> // use "/path/to/typedef.cbmltypedef"
为此 cbml 文件添加要使用的类型.
在 use 关键字的 “前面” 只能有 "空格" "tab" "换行符"

/// 文档注释
// 单行注释
/_
多行注释
_/

语法检查 类型检查
序列化与反序列化

// type def file structure

top fields

typedef:

custom type definetion

自定义类型实现起来好麻烦, 要不只支持匿名类型?

## rules

1. 如果一个自定义类型只被使用了一次, 则建议调整为 匿名类型.


cbml 文件只是数据文件,
不能使用 类型定义, 也不能使用 default 关键字. 

default 关键字:
用于告述写配置文件的人, 如果你不明白这个字段的作用, 先用默认值.
language server 自动补全时优先考虑默认值.



## cbml 文件: 
用于存放数据, 


字段 = "value"