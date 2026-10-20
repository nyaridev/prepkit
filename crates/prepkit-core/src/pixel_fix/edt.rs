//! Exact Euclidean distance transform (Felzenszwalb & Huttenlocher) that also returns, for every pixel, which site
//! is nearest. Rows first, then columns using the row distances as costs. Ported from pixel-fix's Python fallback.

const INF: f64 = f64::INFINITY;

/// For every pixel, the index (`y * width + x`) of the nearest `true` pixel in `sites`, or `None` if there are none.
pub fn nearest_sites(sites: &[bool], width: usize, height: usize) -> Vec<Option<usize>> {
    let mut row_distance = vec![INF; width * height];
    let mut row_nearest_x = vec![-1i32; width * height];

    for y in 0..height {
        let row = &sites[y * width..(y + 1) * width];
        let positions: Vec<i32> = (0..width).filter(|&x| row[x]).map(|x| x as i32).collect();
        let (distances, nearest) = edt_1d(width, &positions, None);
        row_distance[y * width..(y + 1) * width].copy_from_slice(&distances);
        row_nearest_x[y * width..(y + 1) * width].copy_from_slice(&nearest);
    }

    let mut result = vec![None; width * height];
    for x in 0..width {
        let rows: Vec<i32> = (0..height).filter(|&y| row_distance[y * width + x].is_finite()).map(|y| y as i32).collect();
        let costs: Vec<f64> = rows.iter().map(|&y| row_distance[y as usize * width + x]).collect();
        let (_, nearest_y) = edt_1d(height, &rows, Some(&costs));
        for (y, &source_y) in nearest_y.iter().enumerate() {
            if source_y < 0 {
                continue;
            }
            let source_y = source_y as usize;
            let source_x = row_nearest_x[source_y * width + x] as usize;
            result[y * width + x] = Some(source_y * width + source_x);
        }
    }
    result
}

/// 1-D squared distance transform over `length` positions with parabolas rooted at `sites` (sorted ascending), each
/// raised by its cost. Returns the distance and the nearest site per position (`-1` when there are no sites).
fn edt_1d(length: usize, sites: &[i32], costs: Option<&[f64]>) -> (Vec<f64>, Vec<i32>) {
    if sites.is_empty() {
        return (vec![INF; length], vec![-1; length]);
    }
    let cost = |index: usize| costs.map_or(0.0, |c| c[index]);

    // The lower envelope: which parabolas are visible, and where each one starts.
    let mut envelope = vec![0usize; sites.len()];
    let mut boundaries = vec![0.0f64; sites.len() + 1];
    let mut top = 0usize;
    boundaries[0] = -INF;
    boundaries[1] = INF;

    for index in 1..sites.len() {
        let mut start;
        loop {
            let previous = envelope[top];
            start = intersection(sites[index], cost(index), sites[previous], cost(previous));
            if start > boundaries[top] || top == 0 {
                break;
            }
            top -= 1;
        }
        top += 1;
        envelope[top] = index;
        boundaries[top] = start;
        boundaries[top + 1] = INF;
    }

    let mut distances = vec![0.0; length];
    let mut nearest = vec![0i32; length];
    let mut current = 0usize;
    for position in 0..length {
        while boundaries[current + 1] < position as f64 {
            current += 1;
        }
        let index = envelope[current];
        let delta = position as f64 - sites[index] as f64;
        distances[position] = delta * delta + cost(index);
        nearest[position] = sites[index];
    }
    (distances, nearest)
}

/// Where the parabola at `site` starts to beat the one at `previous`.
fn intersection(site: i32, site_cost: f64, previous: i32, previous_cost: f64) -> f64 {
    let (s, p) = (site as f64, previous as f64);
    let denominator = 2.0 * (s - p);
    if denominator == 0.0 {
        return if site_cost >= previous_cost { INF } else { -INF };
    }
    ((site_cost + s * s) - (previous_cost + p * p)) / denominator
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny deterministic generator, so the test needs no extra crate.
    fn next(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    fn squared_distance(a: usize, b: usize, width: usize) -> usize {
        let (ax, ay, bx, by) = (a % width, a / width, b % width, b / width);
        ax.abs_diff(bx).pow(2) + ay.abs_diff(by).pow(2)
    }

    #[test]
    fn matches_brute_force_on_random_masks() {
        let mut state = 0x9E37_79B9_7F4A_7C15;
        for _ in 0..200 {
            let width = (next(&mut state) % 23 + 1) as usize;
            let height = (next(&mut state) % 23 + 1) as usize;
            let density = next(&mut state) % 30 + 1;
            let sites: Vec<bool> = (0..width * height).map(|_| next(&mut state) % 100 < density).collect();
            let site_indices: Vec<usize> = (0..sites.len()).filter(|&i| sites[i]).collect();

            let nearest = nearest_sites(&sites, width, height);
            for (pixel, found) in nearest.iter().enumerate() {
                let best = site_indices.iter().map(|&s| squared_distance(pixel, s, width)).min();
                match (found, best) {
                    (Some(site), Some(best)) => {
                        assert!(sites[*site], "nearest must be a site");
                        assert_eq!(squared_distance(pixel, *site, width), best);
                    }
                    (None, None) => {}
                    other => panic!("mismatch at {pixel}: {other:?}"),
                }
            }
        }
    }
}
