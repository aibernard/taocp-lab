//! 终端输出的最小工具集：颜色、时间格式化、数字千分位。

use std::io::IsTerminal;
use std::time::Duration;

/// 只有真正挂在终端上时才输出 ANSI 颜色，重定向到文件时保持纯净。
pub fn color_on() -> bool {
    std::io::stdout().is_terminal()
}

fn paint(code: &str, s: &str) -> String {
    if color_on() {
        format!("\x1b[{code}m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

pub fn bold(s: &str) -> String {
    paint("1", s)
}
pub fn dim(s: &str) -> String {
    paint("2", s)
}
pub fn red(s: &str) -> String {
    paint("31", s)
}
pub fn green(s: &str) -> String {
    paint("32", s)
}
pub fn yellow(s: &str) -> String {
    paint("33", s)
}
pub fn cyan(s: &str) -> String {
    paint("36", s)
}

/// 人类可读的耗时。
pub fn fmt_duration(d: Duration) -> String {
    let ns = d.as_nanos();
    if ns < 1_000 {
        format!("{ns}ns")
    } else if ns < 1_000_000 {
        format!("{:.1}µs", ns as f64 / 1e3)
    } else if ns < 1_000_000_000 {
        format!("{:.1}ms", ns as f64 / 1e6)
    } else {
        format!("{:.2}s", ns as f64 / 1e9)
    }
}

/// 千分位分隔，用来把"3.3 亿次调用"这种数字变得可读。
pub fn comma(n: u128) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// 两个耗时的倍率。
pub fn ratio(a: u128, b: u128) -> String {
    if b == 0 {
        return "∞".to_string();
    }
    let r = a as f64 / b as f64;
    if r >= 1000.0 {
        format!("{r:.0}×")
    } else {
        format!("{r:.1}×")
    }
}

// ---------------------------------------------------------------- 对齐

/// 单个字符在终端里占几列。
///
/// `{:<8}` 这类格式化按**字符数**补齐，中文会被算成 1 列，
/// 于是表格必然错位。这里做一次粗略的 East-Asian Wide 判定。
fn char_width(c: char) -> usize {
    let cp = c as u32;
    let wide = (0x1100..=0x115F).contains(&cp)        // 韩文字母
        || (0x2E80..=0xA4CF).contains(&cp)            // CJK 部首、假名、汉字
        || (0xAC00..=0xD7A3).contains(&cp)            // 韩文音节
        || (0xF900..=0xFAFF).contains(&cp)            // CJK 兼容
        || (0xFE30..=0xFE6F).contains(&cp)            // CJK 兼容标点
        || (0xFF00..=0xFF60).contains(&cp)            // 全角形式
        || (0xFFE0..=0xFFE6).contains(&cp)
        || (0x20000..=0x3FFFD).contains(&cp); // CJK 扩展
    if wide { 2 } else { 1 }
}

/// 字符串在终端里占用的列数。
pub fn display_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

/// 按显示宽度右补空格。**注意：要对未着色的纯文本调用**，
/// 否则 ANSI 转义序列会被算进宽度。
pub fn pad(s: &str, width: usize) -> String {
    let w = display_width(s);
    if w >= width {
        s.to_string()
    } else {
        format!("{s}{}", " ".repeat(width - w))
    }
}

/// 按显示宽度左补空格（右对齐）。
pub fn pad_left(s: &str, width: usize) -> String {
    let w = display_width(s);
    if w >= width {
        s.to_string()
    } else {
        format!("{}{s}", " ".repeat(width - w))
    }
}

/// 按**显示宽度**截断，超出部分用 `…` 占最后一列。
///
/// `str::chars().take(n)` 会按字符数截断，中文标题照样能撑破列宽；
/// 表格对齐必须用这个。
pub fn truncate_width(s: &str, width: usize) -> String {
    if display_width(s) <= width {
        return s.to_string();
    }
    let budget = width.saturating_sub(1);
    let mut out = String::new();
    let mut used = 0;
    for ch in s.chars() {
        let w = char_width(ch);
        if used + w > budget {
            break;
        }
        out.push(ch);
        used += w;
    }
    out.push('…');
    out
}
