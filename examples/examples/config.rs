use std::process::ExitCode;

use confique::yaml;
use rooibos::config::watch_config::ConfigDir;
use rooibos::config::watch_config::backend::confique::{AppConfig, ConfigSettings, ConfiqueConfig};
use rooibos::config::watch_config::confique::Config;
use rooibos::config::{provide_config, use_config};
use rooibos::reactive::dom::layout::{Borders, borders, height, margin, max_width, padding};
use rooibos::reactive::dom::{Render, span};
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::{col, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::theme::Stylize;

#[derive(Config, PartialEq, Eq, Clone, Debug)]
#[config(layer_attr(derive(Clone)))]
struct AppConfigExample {
    pub number: usize,
    pub string: String,
    pub boolean: bool,
    pub array: Vec<String>,
    pub optional: Option<String>,
}

#[derive(Clone)]
struct ConfigBuilder;

impl ConfiqueConfig<AppConfigExample> for ConfigBuilder {
    fn builder(&self, path: &std::path::Path) -> confique::Builder<AppConfigExample> {
        AppConfigExample::builder().file(path)
    }

    fn template(&self) -> String {
        yaml::template::<AppConfigExample>(yaml::FormatOptions::default())
    }
}

#[rooibos::main]
async fn main() -> Result<ExitCode, RuntimeError> {
    let config = AppConfig::new(ConfigSettings::new(
        ConfigDir::Custom("./.config".into()),
        "config.yml".to_owned(),
        ConfigBuilder,
    ))
    .unwrap();
    provide_config(config);

    Runtime::initialize(DefaultBackend::auto().await?)
        .run(|_| app())
        .await
}

fn app() -> impl Render {
    let config = use_config::<AppConfig<_, ConfigBuilder>>();
    col![
        style(padding(1)),
        wgt!(
            style(margin(1), height(1)),
            "Update ./.config/config.yml and the changes will render live"
                .bold()
                .cyan()
        ),
        col![
            style(padding(1), max_width(150), borders(Borders::all())),
            wgt!(span!("{:?}", config.get().map(|c| c.new)))
        ]
    ]
}
