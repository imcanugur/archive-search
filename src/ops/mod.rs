mod hash_util;
pub mod journal;
pub mod merger;

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MergeTemplate {
    #[default]
    DashJoin,
    UnderscoreJoin,
    Custom,
}

impl MergeTemplate {
    pub fn format_name(self, a: &str, b: &str, custom: &str) -> String {
        match self {
            Self::DashJoin => format!("{a}-{b}"),
            Self::UnderscoreJoin => format!("{a}_{b}"),
            Self::Custom => custom.replace("{A}", a).replace("{B}", b),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictAction {
    MoveDirect,
    SkipDuplicate,
    RenameSuffix,
}

#[derive(Debug, Clone)]
pub struct PlannedOp {
    pub src: PathBuf,
    pub dst: PathBuf,
    pub action: ConflictAction,
}

#[derive(Debug, Clone)]
pub struct MergePlan {
    pub target_dir: PathBuf,
    pub ops: Vec<PlannedOp>,
    pub move_count: usize,
    pub duplicate_count: usize,
    pub rename_count: usize,
}

#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub created_dir: PathBuf,
    pub moves: Vec<MoveRecord>,
}
