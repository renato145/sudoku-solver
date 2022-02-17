use colored::Colorize;
use log::info;
use std::env;
use sudoku_solver::{solve_sudoku, solve_sudoku_parallel, Sudoku};

fn main() {
    env_logger::init();
    info!("Starting...");

    let text = env::args().nth(1).expect("No problem found.");
    match Sudoku::from_text(&text) {
        Ok(board) => {
            println!("Input:\n{board}");
            // match solve_sudoku(board) {
            match solve_sudoku_parallel(board) {
                Ok((solution, time)) => {
                    println!("Found a solution in {time} iterations.\n{solution}");
                }
                Err((err, time)) => {
                    println!("{}", format!("{err} ({time} iterations)").red());
                }
            }
        }
        Err(err) => {
            println!("{}", err.red());
        }
    }
}
