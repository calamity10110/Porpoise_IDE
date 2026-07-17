use clap::Parser;

mod app;
mod commands;
mod completion;
mod config;
mod daemon;
mod output;

#[tokio::main]
async fn main() {
    let cli = app::Cli::parse();

    // Initialize tracing based on verbosity flags
    let filter = if cli.debug {
        "debug"
    } else if cli.verbose {
        "porpoise=debug,info"
    } else {
        "warn"
    };
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(filter));
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .init();

    tracing::debug!("CLI started");

    let format = if cli.json {
        output::OutputFormat::JsonPretty
    } else {
        output::OutputFormat::Plain
    };

    let result = commands::handle_command(cli.command, &format).await;

    match result {
        Ok(output) => {
            println!("{output}");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
