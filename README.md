# Minichess

Minichess is a simplified chess engine and UI written in Rust. It allows you to play a smaller variant of chess against a basic AI or another human, with a focus on learning, experimentation, and fun.

## Features
- Play minichess against a simple AI or another player
- Command-line user interface
- Stockfish integration for stronger AI play (required for advanced AI)
- Modular code structure for easy extension
- **FEN support:** Load and display board positions using Forsyth-Edwards Notation
- **SAN support:** Input and display moves using Standard Algebraic Notation
- **Perspective view:** Play from either White or Black's perspective
- **Game history:** Display move history in SAN format during gameplay

## Example Board Output
When you play, the board is displayed in the terminal like this:

```
White's turn to move
> f1a5
Invalid move: Move is not legal in current position. Try again.
> f1b5        

    a   b   c   d   e   f   g   h
  ┌───┬───┬───┬───┬───┬───┬───┬───┐
8 │ ♜ │ ♞ │ ♝ │ ♛ │ ♚ │ ♝ │ ♞ │ ♜ │ 8
  ├───┼───┼───┼───┼───┼───┼───┼───┤
7 │ ♟ │ ♟ │ ♟ │   │ ♟ │ ♟ │   │ ♟ │ 7
  ├───┼───┼───┼───┼───┼───┼───┼───┤
6 │   │   │   │ ♟ │   │   │ ♟ │   │ 6
  ├───┼───┼───┼───┼───┼───┼───┼───┤
5 │   │ ♗ │   │   │   │   │   │   │ 5
  ├───┼───┼───┼───┼───┼───┼───┼───┤
4 │   │   │   │   │ ♙ │   │   │   │ 4
  ├───┼───┼───┼───┼───┼───┼───┼───┤
3 │   │   │   │   │   │ ♘ │   │   │ 3
  ├───┼───┼───┼───┼───┼───┼───┼───┤
2 │ ♙ │ ♙ │ ♙ │ ♙ │   │ ♙ │ ♙ │ ♙ │ 2
  ├───┼───┼───┼───┼───┼───┼───┼───┤
1 │ ♖ │ ♘ │ ♗ │ ♕ │ ♔ │   │   │ ♖ │ 1
  └───┴───┴───┴───┴───┴───┴───┴───┘
    a   b   c   d   e   f   g   h

Black's turn to move
⚠️  Black is in check!

Computer is thinking...
Computer plays: b8d7 (n b8->d7)

    a   b   c   d   e   f   g   h
  ┌───┬───┬───┬───┬───┬───┬───┬───┐
8 │ ♜ │   │ ♝ │ ♛ │ ♚ │ ♝ │ ♞ │ ♜ │ 8
  ├───┼───┼───┼───┼───┼───┼───┼───┤
7 │ ♟ │ ♟ │ ♟ │ ♞ │ ♟ │ ♟ │   │ ♟ │ 7
  ├───┼───┼───┼───┼───┼───┼───┼───┤
6 │   │   │   │ ♟ │   │   │ ♟ │   │ 6
  ├───┼───┼───┼───┼───┼───┼───┼───┤
5 │   │ ♗ │   │   │   │   │   │   │ 5
  ├───┼───┼───┼───┼───┼───┼───┼───┤
4 │   │   │   │   │ ♙ │   │   │   │ 4
  ├───┼───┼───┼───┼───┼───┼───┼───┤
3 │   │   │   │   │   │ ♘ │   │   │ 3
  ├───┼───┼───┼───┼───┼───┼───┼───┤
2 │ ♙ │ ♙ │ ♙ │ ♙ │   │ ♙ │ ♙ │ ♙ │ 2
  ├───┼───┼───┼───┼───┼───┼───┼───┤
1 │ ♖ │ ♘ │ ♗ │ ♕ │ ♔ │   │   │ ♖ │ 1
  └───┴───┴───┴───┴───┴───┴───┴───┘
    a   b   c   d   e   f   g   h

White's turn to move

Your turn! Enter a move (e.g., 'e2e4') or 'h' for help:
> history

=== Move History (Algebraic Notation) ===
1. e4 g6
2. Nf3 d6
3. Bb5+ Nd7

Detailed coordinate history:
1. White (You): P e2->e4
2. Black (Computer): p g7->g6
3. White (You): N g1->f3
4. Black (Computer): p d7->d6
5. White (You): B f1->b5
6. Black (Computer): n b8->d7
==========================================
```
- Uppercase = White pieces, lowercase = Black pieces
- K/k = King, Q/q = Queen, R/r = Rook, N/n = Knight, B/b = Bishop, P/p = Pawn

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable recommended)
- [Stockfish](https://stockfishchess.org/download/) chess engine (required for advanced AI)
  - You must have the Stockfish binary available in your PATH, or specify its location with the `--stockfish-path` argument.

### Build and Run

Clone the repository and build the project:

```bash
cargo build --release
```

Run the game:

```bash
cargo run --release
```

#### Using Stockfish from a Custom Path
If Stockfish is not in your PATH, you can specify its location:

```bash
cargo run --quiet -- --stockfish-path /path/to/your/stockfish
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
