use colored::Colorize;
use std::env;
use sudoku_solver::{solve_sudoku, Sudoku};

fn main() {
    let text = env::args().nth(1).expect("No problem found.");
    match Sudoku::from_text(&text) {
        Ok(board) => {
            println!("Input:\n{board}");
            match solve_sudoku(board) {
                Ok((solution, time)) => {
                    println!("Found a solution in {time} iterations.\n{solution}");
                }
                Err((err, time)) => {
                    println!("{}", format!("{err} ({time} iterations)").red());
                }
            }
        }
        Err(err) => {
            println!("{}", format!("{err}").red());
        }
    }
}
