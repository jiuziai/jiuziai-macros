# jiuziai-macros

**jiuziai-macros** 是一个面向 Rust 的声明式验证器工作区（workspace），包含两个 crate：

- `jiuziai-macro-libs`：运行时 helpers 与 `Validate` trait 的实现（非宏 crate，可在运行时直接调用）。
- `jiuziai-validator`：基于 `proc-macro` 的 `#[derive(Validate)]` 派生宏，自动为结构体生成字段级校验逻辑。

设计目标

- 提供一套声明式、可组合的字段校验规则（长度、范围、集合大小、正则、空白、自定义函数、枚举校验等）。
- 支持 `Option<T>` / `Vec<T>` / 嵌套结构体的递归校验与分组（group）机制。
- 支持 any/all 两种消息策略：外层 message（any）或内层逐项 message（all，缺失时宏扩展报错）。

仓库结构

```
./
├─ Cargo.toml             # workspace manifest
├─ README.md              # 本文档（仓库根）
├─ libs/                  # jiuziai-macro-libs（运行时 helpers）
│  ├─ Cargo.toml
│  └─ src/
└─ validator/             # jiuziai-validator（proc-macro 派生宏）
   ├─ Cargo.toml
   └─ src/
```

快速上手

开发阶段（从 Git 仓库引用）

```toml
[dependencies]
jiuziai-validator = { git = "https://github.com/<yourname>/jiuziai-macros.git", package = "jiuziai-validator", branch = "main" }
```

发布到 crates.io 后（推荐使用 tag 并指定版本）

```toml
[dependencies]
jiuziai-validator = "0.1.0"
```

安装与使用（crates.io / git）

从 crates.io 安装（推荐稳定依赖）：

```toml
[dependencies]
jiuziai-macro-libs = "0.0.1"
jiuziai-validator = "0.0.1"
```

使用 Git 仓库中的最新版（开发版本）：

```toml
[dependencies]
jiuziai-validator = { git = "https://github.com/jiuziai/jiuziai-macros.git", package = "jiuziai-validator", branch = "main" }
```

示例：派生宏 `Validate`

```rust
use jiuziai_validator::Validate;

#[derive(Validate)]
struct Person {
    // 长度校验，all 模式（内层 check 必须提供 message）
    #[validate(check(len(min = 1, message = "name required")))]
    name: String,

    // 数值范围校验
    #[validate(check(range(min = 0, max = 150, message = "invalid age")))]
    age: i32,
}

fn main() {
    let p = Person { name: "".into(), age: 10 };
    let res = jiuziai_macro_libs::validation::Validate::check(&p);
    assert!(res.is_err());
}
```

核心概念与行为

- 校验项（checks）：通过 `#[validate(...)]` 在字段上声明一组 `check(...)`，每个 `check` 为一类规则（`len`、`range`、`regex` 等）。
- 模式：
  - any 模式（外层 `message = "..."` 存在）：内部多个 `check` 只要有任意一项通过即视为字段通过；若全部失败则返回外层 message。
  - all 模式（无外层 message）：每个内部 `check` 必须提供 `message`，缺失会在宏展开时报 `compile_error!`（防止运行时出现无提示的错误）。
- 分组（group）：字段可被标注到某个组（字符串或实现了所需 trait 的枚举），`check_group` 只对匹配组的字段执行。
- 递归校验：当字段类型实现 `Validate`（或为 `Option<T>`/`Vec<T>` 且内层类型实现 `Validate`）时，会递归调用 `check()`。宏会避免在原始数值/字符串等类型上错误生成 `.check()` 调用。

支持的 check（概览）

- `len(min=?, max=?, message="...")`：字符串长度（字符数）校验。
- `range(min=?, max=?, message="...")`：数值范围校验（内部使用 i128 进行比较）。
- `size(min=?, max=?, message="...")`：集合长度校验（`.len()`）。
- `no_space(message="...")`：不允许包含空白字符。
- `not_empty(message="...")`：不可为空字符串或空集合。
- `not_blank(message="...")`：裁剪后不可为空字符串。
- `func(ident="path::to::fn", message="...")`：调用用户函数 `Fn(&T) -> bool`。
- `regex(pattern="..", message="...")`：正则匹配（支持常量字符串）。
- `enums(...)`：
  - `ident = "TypeName"`：适用于字段为原始值（如 `u8`），通过 `TryFrom<Prim>`（例如由 `num_enum::FromPrimitive` 派生）判断是否为枚举值。示例：`enums=(ident="MyEnum", message="...")`。
  - `list = {Type::A, Type::B}`：适用于字段类型是枚举，值在列表内则通过。
- `require(message="...")`：用于 `Option<T>` 强制非空。

`num_enum` / TryFrom 示例

```rust
use num_enum::FromPrimitive;

#[derive(Debug, FromPrimitive)]
#[repr(u8)]
enum MyEnum { A = 1, B = 2, C = 3 }

let v: u8 = 2;
if let Ok(e) = MyEnum::try_from(v) {
    println!("enum: {:?}", e);
}
```

运行时 trait（位于 `jiuziai-macro-libs`）

```rust
pub trait Validate {
    type Group: PartialEq;
    fn check(&self) -> Result<bool, String>;
    fn check_group(&self, group: Self::Group) -> Result<bool, String>;
}
```

发布与 CI 建议

1. 请先确保根目录下 `.gitignore` 包含 `target/`，并且没有将构建产物提交到仓库。若已提交，请先移除缓存：

```bash
git rm -r --cached target
git commit -m "chore: remove tracked build artifacts"
```

2. 在每个子 crate 的 `Cargo.toml` 填写 `license`、`readme`、`repository`、`description` 等元信息（现仓库中已有占位字段）。

3. 在 GitHub 上创建仓库（单个 workspace 仓库即可包含两个 crate），然后 push：

```bash
git remote add origin git@github.com:<yourname>/jiuziai-macros.git
git branch -M main
git push -u origin main
```

4. 打 tag 发布版本（推荐）：

```bash
git tag v0.1.0
git push origin v0.1.0
```

5. 若发布到 crates.io：先发布 `jiuziai-macro-libs`（运行时库），再发布 `jiuziai-validator`（proc-macro 依赖通常不互相冲突）。使用：

```bash
cargo publish -p jiuziai-macro-libs
cargo publish -p jiuziai-validator
```

CI 建议（GitHub Actions）：在 push/PR 触发下运行：

- `cargo test --workspace`
- `cargo fmt -- --check`
- `cargo clippy -- -D warnings`

贡献与联系

欢迎提交 issue 与 PR。贡献前请运行测试并遵循仓库风格（`rustfmt`）。若你同意我把仓库推到你的 GitHub，请回复：

- 你的 GitHub 用户名或仓库 URL（例如 `github.com/yourname/jiuziai-macros`），
- 以及确认 license（例如 `MIT OR Apache-2.0`）。

---

> 注：本 README 概述基于仓库中 `libs/src/validation/mod.rs`（运行时 helper）和 `validator/src/lib.rs`（派生宏实现）的现有实现。如需扩展示例或把 README 本地化为更完整的 API 文档，我可以继续把示例与函数注释抽取到 docs/ 或生成 rustdoc。
