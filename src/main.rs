use clap::{Args, Parser, Subcommand};
use std::env;

mod cache;
mod commands;
mod converter;
mod helpers;
mod sources;

use commands::{convert, interactive, list};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert an amount from one currency to another
    Convert(ConvertArgs),
    /// List available currencies
    List,
    /// Enter interactive mode
    Interactive,
}

#[derive(Args)]
struct ConvertArgs {
    /// The base currency, e.g. USD
    base: String,
    /// The target currency, e.g. EUR
    target: String,
    /// The amount to convert
    amount: f64,

    /// The precision to use when displaying the result
    #[arg(short, long, default_value_t = 2)]
    precision: usize,

    /// The duration to cache the exchange rate for
    #[arg(short, long, default_value_t = 300)]
    cache_duration: u64,
}

#[tokio::main]
async fn main() {
    env::var("CURRENCY_API_KEY").expect("CURRENCY_API_KEY is not set");
    let cli = Cli::parse();

    match &cli.command {
        Commands::Convert(args) => convert(args).await,
        Commands::List => list().await,
        Commands::Interactive => interactive().await,
    }
}
