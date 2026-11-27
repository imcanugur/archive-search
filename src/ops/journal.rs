use std::fs;

use crate::error::AppError;
use crate::ops::merger::move_file_safe;
use crate::ops::JournalEntry;

#[derive(Debug, Default)]
pub struct UndoJournal {
    entries: Vec<JournalEntry>,
}

impl UndoJournal {
    pub fn push(&mut self, entry: JournalEntry) {
        self.entries.push(entry);
    }

    pub fn can_undo(&self) -> bool {
        !self.entries.is_empty()
    }

    pub fn undo_last(&mut self) -> Result<Option<usize>, AppError> {
        let Some(entry) = self.entries.pop() else {
            return Ok(None);
        };
        let mut restored = 0;
        for mv in entry.moves.iter().rev() {
            if mv.to.exists() {
                move_file_safe(&mv.to, &mv.from)?;
                restored += 1;
            }
        }
        let _ = fs::remove_dir(&entry.created_dir);
        Ok(Some(restored))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::UndoJournal;
    use crate::ops::merger::{execute_merge, plan_merge};

    #[test]
    fn merge_deduplicates_renames_and_undoes() -> Result<(), Box<dyn std::error::Error>> {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let base = std::env::temp_dir().join(format!("archive_search_test_{stamp}"));
        let dir_a = base.join("A");
        let dir_b = base.join("B");
        let target = base.join("A-B");
        fs::create_dir_all(&dir_a)?;
        fs::create_dir_all(&dir_b)?;

        fs::write(dir_a.join("same.txt"), b"identical")?;
        fs::write(dir_b.join("same.txt"), b"identical")?;
        fs::write(dir_a.join("diff.txt"), b"version-a")?;
        fs::write(dir_b.join("diff.txt"), b"version-b")?;

        let plan = plan_merge(&dir_a, &dir_b, target.clone())?;
        assert_eq!(plan.duplicate_count, 1);
        assert_eq!(plan.rename_count, 1);
        assert_eq!(plan.move_count, 3);

        let entry = execute_merge(&plan)?;
        assert!(target.join("same.txt").exists());
        assert!(target.join("diff.txt").exists());
        assert!(target.join("diff_1.txt").exists());

        let mut journal = UndoJournal::default();
        journal.push(entry);
        let restored = journal.undo_last()?;
        assert_eq!(restored, Some(3));
        assert!(dir_a.join("diff.txt").exists());
        assert!(dir_b.join("diff.txt").exists());

        let _ = fs::remove_dir_all(&base);
        Ok(())
    }
}
