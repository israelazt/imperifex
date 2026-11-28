use imperifex_domain::i18n::Language;
use imperifex_domain::theme::{palette, Theme};

use super::*;

fn model(scale: f32, maximized: bool) -> Model {
    Model {
        title: String::new(),
        language: Language::Spanish,
        palette: palette(Theme::Dark),
        scale,
        window: (1000, 600),
        maximized,
        focused: true,
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
        held_from: Some((x, y)),
        ..at(x, y)
    }
}

fn released(from: (i32, i32), x: i32, y: i32) -> Pointer {
    Pointer {
        released: true,
        held_from: Some(from),
        ..at(x, y)
    }
}

fn asked(model: &Model, pointer: &Pointer) -> Option<Action> {
    input(model, pointer).0
}

#[test]
fn the_three_buttons_sit_against_the_right_at_the_windows_scale() {
    let areas = |scale| Button::ALL.map(|button| button_area(button, 1000, scale));
    let found = areas(1.0).map(|area| (area.x, area.width, area.height));
    assert_eq!(found, [(862, 46, 40), (908, 46, 40), (954, 46, 40)]);
    let found = areas(1.25).map(|area| (area.x, area.width, area.height));
    assert_eq!(found, [(826, 58, 50), (884, 58, 50), (942, 58, 50)]);
}

#[test]
fn a_button_acts_on_release_where_the_press_began() {
    let model = model(1.0, false);
    for (x, action) in [
        (880, Action::Minimize),
        (930, Action::ToggleMaximize),
        (970, Action::Close),
    ] {
        assert_eq!(asked(&model, &pressed(x, 20)), None);
        assert_eq!(asked(&model, &released((x, 20), x, 20)), Some(action));
    }
}

#[test]
fn a_release_after_pressing_elsewhere_does_nothing() {
    let model = model(1.0, false);
    assert_eq!(asked(&model, &released((880, 20), 970, 20)), None);
    assert_eq!(asked(&model, &released((300, 20), 970, 20)), None);
    assert_eq!(asked(&model, &released((970, 20), 970, 200)), None);
}

#[test]
fn the_strip_drags_maximizes_with_two_clicks_and_has_the_menu() {
    let model = model(1.0, false);
    assert_eq!(asked(&model, &pressed(300, 20)), Some(Action::Drag));
    let double = Pointer {
        double_click: true,
        ..pressed(300, 20)
    };
    assert_eq!(asked(&model, &double), Some(Action::ToggleMaximize));
    let right = Pointer {
        right_click: true,
        ..at(300, 20)
    };
    assert_eq!(asked(&model, &right), Some(Action::SystemMenu));
    assert_eq!(asked(&model, &at(300, 20)), None);
    assert_eq!(asked(&model, &pressed(300, 40)), None);
}

#[test]
fn the_eight_pixels_of_each_edge_resize_and_say_so_with_the_cursor() {
    let model = model(1.0, false);
    for ((x, y), edge) in [
        ((500, 7), Edge::North),
        ((500, 592), Edge::South),
        ((7, 300), Edge::West),
        ((992, 300), Edge::East),
        ((0, 0), Edge::NorthWest),
        ((999, 0), Edge::NorthEast),
        ((0, 599), Edge::SouthWest),
        ((999, 599), Edge::SouthEast),
    ] {
        let found = input(&model, &pressed(x, y));
        let expected = (Some(Action::Resize(edge)), CursorIcon::Resize(edge));
        assert_eq!(found, expected, "{x},{y}");
    }
    assert_eq!(input(&model, &at(500, 8)).1, CursorIcon::Default);
    assert_eq!(input(&model, &at(8, 300)).1, CursorIcon::Default);
}

#[test]
fn a_maximized_window_has_no_edges_and_its_corner_closes() {
    let model = model(1.0, true);
    let found = input(&model, &released((999, 0), 999, 0));
    assert_eq!(found, (Some(Action::Close), CursorIcon::Default));
    assert_eq!(asked(&model, &pressed(500, 0)), Some(Action::Drag));
}

fn drawn(model: &Model, pointer: &Pointer) -> DrawList {
    let mut list = DrawList::default();
    draw(model, pointer, &mut list);
    list
}

fn background_of(list: &DrawList, area: Rect) -> Option<Rgb> {
    list.rects
        .iter()
        .find(|(found, _)| *found == area)
        .map(|&(_, colour)| colour)
}

#[test]
fn only_the_button_under_the_mouse_is_lit_and_close_turns_red() {
    let model = model(1.0, false);
    let area = |button| button_area(button, 1000, 1.0);
    let over_close = drawn(&model, &at(970, 20));
    let red = Some(model.palette.error);
    assert_eq!(background_of(&over_close, area(Button::Close)), red);
    assert_eq!(background_of(&over_close, area(Button::Minimize)), None);
    let over_minimize = drawn(&model, &at(880, 20));
    let raised = Some(model.palette.raised);
    assert_eq!(
        background_of(&over_minimize, area(Button::Minimize)),
        raised
    );
    assert_eq!(background_of(&over_minimize, area(Button::Close)), None);
}

#[test]
fn moving_over_the_strip_does_not_change_what_is_drawn() {
    let model = model(1.0, false);
    let away = drawn(&model, &Pointer::default());
    assert_eq!(drawn(&model, &at(300, 20)), away);
    assert_eq!(drawn(&model, &at(500, 300)), away);
}

#[test]
fn the_icon_of_maximize_changes_when_maximized_and_the_icons_dim_without_focus() {
    let normal = drawn(&model(1.0, false), &Pointer::default());
    let maximized = drawn(&model(1.0, true), &Pointer::default());
    assert_ne!(normal, maximized);
    let unfocused = Model {
        focused: false,
        ..model(1.0, false)
    };
    let dim = drawn(&unfocused, &Pointer::default());
    let muted = unfocused.palette.muted;
    assert!(dim.rects.iter().any(|&(_, colour)| colour == muted));
    assert!(!normal.rects.iter().any(|&(_, colour)| colour == muted));
}
