# array 
数组

[1, 2, 3]
以一个 [ 开始, 以一个 ] 结束.
[1, 2, 3] 和 [1, 2, 3, ] 都是合法的写法.

数组的每个元素都需要是相同类型的元素.

如果希望每个元素的类型不同, 则可以使用 array<any> 类型.

```typedef 
arr: array<any>
arr: [any]
```
```
arr = [1, "a", true, { name: "hello" } ]
```

示例 1:
array<number>
[number]
```typedef 
arr: array<number>
arr: [number]
```
```
arr = [1, 2, 3, 4, 5]
```

示例 2:
array<Person>
[Person]
```typedef 
struct Person {
    name: string
}
arr: array<Person>
arr: [Person]
```
```
arr = [ 
    { name: "hello" },
    { name: "world" },
]
```

