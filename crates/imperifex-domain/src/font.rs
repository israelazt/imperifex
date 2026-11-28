//! Sizes that come out of a font: of the grid's cell, from what the font says.

/// Typographic points in an inch, and logical pixels in one.
const POINTS_PER_INCH: f32 = 72.0;
const PIXELS_PER_INCH: f32 = 96.0;

/// What a font says about itself, in its own design units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontMetrics {
    pub units_per_em: u16,
    /// From the baseline up, and down.
    pub ascent: u16,
    pub descent: u16,
    /// Added between one line and the next.
    pub line_gap: i16,
    /// How far the pen moves after a `0`.
    pub zero_advance: u32,
}

impl FontMetrics {
    /// Design units in physical pixels for letters of `em` pixels. A font that
    /// says it has no units gives 0.
    pub fn pixels(&self, units: f32, em: f32) -> f32 {
        if self.units_per_em == 0 {
            return 0.0;
        }
        units * em / f32::from(self.units_per_em)
    }

    /// From the top of a line to the next one, in design units.
    pub fn line_units(&self) -> f32 {
        f32::from(self.ascent) + f32::from(self.descent) + f32::from(self.line_gap.max(0))
    }
}

/// A cell of the grid in whole physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CellSize {
    pub width: u32,
    pub height: u32,
    /// From the top of the cell down to the line the letters stand on.
    pub baseline: u32,
}

impl CellSize {
    /// How many columns and rows fit in an area of that many pixels: at least
    /// one of each, and no more than a console takes. What is left over stays
    /// empty at the right and at the bottom.
    pub fn fit(self, width: u32, height: u32) -> (u16, u16) {
        let count = |room: u32, side: u32| {
            let whole = room / side.max(1);
            u16::try_from(whole).unwrap_or(u16::MAX).clamp(1, 1000)
        };
        (count(width, self.width), count(height, self.height))
    }
}

/// The size in physical pixels of letters of `points` at the window's `scale`.
pub fn pixels_of_points(points: f32, scale: f32) -> f32 {
    points * scale * PIXELS_PER_INCH / POINTS_PER_INCH
}

/// The cell for a font at `points` and the window's `scale`, rounded to whole
/// pixels so that every cell falls on the same ones. Never smaller than 1×1.
pub fn cell_size(metrics: &FontMetrics, points: f32, scale: f32) -> CellSize {
    let em = pixels_of_points(points, scale);
    let whole = |units: f32| metrics.pixels(units, em).round().max(0.0) as u32;
    let height = whole(metrics.line_units()).max(1);
    CellSize {
        width: whole(metrics.zero_advance as f32).max(1),
        height,
        baseline: whole(f32::from(metrics.ascent)).min(height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round numbers, close to those of a monospaced font.
    const FONT: FontMetrics = FontMetrics {
        units_per_em: 2000,
        ascent: 1800,
        descent: 500,
        line_gap: 0,
        zero_advance: 1200,
    };

    fn cell(width: u32, height: u32, baseline: u32) -> CellSize {
        CellSize {
            width,
            height,
            baseline,
        }
    }

    #[test]
    fn twelve_points_are_sixteen_pixels_and_grow_with_the_scale() {
        assert!((pixels_of_points(12.0, 1.0) - 16.0).abs() < 0.001);
        assert_eq!(cell_size(&FONT, 12.0, 1.0), cell(10, 18, 14));
        assert_eq!(cell_size(&FONT, 12.0, 1.25), cell(12, 23, 18));
        assert_eq!(cell_size(&FONT, 12.0, 1.5), cell(14, 28, 22));
    }

    #[test]
    fn the_line_gap_makes_the_cell_taller_unless_it_is_negative() {
        let with = |line_gap| {
            let font = FontMetrics { line_gap, ..FONT };
            cell_size(&font, 12.0, 1.0).height
        };
        assert_eq!(with(250), 20);
        assert_eq!(with(-250), 18);
    }

    #[test]
    fn only_whole_cells_fit_with_at_least_one_and_at_most_a_thousand() {
        let cell = cell(9, 19, 15);
        assert_eq!(cell.fit(900, 380), (100, 20));
        assert_eq!(cell.fit(908, 398), (100, 20));
        assert_eq!(cell.fit(0, 5), (1, 1));
        assert_eq!(cell.fit(100_000, 100_000), (1000, 1000));
        assert_eq!(CellSize::default().fit(5, 5), (5, 5));
    }

    #[test]
    fn a_nonsense_font_or_scale_still_gives_a_cell_of_one_pixel() {
        let empty = FontMetrics {
            units_per_em: 0,
            ..FONT
        };
        assert_eq!(cell_size(&empty, 12.0, 1.0), cell(1, 1, 0));
        assert_eq!(cell_size(&FONT, 12.0, -1.0), cell(1, 1, 0));
    }
}
