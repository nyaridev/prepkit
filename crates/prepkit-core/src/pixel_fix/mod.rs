//! Pixel Fix: fills every fully transparent pixel with the color of the nearest visible edge pixel, keeping it
//! transparent. Filtering and scaling then blend toward the sprite's own colors instead of black or white fringes.
//!
//! Results replace the originals, go into a sub-folder, or sit next to them with a prefix (see [`Output`]).
//! Animated GIF/APNG keep only their first frame.

mod edt;
mod fill;
mod output;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use anyhow::{Context, Result};
use image::{DynamicImage, ImageFormat, ImageReader, RgbaImage};
use serde::{Deserialize, Serialize};

use crate::batch::{self, FileResult, Status, Summary};

pub use fill::fix_pixels;
pub use output::{clean_name, Output};

pub const IMAGE_EXTENSIONS: &[&str] = &["apng", "bmp", "gif", "jpg", "jpeg", "png", "tif", "tiff", "webp"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PixelFixOptions {
    /// Look inside sub-folders too.
    pub recursive: bool,
    /// Where results go.
    pub output: Output,
    /// Sub-folder name for [`Output::Folder`].
    pub folder: String,
    /// File name prefix for [`Output::Prefix`].
    pub prefix: String,
    /// Make the filled pixels opaque, to see what was filled.
    pub debug: bool,
}

impl Default for PixelFixOptions {
    fn default() -> Self {
        Self { recursive: false, output: Output::Replace, folder: "fixed".into(), prefix: "fixed_".into(), debug: false }
    }
}

/// The images that `paths` (files and folders) point to, leaving out earlier results (the output folder, or files
/// that already carry the prefix).
pub fn discover(paths: &[PathBuf], options: &PixelFixOptions) -> Vec<PathBuf> {
    batch::collect_files(paths, options.recursive, IMAGE_EXTENSIONS)
        .into_iter()
        .filter(|path| !output::is_result(path, options))
        .collect()
}

/// Fixes every image under `paths`, reporting each file as it finishes.
pub fn run<P>(paths: &[PathBuf], options: &PixelFixOptions, cancel: &AtomicBool, on_result: P) -> Summary
where
    P: Fn(&FileResult, usize, usize) + Sync,
{
    let files = discover(paths, options);
    batch::run_parallel(files, cancel, |path| fix_file(path, options), on_result)
}

/// Fixes one image and writes it where `options.output` says.
pub fn fix_file(path: &Path, options: &PixelFixOptions) -> FileResult {
    let target = output::target(path, options);
    let replacing = target == path;
    let result = match fix_image(path, &target, options.debug) {
        Ok(true) => FileResult::new(path, Status::Written, "Fixed"),
        Ok(false) if replacing => FileResult::new(path, Status::Skipped, "No transparent pixels to fix"),
        Ok(false) => FileResult::new(path, Status::Written, "Copied (nothing to fix)"),
        Err(error) => FileResult::new(path, Status::Failed, format!("{error:#}")),
    };
    if replacing { result } else { result.with_output(target) }
}

/// Returns whether any pixel was filled. When nothing needs filling, a separate target gets a plain copy, so the
/// output folder (or prefixed set) is always complete.
fn fix_image(path: &Path, target: &Path, debug: bool) -> Result<bool> {
    let reader = ImageReader::open(path)?.with_guessed_format()?;
    let format = reader.format().context("Unknown image format")?;
    let mut image = reader.decode().context("Couldn't read the image")?.to_rgba8();
    let filled = fix_pixels(&mut image, debug);

    if target != path {
        fs::create_dir_all(target.parent().context("The output has no folder")?)?;
        if !filled {
            fs::copy(path, target).context("Couldn't copy the image")?;
            return Ok(false);
        }
    } else if !filled {
        return Ok(false);
    }
    save(target, image, format)?;
    Ok(true)
}

/// Writes to a temporary file first and then moves it into place, so a failed write never leaves a broken image.
fn save(path: &Path, image: RgbaImage, format: ImageFormat) -> Result<()> {
    let mut temp = path.as_os_str().to_owned();
    temp.push(".prepkit-tmp");
    let temp = PathBuf::from(temp);
    let written = DynamicImage::ImageRgba8(image).save_with_format(&temp, format);
    if let Err(error) = written {
        let _ = fs::remove_file(&temp);
        return Err(error).context("Couldn't write the image");
    }
    fs::rename(&temp, path).context("Couldn't move the image into place")
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn sprite(path: &Path) {
        let mut image = RgbaImage::from_pixel(8, 8, Rgba([0, 0, 0, 0]));
        image.put_pixel(4, 4, Rgba([200, 50, 25, 255]));
        image.save(path).unwrap();
    }

    #[test]
    fn replace_rewrites_the_original_and_keeps_alpha() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sprite.png");
        sprite(&path);

        let result = fix_file(&path, &PixelFixOptions::default());
        assert_eq!(result.status, Status::Written, "{}", result.message);
        assert!(result.output.is_none());

        let fixed = image::open(&path).unwrap().to_rgba8();
        assert_eq!(fixed.get_pixel(0, 0), &Rgba([200, 50, 25, 0]));
        assert_eq!(fixed.get_pixel(4, 4), &Rgba([200, 50, 25, 255]));
    }

    #[test]
    fn folder_and_prefix_leave_the_original_alone() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sprite.png");
        sprite(&path);
        let original = fs::read(&path).unwrap();

        let folder = PixelFixOptions { output: Output::Folder, ..PixelFixOptions::default() };
        let result = fix_file(&path, &folder);
        assert_eq!(result.output.as_deref(), Some(dir.path().join("fixed").join("sprite.png").as_path()));
        assert!(dir.path().join("fixed").join("sprite.png").is_file());

        let prefix = PixelFixOptions { output: Output::Prefix, ..PixelFixOptions::default() };
        fix_file(&path, &prefix);
        assert!(dir.path().join("fixed_sprite.png").is_file());
        assert_eq!(fs::read(&path).unwrap(), original, "the source is untouched");
    }

    #[test]
    fn earlier_results_are_not_picked_up_again() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("fixed")).unwrap();
        for name in ["a.png", "fixed_a.png", "fixed/a.png"] {
            fs::write(dir.path().join(name), b"").unwrap();
        }
        let roots = [dir.path().to_path_buf()];
        let folder = PixelFixOptions { output: Output::Folder, recursive: true, ..PixelFixOptions::default() };
        assert_eq!(discover(&roots, &folder).len(), 2, "fixed/ is skipped");
        let prefix = PixelFixOptions { output: Output::Prefix, ..PixelFixOptions::default() };
        assert_eq!(discover(&roots, &prefix).len(), 1, "fixed_a.png is skipped");
    }

    #[test]
    fn opaque_images_are_skipped_in_place_but_copied_elsewhere() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("opaque.png");
        RgbaImage::from_pixel(4, 4, Rgba([10, 20, 30, 255])).save(&path).unwrap();
        assert_eq!(fix_file(&path, &PixelFixOptions::default()).status, Status::Skipped);

        let prefix = PixelFixOptions { output: Output::Prefix, ..PixelFixOptions::default() };
        assert_eq!(fix_file(&path, &prefix).status, Status::Written);
        assert!(dir.path().join("fixed_opaque.png").is_file());
    }
}
