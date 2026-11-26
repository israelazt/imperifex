//! Where a key shows (D27): told from what a few presses changed. Whatever
//! changes anywhere else, such as a clock the terminal draws, is not the key.

use crate::desktop::Rect;

/// How far apart each edge of two presses' areas may be.
const SAME_PLACE: i32 = 4;
/// Added on every side of what the presses changed.
const MARGIN: i32 = 24;
/// The most of the watched region the zone may take, in percent.
const MOST_PERCENT: i64 = 10;
const NOT_ONE_PLACE: &str = "la tecla no cambia siempre el mismo sitio";

fn edges(area: Rect) -> [i32; 4] {
    [area.x, area.y, area.x + area.w, area.y + area.h]
}

fn shown(area: Rect) -> String {
    format!("{}x{} en ({}, {})", area.w, area.h, area.x, area.y)
}

/// The zone of the key, from the areas that presses of it changed in a region of
/// this size. Every area has to be where the first one is: one that brought some
/// other change with it must not widen the zone.
pub fn key_zone(areas: &[Rect], width: i32, height: i32) -> Result<Rect, String> {
    let first = *areas.first().ok_or("no hay pulsaciones previas")?;
    let apart = |area: &Rect| {
        edges(first)
            .iter()
            .zip(edges(*area))
            .any(|(a, b)| (a - b).abs() > SAME_PLACE)
    };
    if let Some((number, other)) = (1..).zip(areas).find(|(_, area)| apart(area)) {
        return Err(format!(
            "{NOT_ONE_PLACE}: las pulsaciones previas no coinciden (la 1 cambió {}; la {number}, {})",
            shown(first),
            shown(*other)
        ));
    }
    let [mut left, mut top, mut right, mut bottom] = edges(first);
    for [x, y, end_x, end_y] in areas.iter().map(|&a| edges(a)) {
        (left, top) = (left.min(x), top.min(y));
        (right, bottom) = (right.max(end_x), bottom.max(end_y));
    }
    let (left, top) = ((left - MARGIN).max(0), (top - MARGIN).max(0));
    let zone = Rect {
        x: left,
        y: top,
        w: (right + MARGIN).min(width) - left,
        h: (bottom + MARGIN).min(height) - top,
    };
    let (taken, all) = (
        i64::from(zone.w) * i64::from(zone.h),
        i64::from(width) * i64::from(height),
    );
    if taken * 100 > all * MOST_PERCENT {
        return Err(format!(
            "{NOT_ONE_PLACE}: su zona, {}, pasa del {MOST_PERCENT} % de la región",
            shown(zone)
        ));
    }
    Ok(zone)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn the_zone_is_what_the_presses_changed_with_a_margin() {
        let areas = [area(100, 200, 10, 20), area(104, 200, 10, 20)];
        assert_eq!(key_zone(&areas, 1280, 680), Ok(area(76, 176, 62, 68)));
    }

    #[test]
    fn the_margin_stops_at_the_edges_of_the_region() {
        let areas = [area(5, 660, 10, 20)];
        assert_eq!(key_zone(&areas, 1280, 680), Ok(area(0, 636, 39, 44)));
    }

    #[test]
    fn an_area_somewhere_else_or_of_another_size_is_refused() {
        let letter = area(100, 200, 10, 20);
        for other in [area(105, 200, 10, 20), area(100, 150, 10, 70)] {
            let error = key_zone(&[letter, letter, other], 1280, 680).unwrap_err();
            assert!(error.contains("la 3,"), "{error}");
        }
    }

    #[test]
    fn a_zone_over_a_tenth_of_the_region_is_refused() {
        // 100 by 100 once the margin is on, in 1000 and in 999 rows of 100.
        let areas = [area(24, 24, 52, 52)];
        assert!(key_zone(&areas, 100, 1000).is_ok());
        assert!(key_zone(&areas, 100, 999).unwrap_err().contains("10 %"));
    }

    #[test]
    fn with_no_press_there_is_no_zone() {
        assert!(key_zone(&[], 10, 10).is_err());
    }
}
