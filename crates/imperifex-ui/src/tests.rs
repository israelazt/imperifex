use imperifex_domain::i18n::{text, Language, Text};
use imperifex_domain::theme::{palette, Theme};

use super::*;

fn model(theme: Theme) -> Model {
    Model {
        title: text(Language::Spanish, Text::AppName).into(),
        language: Language::Spanish,
        palette: palette(theme),
        scale: 1.25,
        window: (800, 600),
        maximized: false,
        focused: true,
    }
}

#[test]
fn without_the_mouse_nothing_is_asked() {
    let found = frame(&model(Theme::Dark), &Pointer::default());
    assert!(found.actions.is_empty());
    assert_eq!(found.cursor, CursorIcon::Default);
}

#[test]
fn the_output_goes_under_the_bar_with_the_base_space_around_it() {
    let found = frame(&model(Theme::Dark), &Pointer::default());
    let expected = Rect {
        x: 20,
        y: 70,
        width: 760,
        height: 510,
    };
    assert_eq!(found.terminal_area, expected);
}

#[test]
fn the_colours_are_those_of_the_models_theme() {
    for theme in Theme::ALL {
        let found = frame(&model(theme), &Pointer::default());
        let colours: Vec<_> = found.draw.rects.iter().map(|&(_, colour)| colour).collect();
        let expected = [palette(theme).background, palette(theme).surface];
        assert_eq!(colours[..2], expected);
    }
}

#[test]
fn what_the_bar_is_asked_comes_out_of_the_frame() {
    let pointer = Pointer {
        position: Some((300, 20)),
        pressed: true,
        ..Pointer::default()
    };
    let found = frame(&model(Theme::Dark), &pointer);
    assert_eq!(found.actions, [Action::Drag]);
}
