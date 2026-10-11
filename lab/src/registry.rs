//! 题目注册表。
//!
//! 这里是全仓库唯一的"题目清单来源"：`problems` 里的一个宏
//! （[`problems::for_each_problem`]）同时驱动了
//! CLI 派发、`lab list` 目录、`lab due` 复习队列和 `lab stats` 统计。
//!
//! 新增题目**不需要**改这个文件，只需要改 `problems/src/lib.rs` 里的那个宏。

use problems::Problem;

/// 题目的只读元信息（全部是 `&'static str`，可以随便复制）。
#[derive(Clone, Copy, Debug)]
pub struct Meta {
    pub id: &'static str,
    pub title: &'static str,
    pub topic: &'static str,
    pub taocp: &'static str,
    /// 参考样板题：`brute()` 随仓库附带，不是你写的。见 [`Problem::REFERENCE`]。
    pub reference: bool,
}

/// 从命令行解析出来的、需要派发到具体题目的动作。
#[derive(Clone, Copy, Debug)]
pub enum Cmd {
    Diff {
        cases: usize,
        seed: u64,
    },
    Bench,
    Render {
        tier: problems::Tier,
        seed: u64,
    },
    Run {
        cases: usize,
        seed: u64,
        grade: Option<u8>,
    },
}

macro_rules! impl_dispatch {
    ($($ty:ty),* $(,)?) => {
        /// 把命令派发给唯一匹配 ID 的那道题。
        pub fn dispatch(id: &str, cmd: &Cmd) -> Result<(), String> {
            $(
                if <$ty as Problem>::ID == id {
                    return match cmd {
                        Cmd::Diff { cases, seed } => {
                            let report = crate::diff::differential::<$ty>(*cases, *seed);
                            crate::diff::print(&report);
                            Ok(())
                        }
                        Cmd::Bench => crate::bench::curve::<$ty>(),
                        Cmd::Render { tier, seed } => {
                            crate::bench::render::<$ty>(*tier, *seed);
                            Ok(())
                        }
                        Cmd::Run { cases, seed, grade } => {
                            crate::session::execute::<$ty>(*cases, *seed, *grade)
                        }
                    };
                }
            )*
            Err(format!("未知题目 ID `{id}`（用 `lab list` 看看已注册的题目）"))
        }

        /// 全部已注册题目。
        pub fn catalog() -> Vec<Meta> {
            vec![
                $(Meta {
                    id: <$ty as Problem>::ID,
                    title: <$ty as Problem>::TITLE,
                    topic: <$ty as Problem>::TOPIC,
                    taocp: <$ty as Problem>::TAOCP,
                    reference: <$ty as Problem>::REFERENCE,
                }),*
            ]
        }
    };
}

problems::for_each_problem!(impl_dispatch);
