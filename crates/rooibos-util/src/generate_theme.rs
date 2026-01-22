use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

use convert_case::{Case, Casing};
use fs::File;
use fs_err as fs;
use indexmap::IndexMap;
use rooibos_theme::Color;
use rooibos_util::{EmbedOrFile, EmbedOrPath};

use crate::{parse_theme_css, read_themes_from_path};

pub fn generate(
    cargo_root_dir: &Path,
    theme_dir: &EmbedOrPath,
    palette_dir: &Path,
) -> io::Result<()> {
    // palettes created with https://www.tints.dev
    //  let palette_dir = Path::new("../tui-theme/src/palette");
    fs::remove_dir_all(palette_dir)?;
    fs::create_dir_all(palette_dir)?;
    let mut mod_file = File::create(palette_dir.join("mod.rs"))?;

    let theme_files = read_themes_from_path(theme_dir);
    let palette_dir_str = palette_dir.as_os_str().to_string_lossy();
    for mut theme in theme_files {
        read_theme(&theme.name, &palette_dir_str, &mut theme.file).unwrap();
        let mod_name = theme.name.to_case(Case::Snake);
        writeln!(mod_file, "mod {mod_name};")?;
        writeln!(mod_file, "pub use {mod_name}::*;")?;
    }

    Command::new("cargo")
        .args(["+nightly", "fmt"])
        .current_dir(cargo_root_dir)
        .output()
        .unwrap();

    Ok(())
}

fn read_theme(name: &str, palette_dir: &str, path: &mut EmbedOrFile) -> io::Result<()> {
    let theme = parse_theme_css(path)?;
    let mut out = File::create(format!("{palette_dir}/{}.rs", name.to_case(Case::Snake)))?;
    let name_caps = name.to_case(Case::UpperCamel);
    writeln!(out, "use crate::Color;")?;
    writeln!(out, "use crate::NamedColor;")?;
    writeln!(out, "use std::borrow::Cow;")?;
    writeln!(out, "use crate::ThemeArray;\n")?;
    writeln!(out, "// Auto-generated file. Do not edit.\n")?;
    writeln!(out, "pub struct {name_caps} {{}}\n")?;
    writeln!(out, "impl {name_caps} {{")?;
    writeln!(out, "pub const NAME: &str = \"{name}\";\n")?;
    let mut color_groups: IndexMap<String, Vec<String>> = IndexMap::new();
    let mut all_colors: Vec<String> = Vec::new();
    for named_color in theme {
        let color = named_color.color;
        let group = named_color.group.to_string();
        let group_upper = group.to_ascii_uppercase();
        let variant = named_color.variant.to_string();
        let color_const = format!("{group}_{variant}").to_ascii_uppercase();

        if let Some(colors) = color_groups.get_mut(&group_upper) {
            colors.push(color_const.clone());
        } else {
            color_groups.insert(group_upper.clone(), vec![color_const.clone()]);
        }
        all_colors.push(format!(
            "NamedColor {{ variant: Cow::Borrowed(\"{variant}\"), group: \
             Cow::Borrowed(\"{group}\"), color: Self::{color_const} }}",
        ));

        let Color::Rgb(r, g, b) = color else {
            panic!("invalid color");
        };
        writeln!(
            out,
            "    #[allow(clippy::excessive_precision, clippy::approx_constant)]"
        )?;
        writeln!(out, "{}", generate_const(&color_const, r, g, b))?;
    }

    for (color_group, colors) in &color_groups {
        let color_array_vals: Vec<_> = colors.iter().map(|c| format!("Self::{c}")).collect();
        writeln!(
            out,
            "    pub const {color_group}: ThemeArray<{}> = ThemeArray([{}]);\n",
            colors.len(),
            color_array_vals.join(",")
        )?;
    }
    writeln!(
        out,
        "  pub const ALL_COLORS: [NamedColor<'_>;{}] = [{}];",
        all_colors.len(),
        all_colors.join(",")
    )?;
    writeln!(out, "}}")?;

    Ok(())
}

fn generate_const(name: &str, r: u8, g: u8, b: u8) -> String {
    format!("    pub const {name}: Color = Color::Rgb({r}, {g}, {b});\n")
}
