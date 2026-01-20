use ratatui::widgets::Borders;
use reactive_graph::owner::{Owner, provide_context};
use reactive_graph::traits::Get;
use rooibos_theme::{
    Adaptive, Color, ColorPalette, ColorScheme, Dark, Light, SetTheme, Style, Theme, ThemeContext,
    palette,
};

#[derive(Theme, Default, Clone, Debug, PartialEq, Eq)]
struct AppColorTheme {
    primary: Color,
    secondary: Color,
}

#[derive(Theme, Default, Clone, Debug, PartialEq, Eq)]
struct AppStyleTheme {
    primary: Style,
    secondary: Style,
}

#[derive(Theme, Default, Clone, Debug, PartialEq, Eq)]
struct AppColorTheme2 {
    primary2: Color,
    secondary2: Color,
}

#[derive(Theme, Default, Clone, Debug, PartialEq, Eq)]
struct BorderTheme {
    primary: Borders,
    secondary: Borders,
}

#[derive(Theme, Default, Clone, Debug, PartialEq, Eq)]
struct AppTheme {
    #[subtheme]
    color: AppColorTheme,
    #[subtheme]
    borders: BorderTheme,
    #[subtheme]
    color2: AppColorTheme2,
    #[subtheme]
    style: AppStyleTheme,
}

#[test]
fn set_color_theme() {
    let owner = Owner::new();
    owner.set();
    provide_context(ThemeContext::default());

    let theme = AppColorTheme {
        primary: Color::White,
        secondary: Color::Black,
    };
    theme.set();

    assert_eq!(theme, AppColorTheme::current());
    assert_eq!(theme.primary, AppColorTheme::primary().get());
    assert_eq!(theme.secondary, AppColorTheme::secondary().get());
    assert_eq!(theme.primary, Color::primary().get());
    assert_eq!(theme.secondary, Color::secondary().get());
}

#[test]
fn set_style() {
    let owner = Owner::new();
    owner.set();
    provide_context(ThemeContext::default());

    let theme = AppStyleTheme {
        primary: Style::default().fg(palette::RosePine::ROSE_500),
        secondary: Style::default().fg(palette::RosePine::ROSE_100),
    };
    theme.set();
    assert_eq!(theme, AppStyleTheme::current());
    assert_eq!(theme.primary, AppStyleTheme::primary().get());
    assert_eq!(theme.secondary, AppStyleTheme::secondary().get());
    assert_eq!(theme.primary, Style::primary().get());
    assert_eq!(theme.secondary, Style::secondary().get());
}

#[test]
fn set_custom() {
    let owner = Owner::new();
    owner.set();
    provide_context(ThemeContext::default());

    let theme = BorderTheme {
        primary: Borders::TOP,
        secondary: Borders::BOTTOM,
    };
    theme.set();
    assert_eq!(theme, BorderTheme::current());
    assert_eq!(theme.primary, BorderTheme::primary().get());
    assert_eq!(theme.secondary, BorderTheme::secondary().get());
    assert_eq!(theme.primary, Borders::primary().get());
    assert_eq!(theme.secondary, Borders::secondary().get());
}

#[test]
fn set_nested() {
    let owner = Owner::new();
    owner.set();
    provide_context(ThemeContext::default());

    let theme = AppTheme {
        color: AppColorTheme {
            primary: Color::Red,
            secondary: Color::Blue,
        },
        color2: AppColorTheme2 {
            primary2: Color::Green,
            secondary2: Color::Yellow,
        },
        borders: BorderTheme {
            primary: Borders::TOP,
            secondary: Borders::BOTTOM,
        },
        style: AppStyleTheme {
            primary: Style::default().fg(palette::RosePine::ROSE_500),
            secondary: Style::default().fg(palette::RosePine::ROSE_100),
        },
    };
    theme.set();
    assert_eq!(theme, AppTheme::current());
    assert_eq!(theme.color, AppColorTheme::current());
    assert_eq!(theme.color2, AppColorTheme2::current());
    assert_eq!(theme.borders, BorderTheme::current());
    assert_eq!(theme.style, AppStyleTheme::current());
    assert_eq!(theme.color.primary, Color::primary().get());
    assert_eq!(theme.color.secondary, Color::secondary().get());
    assert_eq!(theme.color2.primary2, Color::primary2().get());
    assert_eq!(theme.color2.secondary2, Color::secondary2().get());
    assert_eq!(theme.style.primary, Style::primary().get());
    assert_eq!(theme.style.secondary, Style::secondary().get());
    assert_eq!(theme.borders.primary, Borders::primary().get());
    assert_eq!(theme.borders.secondary, Borders::secondary().get());
}

#[test]
fn adaptive() {
    let owner = Owner::new();
    owner.set();

    let borders1 = BorderTheme {
        primary: Borders::TOP,
        secondary: Borders::BOTTOM,
    };

    let borders2 = BorderTheme {
        primary: Borders::TOP,
        secondary: Borders::BOTTOM,
    };

    ColorPalette {
        terminal_fg: Color::White,
        terminal_bg: Color::Black,
        color_scheme: ColorScheme::Dark,
    }
    .set();

    Adaptive::new(Dark(borders1.clone()), Light(borders2)).set();
    assert_eq!(borders1, BorderTheme::current());
}
