mod api;
mod cli;
mod config;
mod constants;
mod models;
mod output;
mod utils;

use clap::Parser;
use cli::Cli;

fn main() {
    let _cli = Cli::parse();
    println!("Hello from tickrs!");
}
