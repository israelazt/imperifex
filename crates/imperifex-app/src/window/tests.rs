use imperifex_domain::theme::ThemeChoice;

use super::*;
use crate::testing::FakeAppearance;

mod output;

fn core_with(choice: ThemeChoice, appearance: &FakeAppearance) -> Core {
    let settings = Settings {
        theme: choice,
        ..Settings::default()
    };
    Core::new(settings, Language::English, Box::new(appearance.clone()))
}

#[test]
fn the_model_holds_what_the_window_said_in_the_chosen_language() {
    let mut core = core_with(ThemeChoice::System, &FakeAppearance::dark(false));
    core.resized((800, 600), true);
    core.set_scale(1.5);
    core.set_focused(false);
    let model = core.model();
    assert_eq!(model.window, (800, 600));
    assert!(model.maximized && !model.focused);
    assert!((model.scale - 1.5).abs() < f32::EPSILON);
    assert_eq!(model.language, Language::English);
    assert_eq!(model.title, "Imperifex");
    assert_eq!(model.palette, palette(Theme::Light));
}

#[test]
fn a_change_of_the_systems_theme_changes_the_palette_when_the_system_decides() {
    let appearance = FakeAppearance::dark(true);
    let mut core = core_with(ThemeChoice::System, &appearance);
    assert_eq!(core.model().palette, palette(Theme::Dark));
    appearance.set(Ok(false));
    assert_eq!(core.system_theme_changed(), [WindowCommand::Redraw]);
    assert_eq!(core.model().palette, palette(Theme::Light));
    assert_eq!(core.system_theme_changed(), []);
}

#[test]
fn a_chosen_theme_stays_when_the_systems_changes() {
    let appearance = FakeAppearance::dark(true);
    let mut core = core_with(ThemeChoice::Dark, &appearance);
    appearance.set(Ok(false));
    assert_eq!(core.system_theme_changed(), []);
    assert_eq!(core.model().palette, palette(Theme::Dark));
}

#[test]
fn a_system_that_does_not_say_its_theme_gives_dark() {
    let appearance = FakeAppearance::dark(false);
    appearance.set(Err("no key".into()));
    let core = core_with(ThemeChoice::System, &appearance);
    assert_eq!(core.model().palette, palette(Theme::Dark));
}

#[test]
fn watching_the_system_hands_it_what_to_call() {
    let appearance = FakeAppearance::dark(true);
    let core = core_with(ThemeChoice::System, &appearance);
    core.watch_system(std::sync::Arc::new(|| {})).unwrap();
    assert_eq!(appearance.watched(), 1);
}

#[test]
fn each_action_is_an_order_and_maximize_goes_back_and_forth() {
    let mut core = core_with(ThemeChoice::System, &FakeAppearance::dark(true));
    for (action, command) in [
        (Action::Minimize, WindowCommand::Minimize),
        (Action::Close, WindowCommand::Close),
        (Action::Drag, WindowCommand::Drag),
        (
            Action::Resize(Edge::East),
            WindowCommand::Resize(Edge::East),
        ),
        (Action::SystemMenu, WindowCommand::ShowSystemMenu),
        (Action::ToggleMaximize, WindowCommand::SetMaximized(true)),
    ] {
        assert_eq!(core.apply(action), [command]);
    }
    core.resized((800, 600), true);
    let restore = core.apply(Action::ToggleMaximize);
    assert_eq!(restore, [WindowCommand::SetMaximized(false)]);
}

#[test]
fn only_what_differs_from_what_is_shown_is_news() {
    let mut core = core_with(ThemeChoice::System, &FakeAppearance::dark(true));
    let empty = DrawList::default();
    assert!(core.is_news(&empty));
    core.was_shown(empty.clone());
    assert!(!core.is_news(&empty));
    let mut other = empty;
    other
        .rects
        .push((Default::default(), palette(Theme::Dark).text));
    assert!(core.is_news(&other));
}
