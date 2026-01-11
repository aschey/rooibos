use std::borrow::Cow;

use crate::{Color, NamedColor, ThemeArray};

// Auto-generated file. Do not edit.

pub struct Nord {}

impl Nord {
    pub const NAME: &str = "nord";

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_50: Color = Color::Rgb(242, 244, 246);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_100: Color = Color::Rgb(226, 229, 236);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_200: Color = Color::Rgb(201, 207, 219);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_300: Color = Color::Rgb(173, 182, 201);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_400: Color = Color::Rgb(146, 158, 184);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_500: Color = Color::Rgb(121, 135, 162);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_600: Color = Color::Rgb(103, 114, 138);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_700: Color = Color::Rgb(83, 92, 111);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_800: Color = Color::Rgb(63, 71, 86);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_900: Color = Color::Rgb(46, 52, 64);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_950: Color = Color::Rgb(26, 30, 38);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SEAFOAM_50: Color = Color::Rgb(225, 248, 248);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SEAFOAM_100: Color = Color::Rgb(183, 240, 239);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SEAFOAM_200: Color = Color::Rgb(163, 213, 212);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SEAFOAM_300: Color = Color::Rgb(143, 188, 187);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SEAFOAM_400: Color = Color::Rgb(121, 160, 159);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SEAFOAM_500: Color = Color::Rgb(98, 130, 129);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SEAFOAM_600: Color = Color::Rgb(78, 104, 103);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SEAFOAM_700: Color = Color::Rgb(57, 77, 76);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SEAFOAM_800: Color = Color::Rgb(38, 53, 53);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SEAFOAM_900: Color = Color::Rgb(20, 29, 29);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const SEAFOAM_950: Color = Color::Rgb(11, 18, 18);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_50: Color = Color::Rgb(235, 245, 249);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_100: Color = Color::Rgb(208, 234, 242);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_200: Color = Color::Rgb(160, 215, 232);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_300: Color = Color::Rgb(136, 192, 208);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_400: Color = Color::Rgb(115, 163, 176);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_500: Color = Color::Rgb(93, 133, 144);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_600: Color = Color::Rgb(74, 107, 116);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_700: Color = Color::Rgb(54, 79, 86);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_800: Color = Color::Rgb(35, 53, 59);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_900: Color = Color::Rgb(19, 31, 35);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_950: Color = Color::Rgb(9, 18, 21);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_50: Color = Color::Rgb(240, 244, 248);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_100: Color = Color::Rgb(222, 230, 240);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_200: Color = Color::Rgb(189, 206, 225);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_300: Color = Color::Rgb(159, 185, 213);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_400: Color = Color::Rgb(129, 161, 193);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_500: Color = Color::Rgb(105, 132, 159);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_600: Color = Color::Rgb(82, 104, 125);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_700: Color = Color::Rgb(60, 77, 94);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_800: Color = Color::Rgb(42, 54, 66);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_900: Color = Color::Rgb(22, 30, 38);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_950: Color = Color::Rgb(12, 17, 23);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_50: Color = Color::Rgb(247, 239, 239);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_100: Color = Color::Rgb(240, 226, 227);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_200: Color = Color::Rgb(225, 194, 196);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_300: Color = Color::Rgb(213, 164, 168);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_400: Color = Color::Rgb(202, 130, 136);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_500: Color = Color::Rgb(191, 97, 106);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_600: Color = Color::Rgb(152, 76, 83);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_700: Color = Color::Rgb(117, 57, 63);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_800: Color = Color::Rgb(81, 37, 41);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_900: Color = Color::Rgb(50, 21, 23);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_950: Color = Color::Rgb(31, 11, 13);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_50: Color = Color::Rgb(247, 239, 237);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_100: Color = Color::Rgb(242, 226, 223);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_200: Color = Color::Rgb(230, 197, 189);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_300: Color = Color::Rgb(219, 164, 149);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_400: Color = Color::Rgb(208, 135, 112);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_500: Color = Color::Rgb(173, 111, 92);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_600: Color = Color::Rgb(136, 87, 71);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_700: Color = Color::Rgb(105, 66, 53);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_800: Color = Color::Rgb(72, 43, 35);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_900: Color = Color::Rgb(44, 25, 19);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_950: Color = Color::Rgb(27, 13, 9);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_50: Color = Color::Rgb(251, 246, 238);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_100: Color = Color::Rgb(248, 236, 219);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_200: Color = Color::Rgb(242, 221, 185);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_300: Color = Color::Rgb(235, 203, 139);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_400: Color = Color::Rgb(198, 171, 116);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_500: Color = Color::Rgb(162, 139, 94);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_600: Color = Color::Rgb(127, 109, 73);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_700: Color = Color::Rgb(97, 83, 54);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_800: Color = Color::Rgb(66, 56, 35);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_900: Color = Color::Rgb(37, 31, 18);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_950: Color = Color::Rgb(24, 19, 9);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_50: Color = Color::Rgb(233, 248, 222);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_100: Color = Color::Rgb(206, 240, 178);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_200: Color = Color::Rgb(186, 216, 160);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_300: Color = Color::Rgb(163, 190, 140);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_400: Color = Color::Rgb(137, 160, 117);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_500: Color = Color::Rgb(113, 133, 97);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_600: Color = Color::Rgb(88, 104, 75);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_700: Color = Color::Rgb(67, 79, 56);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_800: Color = Color::Rgb(44, 53, 37);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_900: Color = Color::Rgb(25, 31, 20);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_950: Color = Color::Rgb(14, 18, 10);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_50: Color = Color::Rgb(244, 239, 243);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_100: Color = Color::Rgb(236, 227, 234);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_200: Color = Color::Rgb(217, 199, 213);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_300: Color = Color::Rgb(199, 171, 193);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_400: Color = Color::Rgb(180, 142, 173);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_500: Color = Color::Rgb(153, 115, 146);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_600: Color = Color::Rgb(121, 90, 115);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_700: Color = Color::Rgb(92, 68, 88);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_800: Color = Color::Rgb(63, 45, 60);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_900: Color = Color::Rgb(38, 26, 36);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_950: Color = Color::Rgb(23, 14, 21);

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

    pub const SEAFOAM: ThemeArray<11> = ThemeArray([
        Self::SEAFOAM_50,
        Self::SEAFOAM_100,
        Self::SEAFOAM_200,
        Self::SEAFOAM_300,
        Self::SEAFOAM_400,
        Self::SEAFOAM_500,
        Self::SEAFOAM_600,
        Self::SEAFOAM_700,
        Self::SEAFOAM_800,
        Self::SEAFOAM_900,
        Self::SEAFOAM_950,
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
            group: Cow::Borrowed("seafoam"),
            color: Self::SEAFOAM_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("seafoam"),
            color: Self::SEAFOAM_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("seafoam"),
            color: Self::SEAFOAM_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("seafoam"),
            color: Self::SEAFOAM_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("seafoam"),
            color: Self::SEAFOAM_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("seafoam"),
            color: Self::SEAFOAM_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("seafoam"),
            color: Self::SEAFOAM_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("seafoam"),
            color: Self::SEAFOAM_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("seafoam"),
            color: Self::SEAFOAM_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("seafoam"),
            color: Self::SEAFOAM_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("seafoam"),
            color: Self::SEAFOAM_950,
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
    ];
}
