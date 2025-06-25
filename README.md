# Minichess

Minichess is a simplified chess engine and UI written in Rust. It allows you to play a smaller variant of chess against a basic AI or another human, with a focus on learning, experimentation, and fun.

## Features
- Play minichess against a simple AI or another player
- Command-line user interface
- Stockfish integration for stronger AI play (optional)
- Modular code structure for easy extension

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable recommended)
- (Optional) [Stockfish](https://stockfishchess.org/download/) binary for advanced AI

### Build and Run

Clone the repository and build the project:

```bash
cargo build --release
```

Run the game:

```bash
cargo run --release
```

### Using Stockfish
To enable Stockfish integration, ensure the Stockfish binary is available in your PATH or specify its location in the configuration (see `src/stockfish.rs`).

## Project Structure
- `src/main.rs` — Entry point
- `src/chess_game.rs` — Core minichess logic
- `src/ui.rs` — Command-line interface
- `src/stockfish.rs` — Stockfish engine integration

## Contributing
Pull requests and suggestions are welcome! Please open an issue to discuss any major changes.

## License
This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
