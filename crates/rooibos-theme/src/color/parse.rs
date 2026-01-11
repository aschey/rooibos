use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::LazyLock;

use ::palette::white_point::D50;
use ::palette::{
    Hsl, Hsluv, Hwb, Lab, Lch, Lchuv, Luv, Okhsl, Okhsv, Okhwb, Oklab, Oklch, Srgb, Xyz, Yxy,
};
use palette::Hsv;
use regex::{Captures, Regex};

use super::Color;

static SEP: &str = r"\s*(?:,|\s+)\s*";
static DIGITS: &str = r"\d{1,3}";
static DEC: &str = r"(?:\.\d+)?";
static PCT: &str = r"\d{1,3}(?:\.\d+)?%?";

struct Bounds<T> {
    min: T,
    max: T,
}

impl<T> Bounds<T> {
    fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

fn parse_capture(
    i: usize,
    bounds: impl Into<Option<Bounds<f32>>>,
    captures: &Captures,
) -> Option<f32> {
    let bounds = bounds.into();
    let s = captures.get(i).unwrap().as_str();
    if let Some(bounds) = bounds
        && s.ends_with('%')
    {
        let s = s.trim_end_matches('%');
        let mut val: f32 = s.parse().unwrap();
        val /= 100.0;
        return Some(val * (bounds.max - bounds.min) + bounds.min);
    }
    s.parse().ok()
}

static HEX_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#[a-fA-F0-9]{6};?$").unwrap());

static RGB_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        "^rgb\\(({DIGITS}){SEP}({DIGITS}){SEP}({DIGITS})\\);?$"
    ))
    .unwrap()
});

static HSL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!("^hsl\\(({DIGITS}){SEP}({PCT}){SEP}({PCT})\\);?$")).unwrap()
});

static HSV_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!("^hsv\\(({DIGITS}){SEP}({PCT}){SEP}({PCT})\\);?$")).unwrap()
});

static HSLUV_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        "^hsluv\\(({DIGITS}){SEP}({PCT}){SEP}({PCT})\\);?$"
    ))
    .unwrap()
});

static HWB_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!("^hwb\\(({DIGITS}){SEP}({PCT}){SEP}({PCT});?$\\)")).unwrap()
});

static LAB_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!("^lab\\(({PCT}){SEP}(-?{PCT}){SEP}(-?{PCT})\\);?$")).unwrap()
});

static LCH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!("^lch\\({PCT}{SEP}{PCT}{SEP}{DIGITS}{DEC}\\);?$")).unwrap()
});

static LCHUV_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        "lchuv\\(({PCT}){SEP}({PCT}){SEP}({DIGITS}{DEC})\\);?$"
    ))
    .unwrap()
});

static LUV_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!("^luv\\(({PCT}{SEP})(-?{PCT}){SEP}(-?{PCT})\\);?$")).unwrap()
});

static OKHSL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        "^okhsl\\(({DIGITS}{DEC}){SEP}({PCT}){SEP}({PCT})\\);?$"
    ))
    .unwrap()
});

static OKHSV_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        "^okhsv\\(({DIGITS}{DEC}){SEP}({PCT}){SEP}({PCT})\\);?$"
    ))
    .unwrap()
});

static OKHWB_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        "^okhwb\\(({DIGITS}{DEC}){SEP}({PCT}){SEP}({PCT})\\);?$"
    ))
    .unwrap()
});

static OKLAB_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        "^oklab\\(({PCT}){SEP}(-?{PCT}){SEP}(-?{PCT})\\);?$"
    ))
    .unwrap()
});

static OKLCH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        "^oklch\\(({PCT}){SEP}({PCT}){SEP}({DIGITS}{DEC})\\);?$"
    ))
    .unwrap()
});

static XYZ_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!("^xyz\\(({PCT}){SEP}({PCT}){SEP}({PCT})\\);?$")).unwrap());

static YXY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!("^yxy\\(({PCT}){SEP}({PCT}){SEP}({PCT})\\);?$")).unwrap());

impl Color {
    pub(super) fn parse_hex(s: &str) -> Option<Self> {
        if HEX_RE.is_match(s) {
            let rgb: Srgb<u8> = s.parse().unwrap();
            Some(Self::Rgb(rgb.red, rgb.green, rgb.blue))
        } else {
            None
        }
    }

    fn parse_rgb(s: &str) -> Option<Self> {
        RGB_RE.captures(s).and_then(|captures| {
            Some(
                Srgb::new(
                    parse_capture(1, None, &captures)? / 255.0,
                    parse_capture(2, None, &captures)? / 255.0,
                    parse_capture(3, None, &captures)? / 255.0,
                )
                .into(),
            )
        })
    }

    fn parse_hsl(s: &str) -> Option<Self> {
        HSL_RE.captures(s).and_then(|captures| {
            Some(
                Hsl::new(
                    parse_capture(1, None, &captures)?,
                    parse_capture(
                        2,
                        Bounds::new(Hsl::<Srgb>::min_saturation(), Hsl::<Srgb>::max_saturation()),
                        &captures,
                    )?,
                    parse_capture(
                        3,
                        Bounds::new(Hsl::<Srgb>::min_lightness(), Hsl::<Srgb>::max_lightness()),
                        &captures,
                    )?,
                )
                .into(),
            )
        })
    }

    fn parse_hsv(s: &str) -> Option<Self> {
        HSV_RE.captures(s).and_then(|captures| {
            Some(
                Hsv::new(
                    parse_capture(1, None, &captures)?,
                    parse_capture(
                        2,
                        Bounds::new(Hsv::<Srgb>::min_saturation(), Hsv::<Srgb>::max_saturation()),
                        &captures,
                    )?,
                    parse_capture(
                        3,
                        Bounds::new(Hsv::<Srgb>::min_value(), Hsv::<Srgb>::max_value()),
                        &captures,
                    )?,
                )
                .into(),
            )
        })
    }

    fn parse_hsluv(s: &str) -> Option<Self> {
        HSLUV_RE.captures(s).and_then(|captures| {
            Some(
                Hsluv::new(
                    parse_capture(1, None, &captures)?,
                    parse_capture(
                        2,
                        Bounds::new(
                            Hsluv::<Srgb>::min_saturation(),
                            Hsluv::<Srgb>::max_saturation(),
                        ),
                        &captures,
                    )?,
                    parse_capture(
                        3,
                        Bounds::new(Hsluv::<Srgb>::min_l(), Hsluv::<Srgb>::max_l()),
                        &captures,
                    )?,
                )
                .into(),
            )
        })
    }

    fn parse_hwb(s: &str) -> Option<Self> {
        HWB_RE.captures(s).and_then(|captures| {
            Some(
                Hwb::new(
                    parse_capture(1, None, &captures)?,
                    parse_capture(
                        2,
                        Bounds::new(Hwb::<Srgb>::min_whiteness(), Hwb::<Srgb>::max_whiteness()),
                        &captures,
                    )?,
                    parse_capture(
                        3,
                        Bounds::new(Hwb::<Srgb>::min_blackness(), Hwb::<Srgb>::max_blackness()),
                        &captures,
                    )?,
                )
                .into(),
            )
        })
    }

    fn parse_lab(s: &str) -> Option<Self> {
        LAB_RE.captures(s).and_then(|captures| {
            Some(
                Lab::new(
                    parse_capture(
                        1,
                        Bounds::new(Lab::<Srgb>::min_l(), Lab::<Srgb>::max_l()),
                        &captures,
                    )?,
                    parse_capture(
                        2,
                        Bounds::new(Lab::<Srgb>::min_a(), Lab::<Srgb>::max_a()),
                        &captures,
                    )?,
                    parse_capture(
                        3,
                        Bounds::new(Lab::<Srgb>::min_b(), Lab::<Srgb>::max_b()),
                        &captures,
                    )?,
                )
                .into(),
            )
        })
    }

    fn parse_lch(s: &str) -> Option<Self> {
        LCH_RE.captures(s).and_then(|captures| {
            Some(
                Lch::new(
                    parse_capture(
                        1,
                        Bounds::new(Lch::<Srgb>::min_l(), Lch::<Srgb>::max_l()),
                        &captures,
                    )?,
                    parse_capture(
                        2,
                        Bounds::new(Lch::<Srgb>::min_chroma(), Lch::<Srgb>::max_chroma()),
                        &captures,
                    )?,
                    parse_capture(3, None, &captures)?,
                )
                .into(),
            )
        })
    }

    fn parse_lchuv(s: &str) -> Option<Self> {
        LCHUV_RE.captures(s).and_then(|captures| {
            Some(
                Lchuv::new(
                    parse_capture(
                        1,
                        Bounds::new(Lchuv::<Srgb>::min_l(), Lch::<Srgb>::max_l()),
                        &captures,
                    )?,
                    parse_capture(
                        2,
                        Bounds::new(Lchuv::<Srgb>::min_chroma(), Lch::<Srgb>::max_chroma()),
                        &captures,
                    )?,
                    parse_capture(3, None, &captures)?,
                )
                .into(),
            )
        })
    }

    fn parse_luv(s: &str) -> Option<Self> {
        LUV_RE.captures(s).and_then(|captures| {
            Some(
                Luv::new(
                    parse_capture(
                        1,
                        Bounds::new(Luv::<Srgb>::min_l(), Lch::<Srgb>::max_l()),
                        &captures,
                    )?,
                    parse_capture(
                        2,
                        Bounds::new(Luv::<Srgb>::min_u(), Luv::<Srgb>::max_u()),
                        &captures,
                    )?,
                    parse_capture(
                        3,
                        Bounds::new(Luv::<Srgb>::min_v(), Luv::<Srgb>::max_v()),
                        &captures,
                    )?,
                )
                .into(),
            )
        })
    }

    fn parse_okhsl(s: &str) -> Option<Self> {
        OKHSL_RE.captures(s).and_then(|captures| {
            Some(
                Okhsl::new(
                    parse_capture(1, None, &captures)?,
                    parse_capture(
                        2,
                        Bounds::new(Okhsl::min_saturation(), Okhsl::max_saturation()),
                        &captures,
                    )?,
                    parse_capture(
                        3,
                        Bounds::new(Okhsl::min_lightness(), Okhsl::max_lightness()),
                        &captures,
                    )?,
                )
                .into(),
            )
        })
    }

    fn parse_okhsv(s: &str) -> Option<Self> {
        OKHSV_RE.captures(s).and_then(|captures| {
            Some(
                Okhsv::new(
                    parse_capture(1, None, &captures)?,
                    parse_capture(
                        2,
                        Bounds::new(Okhsv::min_saturation(), Okhsl::max_saturation()),
                        &captures,
                    )?,
                    parse_capture(
                        3,
                        Bounds::new(Okhsv::min_value(), Okhsv::max_value()),
                        &captures,
                    )?,
                )
                .into(),
            )
        })
    }

    fn parse_okhwb(s: &str) -> Option<Self> {
        OKHWB_RE.captures(s).and_then(|captures| {
            Some(
                Okhwb::new(
                    parse_capture(1, None, &captures)?,
                    parse_capture(
                        2,
                        Bounds::new(Okhwb::min_whiteness(), Okhwb::max_whiteness()),
                        &captures,
                    )?,
                    parse_capture(
                        3,
                        Bounds::new(Okhwb::min_blackness(), Okhwb::max_blackness()),
                        &captures,
                    )?,
                )
                .into(),
            )
        })
    }

    fn parse_oklab(s: &str) -> Option<Self> {
        OKLAB_RE.captures(s).and_then(|captures| {
            Some(
                Oklab::new(
                    parse_capture(1, Bounds::new(Oklab::min_l(), Oklab::max_l()), &captures)?,
                    parse_capture(2, None, &captures)?,
                    parse_capture(3, None, &captures)?,
                )
                .into(),
            )
        })
    }

    fn parse_oklch(s: &str) -> Option<Self> {
        OKLCH_RE.captures(s).and_then(|captures| {
            Some(
                Oklch::new(
                    parse_capture(1, Bounds::new(Oklch::min_l(), Oklch::max_l()), &captures)?,
                    parse_capture(2, None, &captures)?,
                    parse_capture(3, None, &captures)?,
                )
                .into(),
            )
        })
    }

    fn parse_xyz(s: &str) -> Option<Self> {
        XYZ_RE.captures(s).and_then(|captures| {
            Some(
                Xyz::new(
                    parse_capture(
                        1,
                        Bounds::new(Xyz::<D50>::min_x(), Xyz::<D50>::max_x()),
                        &captures,
                    )?,
                    parse_capture(
                        2,
                        Bounds::new(Xyz::<D50>::min_y(), Xyz::<D50>::max_y()),
                        &captures,
                    )?,
                    parse_capture(
                        3,
                        Bounds::new(Xyz::<D50>::min_z(), Xyz::<D50>::max_z()),
                        &captures,
                    )?,
                )
                .into(),
            )
        })
    }

    fn parse_yxy(s: &str) -> Option<Self> {
        YXY_RE.captures(s).and_then(|captures| {
            Some(
                Yxy::new(
                    parse_capture(
                        1,
                        Bounds::new(Yxy::<D50>::min_x(), Yxy::<D50>::max_x()),
                        &captures,
                    )?,
                    parse_capture(
                        2,
                        Bounds::new(Yxy::<D50>::min_y(), Yxy::<D50>::max_y()),
                        &captures,
                    )?,
                    parse_capture(
                        3,
                        Bounds::new(Yxy::<D50>::min_luma(), Yxy::<D50>::max_luma()),
                        &captures,
                    )?,
                )
                .into(),
            )
        })
    }

    fn parse_named(s: &str) -> Option<Self> {
        match s
            .to_ascii_lowercase()
            .replace("-", "")
            .replace("_", "")
            .trim()
        {
            "" | "reset" | "ansireset" => Some(Self::Reset),
            "ansiblack" => Some(Self::Black),
            "ansired" => Some(Self::Red),
            "ansigreen" => Some(Self::Green),
            "ansiyellow" => Some(Self::Green),
            "ansiblue" => Some(Self::Blue),
            "ansimagenta" => Some(Self::Magenta),
            "ansicyan" => Some(Self::Cyan),
            "ansigray" | "ansigrey" => Some(Self::Gray),
            "ansidarkgray" | "ansidarkgrey" => Some(Self::DarkGray),
            "ansiwhite" => Some(Self::White),
            "ansilightred" => Some(Self::LightRed),
            "ansilightgreen" => Some(Self::LightGreen),
            "ansilightyellow" => Some(Self::LightYellow),
            "ansilightblue" => Some(Self::LightBlue),
            "ansilightmagenta" => Some(Self::LightMagenta),
            "ansilightcyan" => Some(Self::LightCyan),
            s => ::palette::named::from_str(s).map(Into::into),
        }
    }
}

#[cfg(feature = "serde")]
pub fn deserialize_color<'de, D>(deser: D) -> Result<Color, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let res = <String as serde::Deserialize>::deserialize(deser)?;
    let color = res.parse().map_err(D::Error::custom)?;
    Ok(color)
}

#[derive(Debug)]
pub struct InvalidColor(String);

impl Error for InvalidColor {}

impl Display for InvalidColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid color: {}", self.0)
    }
}

impl FromStr for Color {
    type Err = InvalidColor;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Ok(val) = s.parse::<u8>() {
            return Ok(Self::Indexed(val));
        }

        if let Some(val) = Self::parse_named(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_hex(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_rgb(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_hsl(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_hsv(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_hsluv(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_hwb(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_lab(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_lch(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_lchuv(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_luv(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_okhsl(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_okhsv(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_okhwb(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_oklab(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_oklch(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_xyz(s) {
            return Ok(val);
        }
        if let Some(val) = Self::parse_yxy(s) {
            return Ok(val);
        }
        Err(InvalidColor(s.to_string()))
    }
}
