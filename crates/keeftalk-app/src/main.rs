//! Keeftalk Desktop executable entry point.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use keeftalk_core::KeeftalkCore;
use keeftalk_platform::AppPaths;
use keeftalk_settings::{AppConfig, AppMode};
use tracing_subscriber::EnvFilter;

struct CliArgs {
    mode_override: Option<AppMode>,
    config_path: Option<PathBuf>,
    headless_check: bool,
    show_version: bool,
}

fn parse_args() -> anyhow::Result<CliArgs> {
    let mut args = std::env::args().skip(1);
    let mut parsed = CliArgs {
        mode_override: None,
        config_path: None,
        headless_check: false,
        show_version: false,
    };

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--mode" => {
                let val = args.next().context("--mode requires mock|offline|cloud")?;
                parsed.mode_override = Some(AppMode::from_env_str(&val)?);
            }
            "--config" => {
                let val = args.next().context("--config requires a file path")?;
                parsed.config_path = Some(PathBuf::from(val));
            }
            "--headless-check" => parsed.headless_check = true,
            "--version" | "-V" => parsed.show_version = true,
            other => anyhow::bail!("unknown argument '{other}'"),
        }
    }
    Ok(parsed)
}

fn init_logging(default_level: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{default_level},keeftalk=debug")));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = parse_args()?;
    if cli.show_version {
        println!("keeftalk-desktop {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let paths = AppPaths::resolve().context("failed to resolve OS application directories")?;
    let cfg_path = cli.config_path.or_else(|| {
        let user_cfg = paths.config_file_path();
        user_cfg.exists().then_some(user_cfg)
    });

    let mut config = AppConfig::load(cfg_path.as_deref()).context("failed to load configuration")?;
    if let Some(mode) = cli.mode_override {
        config.app.mode = mode;
        config.validate()?;
    }

    init_logging(&config.logging.level);

    let core = Arc::new(
        KeeftalkCore::bootstrap(config, paths).context("failed to initialize Keeftalk core")?,
    );
    keeftalk_ui::launch_shell(core, cli.headless_check).context("UI shell exited with error")?;

    Ok(())
}
