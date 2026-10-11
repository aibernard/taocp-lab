//! `lab new` —— 把一道题**导入题库**。
//!
//! ## 为什么需要它
//!
//! 在它出现之前，导入一道题要手动改两个文件共三处：
//!
//! 1. `cp problems/src/_template.rs problems/src/p0049_group_anagrams.rs`
//! 2. 在 `problems/src/lib.rs` 里加 `pub mod p0049_group_anagrams;`
//! 3. 在 `for_each_problem!` 里再加一行
//!
//! 手工做这三件事有两个毛病：容易漏，而且**漏了不会提醒你**——
//! 你只是在 `lab list` 里看不到那道题，然后困惑十分钟。
//!
//! ## 它不做什么
//!
//! ⚠️ 它**只生成骨架**。生成出来的 `brute()` 里是 `todo!()`。
//! 这个工具不会、也不该替你写暴力解——那是整个仓库存在的意义。

use crate::{repo_root, ui};
use std::fs;
use std::path::Path;

/// 模板直接读 `problems/src/_template.rs`。
///
/// 这样模板改了，`lab new` 生成的东西就跟着改——**不会有两份真相**。
const TEMPLATE: &str = include_str!("../../problems/src/_template.rs");

/// 一道待导入题目的规格。
pub struct Spec {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub topic: String,
    pub taocp: String,
}

impl Spec {
    /// `p0049` + `group_anagrams` → `p0049_group_anagrams`
    pub fn module(&self) -> String {
        format!("{}_{}", self.id, self.slug)
    }

    /// `group_anagrams` → `GroupAnagrams`
    pub fn type_name(&self) -> String {
        to_pascal_case(&self.slug)
    }

    /// `p0049` → `49`（去掉前缀和补零）
    pub fn number(&self) -> u32 {
        self.id.trim_start_matches('p').parse().unwrap_or(0)
    }
}

/// 导入流程的入口。
pub fn create(
    id_raw: &str,
    slug: &str,
    title: Option<&str>,
    topic: Option<&str>,
    taocp: Option<&str>,
) -> Result<(), String> {
    let spec = Spec {
        id: normalize_id(id_raw)?,
        slug: slug.to_string(),
        title: title.unwrap_or(slug).to_string(),
        topic: topic.unwrap_or("未分类").to_string(),
        taocp: taocp.unwrap_or("—").to_string(),
    };
    validate_slug(&spec.slug)?;

    let root = repo_root();
    let target = root
        .join("problems")
        .join("src")
        .join(format!("{}.rs", spec.module()));

    if target.exists() {
        return Err(format!(
            "{} 已经存在了。要么换个 slug，要么直接去编辑它。",
            rel(&root, &target)
        ));
    }

    let body = render(&spec)?;
    if let Some(dir) = target.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("无法创建 {}: {e}", dir.display()))?;
    }
    fs::write(&target, &body).map_err(|e| format!("无法写入 {}: {e}", target.display()))?;

    // 注册失败就把刚建的文件删掉，别留下"半个导入"的烂摊子。
    if let Err(e) = register(&root, &spec) {
        let _ = fs::remove_file(&target);
        return Err(format!(
            "{e}\n（已回滚：删掉了刚生成的 {}）",
            rel(&root, &target)
        ));
    }

    print_next_steps(&root, &spec);
    Ok(())
}

// ---------------------------------------------------------------- 校验

/// `p0049` / `0049` / `49` 都归一化成 `p0049`。
fn normalize_id(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    let digits = raw.strip_prefix('p').unwrap_or(raw);
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!(
            "题目 ID 必须是 `p0049` / `0049` / `49` 这种形式，收到 `{raw}`"
        ));
    }
    let n: u32 = digits
        .parse()
        .map_err(|_| format!("题目 ID 的数字部分超出范围：`{raw}`"))?;
    Ok(format!("p{n:04}"))
}

fn validate_slug(slug: &str) -> Result<(), String> {
    if slug.is_empty() {
        return Err("slug 不能为空（例如 `group_anagrams`）".into());
    }
    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(format!("slug 只能用小写字母、数字和下划线，收到 `{slug}`"));
    }
    if !slug.starts_with(|c: char| c.is_ascii_lowercase()) {
        return Err(format!(
            "slug 必须以小写字母开头（否则生成的结构体名不是合法标识符），收到 `{slug}`"
        ));
    }
    if slug.ends_with('_') || slug.contains("__") {
        return Err(format!("slug 里不要出现连续或结尾的下划线，收到 `{slug}`"));
    }
    Ok(())
}

// ---------------------------------------------------------------- 渲染

fn render(spec: &Spec) -> Result<String, String> {
    let mut out = TEMPLATE.to_string();
    let ty = spec.type_name();
    let quoted_title = format!("\"{}\"", escape(&spec.title));

    // ① 先把"手动复制 3 件事"那段说明换掉 —— `lab new` 已经替你做完了，
    //    留着它会让新文件自相矛盾。必须在其他替换之前做，否则锚点会被改写。
    replace_once(
        &mut out,
        MANUAL_STEPS_BLOCK,
        "//! ## 导入状态\n\
         //!\n\
         //! 模块文件与 `problems/src/lib.rs` 的注册，都已由 `lab new` 自动完成。\n\
         //! 你只需要做一件事：**亲手写 `brute()`**。",
        "手动复制说明块",
    )?;

    // ② 字符串字面量：各出现一次，必须唯一。
    //    顺序有讲究：先换带引号的标题，才能安全锚定文件头里那处**不带引号**的标题。
    replace_once(
        &mut out,
        "\"pXXXX\"",
        &format!("\"{}\"", spec.id),
        "ID 常量 `\"pXXXX\"`",
    )?;
    replace_once(
        &mut out,
        "\"题目英文名 / 题目中文名\"",
        &quoted_title,
        "标题常量",
    )?;
    replace_once(
        &mut out,
        "\"未分类\"",
        &format!("\"{}\"", escape(&spec.topic)),
        "主题常量 `\"未分类\"`",
    )?;
    replace_once(
        &mut out,
        "\"—\"",
        &format!("\"{}\"", escape(&spec.taocp)),
        "TAOCP 常量 `\"—\"`",
    )?;

    // ③ 文件头注释
    replace_once(
        &mut out,
        "# PXXXX · 题目英文名 / 题目中文名",
        &format!("# {} · {}", spec.id.to_uppercase(), spec.title),
        "文件头标题行",
    )?;
    replace_once(
        &mut out,
        "LeetCode XXXX",
        &format!("LeetCode {}", spec.number()),
        "`LeetCode XXXX`",
    )?;

    // ④ 结构体名：在 `struct` 与 `impl` 各出现一次，全文替换
    replace_all(&mut out, "YourProblem", &ty, "`YourProblem`")?;

    Ok(out)
}

/// `_template.rs` 里那段"手动复制要做的 3 件事"。
///
/// 它对**手动 `cp`** 的场景是对的，但对 `lab new` 已经过时——
/// 所以生成新文件时要整块换掉，不能让它自相矛盾。
const MANUAL_STEPS_BLOCK: &str = "\
//! ## 复制这个文件后要做的 3 件事
//!
//! 1. 重命名为 `problems/src/pXXXX_short_name.rs`
//! 2. 在 `problems/src/lib.rs` 里加 `pub mod pXXXX_short_name;`
//! 3. 在 `for_each_problem!` 宏里加 `$crate::pXXXX_short_name::YourProblem,`";

/// 替换**恰好一处**；0 处或多处都算模板被改坏了，直接报错而不是猜。
fn replace_once(hay: &mut String, needle: &str, with: &str, what: &str) -> Result<(), String> {
    match hay.matches(needle).count() {
        1 => {
            *hay = hay.replacen(needle, with, 1);
            Ok(())
        }
        0 => Err(anchor_missing(what)),
        n => Err(format!(
            "{what}：在模板里出现了 {n} 次（应该只有 1 次）。\n{}",
            TEMPLATE_HINT
        )),
    }
}

/// 替换**所有**出现处；一处都没有则报错。
fn replace_all(hay: &mut String, needle: &str, with: &str, what: &str) -> Result<(), String> {
    if !hay.contains(needle) {
        return Err(anchor_missing(what));
    }
    *hay = hay.replace(needle, with);
    Ok(())
}

const TEMPLATE_HINT: &str = "模板可能被你改过了——请同步更新 `lab/src/scaffold.rs` 里的替换锚点。";

fn anchor_missing(what: &str) -> String {
    format!("{what}：在 `problems/src/_template.rs` 里找不到。\n{TEMPLATE_HINT}")
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn to_pascal_case(slug: &str) -> String {
    slug.split('_')
        .filter(|p| !p.is_empty())
        .map(|part| {
            let mut cs = part.chars();
            match cs.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + cs.as_str(),
            }
        })
        .collect()
}

// ---------------------------------------------------------------- 注册

/// 在 `problems/src/lib.rs` 里插两行：`pub mod` 和宏里的类型。
fn register(root: &Path, spec: &Spec) -> Result<(), String> {
    let path = root.join("problems").join("src").join("lib.rs");
    let src = fs::read_to_string(&path).map_err(|e| format!("读不到 {}: {e}", rel(root, &path)))?;

    let module = spec.module();
    let ty = spec.type_name();

    if src.contains(&format!("pub mod {module};")) {
        return Err(format!(
            "`problems/src/lib.rs` 里已经有 `pub mod {module};` 了（模块名撞车？）"
        ));
    }

    let mut lines: Vec<String> = src.lines().map(str::to_string).collect();

    // ① 插到最后一个 `pub mod xxx;` 后面
    let last_mod = lines
        .iter()
        .rposition(|l| l.starts_with("pub mod ") && l.trim_end().ends_with(';'))
        .ok_or(
            "在 `problems/src/lib.rs` 里找不到任何 `pub mod ...;`，无法自动注册。\n\
             请手动加两行（见 `docs/training/README.md` 第六节）。",
        )?;
    lines.insert(last_mod + 1, format!("pub mod {module};"));

    // ② 插到 `for_each_problem!` 的 `$m! { ... }` 块闭合之前
    let m_open = lines.iter().position(|l| l.trim() == "$m! {").ok_or(
        "在 `problems/src/lib.rs` 里找不到 `for_each_problem!` 的 `$m! {` 块。\n\
             请手动在宏里加一行（见 `docs/training/README.md` 第六节）。",
    )?;
    let m_close = lines[m_open..]
        .iter()
        .position(|l| l.trim() == "}")
        .map(|off| m_open + off)
        .ok_or("`for_each_problem!` 的 `$m! {` 块没有闭合，无法自动注册。")?;
    lines.insert(m_close, format!("            $crate::{module}::{ty},"));

    let mut out = lines.join("\n");
    out.push('\n');
    fs::write(&path, out).map_err(|e| format!("写不进 {}: {e}", rel(root, &path)))?;
    Ok(())
}

// ---------------------------------------------------------------- 输出

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn print_next_steps(root: &Path, spec: &Spec) {
    let file = rel(
        root,
        &root
            .join("problems")
            .join("src")
            .join(format!("{}.rs", spec.module())),
    );
    let id = &spec.id;

    println!(
        "{} 已导入 {}  {}",
        ui::green("✔"),
        ui::cyan(id),
        ui::dim(&format!("({})", spec.title))
    );
    println!("  生成 {file}");
    println!("  注册 problems/src/lib.rs（pub mod + for_each_problem!）");
    println!();
    println!("{}", ui::bold("接下来按这个顺序来："));
    println!("  1. 打开 {file}");
    println!("     填掉文件头那两个 TODO：题目的「输出规范化决定」和「分水岭」");
    println!(
        "  2. {} 由你亲手写（现在里面是 todo!()）",
        ui::bold("brute()")
    );
    println!("  3. cargo run -p lab -- render {id} --tier tiny");
    println!("  4. cargo run -p lab -- diff   {id}          ← 被最优解击穿");
    println!("  5. cargo run -p lab -- bench  {id}          ← 看爆炸点");
    println!("  6. cargo run -p lab -- run    {id} --grade 4");
    println!();
    println!(
        "  {} 在你写完 brute() 之前，`cargo test` 会失败 —— 这是故意的，",
        ui::yellow("⚠")
    );
    println!("      半成品题目不该被误当成「训练通过」。");
    println!(
        "  {} 题目清单是编译期注册的，所以要用 `cargo run -p lab -- ...`",
        ui::dim("注意")
    );
    println!("      （或先 `cargo build`）才能看到新题；直接跑旧二进制是看不到的。");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(id: &str, slug: &str) -> Spec {
        Spec {
            id: id.into(),
            slug: slug.into(),
            title: "Group Anagrams / 字母异位词分组".into(),
            topic: "哈希/排序".into(),
            taocp: "Vol.3 §6.4".into(),
        }
    }

    #[test]
    fn id_is_normalized() {
        assert_eq!(normalize_id("49").unwrap(), "p0049");
        assert_eq!(normalize_id("p49").unwrap(), "p0049");
        assert_eq!(normalize_id("0049").unwrap(), "p0049");
        assert_eq!(normalize_id("p0049").unwrap(), "p0049");
        assert_eq!(normalize_id(" 509 ").unwrap(), "p0509");
        assert_eq!(normalize_id("1").unwrap(), "p0001");
    }

    #[test]
    fn bad_ids_are_rejected() {
        for bad in ["", "p", "pabc", "49a", "p-1", "p 49"] {
            assert!(normalize_id(bad).is_err(), "`{bad}` 应该被拒绝");
        }
    }

    #[test]
    fn slug_rules() {
        assert!(validate_slug("group_anagrams").is_ok());
        assert!(validate_slug("two_sum_2").is_ok());
        assert!(validate_slug("a").is_ok());
        // 必须以小写字母开头：否则生成的结构体名不是合法标识符
        assert!(validate_slug("2sum").is_err());
        assert!(validate_slug("_sum").is_err());
        // 不允许大写/连字符/空/连续或结尾下划线
        assert!(validate_slug("Group").is_err());
        assert!(validate_slug("group-anagrams").is_err());
        assert!(validate_slug("").is_err());
        assert!(validate_slug("group_").is_err());
        assert!(validate_slug("group__anagrams").is_err());
    }

    #[test]
    fn pascal_case_conversion() {
        assert_eq!(to_pascal_case("group_anagrams"), "GroupAnagrams");
        assert_eq!(to_pascal_case("two_sum"), "TwoSum");
        assert_eq!(to_pascal_case("n_queens_2"), "NQueens2");
        assert_eq!(to_pascal_case("a"), "A");
    }

    #[test]
    fn number_strips_prefix_and_padding() {
        assert_eq!(spec("p0049", "x").number(), 49);
        assert_eq!(spec("p0001", "x").number(), 1);
        assert_eq!(spec("p1200", "x").number(), 1200);
    }

    #[test]
    fn template_substitution_leaves_no_placeholders() {
        let out = render(&spec("p0049", "group_anagrams")).expect("模板替换不该失败");

        for leftover in ["YourProblem", "pXXXX", "PXXXX", "未分类", "题目英文名"] {
            assert!(
                !out.contains(leftover),
                "生成的代码里还残留占位符 `{leftover}`"
            );
        }

        assert!(out.contains("pub struct GroupAnagrams;"));
        assert!(out.contains("impl Problem for GroupAnagrams"));
        assert!(out.contains(r#"const ID: &'static str = "p0049";"#));
        assert!(out.contains(r#"const TOPIC: &'static str = "哈希/排序";"#));
        assert!(out.contains(r#"const TAOCP: &'static str = "Vol.3 §6.4";"#));
        assert!(out.contains("# P0049 · Group Anagrams / 字母异位词分组"));
        assert!(out.contains("LeetCode 49 · 难度"));
        // 最要紧的一条：暴力解必须是未实现的，工具绝不能代写。
        assert!(out.contains("todo!"), "brute() 必须是 todo!()，不能被代写");
    }

    #[test]
    fn quotes_in_title_are_escaped() {
        let mut s = spec("p0001", "quote_test");
        s.title = r#"He said "hi""#.into();
        let out = render(&s).expect("替换不该失败");
        assert!(
            out.contains(r#"\"hi\""#),
            "标题里的引号必须被转义，否则生成的文件编译不过"
        );
    }

    /// 这个测试的价值在于：谁改了 `_template.rs` 的锚点，
    /// `lab new` 会**立刻在测试里暴露**，而不是等你导入题目时才炸。
    #[test]
    fn template_anchors_still_exist() {
        for anchor in [
            "YourProblem",
            "\"pXXXX\"",
            "\"题目英文名 / 题目中文名\"",
            "\"未分类\"",
            "\"—\"",
            "# PXXXX · 题目英文名 / 题目中文名",
            "LeetCode XXXX",
            MANUAL_STEPS_BLOCK,
        ] {
            assert!(TEMPLATE.contains(anchor), "模板锚点丢失：`{anchor}`");
        }
    }

    #[test]
    fn replacing_a_missing_anchor_is_an_error_not_a_guess() {
        let mut s = String::from("这里没有那个锚点");
        assert!(replace_once(&mut s, "找不到的东西", "x", "测试锚点").is_err());
        // 出现多次也必须报错，不能闷头替换第一个
        let mut s = String::from("dup dup");
        assert!(replace_once(&mut s, "dup", "x", "测试锚点").is_err());
    }
}
