//! # problems —— 算法训练题库
//!
//! 每道题都是一个实现了 [`Problem`] 的模块，**必须**同时提供两个解：
//!
//! * [`Problem::brute`] —— **你亲手写**的暴力解。它不需要任何技巧，允许又慢又笨，
//!   但必须正确。它是你大脑里问题的"物理基准线"。
//! * [`Problem::fast`] —— 参考最优解。它是"对抗者"，负责在规模变大时把你的暴力解击穿。
//!
//! `lab` 工具会让这两者在同一批用例上决斗（Differential Testing）。
//! 你的暴力解一旦被击穿，终端会把那个致死的输入原样砸回来。
//!
//! ## 加一道新题
//!
//! 1. 复制 `problems/src/_template.rs` 为 `problems/src/pXXXX_name.rs`
//! 2. 在本文件里 `pub mod pXXXX_name;`
//! 3. 在 [`for_each_problem`] 宏里加上一行 `$crate::pXXXX_name::YourType,`

mod rng;
pub use rng::Rng;

/// 参考最优解集中地。**写 `brute()` 之前不要打开。**
/// 见该文件的文档注释。
mod optimal;

pub mod p0001_two_sum;
pub mod p0088_merge_sorted_array;
pub mod p0509_fibonacci;

/// 用例规模档位。Tier 越大，问题的输入规模越大。
///
/// `Medium` 通常是你暴力解的"舒适区边缘"，`Large` 则是"爆炸现场"——
/// 支架会拒绝在 `Large` 上运行指数级暴力解，改为直接打印它的估算运算量。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    Tiny,
    Small,
    Medium,
    Large,
}

impl Tier {
    pub const ALL: [Tier; 4] = [Tier::Tiny, Tier::Small, Tier::Medium, Tier::Large];

    pub fn label(self) -> &'static str {
        match self {
            Tier::Tiny => "Tiny",
            Tier::Small => "Small",
            Tier::Medium => "Medium",
            Tier::Large => "Large",
        }
    }

    pub fn parse(s: &str) -> Option<Tier> {
        match s.trim().to_ascii_lowercase().as_str() {
            "tiny" | "t" => Some(Tier::Tiny),
            "small" | "s" => Some(Tier::Small),
            "medium" | "m" => Some(Tier::Medium),
            "large" | "l" => Some(Tier::Large),
            _ => None,
        }
    }
}

/// 一道训练题的完整契约。
///
/// 只有 `brute` / `fast` / `generate` / `adversarial` / `size` / `render`
/// 是必须实现的；其余方法都有合理默认值。
pub trait Problem: Default {
    /// 输入类型。会被 `Debug` 打印，便于复现失败用例。
    type Input: Clone + core::fmt::Debug;
    /// 输出类型。**必须是可规范化的**，否则请重写 [`Problem::equiv`]。
    type Output: PartialEq + core::fmt::Debug;

    /// 稳定 ID，如 `"p0001"`。台账、调度、CLI 全靠它。
    const ID: &'static str;
    const TITLE: &'static str;
    /// 主题标签，用于统计薄弱环节，如 `"哈希"`。
    const TOPIC: &'static str;
    /// 对应的 TAOCP 章节，用于建立"教材 ↔ 题库"的映射。
    const TAOCP: &'static str = "—";

    /// 是否为**参考样板题**。
    ///
    /// 样板题的 `brute()` 是随仓库附带的示范实现，**不是你写的**。
    /// 它们的作用只有一个：演示这套支架怎么用。因此：
    ///
    /// * `lab list` 会给它们打上「样板」标记
    /// * `lab due` 不会把它们排进复习队列
    ///
    /// ⚠️ 拿样板题练手 = 自欺欺人。真正的训练从复制
    /// `problems/src/_template.rs` 开始，自己写 `brute()`。
    const REFERENCE: bool = false;

    /// 暴力解：凭直觉写，不管复杂度。这是你的逻辑基准线。
    fn brute(&self, input: &Self::Input) -> Self::Output;

    /// 最优解：参考实现，负责击穿暴力解。
    fn fast(&self, input: &Self::Input) -> Self::Output;

    /// 按档位生成随机用例。
    fn generate(&self, rng: &mut Rng, tier: Tier) -> Self::Input;

    /// 手写的恶意边界用例。标着名字，失败时一眼看得出是哪类边界崩的。
    fn adversarial(&self) -> Vec<(&'static str, Self::Input)>;

    /// 输入的规模度量（数组长度、n 等），用于画复杂度曲线。
    fn size(&self, input: &Self::Input) -> usize;

    /// 把输入画成 ASCII 图。这是"数学降维"的落地点：禁止公式，只画内存。
    fn render(&self, input: &Self::Input) -> String;

    /// 暴力解还算可行的规模上限。差分测试只在 `tier <= ceiling` 时调用 `brute`。
    ///
    /// 这条边界不是懒惰，而是**诚实**：差分测试的覆盖范围，
    /// 天然被暴力解的可运行范围锁死。
    fn brute_ceiling(&self) -> Tier {
        Tier::Medium
    }

    /// 暴力解运算量的估算值（**不实际运行**），用于画出爆炸曲线。
    /// 默认按 `size²` 粗估，暴力解是别的复杂度时请重写。
    fn brute_ops(&self, input: &Self::Input) -> u128 {
        let n = self.size(input) as u128;
        n * n
    }

    /// 金标准用例：暴力解跑不动的边界（如整型溢出临界点），直接给定期望值。
    /// 这是对差分测试"覆盖盲区"的补丁。
    fn golden(&self) -> Vec<(&'static str, Self::Input, Self::Output)> {
        Vec::new()
    }

    /// 差分判定口：两个解的输出在何种意义上"等价"。
    ///
    /// 默认按 `PartialEq` 精确比较。**多解题**（例如"任取一组满足条件的下标对"）
    /// 必须重写此方法，把输出映射到同一个等价类上——否则差分测试会疯狂误报。
    fn equiv(&self, a: &Self::Output, b: &Self::Output) -> bool {
        a == b
    }
}

/// 题目注册表：新增题目只需在此宏里加一行，CLI / 统计 / 调度自动跟进。
#[macro_export]
macro_rules! for_each_problem {
    ($m:ident) => {
        $m! {
            $crate::p0001_two_sum::TwoSum,
            $crate::p0509_fibonacci::Fibonacci,
            $crate::p0088_merge_sorted_array::MergeSortedArray,
        }
    };
}
