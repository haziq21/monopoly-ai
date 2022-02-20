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

    /// Return `self.total_value / self.num_visits`.
    fn get_average_value(&self) -> f64 {
        self.total_value as f64 / self.num_visits as f64
    }

    /// Return the index of the child with the greatest average value.
    fn get_best_child_index(&self) -> usize {
        self.children
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.get_average_value()
                    .partial_cmp(&b.get_average_value())
                    .unwrap()
            })
            .map(|(i, _)| i)
            .unwrap()
    }

    /// Generate as many direct child nodes as needed to mirror `state`'s
    /// direct children. This should only be called when this MCTS node
    /// has no children, or has the same amount of children as `state`.
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

    /// Traverse the tree according to the indexes in `walk`.
    /// Replace this node with the node at the end of the traversal.
    fn sync_with_walk(&mut self, walk: &[usize]) {
        for &step in walk {
            if self.children.len() == 0 {
                *self = MCTreeNode::new();
                break;
            }

            *self = std::mem::replace(self.children[step].as_mut(), MCTreeNode::new());
        }
    }

    /// Traverse the MCTS tree and create child nodes as needed. Return rollout result.
    fn traverse(&mut self, state_node: &mut State, temperature: f64) -> u8 {
        // If `self` is not a leaf node, calculate the UCB1 values of its child nodes
        if self.children.len() > 0 {
            // The UCB1 formula is `V_i + C * sqrt( ln(N) / n_i )`

            // mean_value = V_i
            let mean_value = self.total_value as f64 / self.num_visits as f64;

            // All the UCB1 values of `self`'s children
            let ucb1_values: Vec<f64> = self
                .children
                .iter()
                .map(|s| {
                    if self.num_visits == 0 || s.num_visits == 0 {
                        f64::INFINITY
                    } else {
                        mean_value
                            + temperature
                                * ((self.num_visits as f64).ln() / s.num_visits as f64).sqrt()
                    }
                })
                .collect();

            // The index of the child to traverse next
            let child_index = ucb1_values
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, _)| i)
                .unwrap();

            // Value of the rollout to propagate
            let propagated_value = self.children[child_index]
                .traverse(&mut state_node.children[child_index], temperature);

            // Update n and t
            self.num_visits += 1;
            self.total_value += propagated_value;

            return propagated_value;
        }

        // Perform a rollout if the node has never been visited before
        if self.num_visits == 0 {
            let rollout_outcome = state_node.rollout();

            // Update n and t
            self.num_visits += 1;
            self.total_value += rollout_outcome;

            return rollout_outcome;
        }

        // Expand the tree and rollout from the first child if
        // the node is a leaf node that hasn't been visited yet
        state_node.generate_children();

        // Sync the MCTS tree with the game-state tree
        self.sync_children_count(state_node);

        state_node.children[0].rollout()
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
        latest_unseen_move: usize,
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
            latest_unseen_move: 0,
            mcts_tree: MCTreeNode::new(),
        }
    }

    /// Return a new human agent.
    pub fn new_human() -> Agent {
        Agent::Human
    }

    /// Choose a child of `from_node` to move to. Return the index of that child.
    pub fn make_choice(&mut self, from_node: &mut State, move_history: &Vec<usize>) -> usize {
        match self {
            Agent::Ai { .. } => self.ai_choice(from_node, move_history),
            Agent::Human => self.human_choice(from_node),
        }
    }

    /*********        PLAYER LOGIC        *********/

    fn ai_choice(&mut self, state_node: &mut State, move_history: &Vec<usize>) -> usize {
        let start_time = Instant::now();
        let (max_time, temperature, latest_unseen_move, mcts_node) = match self {
            Agent::Ai {
                time_limit,
                temperature,
                latest_unseen_move,
                mcts_tree,
            } => (
                Duration::from_millis(*time_limit),
                *temperature,
                latest_unseen_move,
                mcts_tree,
            ),
            _ => unreachable!(),
        };

        // Update mcts_node to reflect the current game state
        mcts_node.sync_with_walk(&move_history[*latest_unseen_move..]);
        *latest_unseen_move = move_history.len() + 1;

        // Ensure `mcts_node` has all of its direct children
        mcts_node.sync_children_count(state_node);

        // Continue searching until time is up
        while start_time.elapsed() < max_time {
            mcts_node.traverse(state_node, temperature);
        }

        mcts_node.get_best_child_index()
    }

    fn human_choice(&self, _from_node: &mut State) -> usize {
        0
    }
}
