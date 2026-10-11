//! 单次训练会话：对决 → 评级 → 落台账 → 排下次复习。

use crate::{diff, ledger, ui};
use problems::Problem;

pub fn execute<P: Problem>(
    cases: usize,
    seed: u64,
    grade_override: Option<u8>,
) -> Result<(), String> {
    let report = diff::differential::<P>(cases, seed);
    diff::print(&report);

    let today = ledger::today();
    let entries = ledger::load();
    let schedule = ledger::replay(&entries);
    let prev = schedule.get(P::ID).cloned();

    // 自动评级：全过 = 4（独立解出）；有任何一处不一致 = 1（还没掌握）
    let auto_grade = if report.all_pass() { 4 } else { 1 };
    let grade = grade_override.unwrap_or(auto_grade).min(5);

    let next = ledger::sm2(
        &prev.clone().unwrap_or_else(ledger::Sched::fresh),
        grade,
        today,
    );

    let entry = ledger::Entry {
        day: today,
        ts: ledger::now_secs(),
        problem: P::ID.to_string(),
        topic: P::TOPIC.to_string(),
        verdict: if report.all_pass() {
            "PASS".into()
        } else {
            "MISMATCH".into()
        },
        cases: report.total(),
        passed: report.passed(),
        brute_ns: report.brute_time.as_nanos(),
        fast_ns: report.fast_time.as_nanos(),
        grade,
        interval: next.interval,
        due_day: next.due_day,
    };
    ledger::append(&entry)?;

    println!("{}", ui::bold("② 训练台账"));
    println!(
        "  {} 评级 {}/5{}   下次复习 {} （{} 天后）",
        ui::green("已记录"),
        grade,
        if grade_override.is_some() {
            "（手动）"
        } else {
            "（自动）"
        },
        ui::cyan(&ledger::date_str(next.due_day)),
        next.interval
    );

    if let Some(p) = &prev
        && p.due_day <= today
        && p.runs > 0
    {
        println!(
            "  {} 上次的到期日是 {}，已逾期 {} 天 —— 这正是「停止刷题就遗忘」的现场",
            ui::yellow("⚠"),
            ledger::date_str(p.due_day),
            today.saturating_sub(p.due_day)
        );
    }

    if !report.all_pass() {
        println!();
        println!(
            "  {} 别急着改。回到上面那个失败用例，先回答一个问题：",
            ui::yellow("下一步")
        );
        println!("     是**边界条件**没覆盖到（越界、空输入、极值），还是**逻辑**本身分叉了？");
        println!("     前者去补 `adversarial()`，后者才需要动 `brute()`。");
    }

    println!();
    Ok(())
}
