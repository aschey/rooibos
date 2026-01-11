use std::borrow::Cow;

use crate::{Color, NamedColor, ThemeArray};

// Auto-generated file. Do not edit.

pub struct TokyoNight {}

impl TokyoNight {
    pub const NAME: &str = "tokyo-night";

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_50: Color = Color::Rgb(243, 243, 247);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_100: Color = Color::Rgb(227, 229, 237);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_200: Color = Color::Rgb(200, 203, 220);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_300: Color = Color::Rgb(177, 181, 205);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_400: Color = Color::Rgb(150, 156, 189);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_500: Color = Color::Rgb(125, 132, 172);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_600: Color = Color::Rgb(100, 109, 153);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_700: Color = Color::Rgb(82, 90, 127);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_800: Color = Color::Rgb(63, 69, 99);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_900: Color = Color::Rgb(45, 49, 72);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GRAY_950: Color = Color::Rgb(36, 40, 59);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_50: Color = Color::Rgb(240, 243, 254);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_100: Color = Color::Rgb(225, 232, 253);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_200: Color = Color::Rgb(194, 209, 251);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_300: Color = Color::Rgb(158, 184, 249);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_400: Color = Color::Rgb(122, 162, 247);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_500: Color = Color::Rgb(53, 132, 244);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_600: Color = Color::Rgb(37, 105, 197);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_700: Color = Color::Rgb(26, 79, 151);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_800: Color = Color::Rgb(14, 52, 103);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_900: Color = Color::Rgb(5, 29, 62);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const BLUE_950: Color = Color::Rgb(3, 19, 44);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_50: Color = Color::Rgb(218, 245, 254);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_100: Color = Color::Rgb(184, 238, 254);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_200: Color = Color::Rgb(53, 222, 253);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_300: Color = Color::Rgb(43, 195, 222);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_400: Color = Color::Rgb(35, 164, 187);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_500: Color = Color::Rgb(27, 136, 155);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_600: Color = Color::Rgb(20, 109, 125);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_700: Color = Color::Rgb(11, 80, 93);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_800: Color = Color::Rgb(5, 56, 65);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_900: Color = Color::Rgb(2, 31, 37);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const CYAN_950: Color = Color::Rgb(1, 19, 24);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_50: Color = Color::Rgb(223, 252, 252);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_100: Color = Color::Rgb(180, 249, 248);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_200: Color = Color::Rgb(126, 222, 221);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_300: Color = Color::Rgb(109, 193, 192);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_400: Color = Color::Rgb(90, 162, 161);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_500: Color = Color::Rgb(74, 134, 133);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_600: Color = Color::Rgb(58, 107, 107);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_700: Color = Color::Rgb(41, 79, 79);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_800: Color = Color::Rgb(27, 55, 55);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_900: Color = Color::Rgb(12, 30, 30);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const TEAL_950: Color = Color::Rgb(6, 19, 19);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_50: Color = Color::Rgb(226, 252, 205);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_100: Color = Color::Rgb(202, 250, 155);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_200: Color = Color::Rgb(175, 228, 118);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_300: Color = Color::Rgb(158, 206, 106);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_400: Color = Color::Rgb(132, 173, 88);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_500: Color = Color::Rgb(108, 142, 71);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_600: Color = Color::Rgb(85, 112, 55);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_700: Color = Color::Rgb(63, 84, 39);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_800: Color = Color::Rgb(42, 57, 25);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_900: Color = Color::Rgb(22, 33, 11);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const GREEN_950: Color = Color::Rgb(14, 22, 6);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MINT_50: Color = Color::Rgb(224, 252, 247);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MINT_100: Color = Color::Rgb(188, 250, 239);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MINT_200: Color = Color::Rgb(125, 237, 220);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MINT_300: Color = Color::Rgb(114, 218, 202);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MINT_400: Color = Color::Rgb(96, 183, 170);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MINT_500: Color = Color::Rgb(78, 152, 140);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MINT_600: Color = Color::Rgb(60, 119, 110);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MINT_700: Color = Color::Rgb(44, 90, 83);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MINT_800: Color = Color::Rgb(27, 60, 55);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MINT_900: Color = Color::Rgb(13, 35, 32);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const MINT_950: Color = Color::Rgb(6, 22, 20);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const LAVENDER_50: Color = Color::Rgb(245, 242, 254);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const LAVENDER_100: Color = Color::Rgb(236, 229, 253);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const LAVENDER_200: Color = Color::Rgb(220, 205, 251);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const LAVENDER_300: Color = Color::Rgb(202, 179, 249);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const LAVENDER_400: Color = Color::Rgb(187, 154, 247);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const LAVENDER_500: Color = Color::Rgb(164, 112, 243);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const LAVENDER_600: Color = Color::Rgb(144, 59, 238);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const LAVENDER_700: Color = Color::Rgb(113, 33, 194);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const LAVENDER_800: Color = Color::Rgb(77, 20, 135);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const LAVENDER_900: Color = Color::Rgb(46, 8, 84);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const LAVENDER_950: Color = Color::Rgb(30, 4, 58);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSE_50: Color = Color::Rgb(255, 236, 240);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSE_100: Color = Color::Rgb(255, 221, 228);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSE_200: Color = Color::Rgb(255, 185, 202);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSE_300: Color = Color::Rgb(255, 147, 176);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSE_400: Color = Color::Rgb(255, 101, 150);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSE_500: Color = Color::Rgb(255, 1, 124);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSE_600: Color = Color::Rgb(205, 0, 98);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSE_700: Color = Color::Rgb(156, 0, 73);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSE_800: Color = Color::Rgb(110, 0, 49);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSE_900: Color = Color::Rgb(67, 0, 27);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ROSE_950: Color = Color::Rgb(43, 0, 15);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_50: Color = Color::Rgb(255, 241, 236);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_100: Color = Color::Rgb(255, 223, 211);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_200: Color = Color::Rgb(255, 193, 166);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_300: Color = Color::Rgb(255, 158, 100);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_400: Color = Color::Rgb(233, 124, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_500: Color = Color::Rgb(194, 102, 2);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_600: Color = Color::Rgb(154, 79, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_700: Color = Color::Rgb(118, 60, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_800: Color = Color::Rgb(82, 39, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_900: Color = Color::Rgb(50, 22, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const ORANGE_950: Color = Color::Rgb(32, 11, 0);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_50: Color = Color::Rgb(242, 239, 250);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_100: Color = Color::Rgb(230, 223, 245);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_200: Color = Color::Rgb(203, 189, 235);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_300: Color = Color::Rgb(180, 157, 226);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_400: Color = Color::Rgb(157, 124, 216);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_500: Color = Color::Rgb(137, 93, 205);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_600: Color = Color::Rgb(113, 66, 179);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_700: Color = Color::Rgb(85, 48, 136);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_800: Color = Color::Rgb(61, 33, 99);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_900: Color = Color::Rgb(36, 17, 62);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PURPLE_950: Color = Color::Rgb(23, 9, 42);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_50: Color = Color::Rgb(254, 240, 242);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_100: Color = Color::Rgb(252, 222, 226);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_200: Color = Color::Rgb(250, 192, 200);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_300: Color = Color::Rgb(248, 156, 171);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_400: Color = Color::Rgb(247, 118, 142);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_500: Color = Color::Rgb(246, 51, 104);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_600: Color = Color::Rgb(199, 32, 81);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_700: Color = Color::Rgb(152, 22, 59);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_800: Color = Color::Rgb(107, 12, 39);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_900: Color = Color::Rgb(65, 4, 21);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const PINK_950: Color = Color::Rgb(42, 2, 11);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_50: Color = Color::Rgb(250, 238, 238);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_100: Color = Color::Rgb(246, 224, 224);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_200: Color = Color::Rgb(237, 190, 190);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_300: Color = Color::Rgb(231, 157, 157);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_400: Color = Color::Rgb(225, 118, 118);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_500: Color = Color::Rgb(219, 75, 75);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_600: Color = Color::Rgb(175, 58, 58);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_700: Color = Color::Rgb(135, 43, 43);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_800: Color = Color::Rgb(94, 27, 27);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_900: Color = Color::Rgb(59, 14, 14);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const RED_950: Color = Color::Rgb(38, 6, 6);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_50: Color = Color::Rgb(251, 242, 233);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_100: Color = Color::Rgb(248, 229, 210);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_200: Color = Color::Rgb(242, 200, 147);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_300: Color = Color::Rgb(224, 175, 104);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_400: Color = Color::Rgb(190, 148, 87);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_500: Color = Color::Rgb(156, 121, 70);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_600: Color = Color::Rgb(125, 97, 55);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_700: Color = Color::Rgb(94, 72, 39);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_800: Color = Color::Rgb(64, 48, 25);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_900: Color = Color::Rgb(38, 28, 12);

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    pub const YELLOW_950: Color = Color::Rgb(23, 16, 5);

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

    pub const MINT: ThemeArray<11> = ThemeArray([
        Self::MINT_50,
        Self::MINT_100,
        Self::MINT_200,
        Self::MINT_300,
        Self::MINT_400,
        Self::MINT_500,
        Self::MINT_600,
        Self::MINT_700,
        Self::MINT_800,
        Self::MINT_900,
        Self::MINT_950,
    ]);

    pub const LAVENDER: ThemeArray<11> = ThemeArray([
        Self::LAVENDER_50,
        Self::LAVENDER_100,
        Self::LAVENDER_200,
        Self::LAVENDER_300,
        Self::LAVENDER_400,
        Self::LAVENDER_500,
        Self::LAVENDER_600,
        Self::LAVENDER_700,
        Self::LAVENDER_800,
        Self::LAVENDER_900,
        Self::LAVENDER_950,
    ]);

    pub const ROSE: ThemeArray<11> = ThemeArray([
        Self::ROSE_50,
        Self::ROSE_100,
        Self::ROSE_200,
        Self::ROSE_300,
        Self::ROSE_400,
        Self::ROSE_500,
        Self::ROSE_600,
        Self::ROSE_700,
        Self::ROSE_800,
        Self::ROSE_900,
        Self::ROSE_950,
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
            group: Cow::Borrowed("mint"),
            color: Self::MINT_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("mint"),
            color: Self::MINT_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("mint"),
            color: Self::MINT_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("mint"),
            color: Self::MINT_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("mint"),
            color: Self::MINT_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("mint"),
            color: Self::MINT_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("mint"),
            color: Self::MINT_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("mint"),
            color: Self::MINT_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("mint"),
            color: Self::MINT_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("mint"),
            color: Self::MINT_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("mint"),
            color: Self::MINT_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("lavender"),
            color: Self::LAVENDER_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("lavender"),
            color: Self::LAVENDER_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("lavender"),
            color: Self::LAVENDER_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("lavender"),
            color: Self::LAVENDER_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("lavender"),
            color: Self::LAVENDER_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("lavender"),
            color: Self::LAVENDER_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("lavender"),
            color: Self::LAVENDER_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("lavender"),
            color: Self::LAVENDER_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("lavender"),
            color: Self::LAVENDER_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("lavender"),
            color: Self::LAVENDER_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("lavender"),
            color: Self::LAVENDER_950,
        },
        NamedColor {
            variant: Cow::Borrowed("50"),
            group: Cow::Borrowed("rose"),
            color: Self::ROSE_50,
        },
        NamedColor {
            variant: Cow::Borrowed("100"),
            group: Cow::Borrowed("rose"),
            color: Self::ROSE_100,
        },
        NamedColor {
            variant: Cow::Borrowed("200"),
            group: Cow::Borrowed("rose"),
            color: Self::ROSE_200,
        },
        NamedColor {
            variant: Cow::Borrowed("300"),
            group: Cow::Borrowed("rose"),
            color: Self::ROSE_300,
        },
        NamedColor {
            variant: Cow::Borrowed("400"),
            group: Cow::Borrowed("rose"),
            color: Self::ROSE_400,
        },
        NamedColor {
            variant: Cow::Borrowed("500"),
            group: Cow::Borrowed("rose"),
            color: Self::ROSE_500,
        },
        NamedColor {
            variant: Cow::Borrowed("600"),
            group: Cow::Borrowed("rose"),
            color: Self::ROSE_600,
        },
        NamedColor {
            variant: Cow::Borrowed("700"),
            group: Cow::Borrowed("rose"),
            color: Self::ROSE_700,
        },
        NamedColor {
            variant: Cow::Borrowed("800"),
            group: Cow::Borrowed("rose"),
            color: Self::ROSE_800,
        },
        NamedColor {
            variant: Cow::Borrowed("900"),
            group: Cow::Borrowed("rose"),
            color: Self::ROSE_900,
        },
        NamedColor {
            variant: Cow::Borrowed("950"),
            group: Cow::Borrowed("rose"),
            color: Self::ROSE_950,
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
    ];
}
