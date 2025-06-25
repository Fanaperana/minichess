use chess::{Board, Color, Piece, Square};
use std::io::{self, Write};

pub fn display_board(board: &Board) {
    println!("\n    a   b   c   d   e   f   g   h");
    println!("  ┌───┬───┬───┬───┬───┬───┬───┬───┐");

    for rank in (0..8).rev() {
        print!("{} │", rank + 1);
        
        for file in 0..8 {
            let square = Square::make_square(
                chess::Rank::from_index(rank),
                chess::File::from_index(file),
            );
            
            let piece_char = match board.piece_on(square) {
                Some(piece) => {
                    let color = board.color_on(square).unwrap();
                    piece_to_unicode(piece, color)
                }
                None => ' ',
            };
            
            print!(" {} │", piece_char);
        }
        
        print!(" {}", rank + 1);
        
        if rank > 0 {
            println!("\n  ├───┼───┼───┼───┼───┼───┼───┼───┤");
        }
    }
    
    println!("\n  └───┴───┴───┴───┴───┴───┴───┴───┘");
    println!("    a   b   c   d   e   f   g   h\n");
    
    // Show whose turn it is
    let turn = if board.side_to_move() == Color::White {
        "White"
    } else {
        "Black"
    };
    println!("{}'s turn to move", turn);
    
    // Show check status
    if board.checkers().popcnt() > 0 {
        println!("⚠️  {} is in check!", turn);
    }
}

fn piece_to_unicode(piece: Piece, color: Color) -> char {
    match (piece, color) {
        (Piece::King, Color::White) => '♔',
        (Piece::Queen, Color::White) => '♕',
        (Piece::Rook, Color::White) => '♖',
        (Piece::Bishop, Color::White) => '♗',
        (Piece::Knight, Color::White) => '♘',
        (Piece::Pawn, Color::White) => '♙',
        (Piece::King, Color::Black) => '♚',
        (Piece::Queen, Color::Black) => '♛',
        (Piece::Rook, Color::Black) => '♜',
        (Piece::Bishop, Color::Black) => '♝',
        (Piece::Knight, Color::Black) => '♞',
        (Piece::Pawn, Color::Black) => '♟',
    }
}

pub fn get_user_input() -> io::Result<String> {
    print!("> ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

pub fn print_help() {
    println!("\n=== Chess CLI Help ===");
    println!("Commands:");
    println!("  • Enter moves in coordinate notation: g1f3, e2e4, etc.");
    println!("  • For promotions, add the piece: e7e8q (queen), e7e8r (rook), etc.");
    println!("  • 'moves' - Show all legal moves");
    println!("  • 'history' - Show move history");
    println!("  • 'show' or 'board' - Redisplay the current board");
    println!("  • 'h' or 'help' - Show this help");
    println!("  • 'q' or 'quit' - Quit the game");
    println!("\nMove format examples:");
    println!("  • e2e4    - Move pawn from e2 to e4");
    println!("  • g1f3    - Move knight from g1 to f3 (NOT Ng1f3)");
    println!("  • e7e8q   - Promote pawn to queen");
    println!("  • e1g1    - Castle kingside");
    println!("  • e1c1    - Castle queenside");
    println!("\nIMPORTANT: Use coordinate notation (from-square + to-square)");
    println!("NOT standard algebraic notation (no piece letters like N, B, R, Q, K)");
    println!("\nSquares are labeled from a1 (bottom-left) to h8 (top-right)");
    println!("White pieces: ♔♕♖♗♘♙  Black pieces: ♚♛♜♝♞♟");
    println!("====================\n");
}