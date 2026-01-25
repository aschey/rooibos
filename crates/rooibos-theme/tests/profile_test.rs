use reactive_graph::owner::Owner;
use rooibos_theme::palette::Catppuccin;
use rooibos_theme::{
    Adaptive, Color, ColorPalette, ColorScheme, Dark, Light, ProfileVariant, SetTheme, TermProfile,
};

#[test]
fn adaptive() {
    let owner = Owner::new();
    owner.set();
    ColorPalette {
        terminal_fg: Color::White,
        terminal_bg: Color::Black,
        color_scheme: ColorScheme::Dark,
    }
    .set();

    let adaptive = Adaptive(Light("light"), Dark("dark"));
    assert_eq!(&"dark", adaptive.adapt());

    let color = Catppuccin::GRAY[(Dark(0), Light(10))];
    assert_eq!(Catppuccin::GRAY_50, color);

    ColorPalette {
        terminal_fg: Color::Black,
        terminal_bg: Color::White,
        color_scheme: ColorScheme::Light,
    }
    .set();

    assert_eq!(&"light", adaptive.adapt());
    let color = Catppuccin::GRAY[(Dark(0), Light(10))];
    assert_eq!(Catppuccin::GRAY_950, color);
}

#[test]
fn profile_variant() {
    let owner = Owner::new();
    owner.set();
    let variant = ProfileVariant::new("default").ansi_256("256").ansi_16("16");
    TermProfile::TrueColor.set();
    assert_eq!("default", variant.adapt());

    TermProfile::Ansi256.set();
    assert_eq!("256", variant.adapt());

    TermProfile::Ansi16.set();
    assert_eq!("16", variant.adapt());
}
