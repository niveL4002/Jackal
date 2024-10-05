use spear::ChessPosition;

use crate::search::{tree::Edge, NodeIndex, Score, SearchHelpers};

use super::Mcts;

impl<'a> Mcts<'a> {
    pub(super) fn process_deeper_node<
        const STM_WHITE: bool,
        const NSTM_WHITE: bool,
        const ROOT: bool,
    >(
        &self,
        current_node_index: NodeIndex,
        action_cpy: &Edge,
        current_position: &mut ChessPosition,
        depth: &mut u32,
    ) -> Option<Score> {
        //If current non-root node is terminal or it's first visit, we don't want to go deeper into the tree
        //therefore we just evaluate the node and thats where recursion ends
        let score = if !ROOT
            && (self.tree[current_node_index].is_termial() || action_cpy.visits() == 0)
        {
            SearchHelpers::get_node_score::<STM_WHITE, NSTM_WHITE>(
                current_position,
                self.tree[current_node_index].state(),
            )
        } else {
            //On second visit we expand the node, if it wasn't already expanded.
            //This allows us to reduce amount of time we evaluate policy net
            if !self.tree[current_node_index].has_children() {
                self.expand::<STM_WHITE, NSTM_WHITE, false>(current_node_index, current_position)
            }

            //We then select the best action to evaluate and advance the position to the move of this action
            let best_action_index = self.select_action::<ROOT>(
                current_node_index,
                action_cpy.visits(),
                self.options.cpuct_value(),
            );
            let new_edge_cpy = self
                .tree
                .get_edge_clone(current_node_index, best_action_index);
            current_position.make_move::<STM_WHITE, NSTM_WHITE>(new_edge_cpy.mv());

            //Process the new action on the tree and obtain it's updated index
            let new_node_index = self.tree.get_node_index::<NSTM_WHITE, STM_WHITE>(
                current_position,
                new_edge_cpy.node_index(),
                current_node_index,
                best_action_index,
            )?;

            //Descend deeper into the tree
            *depth += 1;
            let score = self.process_deeper_node::<NSTM_WHITE, STM_WHITE, false>(
                new_node_index,
                &new_edge_cpy,
                current_position,
                depth,
            )?;

            //Backpropagate the score up the tree
            self.tree
                .add_edge_score(current_node_index, best_action_index, score);

            //Backpropagate mates to assure our engine avoids/follows mating line
            self.tree
                .backpropagate_mates(current_node_index, self.tree[new_node_index].state());

            score
        };

        Some(score.reversed())
    }

    pub fn expand<const STM_WHITE: bool, const NSTM_WHITE: bool, const ROOT: bool>(
        &self,
        node_idx: NodeIndex,
        position: &ChessPosition,
    ) {
        let mut actions = self.tree[node_idx].actions_mut();

        //Map moves into actions and set initial policy to 1
        position
            .board()
            .map_moves::<_, STM_WHITE, NSTM_WHITE>(|mv| {
                actions.push(Edge::new(NodeIndex::NULL, mv, 1.0))
            });

        //Update the policy to 1/action_count for uniform policy
        let action_count = actions.len() as f32;
        for action in actions.iter_mut() {
            action.update_policy(1.0 / action_count)
        }
    }

    //PUCT formula V + C * P * (N.max(1).sqrt()/n + 1) where N = number of visits to parent node, n = number of visits to a child
    #[inline]
    fn select_action<const ROOT: bool>(
        &self,
        node_idx: NodeIndex,
        parent_visits: u32,
        cpuct: f32,
    ) -> usize {
        assert!(self.tree[node_idx].has_children());

        let explore_value = cpuct * (parent_visits.max(1) as f32).sqrt();
        self.tree[node_idx].get_best_action_by_key(|action| {
            let visits = action.visits();
            let score = if visits == 0 {
                0.5
            } else {
                f32::from(action.score())
            };

            score + (explore_value * action.policy() / (visits as f32 + 1.0))
        })
    }
}
