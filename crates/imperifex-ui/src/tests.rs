use imperifex_domain::draw::UiFont;
use imperifex_domain::i18n::{text, Language, Text};
use imperifex_domain::theme::{palette, Theme};

use super::*;

struct NoText;

impl TextMeasure for NoText {
    fn width(&self, _text: &str, _font: UiFont, _scale: f32) -> u32 {
        0
    }

    fn line_height(&self, _font: UiFont, _scale: f32) -> u32 {
        0
    }
}

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
    let found = frame(&model(Theme::Dark), &Pointer::default(), &NoText);
    assert!(found.actions.is_empty());
    assert_eq!(found.cursor, CursorIcon::Default);
}

#[test]
fn the_output_goes_inside_the_window_with_the_base_space_around_it() {
    let found = frame(&model(Theme::Dark), &Pointer::default(), &NoText);
    let expected = Rect {
        x: 20,
        y: 20,
        width: 760,
        height: 560,
    };
    assert_eq!(found.terminal_area, expected);
}

#[test]
fn the_colours_are_those_of_the_models_theme() {
    for theme in Theme::ALL {
        let found = frame(&model(theme), &Pointer::default(), &NoText);
        let colours: Vec<_> = found.draw.rects.iter().map(|&(_, colour)| colour).collect();
        let expected = [palette(theme).background, palette(theme).surface];
        assert_eq!(colours, expected);
    }
}
