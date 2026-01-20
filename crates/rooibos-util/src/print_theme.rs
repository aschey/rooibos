use std::borrow::Cow;
use std::io::{self, stdout};

use anstyle_crossterm::to_crossterm;
use crossterm::style::Stylize;
use indexmap::{IndexMap, IndexSet};
use rooibos_theme::profile::DetectorSettings;
use rooibos_theme::{Color, ColorPalette, NamedColor, SetTheme, Style, TermProfile};
use rooibos_util::{EmbedOrPath, theme_selector};

use crate::{parse_theme_css, read_themes_from_path};

struct PrintableTheme {
    column_width: usize,
    sections: Vec<Section>,
}

struct Row {
    label: String,
    cells: Vec<Cell>,
}

impl Row {
    fn new(variant: &str, colors: Vec<&NamedColor<'_>>) -> Self {
        Row {
            label: variant.to_string(),
            cells: colors.iter().map(|c| Cell::new(&c.color)).collect(),
        }
    }
}

struct Cell {
    fg: Color,
    bg: Color,
}

impl Cell {
    fn new(color: &Color) -> Self {
        let rgb_color = color.to_rgb_bg();
        let color_luminance = color.luminance_bg();
        let fg = if color_luminance < 0.179 {
            Color::Rgb(220, 220, 220)
        } else {
            Color::Rgb(20, 20, 20)
        };
        Cell {
            fg,
            bg: rgb_color.into(),
        }
    }
}

struct Section {
    headers: Vec<String>,
    rows: Vec<Row>,
}

impl Section {
    fn new(header_group: &[&Cow<'_, str>], colors: &[NamedColor]) -> Self {
        let headers = header_group.iter().map(|h| h.to_string()).collect();
        let grouped_colors = group(
            colors
                .iter()
                .filter(|c| header_group.contains(&&c.group))
                .map(|c| (&c.variant, c)),
        );
        let rows = grouped_colors
            .into_iter()
            .map(|(variant, colors)| Row::new(variant, colors))
            .collect();

        Self { headers, rows }
    }
}

impl PrintableTheme {
    fn new<'a, I>(colors: I, width: usize) -> Self
    where
        I: Into<Vec<NamedColor<'a>>>,
    {
        let colors = colors.into();
        let headers: IndexSet<&Cow<'_, str>> = colors.iter().map(|c| &c.group).collect();
        let headers: Vec<_> = headers.into_iter().collect();

        let max_header_len = (headers.iter().map(|h| h.len()).max().unwrap() + 2).max(9);
        let columns_per_section = width / (max_header_len + 1);

        let sections = headers
            .chunks(columns_per_section)
            .map(|header_group| Section::new(header_group, &colors))
            .collect();
        PrintableTheme {
            column_width: max_header_len,
            sections,
        }
    }
}

pub fn print(theme_dir: &EmbedOrPath) -> io::Result<()> {
    ColorPalette::detect().set();
    TermProfile::detect(&stdout(), DetectorSettings::new()).set();
    let columns = crossterm::terminal::window_size().unwrap().columns;

    let mut theme_files = read_themes_from_path(theme_dir);
    let selection = theme_selector(&theme_files)?;
    let colors = parse_theme_css(&mut theme_files[selection].file)?;

    let theme = PrintableTheme::new(colors, columns as usize);
    let column_width = theme.column_width;
    println!();
    for section in theme.sections {
        for header in section.headers {
            let formatted_header = format!("{header:^column_width$}");
            print!("{} ", formatted_header.bold());
        }
        println!();
        for row in section.rows {
            let label = row.label;
            for cell in row.cells {
                let an: anstyle::Style = Style::new().fg(cell.fg).bg(cell.bg).into();
                let style = to_crossterm(an);
                print!("{} ", style.apply(format!("{label:^column_width$}")));
            }
            println!();
        }
        println!();
    }
    Ok(())
}

fn group<K: Eq + std::hash::Hash + Ord, V, I: Iterator<Item = (K, V)>>(
    iter: I,
) -> IndexMap<K, Vec<V>> {
    iter.fold(IndexMap::new(), |mut map, (k, v)| {
        map.entry(k).or_insert_with(|| Vec::new()).push(v);
        map
    })
}
