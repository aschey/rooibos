use ratatui::widgets::Borders;
use rooibos_theme::{
    Adaptive, Color, ColorPalette, ColorScheme, Dark, Light, SetTheme, Style, Theme, palette,
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
#[theme(prefix = "base")]
struct AppColorTheme3 {
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
fn set_color() {
    let theme = AppColorTheme {
        primary: Color::White,
        secondary: Color::Black,
    };
    theme.set_local();
    assert_eq!(theme, AppColorTheme::current());
    assert_eq!(theme.primary, AppColorTheme::primary());
    assert_eq!(theme.secondary, AppColorTheme::secondary());
    assert_eq!(theme.primary, Color::primary());
    assert_eq!(theme.secondary, Color::secondary());
}

#[test]
fn set_style() {
    let theme = AppStyleTheme {
        primary: Style::default().fg(palette::RosePine::ROSE_500),
        secondary: Style::default().fg(palette::RosePine::ROSE_100),
    };
    theme.set_local();
    assert_eq!(theme, AppStyleTheme::current());
    assert_eq!(theme.primary, AppStyleTheme::primary());
    assert_eq!(theme.secondary, AppStyleTheme::secondary());
    assert_eq!(theme.primary, Style::primary());
    assert_eq!(theme.secondary, Style::secondary());
}

#[test]
fn set_custom() {
    let theme = BorderTheme {
        primary: Borders::TOP,
        secondary: Borders::BOTTOM,
    };
    theme.set_local();
    assert_eq!(theme, BorderTheme::current());
    assert_eq!(theme.primary, BorderTheme::primary());
    assert_eq!(theme.secondary, BorderTheme::secondary());
    assert_eq!(theme.primary, Borders::primary());
    assert_eq!(theme.secondary, Borders::secondary());
}

#[test]
fn set_nested() {
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
    theme.set_local();
    assert_eq!(theme, AppTheme::current());
    assert_eq!(theme.color, AppColorTheme::current());
    assert_eq!(theme.color2, AppColorTheme2::current());
    assert_eq!(theme.borders, BorderTheme::current());
    assert_eq!(theme.style, AppStyleTheme::current());
    assert_eq!(theme.color.primary, Color::primary());
    assert_eq!(theme.color.secondary, Color::secondary());
    assert_eq!(theme.color2.primary2, Color::primary2());
    assert_eq!(theme.color2.secondary2, Color::secondary2());
    assert_eq!(theme.style.primary, Style::primary());
    assert_eq!(theme.style.secondary, Style::secondary());
    assert_eq!(theme.borders.primary, Borders::primary());
    assert_eq!(theme.borders.secondary, Borders::secondary());
}

#[test]
fn adaptive() {
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
    .set_local();

    Adaptive::new(Dark(borders1.clone()), Light(borders2)).set_local();
    assert_eq!(borders1, BorderTheme::current());
}

#[test]
fn prefix() {
    let theme = AppColorTheme3 {
        primary2: Color::Black,
        secondary2: Color::White,
    };
    theme.set_local();
    assert_eq!(theme.primary2, AppColorTheme3::base_primary2());
    assert_eq!(theme.secondary2, AppColorTheme3::base_secondary2());
}
