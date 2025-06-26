use crate::stockfish::StockfishEngine;
use crate::ui::{display_board_for_player, get_user_input, print_help};
use anyhow::{Result, anyhow};
use chess::{ChessMove, Color, Game, MoveGen, Piece, Square};
use std::str::FromStr;

pub struct ChessGame {
    game: Game,
    engine: StockfishEngine,
    player_color: Color,
    move_history: Vec<(ChessMove, String, String)>, // (move, description, detailed_description)
    game_states: Vec<Game>,                         // Stack of game states for undo/redo
    current_state_index: usize,                     // Current position in the game_states stack
}

impl ChessGame {
    pub async fn new(stockfish_path: &str, difficulty: u8) -> Result<Self> {
        let mut engine = StockfishEngine::new(stockfish_path).await?;
        engine.set_difficulty(difficulty).await?;

        // Ask player for color preference
        println!("Choose your color:");
        println!("1. White (you move first)");
        println!("2. Black (computer moves first)");
        print!("Enter choice (1 or 2): ");

        let choice = get_user_input()?;
        let player_color = match choice.trim() {
            "1" => Color::White,
            "2" => Color::Black,
            _ => {
                println!("Invalid choice, defaulting to White");
                Color::White
            }
        };

        let game = Game::new();

        Ok(ChessGame {
            game: game.clone(),
            engine,
            player_color,
            move_history: Vec::new(),
            game_states: vec![game], // Start with initial position
            current_state_index: 0,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("\nGame started! You are playing as {:?}", self.player_color);
        display_board_for_player(&self.game.current_position(), self.player_color);

        // If player is black, let computer make first move
        if self.player_color == Color::Black {
            self.make_computer_move().await?;
        }

        loop {
            if self.game.result().is_some() {
                self.display_game_result();
                break;
            }

            if self.game.current_position().side_to_move() == self.player_color {
                // Player's turn
                match self.handle_player_turn().await? {
                    GameAction::Quit => break,
                    GameAction::Continue => {}
                }
            } else {
                // Computer's turn
                self.make_computer_move().await?;
            }
        }

        Ok(())
    }

    async fn handle_player_turn(&mut self) -> Result<GameAction> {
        println!("\nYour turn! Enter a move (e.g., 'e2e4') or 'h' for help:");

        loop {
            let input = get_user_input()?.trim().to_lowercase();

            match input.as_str() {
                "q" | "quit" => return Ok(GameAction::Quit),
                "h" | "help" => {
                    print_help();
                    continue;
                }
                "moves" => {
                    self.show_legal_moves();
                    continue;
                }
                "history" => {
                    self.show_move_history();
                    continue;
                }
                "show" | "showboard" | "board" => {
                    display_board_for_player(&self.game.current_position(), self.player_color);
                    continue;
                }
                "fen" => {
                    self.show_fen();
                    continue;
                }
                "undo" | "u" => {
                    if self.undo_move() {
                        display_board_for_player(&self.game.current_position(), self.player_color);
                    }
                    continue;
                }
                "redo" | "re" => {
                    if self.redo_move() {
                        display_board_for_player(&self.game.current_position(), self.player_color);
                    }
                    continue;
                }
                _ => {
                    match self.parse_and_make_move(&input) {
                        Ok(_move_made) => {
                            // Add player move to history (describe_move is called inside parse_and_make_move now)
                            display_board_for_player(
                                &self.game.current_position(),
                                self.player_color,
                            );
                            return Ok(GameAction::Continue);
                        }
                        Err(e) => {
                            println!("Invalid move: {}. Try again.", e);
                            continue;
                        }
                    }
                }
            }
        }
    }

    fn parse_and_make_move(&mut self, input: &str) -> Result<ChessMove> {
        // Handle different input formats
        let chess_move = if input.len() == 4 {
            // Standard algebraic notation like "e2e4"
            let from_str = &input[0..2];
            let to_str = &input[2..4];

            let from = Square::from_str(from_str)
                .map_err(|_| anyhow!("Invalid from square: {}", from_str))?;
            let to =
                Square::from_str(to_str).map_err(|_| anyhow!("Invalid to square: {}", to_str))?;

            ChessMove::new(from, to, None)
        } else if input.len() == 5 {
            // Promotion moves like "e7e8q"
            let from_str = &input[0..2];
            let to_str = &input[2..4];
            let promotion_str = &input[4..5];

            let from = Square::from_str(from_str)
                .map_err(|_| anyhow!("Invalid from square: {}", from_str))?;
            let to =
                Square::from_str(to_str).map_err(|_| anyhow!("Invalid to square: {}", to_str))?;

            let promotion = match promotion_str {
                "q" => Some(chess::Piece::Queen),
                "r" => Some(chess::Piece::Rook),
                "b" => Some(chess::Piece::Bishop),
                "n" => Some(chess::Piece::Knight),
                _ => return Err(anyhow!("Invalid promotion piece: {}", promotion_str)),
            };

            ChessMove::new(from, to, promotion)
        } else {
            return Err(anyhow!(
                "Invalid move format. Use format like 'e2e4' or 'e7e8q' for promotions"
            ));
        };

        // Verify the move is legal
        let legal_moves: Vec<ChessMove> =
            MoveGen::new_legal(&self.game.current_position()).collect();

        if !legal_moves.contains(&chess_move) {
            return Err(anyhow!("Move is not legal in current position"));
        }

        // Describe the move BEFORE making it (when we can still see the piece)
        let move_description = self.describe_move(&chess_move, &self.game.current_position());

        // Make the move
        self.game.make_move(chess_move);

        // Save game state for undo/redo
        self.save_game_state();

        // Add to history
        let player_color_str = if self.player_color == Color::White {
            "White"
        } else {
            "Black"
        };
        let detailed_description = format!("{} (You): {}", player_color_str, move_description);
        self.move_history.push((
            chess_move,
            player_color_str.to_string(),
            detailed_description,
        ));

        Ok(chess_move)
    }

    async fn make_computer_move(&mut self) -> Result<()> {
        println!("\nComputer is thinking...");

        let best_move = self
            .engine
            .get_best_move(&self.game.current_position())
            .await?;

        // Describe the move before making it
        let move_description = self.describe_move(&best_move, &self.game.current_position());

        println!("Computer plays: {} ({})", best_move, move_description);

        // Add computer move to history
        let computer_color_str = if self.player_color == Color::White {
            "Black"
        } else {
            "White"
        };
        let detailed_description =
            format!("{} (Computer): {}", computer_color_str, move_description);
        self.move_history.push((
            best_move,
            computer_color_str.to_string(),
            detailed_description,
        ));

        self.game.make_move(best_move);

        // Save game state for undo/redo
        self.save_game_state();

        display_board_for_player(&self.game.current_position(), self.player_color);

        Ok(())
    }

    fn show_legal_moves(&self) {
        let legal_moves: Vec<ChessMove> =
            MoveGen::new_legal(&self.game.current_position()).collect();

        println!("\nLegal moves:");
        for (i, mv) in legal_moves.iter().enumerate() {
            print!("{} ", mv);
            if (i + 1) % 8 == 0 {
                println!();
            }
        }
        println!();
    }

    fn show_move_history(&self) {
        if self.move_history.is_empty() {
            println!("\nNo moves played yet.");
            return;
        }

        println!("\n=== Move History (Algebraic Notation) ===");
        for (i, (chess_move, _player, _detailed)) in self.move_history.iter().enumerate() {
            let move_number = (i / 2) + 1;

            if i % 2 == 0 {
                // White's move (or first player's move)
                let algebraic = self.to_algebraic_notation(chess_move, i);
                print!("{}. {} ", move_number, algebraic);
            } else {
                // Black's move (or second player's move)
                let algebraic = self.to_algebraic_notation(chess_move, i);
                println!("{}", algebraic);
            }
        }

        // If the last move was white's, add a newline
        if self.move_history.len() % 2 == 1 {
            println!();
        }

        println!("\nDetailed coordinate history:");
        for (i, (_chess_move, _player, detailed_description)) in
            self.move_history.iter().enumerate()
        {
            println!("{}. {}", i + 1, detailed_description);
        }
        println!("==========================================\n");
    }

    fn describe_move(&self, chess_move: &ChessMove, board: &chess::Board) -> String {
        let from_square = chess_move.get_source();
        let to_square = chess_move.get_dest();

        // Get the piece that's moving
        let piece = board.piece_on(from_square);
        let piece_color = board.color_on(from_square);

        let piece_fen = match (piece, piece_color) {
            (Some(Piece::King), Some(Color::White)) => "K",
            (Some(Piece::Queen), Some(Color::White)) => "Q",
            (Some(Piece::Rook), Some(Color::White)) => "R",
            (Some(Piece::Bishop), Some(Color::White)) => "B",
            (Some(Piece::Knight), Some(Color::White)) => "N",
            (Some(Piece::Pawn), Some(Color::White)) => "P",
            (Some(Piece::King), Some(Color::Black)) => "k",
            (Some(Piece::Queen), Some(Color::Black)) => "q",
            (Some(Piece::Rook), Some(Color::Black)) => "r",
            (Some(Piece::Bishop), Some(Color::Black)) => "b",
            (Some(Piece::Knight), Some(Color::Black)) => "n",
            (Some(Piece::Pawn), Some(Color::Black)) => "p",
            _ => "?",
        };

        // Check if it's a capture
        let is_capture = board.piece_on(to_square).is_some();
        let captured_piece = if is_capture {
            match board.piece_on(to_square) {
                Some(Piece::Queen) => " captures Queen",
                Some(Piece::Rook) => " captures Rook",
                Some(Piece::Bishop) => " captures Bishop",
                Some(Piece::Knight) => " captures Knight",
                Some(Piece::Pawn) => " captures Pawn",
                _ => " captures piece",
            }
        } else {
            ""
        };

        // Check for promotion
        let promotion = match chess_move.get_promotion() {
            Some(Piece::Queen) => " promotes to Queen",
            Some(Piece::Rook) => " promotes to Rook",
            Some(Piece::Bishop) => " promotes to Bishop",
            Some(Piece::Knight) => " promotes to Knight",
            _ => "",
        };

        // Check for castling
        if piece == Some(Piece::King) {
            let king_start = if piece_color == Some(Color::White) {
                Square::E1
            } else {
                Square::E8
            };
            if from_square == king_start {
                if to_square == Square::G1 || to_square == Square::G8 {
                    return format!("{} castles kingside", piece_fen);
                } else if to_square == Square::C1 || to_square == Square::C8 {
                    return format!("{} castles queenside", piece_fen);
                }
            }
        }

        format!(
            "{} {}->{}{}{}",
            piece_fen, from_square, to_square, captured_piece, promotion
        )
    }

    fn to_algebraic_notation(&self, chess_move: &ChessMove, move_index: usize) -> String {
        // Reconstruct the board state at the time of this move
        let mut temp_game = Game::new();

        // Replay all moves up to (but not including) this move
        for i in 0..move_index {
            temp_game.make_move(self.move_history[i].0);
        }

        let board = temp_game.current_position();
        let from_square = chess_move.get_source();
        let to_square = chess_move.get_dest();

        // Get the piece that's moving
        let piece = board.piece_on(from_square);
        let piece_color = board.color_on(from_square);

        // Check if it's a capture
        let is_capture = board.piece_on(to_square).is_some();

        // Check for castling first
        if piece == Some(Piece::King) {
            let king_start = if piece_color == Some(Color::White) {
                Square::E1
            } else {
                Square::E8
            };
            if from_square == king_start {
                if to_square == Square::G1 || to_square == Square::G8 {
                    return "0-0".to_string();
                } else if to_square == Square::C1 || to_square == Square::C8 {
                    return "0-0-0".to_string();
                }
            }
        }

        let mut notation = String::new();

        // Add piece letter (except for pawns)
        match piece {
            Some(Piece::King) => notation.push('K'),
            Some(Piece::Queen) => notation.push('Q'),
            Some(Piece::Rook) => notation.push('R'),
            Some(Piece::Bishop) => notation.push('B'),
            Some(Piece::Knight) => notation.push('N'),
            Some(Piece::Pawn) => {
                // For pawn captures, include the file
                if is_capture {
                    notation.push(from_square.to_string().chars().next().unwrap());
                }
            }
            None => return chess_move.to_string(), // Fallback
        }

        // Check for disambiguation (if multiple pieces of same type can reach the destination)
        if piece != Some(Piece::Pawn) && piece != Some(Piece::King) {
            let legal_moves: Vec<ChessMove> = MoveGen::new_legal(&board).collect();
            let same_piece_moves: Vec<ChessMove> = legal_moves
                .iter()
                .filter(|m| {
                    m.get_dest() == to_square
                        && board.piece_on(m.get_source()) == piece
                        && m.get_source() != from_square
                })
                .cloned()
                .collect();

            if !same_piece_moves.is_empty() {
                // Need disambiguation
                let from_file = from_square.to_string().chars().next().unwrap();
                let from_rank = from_square.to_string().chars().nth(1).unwrap();

                // Check if file disambiguation is enough
                let same_file = same_piece_moves
                    .iter()
                    .any(|m| m.get_source().to_string().chars().next().unwrap() == from_file);

                if !same_file {
                    notation.push(from_file);
                } else {
                    // Need rank disambiguation
                    notation.push(from_rank);
                }
            }
        }

        // Add capture notation
        if is_capture {
            notation.push('x');
        }

        // Add destination square
        notation.push_str(&to_square.to_string());

        // Add promotion
        if let Some(promotion_piece) = chess_move.get_promotion() {
            notation.push('=');
            match promotion_piece {
                Piece::Queen => notation.push('Q'),
                Piece::Rook => notation.push('R'),
                Piece::Bishop => notation.push('B'),
                Piece::Knight => notation.push('N'),
                _ => {}
            }
        }

        // Check for check or checkmate (we'd need to make the move and see)
        let mut temp_board = board.clone();
        temp_board = temp_board.make_move_new(*chess_move);

        if temp_board.checkers().popcnt() > 0 {
            // It's check, but is it checkmate?
            let legal_after: Vec<ChessMove> = MoveGen::new_legal(&temp_board).collect();
            if legal_after.is_empty() {
                notation.push('#'); // Checkmate
            } else {
                notation.push('+'); // Check
            }
        }

        notation
    }

    fn show_fen(&self) {
        let fen = self.game.current_position().to_string();

        println!("\n=== Current Position FEN ===");
        println!("{}", fen);

        // Break down the FEN for educational purposes
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() >= 6 {
            println!("\nFEN Breakdown:");
            println!("Position:       {}", parts[0]);
            println!(
                "Active color:   {} ({})",
                parts[1],
                if parts[1] == "w" {
                    "White to move"
                } else {
                    "Black to move"
                }
            );
            println!(
                "Castling:       {} ({})",
                parts[2],
                if parts[2] == "-" {
                    "No castling available"
                } else {
                    "K=White kingside, Q=White queenside, k=Black kingside, q=Black queenside"
                }
            );
            println!(
                "En passant:     {} ({})",
                parts[3],
                if parts[3] == "-" {
                    "No en passant target"
                } else {
                    "En passant target square"
                }
            );
            println!(
                "Halfmove clock: {} (moves since last pawn move or capture)",
                parts[4]
            );
            println!(
                "Fullmove:       {} (increments after Black's move)",
                parts[5]
            );
        }
        println!("=============================\n");
    }

    fn save_game_state(&mut self) {
        // Remove any future states if we're in the middle of history
        if self.current_state_index < self.game_states.len() - 1 {
            self.game_states.truncate(self.current_state_index + 1);
        }

        // Add the new state
        self.game_states.push(self.game.clone());
        self.current_state_index = self.game_states.len() - 1;
    }

    fn undo_move(&mut self) -> bool {
        if self.current_state_index == 0 {
            println!("Cannot undo: Already at the beginning of the game.");
            return false;
        }

        // Check if we're trying to undo into the middle of a computer move sequence
        if !self.move_history.is_empty() {
            let moves_to_undo = if self.is_in_computer_turn() { 2 } else { 1 };

            if self.current_state_index < moves_to_undo {
                println!("Cannot undo: Would go before game start.");
                return false;
            }

            // Undo the appropriate number of moves
            for _ in 0..moves_to_undo {
                if self.current_state_index > 0 {
                    self.current_state_index -= 1;
                    if !self.move_history.is_empty() {
                        let (undone_move, _player, description) = self.move_history.pop().unwrap();
                        println!("Undone: {} - {}", undone_move, description);
                    }
                }
            }

            self.game = self.game_states[self.current_state_index].clone();
            return true;
        }

        false
    }

    fn redo_move(&mut self) -> bool {
        if self.current_state_index >= self.game_states.len() - 1 {
            println!("Cannot redo: Already at the latest position.");
            return false;
        }

        // Redo moves (typically 1 or 2 to get back to player's turn)
        let moves_to_redo = if self.current_state_index + 2 < self.game_states.len()
            && !self.is_in_computer_turn()
        {
            2
        } else {
            1
        };

        for _ in 0..moves_to_redo {
            if self.current_state_index < self.game_states.len() - 1 {
                self.current_state_index += 1;
                // We'd need to reconstruct move history here, but for simplicity
                // we'll just show the position
            }
        }

        self.game = self.game_states[self.current_state_index].clone();
        println!("Redone to position {}", self.current_state_index);
        return true;
    }

    fn is_in_computer_turn(&self) -> bool {
        self.game.current_position().side_to_move() != self.player_color
    }

    fn display_game_result(&self) {
        match self.game.result() {
            Some(chess::GameResult::WhiteCheckmates) => {
                if self.player_color == Color::White {
                    println!("\nCongratulations! You won by checkmate!");
                } else {
                    println!("\nComputer wins by checkmate!");
                }
            }
            Some(chess::GameResult::BlackCheckmates) => {
                if self.player_color == Color::Black {
                    println!("\nCongratulations! You won by checkmate!");
                } else {
                    println!("\nComputer wins by checkmate!");
                }
            }
            Some(chess::GameResult::WhiteResigns) => {
                println!("\nWhite resigns!");
            }
            Some(chess::GameResult::BlackResigns) => {
                println!("\nBlack resigns!");
            }
            Some(chess::GameResult::Stalemate) => {
                println!("\nGame ended in stalemate - it's a draw!");
            }
            Some(chess::GameResult::DrawAccepted) => {
                println!("\nGame ended in a draw!");
            }
            Some(chess::GameResult::DrawDeclared) => {
                println!("\nGame ended in a draw (insufficient material or repetition)!");
            }
            None => {
                println!("\nGame in progress...");
            }
        }
    }
}

enum GameAction {
    Continue,
    Quit,
}
