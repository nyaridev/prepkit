//! The pixel work: find the visible pixels that touch transparency, then copy each one's color into the transparent
//! pixels nearest to it.

use image::RgbaImage;

use super::edt;

/// The 8 neighbors of a pixel, as (dx, dy).
const NEIGHBORS: [(isize, isize); 8] = [(-1, -1), (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0)];

/// Fills the RGB of every fully transparent pixel with its nearest edge pixel's color. Alpha is left as it was,
/// unless `debug` is set (then filled pixels become opaque). Returns whether any pixel was filled.
pub fn fix_pixels(image: &mut RgbaImage, debug: bool) -> bool {
    let (width, height) = (image.width() as usize, image.height() as usize);
    if width == 0 || height == 0 {
        return false;
    }

    let alpha: Vec<u8> = image.pixels().map(|pixel| pixel[3]).collect();
    if !alpha.contains(&0) {
        return false;
    }
    let edges = edge_mask(&alpha, width, height);
    if !edges.contains(&true) {
        return false;
    }

    let nearest = edt::nearest_sites(&edges, width, height);
    let raw: &mut [u8] = image.as_mut();
    let mut filled = false;
    for (index, source) in nearest.into_iter().enumerate() {
        let Some(source) = source else { continue };
        if alpha[index] != 0 {
            continue;
        }
        let (from, to) = (source * 4, index * 4);
        raw.copy_within(from..from + 3, to);
        if debug {
            raw[to + 3] = 255;
        }
        filled = true;
    }
    filled
}

/// Visible pixels (alpha > 0) with at least one fully transparent neighbor.
fn edge_mask(alpha: &[u8], width: usize, height: usize) -> Vec<bool> {
    let mut edges = vec![false; width * height];
    for y in 0..height {
        for x in 0..width {
            if alpha[y * width + x] == 0 {
                continue;
            }
            edges[y * width + x] = NEIGHBORS.iter().any(|&(dx, dy)| {
                let (nx, ny) = (x as isize + dx, y as isize + dy);
                nx >= 0
                    && ny >= 0
                    && (nx as usize) < width
                    && (ny as usize) < height
                    && alpha[ny as usize * width + nx as usize] == 0
            });
        }
    }
    edges
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn transparent_pixels_take_the_nearest_edge_color() {
        let mut image = RgbaImage::from_pixel(5, 1, Rgba([0, 0, 0, 0]));
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        image.put_pixel(4, 0, Rgba([0, 0, 255, 128]));

        assert!(fix_pixels(&mut image, false));
        assert_eq!(image.get_pixel(1, 0), &Rgba([255, 0, 0, 0]));
        assert_eq!(image.get_pixel(3, 0), &Rgba([0, 0, 255, 0]));
        assert_eq!(image.get_pixel(4, 0), &Rgba([0, 0, 255, 128]), "visible pixels are untouched");
    }

    #[test]
    fn debug_makes_filled_pixels_opaque() {
        let mut image = RgbaImage::from_pixel(2, 1, Rgba([0, 0, 0, 0]));
        image.put_pixel(0, 0, Rgba([9, 9, 9, 255]));
        fix_pixels(&mut image, true);
        assert_eq!(image.get_pixel(1, 0), &Rgba([9, 9, 9, 255]));
    }

    #[test]
    fn fully_transparent_or_opaque_images_are_left_alone() {
        let mut clear = RgbaImage::from_pixel(3, 3, Rgba([1, 2, 3, 0]));
        assert!(!fix_pixels(&mut clear, false));
        let mut solid = RgbaImage::from_pixel(3, 3, Rgba([1, 2, 3, 255]));
        assert!(!fix_pixels(&mut solid, false));
    }
}
