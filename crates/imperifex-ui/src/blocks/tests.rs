use imperifex_domain::draw::Icon;
use imperifex_domain::i18n::Language;
use imperifex_domain::theme::{palette, Theme};
use imperifex_domain::view::{CommandRow, RanView};

use super::*;
use crate::testing::FixedWidth;

/// A window of 800 by 600 at 100 % with the blocks given.
fn model(blocks: Vec<BlockView>) -> Model {
    Model {
        title: String::new(),
        tab: None,
        language: Language::Spanish,
        palette: palette(Theme::Dark),
        scale: 1.0,
        window: (800, 600),
        maximized: false,
        focused: true,
        full_screen: false,
        notice: Vec::new(),
        editor: None,
        warning: None,
        blocks,
    }
}

/// A block from 100 to 160 whose command, on its first row, left `free`.
fn block(free: u32, code: i32) -> BlockView {
    BlockView {
        id: 4,
        top: 100,
        height: 60,
        starts_here: true,
        command: Some(CommandRow {
            y: 100,
            height: 20,
            free,
        }),
        ran: Some(RanView {
            code,
            took: Some("0,1 s".into()),
        }),
        selected: false,
    }
}

fn at(x: i32, y: i32) -> Pointer {
    Pointer {
        position: Some((x, y)),
        ..Pointer::default()
    }
}

fn pressed(x: i32, y: i32) -> Pointer {
    Pointer {
        pressed: true,
        ..at(x, y)
    }
}

fn drawn(model: &Model, pointer: &Pointer) -> DrawList {
    let mut list = DrawList::default();
    draw_behind(model, 40, &mut list);
    draw_labels(model, pointer, &FixedWidth, &mut list);
    list
}

fn said(list: &DrawList) -> Vec<String> {
    list.texts.iter().map(|run| run.text.clone()).collect()
}

#[test]
fn a_line_goes_over_each_block_that_starts_in_view_but_not_at_the_very_top() {
    let model_with = |starts_here, top| {
        model(vec![BlockView {
            starts_here,
            top,
            ..block(700, 0)
        }])
    };
    let lines = |model: &Model| {
        let list = drawn(model, &Pointer::default());
        let color = model.palette.line;
        let found = list.rects.iter().filter(|rect| rect.1 == color);
        found.map(|rect| rect.0).collect::<Vec<_>>()
    };
    let line = Rect {
        x: 16,
        y: 100,
        width: 768,
        height: 1,
    };
    assert_eq!(lines(&model_with(true, 100)), [line]);
    assert!(lines(&model_with(false, 100)).is_empty());
    assert!(lines(&model_with(true, 40)).is_empty());
}

#[test]
fn a_good_end_is_a_green_check_and_a_bad_one_a_red_cross_with_its_code() {
    let good = model(vec![block(700, 0)]);
    let found = drawn(&good, &Pointer::default());
    assert_eq!(
        said(&found),
        [Icon::Check.character().to_string(), "0,1 s".into()]
    );
    assert_eq!(found.texts[0].color, good.palette.ok);
    assert_eq!(found.texts[1].color, good.palette.muted);
    // The check ends at the right side of the output.
    assert_eq!(found.texts[0].x + 14, 784);
    let bad = model(vec![block(700, 101)]);
    let found = drawn(&bad, &Pointer::default());
    let cross = Icon::Cross.character().to_string();
    assert_eq!(said(&found), ["101".into(), cross, "0,1 s".into()]);
    let colours: Vec<_> = found.texts.iter().map(|run| run.color).collect();
    let error = bad.palette.error;
    assert_eq!(colours[..2], [error, error]);
}

#[test]
fn where_they_would_cover_text_they_show_only_under_the_mouse_and_on_top() {
    let model = model(vec![block(50, 0)]);
    let away = drawn(&model, &at(300, 400));
    assert!(said(&away).is_empty() && away.floating_texts.is_empty());
    let over = drawn(&model, &at(300, 130));
    assert!(over.texts.is_empty());
    assert_eq!(over.floating_texts.len(), 2);
    let patch = over.floating_rects[0];
    assert_eq!(patch.1, model.palette.surface);
    assert_eq!((patch.0.y, patch.0.height), (100, 20));
}

#[test]
fn the_chosen_block_is_raised_and_has_the_button_that_copies() {
    let mut chosen = block(700, 0);
    chosen.selected = true;
    let model = model(vec![chosen]);
    let found = drawn(&model, &Pointer::default());
    let back = found.rounded[0];
    assert_eq!(back.color, model.palette.raised);
    assert_eq!((back.area.x, back.area.width, back.radius), (8, 784, 4));
    assert!(said(&found).contains(&"Copiar salida".to_string()));
    let button = found.rounded[1].area;
    let press = |x, y| input(&model, &pressed(x, y), &FixedWidth);
    let inside = (button.x + 5, button.y + 5);
    assert_eq!(press(inside.0, inside.1), Some(Action::CopyBlock(4)));
    assert_eq!(press(20, 140), Some(Action::SelectBlock(4)));
}

#[test]
fn only_a_press_on_a_block_that_ran_something_chooses_it() {
    let mut waiting = block(700, 0);
    (waiting.command, waiting.ran) = (None, None);
    let empty = model(vec![waiting]);
    assert_eq!(input(&empty, &pressed(20, 120), &FixedWidth), None);
    let ran = model(vec![block(700, 0)]);
    assert_eq!(input(&ran, &pressed(20, 500), &FixedWidth), None);
    assert_eq!(input(&ran, &at(20, 120), &FixedWidth), None, "not pressed");
}
