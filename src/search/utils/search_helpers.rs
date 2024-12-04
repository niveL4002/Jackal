use spear::ChessPosition;

use crate::{
    search::{networks::ValueNetwork, Score}, EngineOptions, GameState, Tree
};

pub struct SearchHelpers;
impl SearchHelpers {
    #[inline]
    pub fn get_node_score<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        current_position: &mut ChessPosition,
        state: GameState,
        key: u64,
        tree: &Tree,
        material_difference: i32,
        options: &EngineOptions
    ) -> Score {

        let score_bonus = if material_difference != 0 {
            options.material_reduction_bonus() / 10.0
        } else {
            0.0
        };

        match state {
            GameState::Drawn => Score::DRAW,
            GameState::Lost(_) => Score::LOSE,
            GameState::Won(_) => Score::WIN,
            GameState::Unresolved => {
                if let Some(score) = tree.hash_table().probe(key) {
                    Score::new(score.win_chance() + score_bonus, score.draw_chance())
                } else {
                    let (win_chance, draw_chance, _) = ValueNetwork.forward::<STM_WHITE, NSTM_WHITE>(current_position.board());
                    let score = Score::new(win_chance, draw_chance);

                    tree.hash_table().store(key, score);

                    Score::new(score.win_chance() + score_bonus, score.draw_chance())
                }
            },
        }
    }

    pub fn get_position_state<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        position: &ChessPosition,
    ) -> GameState {
        if position.is_repetition()
            || position.board().is_insufficient_material()
            || position.board().half_move_counter() >= 100
        {
            return GameState::Drawn;
        }

        let mut move_count = 0;
        position.board().map_moves::<_, STM_WHITE, NSTM_WHITE>(|_| {
            move_count += 1;
        });

        if move_count == 0 {
            if position.board().is_in_check::<STM_WHITE, NSTM_WHITE>() {
                return GameState::Lost(0);
            } else {
                return GameState::Drawn;
            }
        }

        GameState::Unresolved
    }
}
