//! 差分测试引擎（Differential Testing）。
//!
//! 让 [`Problem::brute`]（你手写的暴力解）和 [`Problem::fast`]（参考最优解）
//! 在同一批用例上决斗。任何一处不一致，就把**那个具体的输入**原样打印出来。
//!
//! 用例分三类，覆盖三种不同的盲区：
//!
//! | 类别 | 作用 |
//! |------|------|
//! | `golden`  | 暴力解跑不动的边界（溢出临界点），直接钉死期望值 |
//! | `adversarial` | 手写恶意用例：空输入、全零、负数、极值…… |
//! | 随机   | 你**想不到**的那些输入，由种子确定、可复现 |

use crate::ui;
use problems::{Problem, Rng, Tier};
use std::time::{Duration, Instant};

/// 单条用例的判定结果。
pub struct Case {
    pub label: String,
    /// 只在失败时才渲染，避免为每个用例都生成 ASCII 图。
    pub input_repr: String,
    pub detail: String,
    pub ok: bool,
}

pub struct DiffReport {
    pub id: &'static str,
    pub title: &'static str,
    pub topic: &'static str,
    pub taocp: &'static str,
    pub cases: Vec<Case>,
    pub brute_time: Duration,
    pub fast_time: Duration,
    pub seed: u64,
}

impl DiffReport {
    pub fn total(&self) -> usize {
        self.cases.len()
    }
    pub fn passed(&self) -> usize {
        self.cases.iter().filter(|c| c.ok).count()
    }
    pub fn all_pass(&self) -> bool {
        self.cases.iter().all(|c| c.ok)
    }
}

/// 跑完整的差分对决。
pub fn differential<P: Problem>(cases: usize, seed: u64) -> DiffReport {
    let p = P::default();
    let mut rng = Rng::new(seed);

    let mut out = Vec::new();
    let mut brute_time = Duration::ZERO;
    let mut fast_time = Duration::ZERO;

    // ① 金标准：暴力解够不到的边界
    for (label, input, expected) in p.golden() {
        let t = Instant::now();
        let got = p.fast(&input);
        fast_time += t.elapsed();

        let ok = p.equiv(&got, &expected);
        out.push(Case {
            label: format!("golden · {label}"),
            input_repr: if ok { String::new() } else { p.render(&input) },
            detail: if ok {
                String::new()
            } else {
                format!("期望 = {expected:?}\n最优解 = {got:?}")
            },
            ok,
        });
    }

    // ② 手写恶意用例
    for (label, input) in p.adversarial() {
        let label = format!("恶意 · {label}");
        out.push(duel(&p, &label, &input, &mut brute_time, &mut fast_time));
    }

    // ③ 随机用例：只在暴力解可行的档位内生成
    let mut tiers: Vec<Tier> = Tier::ALL
        .iter()
        .copied()
        .filter(|t| *t <= p.brute_ceiling())
        .collect();
    if tiers.is_empty() {
        tiers.push(Tier::Tiny);
    }

    for i in 0..cases {
        let tier = tiers[i % tiers.len()];
        let input = p.generate(&mut rng, tier);
        let label = format!("随机 #{i:<3} [{:<6}] n={}", tier.label(), p.size(&input));
        out.push(duel(&p, &label, &input, &mut brute_time, &mut fast_time));
    }

    DiffReport {
        id: P::ID,
        title: P::TITLE,
        topic: P::TOPIC,
        taocp: P::TAOCP,
        cases: out,
        brute_time,
        fast_time,
        seed,
    }
}

fn duel<P: Problem>(
    p: &P,
    label: &str,
    input: &P::Input,
    brute_time: &mut Duration,
    fast_time: &mut Duration,
) -> Case {
    let t = Instant::now();
    let b = p.brute(input);
    *brute_time += t.elapsed();

    let t = Instant::now();
    let f = p.fast(input);
    *fast_time += t.elapsed();

    let ok = p.equiv(&b, &f);
    Case {
        label: label.to_string(),
        input_repr: if ok { String::new() } else { p.render(input) },
        detail: if ok {
            String::new()
        } else {
            format!("暴力解 = {b:?}\n最优解 = {f:?}")
        },
        ok,
    }
}

/// 打印对决结果。
pub fn print(r: &DiffReport) {
    println!(
        "{} {}",
        ui::bold("▶"),
        ui::bold(&format!("{} · {}", r.id, r.title))
    );
    println!(
        "  {} {}   {}",
        ui::dim("主题"),
        r.topic,
        ui::dim(&format!("TAOCP {}", r.taocp))
    );
    println!(
        "  {} seed={}   用例 {} 个",
        ui::dim("复现"),
        r.seed,
        r.total()
    );
    println!();

    let failed: Vec<&Case> = r.cases.iter().filter(|c| !c.ok).collect();

    if failed.is_empty() {
        println!("  {} 全部 {} 个用例通过", ui::green("✔"), r.total());
    } else {
        println!(
            "  {} {} / {} 个用例不一致",
            ui::red("✗"),
            failed.len(),
            r.total()
        );
        for c in failed.iter().take(3) {
            println!();
            println!("  {} {}", ui::red("不一致:"), ui::bold(&c.label));
            for line in c.input_repr.lines() {
                println!("      {line}");
            }
            for line in c.detail.lines() {
                println!("      {}", ui::yellow(line));
            }
        }
        if failed.len() > 3 {
            println!(
                "\n  {} 还有 {} 个失败用例未展开",
                ui::dim("..."),
                failed.len() - 3
            );
        }
    }

    println!();
    println!(
        "  {} 暴力 {}   最优 {}   倍率 {}",
        ui::dim("累计耗时"),
        ui::fmt_duration(r.brute_time),
        ui::fmt_duration(r.fast_time),
        ui::ratio(r.brute_time.as_nanos(), r.fast_time.as_nanos())
    );
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 一道"故意写错"的题：`brute` 是正确的，`fast` 遇到负数就翻车。
    /// 它存在的唯一意义，是证明差分引擎真的能抓到注入的 bug。
    #[derive(Default)]
    struct Buggy;

    impl Problem for Buggy {
        type Input = Vec<i64>;
        type Output = i64;

        const ID: &'static str = "buggy";
        const TITLE: &'static str = "故意写错的题";
        const TOPIC: &'static str = "自测";

        fn brute(&self, xs: &Vec<i64>) -> i64 {
            xs.iter().sum()
        }

        fn fast(&self, xs: &Vec<i64>) -> i64 {
            // BUG: 只累加正数
            xs.iter().filter(|x| **x > 0).sum()
        }

        fn generate(&self, rng: &mut Rng, _tier: Tier) -> Vec<i64> {
            (0..8).map(|_| rng.range(-50, 50)).collect()
        }

        fn adversarial(&self) -> Vec<(&'static str, Vec<i64>)> {
            vec![("全负数", vec![-1, -2, -3])]
        }

        fn size(&self, xs: &Vec<i64>) -> usize {
            xs.len()
        }

        fn render(&self, xs: &Vec<i64>) -> String {
            format!("{xs:?}")
        }
    }

    #[test]
    fn catches_injected_bug() {
        let r = differential::<Buggy>(30, 1);
        assert!(!r.all_pass(), "差分引擎漏掉了注入的 bug！");
        assert!(
            r.cases.iter().any(|c| c.label.starts_with("恶意")),
            "手写恶意用例应该最先抓到负数问题"
        );
    }

    #[test]
    fn same_seed_same_cases() {
        let a = differential::<Buggy>(10, 42);
        let b = differential::<Buggy>(10, 42);
        let la: Vec<&str> = a.cases.iter().map(|c| c.label.as_str()).collect();
        let lb: Vec<&str> = b.cases.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(la, lb, "同一个 seed 必须复现同一批用例");
        assert_eq!(a.passed(), b.passed());
    }

    #[test]
    fn different_seed_explores_differently() {
        // 注意：用例标签只编码序号和档位，跟输入内容无关。
        // 想验证"换种子 = 换输入"，必须直接比较 generate 的产物。
        let make_inputs = |seed: u64| -> Vec<Vec<i64>> {
            let mut rng = Rng::new(seed);
            (0..20)
                .map(|i| {
                    let tier = [Tier::Tiny, Tier::Small, Tier::Medium][i % 3];
                    Buggy.generate(&mut rng, tier)
                })
                .collect()
        };
        assert_ne!(make_inputs(1), make_inputs(2), "换种子应该生成不同的输入");
        assert_eq!(make_inputs(7), make_inputs(7), "同种子必须生成相同的输入");
    }
}
