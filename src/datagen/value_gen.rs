use std::sync::atomic::AtomicBool;

use crossbeam_queue::SegQueue;

use crate::{
    options::EngineOptions,
    search::{GameState, Mcts, NoPrint, SearchLimits, SearchStats, SearchTree},
    spear::{ChessBoardPacked, ChessPosition, Move, Side},
};

use super::{printer::Printer, utils::DataGenUtils};

pub struct ValueGen;
impl ValueGen {
    pub fn start_game_loop(save_queue: &SegQueue<Vec<u8>>, iter_count: u32, printer: &Printer) {
        let mut tree = SearchTree::new();
        let options = EngineOptions::new();
        let interruption_token = AtomicBool::new(false);
        let mut limits = SearchLimits::new();
        limits.add_iters(iter_count);

        loop {
            let mut position = DataGenUtils::get_random_position();
            tree.clear();

            let mut packed_positions: Vec<ChessBoardPacked> = Vec::new();
            let mut state = GameState::Unresolved;

            while state == GameState::Unresolved {
                tree.clear();

                let search_stats = SearchStats::new();

                let mcts = Mcts::new(
                    position,
                    &tree,
                    &interruption_token,
                    &options,
                    &search_stats,
                    &limits,
                );

                let (best_move, best_score) = mcts.search::<NoPrint>();
                let packed_position =
                    ChessBoardPacked::from_board(position.board(), best_score as f32);

                let is_game_end = if position.board().side_to_move() == Side::WHITE {
                    Self::process_move::<true, false>(&mut position, best_move, &mut state)
                } else {
                    Self::process_move::<false, true>(&mut position, best_move, &mut state)
                };

                if is_game_end {
                    continue;
                }

                packed_positions.push(packed_position);
            }

            if state != GameState::Drawn {
                for pos in &mut packed_positions {
                    pos.apply_result( position.board().side_to_move().flipped() )
                }
            }

            printer.add_position(packed_positions.len() as u64);

            let bytes = bytemuck::cast_slice(&packed_positions);
            save_queue.push(bytes.to_vec());
        }
    }

    fn process_move<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        position: &mut ChessPosition,
        best_move: Move,
        state: &mut GameState,
    ) -> bool {
        let mut no_moves = true;
        position.make_move::<STM_WHITE, NSTM_WHITE>(best_move);
        position.board().map_moves::<_, NSTM_WHITE, STM_WHITE>(|_| {
            no_moves = false;
        });

        if no_moves {
            *state = if position.board().is_in_check::<NSTM_WHITE, STM_WHITE>() {
                GameState::Lost(0)
            } else {
                GameState::Drawn
            };
            return true;
        } else if position.is_repetition()
            || position.board().is_insufficient_material()
            || position.board().half_move_counter() >= 100
        {
            *state = GameState::Drawn;
            return true;
        }

        return false;
    }
}