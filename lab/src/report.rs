//! 统计报告：把 append-only 的台账变成一份可读的成绩单。
//!
//! `lab stats --write` 会把它写进 `docs/training/stats.md`，
//! 这样你的训练轨迹就自动沉淀成了仓库里的 Markdown 产物。

use crate::{ledger, registry, ui};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::time::Duration;

pub fn print(write_file: bool) -> Result<(), String> {
    let entries = ledger::load();
    let schedule = ledger::replay(&entries);
    let md = render(&entries, &schedule);

    print!("{md}");

    if write_file {
        let path = crate::repo_root()
            .join("docs")
            .join("training")
            .join("stats.md");
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("无法创建 {}: {e}", dir.display()))?;
        }
        std::fs::write(&path, &md).map_err(|e| format!("无法写入 {}: {e}", path.display()))?;
        println!("{} 已写入 {}", ui::green("✔"), path.display());
    }
    Ok(())
}

#[derive(Default)]
struct Agg {
    runs: usize,
    cases: usize,
    passed: usize,
    grade_sum: u32,
}

fn render(entries: &[ledger::Entry], schedule: &BTreeMap<String, ledger::Sched>) -> String {
    let today = ledger::today();
    let mut md = String::new();

    let _ = writeln!(md, "# 训练统计\n");
    let _ = writeln!(md, "> 由 `lab stats --write` 自动生成，请勿手工编辑。");
    let _ = writeln!(md, "> 生成日期：{}\n", ledger::date_str(today));

    let runs = entries.len();
    let total_cases: usize = entries.iter().map(|e| e.cases).sum();
    let passed_cases: usize = entries.iter().map(|e| e.passed).sum();
    let brute_ns: u128 = entries.iter().map(|e| e.brute_ns).sum();
    let fast_ns: u128 = entries.iter().map(|e| e.fast_ns).sum();

    // ---------------- 总览 ----------------
    let _ = writeln!(md, "## 总览\n");
    let _ = writeln!(md, "| 指标 | 数值 |");
    let _ = writeln!(md, "|---|---|");
    let _ = writeln!(md, "| 训练场次 | {runs} |");
    let _ = writeln!(
        md,
        "| 覆盖题目 | {} / {} |",
        schedule.len(),
        registry::catalog().len()
    );
    let _ = writeln!(md, "| 累计测试用例 | {} |", ui::comma(total_cases as u128));
    let _ = writeln!(md, "| 用例通过率 | {} |", pct(passed_cases, total_cases));
    let _ = writeln!(md, "| 连续训练天数 | {} |", ledger::streak(entries));
    let _ = writeln!(md, "| 累计暴力解耗时 | {} |", dur(brute_ns));
    let _ = writeln!(md, "| 累计最优解耗时 | {} |", dur(fast_ns));
    let _ = writeln!(md, "| 暴力/最优倍率 | {} |", ui::ratio(brute_ns, fast_ns));

    // ---------------- 逐题 ----------------
    let mut aggs: BTreeMap<&str, Agg> = BTreeMap::new();
    for e in entries {
        let a = aggs.entry(e.problem.as_str()).or_default();
        a.runs += 1;
        a.cases += e.cases;
        a.passed += e.passed;
        a.grade_sum += e.grade as u32;
    }

    let _ = writeln!(md, "\n## 逐题\n");
    let _ = writeln!(
        md,
        "| ID | 题目 | 主题 | 场次 | 通过率 | 平均评级 | 下次复习 | 状态 |"
    );
    let _ = writeln!(md, "|---|---|---|---|---|---|---|---|");
    for m in registry::catalog() {
        let a = aggs.get(m.id);
        let (runs, rate, avg) = match a {
            None => (0, "—".to_string(), "—".to_string()),
            Some(a) => (
                a.runs,
                pct(a.passed, a.cases),
                format!("{:.1}", a.grade_sum as f64 / a.runs.max(1) as f64),
            ),
        };
        let (due, status) = match schedule.get(m.id) {
            None => ("—".to_string(), "未训练".to_string()),
            Some(s) if s.due_day <= today => (
                ledger::date_str(s.due_day),
                format!("**逾期 {} 天**", today - s.due_day),
            ),
            Some(s) => (
                ledger::date_str(s.due_day),
                format!("{} 天后", s.due_day - today),
            ),
        };
        let _ = writeln!(
            md,
            "| `{}` | {} | {} | {} | {} | {} | {} | {} |",
            m.id, m.title, m.topic, runs, rate, avg, due, status
        );
    }

    // ---------------- 主题 ----------------
    let mut topics: BTreeMap<&str, Agg> = BTreeMap::new();
    for e in entries {
        let a = topics.entry(e.topic.as_str()).or_default();
        a.runs += 1;
        a.cases += e.cases;
        a.passed += e.passed;
        a.grade_sum += e.grade as u32;
    }

    if !topics.is_empty() {
        let _ = writeln!(md, "\n## 主题（按通过率升序 = 薄弱环节优先）\n");
        let _ = writeln!(md, "| 主题 | 场次 | 通过率 | 平均评级 |");
        let _ = writeln!(md, "|---|---|---|---|");
        let mut rows: Vec<(&str, &Agg)> = topics.iter().map(|(k, v)| (*k, v)).collect();
        rows.sort_by(|a, b| {
            let ra = rate_of(a.1);
            let rb = rate_of(b.1);
            ra.partial_cmp(&rb).unwrap_or(std::cmp::Ordering::Equal)
        });
        for (topic, a) in rows {
            let _ = writeln!(
                md,
                "| {} | {} | {} | {:.1} |",
                topic,
                a.runs,
                pct(a.passed, a.cases),
                a.grade_sum as f64 / a.runs.max(1) as f64
            );
        }
    }

    if entries.is_empty() {
        let _ = writeln!(
            md,
            "\n> 台账还是空的。跑一次 `lab run p0001`，这里就会长出数据。\n"
        );
    }

    md
}

fn rate_of(a: &Agg) -> f64 {
    if a.cases == 0 {
        return 1.0;
    }
    a.passed as f64 / a.cases as f64
}

fn pct(passed: usize, total: usize) -> String {
    if total == 0 {
        return "—".to_string();
    }
    format!("{:.1}%", passed as f64 * 100.0 / total as f64)
}

fn dur(ns: u128) -> String {
    ui::fmt_duration(Duration::from_nanos(ns.min(u64::MAX as u128) as u64))
}
