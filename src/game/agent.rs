use super::globals::*;
use super::Game;
use rand::Rng;
use std::iter::zip;
use std::time::{Duration, Instant};

use super::state_diff::BranchType;

/// An MTCS tree is essentially a mirror copy of the game tree,
/// except with property + auction states combined into one node.
pub struct MCTreeNode {
    total_value: f64,
    num_visits: u32,
    branch_type: BranchType,
    children: Vec<Box<MCTreeNode>>,
}

impl MCTreeNode {
    /// Return a new MCTS node with `t` and `n` set to 0.
    fn new(branch_type: BranchType) -> MCTreeNode {
        MCTreeNode {
            total_value: 0.,
            num_visits: 0,
            branch_type,
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
    fn sync_children_count(&mut self, game: &mut Game, handle: usize) {
        let mctree_children_count = self.children.len();
        let count = game.nodes[handle].children.len();

        if mctree_children_count == count {
            return;
        }

        if mctree_children_count != 0 {
            panic!(
                "MCTreeNode::sync_children_count() - mctree_children_count == {}",
                mctree_children_count
            );
        }

        for i in 0..count {
            let bt = game.nodes[game.nodes[handle].children[i]].branch_type;
            self.children.push(Box::new(MCTreeNode::new(bt)));
        }
    }

    /// Traverse the tree according to the indexes in `walk`.
    /// Replace this node with the node at the end of the traversal.
    fn sync_with_walk(&mut self, game: &mut Game, latest_unseen_move: usize) {
        for &step in &game.move_history[latest_unseen_move..] {
            if self.children.len() == 0 {
                let ending_node = &game.nodes[game.root_handle];
                *self = MCTreeNode::new(ending_node.branch_type);
                break;
            }

            *self = std::mem::replace(
                self.children[step].as_mut(),
                MCTreeNode::new(BranchType::Choice),
            );
        }
    }

    /// Traverse the MCTS tree and create child nodes as needed. Return rollout result.
    fn traverse(&mut self, game: &mut Game, handle: usize, pindex: usize, temperature: f64) -> f64 {
        let value_multiplier = match self.branch_type {
            BranchType::Chance(p) => p,
            _ => 1.,
        };

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

            let next_handle = game.nodes[handle].children[child_index];

            // Value of the rollout to propagate
            let propagated_value =
                self.children[child_index].traverse(game, next_handle, pindex, temperature);

            // Update n and t
            self.num_visits += 1;
            self.total_value += propagated_value * value_multiplier;

            return propagated_value;
        }

        // Perform a rollout if the node has never been visited before
        if self.num_visits == 0 {
            let rollout_outcome = MCTreeNode::rollout(game, handle, pindex);

            // Update n and t
            self.num_visits += 1;
            self.total_value += rollout_outcome * value_multiplier;

            return rollout_outcome;
        }

        // We can't generate any more child states if we're at a terminal game state
        if game.is_terminal(handle) {
            return MCTreeNode::rollout(game, handle, pindex) * value_multiplier;
        }

        // Expand the tree and rollout from the first child if
        // the node is a leaf node that hasn't been visited yet
        game.gen_children_save(handle);

        // Sync the MCTS tree with the game-state tree
        self.sync_children_count(game, handle);

        MCTreeNode::rollout(game, game.nodes[handle].children[0], pindex) * value_multiplier
    }

    fn rollout(game: &mut Game, mut handle: usize, pindex: usize) -> f64 {
        let mut rng = rand::thread_rng();

        // Play the game randomly until game-over
        while !game.is_terminal(handle) {
            game.gen_children_save(handle);
            let first_child_i = game.nodes[handle].children[0];

            match game.nodes[first_child_i].branch_type {
                BranchType::Chance(_) => {
                    let child_index = game.get_any_chance_child(handle);
                    handle = game.nodes[handle].children[child_index];
                }
                BranchType::Choice => {
                    let children = &game.nodes[handle].children;
                    handle = children[rng.gen_range(0..children.len())];
                }
                BranchType::Undefined => unreachable!(),
            }
        }

        // Tabulate everyone's balances
        let player_balances = game.diff_players(handle).iter().map(|p| p.balance as f64);

        // Tabulate everyone's property worths
        let props = game.diff_owned_properties(handle);
        let mut total_prop_worths = vec![0.; game.get_player_count()];
        for (pos, prop) in props {
            total_prop_worths[prop.owner] += PROPERTIES[pos].price as f64;
        }

        let scores: Vec<f64> = zip(player_balances, total_prop_worths)
            .map(|(balance, prop_worth)| balance * prop_worth)
            .collect();
        let mean_score: f64 = scores.iter().sum::<f64>() / scores.len() as f64;

        // The value of the game state is calculated as a player's distance from the mean balance
        scores[pindex] - mean_score
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
        /// Index of this agent in `Game.agents`.
        index: usize,
        /// Index of the last move that this agent played, from `Game.move_history`.
        latest_unseen_move: usize,
        /// The Monte-Carlo search tree associated with this AI.
        mcts_tree: MCTreeNode,
    },
    /// A physical human player.
    Human,
    /// An agent that plays randomly
    Random,
}

impl Agent {
    /*********        PUBLIC INTERFACES        *********/

    /// Return a new AI agent.
    pub fn new_ai(time_limit: u64, temperature: f64, index: usize) -> Agent {
        Agent::Ai {
            time_limit,
            temperature,
            index,
            latest_unseen_move: 0,
            mcts_tree: MCTreeNode::new(BranchType::Choice),
        }
    }

    /// Return a new human agent.
    pub fn new_human() -> Agent {
        Agent::Human
    }

    /// Return an agent that plays randomly.
    pub fn new_random() -> Agent {
        Agent::Random
    }

    /// Choose a child of `from_node` to move to. Return the index of that child.
    pub fn make_choice(&mut self, game: &mut Game) -> usize {
        match self {
            Agent::Ai { .. } => self.ai_choice(game),
            Agent::Human => self.human_choice(game),
            Agent::Random => self.random_choice(game),
        }
    }

    /*********        PLAYER LOGIC        *********/

    fn ai_choice(&mut self, game: &mut Game) -> usize {
        let start_time = Instant::now();

        // Extract relevant fields from agent
        let (max_time, temperature, agent_index, latest_unseen_move, mcts_node) = match self {
            Agent::Ai {
                time_limit,
                temperature,
                index,
                latest_unseen_move,
                mcts_tree,
            } => (
                Duration::from_millis(*time_limit),
                *temperature,
                *index,
                latest_unseen_move,
                mcts_tree,
            ),
            _ => unreachable!(),
        };

        // Update mcts_node to reflect the current game state
        mcts_node.sync_with_walk(game, *latest_unseen_move);
        // Set the lastest unseen move to the move after this one
        *latest_unseen_move = game.move_history.len();

        // Ensure `mcts_node` has all of its direct children
        game.gen_children_save(game.root_handle);
        mcts_node.sync_children_count(game, game.root_handle);

        // Continue searching until time is up
        while start_time.elapsed() < max_time
            || mcts_node
                .children
                .iter()
                .any(|n| n.get_average_value().is_nan())
        {
            mcts_node.traverse(game, game.root_handle, agent_index, temperature);
        }

        let p = mcts_node
            .children
            .iter()
            .map(|n| n.get_average_value())
            .collect::<Vec<f64>>();
        println!("{:?}", p);
        mcts_node.get_best_child_index()
    }

    fn human_choice(&self, _game: &mut Game) -> usize {
        0
    }

    fn random_choice(&self, game: &mut Game) -> usize {
        let mut rng = rand::thread_rng();
        game.gen_children_save(game.root_handle);
        rng.gen_range(0..game.nodes[game.root_handle].children.len())
    }
}
