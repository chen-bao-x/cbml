
string 以一个双引号开始, 以一个单引号结束.
"this is an string"
"this is 
an 
string 
"
字符串中允许不转义的 “换行符” “制表符” “双引号” “单引号”
如果字符串中有 “双引号” , 则可以在 string 的开始符号中添加双引号来避免冲突:
""
hello, what is your name?
my name is "hello" 
""
字符串中可以使用的转义符号:
\n -> 换行符
\r 
\t 
\u{HEX} -> unicode 示例: "\u{4F60}\u{597D}，\u{4E16}\u{754C}"  // "你好，世界"