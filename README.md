
发布与 CI 建议

1. 请先确保根目录下 `.gitignore` 包含 `target/`，并且没有将构建产物提交到仓库。若已提交，请先移除缓存：

```bash
git rm -r --cached target
git commit -m "chore: remove tracked build artifacts"
```

2. 在每个子 crate 的 `Cargo.toml` 填写 `license`、`readme`、`repository`、`description` 等元信息（现仓库中已有占位字段）。

3. 在 GitHub 上创建仓库（单个 workspace 仓库即可包含两个 crate），然后 push：

```bash
git remote add origin git@github.com:<yourname>/jiuziai-crates.git
git branch -M main
git push -u origin main
```

4. 打 tag 发布版本（推荐）：

```bash
git tag v0.1.0
git push origin v0.1.0
```

5. 若发布到 crates.io：先发布 `jiuziai-libs`（运行时库），再发布 `jiuziai-macros`（proc-macro 依赖通常不互相冲突）。使用：

```bash
cargo publish -p jiuziai-libs
cargo publish -p jiuziai-crates
```

CI 建议（GitHub Actions）：在 push/PR 触发下运行：

- `cargo test --workspace`
- `cargo fmt -- --check`
- `cargo clippy -- -D warnings`

贡献与联系

欢迎提交 issue 与 PR。贡献前请运行测试并遵循仓库风格（`rustfmt`）。若你同意我把仓库推到你的 GitHub，请回复：

- 你的 GitHub 用户名或仓库 URL（例如 `github.com/yourname/jiuziai-crates`），
- 以及确认 license（例如 `MIT OR Apache-2.0`）。

---
