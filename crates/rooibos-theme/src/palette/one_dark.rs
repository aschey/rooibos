use std::borrow::Cow;

use crate::{Color, NamedColor, ThemeArray};

// Auto-generated file. Do not edit.

pub struct OneDark {}

impl OneDark {
    pub const NAME: &str = "one-dark";

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_50: Color = Color::Rgb(243, 243, 246);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_100: Color = Color::Rgb(227, 229, 234);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_200: Color = Color::Rgb(202, 207, 217);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_300: Color = Color::Rgb(175, 182, 197);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_400: Color = Color::Rgb(148, 158, 178);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_500: Color = Color::Rgb(127, 137, 158);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_600: Color = Color::Rgb(105, 114, 132);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_700: Color = Color::Rgb(87, 95, 110);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_800: Color = Color::Rgb(67, 73, 85);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_900: Color = Color::Rgb(48, 53, 62);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_950: Color = Color::Rgb(40, 44, 52);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_50: Color = Color::Rgb(250, 238, 239);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_100: Color = Color::Rgb(245, 221, 222);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_200: Color = Color::Rgb(237, 186, 189);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_300: Color = Color::Rgb(230, 149, 155);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_400: Color = Color::Rgb(224, 108, 117);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_500: Color = Color::Rgb(206, 72, 85);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_600: Color = Color::Rgb(165, 56, 66);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_700: Color = Color::Rgb(126, 41, 49);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_800: Color = Color::Rgb(89, 26, 32);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_900: Color = Color::Rgb(55, 13, 17);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_950: Color = Color::Rgb(37, 6, 9);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_50: Color = Color::Rgb(229, 250, 216);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_100: Color = Color::Rgb(191, 245, 153);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_200: Color = Color::Rgb(172, 221, 138);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_300: Color = Color::Rgb(152, 195, 121);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_400: Color = Color::Rgb(129, 166, 102);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_500: Color = Color::Rgb(105, 135, 83);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_600: Color = Color::Rgb(82, 106, 64);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_700: Color = Color::Rgb(61, 81, 47);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_800: Color = Color::Rgb(40, 55, 30);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_900: Color = Color::Rgb(23, 32, 16);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_950: Color = Color::Rgb(12, 19, 7);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_50: Color = Color::Rgb(252, 246, 237);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_100: Color = Color::Rgb(248, 233, 212);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_200: Color = Color::Rgb(241, 211, 161);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_300: Color = Color::Rgb(229, 192, 123);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_400: Color = Color::Rgb(193, 161, 103);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_500: Color = Color::Rgb(160, 133, 84);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_600: Color = Color::Rgb(125, 104, 64);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_700: Color = Color::Rgb(94, 78, 47);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_800: Color = Color::Rgb(65, 53, 31);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_900: Color = Color::Rgb(36, 28, 14);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_950: Color = Color::Rgb(24, 18, 8);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_50: Color = Color::Rgb(251, 242, 236);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_100: Color = Color::Rgb(247, 229, 217);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_200: Color = Color::Rgb(239, 203, 175);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_300: Color = Color::Rgb(234, 176, 123);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_400: Color = Color::Rgb(209, 154, 102);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_500: Color = Color::Rgb(171, 125, 82);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_600: Color = Color::Rgb(136, 99, 64);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_700: Color = Color::Rgb(103, 74, 47);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_800: Color = Color::Rgb(72, 51, 31);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_900: Color = Color::Rgb(40, 27, 14);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_950: Color = Color::Rgb(27, 17, 8);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_50: Color = Color::Rgb(238, 244, 253);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_100: Color = Color::Rgb(220, 233, 251);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_200: Color = Color::Rgb(188, 215, 247);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_300: Color = Color::Rgb(147, 195, 243);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_400: Color = Color::Rgb(97, 175, 239);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_500: Color = Color::Rgb(69, 145, 204);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_600: Color = Color::Rgb(53, 114, 160);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_700: Color = Color::Rgb(38, 86, 123);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_800: Color = Color::Rgb(23, 58, 84);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_900: Color = Color::Rgb(11, 33, 51);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_950: Color = Color::Rgb(5, 21, 33);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_50: Color = Color::Rgb(247, 238, 250);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_100: Color = Color::Rgb(240, 224, 247);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_200: Color = Color::Rgb(225, 189, 238);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_300: Color = Color::Rgb(212, 157, 230);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_400: Color = Color::Rgb(198, 120, 221);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_500: Color = Color::Rgb(179, 83, 204);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_600: Color = Color::Rgb(141, 64, 161);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_700: Color = Color::Rgb(107, 47, 123);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_800: Color = Color::Rgb(76, 31, 88);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_900: Color = Color::Rgb(44, 15, 51);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_950: Color = Color::Rgb(29, 8, 35);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_50: Color = Color::Rgb(224, 248, 252);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_100: Color = Color::Rgb(188, 241, 250);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_200: Color = Color::Rgb(110, 226, 241);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_300: Color = Color::Rgb(98, 205, 219);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_400: Color = Color::Rgb(85, 182, 194);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_500: Color = Color::Rgb(70, 148, 158);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_600: Color = Color::Rgb(53, 118, 126);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_700: Color = Color::Rgb(39, 87, 93);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_800: Color = Color::Rgb(24, 60, 65);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_900: Color = Color::Rgb(10, 33, 36);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_950: Color = Color::Rgb(5, 22, 24);

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

    pub const RED: ThemeArray<11> = ThemeArray([
        Self::RED_50,
        Self::RED_100,
        Self::RED_200,
        Self::RED_300,
        Self::RED_400,
        Self::RED_500,
        Self::RED_600,
        Self::RED_700,
        Self::RED_800,
        Self::RED_900,
        Self::RED_950,
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

    pub const MAGENTA: ThemeArray<11> = ThemeArray([
        Self::MAGENTA_50,
        Self::MAGENTA_100,
        Self::MAGENTA_200,
        Self::MAGENTA_300,
        Self::MAGENTA_400,
        Self::MAGENTA_500,
        Self::MAGENTA_600,
        Self::MAGENTA_700,
        Self::MAGENTA_800,
        Self::MAGENTA_900,
        Self::MAGENTA_950,
    ]);

    pub const CYAN: ThemeArray<11> = ThemeArray([
        Self::CYAN_50,
        Self::CYAN_100,
        Self::CYAN_200,
        Self::CYAN_300,
        Self::CYAN_400,
        Self::CYAN_500,
        Self::CYAN_600,
        Self::CYAN_700,
        Self::CYAN_800,
        Self::CYAN_900,
        Self::CYAN_950,
    ]);

    pub const ALL_COLORS: [NamedColor<'_>; 88] = [
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
            group: Cow::Borrowed("red"),
            color: Self::RED_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("red"),
            color: Self::RED_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("red"),
            color: Self::RED_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("red"),
            color: Self::RED_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("red"),
            color: Self::RED_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("red"),
            color: Self::RED_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("red"),
            color: Self::RED_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("red"),
            color: Self::RED_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("red"),
            color: Self::RED_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("red"),
            color: Self::RED_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("red"),
            color: Self::RED_950,
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
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("magenta"),
            color: Self::MAGENTA_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("magenta"),
            color: Self::MAGENTA_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("magenta"),
            color: Self::MAGENTA_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("magenta"),
            color: Self::MAGENTA_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("magenta"),
            color: Self::MAGENTA_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("magenta"),
            color: Self::MAGENTA_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("magenta"),
            color: Self::MAGENTA_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("magenta"),
            color: Self::MAGENTA_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("magenta"),
            color: Self::MAGENTA_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("magenta"),
            color: Self::MAGENTA_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("magenta"),
            color: Self::MAGENTA_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("cyan"),
            color: Self::CYAN_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("cyan"),
            color: Self::CYAN_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("cyan"),
            color: Self::CYAN_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("cyan"),
            color: Self::CYAN_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("cyan"),
            color: Self::CYAN_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("cyan"),
            color: Self::CYAN_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("cyan"),
            color: Self::CYAN_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("cyan"),
            color: Self::CYAN_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("cyan"),
            color: Self::CYAN_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("cyan"),
            color: Self::CYAN_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("cyan"),
            color: Self::CYAN_950,
        },
    ];
}
