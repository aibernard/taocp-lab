//! 零依赖的确定性伪随机数发生器（xorshift64*）。
//!
//! 确定性是差分测试的生命线：同一个 `seed` 必须复现出完全相同的用例序列，
//! 否则失败用例无法被重放，调试就变成了玄学。

/// 固定种子的伪随机数发生器。
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
    seed: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        // 混洗一次，避免相邻 seed 产生高度相似的序列。
        Self {
            state: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
            seed,
        }
    }

    /// 记录在案、可用于复现的原始种子。
    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// `[0, n)` 内的均匀整数；`n == 0` 时返回 0。
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u64() % n as u64) as usize
    }

    /// `[lo, hi]` 闭区间内的整数。
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        debug_assert!(lo <= hi, "range: lo={lo} > hi={hi}");
        if lo == hi {
            return lo;
        }
        lo + (self.next_u64() % (hi - lo + 1) as u64) as i64
    }

    /// 以 `pct`% 的概率返回 true。
    pub fn chance(&mut self, pct: u32) -> bool {
        self.below(100) < pct as usize
    }

    /// 从切片里随机取一个元素。
    pub fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len())]
    }
}
