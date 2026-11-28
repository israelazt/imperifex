//! The icon of a shell, a bare prompt `>_`: the one of the design, which the
//! icon fonts of Windows do not have. It is worked out at the size asked for,
//! so that it is sharp at every scale.

/// The design draws it in a box of 16 by 16, with these measures.
const BOX: f32 = 16.0;
const STROKE: f32 = 1.5;
/// Each line from one end to the other: the two of the `>` and the `_`.
const LINES: [[(f32, f32); 2]; 3] = [
    [(3.0, 4.0), (7.0, 8.0)],
    [(7.0, 8.0), (3.0, 12.0)],
    [(8.75, 12.0), (13.25, 12.0)],
];

/// The largest side worked out: far more than an icon at any scale.
const MOST: u32 = 256;

/// How much of each pixel of a square of `side` pixels the icon covers, row by
/// row: `side × side` of them. A side over 256 gives that of 256.
pub fn coverage(side: u32) -> Vec<u8> {
    let side = side.min(MOST);
    let scale = side as f32 / BOX;
    (0..side)
        .flat_map(|y| (0..side).map(move |x| (x, y)))
        .map(|(x, y)| at((x as f32 + 0.5, y as f32 + 0.5), scale))
        .collect()
}

/// That of the pixel whose middle is at `point`. The ends of the lines are
/// round, and a pixel an edge crosses is covered in part.
fn at(point: (f32, f32), scale: f32) -> u8 {
    let nearest = LINES
        .iter()
        .map(|&[from, to]| distance(point, scaled(from, scale), scaled(to, scale)))
        .fold(f32::MAX, f32::min);
    let covered = (0.5 + STROKE * scale / 2.0 - nearest).clamp(0.0, 1.0);
    // From 0 to 255 before the cast, which then cannot wrap.
    (covered * 255.0).round() as u8
}

fn scaled((x, y): (f32, f32), scale: f32) -> (f32, f32) {
    (x * scale, y * scale)
}

/// From `point` to the closest place of the line between two others.
fn distance(point: (f32, f32), from: (f32, f32), to: (f32, f32)) -> f32 {
    let (along_x, along_y) = (to.0 - from.0, to.1 - from.1);
    let (x, y) = (point.0 - from.0, point.1 - from.1);
    let length = along_x * along_x + along_y * along_y;
    // How far along the line the closest place is, from 0 to 1.
    let part = if length > 0.0 {
        ((x * along_x + y * along_y) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (x - part * along_x).hypot(y - part * along_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How much of the pixel at a place of the design's box is covered, in a
    /// square of `side` pixels.
    fn of(found: &[u8], side: u32, x: f32, y: f32) -> u8 {
        let pixel = |value: f32| (value * side as f32 / BOX) as u32;
        found[(pixel(y) * side + pixel(x)) as usize]
    }

    #[test]
    fn the_lines_are_covered_and_what_is_away_from_them_is_not() {
        for side in [16, 20, 24, 32, 64] {
            let found = coverage(side);
            assert_eq!(found.len(), (side * side) as usize);
            // The middle of each of the three lines, and the tip.
            for (x, y) in [(4.5, 5.5), (4.5, 10.5), (11.0, 12.0), (6.8, 8.0)] {
                let covered = of(&found, side, x, y);
                assert!(covered > 150, "{side}: ({x}, {y}) {covered}");
            }
            // The corners, inside the `>` and over the `_`.
            for (x, y) in [(0.5, 0.5), (15.5, 15.5), (3.0, 8.0), (11.0, 8.0)] {
                assert_eq!(of(&found, side, x, y), 0, "{side}: ({x}, {y})");
            }
        }
    }

    #[test]
    fn its_edges_are_smoothed() {
        let found = coverage(16);
        assert!(found.iter().any(|&covered| covered > 0 && covered < 255));
    }

    #[test]
    fn the_greater_than_is_the_same_above_and_below() {
        let (side, found) = (32u32, coverage(32));
        // Its rows 4 to 12 of the box, left of the underscore.
        let part = |y: u32| &found[(y * side) as usize..(y * side + 15) as usize];
        for offset in 0..8 {
            assert_eq!(part(15 - offset), part(16 + offset), "{offset}");
        }
    }

    #[test]
    fn nothing_and_too_much_are_taken_in_bounds() {
        assert!(coverage(0).is_empty());
        assert_eq!(coverage(100_000).len(), (MOST * MOST) as usize);
    }
}
