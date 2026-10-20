//! Where a fixed image goes: over the original, into a sub-folder, or next to it with a prefix.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::PixelFixOptions;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Output {
    /// Overwrite the original.
    #[default]
    Replace,
    /// `<dir>/<folder>/<name>`.
    Folder,
    /// `<dir>/<prefix><name>`.
    Prefix,
}

/// Keeps a folder name or prefix to a single safe file-name part.
pub fn clean_name(value: &str) -> String {
    value.chars().filter(|c| !matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|')).collect::<String>().trim().to_string()
}

/// Where `input` is written.
pub fn target(input: &Path, options: &PixelFixOptions) -> PathBuf {
    let name = input.file_name().unwrap_or_default();
    let dir = input.parent().unwrap_or(Path::new("."));
    match options.output {
        Output::Replace => input.to_path_buf(),
        Output::Folder => dir.join(folder_name(options)).join(name),
        Output::Prefix => dir.join(format!("{}{}", prefix(options), name.to_string_lossy())),
    }
}

/// Whether `path` is itself an earlier result that shouldn't be fixed again.
pub fn is_result(path: &Path, options: &PixelFixOptions) -> bool {
    match options.output {
        Output::Replace => false,
        Output::Folder => path
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|parent| parent.to_string_lossy().eq_ignore_ascii_case(&folder_name(options))),
        Output::Prefix => path.file_name().is_some_and(|name| name.to_string_lossy().starts_with(&prefix(options))),
    }
}

fn folder_name(options: &PixelFixOptions) -> String {
    let name = clean_name(&options.folder);
    if name.is_empty() || name == "." || name == ".." { "fixed".into() } else { name }
}

fn prefix(options: &PixelFixOptions) -> String {
    let prefix = clean_name(&options.prefix);
    if prefix.is_empty() { "fixed_".into() } else { prefix }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_follow_the_mode() {
        let input = Path::new("ui").join("coin.png");
        let mut options = PixelFixOptions::default();
        assert_eq!(target(&input, &options), input);

        options.output = Output::Folder;
        options.folder = "clean".into();
        assert_eq!(target(&input, &options), Path::new("ui").join("clean").join("coin.png"));

        options.output = Output::Prefix;
        options.prefix = "fx_".into();
        assert_eq!(target(&input, &options), Path::new("ui").join("fx_coin.png"));
    }

    #[test]
    fn unsafe_names_are_cleaned_or_defaulted() {
        assert_eq!(clean_name("../evil"), "..evil");
        let options = PixelFixOptions { output: Output::Folder, folder: "..".into(), ..PixelFixOptions::default() };
        assert_eq!(target(Path::new("a.png"), &options), Path::new("fixed").join("a.png"));
    }
}
