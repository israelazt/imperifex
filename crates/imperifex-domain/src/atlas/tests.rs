use super::*;

fn key(glyph: u16) -> GlyphKey {
    GlyphKey {
        face: 1,
        glyph,
        size: 16 * 64,
    }
}

fn picture(width: u32, height: u32) -> Slot {
    Slot {
        width,
        height,
        left: -1,
        top: -9,
        ..Slot::default()
    }
}

fn place(atlas: &mut Atlas, glyph: u16, width: u32, height: u32) -> Option<(u32, u32)> {
    let slot = atlas.insert(key(glyph), picture(width, height));
    slot.map(|s| (s.x, s.y))
}

#[test]
fn glyphs_go_side_by_side_and_a_new_shelf_goes_under_the_last() {
    let mut atlas = Atlas::new(32);
    assert_eq!(place(&mut atlas, 1, 20, 10), Some((0, 0)));
    assert_eq!(place(&mut atlas, 2, 12, 8), Some((20, 0)));
    assert_eq!(place(&mut atlas, 3, 5, 12), Some((0, 10)));
    assert_eq!(place(&mut atlas, 4, 30, 10), Some((0, 22)));
}

#[test]
fn a_glyph_goes_on_the_lowest_shelf_that_holds_it() {
    let mut atlas = Atlas::new(32);
    place(&mut atlas, 1, 10, 14);
    place(&mut atlas, 2, 30, 8);
    place(&mut atlas, 3, 20, 14);
    place(&mut atlas, 4, 2, 9);
    assert_eq!(place(&mut atlas, 5, 2, 7), Some((30, 14)));
}

#[test]
fn it_remembers_each_glyph_with_where_its_picture_goes() {
    let mut atlas = Atlas::new(32);
    let given = atlas.insert(key(7), picture(4, 6));
    assert_eq!(atlas.get(key(7)), given);
    assert_eq!(given.map(|s| (s.left, s.top)), Some((-1, -9)));
    assert_eq!(atlas.get(key(8)), None);
    let other_size = GlyphKey { size: 64, ..key(7) };
    assert_eq!(atlas.get(other_size), None);
}

#[test]
fn a_glyph_with_nothing_to_see_takes_no_room() {
    let mut atlas = Atlas::new(8);
    place(&mut atlas, 1, 8, 8);
    let space = atlas.insert(key(2), picture(0, 0));
    assert_eq!(space.map(|s| s.width), Some(0));
    assert!(atlas.get(key(2)).is_some());
}

#[test]
fn when_full_it_says_so_and_after_clearing_it_has_room_again() {
    let mut atlas = Atlas::new(8);
    assert_eq!(place(&mut atlas, 1, 8, 5), Some((0, 0)));
    assert_eq!(place(&mut atlas, 2, 4, 4), None);
    assert_eq!(atlas.get(key(2)), None);
    atlas.clear();
    assert_eq!(atlas.get(key(1)), None);
    assert_eq!(place(&mut atlas, 2, 4, 4), Some((0, 0)));
}

#[test]
fn what_is_larger_than_the_whole_texture_never_fits() {
    let mut atlas = Atlas::new(8);
    assert_eq!(place(&mut atlas, 1, 9, 1), None);
    assert_eq!(place(&mut atlas, 2, 1, 9), None);
}

#[test]
fn with_many_glyphs_none_overlaps_another_or_leaves_the_texture() {
    let mut atlas = Atlas::new(128);
    let mut taken: Vec<Slot> = Vec::new();
    // A fixed sequence of sizes from 1 to 17.
    for glyph in 0..400u16 {
        let size = |step: u32| 1 + (u32::from(glyph) * step) % 17;
        let Some(slot) = atlas.insert(key(glyph), picture(size(7), size(11))) else {
            break;
        };
        assert!(slot.x + slot.width <= 128 && slot.y + slot.height <= 128);
        for other in &taken {
            let apart = slot.x + slot.width <= other.x
                || other.x + other.width <= slot.x
                || slot.y + slot.height <= other.y
                || other.y + other.height <= slot.y;
            assert!(apart, "{slot:?} over {other:?}");
        }
        taken.push(slot);
    }
    assert!(taken.len() > 50, "{}", taken.len());
}
