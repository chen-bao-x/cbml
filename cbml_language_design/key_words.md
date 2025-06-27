# key words

string
number
boolean
true
false
none
any

struct
union

todo
use
default

: = ( ) [ ] { } , '\n'  ?  "  |

## todo 
todo 是写给 代码编辑器的语法检查工具看的, 代码编辑器的语法检查工具 在遇到 todo 的时候要暂时忽略这个 error.
在尝试将一个 todo 序列化为某个编程语言的具体类型是 panic: not yet implemented.

## use 
use <string> // use "/path/to/typedef.cbmltypedef"

为此 cbml 文件添加要使用的类型.
在 use 关键字的 “前面” 只能有 "空格" "tab" "换行符"

## Q&A

optional 和 default 是不是有点冲突了?
name: ?string = none // default is none 
name: ?string = "hello" // default is "hello"
none 和 "" 0 [] {} true false 所表达的含义是不同的, 所以不冲突.

name: ?string default = none // default is none 
name: ?string default = "hello" // default is "hello"



## default
name = default
age = default 
