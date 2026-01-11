use std::borrow::Cow;

use crate::{Color, NamedColor, ThemeArray};

// Auto-generated file. Do not edit.

pub struct Solarized {}

impl Solarized {
    pub const NAME: &str = "solarized";

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BASE_BLUE_50: Color = Color::Rgb(229, 246, 255);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BASE_BLUE_100: Color = Color::Rgb(193, 236, 255);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BASE_BLUE_200: Color = Color::Rgb(103, 218, 255);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BASE_BLUE_300: Color = Color::Rgb(0, 195, 236);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BASE_BLUE_400: Color = Color::Rgb(3, 169, 204);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BASE_BLUE_500: Color = Color::Rgb(0, 147, 177);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BASE_BLUE_600: Color = Color::Rgb(1, 122, 148);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BASE_BLUE_700: Color = Color::Rgb(0, 98, 120);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BASE_BLUE_800: Color = Color::Rgb(1, 75, 92);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BASE_BLUE_900: Color = Color::Rgb(1, 53, 66);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BASE_BLUE_950: Color = Color::Rgb(0, 43, 54);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_50: Color = Color::Rgb(233, 242, 246);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_100: Color = Color::Rgb(215, 232, 238);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_200: Color = Color::Rgb(172, 210, 222);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_300: Color = Color::Rgb(147, 182, 193);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_400: Color = Color::Rgb(127, 157, 167);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_500: Color = Color::Rgb(107, 133, 142);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_600: Color = Color::Rgb(88, 110, 117);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_700: Color = Color::Rgb(66, 83, 88);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_800: Color = Color::Rgb(42, 55, 59);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_900: Color = Color::Rgb(23, 31, 33);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_950: Color = Color::Rgb(14, 20, 22);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_50: Color = Color::Rgb(246, 243, 235);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_100: Color = Color::Rgb(238, 232, 213);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_200: Color = Color::Rgb(212, 204, 178);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_300: Color = Color::Rgb(183, 176, 154);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_400: Color = Color::Rgb(153, 147, 128);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_500: Color = Color::Rgb(126, 121, 106);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_600: Color = Color::Rgb(101, 97, 84);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_700: Color = Color::Rgb(76, 73, 63);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_800: Color = Color::Rgb(51, 48, 41);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_900: Color = Color::Rgb(29, 27, 23);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_950: Color = Color::Rgb(18, 17, 13);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_50: Color = Color::Rgb(255, 238, 217);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_100: Color = Color::Rgb(255, 222, 171);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_200: Color = Color::Rgb(249, 189, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_300: Color = Color::Rgb(215, 163, 2);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_400: Color = Color::Rgb(181, 137, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_500: Color = Color::Rgb(150, 113, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_600: Color = Color::Rgb(120, 90, 1);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_700: Color = Color::Rgb(91, 67, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_800: Color = Color::Rgb(63, 46, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_900: Color = Color::Rgb(37, 26, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_950: Color = Color::Rgb(24, 16, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_50: Color = Color::Rgb(254, 237, 235);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_100: Color = Color::Rgb(254, 219, 214);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_200: Color = Color::Rgb(253, 181, 170);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_300: Color = Color::Rgb(252, 140, 117);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_400: Color = Color::Rgb(244, 92, 29);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_500: Color = Color::Rgb(203, 75, 22);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_600: Color = Color::Rgb(161, 58, 15);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_700: Color = Color::Rgb(126, 43, 10);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_800: Color = Color::Rgb(89, 28, 4);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_900: Color = Color::Rgb(55, 14, 2);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_950: Color = Color::Rgb(37, 7, 1);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_50: Color = Color::Rgb(253, 237, 237);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_100: Color = Color::Rgb(250, 219, 219);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_200: Color = Color::Rgb(246, 182, 182);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_300: Color = Color::Rgb(243, 139, 138);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_400: Color = Color::Rgb(241, 92, 90);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_500: Color = Color::Rgb(220, 50, 47);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_600: Color = Color::Rgb(175, 38, 35);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_700: Color = Color::Rgb(133, 26, 24);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_800: Color = Color::Rgb(97, 16, 15);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_900: Color = Color::Rgb(60, 7, 6);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_950: Color = Color::Rgb(41, 3, 3);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_50: Color = Color::Rgb(252, 237, 242);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_100: Color = Color::Rgb(250, 218, 229);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_200: Color = Color::Rgb(245, 181, 203);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_300: Color = Color::Rgb(241, 140, 179);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_400: Color = Color::Rgb(238, 91, 156);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_500: Color = Color::Rgb(211, 54, 130);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_600: Color = Color::Rgb(171, 42, 104);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_700: Color = Color::Rgb(130, 30, 78);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_800: Color = Color::Rgb(92, 18, 54);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_900: Color = Color::Rgb(57, 8, 31);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MAGENTA_950: Color = Color::Rgb(39, 4, 20);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_50: Color = Color::Rgb(240, 240, 249);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_100: Color = Color::Rgb(224, 225, 243);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_200: Color = Color::Rgb(194, 196, 231);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_300: Color = Color::Rgb(164, 167, 220);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_400: Color = Color::Rgb(138, 142, 210);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_500: Color = Color::Rgb(108, 113, 196);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_600: Color = Color::Rgb(81, 87, 180);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_700: Color = Color::Rgb(56, 62, 141);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_800: Color = Color::Rgb(38, 42, 100);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_900: Color = Color::Rgb(20, 23, 62);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_950: Color = Color::Rgb(12, 14, 42);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_50: Color = Color::Rgb(237, 244, 254);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_100: Color = Color::Rgb(215, 231, 253);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_200: Color = Color::Rgb(171, 207, 252);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_300: Color = Color::Rgb(119, 184, 250);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_400: Color = Color::Rgb(46, 162, 243);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_500: Color = Color::Rgb(39, 139, 210);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_600: Color = Color::Rgb(28, 109, 166);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_700: Color = Color::Rgb(18, 81, 125);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_800: Color = Color::Rgb(9, 55, 87);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_900: Color = Color::Rgb(3, 31, 52);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_950: Color = Color::Rgb(2, 21, 37);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_50: Color = Color::Rgb(184, 254, 246);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_100: Color = Color::Rgb(71, 251, 237);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_200: Color = Color::Rgb(60, 220, 208);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_300: Color = Color::Rgb(52, 190, 179);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_400: Color = Color::Rgb(42, 161, 152);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_500: Color = Color::Rgb(33, 132, 125);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_600: Color = Color::Rgb(24, 105, 99);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_700: Color = Color::Rgb(17, 79, 74);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_800: Color = Color::Rgb(7, 55, 51);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_900: Color = Color::Rgb(3, 32, 29);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_950: Color = Color::Rgb(1, 20, 18);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_50: Color = Color::Rgb(224, 255, 52);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_100: Color = Color::Rgb(210, 240, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_200: Color = Color::Rgb(184, 211, 3);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_300: Color = Color::Rgb(158, 182, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_400: Color = Color::Rgb(133, 153, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_500: Color = Color::Rgb(110, 127, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_600: Color = Color::Rgb(87, 101, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_700: Color = Color::Rgb(65, 76, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_800: Color = Color::Rgb(44, 52, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_900: Color = Color::Rgb(25, 30, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_950: Color = Color::Rgb(15, 19, 0);

    pub const BASE_BLUE: ThemeArray<11> = ThemeArray([
        Self::BASE_BLUE_50,
        Self::BASE_BLUE_100,
        Self::BASE_BLUE_200,
        Self::BASE_BLUE_300,
        Self::BASE_BLUE_400,
        Self::BASE_BLUE_500,
        Self::BASE_BLUE_600,
        Self::BASE_BLUE_700,
        Self::BASE_BLUE_800,
        Self::BASE_BLUE_900,
        Self::BASE_BLUE_950,
    ]);

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

    pub const BROWN: ThemeArray<11> = ThemeArray([
        Self::BROWN_50,
        Self::BROWN_100,
        Self::BROWN_200,
        Self::BROWN_300,
        Self::BROWN_400,
        Self::BROWN_500,
        Self::BROWN_600,
        Self::BROWN_700,
        Self::BROWN_800,
        Self::BROWN_900,
        Self::BROWN_950,
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

    pub const ORANGE: ThemeArray<11> = ThemeArray([
        Self::ORANGE_50,
        Self::ORANGE_100,
        Self::ORANGE_200,
        Self::ORANGE_300,
        Self::ORANGE_400,
        Self::ORANGE_500,
        Self::ORANGE_600,
        Self::ORANGE_700,
        Self::ORANGE_800,
        Self::ORANGE_900,
        Self::ORANGE_950,
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

    pub const VIOLET: ThemeArray<11> = ThemeArray([
        Self::VIOLET_50,
        Self::VIOLET_100,
        Self::VIOLET_200,
        Self::VIOLET_300,
        Self::VIOLET_400,
        Self::VIOLET_500,
        Self::VIOLET_600,
        Self::VIOLET_700,
        Self::VIOLET_800,
        Self::VIOLET_900,
        Self::VIOLET_950,
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

    pub const ALL_COLORS: [NamedColor<'_>; 121] = [
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("base_blue"),
            color: Self::BASE_BLUE_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("base_blue"),
            color: Self::BASE_BLUE_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("base_blue"),
            color: Self::BASE_BLUE_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("base_blue"),
            color: Self::BASE_BLUE_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("base_blue"),
            color: Self::BASE_BLUE_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("base_blue"),
            color: Self::BASE_BLUE_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("base_blue"),
            color: Self::BASE_BLUE_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("base_blue"),
            color: Self::BASE_BLUE_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("base_blue"),
            color: Self::BASE_BLUE_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("base_blue"),
            color: Self::BASE_BLUE_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("base_blue"),
            color: Self::BASE_BLUE_950,
        },
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
            group: Cow::Borrowed("brown"),
            color: Self::BROWN_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("brown"),
            color: Self::BROWN_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("brown"),
            color: Self::BROWN_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("brown"),
            color: Self::BROWN_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("brown"),
            color: Self::BROWN_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("brown"),
            color: Self::BROWN_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("brown"),
            color: Self::BROWN_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("brown"),
            color: Self::BROWN_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("brown"),
            color: Self::BROWN_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("brown"),
            color: Self::BROWN_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("brown"),
            color: Self::BROWN_950,
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
            group: Cow::Borrowed("orange"),
            color: Self::ORANGE_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("orange"),
            color: Self::ORANGE_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("orange"),
            color: Self::ORANGE_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("orange"),
            color: Self::ORANGE_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("orange"),
            color: Self::ORANGE_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("orange"),
            color: Self::ORANGE_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("orange"),
            color: Self::ORANGE_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("orange"),
            color: Self::ORANGE_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("orange"),
            color: Self::ORANGE_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("orange"),
            color: Self::ORANGE_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("orange"),
            color: Self::ORANGE_950,
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
            group: Cow::Borrowed("violet"),
            color: Self::VIOLET_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("violet"),
            color: Self::VIOLET_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("violet"),
            color: Self::VIOLET_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("violet"),
            color: Self::VIOLET_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("violet"),
            color: Self::VIOLET_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("violet"),
            color: Self::VIOLET_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("violet"),
            color: Self::VIOLET_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("violet"),
            color: Self::VIOLET_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("violet"),
            color: Self::VIOLET_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("violet"),
            color: Self::VIOLET_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("violet"),
            color: Self::VIOLET_950,
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
    ];
}
