// Choices:
// - optional reference to parent state
//   -> implement a getter for each field (tedious, messy)
//   -> changed fields stored as vector of Diff enums
//   -> iterate through vec of Diffs everytime a field needs to be retrieved (ineffecient)
// - each property is a Field<T> enum
//   -> implement Field.ref() and Field.value() (tedious, messy)
//   -> state.players.ref(), state.owned_properties.val()

struct State {
    state_type: StateType,
    players: Vec<Player>,
    owned_properties: HashMap<u8, PropertyOwnership>,
    current_player_index: usize,
    next_move_is_chance: bool,
    active_cc: Option<ChanceCard>,
    lvl1rent_cc: u8,
    seen_ccs: Vec<ChanceCard>,
    pub children: Vec<Box<State>>,
}
