use std::io::{self};

use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use rooibos_util::{EmbedOrPath, theme_selector};

use crate::{parse_theme_css, read_themes_from_path};

pub fn open(path: &EmbedOrPath) -> io::Result<()> {
    let mut theme_files = read_themes_from_path(path);
    let selection = theme_selector(&theme_files)?;
    let colors = parse_theme_css(&mut theme_files[selection].file)?;

    let formatted: Vec<String> = colors
        .into_iter()
        .filter_map(|line| {
            let color_num = "500";
            if line.variant == color_num {
                let hex = line.color.to_hex_fg();
                Some(format_color(&line.group, &hex, color_num))
            } else {
                None
            }
        })
        .collect();
    open::that(format!(
        "https://www.tints.dev/palette/v1:{}",
        BASE64_URL_SAFE_NO_PAD.encode(formatted.join("~"))
    ))
    .unwrap();
    Ok(())
}

fn format_color(name: &str, hex: &str, number: &str) -> String {
    format!(
        "{}|{}|{number}|p|0|0|0|100|a",
        name.replace("_", "-"),
        &hex[1..]
    )
}
