use imperifex_domain::draw::Icon;
use imperifex_domain::i18n::{text, Language, Text};
use imperifex_domain::theme::{palette, Theme};

use super::*;
use crate::testing::FixedWidth;

fn model(theme: Theme) -> Model {
    Model {
        title: text(Language::Spanish, Text::AppName).into(),
        tab: Some("PowerShell".into()),
        language: Language::Spanish,
        palette: palette(theme),
        scale: 1.25,
        window: (800, 600),
        maximized: false,
        focused: true,
        full_screen: false,
        notice: Vec::new(),
        editor: None,
        warning: None,
    }
}

#[test]
fn without_the_mouse_nothing_is_asked() {
    let found = frame(&model(Theme::Dark), &Pointer::default(), &FixedWidth);
    assert!(found.actions.is_empty());
    assert_eq!(found.cursor, CursorIcon::Default);
}

#[test]
fn the_output_goes_between_the_two_bars_with_the_base_space_around_it() {
    let found = frame(&model(Theme::Dark), &Pointer::default(), &FixedWidth);
    let expected = Rect {
        x: 20,
        y: 70,
        width: 760,
        height: 475,
    };
    assert_eq!(found.terminal_area, expected);
}

#[test]
fn the_colours_are_those_of_the_models_theme() {
    for theme in Theme::ALL {
        let found = frame(&model(theme), &Pointer::default(), &FixedWidth);
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
    let found = frame(&model(Theme::Dark), &pointer, &FixedWidth);
    assert_eq!(found.actions, [Action::Drag]);
}

#[test]
fn a_program_that_uses_the_whole_screen_gets_no_margins() {
    let model = Model {
        full_screen: true,
        ..model(Theme::Dark)
    };
    let found = frame(&model, &Pointer::default(), &FixedWidth);
    let under_the_bar = Rect {
        x: 0,
        y: 50,
        width: 800,
        height: 515,
    };
    assert_eq!(found.terminal_area, under_the_bar);
}

#[test]
fn each_line_of_a_notice_is_written_where_the_output_goes() {
    let model = Model {
        notice: vec!["first".into(), "second".into()],
        ..model(Theme::Dark)
    };
    let found = frame(&model, &Pointer::default(), &FixedWidth);
    let said: Vec<_> = found
        .draw
        .texts
        .iter()
        .filter(|run| run.color == model.palette.error)
        .map(|run| (run.x, run.y, run.text.as_str(), run.color))
        .collect();
    let error = model.palette.error;
    // At 125 % the lines of the fake font are 20 high.
    assert_eq!(said, [(20, 70, "first", error), (20, 90, "second", error)]);
}

#[test]
fn the_bar_at_the_bottom_says_the_profile_of_the_shell() {
    let model = model(Theme::Dark);
    let found = frame(&model, &Pointer::default(), &FixedWidth);
    let bar = Rect {
        x: 0,
        y: 565,
        width: 800,
        height: 35,
    };
    assert!(found.draw.rects.contains(&(bar, model.palette.background)));
    let said: Vec<_> = found
        .draw
        .texts
        .iter()
        .filter(|run| run.y >= bar.y)
        .map(|run| (run.text.clone(), run.x, run.size, run.color))
        .collect();
    // At 125 %: 20 from the side, an icon of 15, 5 and the name.
    let icon = Icon::Shell.character().to_string();
    let muted = model.palette.muted;
    assert_eq!(
        said,
        [
            (icon, 20, 15, muted),
            ("PowerShell".to_string(), 40, 15, muted)
        ]
    );
}

#[test]
fn a_warning_goes_against_the_right_of_the_status_bar() {
    let model = Model {
        warning: Some("ojo".into()),
        ..model(Theme::Dark)
    };
    let found = frame(&model, &Pointer::default(), &FixedWidth).draw;
    let said = found.texts.iter().find(|run| run.text == "ojo");
    let said = said.expect("the warning");
    // 3 letters of 15 and 20 from the side, in a window of 800.
    assert_eq!((said.x, said.color), (735, model.palette.warning));
}
