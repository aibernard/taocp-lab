//! # 参考最优解集中地
//!
//! ## ⚠️ 人类：写 `brute()` 之前不要打开这个文件
//!
//! 这个文件存在的唯一理由，是把最优解**从你的视线里挪开**。
//!
//! 之前 `fast()` 和 `brute()` 在同一个文件里，最优解就躺在你的 `todo!()`
//! 下面几十行——你只要往下滚一下，训练就结束了。现在它在这里，
//! 每个题目文件里的 `fast()` 只剩一行转发：
//!
//! ```ignore
//! fn fast(&self, input: &Self::Input) -> Self::Output {
//!     crate::optimal::p0001_two_sum(input)
//! }
//! ```
//!
//! ## 什么时候可以打开
//!
//! `lab run <ID>` 通过之后。那时候对照着看，才是"复盘"而不是"抄答案"。
//! 顺序反了，这个仓库对你就没有任何价值。
//!
//! ## 约定
//!
//! 每个最优解是一个自由函数，签名就是该题的 `fn(&Input) -> Output`。
//! 由 AI 负责实现和搬运，人类负责在写完 `brute()` 之前不看。

use crate::p0001_two_sum;

/// P0001 · Two Sum —— `O(N)` 期望时间。
///
/// 暴力解的浪费在于：对每个 `i` 都重新扫一遍右边。
/// 反过来想——**扫到 `j` 的时候，需要的是"之前有没有见过 `target - nums[j]`"**。
/// 把见过的值存进哈希表，"找配对"就退化成了"查表"。
///
/// 为了与暴力解输出一致，这里同样保证返回字典序最小的那一对。
pub fn p0001_two_sum((nums, target): &p0001_two_sum::Input) -> p0001_two_sum::Output {
    use std::collections::HashMap;

    // 值 -> 该值出现的全部下标（天然升序）
    let mut pos: HashMap<i64, Vec<usize>> = HashMap::with_capacity(nums.len());
    for (i, &v) in nums.iter().enumerate() {
        pos.entry(v).or_default().push(i);
    }

    for (i, &v) in nums.iter().enumerate() {
        if let Some(js) = pos.get(&(target - v)) {
            // js 是升序的，第一个 j > i 就是该 i 下的最优搭档
            if let Some(&j) = js.iter().find(|&&j| j > i) {
                return Some((i, j));
            }
        }
    }
    None
}

/// P0509 · Fibonacci —— `O(N)` 时间、`O(1)` 空间。
///
/// 只用两个变量滚动，连数组都不需要——因为计算 `fib(n)` 只依赖前两项。
pub fn p0509_fibonacci(n: &u32) -> u64 {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..*n {
        let t = a + b;
        a = b;
        b = t;
    }
    a
}

/// P0088 · Merge Sorted Array —— `O(m + n)` 时间、`O(1)` 额外空间。
///
/// 关键在于**从后往前**填：`nums1` 的尾部 `n` 个格子本来就是空的，
/// 而 `nums1` 的有效数据全在前 `m` 个格子里。
/// 从最大的往 `k = m+n-1` 填，读指针永远追不上写指针，所以不会覆盖未读数据。
///
/// 反过来（从前往后填）就必然要开一个新数组，或者不断搬移元素。
pub fn p0088_merge_sorted_array(input: &crate::p0088_merge_sorted_array::Input) -> Vec<i64> {
    let (buf, m, nums2) = input;
    let (m, n) = (*m, nums2.len());

    let mut out = buf.clone();
    if n == 0 {
        return out;
    }
    if m == 0 {
        out[..n].copy_from_slice(nums2);
        return out;
    }

    // 全程用 isize，避免 i/j/k 减到 -1 时下溢。
    let mut i = m as isize - 1; // nums1 有效区末尾
    let mut j = n as isize - 1; // nums2 末尾
    let mut k = (m + n) as isize - 1; // 写入位置：从后往前

    while j >= 0 {
        if i >= 0 && out[i as usize] > nums2[j as usize] {
            out[k as usize] = out[i as usize];
            i -= 1;
        } else {
            out[k as usize] = nums2[j as usize];
            j -= 1;
        }
        k -= 1;
    }
    // j < 0 时收工：nums1 剩下的前缀本来就已经在正确位置上。
    out
}
