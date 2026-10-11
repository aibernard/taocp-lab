//! # P0088 · Merge Sorted Array / 合并两个有序数组
//!
//! * LeetCode 88 · Easy
//! * 主题：双指针
//! * TAOCP：Vol.3 §5.2.4 Sorting by Merging
//!
//! ## 输出规范化决定（**先想清楚再动手**）
//!
//! 这题**不是多解题**：合并后的有序数组是唯一的。
//! 所以默认的 `equiv`（精确比较）就够用，不需要重写。
//!
//! 但它有一个**更隐蔽的翻译问题**——题目在 LeetCode 上要求"就地修改 nums1、
//! 不返回任何东西"。Rust 的函数签名表达不了"就地"，所以本仓库把它翻译成：
//!
//! ```text
//! Input  = (nums1 的完整缓冲区, m, nums2)   // 缓冲区长度 = m + n，后 n 个是占位 0
//! Output = 合并之后的完整数组
//! ```
//!
//! ⚠️ 这个翻译**丢掉了"就地"约束**：你可以随便另开一个数组。
//! 也就是说 `brute` 只要结果对就能过，**空间上的差别在这里根本观察不到**。
//! 这题真正的价值，只能靠 `lab bench`（时间）和 `render()`（内存布局）去感知。


use crate::{Problem, Rng, Tier};

/// 输入：`(nums1 的完整缓冲区, m, nums2)`。缓冲区长度必须是 `m + nums2.len()`。
pub type Input = (Vec<i64>, usize, Vec<i64>);
/// 输出：合并后的完整数组，长度 `m + n`。
pub type Output = Vec<i64>;

#[derive(Default)]
pub struct MergeSortedArray;

impl Problem for MergeSortedArray {
    type Input = Input;
    type Output = Output;

    const ID: &'static str = "p0088";
    const TITLE: &'static str = "Merge Sorted Array / 合并两个有序数组";
    const TOPIC: &'static str = "双指针";
    const TAOCP: &'static str = "Vol.3 §5.2.4";

    // ================================================================
    // 🔴 下面这个函数体必须由你亲手写。AI 严禁代写。
    // ================================================================
    fn brute(&self, input: &Self::Input) -> Self::Output {
        // 契约：允许又笨又慢，但必须正确。
        //
        // 这题没有"候选答案集合"要枚举，别被模板里那三问套住。
        // 动手前先问自己两句：
        //   1. 你打算把结果写到哪里？另开一个新数组，是不是最容易写对？
        //   2. 如果非要写回 nums1，你会**先**撞上什么问题？
        let (nums1, m, nums2) = input;
        let mut num = vec![];
        // println!("nums1: {:?}, m: {m}, nums2: {:?}", nums1, nums2);
        let (mut i, mut j) = (0, 0);
        while i < *m && j < nums2.len() {
            if nums1[i] <= nums2[j] {
                num.push(nums1[i]);
                i += 1;
            } else {
                num.push(nums2[j]);
                j += 1;
            }
        };
        while i < *m {
            num.push(nums1[i]);
            i += 1;
        }
        while j < nums2.len() {
            num.push(nums2[j]);
            j += 1;
        }
        num
    }

    /// 参考最优解。它是**对抗者**，不是给你抄的答案。
    ///
    /// ⚠️ 实现放在 `problems/src/optimal.rs` —— **写 brute() 之前不要打开它**。
    fn fast(&self, input: &Self::Input) -> Self::Output {
        crate::optimal::p0088_merge_sorted_array(input)
    }

    /// 按档位生成随机用例。
    ///
    /// 动手前先让用户想清楚：**什么规模才会让你的暴力解崩？**
    fn generate(&self, rng: &mut Rng, tier: Tier) -> Self::Input {
        // 两个数组都必须**已经有序**，这才是题目要的输入。
        //
        // 值域刻意压窄：让两个数组里的值大量重复、交错。
        // 否则随机数据下两个数组几乎不会重叠，
        // "等值时候取哪一边"这类边界根本测不到。
        let (m, n, span) = match tier {
            Tier::Tiny => (3, 3, 6),
            Tier::Small => (30, 30, 50),
            Tier::Medium => (300, 300, 400),
            Tier::Large => (3_000, 3_000, 4_000),
        };

        let nums2 = sorted_sample(rng, n, span);
        let mut buf = sorted_sample(rng, m, span);
        buf.resize(m + n, 0); // 尾部的 n 个占位
        (buf, m, nums2)
    }

    /// 手写恶意边界。标上名字，失败时一眼看得出是哪类边界崩的。
    fn adversarial(&self) -> Vec<(&'static str, Self::Input)> {
        vec![
            ("n=0，nums2 是空的", (vec![1, 2, 3], 3, vec![])),
            ("m=0，nums1 全是占位", (vec![0], 0, vec![1])),
            ("m=0 且单个负数", (vec![0], 0, vec![-1])),
            (
                "nums2 全部小于 nums1",
                (vec![4, 5, 6, 0, 0, 0], 3, vec![1, 2, 3]),
            ),
            (
                "nums2 全部大于 nums1",
                (vec![1, 2, 3, 0, 0, 0], 3, vec![4, 5, 6]),
            ),
            ("两个数组元素全相等", (vec![2, 2, 2, 0, 0], 3, vec![2, 2])),
            ("重复值交错", (vec![1, 1, 3, 0, 0], 3, vec![1, 3])),
            ("含负数交错", (vec![-3, -1, 0, 0], 2, vec![-2, -1])),
            ("全零", (vec![0, 0, 0, 0], 2, vec![0, 0])),
            ("i64 极值", (vec![i64::MIN, i64::MAX, 0], 2, vec![0])),
            ("m=1, n=1 最小非平凡", (vec![2, 0], 1, vec![1])),
            ("m=1 且该元素最大", (vec![9, 0, 0], 1, vec![1, 2])),
        ]
    }

    fn size(&self, (buf, _, _): &Self::Input) -> usize {
        buf.len()
    }

    /// 暴力解运算量估计。
    ///
    /// 这里假设你的暴力解是「把 nums2 一个个插进 nums1 的正确位置，
    /// 每次插入都线性找位置并搬移元素」→ `O(m·n)`。
    ///
    /// ⚠️ **如果你的写法不是这个复杂度，必须把这里改掉**，
    /// 否则 `lab bench` 的"估算运算量"列会漏你。
    /// （实测耗时列不受影响，它永远是真的。）
    fn brute_ops(&self, (_, m, nums2): &Self::Input) -> u128 {
        (*m as u128) * (nums2.len() as u128)
    }

    /// 暴力解撑到哪一档。超出的档位差分测试会跳过 brute。
    fn brute_ceiling(&self) -> Tier {
        Tier::Medium
    }

    /// 题目原文里给的那三个例子。它们是**规范**，不是提示。
    ///
    /// 顺便带来一个副作用：它让 `fast()` 在你写完 `brute()` 之前
    /// 就能被独立验证（`cargo test` 里的 `every_golden_case_holds`）。
    fn golden(&self) -> Vec<(&'static str, Self::Input, Self::Output)> {
        vec![
            (
                "例 1：标准交错",
                (vec![1, 2, 3, 0, 0, 0], 3, vec![2, 5, 6]),
                vec![1, 2, 2, 3, 5, 6],
            ),
            ("例 2：nums2 为空", (vec![1], 1, vec![]), vec![1]),
            ("例 3：m = 0", (vec![0], 0, vec![1]), vec![1]),
        ]
    }

    // `equiv()` 用默认实现：这题输出唯一，不需要做规范化。

    // ================================================================
    // 禁止公式。只画内存。
    // ================================================================
    fn render(&self, (buf, m, nums2): &Self::Input) -> String {
        let m = *m;
        let n = nums2.len();
        let mut s = String::new();

        s.push_str(&format!(
            "nums1 缓冲区：len = m + n = {m} + {n} = {}\n",
            m + n
        ));
        s.push_str(&grid(buf));
        s.push_str(&format!(
            "      ↑ 下标 0..{m} 是有效数据，{m}..{} 是占位（题目说忽略）\n",
            m + n
        ));

        s.push('\n');
        s.push_str(&format!("nums2：len = n = {n}\n"));
        s.push_str(&grid(nums2));

        s.push('\n');
        s.push_str("想一想：\n");
        s.push_str("  · 你打算把结果写到哪里？\n");
        s.push_str("  · 如果写回 nums1，尾部那 n 个 0 对你来说是「垃圾」还是「空间」？\n");
        s.push_str("  · 如果你从下标 0 开始往右填，第一个会被你盖掉的元素是谁？\n");
        s
    }
}

/// 按档位生成一个"已经排好序"的随机数组。
fn sorted_sample(rng: &mut Rng, len: usize, span: i64) -> Vec<i64> {
    let mut v: Vec<i64> = (0..len).map(|_| rng.range(-span, span)).collect();
    v.sort_unstable();
    v
}

/// 把一串数字画成带下标的表格。故意不画指针——
/// 指针该放哪里，是你要自己想出来的东西。
fn grid(xs: &[i64]) -> String {
    let shown = xs.len().min(12);
    let mut s = String::new();

    s.push_str(" idx |");
    for i in 0..shown {
        s.push_str(&format!("{i:>6}"));
    }
    if xs.len() > shown {
        s.push_str("   ...");
    }
    s.push('\n');

    s.push_str("-----+");
    for _ in 0..shown {
        s.push_str("------");
    }
    s.push('\n');

    s.push_str(" val |");
    for v in &xs[..shown] {
        s.push_str(&format!("{v:>6}"));
    }
    if xs.len() > shown {
        s.push_str("   ...");
    }
    s.push('\n');

    s
}
