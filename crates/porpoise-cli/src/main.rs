use clap::Parser;

mod app;
mod commands;
mod completion;
mod config;
mod output;

#[tokio::main]
async fn main() {
    let cli = app::Cli::parse();

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
