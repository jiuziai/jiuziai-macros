use proc_macro2::TokenStream;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// ident的类型，只允许基本整型，字符串，布尔型，大数（rust_decimal），时间类型（chrono），枚举，Vec，HashSet，HashMap，自定义结构体，以及Option可选类型
///
/// 自定义结构体必须实现 Validate trait，该trait定义如下：
/// ```rust
/// pub trait Validate {
///     type Group: PartialEq;
///     fn check(&self) -> Result<bool, String>;
///     fn check_group(&self, group: Self::Group) -> Result<bool, String>;
/// }
/// ```
/// 超出上述类型范围的 ident 会在编译时报错，以保证类型安全，没有实现上述 trait 也会在编译时报错
///
/// not_empty 只能校验字符串，Vec，HashSet，HashMap 类型，不允许为空
///
/// not_blank 只能校验字符串类型，不允许全是空白字符
///
/// no_space 只能校验字符串类型，不允许包含空白字符
///
/// regex 只能校验字符串类型，必须是合法的正则表达式Regex，不是字符串，是 Regex 类型
///
/// range 只能校验整型，浮点型，大数（rust_decimal），时间
///
/// size 只能校验字符串，Vec，HashSet，HashMap 类型
///
/// within 和 exclude 可以校验上述所有类型，但必须保证 values 中的类型和 ident 的类型一致
///
/// deep 只能校验自定义结构体类型或者Vec,HashSet，HashMap 嵌套自定义结构体类型，同时只允许单层嵌套，多层嵌套会在编译时报错
///
/// required 只能校验Option<T> 可选类型,用于标记该字段不能为None,否则会报错
///
/// func 可以校验所有类型，用户需要传入一个闭包函数，函数的参数类型必须和 ident 的类型一致，返回值必须是 bool 类型，true 表示通过校验，false 表示不通过校验
///
/// 对于Option<T> 可选类型，会自动忽略 None 的值进行校验，只对 Some(v) 的 v 进行校验,但是如果 required 标记了该字段，则校验该字段不能为None，且会进行后续的校验
///
/// 当顶层有message的时候，各个规则的message内不允许有message，否则会在编译时报错
///
/// 当顶层没有message的时候，各个规则的message必须有message，否则会在编译时报错
///
/// 当顶层有message的时候，多个条件满足一个即可通过校验，全部不满则返回顶层的message
///
/// 当顶层没有message的时候，必须慢则所有条件，遇到不满足的条件则返回该条件的message
///
/// group 必须是枚举类型，且该枚举类型必须实现 PartialEq 特征,调用 check_group 方法时传入对应的枚举值进行分组校验，如过结构体内的 group 没有对应的枚举值，则编译时报错
///
/// 所有检验的返回message，均使用用户定义的message，不允许派生宏生成或修改验证message
///
/// MetaData 结构体用于存储字段的验证元数据
pub struct MetaData<A, B, C, D>
where
    A: Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de>,
    B: Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de>,
    C: Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de>,
    D: Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de>,
{
    pub field: String,
    pub ident: String,
    pub func: Option<FuncOptions<A>>,
    pub not_blank: Option<BoolOptions>,
    pub not_empty: Option<BoolOptions>,
    pub no_space: Option<BoolOptions>,
    pub range: Option<RangeOptions>,
    pub regex: Option<Regex>,
    pub required: Option<BoolOptions>,
    pub size: Option<SizeOptions>,
    pub within: Option<VecOptions<B>>,
    pub exclude: Option<VecOptions<C>>,
    pub deep: Option<BoolOptions>,
    pub message: Option<String>,
    pub group: Option<Vec<D>>,
}

pub struct VecOptions<T>
where
    T: Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de>,
{
    pub values: Vec<T>,
    pub message: Option<String>,
}
pub struct BoolOptions {
    pub message: Option<String>,
}
pub struct FuncOptions<T>
where
    T: Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de>,
{
    pub ident: Box<dyn Fn(&T) -> bool + Send + Sync>,
    pub message: Option<String>,
}
pub struct RangeOptions {
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub message: Option<String>,
}

pub struct SizeOptions {
    pub min: Option<u64>,
    pub max: Option<u64>,
    pub message: Option<String>,
}
