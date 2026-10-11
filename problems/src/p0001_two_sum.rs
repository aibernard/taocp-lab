//! # P0001 · Two Sum（两数之和）
//!
//! * LeetCode 1 · Easy
//! * 主题：哈希表 / 空间换时间
//! * TAOCP：Vol.3 §6.4 Hashing
//!
//! ## 这道题在教你差分测试的第一条铁律
//!
//! Two Sum 是**多解题**：返回任意一组合法下标对都算对。
//! 如果暴力解返回 `(0, 5)` 而最优解返回 `(2, 3)`，两者都正确，
//! 但朴素的 `==` 比较会判定"不一致"——于是差分测试开始疯狂误报。
//!
//! 解法只有两条路：
//! 1. 把输出**规范化**成唯一形式（本题就选这条路：约定返回字典序最小的下标对）
//! 2. 重写 [`Problem::equiv`]，换成一个能容纳多解的等价判定
//!
//! 本题刻意选了第 1 条路，因为"先想清楚输出的规范形式"是工程直觉的核心训练。

use crate::{Problem, Rng, Tier};

#[derive(Default)]
pub struct TwoSum;

/// 输入：`(数组, 目标和)`
pub type Input = (Vec<i64>, i64);

/// 输出：满足 `nums[i] + nums[j] == target` 的**字典序最小**下标对 `(i, j)`，`i < j`。
pub type Output = Option<(usize, usize)>;

impl Problem for TwoSum {
    type Input = Input;
    type Output = Output;

    const ID: &'static str = "p0001";
    const TITLE: &'static str = "Two Sum / 两数之和";
    const TOPIC: &'static str = "哈希";
    const TAOCP: &'static str = "Vol.3 §6.4";

    /// ⚠️ 本题的 `brute()` 是**随仓库附带的示范实现，不是你写的**。
    /// 它只用来演示差分测试怎么工作。别把它算进你的训练记录。
    const REFERENCE: bool = true;

    /// 暴力解：`O(N²)`，两两配对，先到先得。
    ///
    /// 因为 `i` 从 0 递增、`j` 从 `i+1` 递增，第一个命中就是字典序最小的那对。
    /// 这个"副产物"正是我们用来规范化输出的依据。
    fn brute(&self, (nums, target): &Input) -> Output {
        for i in 0..nums.len() {
            for j in (i + 1)..nums.len() {
                if nums[i] + nums[j] == *target {
                    return Some((i, j));
                }
            }
        }
        None
    }

    /// 最优解：`O(N)` 期望时间。
    ///
    /// ⚠️ 实现放在 `problems/src/optimal.rs` —— **写 brute() 之前不要打开它**。
    /// 那不是防作弊，是防止你把自己训练成"识图机器"。
    fn fast(&self, input: &Input) -> Output {
        crate::optimal::p0001_two_sum(input)
    }

    fn generate(&self, rng: &mut Rng, tier: Tier) -> Input {
        let (n, span) = match tier {
            Tier::Tiny => (3, 6),
            Tier::Small => (16, 20),
            Tier::Medium => (200, 1_000),
            Tier::Large => (50_000, 100_000),
        };
        let nums: Vec<i64> = (0..n).map(|_| rng.range(-span, span)).collect();

        // 60% 的用例刻意保证"有解"，否则随机目标在稀疏数组里多半无解，
        // 会让测试覆盖不到命中分支。
        let target = if nums.len() >= 2 && rng.chance(60) {
            let i = rng.below(nums.len());
            let mut j = rng.below(nums.len());
            if j == i {
                j = (j + 1) % nums.len();
            }
            nums[i] + nums[j]
        } else {
            rng.range(-span, span)
        };
        (nums, target)
    }

    fn adversarial(&self) -> Vec<(&'static str, Input)> {
        vec![
            ("空数组", (vec![], 0)),
            ("单元素（不可能有解）", (vec![7], 7)),
            ("明确无解", (vec![1, 2, 3], 100)),
            ("答案在末尾", (vec![1, 2, 3, 4, 9], 13)),
            ("答案是最前一对", (vec![5, 5, 9, 9], 10)),
            ("全零 target=0", (vec![0, 0, 0, 0], 0)),
            ("含负数", (vec![-3, 4, 3, 90], 0)),
            ("重复值抢位", (vec![2, 2, 2, 2], 4)),
            ("同值只有一个", (vec![3, 3, 3], 6)),
            ("i64 极值边界", (vec![i64::MAX, i64::MIN, 1], -1)),
        ]
    }

    fn size(&self, (nums, _): &Input) -> usize {
        nums.len()
    }

    fn brute_ops(&self, (nums, _): &Input) -> u128 {
        let n = nums.len() as u128;
        n * (n.saturating_sub(1)) / 2
    }

    fn render(&self, (nums, target): &Input) -> String {
        if nums.is_empty() {
            return format!("target = {target}\n\n[ 空数组 —— 直接返回 None ]\n");
        }

        let shown = nums.len().min(12);
        let ans = self.brute(&(nums.clone(), *target));

        let mut s = String::new();
        s.push_str(&format!("target = {target}   (len = {})\n\n", nums.len()));

        s.push_str(" idx |");
        for i in 0..shown {
            s.push_str(&format!("{i:>6}"));
        }
        if nums.len() > shown {
            s.push_str("   ...");
        }
        s.push('\n');

        s.push_str("-----+");
        for _ in 0..shown {
            s.push_str("------");
        }
        s.push('\n');

        s.push_str(" val |");
        for v in &nums[..shown] {
            s.push_str(&format!("{v:>6}"));
        }
        if nums.len() > shown {
            s.push_str("   ...");
        }
        s.push('\n');

        // 在答案所在的列下方画箭头
        if let Some((i, j)) = ans {
            let mut marks = String::from("     |");
            for k in 0..shown {
                let mark = if k == i {
                    "  i->"
                } else if k == j {
                    "  j<-"
                } else {
                    "     "
                };
                marks.push_str(mark);
            }
            s.push_str(&marks);
            s.push('\n');
            s.push_str(&format!(
                "\n命中 (i={i}, j={j})：{} + {} = {target}\n",
                nums[i], nums[j]
            ));
        } else {
            s.push_str("\n无解：不存在任何一对下标满足条件。\n");
        }

        s.push_str(
            "\n暴力解：对每个 i 重新扫一遍右边        → O(N²)\n\
             最优解：扫到 j 时查表找 target-nums[j]  → O(N) 期望\n",
        );
        s
    }
}
