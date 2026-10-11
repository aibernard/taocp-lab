//! # P0509 · Fibonacci Number（斐波那契数）
//!
//! * LeetCode 509 · Easy
//! * 主题：递归树 / 记忆化 / 动态规划
//! * TAOCP：Vol.1 §1.2.8 Fibonacci numbers
//!
//! ## 这是整个实验室的样板题
//!
//! 它要让你**亲眼看见**"重复子问题"长什么样。
//! 运行 `lab render p0509 --tier small` 会得到两样东西：
//!
//! 1. 朴素递归的调用树 ASCII 图，重复的子树被标上 `⟲` —— 这就是可剪枝的节点
//! 2. 每个子问题被重算的次数表 —— 你会看到 `fib(0)` 被调用了 3 次、`fib(2)` 被调用 3 次……
//!
//! 不需要任何数学公式，你就能意识到：**"把算过的结果存起来"就是全部秘密**。
//!
//! ## 关于暴力解的可运行边界
//!
//! 朴素递归是 `O(φⁿ)`。`n = 40` 就需要约 3.3 亿次调用。
//! 所以本题把 [`Problem::brute_ceiling`] 卡在 `Medium`，
//! 并且用 [`Problem::brute_ops`] 给出**精确**的调用次数公式，
//! 让支架在不实际运行的前提下照样能画出爆炸曲线。

use crate::{Problem, Rng, Tier};

#[derive(Default)]
pub struct Fibonacci;

impl Problem for Fibonacci {
    /// 输入直接就是 `n`。
    type Input = u32;
    type Output = u64;

    const ID: &'static str = "p0509";
    const TITLE: &'static str = "Fibonacci Number / 斐波那契数";
    const TOPIC: &'static str = "递归树/记忆化/DP";
    const TAOCP: &'static str = "Vol.1 §1.2.8";

    /// ⚠️ 本题的 `brute()` 是**随仓库附带的示范实现，不是你写的**。
    /// 它只用来演示"递归树 → 重复子问题 → 记忆化"这条认知路径。
    const REFERENCE: bool = true;

    /// 暴力解：教科书式朴素递归。指数爆炸的活教材。
    fn brute(&self, n: &u32) -> u64 {
        fn go(n: u32) -> u64 {
            if n < 2 {
                n as u64
            } else {
                go(n - 1) + go(n - 2)
            }
        }
        go(*n)
    }

    /// 最优解：自底向上迭代。`O(N)` 时间、`O(1)` 空间。
    ///
    /// ⚠️ 实现放在 `problems/src/optimal.rs` —— **写 brute() 之前不要打开它**。
    fn fast(&self, n: &u32) -> u64 {
        crate::optimal::p0509_fibonacci(n)
    }

    fn generate(&self, rng: &mut Rng, tier: Tier) -> u32 {
        // 各档位区间**刻意不重叠**：否则随机取值会让曲线上下抖动，
        // 破坏"越往后越慢"这个唯一要让你看见的事实。
        // Tiny 压到 ≤6，保证调用树一定画得下。
        let (lo, hi) = match tier {
            Tier::Tiny => (4, 6),
            Tier::Small => (8, 12),
            Tier::Medium => (16, 20),
            Tier::Large => (30, 36),
        };
        rng.range(lo, hi) as u32
    }

    /// 只放**暴力解跑得动**的边界。跑不动的溢出临界点放进 `golden`。
    fn adversarial(&self) -> Vec<(&'static str, u32)> {
        vec![
            ("n=0 基础情形", 0),
            ("n=1 基础情形", 1),
            ("n=2 最小递归", 2),
            ("n=3", 3),
            ("n=10 进入指数区", 10),
            ("n=20 开始肉疼", 20),
        ]
    }

    /// 溢出临界点：`fib(92)` 是 `u64` 装得下的最后一项。
    /// 朴素递归永远算不到这里，所以直接钉死期望值。
    fn golden(&self) -> Vec<(&'static str, u32, u64)> {
        vec![
            ("n=50", 50, 12_586_269_025),
            ("n=90", 90, 2_880_067_194_370_816_120),
            ("n=92（u64 上限附近）", 92, 7_540_113_804_746_346_429),
        ]
    }

    fn size(&self, n: &u32) -> usize {
        *n as usize
    }

    /// 朴素递归的**精确**调用次数：`T(n) = 2·F(n+1) − 1`。
    fn brute_ops(&self, n: &u32) -> u128 {
        let n = *n as usize;
        let (mut a, mut b) = (0u128, 1u128);
        for _ in 0..=n {
            let t = a + b;
            a = b;
            b = t;
        }
        // 循环 n+1 次后，a == F(n+1)
        2 * a - 1
    }

    /// 暴力解的可行上限：`n = 24` 约 15 万次调用，`n = 40` 就是 3.3 亿次。
    fn brute_ceiling(&self) -> Tier {
        Tier::Medium
    }

    fn render(&self, n: &u32) -> String {
        let raw = *n;
        let n = raw as usize;
        let mut s = String::new();

        s.push_str(&format!("fib({n}) 的朴素递归调用树\n"));
        s.push_str("同一个子问题被反复重算 —— 这就是超时的根源\n\n");

        if n <= 6 {
            let mut expanded = vec![false; n + 1];
            s.push_str(&format!("fib({n})\n"));
            if n >= 2 {
                expanded[n] = true;
                draw(&mut s, n - 1, "├─ ", &mut expanded);
                draw(&mut s, n - 2, "└─ ", &mut expanded);
            }
        } else {
            s.push_str(&format!(
                "（n = {n} 太大，调用树有约 {} 行，此处省略）\n",
                self.brute_ops(&raw)
            ));
        }

        // 子问题重算次数表。n 一大表格就会失控，所以设了上限。
        if n <= 12 {
            let counts = call_counts(n);
            s.push_str(&format!("\n各子问题被计算的次数（算的就是 fib({n})）\n"));
            s.push_str("   k  |");
            for k in 0..counts.len() {
                s.push_str(&format!("{k:>6}"));
            }
            s.push('\n');
            s.push_str("------+");
            for _ in 0..counts.len() {
                s.push_str("------");
            }
            s.push('\n');
            s.push_str("  次数 |");
            for c in &counts {
                s.push_str(&format!("{c:>6}"));
            }
            s.push('\n');
            let total: u64 = counts.iter().sum();
            s.push_str(&format!(
                "\n不同子问题只有 {} 个，却发生了 {} 次函数调用。\n",
                counts.len(),
                total
            ));
        }

        s.push_str("\n朴素递归总调用次数 = 2·F(n+1) − 1，随 n 指数暴涨：\n");
        for k in [10u32, 20, 30, 40] {
            let mark = if k == raw { "   ← 当前" } else { "" };
            s.push_str(&format!(
                "  n={k:<3} → {:>15} 次调用{}\n",
                fmt_u128(self.brute_ops(&k)),
                mark
            ));
        }
        if ![10u32, 20, 30, 40].contains(&raw) {
            s.push_str(&format!(
                "  n={n:<3} → {:>15} 次调用   ← 当前\n",
                fmt_u128(self.brute_ops(&raw))
            ));
        }

        s.push_str(
            "\n暴力解：每个 fib(k) 都被从零重算一遍        → O(φⁿ)\n\
             最优解：两个变量滚动，只保留最近两项        → O(N)\n\
             \n\
             ⟲ 标记的节点 = 重复子问题。把它们的结果缓存进数组，\n\
             就得到了「记忆化搜索」；再改成自底向上遍历，就是「动态规划」。\n",
        );
        s
    }
}

/// 递归地画调用树；已经展开过的子问题直接标 `⟲` 并停止下钻，
/// 模拟"如果加了缓存会发生什么"。
fn draw(s: &mut String, n: usize, connector: &str, expanded: &mut Vec<bool>) {
    s.push_str(connector);
    s.push_str(&format!("fib({n})"));

    if n >= 2 && expanded[n] {
        s.push_str("   ⟲ 重复子问题 —— 缓存可直接剪掉\n");
        return;
    }
    s.push('\n');

    if n < 2 {
        return;
    }
    expanded[n] = true;

    // 把本行的连接符转换成子行需要的前缀
    let base = connector.replace("├─ ", "│  ").replace("└─ ", "   ");
    draw(s, n - 1, &format!("{base}├─ "), expanded);
    draw(s, n - 2, &format!("{base}└─ "), expanded);
}

/// 直接模拟一遍朴素递归，统计每个子问题被调用的次数。
/// 只在 `n` 很小时调用（`n = 6` 也才 41 次调用）。
fn call_counts(n: usize) -> Vec<u64> {
    fn go(n: usize, counts: &mut [u64]) {
        counts[n] += 1;
        if n >= 2 {
            go(n - 1, counts);
            go(n - 2, counts);
        }
    }
    let mut counts = vec![0u64; n + 1];
    if n > 0 || counts.len() == 1 {
        go(n, &mut counts);
    }
    counts
}

fn fmt_u128(n: u128) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}
