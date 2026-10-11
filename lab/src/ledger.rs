//! 训练台账与间隔重复调度。
//!
//! ## 为什么要台账
//!
//! "停止刷题就遗忘"的根因是：**没有调度**。你凭心情决定今天刷什么，
//! 于是永远在刷已经会的东西，而快忘的东西永远等不到第二次接触。
//!
//! 台账是一行行 append-only 的记录，`lab` 每次跑题都会追加一条。
//! 调度信息（复习间隔、到期日）由台账**重放**推导出来——
//! 台账是唯一的事实来源，不怕状态文件和记录对不上。
//!
//! ## 调度算法
//!
//! 精简版 SM-2（SuperMemo 2）：
//! * 评级 `grade ∈ [0,5]`，`< 3` 视为"没掌握"，间隔重置为 1 天
//! * 掌握后间隔按 `1 → 3 → 上次间隔 × ease` 递增
//! * `ease` 是难度因子，稳定在 `[1.3, 2.8]`

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub const HEADER: &str = "\
# taocp-lab 训练台账 · 由 `lab run` 自动追加
# 手工改坏会让复习调度错乱，请只改 grade 列（第 10 列）
# day|ts|problem|topic|verdict|cases|passed|brute_ns|fast_ns|grade|interval|due_day";

/// 一行台账。
#[derive(Clone, Debug)]
pub struct Entry {
    /// 距 1970-01-01 的天数，避免时区/夏令时的所有麻烦。
    pub day: u64,
    pub ts: u64,
    pub problem: String,
    pub topic: String,
    pub verdict: String,
    pub cases: usize,
    pub passed: usize,
    pub brute_ns: u128,
    pub fast_ns: u128,
    pub grade: u8,
    pub interval: u32,
    pub due_day: u64,
}

impl Entry {
    pub fn to_line(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.day,
            self.ts,
            self.problem,
            self.topic,
            self.verdict,
            self.cases,
            self.passed,
            self.brute_ns,
            self.fast_ns,
            self.grade,
            self.interval,
            self.due_day
        )
    }

    /// 解析一行；注释和坏行返回 `None`。
    pub fn parse(line: &str) -> Option<Entry> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        let f: Vec<&str> = line.split('|').collect();
        if f.len() != 12 {
            return None;
        }
        Some(Entry {
            day: f[0].parse().ok()?,
            ts: f[1].parse().ok()?,
            problem: f[2].to_string(),
            topic: f[3].to_string(),
            verdict: f[4].to_string(),
            cases: f[5].parse().ok()?,
            passed: f[6].parse().ok()?,
            brute_ns: f[7].parse().ok()?,
            fast_ns: f[8].parse().ok()?,
            grade: f[9].parse().ok()?,
            interval: f[10].parse().ok()?,
            due_day: f[11].parse().ok()?,
        })
    }
}

// ---------------------------------------------------------------- 时间

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn today() -> u64 {
    now_secs() / 86_400
}

/// 把"距 1970-01-01 的天数"转成 `YYYY-MM-DD`（Howard Hinnant 的 civil_from_days）。
pub fn date_str(day: u64) -> String {
    let z = day as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

// ---------------------------------------------------------------- 调度

/// 一道题的复习状态。
#[derive(Clone, Debug)]
pub struct Sched {
    pub reps: u32,
    pub interval: u32,
    pub ease: f64,
    pub due_day: u64,
    pub last_day: u64,
    pub last_grade: u8,
    pub runs: u32,
}

impl Sched {
    /// 从未训练过的初始状态。
    pub fn fresh() -> Sched {
        Sched {
            reps: 0,
            interval: 0,
            ease: 2.5,
            due_day: 0,
            last_day: 0,
            last_grade: 0,
            runs: 0,
        }
    }
}

impl Default for Sched {
    fn default() -> Self {
        Sched::fresh()
    }
}

/// 由台账重放出每道题的最新调度状态。
pub fn replay(entries: &[Entry]) -> BTreeMap<String, Sched> {
    let mut map: BTreeMap<String, Sched> = BTreeMap::new();
    for e in entries {
        let prev = map.get(&e.problem).cloned().unwrap_or_else(Sched::fresh);
        map.insert(e.problem.clone(), sm2(&prev, e.grade, e.day));
    }
    map
}

/// 精简版 SM-2：算出一条新记录之后的调度状态。
pub fn sm2(prev: &Sched, grade: u8, day: u64) -> Sched {
    let mut s = prev.clone();
    let g = grade.min(5) as f64;

    // 难度因子：评级越低，ease 掉得越快
    s.ease = (s.ease + (0.1 - (5.0 - g) * (0.08 + (5.0 - g) * 0.02))).clamp(1.3, 2.8);

    if grade < 3 {
        // 没掌握：重来
        s.reps = 0;
        s.interval = 1;
    } else {
        s.reps += 1;
        s.interval = match s.reps {
            1 => 1,
            2 => 3,
            _ => ((prev.interval.max(1) as f64) * s.ease).round().max(1.0) as u32,
        };
    }

    s.last_grade = grade;
    s.last_day = day;
    s.runs = prev.runs + 1;
    s.due_day = day + s.interval as u64;
    s
}

// ---------------------------------------------------------------- 读写

pub fn path() -> PathBuf {
    crate::repo_root().join("records").join("ledger.tsv")
}

pub fn load() -> Vec<Entry> {
    let p = path();
    let Ok(f) = fs::File::open(&p) else {
        return Vec::new();
    };
    BufReader::new(f)
        .lines()
        .map_while(Result::ok)
        .filter_map(|l| Entry::parse(&l))
        .collect()
}

pub fn append(e: &Entry) -> Result<(), String> {
    let p = path();
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir).map_err(|err| format!("无法创建 {}: {err}", dir.display()))?;
    }
    let existed = p.exists();
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&p)
        .map_err(|err| format!("无法写入 {}: {err}", p.display()))?;

    if !existed {
        writeln!(f, "{HEADER}").map_err(|err| err.to_string())?;
    }
    writeln!(f, "{}", e.to_line()).map_err(|err| err.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------- 统计辅助

/// 连续训练天数（以今天或昨天为终点）。
pub fn streak(entries: &[Entry]) -> u64 {
    let mut days: Vec<u64> = entries.iter().map(|e| e.day).collect();
    days.sort_unstable();
    days.dedup();
    if days.is_empty() {
        return 0;
    }

    let today = today();
    let last = *days.last().unwrap();
    if today.saturating_sub(last) > 1 {
        return 0; // 断了
    }

    let mut streak = 1u64;
    let mut cursor = last;
    for &d in days.iter().rev().skip(1) {
        if cursor.saturating_sub(d) == 1 {
            streak += 1;
            cursor = d;
        } else {
            break;
        }
    }
    streak
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_conversion_anchors() {
        assert_eq!(date_str(0), "1970-01-01");
        assert_eq!(date_str(1), "1970-01-02");
        assert_eq!(date_str(365), "1971-01-01");
        // 2024-02-29 —— 闰日
        assert_eq!(date_str(19_782), "2024-02-29");
    }

    #[test]
    fn entry_roundtrip() {
        let e = Entry {
            day: 20_000,
            ts: 1_728_000_000,
            problem: "p0001".into(),
            topic: "哈希".into(),
            verdict: "PASS".into(),
            cases: 40,
            passed: 40,
            brute_ns: 1234,
            fast_ns: 567,
            grade: 4,
            interval: 3,
            due_day: 20_003,
        };
        let line = e.to_line();
        let back = Entry::parse(&line).expect("应能解析回来");
        assert_eq!(back.problem, e.problem);
        assert_eq!(back.cases, e.cases);
        assert_eq!(back.grade, e.grade);
        assert_eq!(back.due_day, e.due_day);
    }

    #[test]
    fn comments_and_garbage_are_skipped() {
        assert!(Entry::parse("# 注释").is_none());
        assert!(Entry::parse("").is_none());
        assert!(Entry::parse("只有|三|列").is_none());
    }

    #[test]
    fn failing_grade_resets_the_interval() {
        let mut s = Sched::fresh();
        for _ in 0..3 {
            s = sm2(&s, 5, 100);
        }
        assert!(
            s.interval > 3,
            "连续高分应该把间隔拉长，实际 {}",
            s.interval
        );

        let after_fail = sm2(&s, 1, 200);
        assert_eq!(after_fail.reps, 0);
        assert_eq!(after_fail.interval, 1, "评级 < 3 必须把间隔打回 1 天");
        assert!(after_fail.ease < s.ease, "失败应该降低难度因子");
    }

    #[test]
    fn interval_progression_is_1_3_then_growing() {
        let mut s = Sched::fresh();
        s = sm2(&s, 4, 0);
        assert_eq!(s.interval, 1);
        s = sm2(&s, 4, 1);
        assert_eq!(s.interval, 3);
        s = sm2(&s, 4, 4);
        assert!(
            s.interval > 3,
            "第三次开始应按 ease 增长，实际 {}",
            s.interval
        );
    }

    #[test]
    fn ease_stays_in_sane_bounds() {
        let mut s = Sched::fresh();
        for _ in 0..50 {
            s = sm2(&s, 0, 0);
        }
        assert!(s.ease >= 1.3, "ease 不应跌破 1.3，实际 {}", s.ease);

        let mut s = Sched::fresh();
        for _ in 0..50 {
            s = sm2(&s, 5, 0);
        }
        assert!(s.ease <= 2.8, "ease 不应超过 2.8，实际 {}", s.ease);
    }

    #[test]
    fn replay_rebuilds_from_ledger_only() {
        let mk = |day: u64, grade: u8| Entry {
            day,
            ts: 0,
            problem: "p0001".into(),
            topic: "哈希".into(),
            verdict: "PASS".into(),
            cases: 10,
            passed: 10,
            brute_ns: 0,
            fast_ns: 0,
            grade,
            interval: 0,
            due_day: 0,
        };
        let entries = vec![mk(10, 4), mk(11, 4), mk(14, 4)];
        let map = replay(&entries);
        let s = map.get("p0001").expect("应有调度状态");
        assert_eq!(s.runs, 3);
        assert_eq!(s.last_day, 14);
        assert_eq!(s.due_day, 14 + s.interval as u64);
    }
}
