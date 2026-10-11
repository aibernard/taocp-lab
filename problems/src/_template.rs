//! # PXXXX · 题目英文名 / 题目中文名
//!
//! * LeetCode XXXX · 难度
//! * 主题：<填一个能聚类的标签，如 `双指针` / `递归树/记忆化/DP`>
//! * TAOCP：<相关章节，找不到就写 —，不要硬凑>
//!
//! ## 输出规范化决定（**先想清楚再动手**）
//!
//! 这题是多解吗？如果是，写出你的规范约定：
//! <例如：约定返回字典序最小的下标对；否则必须重写 equiv()>
//!
//! ## 复制这个文件后要做的 3 件事
//!
//! 1. 重命名为 `problems/src/pXXXX_short_name.rs`
//! 2. 在 `problems/src/lib.rs` 里加 `pub mod pXXXX_short_name;`
//! 3. 在 `for_each_problem!` 宏里加 `$crate::pXXXX_short_name::YourProblem,`

use crate::{Problem, Rng, Tier};

#[derive(Default)]
pub struct YourProblem;

impl Problem for YourProblem {
    type Input = Vec<i64>;
    type Output = i64;

    const ID: &'static str = "pXXXX";
    const TITLE: &'static str = "题目英文名 / 题目中文名";
    const TOPIC: &'static str = "未分类";
    const TAOCP: &'static str = "—";

    // ================================================================
    // 🔴 下面这个函数体必须由你亲手写。AI 严禁代写。
    // ================================================================
    fn brute(&self, input: &Self::Input) -> Self::Output {
        // 契约：允许 O(N²) 甚至 O(N³)，允许又笨又慢，但**必须正确**。
        //
        // 动手前先问自己：
        //   1. 最笨的枚举方式是什么？（枚举所有对 / 所有子数组 / 所有路径）
        //   2. 我要枚举的"候选答案集合"是什么？
        //   3. 怎么判断一个候选是不是合法答案？
        let _ = input;
        todo!("请亲手实现暴力解：先别管快慢，先让它对")
    }

    /// 参考最优解（AI 负责写）。它是**对抗者**，不是给你抄的答案。
    ///
    /// AI 实现它的时候会做两件事：
    ///   1. 把真正的实现写进 `problems/src/optimal.rs`（**人类不要看的文件**）
    ///   2. 把下面这几行换成一行转发：`crate::optimal::<函数名>(input)`
    ///
    /// 在 AI 动手之前，这里保持 `todo!()` 就是对的。
    /// 关键是：**最优解绝不能躺在你的 `brute()` 下面**。
    fn fast(&self, input: &Self::Input) -> Self::Output {
        let _ = input;
        todo!("AI: 实现最优解并搬进 problems/src/optimal.rs")
    }

    /// 按档位生成随机用例。
    ///
    /// 动手前先让用户想清楚：**什么规模才会让你的暴力解崩？**
    fn generate(&self, rng: &mut Rng, tier: Tier) -> Self::Input {
        let n = match tier {
            Tier::Tiny => 5,
            Tier::Small => 20,
            Tier::Medium => 200,
            Tier::Large => 20_000,
        };
        (0..n).map(|_| rng.range(-50, 50)).collect()
    }

    /// 手写恶意边界。标上名字，失败时一眼看得出是哪类边界崩的。
    fn adversarial(&self) -> Vec<(&'static str, Self::Input)> {
        vec![
            ("空输入", vec![]),
            ("单元素", vec![7]),
            ("全零", vec![0, 0, 0]),
            ("全负数", vec![-1, -2, -3]),
            ("重复值", vec![5, 5, 5, 5]),
            ("极值", vec![i64::MAX, i64::MIN]),
            ("已排序", vec![1, 2, 3, 4, 5]),
            ("逆序", vec![5, 4, 3, 2, 1]),
        ]
    }

    fn size(&self, input: &Self::Input) -> usize {
        input.len()
    }

    /// 暴力解运算量估计。不是 O(N²) 就**必须**重写这个。
    fn brute_ops(&self, input: &Self::Input) -> u128 {
        let n = input.len() as u128;
        n * n
    }

    /// 暴力解撑到哪一档。超出的档位差分测试会跳过 brute。
    fn brute_ceiling(&self) -> Tier {
        Tier::Medium
    }

    /// 暴力解永远跑不到、但有已知答案的边界（溢出临界点等）。
    fn golden(&self) -> Vec<(&'static str, Self::Input, Self::Output)> {
        Vec::new()
    }

    /// 差分判定口。多解题必须重写。
    fn equiv(&self, a: &Self::Output, b: &Self::Output) -> bool {
        a == b
    }

    // ================================================================
    // 禁止公式。只画内存。
    // ================================================================
    fn render(&self, input: &Self::Input) -> String {
        let mut s = String::new();
        s.push_str(&format!("len = {}\n\n", input.len()));

        s.push_str(" idx |");
        for i in 0..input.len().min(16) {
            s.push_str(&format!("{i:>6}"));
        }
        s.push('\n');

        s.push_str("-----+");
        for _ in 0..input.len().min(16) {
            s.push_str("------");
        }
        s.push('\n');

        s.push_str(" val |");
        for v in &input[..input.len().min(16)] {
            s.push_str(&format!("{v:>6}"));
        }
        s.push('\n');

        s
    }
}
