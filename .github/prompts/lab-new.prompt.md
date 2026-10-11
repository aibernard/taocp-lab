---
mode: agent
description: 新题脚手架 —— 建一道题，但绝不代写暴力解
---

# 技能：新题脚手架（New Problem）

## 触发

用户说「开一道新题」「加一道 LeetCode XXX」「我想练 XX 类型」。

## 铁律

🔴 **你可以写 `fast()` / `generate()` / `adversarial()` / `golden()` / `render()`，
但绝对不能写 `brute()` 的函数体。**

正确做法：把 `brute()` 留成 `todo!()`，并写清楚它的契约注释。

```rust
fn brute(&self, input: &Self::Input) -> Self::Output {
    // 🔴 这一行必须由你亲手写。
    // 契约：允许 O(N²) 甚至 O(N³)，允许又笨又慢，但必须正确。
    // 先想：如果只能用最笨的办法，你会怎么遍历？
    todo!("请亲手实现暴力解")
}
```

## 步骤

### 1. 建文件（一条命令，别手动复制）

```bash
cargo run -p lab -- new <题号> <slug> --title "<中英双名>" --topic "<标签>" --taocp "<章节>"
```

例如：

```bash
cargo run -p lab -- new 49 group_anagrams \
    --title "Group Anagrams / 字母异位词分组" --topic "哈希/排序" --taocp "Vol.3 §6.4"
```

它会生成题目文件**并**在 `problems/src/lib.rs` 里注册好。
生成出来的 `brute()` 是 `todo!()` —— 这正是你要保持的状态。

### 2. 核对元信息

`lab new` 已经填好，你只需要检查：
- `ID`：`p` + 四位 LeetCode 题号（如 `p0070`）
- `TITLE`：中英双名
- `TOPIC`：**用能聚类的标签**（如 `哈希`、`双指针`、`递归树/记忆化/DP`），
  `lab stats` 的薄弱环节分析全靠它
- `TAOCP`：最相关的教材章节。**找不到就写 `—`，不要硬凑**

### 3. 定 `Input` / `Output`，并想清楚**规范化问题**

这是最容易埋雷的一步。逐条自问：

- [ ] 这题是**多解**吗？（任取一组解、任意顺序、任意合法答案）
      如果是 → 必须在注释里写死一个**规范化约定**，或重写 `equiv()`。
- [ ] 输出是浮点数吗？→ 需要容差比较，`PartialEq` 不够用。
- [ ] 输出是集合/列表吗？→ 需要先排序或用集合语义比较。

### 4. 定 `Tier` 的规模

| 档位 | 用途 | 要求 |
|---|---|---|
| `Tiny` | 画得下 ASCII 图 | 小到能手工验算 |
| `Small` | 差分对拍主力 | 暴力解毫秒级 |
| `Medium` | 找到分水岭 | 暴力解接近可忍受上限 |
| `Large` | 爆炸现场 | 暴力解**必须**崩，只能靠 `brute_ops()` 估算 |

### 5. 设边界钩子

- `brute_ceiling()`：暴力解撑到哪一档
- `brute_ops()`：暴力解的运算量公式（不是 `O(N²)` 就必须重写）
- `golden()`：暴力解永远跑不到、但有已知答案的边界

### 6. 注册

在 `problems/src/lib.rs` 中：

```rust
pub mod pXXXX_short_name;   // ← 加这行
```

并在 `for_each_problem!` 宏里加一行：

```rust
$crate::pXXXX_short_name::YourType,
```

加了这一行，`lab list` / `lab due` / `lab stats` / `lab diff` 会自动认得它。

### 7. 验收

```bash
cargo test                     # every_registered_problem_agrees 会立刻检查两个解是否一致
cargo clippy --all-targets
cargo run -p lab -- render pXXXX --tier tiny
```

## 收尾

告诉用户这条命令，然后把键盘交给他：

```
cargo run -p lab -- render pXXXX --tier tiny
```

> 「先看输入长什么样，然后自己写 `brute()`。写完叫我，我来当你的对手。」
