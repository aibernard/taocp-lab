---
applyTo: "problems/**/*.rs,lab/**/*.rs"
---

# taocp-lab 的 Rust 约定

## 硬约束

- **零外部依赖**。`problems` 与 `lab` 都只用标准库。
  训练支架不该被依赖解析、版本冲突和网络卡住。需要随机数就用 `problems::Rng`。
- **edition 2024**。注意 `gen` 已是保留字——变量/闭包**不能**叫 `gen`，
  要叫 `generate` / `make_inputs`。
- 提交前必须 `cargo test` 与 `cargo clippy --all-targets` 全绿。
- 中文注释用**全角引号**「」或 「」，不要用 ASCII `"`——否则字符串字面量会被截断。

## `problems` crate

新增题目流程见 `.github/prompts/lab-new.prompt.md`。要点：

1. 每题一个文件：`problems/src/pXXXX_short_name.rs`
2. 在 `problems/src/lib.rs` 里 `pub mod pXXXX_short_name;`
3. 在 `for_each_problem!` 宏里加一行 `$crate::pXXXX_short_name::YourType,`
   —— 加了这一行，`lab` 的 CLI、`lab list`、`lab due`、`lab stats` 会**自动**认得它。

### `brute()` 与 `fast()` 的关系

- `brute()` 由**用户亲手写**。AI 严禁代写（见 `copilot-instructions.md`）。
- `fast()` 是参考最优解，由 AI 写。
- 两者输出必须能被 `equiv()` 判定为等价。**多解题**务必重写 `equiv()`，
  或者像 `p0001` 那样把输出规范化成唯一形式。

### 边界设置

- `brute_ceiling()`：暴力解还算可行的最高档位。超出的档位，差分测试会跳过 `brute`。
- `brute_ops()`：暴力解运算量的**公式化估计**，用于在不实际运行的前提下画爆炸曲线。
  暴力解不是 `O(N²)` 时务必重写。
- `golden()`：暴力解永远跑不到的边界（如 `u64` 溢出临界点）用期望值钉死。

## `lab` crate

- 所有终端表格用 `ui::pad` / `ui::pad_left` 对齐，**不要**用 `{:<8}`——
  中文是双宽字符，`{:<8}` 按字符数补齐必然错位。
- 颜色只在 `stdout` 是终端时输出（`ui::color_on()` 已处理）。
- 台账是 append-only 的 `records/ledger.tsv`，调度状态由 `ledger::replay()` 重放推导，
  **不要**另建状态文件。
