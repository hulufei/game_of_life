use board::Board;
use crossterm::{style, terminal, QueueableCommand};
use std::{
    convert::TryFrom,
    fs,
    io::{stdout, Write},
    path::PathBuf,
    thread::sleep,
    time::Duration,
};
use structopt::StructOpt;

mod board;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, StructOpt)]
#[structopt(about = "Game of life")]
struct Opt {
    /// Load from a file
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,
    #[structopt(short, long, default_value = "60")]
    with: usize,
    #[structopt(short, long, default_value = "60")]
    height: usize,
    /// Specify FPS
    #[structopt(short, long, default_value = "10")]
    fps: f64,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let mut board = match opt.input {
        Some(path) => Board::try_from(fs::read_to_string(path)?.as_str())?,
        None => {
            let mut board = Board::new(opt.with, opt.height);
            board.random_state();
            board
        }
    };

    let mut stdout = stdout();
    loop {
        stdout
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(style::Print(&board))?
            .flush()?;

        board = board.next_board_state();

        sleep(Duration::from_secs_f64(1. / opt.fps));
    }
}
