mod api;
mod cli;
mod config;
mod models;
mod output;

use clap::Parser;
use cli::Cli;

fn main() {
    let _cli = Cli::parse();
    println!("Hello from tickrs!");
}
