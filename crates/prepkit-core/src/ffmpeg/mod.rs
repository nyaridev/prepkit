//! Finding ffmpeg, and installing it for the user when it's missing.

mod install;
mod locate;
mod user_path;

pub use install::{install, install_dir, InstallOptions, InstallProgress, Stage};
pub use locate::{command, locate, version_of, FfmpegInfo};
pub use user_path::with_entry;
