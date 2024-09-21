use std::error::Error;

use rooibos::config::watch_config::backend::schematic::AppConfig;
use rooibos::config::watch_config::schematic::Config;
use rooibos::config::watch_config::{ConfigDir, ConfigSettings};
use rooibos::config::{provide_config, use_config};
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::{Render, mount, text, wgt};
use rooibos::runtime::Runtime;
use rooibos::terminal::crossterm::CrosstermBackend;
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

    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let config = use_config::<AppConfigExample>();
    wgt!(text!(format!("{:?}", config.get().new)))
}
