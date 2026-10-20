//! What every tool reports, and the shared helpers that run a tool over many files.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use rayon::prelude::*;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Written,
    Skipped,
    Failed,
}

/// The outcome for one input file.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileResult {
    pub path: PathBuf,
    /// The file that was written, when it differs from `path` (audio writes into `normalized/`).
    pub output: Option<PathBuf>,
    pub status: Status,
    pub message: String,
}

impl FileResult {
    pub fn new(path: &Path, status: Status, message: impl Into<String>) -> Self {
        Self { path: path.to_path_buf(), output: None, status, message: message.into() }
    }

    pub fn with_output(mut self, output: PathBuf) -> Self {
        self.output = Some(output);
        self
    }
}

/// The outcome of a whole run.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub total: usize,
    pub written: usize,
    pub skipped: usize,
    pub failed: usize,
    pub cancelled: bool,
    pub results: Vec<FileResult>,
}

impl Summary {
    pub fn from_results(total: usize, results: Vec<FileResult>, cancelled: bool) -> Self {
        let count = |status| results.iter().filter(|r| r.status == status).count();
        Self {
            total,
            written: count(Status::Written),
            skipped: count(Status::Skipped),
            failed: count(Status::Failed),
            cancelled,
            results,
        }
    }
}

/// Runs `work` on every file in parallel and reports each result as it finishes (with how many are done so far).
/// Files that haven't started when `cancel` is set are left out of the summary.
pub fn run_parallel<W, P>(files: Vec<PathBuf>, cancel: &AtomicBool, work: W, on_result: P) -> Summary
where
    W: Fn(&Path) -> FileResult + Sync,
    P: Fn(&FileResult, usize, usize) + Sync,
{
    let total = files.len();
    let done = AtomicUsize::new(0);
    let results: Vec<FileResult> = files
        .par_iter()
        .filter_map(|path| {
            if cancel.load(Ordering::Relaxed) {
                return None;
            }
            let result = work(path);
            let finished = done.fetch_add(1, Ordering::Relaxed) + 1;
            on_result(&result, finished, total);
            Some(result)
        })
        .collect();
    Summary::from_results(total, results, cancel.load(Ordering::Relaxed))
}

/// Expands files and folders into the files whose extension is in `extensions`, sorted and without duplicates.
/// Folders are read one level deep unless `recursive` is set. Files given directly are kept only if they match.
pub fn collect_files(paths: &[PathBuf], recursive: bool, extensions: &[&str]) -> Vec<PathBuf> {
    let mut found = BTreeSet::new();
    for path in paths {
        if path.is_dir() {
            collect_dir(path, recursive, extensions, &mut found);
        } else if path.is_file() && has_extension(path, extensions) {
            found.insert(path.clone());
        }
    }
    found.into_iter().collect()
}

fn collect_dir(dir: &Path, recursive: bool, extensions: &[&str], found: &mut BTreeSet<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                collect_dir(&path, recursive, extensions, found);
            }
        } else if has_extension(&path, extensions) {
            found.insert(path);
        }
    }
}

pub fn has_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| extensions.iter().any(|wanted| wanted.eq_ignore_ascii_case(ext)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_matching_files_from_folders_and_direct_paths() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("nested");
        fs::create_dir(&nested).unwrap();
        for name in ["a.png", "b.PNG", "c.txt"] {
            fs::write(dir.path().join(name), b"").unwrap();
        }
        fs::write(nested.join("d.png"), b"").unwrap();

        let flat = collect_files(&[dir.path().to_path_buf()], false, &["png"]);
        assert_eq!(flat.len(), 2);

        let deep = collect_files(&[dir.path().to_path_buf(), nested.join("d.png")], true, &["png"]);
        assert_eq!(deep.len(), 3, "duplicates are removed");
    }

    #[test]
    fn summary_counts_statuses() {
        let path = Path::new("x");
        let results = vec![
            FileResult::new(path, Status::Written, ""),
            FileResult::new(path, Status::Skipped, ""),
            FileResult::new(path, Status::Failed, ""),
            FileResult::new(path, Status::Written, ""),
        ];
        let summary = Summary::from_results(5, results, true);
        assert_eq!((summary.written, summary.skipped, summary.failed), (2, 1, 1));
        assert!(summary.cancelled);
    }
}
