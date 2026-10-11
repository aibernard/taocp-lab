//! 复杂度曲线与 ASCII 降维。
//!
//! `curve` 是整个支架最"暴力"的部分：它按档位放大输入，让你**亲眼看着**
//! 暴力解从微秒滑向毫秒，然后在某个档位变成"拒绝运行"。
//!
//! 由于 [`Problem::brute_ops`] 能在**不实际运行**的前提下估算运算量，
//! 我们可以一路把估算值画到爆炸区间，而不用真的烧掉 CPU（和你的耐心）。

use crate::ui;
use problems::{Problem, Rng, Tier};
use std::time::Instant;

pub fn curve<P: Problem>() -> Result<(), String> {
    let p = P::default();

    println!(
        "{} {}",
        ui::bold("▶ 复杂度曲线"),
        ui::bold(&format!("{} · {}", P::ID, P::TITLE))
    );
    println!(
        "  暴力解可行上限: {}   （超过此档位拒绝运行，只给估算值）",
        ui::cyan(p.brute_ceiling().label())
    );
    println!();
    let head = format!(
        "{} {} {} {} {}  {}",
        ui::pad_left("档位", 7),
        ui::pad_left("规模", 8),
        ui::pad_left("暴力估算运算量", 15),
        ui::pad_left("暴力实测", 11),
        ui::pad_left("最优实测", 11),
        "判定"
    );
    println!("  {head}");
    println!("  {}", "─".repeat(62));

    for tier in Tier::ALL {
        let mut rng = Rng::new(0xC0FFEE ^ (tier_offset(tier)));
        let input = p.generate(&mut rng, tier);
        let n = p.size(&input);
        let ops = p.brute_ops(&input);

        let within_ceiling = tier <= p.brute_ceiling();

        let brute_cell = if within_ceiling {
            let t = Instant::now();
            let r = p.brute(&input);
            let d = t.elapsed();
            std::hint::black_box(r);
            ui::fmt_duration(d)
        } else {
            ui::red("拒绝运行")
        };

        let t = Instant::now();
        let r = p.fast(&input);
        let fast_d = t.elapsed();
        std::hint::black_box(r);

        let verdict = if within_ceiling {
            String::new()
        } else {
            ui::yellow("← 💥 此处暴力解已死")
        };

        println!(
            "  {} {} {} {} {}  {}",
            ui::pad_left(tier.label(), 7),
            ui::pad_left(&n.to_string(), 8),
            ui::pad_left(&ui::comma(ops), 15),
            ui::pad_left(&brute_cell, 11),
            ui::pad_left(&ui::fmt_duration(fast_d), 11),
            verdict
        );
    }

    println!();
    println!(
        "  {}",
        ui::dim("暴力估算运算量 = 不实际运行、纯靠公式推算出的运算次数。")
    );
    println!(
        "  {}",
        ui::dim("它就是「爆炸现场」的现场照片 —— 你不需要真的把 CPU 烧穿才能看见它。")
    );
    println!(
        "  {}",
        ui::dim("注意：单次采样的亚微秒实测受噪声支配，别盯着 Tiny/Small 的差异。")
    );
    println!(
        "  {}",
        ui::dim("      真正该看的是「暴力估算运算量」那一列的量级跃迁。")
    );
    println!();
    Ok(())
}

/// 把输入画成 ASCII 图。禁止公式，只画内存。
pub fn render<P: Problem>(tier: Tier, seed: u64) {
    let p = P::default();
    let mut rng = Rng::new(seed);
    let input = p.generate(&mut rng, tier);
    println!();
    println!("{}", p.render(&input));
}

fn tier_offset(t: Tier) -> u64 {
    match t {
        Tier::Tiny => 1,
        Tier::Small => 2,
        Tier::Medium => 3,
        Tier::Large => 4,
    }
}
