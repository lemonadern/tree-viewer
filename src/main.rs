mod cli;

use cli::Cli;
use clap::Parser;

fn main() {
    let cli = Cli::parse();
    
    println!("SQLファイル: {:?}", cli.sql_file);
    println!("深さ指定: {:?}", cli.depth);
}
