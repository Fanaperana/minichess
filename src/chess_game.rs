use crate::stockfish::StockfishEngine;
use crate::ui::{display_board, get_user_input, print_help};
use anyhow::{anyhow, Result};
use chess::{ChessMove, Color, Game, Square, MoveGen, Piece};
use std::str::FromStr;

pub struct ChessGame {
    game: Game,
    engine: StockfishEngine,
    player_color: Color,
    move_history: Vec<(ChessMove, String, String)>, // (move, description, detailed_description)
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
            game,
            engine,
            player_color,
            move_history: Vec::new(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("\nGame started! You are playing as {:?}", self.player_color);
        display_board(&self.game.current_position());

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
                    display_board(&self.game.current_position());
                    continue;
                }
                _ => {
                    match self.parse_and_make_move(&input) {
                        Ok(move_made) => {
                            // Add player move to history
                            let player_color_str = if self.player_color == Color::White { "White" } else { "Black" };
                            let move_description = self.describe_move(&move_made, &self.game.current_position());
                            let detailed_description = format!("{} (You): {}", player_color_str, move_description);
                            self.move_history.push((move_made, player_color_str.to_string(), detailed_description));
                            
                            display_board(&self.game.current_position());
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
            let to = Square::from_str(to_str)
                .map_err(|_| anyhow!("Invalid to square: {}", to_str))?;
            
            ChessMove::new(from, to, None)
        } else if input.len() == 5 {
            // Promotion moves like "e7e8q"
            let from_str = &input[0..2];
            let to_str = &input[2..4];
            let promotion_str = &input[4..5];
            
            let from = Square::from_str(from_str)
                .map_err(|_| anyhow!("Invalid from square: {}", from_str))?;
            let to = Square::from_str(to_str)
                .map_err(|_| anyhow!("Invalid to square: {}", to_str))?;
            
            let promotion = match promotion_str {
                "q" => Some(chess::Piece::Queen),
                "r" => Some(chess::Piece::Rook),
                "b" => Some(chess::Piece::Bishop),
                "n" => Some(chess::Piece::Knight),
                _ => return Err(anyhow!("Invalid promotion piece: {}", promotion_str)),
            };
            
            ChessMove::new(from, to, promotion)
        } else {
            return Err(anyhow!("Invalid move format. Use format like 'e2e4' or 'e7e8q' for promotions"));
        };

        // Verify the move is legal
        let legal_moves: Vec<ChessMove> = MoveGen::new_legal(&self.game.current_position()).collect();
        
        if !legal_moves.contains(&chess_move) {
            return Err(anyhow!("Move is not legal in current position"));
        }

        self.game.make_move(chess_move);
        Ok(chess_move)
    }

    async fn make_computer_move(&mut self) -> Result<()> {
        println!("\nComputer is thinking...");
        
        let best_move = self.engine.get_best_move(&self.game.current_position()).await?;
        
        // Describe the move before making it
        let move_description = self.describe_move(&best_move, &self.game.current_position());
        
        println!("Computer plays: {} ({})", best_move, move_description);
        
        // Add computer move to history
        let computer_color_str = if self.player_color == Color::White { "Black" } else { "White" };
        let detailed_description = format!("{} (Computer): {}", computer_color_str, move_description);
        self.move_history.push((best_move, computer_color_str.to_string(), detailed_description));
        
        self.game.make_move(best_move);
        display_board(&self.game.current_position());
        
        Ok(())
    }

    fn show_legal_moves(&self) {
        let legal_moves: Vec<ChessMove> = MoveGen::new_legal(&self.game.current_position()).collect();
        
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

        println!("\n=== Move History ===");
        for (i, (chess_move, _player, _detailed)) in self.move_history.iter().enumerate() {
            let move_number = (i / 2) + 1;
            
            if i % 2 == 0 {
                // White's move (or first player's move)
                print!("{}. {} ", move_number, chess_move);
            } else {
                // Black's move (or second player's move)
                println!("{}", chess_move);
            }
        }
        
        // If the last move was white's, add a newline
        if self.move_history.len() % 2 == 1 {
            println!();
        }
        
        println!("\nDetailed history:");
        for (i, (_chess_move, _player, detailed_description)) in self.move_history.iter().enumerate() {
            println!("{}. {}", i + 1, detailed_description);
        }
        println!("==================\n");
    }

    fn describe_move(&self, chess_move: &ChessMove, board: &chess::Board) -> String {
        let from_square = chess_move.get_source();
        let to_square = chess_move.get_dest();
        
        // Get the piece that's moving
        let piece = board.piece_on(from_square);
        let piece_color = board.color_on(from_square);
        
        let _piece_name = match piece {
            Some(Piece::King) => "King",
            Some(Piece::Queen) => "Queen", 
            Some(Piece::Rook) => "Rook",
            Some(Piece::Bishop) => "Bishop",
            Some(Piece::Knight) => "Knight",
            Some(Piece::Pawn) => "Pawn",
            None => "Unknown",
        };
        
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
            let king_start = if piece_color == Some(Color::White) { Square::E1 } else { Square::E8 };
            if from_square == king_start {
                if to_square == Square::G1 || to_square == Square::G8 {
                    return format!("{} castles kingside", piece_fen);
                } else if to_square == Square::C1 || to_square == Square::C8 {
                    return format!("{} castles queenside", piece_fen);
                }
            }
        }
        
        format!("{} {}->{}{}{}", 
                piece_fen, from_square, to_square, captured_piece, promotion)
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