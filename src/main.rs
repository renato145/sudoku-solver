use colored::Colorize;
use itertools::Itertools;
use log::info;
use std::env;
use sudoku_solver::{solve_sudoku, solve_sudoku_parallel, Sudoku};

fn main() {
    env_logger::init();
    info!("Starting...");

    let args = env::args().skip(1).collect_vec();
    let (parallel, text) = match args.len() {
        0 => {
            eprintln!("No input found");
            std::process::exit(1);
        }
        1 => (false, args[0].clone()),
        2 => (true, args[1].clone()),
        _ => {
            eprintln!("Invalid number of arguments");
            std::process::exit(1);
        }
    };

    match Sudoku::from_text(&text) {
        Ok(board) => {
            println!("Input:\n{board}");
            let res = if parallel {
                println!("Solving in parallel mode...");
                solve_sudoku_parallel(board)
            } else {
                println!("Solving in sequential mode...");
                solve_sudoku(board)
            };
            match res {
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
