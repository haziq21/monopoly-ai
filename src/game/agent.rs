use super::state::State;
use std::time::{Duration, Instant};

pub struct MCTreeNode {
    total_value: u8,
    num_visits: u32,
    children: Vec<Box<MCTreeNode>>,
}

impl MCTreeNode {
    /// Return a new MCTS node with `t` and `n` set to 0.
    fn new() -> MCTreeNode {
        MCTreeNode {
            total_value: 0,
            num_visits: 0,
            children: vec![],
        }
    }

    /// Generate as many child tree nodes as needed to mirror `state`'s children.
    /// This should only be called when this MCTS node has no children, or has
    /// the same amount of children as `state`.
    fn sync_children_count(&mut self, state: &State) {
        let mctree_children_count = self.children.len();
        let state_children_count = state.children.len();

        if mctree_children_count == state_children_count {
            return;
        }

        if mctree_children_count != 0 {
            panic!(
                "MCTreeNode::sync_children_count() - mctree_children_count == {}",
                mctree_children_count
            );
        }

        for _ in &state.children {
            self.children.push(Box::new(MCTreeNode::new()))
        }
    }
}

/// An agent playing the game, or the "brains" of a player.
pub enum Agent {
    /// An MCTS AI agent.
    Ai {
        /// Amount of time that the AI is given to "think", in milliseconds.
        time_limit: u64,
        /// Value of `C` constant in UCB1 formula.
        temperature: f64,
        /// Index of the last move that this agent played, from `Game.move_history`.
        last_move_index: usize,
        /// The Monte-Carlo search tree associated with this AI.
        mcts_tree: MCTreeNode,
    },
    /// A physical human player.
    Human,
}

impl Agent {
    /*********        PUBLIC INTERFACES        *********/

    /// Return a new AI agent.
    pub fn new_ai(time_limit: u64, temperature: f64) -> Agent {
        Agent::Ai {
            time_limit,
            temperature,
            last_move_index: 0,
            mcts_tree: MCTreeNode::new(),
        }
    }

    /// Return a new human agent.
    pub fn new_human() -> Agent {
        Agent::Human
    }

    /// Let the agent make a move to generate. Return the index from `from_node`'s children resulting state after the agent makes a choice.
    pub fn make_choice(&mut self, from_node: &mut State, move_history: &Vec<usize>) -> usize {
        match self {
            Agent::Ai { .. } => self.ai_choice(from_node, move_history),
            Agent::Human => self.human_choice(from_node),
        }
    }

    /*********        FOR AI PLAYERS        *********/

    fn ai_choice(&mut self, state_node: &mut State, _move_history: &Vec<usize>) -> usize {
        let start = Instant::now();
        let (max_time, temperature, mcts_node) = match self {
            Agent::Ai {
                time_limit,
                temperature,
                mcts_tree,
                ..
            } => (Duration::from_millis(*time_limit), *temperature, mcts_tree),
            _ => unreachable!(),
        };

        // Ensure `mcts_node` has all of its direct children
        mcts_node.sync_children_count(state_node);

        // Continue searching until time is up
        while start.elapsed() < max_time {
            Agent::mcts_traverse(state_node, mcts_node, temperature);
        }

        0
    }

    /// Traverse the MCTS tree from the root node and create child nodes as needed. Return rollout result.
    fn mcts_traverse(state_node: &mut State, mcts_node: &mut MCTreeNode, temperature: f64) -> u8 {
        // If mcts_node is not a leaf node, calculate the UCB1 values of its child nodes
        if mcts_node.children.len() > 0 {
            // The UCB1 formula is `V_i + C * sqrt( ln(N) / n_i )`

            // mean_value = V_i
            let mean_value = mcts_node.total_value as f64 / mcts_node.num_visits as f64;
            // All the UCB1 values with respect to from_node's children
            let ucb1_values: Vec<f64> = mcts_node
                .children
                .iter()
                .map(|s| {
                    if mcts_node.num_visits == 0 || s.num_visits == 0 {
                        f64::INFINITY
                    } else {
                        mean_value
                            + temperature
                                * ((mcts_node.num_visits as f64).ln() / s.num_visits as f64).sqrt()
                    }
                })
                .collect();

            // The child to select next
            let child_index = ucb1_values
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, _)| i)
                .unwrap();

            let propagated_value = Agent::mcts_traverse(
                &mut state_node.children[child_index],
                &mut mcts_node.children[child_index],
                temperature,
            );

            // Update n and t
            mcts_node.num_visits += 1;
            mcts_node.total_value += propagated_value;

            return propagated_value;
        }

        // Perform a rollout if the node has never been visited before
        if mcts_node.num_visits == 0 {
            let rollout_outcome = state_node.rollout();

            // Update n and t
            mcts_node.num_visits += 1;
            mcts_node.total_value += rollout_outcome;

            return rollout_outcome;
        }

        // Expand the tree and rollout from the first child if
        // the node is a leaf node that hasn't been visited yet
        state_node.generate_children();

        // Sync the MCTS tree with the game-state tree
        mcts_node.sync_children_count(state_node);

        state_node.children[0].rollout()
    }

    /*********        FOR HUMAN PLAYERS        *********/

    fn human_choice(&self, _from_node: &mut State) -> usize {
        0
    }
}
