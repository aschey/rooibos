use std::ffi::OsStr;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;
use fs::File;
use fs_err as fs;
use include_dir::{Dir, include_dir};
use rooibos_theme::{Color, NamedColor};

pub static THEMES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/themes");

#[derive(Clone)]
pub enum EmbedOrPath {
    Embed,
    Path(PathBuf),
}

impl EmbedOrPath {
    pub fn from_optional_path(path: Option<PathBuf>) -> Self {
        match path {
            Some(path) => EmbedOrPath::Path(path),
            None => EmbedOrPath::Embed,
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            Self::Embed => false,
            Self::Path(path) => path.is_file(),
        }
    }

    pub fn extension(&self) -> Option<&OsStr> {
        match self {
            Self::Embed => None,
            Self::Path(p) => p.extension(),
        }
    }

    pub fn as_file(&self) -> Option<EmbedOrFile> {
        match self {
            Self::Embed => None,
            Self::Path(path) => Some(EmbedOrFile::File(fs::File::open(path).unwrap())),
        }
    }

    pub fn read_css(&self) -> Vec<ThemeFile> {
        let mut files: Vec<_> = match self {
            Self::Embed => THEMES_DIR
                .entries()
                .iter()
                .map(|e| ThemeFile::from_file(EmbedOrFile::Embed(e.as_file().unwrap().clone())))
                .collect(),
            Self::Path(path) => fs::read_dir(path)
                .unwrap()
                .filter_map(|f| f.ok().map(|f| f.path()))
                .filter(|f| is_css_file(&EmbedOrPath::Path(f.clone())))
                .map(|f| ThemeFile::from_file(EmbedOrFile::File(File::open(f).unwrap())))
                .collect(),
        };
        // Sort to ensure consistent ordering in output
        files.sort_by(|a, b| a.name.cmp(&b.name));
        files
    }
}

pub enum EmbedOrFile {
    Embed(include_dir::File<'static>),
    File(fs::File),
}

impl EmbedOrFile {
    fn path(&self) -> &Path {
        match self {
            Self::Embed(f) => f.path(),
            Self::File(f) => f.path(),
        }
    }

    fn read_to_string(&mut self) -> io::Result<String> {
        match self {
            Self::Embed(f) => Ok(f.contents_utf8().unwrap_or_default().to_string()),
            Self::File(f) => {
                let mut s = String::new();
                f.read_to_string(&mut s)?;
                Ok(s)
            }
        }
    }
}

pub struct ThemeFile {
    pub file: EmbedOrFile,
    pub name: String,
}

impl ThemeFile {
    fn from_file(file: EmbedOrFile) -> Self {
        Self {
            name: file
                .path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
                .replace(".css", ""),
            file,
        }
    }
}

pub fn read_themes_from_path(path: &EmbedOrPath) -> Vec<ThemeFile> {
    if is_css_file(path)
        && let Some(file) = path.as_file()
    {
        vec![ThemeFile::from_file(file)]
    } else {
        path.read_css()
    }
}

fn is_css_file(path: &EmbedOrPath) -> bool {
    path.is_file() && path.extension() == Some(OsStr::new("css"))
}

pub fn parse_theme_css(file: &mut EmbedOrFile) -> io::Result<Vec<NamedColor<'static>>> {
    let contents = file.read_to_string()?;
    let lines = contents
        .split("\n")
        .filter_map(|l| {
            let l = l.trim();
            if !l.starts_with("--") {
                return None;
            }
            Some(l)
        })
        .map(|line| {
            let line = line.to_ascii_lowercase();
            let parts: Vec<_> = line.split(": ").collect();
            let [name, val] = parts.as_slice() else {
                panic!("invalid format");
            };
            let name = name
                .replacen("--", "", 1)
                .replace("-", "_")
                .replacen("color_", "", 1);
            let color: Color = val.parse().unwrap();
            let group = name.rsplitn(2, "_").last().unwrap();

            let variant = name.rsplit("_").next().unwrap();

            NamedColor {
                variant: variant.to_ascii_lowercase().into(),
                group: group.to_ascii_lowercase().into(),
                color,
            }
        });
    Ok(lines.collect())
}

pub fn theme_selector(theme_files: &[ThemeFile]) -> io::Result<usize> {
    match theme_files.len() {
        0 => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no themes found",
        )),
        1 => Ok(0),
        _ => Ok(Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select theme")
            .default(0)
            .items(theme_files.iter().map(|t| &t.name))
            .interact()
            .unwrap()),
    }
}
