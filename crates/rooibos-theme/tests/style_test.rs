use ratatui::style::{Styled, Stylize};
use rooibos_theme::{
    Color, ColorPalette, ColorPaletteColorThemeExt, ColorScheme, Modifier, SetTheme, Style,
    TermProfile,
};
use rstest::rstest;

#[test]
fn set_color() {
    assert_eq!("a".fg(Color::Red), "a".fg(ratatui::style::Color::Red));
}

#[test]
fn set_style() {
    assert_eq!(
        "a".set_style(Style::new().fg(Color::Red)),
        "a".set_style(ratatui::style::Style::new().fg(ratatui::style::Color::Red))
    );
    assert_eq!(
        "a".set_style(Style::new().fg(Color::Red)),
        "a".set_style(ratatui::style::Style::new().fg(Color::Red.into()))
    );
}

#[rstest]
#[case(TermProfile::TrueColor)]
#[case(TermProfile::Ansi256)]
#[case(TermProfile::Ansi16)]
#[case(TermProfile::NoColor)]
#[case(TermProfile::NoTty)]
fn profile_color(#[case] profile: TermProfile) {
    profile.set_local();
    let color = Color::Rgb(120, 67, 84);
    let color_adapted: ratatui::style::Color =
        profile.adapt_color(color.into()).unwrap_or_default();
    assert_eq!("a".fg(color), "a".fg(color_adapted));
}

#[rstest]
#[case(TermProfile::TrueColor)]
#[case(TermProfile::Ansi256)]
#[case(TermProfile::Ansi16)]
#[case(TermProfile::NoColor)]
#[case(TermProfile::NoTty)]
fn profile_style(#[case] profile: TermProfile) {
    profile.set_local();
    let style = Style::new()
        .fg(Color::Rgb(120, 67, 84))
        .add_modifier(Modifier::UNDERLINED);
    let style_adapted: ratatui::style::Style = profile.adapt_style(style.into());
    assert_eq!("a".set_style(style), "a".set_style(style_adapted));
}

#[test]
fn parse_color() {
    TermProfile::Ansi256.set_local();
    let color: Color = "chartreuse".parse().unwrap();
    let color_adapted = color.into_adaptive();
    assert!(matches!(color_adapted, Color::Indexed(_)));
}

#[test]
fn terminal_colors() {
    let palette = ColorPalette {
        terminal_fg: Color::White,
        terminal_bg: Color::Black,
        color_scheme: ColorScheme::Dark,
    };
    palette.set_local();
    assert_eq!(palette.terminal_fg, Color::terminal_fg());
    assert_eq!(palette.terminal_bg, Color::terminal_bg());
    assert_eq!(palette.color_scheme, ColorScheme::current());
}
