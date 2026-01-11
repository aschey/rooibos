use std::borrow::Cow;

use crate::{Color, NamedColor, ThemeArray};

// Auto-generated file. Do not edit.

pub struct Gruvbox {}

impl Gruvbox {
    pub const NAME: &str = "gruvbox";

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_50: Color = Color::Rgb(238, 241, 243);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_100: Color = Color::Rgb(224, 230, 233);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_200: Color = Color::Rgb(189, 204, 208);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_300: Color = Color::Rgb(166, 179, 183);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_400: Color = Color::Rgb(143, 154, 158);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_500: Color = Color::Rgb(121, 131, 134);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_600: Color = Color::Rgb(98, 105, 108);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_700: Color = Color::Rgb(77, 83, 86);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_800: Color = Color::Rgb(57, 62, 64);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_900: Color = Color::Rgb(39, 43, 44);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_950: Color = Color::Rgb(29, 32, 33);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_50: Color = Color::Rgb(254, 240, 240);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_100: Color = Color::Rgb(254, 226, 225);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_200: Color = Color::Rgb(253, 192, 189);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_300: Color = Color::Rgb(252, 160, 155);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_400: Color = Color::Rgb(251, 121, 112);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_500: Color = Color::Rgb(251, 73, 52);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_600: Color = Color::Rgb(206, 47, 20);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_700: Color = Color::Rgb(159, 34, 13);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_800: Color = Color::Rgb(109, 20, 6);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_900: Color = Color::Rgb(68, 9, 2);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_950: Color = Color::Rgb(46, 4, 1);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_50: Color = Color::Rgb(248, 252, 54);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_100: Color = Color::Rgb(234, 237, 50);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_200: Color = Color::Rgb(208, 211, 44);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_300: Color = Color::Rgb(184, 187, 38);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_400: Color = Color::Rgb(155, 158, 32);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_500: Color = Color::Rgb(127, 129, 24);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_600: Color = Color::Rgb(101, 103, 15);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_700: Color = Color::Rgb(75, 76, 9);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_800: Color = Color::Rgb(52, 53, 5);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_900: Color = Color::Rgb(28, 29, 2);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_950: Color = Color::Rgb(17, 18, 1);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_50: Color = Color::Rgb(255, 245, 234);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_100: Color = Color::Rgb(254, 232, 204);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_200: Color = Color::Rgb(253, 212, 150);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_300: Color = Color::Rgb(250, 189, 48);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_400: Color = Color::Rgb(212, 160, 38);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_500: Color = Color::Rgb(173, 130, 29);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_600: Color = Color::Rgb(139, 103, 21);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_700: Color = Color::Rgb(103, 76, 13);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_800: Color = Color::Rgb(71, 52, 6);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_900: Color = Color::Rgb(40, 27, 2);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_950: Color = Color::Rgb(27, 18, 1);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_50: Color = Color::Rgb(231, 247, 241);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_100: Color = Color::Rgb(198, 238, 222);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_200: Color = Color::Rgb(170, 213, 197);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_300: Color = Color::Rgb(152, 190, 175);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_400: Color = Color::Rgb(131, 165, 152);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_500: Color = Color::Rgb(107, 135, 124);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_600: Color = Color::Rgb(84, 107, 98);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_700: Color = Color::Rgb(61, 79, 72);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_800: Color = Color::Rgb(42, 56, 50);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_900: Color = Color::Rgb(23, 31, 28);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_950: Color = Color::Rgb(12, 18, 16);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_50: Color = Color::Rgb(249, 242, 243);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_100: Color = Color::Rgb(242, 225, 229);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_200: Color = Color::Rgb(230, 195, 203);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_300: Color = Color::Rgb(220, 165, 178);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_400: Color = Color::Rgb(211, 134, 155);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_500: Color = Color::Rgb(199, 95, 128);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_600: Color = Color::Rgb(158, 74, 100);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_700: Color = Color::Rgb(119, 54, 74);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_800: Color = Color::Rgb(82, 35, 50);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_900: Color = Color::Rgb(51, 19, 29);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_950: Color = Color::Rgb(32, 10, 17);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_50: Color = Color::Rgb(222, 248, 214);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_100: Color = Color::Rgb(190, 244, 172);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_200: Color = Color::Rgb(162, 219, 142);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_300: Color = Color::Rgb(142, 192, 124);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_400: Color = Color::Rgb(119, 161, 103);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_500: Color = Color::Rgb(98, 133, 85);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_600: Color = Color::Rgb(78, 107, 67);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_700: Color = Color::Rgb(56, 79, 48);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_800: Color = Color::Rgb(38, 55, 32);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_900: Color = Color::Rgb(20, 30, 16);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_950: Color = Color::Rgb(11, 19, 8);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_50: Color = Color::Rgb(255, 241, 237);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_100: Color = Color::Rgb(255, 226, 218);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_200: Color = Color::Rgb(254, 193, 171);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_300: Color = Color::Rgb(254, 162, 121);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_400: Color = Color::Rgb(254, 128, 25);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_500: Color = Color::Rgb(211, 104, 12);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_600: Color = Color::Rgb(169, 82, 6);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_700: Color = Color::Rgb(125, 59, 3);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_800: Color = Color::Rgb(87, 39, 2);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_900: Color = Color::Rgb(52, 21, 1);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_950: Color = Color::Rgb(37, 13, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_50: Color = Color::Rgb(246, 243, 241);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_100: Color = Color::Rgb(235, 228, 223);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_200: Color = Color::Rgb(219, 204, 193);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_300: Color = Color::Rgb(199, 178, 162);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_400: Color = Color::Rgb(172, 155, 140);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_500: Color = Color::Rgb(147, 132, 119);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_600: Color = Color::Rgb(124, 111, 100);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_700: Color = Color::Rgb(93, 83, 74);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_800: Color = Color::Rgb(63, 56, 50);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_900: Color = Color::Rgb(35, 31, 27);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BROWN_950: Color = Color::Rgb(22, 19, 16);

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

    pub const PURPLE: ThemeArray<11> = ThemeArray([
        Self::PURPLE_50,
        Self::PURPLE_100,
        Self::PURPLE_200,
        Self::PURPLE_300,
        Self::PURPLE_400,
        Self::PURPLE_500,
        Self::PURPLE_600,
        Self::PURPLE_700,
        Self::PURPLE_800,
        Self::PURPLE_900,
        Self::PURPLE_950,
    ]);

    pub const AQUA: ThemeArray<11> = ThemeArray([
        Self::AQUA_50,
        Self::AQUA_100,
        Self::AQUA_200,
        Self::AQUA_300,
        Self::AQUA_400,
        Self::AQUA_500,
        Self::AQUA_600,
        Self::AQUA_700,
        Self::AQUA_800,
        Self::AQUA_900,
        Self::AQUA_950,
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

    pub const ALL_COLORS: [NamedColor<'_>; 99] = [
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
            group: Cow::Borrowed("purple"),
            color: Self::PURPLE_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("purple"),
            color: Self::PURPLE_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("purple"),
            color: Self::PURPLE_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("purple"),
            color: Self::PURPLE_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("purple"),
            color: Self::PURPLE_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("purple"),
            color: Self::PURPLE_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("purple"),
            color: Self::PURPLE_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("purple"),
            color: Self::PURPLE_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("purple"),
            color: Self::PURPLE_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("purple"),
            color: Self::PURPLE_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("purple"),
            color: Self::PURPLE_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("aqua"),
            color: Self::AQUA_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("aqua"),
            color: Self::AQUA_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("aqua"),
            color: Self::AQUA_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("aqua"),
            color: Self::AQUA_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("aqua"),
            color: Self::AQUA_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("aqua"),
            color: Self::AQUA_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("aqua"),
            color: Self::AQUA_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("aqua"),
            color: Self::AQUA_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("aqua"),
            color: Self::AQUA_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("aqua"),
            color: Self::AQUA_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("aqua"),
            color: Self::AQUA_950,
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
    ];
}
