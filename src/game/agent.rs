use super::state::State;
use std::cell::RefCell;
use std::iter::zip;
use std::time::{Duration, Instant};

struct MCTreeNode<'a> {
    total_value: f64,
    num_visits: u32,
    childen: Vec<Box<MCTreeNode<'a>>>,
    corresponding_state: &'a RefCell<State>,
}

impl MCTreeNode<'_> {
    fn new(corresponding_state: &RefCell<State>) -> MCTreeNode {
        MCTreeNode {
            total_value: 0.,
            num_visits: 0,
            childen: vec![],
            corresponding_state,
        }
    }

    fn sync_with_state(&mut self, state: &RefCell<State>, walk: &Vec<usize>) {
        // for step in walk {
        //     if self.childen.len() > 0 {
        //         self = &mut *self.childen[*step];
        //     } else {
        //         self = &mut MCTreeNode::new(state);
        //         break;
        //     }
        // }
    }
}

/// An agent playing the game, or the "brains" of a player.
pub enum Agent<'a> {
    /// An MCTS AI agent.
    Ai {
        /// Amount of time that the AI is given to "think", in milliseconds.
        time_limit: u64,
        /// Value of `C` constant in UCB1 formula.
        temperature: f64,
        /// Root node of the Monte-Carlo tree.
        root_node: MCTreeNode<'a>,
        /// RefCell of the game's `move_history`.
        move_history: &'a RefCell<Vec<usize>>,
        /// Index of the last move that this agent played, from `game.move_history`.
        last_move_index: usize,
    },
    /// A physical human player.
    Human,
}

impl Agent<'_> {
    /*********        PUBLIC INTERFACES        *********/

    /// Return a new AI agent.
    pub fn new_ai<'a>(
        current_state: &'a RefCell<State>,
        move_history: &'a RefCell<Vec<usize>>,
    ) -> Agent<'a> {
        Agent::Ai {
            time_limit: 1000,
            temperature: 2.,
            root_node: MCTreeNode::new(current_state),
            move_history,
            last_move_index: 0,
        }
    }

    /// Return a new human agent.
    pub fn new_human<'a>() -> Agent<'a> {
        Agent::Human
    }

    /// Return the resulting state after the agent makes a choice.
    pub fn make_choice(&mut self, from_node: &mut State) -> State {
        match self {
            Agent::Ai { .. } => self.ai_choice(from_node),
            Agent::Human => Agent::human_choice(from_node),
        }
    }

    /*********        HELPER FUNCTIONS        *********/

    fn get_temp(&self) -> f64 {
        match self {
            Agent::Ai { temperature, .. } => *temperature,
            _ => unreachable!(),
        }
    }

    /*********        DECISION MAKING        *********/

    fn ai_choice(&mut self, from_node: &mut State) -> State {
        let start = Instant::now();
        let (max_time, temp, root) = match self {
            Agent::Ai {
                time_limit,
                temperature,
                root_node,
                ..
            } => (Duration::from_millis(*time_limit), *temperature, root_node),
            _ => unreachable!(),
        };

        loop {
            if start.elapsed() >= max_time {
                break;
            }
        }

        State::new(2)
    }

    // fn mcts_traverse(&self, from_node: &mut MCTreeNode) -> u8 {
    //     // If from_node is not a leaf node, calculate the UCB1 values of its child nodes
    //     if from_node.children.len() > 0 {
    //         // The UCB1 formula is `V_i + C * sqrt( ln(N) / n_i )`

    //         // mean_value = V_i
    //         let mean_value = from_node.total_value / from_node.num_visits as f64;
    //         // All the UCB1 values with respect to from_node's children
    //         let ucb1_values: Vec<f64> = from_node
    //             .children
    //             .iter()
    //             .map(|s| {
    //                 if from_node.num_visits == 0 || s.num_visits == 0 {
    //                     f64::INFINITY
    //                 } else {
    //                     mean_value
    //                         + self.get_temp()
    //                             * ((from_node.num_visits as f64).ln() / s.num_visits as f64).sqrt()
    //                 }
    //             })
    //             .collect();

    //         // The child to select next
    //         let interesting_child = zip(ucb1_values, &mut from_node.children)
    //             .max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
    //             .map(|(_, s)| s)
    //             .unwrap();

    //         return self.mcts_traverse(interesting_child);
    //     };
    // }

    fn human_choice(from_node: &mut State) -> State {
        State::new(2)
    }
}
