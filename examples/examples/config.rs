use std::process::ExitCode;

use rooibos::config::watch_config::backend::schematic::AppConfig;
use rooibos::config::watch_config::schematic::Config;
use rooibos::config::watch_config::{ConfigDir, ConfigSettings};
use rooibos::config::{provide_config, use_config};
use rooibos::reactive::dom::layout::{Borders, borders};
use rooibos::reactive::dom::{Render, text};
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::{col, height, margin, max_width, padding, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::tui::style::Stylize;
use schematic::Format;

#[derive(Config, PartialEq, Eq, Clone, Debug)]
struct AppConfigExample {
    pub number: usize,
    pub string: String,
    pub boolean: bool,
    pub array: Vec<String>,
    pub optional: Option<String>,
}

#[rooibos::main]
async fn main() -> Result<ExitCode, RuntimeError> {
    let config = AppConfig::<AppConfigExample>::new(ConfigSettings::new(
        ConfigDir::Custom("./.config".into()),
        Format::Yaml,
        "config.yml".to_owned(),
    ));
    provide_config(config);

    Runtime::initialize(DefaultBackend::auto()).run(app).await
}

fn app() -> impl Render {
    let config = use_config::<AppConfigExample>();
    col![
        props(padding!(1)),
        wgt!(
            props(margin!(1), height!(1)),
            text!("Update ./.config/config.yml and the changes will render live")
                .bold()
                .cyan()
        ),
        col![
            props(padding!(1), max_width!(150), borders(Borders::all())),
            wgt!(format!("{:?}", config.get().map(|c| c.new)))
        ]
    ]
}
