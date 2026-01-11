use std::borrow::Cow;

use crate::{Color, NamedColor, ThemeArray};

// Auto-generated file. Do not edit.

pub struct Kanagawa {}

impl Kanagawa {
    pub const NAME: &str = "kanagawa";

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_50: Color = Color::Rgb(240, 240, 242);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_100: Color = Color::Rgb(229, 229, 232);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_200: Color = Color::Rgb(203, 203, 209);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_300: Color = Color::Rgb(176, 176, 185);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_400: Color = Color::Rgb(151, 151, 164);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_500: Color = Color::Rgb(128, 128, 143);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_600: Color = Color::Rgb(105, 105, 122);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_700: Color = Color::Rgb(81, 81, 98);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_800: Color = Color::Rgb(60, 60, 75);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_900: Color = Color::Rgb(41, 41, 52);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_950: Color = Color::Rgb(31, 31, 39);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_50: Color = Color::Rgb(241, 244, 250);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_100: Color = Color::Rgb(223, 230, 244);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_200: Color = Color::Rgb(190, 205, 233);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_300: Color = Color::Rgb(156, 180, 223);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_400: Color = Color::Rgb(124, 159, 213);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_500: Color = Color::Rgb(98, 136, 190);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_600: Color = Color::Rgb(81, 113, 158);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_700: Color = Color::Rgb(64, 91, 128);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_800: Color = Color::Rgb(48, 69, 99);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_900: Color = Color::Rgb(34, 50, 73);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_950: Color = Color::Rgb(17, 28, 43);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const INDIGO_50: Color = Color::Rgb(238, 244, 251);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const INDIGO_100: Color = Color::Rgb(220, 234, 248);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const INDIGO_200: Color = Color::Rgb(178, 211, 240);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const INDIGO_300: Color = Color::Rgb(137, 191, 233);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const INDIGO_400: Color = Color::Rgb(101, 167, 214);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const INDIGO_500: Color = Color::Rgb(87, 145, 186);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const INDIGO_600: Color = Color::Rgb(72, 122, 156);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const INDIGO_700: Color = Color::Rgb(59, 101, 131);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const INDIGO_800: Color = Color::Rgb(45, 79, 103);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const INDIGO_900: Color = Color::Rgb(21, 42, 56);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const INDIGO_950: Color = Color::Rgb(11, 25, 35);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_50: Color = Color::Rgb(225, 249, 241);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_100: Color = Color::Rgb(192, 243, 228);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_200: Color = Color::Rgb(158, 219, 202);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_300: Color = Color::Rgb(140, 196, 180);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_400: Color = Color::Rgb(124, 173, 159);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_500: Color = Color::Rgb(106, 149, 137);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_600: Color = Color::Rgb(83, 118, 109);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_700: Color = Color::Rgb(61, 88, 80);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_800: Color = Color::Rgb(40, 59, 53);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_900: Color = Color::Rgb(21, 34, 31);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const AQUA_950: Color = Color::Rgb(12, 21, 19);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EGGPLANT_50: Color = Color::Rgb(247, 242, 243);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EGGPLANT_100: Color = Color::Rgb(238, 226, 229);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EGGPLANT_200: Color = Color::Rgb(224, 201, 205);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EGGPLANT_300: Color = Color::Rgb(210, 173, 180);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EGGPLANT_400: Color = Color::Rgb(198, 147, 157);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EGGPLANT_500: Color = Color::Rgb(187, 117, 132);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EGGPLANT_600: Color = Color::Rgb(165, 96, 112);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EGGPLANT_700: Color = Color::Rgb(134, 78, 90);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EGGPLANT_800: Color = Color::Rgb(105, 60, 70);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EGGPLANT_900: Color = Color::Rgb(80, 44, 52);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EGGPLANT_950: Color = Color::Rgb(67, 36, 43);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_50: Color = Color::Rgb(232, 248, 226);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_100: Color = Color::Rgb(206, 242, 192);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_200: Color = Color::Rgb(176, 219, 158);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_300: Color = Color::Rgb(157, 195, 141);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_400: Color = Color::Rgb(136, 170, 122);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_500: Color = Color::Rgb(118, 148, 106);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_600: Color = Color::Rgb(92, 116, 82);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_700: Color = Color::Rgb(69, 88, 61);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_800: Color = Color::Rgb(45, 59, 40);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_900: Color = Color::Rgb(25, 34, 22);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_950: Color = Color::Rgb(15, 21, 12);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_50: Color = Color::Rgb(251, 241, 241);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_100: Color = Color::Rgb(246, 224, 224);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_200: Color = Color::Rgb(239, 193, 193);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_300: Color = Color::Rgb(233, 160, 161);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_400: Color = Color::Rgb(228, 126, 127);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_500: Color = Color::Rgb(224, 84, 87);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_600: Color = Color::Rgb(195, 64, 67);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_700: Color = Color::Rgb(148, 46, 49);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_800: Color = Color::Rgb(102, 29, 31);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_900: Color = Color::Rgb(63, 15, 16);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_950: Color = Color::Rgb(42, 7, 8);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EARTH_YELLOW_50: Color = Color::Rgb(251, 239, 229);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EARTH_YELLOW_100: Color = Color::Rgb(248, 225, 207);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EARTH_YELLOW_200: Color = Color::Rgb(241, 192, 140);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EARTH_YELLOW_300: Color = Color::Rgb(220, 165, 97);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EARTH_YELLOW_400: Color = Color::Rgb(185, 138, 81);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EARTH_YELLOW_500: Color = Color::Rgb(153, 114, 65);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EARTH_YELLOW_600: Color = Color::Rgb(123, 91, 51);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EARTH_YELLOW_700: Color = Color::Rgb(94, 68, 37);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EARTH_YELLOW_800: Color = Color::Rgb(63, 45, 23);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EARTH_YELLOW_900: Color = Color::Rgb(37, 25, 10);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const EARTH_YELLOW_950: Color = Color::Rgb(24, 15, 5);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_50: Color = Color::Rgb(254, 237, 237);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_100: Color = Color::Rgb(252, 219, 219);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_200: Color = Color::Rgb(250, 181, 181);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_300: Color = Color::Rgb(248, 140, 140);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_400: Color = Color::Rgb(247, 92, 92);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_500: Color = Color::Rgb(232, 36, 36);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_600: Color = Color::Rgb(186, 27, 27);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_700: Color = Color::Rgb(142, 18, 18);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_800: Color = Color::Rgb(101, 9, 9);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_900: Color = Color::Rgb(63, 4, 4);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_950: Color = Color::Rgb(43, 2, 2);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_50: Color = Color::Rgb(255, 241, 234);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_100: Color = Color::Rgb(255, 223, 206);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_200: Color = Color::Rgb(255, 190, 148);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_300: Color = Color::Rgb(255, 158, 59);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_400: Color = Color::Rgb(221, 131, 1);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_500: Color = Color::Rgb(185, 108, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_600: Color = Color::Rgb(146, 84, 1);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_700: Color = Color::Rgb(112, 64, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_800: Color = Color::Rgb(77, 42, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_900: Color = Color::Rgb(47, 24, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PEACH_950: Color = Color::Rgb(30, 13, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_50: Color = Color::Rgb(244, 243, 248);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_100: Color = Color::Rgb(231, 227, 239);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_200: Color = Color::Rgb(211, 203, 226);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_300: Color = Color::Rgb(189, 176, 212);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_400: Color = Color::Rgb(170, 153, 199);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_500: Color = Color::Rgb(149, 127, 184);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_600: Color = Color::Rgb(124, 96, 164);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_700: Color = Color::Rgb(92, 69, 124);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_800: Color = Color::Rgb(64, 47, 87);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_900: Color = Color::Rgb(35, 25, 50);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const VIOLET_950: Color = Color::Rgb(23, 16, 35);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSY_BROWN_50: Color = Color::Rgb(245, 239, 238);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSY_BROWN_100: Color = Color::Rgb(238, 227, 223);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSY_BROWN_200: Color = Color::Rgb(221, 196, 187);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSY_BROWN_300: Color = Color::Rgb(208, 168, 152);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSY_BROWN_400: Color = Color::Rgb(185, 141, 123);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSY_BROWN_500: Color = Color::Rgb(153, 116, 101);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSY_BROWN_600: Color = Color::Rgb(123, 92, 80);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSY_BROWN_700: Color = Color::Rgb(91, 68, 58);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSY_BROWN_800: Color = Color::Rgb(64, 47, 40);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSY_BROWN_900: Color = Color::Rgb(36, 25, 20);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSY_BROWN_950: Color = Color::Rgb(23, 15, 12);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const HONEYDEW_50: Color = Color::Rgb(244, 247, 245);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const HONEYDEW_100: Color = Color::Rgb(237, 242, 237);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const HONEYDEW_200: Color = Color::Rgb(215, 227, 216);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const HONEYDEW_300: Color = Color::Rgb(184, 196, 185);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const HONEYDEW_400: Color = Color::Rgb(156, 166, 156);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const HONEYDEW_500: Color = Color::Rgb(128, 137, 129);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const HONEYDEW_600: Color = Color::Rgb(101, 108, 102);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const HONEYDEW_700: Color = Color::Rgb(74, 79, 74);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const HONEYDEW_800: Color = Color::Rgb(50, 54, 50);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const HONEYDEW_900: Color = Color::Rgb(28, 30, 28);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const HONEYDEW_950: Color = Color::Rgb(18, 20, 18);

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

    pub const INDIGO: ThemeArray<11> = ThemeArray([
        Self::INDIGO_50,
        Self::INDIGO_100,
        Self::INDIGO_200,
        Self::INDIGO_300,
        Self::INDIGO_400,
        Self::INDIGO_500,
        Self::INDIGO_600,
        Self::INDIGO_700,
        Self::INDIGO_800,
        Self::INDIGO_900,
        Self::INDIGO_950,
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

    pub const EGGPLANT: ThemeArray<11> = ThemeArray([
        Self::EGGPLANT_50,
        Self::EGGPLANT_100,
        Self::EGGPLANT_200,
        Self::EGGPLANT_300,
        Self::EGGPLANT_400,
        Self::EGGPLANT_500,
        Self::EGGPLANT_600,
        Self::EGGPLANT_700,
        Self::EGGPLANT_800,
        Self::EGGPLANT_900,
        Self::EGGPLANT_950,
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

    pub const EARTH_YELLOW: ThemeArray<11> = ThemeArray([
        Self::EARTH_YELLOW_50,
        Self::EARTH_YELLOW_100,
        Self::EARTH_YELLOW_200,
        Self::EARTH_YELLOW_300,
        Self::EARTH_YELLOW_400,
        Self::EARTH_YELLOW_500,
        Self::EARTH_YELLOW_600,
        Self::EARTH_YELLOW_700,
        Self::EARTH_YELLOW_800,
        Self::EARTH_YELLOW_900,
        Self::EARTH_YELLOW_950,
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

    pub const ROSY_BROWN: ThemeArray<11> = ThemeArray([
        Self::ROSY_BROWN_50,
        Self::ROSY_BROWN_100,
        Self::ROSY_BROWN_200,
        Self::ROSY_BROWN_300,
        Self::ROSY_BROWN_400,
        Self::ROSY_BROWN_500,
        Self::ROSY_BROWN_600,
        Self::ROSY_BROWN_700,
        Self::ROSY_BROWN_800,
        Self::ROSY_BROWN_900,
        Self::ROSY_BROWN_950,
    ]);

    pub const HONEYDEW: ThemeArray<11> = ThemeArray([
        Self::HONEYDEW_50,
        Self::HONEYDEW_100,
        Self::HONEYDEW_200,
        Self::HONEYDEW_300,
        Self::HONEYDEW_400,
        Self::HONEYDEW_500,
        Self::HONEYDEW_600,
        Self::HONEYDEW_700,
        Self::HONEYDEW_800,
        Self::HONEYDEW_900,
        Self::HONEYDEW_950,
    ]);

    pub const ALL_COLORS: [NamedColor<'_>; 143] = [
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
            group: Cow::Borrowed("indigo"),
            color: Self::INDIGO_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("indigo"),
            color: Self::INDIGO_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("indigo"),
            color: Self::INDIGO_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("indigo"),
            color: Self::INDIGO_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("indigo"),
            color: Self::INDIGO_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("indigo"),
            color: Self::INDIGO_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("indigo"),
            color: Self::INDIGO_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("indigo"),
            color: Self::INDIGO_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("indigo"),
            color: Self::INDIGO_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("indigo"),
            color: Self::INDIGO_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("indigo"),
            color: Self::INDIGO_950,
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
            group: Cow::Borrowed("eggplant"),
            color: Self::EGGPLANT_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("eggplant"),
            color: Self::EGGPLANT_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("eggplant"),
            color: Self::EGGPLANT_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("eggplant"),
            color: Self::EGGPLANT_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("eggplant"),
            color: Self::EGGPLANT_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("eggplant"),
            color: Self::EGGPLANT_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("eggplant"),
            color: Self::EGGPLANT_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("eggplant"),
            color: Self::EGGPLANT_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("eggplant"),
            color: Self::EGGPLANT_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("eggplant"),
            color: Self::EGGPLANT_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("eggplant"),
            color: Self::EGGPLANT_950,
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
            group: Cow::Borrowed("earth_yellow"),
            color: Self::EARTH_YELLOW_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("earth_yellow"),
            color: Self::EARTH_YELLOW_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("earth_yellow"),
            color: Self::EARTH_YELLOW_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("earth_yellow"),
            color: Self::EARTH_YELLOW_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("earth_yellow"),
            color: Self::EARTH_YELLOW_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("earth_yellow"),
            color: Self::EARTH_YELLOW_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("earth_yellow"),
            color: Self::EARTH_YELLOW_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("earth_yellow"),
            color: Self::EARTH_YELLOW_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("earth_yellow"),
            color: Self::EARTH_YELLOW_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("earth_yellow"),
            color: Self::EARTH_YELLOW_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("earth_yellow"),
            color: Self::EARTH_YELLOW_950,
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
            group: Cow::Borrowed("rosy_brown"),
            color: Self::ROSY_BROWN_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("rosy_brown"),
            color: Self::ROSY_BROWN_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("rosy_brown"),
            color: Self::ROSY_BROWN_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("rosy_brown"),
            color: Self::ROSY_BROWN_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("rosy_brown"),
            color: Self::ROSY_BROWN_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("rosy_brown"),
            color: Self::ROSY_BROWN_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("rosy_brown"),
            color: Self::ROSY_BROWN_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("rosy_brown"),
            color: Self::ROSY_BROWN_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("rosy_brown"),
            color: Self::ROSY_BROWN_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("rosy_brown"),
            color: Self::ROSY_BROWN_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("rosy_brown"),
            color: Self::ROSY_BROWN_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("honeydew"),
            color: Self::HONEYDEW_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("honeydew"),
            color: Self::HONEYDEW_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("honeydew"),
            color: Self::HONEYDEW_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("honeydew"),
            color: Self::HONEYDEW_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("honeydew"),
            color: Self::HONEYDEW_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("honeydew"),
            color: Self::HONEYDEW_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("honeydew"),
            color: Self::HONEYDEW_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("honeydew"),
            color: Self::HONEYDEW_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("honeydew"),
            color: Self::HONEYDEW_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("honeydew"),
            color: Self::HONEYDEW_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("honeydew"),
            color: Self::HONEYDEW_950,
        },
    ];
}
