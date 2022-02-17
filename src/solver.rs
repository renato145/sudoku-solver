use crate::{
    graph::{dfs, Graph, GraphControl},
    sudoku::{Sudoku, SudokuState},
};

struct SudokuSolver;

impl Graph for SudokuSolver {
    type Node = Sudoku;

    fn neighbours(&self, node: &Self::Node) -> Vec<Self::Node> {
        match node.state {
            SudokuState::Unknown | SudokuState::Solved => unreachable!(),
            SudokuState::HaveGuesses((i, j)) => node
                .get_guesses(i, j)
                .into_iter()
                .map(|guess| {
                    let mut new_node = node.clone();
                    new_node.set(i, j, guess);
                    new_node
                })
                .collect(),
            SudokuState::Invalid => Vec::new(),
        }
    }

    fn check_goal(&self, node: &mut Self::Node) -> GraphControl {
        node.compute_guesses();
        match node.state {
            SudokuState::Invalid => GraphControl::Prune,
            SudokuState::Solved => GraphControl::Finish,
            _ => GraphControl::Continue,
        }
    }
}

pub fn solve_sudoku(board: Sudoku) -> Result<(Sudoku, usize), (String, usize)> {
    let graph = SudokuSolver;
    dfs(graph, board)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_sudoku_works() {
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
        let (solved_board, time) = solve_sudoku(board).unwrap();
        println!("({time} iterations)\n{solved_board}");
    }
}
