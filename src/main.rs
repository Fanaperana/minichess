mod chess_game;
mod stockfish;
mod ui;

use anyhow::Result;
use chess_game::ChessGame;
use clap::{Arg, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("Chess CLI")
        .version("1.0")
        .author("Your Name")
        .about("A CLI chess game using Stockfish")
        .arg(
            Arg::new("stockfish-path")
                .long("stockfish-path")
                .value_name("PATH")
                .help("Path to Stockfish executable")
                .default_value("stockfish"), // Adjust this path as needed to the stockfish binary
        )
        .arg(
            Arg::new("difficulty")
                .long("difficulty")
                .value_name("LEVEL")
                .help("Stockfish difficulty level (1-20)")
                .default_value("5"),
        )
        .get_matches();

    let stockfish_path = matches.get_one::<String>("stockfish-path").unwrap();
    let difficulty: u8 = matches
        .get_one::<String>("difficulty")
        .unwrap()
        .parse()
        .unwrap_or(5);

    println!("Starting chess game...");
    println!("Stockfish path: {}", stockfish_path);
    println!("Difficulty: {}", difficulty);
    println!("Press 'q' to quit, 'h' for help");
    println!();

    let mut game = ChessGame::new(stockfish_path, difficulty).await?;
    game.run().await?;

    Ok(())
}
