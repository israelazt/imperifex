use imperifex_domain::draw::{Icon, Rounded, TextRun, UiFont};
use imperifex_domain::i18n::Language;
use imperifex_domain::theme::{palette, Theme};

use super::*;
use crate::testing::FixedWidth;

fn model(scale: f32, maximized: bool) -> Model {
    Model {
        title: "Imperifex".into(),
        tab: Some("PowerShell".into()),
        language: Language::Spanish,
        palette: palette(Theme::Dark),
        scale,
        window: (1000, 600),
        maximized,
        focused: true,
        full_screen: false,
        notice: Vec::new(),
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
    draw(model, pointer, &FixedWidth, &mut list);
    draw_help(model, pointer, &FixedWidth, &mut list);
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
    let red = Some(model.palette.close_hover);
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

/// Those of the window's buttons.
fn icons(list: &DrawList) -> Vec<&TextRun> {
    list.texts
        .iter()
        .filter(|run| run.font == UiFont::Icons && run.size == 10)
        .collect()
}

#[test]
fn each_button_has_its_icon_in_its_middle_and_maximized_offers_to_restore() {
    let characters = |list: &DrawList| {
        let all: String = icons(list).iter().map(|run| run.text.clone()).collect();
        all
    };
    let of = |icons: [Icon; 3]| icons.map(Icon::character).iter().collect::<String>();
    let normal = drawn(&model(1.0, false), &Pointer::default());
    let expected = of([Icon::Minimize, Icon::Maximize, Icon::Close]);
    assert_eq!(characters(&normal), expected);
    let maximized = drawn(&model(1.0, true), &Pointer::default());
    let expected = of([Icon::Minimize, Icon::Restore, Icon::Close]);
    assert_eq!(characters(&maximized), expected);
    // The fake font's icon is 10 wide and 14 high, in a button of 46 by 40.
    let found: Vec<_> = icons(&normal).iter().map(|run| (run.x, run.y)).collect();
    assert_eq!(found, [(880, 13), (926, 13), (972, 13)]);
}

#[test]
fn the_tab_of_the_shell_has_its_icon_and_name_on_a_raised_rounded_box() {
    let model = model(1.0, false);
    let found = drawn(&model, &Pointer::default());
    let area = Rect {
        x: 8,
        y: 4,
        width: 178,
        height: 32,
    };
    let expected = Rounded {
        area,
        radius: 6,
        color: model.palette.raised,
    };
    assert_eq!(found.rounded, [expected]);
    let said: Vec<_> = found.texts[..2]
        .iter()
        .map(|run| (run.text.clone(), run.x, run.y, run.size))
        .collect();
    let icon = Icon::Shell.character().to_string();
    assert_eq!(
        said,
        [(icon, 20, 10, 16), ("PowerShell".into(), 44, 11, 13)]
    );
}

#[test]
fn the_tab_grows_with_the_windows_scale() {
    let found = drawn(&model(1.25, false), &Pointer::default());
    let name = &found.texts[1];
    assert_eq!((name.x, name.y, name.size), (55, 15, 16));
    assert_eq!(
        (found.rounded[0].area.height, found.rounded[0].radius),
        (40, 8)
    );
}

#[test]
fn there_is_no_tab_without_a_shell_or_where_it_does_not_fit() {
    let none = Model {
        tab: None,
        ..model(1.0, false)
    };
    let narrow = Model {
        window: (300, 600),
        ..model(1.0, false)
    };
    for model in [none, narrow] {
        let found = drawn(&model, &Pointer::default());
        assert!(found.rounded.is_empty());
        assert_eq!(found.texts.len(), 3, "only the buttons' icons");
    }
}

#[test]
fn a_press_on_the_tab_moves_the_window_as_on_the_rest_of_the_strip() {
    let model = model(1.0, false);
    assert_eq!(asked(&model, &pressed(60, 20)), Some(Action::Drag));
}

#[test]
fn the_icon_that_closes_turns_white_on_its_red() {
    let model = model(1.0, false);
    let found = drawn(&model, &at(970, 20));
    let close = icons(&found)[2];
    assert_eq!(close.color, model.palette.on_close_hover);
}

#[test]
fn without_focus_the_tab_and_the_icons_dim() {
    let unfocused = Model {
        focused: false,
        ..model(1.0, false)
    };
    let dim = drawn(&unfocused, &Pointer::default());
    let muted = unfocused.palette.muted;
    assert!(dim.texts.iter().all(|run| run.color == muted));
    let normal = drawn(&model(1.0, false), &Pointer::default());
    let text = model(1.0, false).palette.text;
    assert!(normal.texts.iter().all(|run| run.color == text));
}

fn resting(x: i32, y: i32) -> Pointer {
    Pointer {
        rested: true,
        ..at(x, y)
    }
}

fn help_in(list: &DrawList) -> Option<&TextRun> {
    list.floating_texts.first()
}

#[test]
fn a_mouse_resting_on_a_button_is_told_what_it_does_in_the_chosen_language() {
    for (language, maximized, x, said) in [
        (Language::Spanish, false, 880, "Minimizar"),
        (Language::Spanish, false, 930, "Maximizar"),
        (Language::Spanish, true, 930, "Restaurar"),
        (Language::English, false, 970, "Close"),
    ] {
        let model = Model {
            language,
            ..model(1.0, maximized)
        };
        let found = drawn(&model, &resting(x, 20));
        let help = help_in(&found).map(|run| run.text.as_str());
        assert_eq!(help, Some(said));
    }
}

#[test]
fn the_help_waits_for_the_rest_and_is_only_for_buttons() {
    let model = model(1.0, false);
    assert_eq!(help_in(&drawn(&model, &at(930, 20))), None);
    assert_eq!(help_in(&drawn(&model, &resting(300, 20))), None);
    assert_eq!(help_in(&drawn(&model, &resting(930, 200))), None);
}

#[test]
fn the_help_stays_inside_the_window_under_the_bar() {
    let spanish = model(1.0, false);
    let english = Model {
        language: Language::English,
        ..spanish.clone()
    };
    // "Minimize" is 96 wide and its box 112: centred under its button.
    let found = drawn(&english, &resting(880, 20));
    assert_eq!(help_in(&found).map(|run| (run.x, run.y)), Some((837, 52)));
    // "Cerrar" is 72 and its box 88: centred it would stick out, so it is
    // pushed in to leave 4.
    let found = drawn(&spanish, &resting(970, 20));
    let help = found.floating_rects[0].0;
    assert_eq!((help.x, help.width, help.y), (908, 88, 44));
}
