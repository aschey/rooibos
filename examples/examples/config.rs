use std::error::Error;

use rooibos::config::watch_config::backend::schematic::AppConfig;
use rooibos::config::watch_config::schematic::Config;
use rooibos::config::watch_config::{ConfigDir, ConfigSettings};
use rooibos::config::{provide_config, use_config};
use rooibos::dom::{text, widget_ref, Render};
use rooibos::reactive::traits::Get;
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::Runtime;
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
async fn main() -> Result<(), Box<dyn Error>> {
    let config = AppConfig::<AppConfigExample>::new(ConfigSettings::new(
        ConfigDir::Custom("./.config".into()),
        Format::Yaml,
        "config.yml".to_owned(),
    ));
    provide_config(config);

    let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let config = use_config::<AppConfigExample>();
    widget_ref!(text!(format!("{:?}", config.get().new)))
}
