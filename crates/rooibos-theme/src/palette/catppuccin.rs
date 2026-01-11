use std::borrow::Cow;

use crate::{Color, NamedColor, ThemeArray};

// Auto-generated file. Do not edit.

pub struct Catppuccin {}

impl Catppuccin {
    pub const NAME: &str = "catppuccin";

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_50: Color = Color::Rgb(243, 243, 246);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_100: Color = Color::Rgb(231, 232, 237);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_200: Color = Color::Rgb(208, 209, 219);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_300: Color = Color::Rgb(182, 184, 200);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_400: Color = Color::Rgb(160, 162, 183);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_500: Color = Color::Rgb(138, 140, 166);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_600: Color = Color::Rgb(116, 120, 149);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_700: Color = Color::Rgb(96, 100, 131);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_800: Color = Color::Rgb(76, 79, 105);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_900: Color = Color::Rgb(39, 41, 56);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_950: Color = Color::Rgb(24, 25, 36);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSEWATER_50: Color = Color::Rgb(249, 238, 236);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSEWATER_100: Color = Color::Rgb(245, 224, 220);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSEWATER_200: Color = Color::Rgb(236, 191, 181);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSEWATER_300: Color = Color::Rgb(229, 156, 138);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSEWATER_400: Color = Color::Rgb(212, 120, 90);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSEWATER_500: Color = Color::Rgb(176, 98, 73);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSEWATER_600: Color = Color::Rgb(140, 77, 57);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSEWATER_700: Color = Color::Rgb(107, 57, 42);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSEWATER_800: Color = Color::Rgb(75, 39, 27);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSEWATER_900: Color = Color::Rgb(45, 21, 13);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSEWATER_950: Color = Color::Rgb(30, 12, 7);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const FLAMINGO_50: Color = Color::Rgb(251, 241, 241);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const FLAMINGO_100: Color = Color::Rgb(248, 231, 231);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const FLAMINGO_200: Color = Color::Rgb(242, 205, 205);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const FLAMINGO_300: Color = Color::Rgb(234, 168, 168);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const FLAMINGO_400: Color = Color::Rgb(228, 125, 125);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const FLAMINGO_500: Color = Color::Rgb(224, 73, 73);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const FLAMINGO_600: Color = Color::Rgb(181, 58, 58);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const FLAMINGO_700: Color = Color::Rgb(137, 41, 41);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const FLAMINGO_800: Color = Color::Rgb(95, 26, 26);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const FLAMINGO_900: Color = Color::Rgb(60, 13, 13);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const FLAMINGO_950: Color = Color::Rgb(38, 6, 6);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_50: Color = Color::Rgb(252, 240, 249);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_100: Color = Color::Rgb(250, 225, 243);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_200: Color = Color::Rgb(245, 194, 231);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_300: Color = Color::Rgb(239, 150, 217);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_400: Color = Color::Rgb(233, 104, 205);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_500: Color = Color::Rgb(205, 67, 177);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_600: Color = Color::Rgb(165, 52, 143);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_700: Color = Color::Rgb(124, 37, 106);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_800: Color = Color::Rgb(88, 24, 75);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_900: Color = Color::Rgb(52, 10, 43);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_950: Color = Color::Rgb(35, 5, 29);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAUVE_50: Color = Color::Rgb(246, 241, 254);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAUVE_100: Color = Color::Rgb(236, 225, 252);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAUVE_200: Color = Color::Rgb(220, 197, 250);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAUVE_300: Color = Color::Rgb(203, 166, 247);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAUVE_400: Color = Color::Rgb(184, 126, 243);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAUVE_500: Color = Color::Rgb(169, 84, 239);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAUVE_600: Color = Color::Rgb(144, 43, 214);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAUVE_700: Color = Color::Rgb(111, 31, 166);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAUVE_800: Color = Color::Rgb(76, 18, 116);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAUVE_900: Color = Color::Rgb(47, 8, 74);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAUVE_950: Color = Color::Rgb(29, 3, 48);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAROON_50: Color = Color::Rgb(251, 241, 242);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAROON_100: Color = Color::Rgb(247, 224, 227);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAROON_200: Color = Color::Rgb(241, 192, 199);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAROON_300: Color = Color::Rgb(235, 160, 172);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAROON_400: Color = Color::Rgb(229, 118, 139);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAROON_500: Color = Color::Rgb(216, 71, 105);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAROON_600: Color = Color::Rgb(175, 55, 84);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAROON_700: Color = Color::Rgb(131, 39, 61);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAROON_800: Color = Color::Rgb(94, 26, 42);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAROON_900: Color = Color::Rgb(55, 11, 22);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAROON_950: Color = Color::Rgb(38, 6, 13);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_50: Color = Color::Rgb(254, 241, 235);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_100: Color = Color::Rgb(253, 231, 220);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_200: Color = Color::Rgb(251, 206, 182);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_300: Color = Color::Rgb(250, 179, 135);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_400: Color = Color::Rgb(234, 140, 51);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_500: Color = Color::Rgb(194, 115, 41);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_600: Color = Color::Rgb(155, 91, 30);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_700: Color = Color::Rgb(118, 68, 21);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_800: Color = Color::Rgb(79, 44, 11);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_900: Color = Color::Rgb(47, 24, 4);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_950: Color = Color::Rgb(33, 15, 2);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_50: Color = Color::Rgb(252, 240, 216);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_100: Color = Color::Rgb(249, 226, 175);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_200: Color = Color::Rgb(232, 198, 102);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_300: Color = Color::Rgb(198, 168, 85);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_400: Color = Color::Rgb(167, 142, 71);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_500: Color = Color::Rgb(138, 117, 57);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_600: Color = Color::Rgb(110, 93, 44);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_700: Color = Color::Rgb(83, 69, 31);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_800: Color = Color::Rgb(57, 47, 19);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_900: Color = Color::Rgb(33, 27, 8);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_950: Color = Color::Rgb(21, 16, 4);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_50: Color = Color::Rgb(231, 248, 229);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_100: Color = Color::Rgb(203, 242, 200);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_200: Color = Color::Rgb(166, 227, 161);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_300: Color = Color::Rgb(142, 195, 138);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_400: Color = Color::Rgb(121, 167, 117);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_500: Color = Color::Rgb(98, 136, 95);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_600: Color = Color::Rgb(76, 107, 74);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_700: Color = Color::Rgb(57, 81, 55);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_800: Color = Color::Rgb(37, 55, 36);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_900: Color = Color::Rgb(21, 32, 20);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_950: Color = Color::Rgb(11, 19, 10);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_50: Color = Color::Rgb(220, 250, 244);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_100: Color = Color::Rgb(179, 245, 233);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_200: Color = Color::Rgb(148, 226, 213);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_300: Color = Color::Rgb(127, 194, 183);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_400: Color = Color::Rgb(107, 166, 156);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_500: Color = Color::Rgb(87, 136, 127);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_600: Color = Color::Rgb(67, 106, 100);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_700: Color = Color::Rgb(50, 81, 76);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_800: Color = Color::Rgb(32, 55, 51);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_900: Color = Color::Rgb(17, 32, 30);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_950: Color = Color::Rgb(8, 19, 17);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SAPPHIRE_50: Color = Color::Rgb(235, 245, 252);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SAPPHIRE_100: Color = Color::Rgb(214, 235, 248);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SAPPHIRE_200: Color = Color::Rgb(169, 216, 242);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SAPPHIRE_300: Color = Color::Rgb(116, 199, 236);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SAPPHIRE_400: Color = Color::Rgb(95, 170, 202);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SAPPHIRE_500: Color = Color::Rgb(77, 139, 167);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SAPPHIRE_600: Color = Color::Rgb(60, 110, 132);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SAPPHIRE_700: Color = Color::Rgb(43, 82, 100);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SAPPHIRE_800: Color = Color::Rgb(28, 56, 69);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SAPPHIRE_900: Color = Color::Rgb(13, 32, 40);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SAPPHIRE_950: Color = Color::Rgb(6, 18, 24);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_50: Color = Color::Rgb(235, 241, 254);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_100: Color = Color::Rgb(219, 230, 253);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_200: Color = Color::Rgb(181, 205, 252);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_300: Color = Color::Rgb(137, 180, 250);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_400: Color = Color::Rgb(67, 152, 248);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_500: Color = Color::Rgb(37, 126, 214);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_600: Color = Color::Rgb(28, 101, 172);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_700: Color = Color::Rgb(18, 74, 129);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_800: Color = Color::Rgb(10, 51, 92);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_900: Color = Color::Rgb(3, 28, 54);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_950: Color = Color::Rgb(2, 17, 37);

    pub const GRAY: ThemeArray<11> = ThemeArray([
        Self::GRAY_50,
        Self::GRAY_100,
        Self::GRAY_200,
        Self::GRAY_300,
        Self::GRAY_400,
        Self::GRAY_500,
        Self::GRAY_600,
        Self::GRAY_700,
        Self::GRAY_800,
        Self::GRAY_900,
        Self::GRAY_950,
    ]);

    pub const ROSEWATER: ThemeArray<11> = ThemeArray([
        Self::ROSEWATER_50,
        Self::ROSEWATER_100,
        Self::ROSEWATER_200,
        Self::ROSEWATER_300,
        Self::ROSEWATER_400,
        Self::ROSEWATER_500,
        Self::ROSEWATER_600,
        Self::ROSEWATER_700,
        Self::ROSEWATER_800,
        Self::ROSEWATER_900,
        Self::ROSEWATER_950,
    ]);

    pub const FLAMINGO: ThemeArray<11> = ThemeArray([
        Self::FLAMINGO_50,
        Self::FLAMINGO_100,
        Self::FLAMINGO_200,
        Self::FLAMINGO_300,
        Self::FLAMINGO_400,
        Self::FLAMINGO_500,
        Self::FLAMINGO_600,
        Self::FLAMINGO_700,
        Self::FLAMINGO_800,
        Self::FLAMINGO_900,
        Self::FLAMINGO_950,
    ]);

    pub const PINK: ThemeArray<11> = ThemeArray([
        Self::PINK_50,
        Self::PINK_100,
        Self::PINK_200,
        Self::PINK_300,
        Self::PINK_400,
        Self::PINK_500,
        Self::PINK_600,
        Self::PINK_700,
        Self::PINK_800,
        Self::PINK_900,
        Self::PINK_950,
    ]);

    pub const MAUVE: ThemeArray<11> = ThemeArray([
        Self::MAUVE_50,
        Self::MAUVE_100,
        Self::MAUVE_200,
        Self::MAUVE_300,
        Self::MAUVE_400,
        Self::MAUVE_500,
        Self::MAUVE_600,
        Self::MAUVE_700,
        Self::MAUVE_800,
        Self::MAUVE_900,
        Self::MAUVE_950,
    ]);

    pub const MAROON: ThemeArray<11> = ThemeArray([
        Self::MAROON_50,
        Self::MAROON_100,
        Self::MAROON_200,
        Self::MAROON_300,
        Self::MAROON_400,
        Self::MAROON_500,
        Self::MAROON_600,
        Self::MAROON_700,
        Self::MAROON_800,
        Self::MAROON_900,
        Self::MAROON_950,
    ]);

    pub const PEACH: ThemeArray<11> = ThemeArray([
        Self::PEACH_50,
        Self::PEACH_100,
        Self::PEACH_200,
        Self::PEACH_300,
        Self::PEACH_400,
        Self::PEACH_500,
        Self::PEACH_600,
        Self::PEACH_700,
        Self::PEACH_800,
        Self::PEACH_900,
        Self::PEACH_950,
    ]);

    pub const YELLOW: ThemeArray<11> = ThemeArray([
        Self::YELLOW_50,
        Self::YELLOW_100,
        Self::YELLOW_200,
        Self::YELLOW_300,
        Self::YELLOW_400,
        Self::YELLOW_500,
        Self::YELLOW_600,
        Self::YELLOW_700,
        Self::YELLOW_800,
        Self::YELLOW_900,
        Self::YELLOW_950,
    ]);

    pub const GREEN: ThemeArray<11> = ThemeArray([
        Self::GREEN_50,
        Self::GREEN_100,
        Self::GREEN_200,
        Self::GREEN_300,
        Self::GREEN_400,
        Self::GREEN_500,
        Self::GREEN_600,
        Self::GREEN_700,
        Self::GREEN_800,
        Self::GREEN_900,
        Self::GREEN_950,
    ]);

    pub const TEAL: ThemeArray<11> = ThemeArray([
        Self::TEAL_50,
        Self::TEAL_100,
        Self::TEAL_200,
        Self::TEAL_300,
        Self::TEAL_400,
        Self::TEAL_500,
        Self::TEAL_600,
        Self::TEAL_700,
        Self::TEAL_800,
        Self::TEAL_900,
        Self::TEAL_950,
    ]);

    pub const SAPPHIRE: ThemeArray<11> = ThemeArray([
        Self::SAPPHIRE_50,
        Self::SAPPHIRE_100,
        Self::SAPPHIRE_200,
        Self::SAPPHIRE_300,
        Self::SAPPHIRE_400,
        Self::SAPPHIRE_500,
        Self::SAPPHIRE_600,
        Self::SAPPHIRE_700,
        Self::SAPPHIRE_800,
        Self::SAPPHIRE_900,
        Self::SAPPHIRE_950,
    ]);

    pub const BLUE: ThemeArray<11> = ThemeArray([
        Self::BLUE_50,
        Self::BLUE_100,
        Self::BLUE_200,
        Self::BLUE_300,
        Self::BLUE_400,
        Self::BLUE_500,
        Self::BLUE_600,
        Self::BLUE_700,
        Self::BLUE_800,
        Self::BLUE_900,
        Self::BLUE_950,
    ]);

    pub const ALL_COLORS: [NamedColor<'_>; 132] = [
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("gray"),
            color: Self::GRAY_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("gray"),
            color: Self::GRAY_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("gray"),
            color: Self::GRAY_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("gray"),
            color: Self::GRAY_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("gray"),
            color: Self::GRAY_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("gray"),
            color: Self::GRAY_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("gray"),
            color: Self::GRAY_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("gray"),
            color: Self::GRAY_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("gray"),
            color: Self::GRAY_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("gray"),
            color: Self::GRAY_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("gray"),
            color: Self::GRAY_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("rosewater"),
            color: Self::ROSEWATER_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("rosewater"),
            color: Self::ROSEWATER_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("rosewater"),
            color: Self::ROSEWATER_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("rosewater"),
            color: Self::ROSEWATER_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("rosewater"),
            color: Self::ROSEWATER_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("rosewater"),
            color: Self::ROSEWATER_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("rosewater"),
            color: Self::ROSEWATER_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("rosewater"),
            color: Self::ROSEWATER_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("rosewater"),
            color: Self::ROSEWATER_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("rosewater"),
            color: Self::ROSEWATER_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("rosewater"),
            color: Self::ROSEWATER_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("flamingo"),
            color: Self::FLAMINGO_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("flamingo"),
            color: Self::FLAMINGO_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("flamingo"),
            color: Self::FLAMINGO_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("flamingo"),
            color: Self::FLAMINGO_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("flamingo"),
            color: Self::FLAMINGO_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("flamingo"),
            color: Self::FLAMINGO_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("flamingo"),
            color: Self::FLAMINGO_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("flamingo"),
            color: Self::FLAMINGO_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("flamingo"),
            color: Self::FLAMINGO_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("flamingo"),
            color: Self::FLAMINGO_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("flamingo"),
            color: Self::FLAMINGO_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("pink"),
            color: Self::PINK_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("pink"),
            color: Self::PINK_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("pink"),
            color: Self::PINK_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("pink"),
            color: Self::PINK_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("pink"),
            color: Self::PINK_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("pink"),
            color: Self::PINK_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("pink"),
            color: Self::PINK_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("pink"),
            color: Self::PINK_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("pink"),
            color: Self::PINK_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("pink"),
            color: Self::PINK_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("pink"),
            color: Self::PINK_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("mauve"),
            color: Self::MAUVE_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("mauve"),
            color: Self::MAUVE_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("mauve"),
            color: Self::MAUVE_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("mauve"),
            color: Self::MAUVE_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("mauve"),
            color: Self::MAUVE_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("mauve"),
            color: Self::MAUVE_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("mauve"),
            color: Self::MAUVE_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("mauve"),
            color: Self::MAUVE_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("mauve"),
            color: Self::MAUVE_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("mauve"),
            color: Self::MAUVE_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("mauve"),
            color: Self::MAUVE_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("maroon"),
            color: Self::MAROON_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("maroon"),
            color: Self::MAROON_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("maroon"),
            color: Self::MAROON_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("maroon"),
            color: Self::MAROON_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("maroon"),
            color: Self::MAROON_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("maroon"),
            color: Self::MAROON_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("maroon"),
            color: Self::MAROON_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("maroon"),
            color: Self::MAROON_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("maroon"),
            color: Self::MAROON_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("maroon"),
            color: Self::MAROON_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("maroon"),
            color: Self::MAROON_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("peach"),
            color: Self::PEACH_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("peach"),
            color: Self::PEACH_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("peach"),
            color: Self::PEACH_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("peach"),
            color: Self::PEACH_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("peach"),
            color: Self::PEACH_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("peach"),
            color: Self::PEACH_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("peach"),
            color: Self::PEACH_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("peach"),
            color: Self::PEACH_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("peach"),
            color: Self::PEACH_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("peach"),
            color: Self::PEACH_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("peach"),
            color: Self::PEACH_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("yellow"),
            color: Self::YELLOW_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("yellow"),
            color: Self::YELLOW_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("yellow"),
            color: Self::YELLOW_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("yellow"),
            color: Self::YELLOW_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("yellow"),
            color: Self::YELLOW_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("yellow"),
            color: Self::YELLOW_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("yellow"),
            color: Self::YELLOW_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("yellow"),
            color: Self::YELLOW_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("yellow"),
            color: Self::YELLOW_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("yellow"),
            color: Self::YELLOW_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("yellow"),
            color: Self::YELLOW_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("green"),
            color: Self::GREEN_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("green"),
            color: Self::GREEN_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("green"),
            color: Self::GREEN_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("green"),
            color: Self::GREEN_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("green"),
            color: Self::GREEN_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("green"),
            color: Self::GREEN_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("green"),
            color: Self::GREEN_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("green"),
            color: Self::GREEN_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("green"),
            color: Self::GREEN_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("green"),
            color: Self::GREEN_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("green"),
            color: Self::GREEN_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("teal"),
            color: Self::TEAL_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("teal"),
            color: Self::TEAL_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("teal"),
            color: Self::TEAL_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("teal"),
            color: Self::TEAL_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("teal"),
            color: Self::TEAL_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("teal"),
            color: Self::TEAL_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("teal"),
            color: Self::TEAL_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("teal"),
            color: Self::TEAL_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("teal"),
            color: Self::TEAL_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("teal"),
            color: Self::TEAL_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("teal"),
            color: Self::TEAL_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("sapphire"),
            color: Self::SAPPHIRE_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("sapphire"),
            color: Self::SAPPHIRE_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("sapphire"),
            color: Self::SAPPHIRE_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("sapphire"),
            color: Self::SAPPHIRE_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("sapphire"),
            color: Self::SAPPHIRE_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("sapphire"),
            color: Self::SAPPHIRE_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("sapphire"),
            color: Self::SAPPHIRE_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("sapphire"),
            color: Self::SAPPHIRE_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("sapphire"),
            color: Self::SAPPHIRE_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("sapphire"),
            color: Self::SAPPHIRE_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("sapphire"),
            color: Self::SAPPHIRE_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("blue"),
            color: Self::BLUE_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("blue"),
            color: Self::BLUE_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("blue"),
            color: Self::BLUE_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("blue"),
            color: Self::BLUE_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("blue"),
            color: Self::BLUE_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("blue"),
            color: Self::BLUE_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("blue"),
            color: Self::BLUE_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("blue"),
            color: Self::BLUE_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("blue"),
            color: Self::BLUE_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("blue"),
            color: Self::BLUE_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("blue"),
            color: Self::BLUE_950,
        },
    ];
}
