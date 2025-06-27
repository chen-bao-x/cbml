# Optional

?<T> // optional<T> defaltvalue is none `?string = none`

示例:
name: ?string
age: ?number 
arr: ?array<any>
arr_2: ?array<?number>
structure: ?{ name:string,age:number }
union_: ? 1 | 2 | 3
union_: ? | 1 | 2 | 3
union_:  1 | 2 | 3 | none

name_2: ?string = none // default is none 
name_3: ?string = "hello" // default is "hello", name_3 = default