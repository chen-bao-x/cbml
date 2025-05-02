# union 
具体的某几个值.

union(string) naames {
    "hello" | "alex" | "张三"
}

union(string) naames_2 {
    | "hello"
    | "alex" 
    | "张三"
}

union(string) naames_2 =
    | "hello"
    | "alex" 
    | "张三"


union(string) naames_3 {
    | "hello" | "alex" | "张三"
}

union(string) naames_3 = | "hello" | "alex" | "张三"
union(any) names =
    | "hello"
    | "alex" 
    | "张三"
    | 1323



type names =  "hello" | "alex" | "张三"
type names = | "hello" | "alex" | "张三"
type names =
    | "hello"
    | "alex" 
    | "张三"
    | 1323

