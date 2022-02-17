use colored::Colorize;
use itertools::Itertools;
use std::collections::HashSet;

const N: usize = 9;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Sudoku {
    rows: [[Item; N]; N],
    pub state: SudokuState,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Item {
    Number(u16),
    Empty,
    Guesses(Vec<u16>),
    Error,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SudokuState {
    Unknown,
    /// Contains index for the next guess
    HaveGuesses((usize, usize)),
    Invalid,
    Solved,
}

impl Item {
    fn get_number(&self) -> Option<u16> {
        if let Item::Number(x) = self {
            Some(*x)
        } else {
            None
        }
    }
}

impl Sudoku {
    pub fn from_text(text: &str) -> Result<Self, String> {
        let mut rows: [[Item; N]; N] = (0..N)
            .map(|_| {
                (0..N)
                    .map(|_| Item::Empty)
                    .collect_vec()
                    .try_into()
                    .unwrap()
            })
            .collect_vec()
            .try_into()
            .unwrap();
        for (i, line) in text.lines().enumerate() {
            for (j, c) in line.chars().enumerate() {
                match c {
                    ' ' => {
                        rows[i][j] = Item::Empty;
                    }
                    c => {
                        let x = c
                            .to_digit(10)
                            .unwrap_or_else(|| panic!("Invalid char: {c}"))
                            as u16;
                        rows[i][j] = Item::Number(x);
                    }
                }
            }
        }
        let board = Self {
            rows,
            state: SudokuState::Unknown,
        };

        if board.is_valid() {
            Ok(board)
        } else {
            Err("Invalid board".to_string())
        }
    }

    pub fn is_solved(&self) -> bool {
        matches!(self.state, SudokuState::Solved)
    }

    pub fn get(&self, i: usize, j: usize) -> &Item {
        &self.rows[i][j]
    }

    pub fn set(&mut self, i: usize, j: usize, number: u16) {
        self.rows[i][j] = Item::Number(number);
    }

    fn get_row_values(&self, i: usize) -> Vec<u16> {
        self.rows[i]
            .iter()
            .filter_map(|x| x.get_number())
            .collect_vec()
    }

    fn get_col_values(&self, j: usize) -> Vec<u16> {
        self.rows
            .iter()
            .map(|row| &row[j])
            .filter_map(|x| x.get_number())
            .collect_vec()
    }

    fn get_square_values(&self, i: usize, j: usize) -> Vec<u16> {
        let i0 = (i / 3) * 3;
        let j0 = (j / 3) * 3;
        (i0..i0 + 3)
            .cartesian_product(j0..j0 + 3)
            .map(|(i, j)| self.get(i, j))
            .filter_map(|x| x.get_number())
            .collect_vec()
    }

    pub fn get_guesses(&self, i: usize, j: usize) -> Vec<u16> {
        let others = self
            .get_row_values(i)
            .into_iter()
            .chain(self.get_col_values(j).into_iter())
            .chain(self.get_square_values(i, j).into_iter())
            .collect::<HashSet<_>>();
        (1..=9).filter(|x| !others.contains(x)).collect()
    }

    pub fn compute_guesses(&mut self) {
        let guess_idxs = self
            .rows
            .iter()
            .enumerate()
            .map(|(i, row)| {
                row.iter().enumerate().filter_map(move |(j, x)| {
                    if let Item::Number(_) = x {
                        None
                    } else {
                        Some((i, j))
                    }
                })
            })
            .flatten()
            .collect_vec();

        let mut invalid = false;
        let guess_idxs = guess_idxs
            .into_iter()
            .filter_map(|(i, j)| {
                let guesses = self.get_guesses(i, j);
                match guesses.len() {
                    0 => {
                        invalid = true;
                        self.rows[i][j] = Item::Error;
                        None
                    }
                    1 => {
                        self.rows[i][j] = Item::Number(guesses[0]);
                        None
                    }
                    _ => {
                        self.rows[i][j] = Item::Guesses(guesses);
                        Some((i, j))
                    }
                }
            })
            .collect_vec();
        if invalid {
            self.state = SudokuState::Invalid;
        } else if let Some(&idx) = guess_idxs.first() {
            self.state = SudokuState::HaveGuesses(idx);
        } else {
            self.state = SudokuState::Solved;
        }
    }

    fn is_valid(&self) -> bool {
        let groups = (0..N)
            .map(|i| self.get_row_values(i))
            .chain((0..N).map(|j| self.get_col_values(j)))
            .chain(
                (0..2)
                    .cartesian_product(0..2)
                    .map(|(i, j)| self.get_square_values(i, j)),
            )
            .map(|group| group.into_iter().counts().into_values().max().unwrap_or(0))
            .max()
            .unwrap_or(0);
        groups <= 1
    }
}

impl std::fmt::Display for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut line = String::new();
        let horizontal_line = " ----------------- ";
        for (i, row) in self.rows.iter().enumerate() {
            if i % 3 == 0 {
                writeln!(f, "{}", horizontal_line)?;
            }
            for (j, x) in row.iter().enumerate() {
                line.push(if j % 3 == 0 { '|' } else { ' ' });
                match x {
                    Item::Number(n) => {
                        line.push_str(&format!("{n}"));
                    }
                    Item::Empty => {
                        line.push_str(&" ".on_blue().to_string());
                    }
                    Item::Guesses(_) => {
                        line.push_str(&"G".green().to_string());
                    }
                    Item::Error => {
                        line.push_str(&" ".on_red().to_string());
                    }
                }
            }
            writeln!(f, "{line}|")?;
            line.clear();
        }
        writeln!(f, "{}", horizontal_line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_sudoku_from_text_works() {
        let text = " 1
69  2  57
    692
  9   4
47     2
581 9   3
  5  86
 4 2  8 1
   6   4";
        let board = Sudoku::from_text(text).unwrap();
        println!("{board}");
    }

    #[test]
    fn create_sudoku_from_text_fails_on_invalid_input() {
        let text = " 1
699 2  57
    692
  9   4
47     2
581 9   3
  5  86
 4 2  8 1
   6   4";
        let err = Sudoku::from_text(text).unwrap_err();
        println!("{err}");
    }

    #[test]
    fn get_row_values_works() {
        let text = "926817345
851394726
473265891
685123479
734589162
219746538
586472 1
342951687
197638254";
        let board = Sudoku::from_text(text).unwrap();
        let row = board.get_row_values(0);
        let expected = vec![9, 2, 6, 8, 1, 7, 3, 4, 5];
        assert_eq!(row, expected);
    }

    #[test]
    fn get_col_values_works() {
        let text = "926817345
851394726
473265891
685123479
734589162
219746538
586472 1
342951687
197638254";
        let board = Sudoku::from_text(text).unwrap();
        let col = board.get_col_values(0);
        let expected = vec![9, 8, 4, 6, 7, 2, 5, 3, 1];
        assert_eq!(col, expected);
    }

    #[test]
    fn get_square_values_works() {
        let text = "926817345
851394726
473265891
685123479
734589162
219746538
586472 1
342951687
197638254";
        let board = Sudoku::from_text(text).unwrap();
        let cases = [
            ((1, 1), vec![9, 2, 6, 8, 5, 1, 4, 7, 3]),
            ((6, 8), vec![1, 6, 8, 7, 2, 5, 4]),
        ];
        for ((i, j), expected) in cases {
            let square = board.get_square_values(i, j);
            assert_eq!(square, expected);
        }
    }

    #[test]
    fn get_guesses_works() {
        let text = " 26817345
851394726
473265891
685123479
734589162
219746538
586472 1
342951687
197638254";
        let board = Sudoku::from_text(text).unwrap();
        let guesses = board.get_guesses(0, 0);
        let expected = vec![9];
        println!("{board}");
        println!("{guesses:?}");
        assert_eq!(guesses, expected);
    }

    #[test]
    fn compute_guesses_works() {
        let text = "926817 45
8 139 726
4  26 891
6 5   47
73  8 1 2
2 97465 8
    72  
 42  1  7
1 76 8  4";
        let mut board = Sudoku::from_text(text).unwrap();
        println!("{board}");
        board.compute_guesses();
        println!("{board}");
        if let Item::Guesses(guesses) = board.get(6, 8) {
            assert_eq!(*guesses, vec![3, 9]);
        } else {
            unreachable!();
        }
    }
}
