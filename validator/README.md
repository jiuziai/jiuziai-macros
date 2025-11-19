# 自定义验证器

## 1.需求简述

- 宏规则

```rust
use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromPrimitive, PartialEq)]
enum TestEnum {
    A,
    B,
    C,
}
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct Demo1 {
    // 此时是多条件多个组注意，组的地方传入的不是字符串而是枚举值，此时每个check都有message，如过任意一个条件不满足返回他的message，并停止后续的判断，分组的枚举必须实现Debug, Clone, Serialize, Deserialize, PartialEq这五个trait，否则会编译报错
    #[validate(check=(len(max=20,message="此条件的错误消息"),regex=(pattern="^[a-Z]+$",message="此条件的消息")),group={TestEnum::Unit1,TestEnum::Unit2}
    )]
    a: String,
    // 此时是多个条件，但是条件没有对应的message，此时属于满足一个条件就可以通过，所有条件都不满足是，返回外部统一的message，注意，如过check内部有messge，则此忽略此check的message，全部不通过才返回外部消息，并停止后续判断，满足一个的时候，就终止此字段的后续判断，进入下一个字段的校验
    #[validate(check=(range(min=99999),func(ident="func1"), message="必须同时满足所有check"))]
    b: i32,
    // 此时遇到regex不通过则立即返回对应的message，停止后续判断，size是判断数组大小，注意，每个check都有他们适配的数据类型
    #[validate(check=(size(min = 1),regex(pattern="^0-9$",message="只能是数字")),message="不能是数字",group={TestEnum::Unit1,TestEnum::Unit2})]
    c: Vec<String>,
    // 此时要对嵌套结构体进行校验，如过字段是结构体、结构体数组则必须实现Debug, Clone, Serialize, Deserialize, Validate这五个trait，否则会编译报错，并且如过Demo2内部有validate属性，也会进行递归校验，任意一层不通过都，错误信息立即返回，并停止所有后续綖
    #[validate(...)]
    d: Demo2,
    // 此时不能为空且，进行递归
    #[validate(check(require(message = "不能为空")))]
    e: Option<Demo2>,
    // 此时如果Option有值，则对值进行校验，如果None则跳过校验
    #[validate(check(len(min = 3, message = "长度不能小于3")))]
    e: Option<String>,
    // 此时枚举值只能是TestEnum中的值，否则报错,字段的值是枚举的话，必须要实现Debug, Clone, Serialize, Deserialize,FromPrimitive这五个trait，否则会编译报错
    #[validate(check(enums=(ident="TestEnum",message="必须是枚举值之一")))]
    f: TestEnum,
    // 此时枚举值只能是TestEnum中的A或B，否则报错并且要递归
    #[validate(check(enums=(list={TestEnum::A,TestEnum::B},message="必须是枚举值之一")))]
    f: Vec<TestEnum>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct Demo2 {
    #[validate(...)]
    x: String,
    #[validate(...)]
    y: i32,
    #[validate(...)]
    z: Vec<Demo3>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Demo3 {
    #[validate(...)]
    o: String,
    #[validate(...)]
    p: i32,
}

pub fn func1(v: &i32) -> bool {
    *v % 2 == 0
}

```

- 特性

```rust
use num_enum::FromPrimitive;
/// 这是我验证器派生宏的特性
pub trait Validate {
    type Group: FromPrimitive;
    // 无视分组验证所有字段
    fn check(&self) -> Result<bool, String>;
    // 验证含有该分组的所有字段
    fn check_group(&self, group: Self::Group) -> Result<bool, String>;
}
```

- `num_enum::FromPrimitive` 说明

```rust
use num_enum::FromPrimitive;

#[derive(Debug, FromPrimitive)]
#[repr(u8)]
enum MyEnum {
    A = 1,
    B = 2,
    C = 3,
}

fn main() {
    let val: u8 = 2;
    // TryFrom<u8> is implemented for enums with FromPrimitive
    if let Ok(e) = MyEnum::try_from(val) {
        println!("包含: {:?}", e);
    } else {
        println!("不包含");
    }
}
```

```toml
[dependencies]
num_enum = "0.7.5"

```

## 2.需求大纲

  1.可以分组验证  
  2.可以多条件验证  
  3.可以多模式验证  
  4.可以深度验证  
  5.返回自定义消息  

## 3.详细说明

### 1.分组验证

- 可以传入外部字符串(&str)，或者实现了Debug, Clone, Serialize, Deserialize, PartialEq这五个trait的枚举

### 2.多条件验证

- check内部可以有多个check，每个check都可以自定义message，check如过有多个参数，可以只有一个，比如，大小，长度等等

### 3.多模式验证

- 通过是否有外部message判断模式，如过如过有外部message，就是any模式，any模式忽略内部message，只要有一个check满足，就算此字段验证通过，全部不通过则返回外部消息

### 4.深度验证

- 如过结构体上有#[validate]，则进行深度验证，如过深度校验遇到错误，立即返回不进行任何后续校验
- 若果是数组结构体，则可以加size的check验证当前字段的大小，如过是Option<结构体>，则需要加required才能让size进行校验

### 5.返回自定义消息

- 所有验证错误消息必须手动输入，验证器不输出和包装修改任何校验不通过的错误消息，全部由开发者手动输入，校验器也不会修改开发者定义的错误消息
- 错误消息直接已String返回，不需要用其他包装类型

## 4.需要满足的check

- len - 字符串类长度校验
- range - 数值类大小校验
- size 数组类大小校验
- no_space - 不能有空格
- not_empty - 不能是空字符串或者空数组
- not_blank - 去掉空格制表符等之后不能是空字符串
- fucn - 自定义校验函数
- regex - 正则校验，正则校验同时要兼容常量规则
- enum - 枚举的校验

## 5.项目结构说明
- 因为派生宏不能和lib同时导出，所有有部分代码需要放到libs/src/validation内，派生宏的代码放在validator/src/内