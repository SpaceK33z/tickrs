mod api;
mod cli;
mod config;
mod models;

use clap::Parser;
use cli::Cli;

fn main() {
    let _cli = Cli::parse();
    println!("Hello from tickrs!");
}
