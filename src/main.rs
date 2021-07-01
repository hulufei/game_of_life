use board::Board;
use crossterm::{style, terminal, ExecutableCommand};
use std::{
    io::{stdout, Write},
    thread::sleep,
    time::Duration,
};

mod board;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let mut board = Board::new(50, 50);
    board.random_state();

    let mut stdout = stdout();
    loop {
        board = board.next_board_state();
        stdout
            .execute(terminal::Clear(terminal::ClearType::All))?
            .execute(style::Print(&board))?
            .flush()?;
        // 30FPS = 33ms
        sleep(Duration::from_millis(80));
    }
}
