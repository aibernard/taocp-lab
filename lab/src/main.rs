//! `lab` —— 算法实验室命令行。
//!
//! 零依赖，标准库直出。运行 `lab help` 看用法。

mod bench;
mod diff;
mod ledger;
mod registry;
mod report;
mod scaffold;
mod session;
mod ui;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;

use problems::Tier;
use registry::Cmd;

const DEFAULT_CASES: usize = 50;
const DEFAULT_SEED: u64 = 0x5EED;

/// 从当前目录向上找到仓库根（同时含 `lab/` 与 `problems/` 的目录）。
pub fn repo_root() -> PathBuf {
    let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut dir = start.as_path();
    loop {
        if dir.join("problems").is_dir() && dir.join("lab").is_dir() {
            return dir.to_path_buf();
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => return start,
        }
    }
}

fn main() -> ExitCode {
    install_brute_hint_hook();

    let args: Vec<String> = std::env::args().skip(1).collect();
    match dispatch(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("{} {msg}", ui::red("错误:"));
            eprintln!("{}", ui::dim("运行 `lab help` 查看用法"));
            ExitCode::FAILURE
        }
    }
}

/// `brute()` 里还是 `todo!()` 时，默认的 panic 输出是给程序员看的，
/// 不是给正在训练的你看的。这里把它换成一句人话。
///
/// 放在 `main` 里而不是逐个命令处理，是为了让 `diff` / `bench` / `run`
/// 三条路径同时受益——它们都会在某个时刻调用 `brute()`。
fn install_brute_hint_hook() {
    let fallback = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let payload = info.payload();
        let msg = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_default();

        if msg.contains("请亲手实现暴力解") {
            eprintln!();
            eprintln!("{} 这道题的 brute() 还没有实现。", ui::yellow("⏳"));
            eprintln!("   `problems/src/` 里那道题的文件，把 `todo!()` 换成你的暴力解。");
            eprintln!("   那是这道题唯一该由你写的东西 —— 写完再回来跑这条命令。");
            eprintln!();
        } else {
            fallback(info);
        }
    }));
}

fn dispatch(args: &[String]) -> Result<(), String> {
    let p = Parsed::parse(args);

    match p.cmd.as_str() {
        "" | "help" | "-h" | "--help" => {
            help();
            Ok(())
        }
        "list" | "ls" => {
            list();
            Ok(())
        }
        "due" => {
            due();
            Ok(())
        }
        "stats" => report::print(p.has("write")),
        "log" => {
            log(p.usize_flag("limit", 20)?);
            Ok(())
        }
        "new" | "import" | "add" => {
            let id = p.pos.first().ok_or(
                "`lab new` 需要两个参数：题目 ID 和 slug\n例如：lab new 49 group_anagrams",
            )?;
            let slug = p.pos.get(1).ok_or(
                "`lab new` 还缺一个 slug\n例如：lab new 49 group_anagrams --title \"Group Anagrams / 字母异位词分组\"",
            )?;
            scaffold::create(id, slug, p.flag("title"), p.flag("topic"), p.flag("taocp"))
        }
        "diff" => {
            let id = p.need_id()?;
            let cmd = Cmd::Diff {
                cases: p.usize_flag("cases", DEFAULT_CASES)?,
                seed: p.u64_flag("seed", DEFAULT_SEED)?,
            };
            registry::dispatch(&id, &cmd)
        }
        "bench" => {
            let id = p.need_id()?;
            registry::dispatch(&id, &Cmd::Bench)
        }
        "render" => {
            let id = p.need_id()?;
            let tier = match p.flag("tier") {
                Some(t) => Tier::parse(t).ok_or_else(|| {
                    format!("未知档位 `{t}`，可选：tiny / small / medium / large")
                })?,
                None => Tier::Small,
            };
            let cmd = Cmd::Render {
                tier,
                seed: p.u64_flag("seed", DEFAULT_SEED)?,
            };
            registry::dispatch(&id, &cmd)
        }
        "run" => {
            let id = p.need_id()?;
            let grade = match p.flag("grade") {
                Some(g) => Some(
                    g.parse::<u8>()
                        .map_err(|_| format!("--grade 需要 0..5 的整数，收到 `{g}`"))?,
                ),
                None => None,
            };
            let cmd = Cmd::Run {
                cases: p.usize_flag("cases", DEFAULT_CASES)?,
                seed: p.u64_flag("seed", DEFAULT_SEED)?,
                grade,
            };
            registry::dispatch(&id, &cmd)
        }
        other => Err(format!("未知命令 `{other}`")),
    }
}

// ---------------------------------------------------------------- 命令实现

fn list() {
    let entries = ledger::load();
    let schedule = ledger::replay(&entries);
    let today = ledger::today();

    println!("{}", ui::bold("题库"));
    println!(
        "  {} {} {} {}下次复习",
        ui::pad("ID", 8),
        ui::pad("题目", 30),
        ui::pad("主题", 16),
        ui::pad("TAOCP", 13),
    );
    println!("  {}", "─".repeat(70));

    for m in registry::catalog() {
        let due = if m.reference {
            ui::dim("样板题")
        } else {
            match schedule.get(m.id) {
                None => ui::dim("未训练"),
                Some(s) if s.due_day <= today => ui::red(&format!("逾期{}天", today - s.due_day)),
                Some(s) => format!("{}天后", s.due_day - today),
            }
        };
        println!(
            "  {} {} {} {}{}",
            ui::cyan(&ui::pad(m.id, 8)),
            ui::pad(&ui::truncate_width(m.title, 30), 30),
            ui::pad(&ui::truncate_width(m.topic, 16), 16),
            ui::pad(&ui::truncate_width(m.taocp, 13), 13),
            due
        );
    }

    if registry::catalog().iter().any(|m| m.reference) {
        println!();
        println!(
            "  {} 标「样板题」的条目，`brute()` 由仓库附带（不是你写的），",
            ui::yellow("⚠")
        );
        println!("      仅供演示支架用法，不排进复习队列。真正的训练从 `_template.rs` 开始。");
    }

    println!();
    println!(
        "  {}",
        ui::dim(
            "单题闭环: lab render <ID>  →  手写 brute  →  lab diff <ID>  →  lab bench <ID>  →  lab run <ID>"
        )
    );
    println!();
}

fn due() {
    let entries = ledger::load();
    let schedule = ledger::replay(&entries);
    let today = ledger::today();

    println!(
        "{} {}",
        ui::bold("📅 今日复习队列"),
        ui::dim(&ledger::date_str(today))
    );
    println!();

    let mut overdue: Vec<(registry::Meta, ledger::Sched)> = Vec::new();
    let mut fresh: Vec<registry::Meta> = Vec::new();
    let mut upcoming: Vec<(registry::Meta, u64)> = Vec::new();
    let mut skipped_reference = 0usize;

    for m in registry::catalog() {
        // 样板题的 brute() 不是你写的，把它排进复习队列毫无意义。
        if m.reference {
            skipped_reference += 1;
            continue;
        }
        match schedule.get(m.id) {
            None => fresh.push(m),
            Some(s) if s.due_day <= today => overdue.push((m, s.clone())),
            Some(s) => upcoming.push((m, s.due_day - today)),
        }
    }

    if overdue.is_empty() && fresh.is_empty() {
        println!("  {} 今天没有到期的题。", ui::green("✔"));
        println!(
            "  {}",
            ui::dim("按间隔重复的原理，今天应该让大脑休息，或者去开一道新题。")
        );
    }

    for (m, s) in &overdue {
        println!(
            "  {} {}{} 逾期 {} 天   上次 {}/5，间隔 {} 天",
            ui::red("●"),
            ui::pad(m.id, 8),
            ui::pad(&ui::truncate_width(m.title, 36), 36),
            today - s.due_day,
            s.last_grade,
            s.interval
        );
    }
    for m in &fresh {
        println!(
            "  {} {}{} 未训练 —— 首刷请先写暴力解，再让 fuzzer 来打你",
            ui::yellow("○"),
            ui::pad(m.id, 8),
            ui::pad(&ui::truncate_width(m.title, 36), 36),
        );
    }

    if !upcoming.is_empty() {
        println!();
        println!("  {}", ui::dim("排队中:"));
        for (m, in_days) in &upcoming {
            println!(
                "    {} {}{} {} 天后",
                ui::dim("·"),
                ui::pad(m.id, 8),
                ui::pad(&ui::truncate_width(m.title, 36), 36),
                in_days
            );
        }
    }

    if skipped_reference > 0 {
        println!();
        println!(
            "  {}",
            ui::dim(&format!(
                "（{skipped_reference} 道样板题未列入：它们的 brute() 不是你的产出，复习它们等于骗自己）"
            ))
        );
    }
    println!();
}

fn log(limit: usize) {
    let entries = ledger::load();
    if entries.is_empty() {
        println!("{}", ui::dim("台账还是空的。先跑一次 `lab run p0001`。"));
        return;
    }

    println!("{}", ui::bold("最近训练记录"));
    println!(
        "  {}{}{}{}{:>6}{:>9}",
        ui::pad("日期", 12),
        ui::pad("题目", 9),
        ui::pad("主题", 22),
        ui::pad("判定", 10),
        "评级",
        "倍率"
    );
    println!("  {}", "─".repeat(70));

    for e in entries.iter().rev().take(limit) {
        let verdict = if e.verdict == "PASS" {
            ui::green(&ui::pad("PASS", 10))
        } else {
            ui::red(&ui::pad(&e.verdict, 10))
        };
        println!(
            "  {}{}{}{}{:>6}{:>9}",
            ui::pad(&ledger::date_str(e.day), 12),
            ui::pad(&e.problem, 9),
            ui::pad(&ui::truncate_width(&e.topic, 22), 22),
            verdict,
            format!("{}/5", e.grade),
            ui::ratio(e.brute_ns, e.fast_ns)
        );
    }

    println!();
    println!(
        "  {}",
        ui::dim(&format!(
            "共 {} 条记录 · 连续训练 {} 天",
            entries.len(),
            ledger::streak(&entries)
        ))
    );
    println!();
}

fn help() {
    let b = ui::bold;
    println!("{}", b("lab —— 算法实验室"));
    println!();
    println!("{}", b("导入新题"));
    println!("  lab new <ID> <slug>          生成题目骨架并自动注册（brute() 留给你写）");
    println!("      [--title T] [--topic T] [--taocp T]");
    println!("      例: lab new 49 group_anagrams --title \"Group Anagrams / 字母异位词分组\"");
    println!();
    println!("{}", b("观察"));
    println!("  lab list                     题目目录 + 复习到期状态");
    println!("  lab due                      今日复习队列（间隔重复调度）");
    println!("  lab log [--limit N]          最近训练记录");
    println!("  lab stats [--write]          统计报告（--write 写入 docs/training/stats.md）");
    println!();
    println!("{}", b("单题闭环"));
    println!("  lab render <ID> [--tier T]   把输入画成 ASCII 图（禁止公式，只画内存）");
    println!("  lab diff   <ID> [--cases N]  差分对决：你的暴力解 vs 最优解");
    println!("  lab bench  <ID>              复杂度曲线：看着暴力解爆炸");
    println!("  lab run    <ID> [--grade G]  完整跑一遍并记入台账");
    println!();
    println!("{}", b("参数"));
    println!("  --cases N    随机用例数量（默认 {DEFAULT_CASES}）");
    println!("  --seed  N    随机种子（默认 {DEFAULT_SEED}），换种子 = 换一批没见过的输入");
    println!("  --tier  T    tiny | small | medium | large（默认 small）");
    println!("  --grade G    自评 0..5。0-2 表示没掌握（间隔重置），3-5 表示掌握");
    println!("               不填则自动：全过 = 4，有不一致 = 1");
    println!();
    println!("{}", b("核心原则"));
    println!("  · 暴力解必须由你亲手写。最优解存在的唯一意义是打你的脸。");
    println!("  · 失败时先看具体输入，再决定改代码 —— 不要凭感觉改。");
    println!();
}

// ---------------------------------------------------------------- 参数解析

struct Parsed {
    cmd: String,
    pos: Vec<String>,
    flags: BTreeMap<String, String>,
}

impl Parsed {
    fn parse(args: &[String]) -> Parsed {
        let mut cmd = String::new();
        let mut pos: Vec<String> = Vec::new();
        let mut flags: BTreeMap<String, String> = BTreeMap::new();

        let mut i = 0;
        while i < args.len() {
            let a = &args[i];
            if let Some(rest) = a.strip_prefix("--") {
                if let Some((k, v)) = rest.split_once('=') {
                    flags.insert(k.to_string(), v.to_string());
                } else if !cmd.is_empty() && i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    flags.insert(rest.to_string(), args[i + 1].clone());
                    i += 1;
                } else {
                    flags.insert(rest.to_string(), "true".to_string());
                }
            } else if cmd.is_empty() {
                cmd = a.clone();
            } else {
                pos.push(a.clone());
            }
            i += 1;
        }

        Parsed { cmd, pos, flags }
    }

    fn flag(&self, k: &str) -> Option<&str> {
        self.flags.get(k).map(|s| s.as_str())
    }

    fn has(&self, k: &str) -> bool {
        self.flags.contains_key(k)
    }

    fn usize_flag(&self, k: &str, dflt: usize) -> Result<usize, String> {
        match self.flag(k) {
            None => Ok(dflt),
            Some(v) => v
                .parse()
                .map_err(|_| format!("--{k} 需要一个非负整数，收到 `{v}`")),
        }
    }

    fn u64_flag(&self, k: &str, dflt: u64) -> Result<u64, String> {
        match self.flag(k) {
            None => Ok(dflt),
            Some(v) => v
                .parse()
                .map_err(|_| format!("--{k} 需要一个整数，收到 `{v}`")),
        }
    }

    fn need_id(&self) -> Result<String, String> {
        if let Some(id) = self.pos.first() {
            return Ok(id.clone());
        }
        if let Some(id) = self.flag("id") {
            return Ok(id.to_string());
        }
        Err(format!(
            "`{}` 需要一个题目 ID，例如 `lab {} p0001`",
            self.cmd, self.cmd
        ))
    }
}

#[cfg(test)]
mod tests {
    use problems::Problem;

    // 用注册表宏自动为**每一道**注册题目生成断言：
    // 暴力解与最优解必须在同一批用例上完全一致。
    // 加了新题却忘了让两个解对齐，`cargo test` 会立刻拦下来。
    macro_rules! make_agreement_test {
        ($($ty:ty),* $(,)?) => {
            #[test]
            fn every_registered_problem_agrees() {
                let mut unfinished: Vec<&'static str> = Vec::new();

                $(
                    match run_isolated::<$ty>() {
                        // brute() 还是 todo!() —— 这道题还没开工，跳过不算失败
                        Err(()) => unfinished.push(<$ty as Problem>::ID),
                        Ok(r) => assert!(
                            r.all_pass(),
                            "{} 的暴力解与最优解不一致，失败用例: {:?}",
                            <$ty as Problem>::ID,
                            r.cases
                                .iter()
                                .filter(|c| !c.ok)
                                .map(|c| c.label.as_str())
                                .collect::<Vec<_>>()
                        ),
                    }
                )*

                if !unfinished.is_empty() {
                    println!(
                        "⏳ 尚未实现 brute()，本次跳过: {} （写完之后这里会自动开始把关）",
                        unfinished.join(", ")
                    );
                }
            }
        };
    }

    problems::for_each_problem!(make_agreement_test);

    // 金标准用例是题目给的**规范**，不依赖 `brute()`。
    // 它保证：不管你有没有开始写暴力解，AI 写的最优解都必须先把
    // 题目原文的例子跑对。
    macro_rules! make_golden_test {
        ($($ty:ty),* $(,)?) => {
            #[test]
            fn every_golden_case_holds() {
                $(
                    {
                        let p = <$ty as Default>::default();
                        for (label, input, expected) in p.golden() {
                            let got = p.fast(&input);
                            assert!(
                                p.equiv(&got, &expected),
                                "{} 的金标准「{label}」失败：期望 {expected:?}，最优解给出 {got:?}",
                                <$ty as Problem>::ID
                            );
                        }
                    }
                )*
            }
        };
    }

    problems::for_each_problem!(make_golden_test);

    /// 跑一次差分测试；`brute()` 还是 `todo!()`（触发 panic）时返回 `Err`。
    ///
    /// 为什么要区分这件事：如果你正写到一半，整个测试套件一直是红的，
    /// 那"红色"就不再意味着"这里有问题"了 —— 信号会失效。
    /// 现在只有**真正不一致**才会让测试失败。
    fn run_isolated<P: Problem>() -> Result<crate::diff::DiffReport, ()> {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {})); // 静音：我们只关心"炸没炸"
        let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::diff::differential::<P>(60, 0xABCD)
        }));
        std::panic::set_hook(prev);
        out.map_err(|_| ())
    }

    #[test]
    fn repo_root_contains_crates() {
        let root = crate::repo_root();
        assert!(
            root.join("problems").is_dir(),
            "repo_root 定位失败: {}",
            root.display()
        );
        assert!(root.join("lab").is_dir());
    }
}
