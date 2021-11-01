/** A player playing the game */
interface Player {
    position: number;
    balance: number;
}

/** A tile on the game board */
interface Tile {
    type:
        | "go"
        | "jail"
        | "free parking"
        | "go to jail"
        | "property"
        | "event"
        | "location";
}

/** The game board */
interface Board {
    tiles: Tile[];
}

/** Extra information about the state of the game */
interface StateInfo {
    currentPlayer: number;
    moveIsChance: boolean;
}

/** Produce a game state, or a node on the game tree */
function gameStateFactory(
    players: Player[],
    board: Board,
    stateInfo: StateInfo
) {
    return {
        getChildren: () => {},
        minimax: () => {},
    };
}

// Initialise players
let i = 10;
let players: Player[] = Array(i);
while (i--) players[i] = { position: 0, balance: 0 };

console.log(players);
