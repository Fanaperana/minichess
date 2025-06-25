use anyhow::{anyhow, Result};
use chess::{Board, ChessMove};
use std::str::FromStr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

pub struct StockfishEngine {
    process: Child,
    reader: BufReader<tokio::process::ChildStdout>,
}

impl StockfishEngine {
    pub async fn new(stockfish_path: &str) -> Result<Self> {
        let mut process = Command::new(stockfish_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("Failed to start Stockfish: {}. Make sure Stockfish is installed and in PATH", e))?;

        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to get stdout from Stockfish"))?;

        let reader = BufReader::new(stdout);
        let mut engine = StockfishEngine { process, reader };

        // Initialize UCI
        engine.send_command("uci").await?;
        engine.wait_for_response("uciok").await?;

        // Set up the engine
        engine.send_command("isready").await?;
        engine.wait_for_response("readyok").await?;

        Ok(engine)
    }

    pub async fn set_difficulty(&mut self, level: u8) -> Result<()> {
        let level = level.clamp(1, 20);
        self.send_command(&format!("setoption name Skill Level value {}", level))
            .await?;
        Ok(())
    }

    pub async fn get_best_move(&mut self, position: &Board) -> Result<ChessMove> {
        // Set up position
        let fen = position.to_string();
        self.send_command(&format!("position fen {}", fen)).await?;

        // Request best move
        self.send_command("go depth 10").await?;

        // Wait for bestmove response
        let mut line = String::new();
        loop {
            line.clear();
            self.reader.read_line(&mut line).await?;
            
            if line.starts_with("bestmove") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let move_str = parts[1];
                    return ChessMove::from_str(move_str)
                        .map_err(|_| anyhow!("Invalid move from Stockfish: {}", move_str));
                }
            }
        }
    }

    async fn send_command(&mut self, command: &str) -> Result<()> {
        if let Some(stdin) = self.process.stdin.as_mut() {
            stdin.write_all(format!("{}\n", command).as_bytes()).await?;
            stdin.flush().await?;
        }
        Ok(())
    }

    async fn wait_for_response(&mut self, expected: &str) -> Result<()> {
        let mut line = String::new();
        loop {
            line.clear();
            self.reader.read_line(&mut line).await?;
            
            if line.trim() == expected {
                break;
            }
        }
        Ok(())
    }
}

impl Drop for StockfishEngine {
    fn drop(&mut self) {
        // Kill the process when the engine is dropped
        let _ = self.process.start_kill();
    }
}