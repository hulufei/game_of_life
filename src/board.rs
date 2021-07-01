use crate::Error;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::{convert::TryFrom, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum State {
    ALIVE,
    DEAD,
}

impl From<bool> for State {
    fn from(b: bool) -> Self {
        if b {
            State::ALIVE
        } else {
            State::DEAD
        }
    }
}

impl From<u8> for State {
    fn from(c: u8) -> Self {
        match c {
            b'.' | b'0' => State::DEAD,
            _ => State::ALIVE,
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                State::ALIVE => '#',
                State::DEAD => '.',
            }
        )
    }
}

impl Distribution<State> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> State {
        match rng.gen_range(0..2) {
            0 => State::DEAD,
            _ => State::ALIVE,
        }
    }
}

#[derive(Clone)]
pub struct Board {
    state: Vec<Vec<State>>,
}

impl Board {
    pub fn new(w: usize, h: usize) -> Self {
        let row = vec![State::DEAD; w];
        let s = vec![row; h];
        Self { state: s }
    }

    pub fn width(&self) -> usize {
        self.state.first().map_or(0, |row| row.len())
    }

    pub fn height(&self) -> usize {
        self.state.len()
    }

    pub fn random_state(&mut self) {
        // Cache for performance
        let mut rng = rand::thread_rng();
        for row in self.state.iter_mut() {
            for cell in row.iter_mut() {
                *cell = rng.gen();
            }
        }
    }

    pub fn next_board_state(&self) -> Self {
        // Rules:
        // 1. Any live cell with 0 or 1 live neighbors becomes dead, because of underpopulation
        // 2. Any live cell with 2 or 3 live neighbors stays alive, because its neighborhood is just right
        // 3. Any live cell with more than 3 live neighbors becomes dead, because of overpopulation
        // 4. Any dead cell with exactly 3 live neighbors becomes alive, by reproduction
        let mut new_board = self.clone();
        for y in 0..self.height() {
            for x in 0..self.width() {
                let prev_row = y.checked_sub(1);
                let next_row = y.checked_add(1);
                let prev_col = x.checked_sub(1);
                let next_col = x.checked_add(1);
                // Clockwise
                let neighbors = [
                    prev_row.zip(Some(x)),
                    prev_row.zip(next_col),
                    Some(y).zip(next_col),
                    next_row.zip(next_col),
                    next_row.zip(Some(x)),
                    next_row.zip(prev_col),
                    Some(y).zip(prev_col),
                    prev_row.zip(prev_col),
                ];
                let live_counts = neighbors
                    .iter()
                    .filter_map(|pos| {
                        pos.and_then(|(y, x)| self.state.get(y).and_then(|row| row.get(x)))
                    })
                    .filter(|&&cell| cell == State::ALIVE)
                    .count();
                match (self.state[y][x], live_counts) {
                    (State::ALIVE, 0 | 1) => new_board.state[y][x] = State::DEAD,
                    (State::ALIVE, count) if count > 3 => new_board.state[y][x] = State::DEAD,
                    (State::DEAD, 3) => new_board.state[y][x] = State::ALIVE,
                    _ => (),
                }
            }
        }
        new_board
    }
}

impl<const W: usize, const H: usize> From<[[bool; W]; H]> for Board {
    fn from(s: [[bool; W]; H]) -> Self {
        Self {
            state: s
                .iter()
                .map(|r| r.iter().copied().map(State::from).collect())
                .collect(),
        }
    }
}

impl TryFrom<&str> for Board {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lines = value.lines().collect::<Vec<_>>();
        if lines.windows(2).any(|w| w[0].len() != w[1].len()) {
            return Err("Unable to convert &str to Board, all lines' length should be equal")?;
        }
        let s = value
            .lines()
            .map(|line| line.bytes().map(State::from).collect())
            .collect();
        Ok(Self { state: s })
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.state {
            for &cell in row {
                write!(f, "{}", cell)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;
    use std::collections::HashSet;

    #[test]
    fn const_generic_init_works() {
        let board = Board::new(1, 1);
        assert_eq!(board.state, vec![vec![State::DEAD]]);
    }

    #[test]
    fn random_state() {
        let mut board = Board::new(100, 100);
        board.random_state();
        assert!(board
            .state
            .iter()
            .all(|row| row.iter().collect::<HashSet<_>>().len() == 2));
    }

    #[test]
    fn convert_from_bool_arr_works() {
        let board = Board::from([[true, false], [false, true]]);
        assert_eq!(
            board.state,
            vec![
                vec![State::ALIVE, State::DEAD],
                vec![State::DEAD, State::ALIVE],
            ]
        );
    }

    #[test]
    fn convert_from_str_works() -> Result<()> {
        let s = "#.\n.#";
        let board = Board::try_from(s)?;
        assert_eq!(
            board.state,
            vec![
                vec![State::ALIVE, State::DEAD],
                vec![State::DEAD, State::ALIVE],
            ]
        );
        Ok(())
    }

    #[test]
    fn convert_from_str_should_be_equal_line() {
        let s = r"#.....#
#..
#.....#
";
        let board = Board::try_from(s);
        assert!(board.is_err());
    }

    #[test]
    fn to_string_works() {
        let board = Board::from([[true, false], [false, true]]);
        assert_eq!(board.to_string(), "#.\n.#\n");
    }

    #[test]
    fn next_board_state_edge_check() -> Result<()> {
        let board = Board::try_from("#")?.next_board_state();
        assert_eq!(board.to_string().trim(), ".");
        Ok(())
    }

    #[test]
    fn next_board_state_under_population() -> Result<()> {
        let s = r"...
#..
#..
...";
        let board = Board::try_from(s)?.next_board_state();
        insta::assert_display_snapshot!(board, @r###"
        ...
        ...
        ...
        ...
        "###);
        Ok(())
    }

    #[test]
    fn next_board_state_just_right() -> Result<()> {
        let s = r"...
##.
##.
...";
        let board = Board::try_from(s)?.next_board_state();
        insta::assert_display_snapshot!(board, @r###"
        ...
        ##.
        ##.
        ...
        "###);
        Ok(())
    }

    #[test]
    fn next_board_state_over_population() -> Result<()> {
        let s = r"#.#
###
#.#";
        let board = Board::try_from(s)?.next_board_state();
        insta::assert_display_snapshot!(board, @r###"
        #.#
        #.#
        #.#
        "###);
        Ok(())
    }

    #[test]
    fn next_board_state_reproduction() -> Result<()> {
        let s = r".....
.....
##.##
#.#.#
.....";
        let board = Board::try_from(s)?.next_board_state();
        insta::assert_display_snapshot!(board, @r###"
        .....
        .....
        #####
        #.#.#
        .....
        "###);
        Ok(())
    }
}
