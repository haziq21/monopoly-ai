import clone from "just-clone";

/** Information about a dice roll */
interface Roll {
    sum: number;
    doubles: number | null;
    probability: number;
}

/** A player playing the game */
interface Player {
    position: number;
    balance: number;
    inJail: boolean;
    doublesRolled: number;
    toString(): string;
}

function PlayerFactory(
    position = 0,
    balance = 1500,
    inJail = false,
    doublesRolled = 0
): Player {
    return {
        position,
        balance,
        inJail,
        doublesRolled,

        toString: function () {
            let formattedPos = this.position.toLocaleString("en-US", {
                minimumIntegerDigits: 2,
            });

            formattedPos = `\x1b[${
                this.inJail ? 31 : 36
            }m${formattedPos}\x1b[0m`;

            const formattedBalance = `\x1b[32m$${this.balance.toFixed(
                2
            )}\x1b[0m`;

            return `[${formattedPos}] \x1b[33m${this.doublesRolled}\x1b[0mdbls ${formattedBalance}`;
        },
    };
}

/** A property tile on the game board */
interface PropertyTile {
    type: "property";
    color: "green";
    price: number;
    rents: number[];
}

/** A tile on the game board that is not a property */
interface NonPropertyTile {
    type: "go" | "jail" | "free parking" | "go to jail" | "event" | "location";
}

/** A tile on the game board */
type Tile = PropertyTile | NonPropertyTile;

/** The game board */
interface Board {
    tiles: Tile[];
    currentPlayer: number;
    moveIsChance: boolean;
}

/** The current state of the game represented by a node on the game tree */
class GameState {
    players: Player[];
    board: Board;
    probability: number;
    // Hard-coded results of all possible dice rolls
    static readonly significantRolls: Roll[] = [
        { sum: 2, doubles: 1, probability: 0.027777777777777776 },
        { sum: 3, doubles: null, probability: 0.05555555555555555 },
        { sum: 4, doubles: null, probability: 0.05555555555555555 },
        { sum: 5, doubles: null, probability: 0.1111111111111111 },
        { sum: 6, doubles: null, probability: 0.1111111111111111 },
        { sum: 7, doubles: null, probability: 0.16666666666666669 },
        { sum: 4, doubles: 2, probability: 0.027777777777777776 },
        { sum: 8, doubles: null, probability: 0.1111111111111111 },
        { sum: 6, doubles: 3, probability: 0.027777777777777776 },
        { sum: 9, doubles: null, probability: 0.1111111111111111 },
        { sum: 8, doubles: 4, probability: 0.027777777777777776 },
        { sum: 10, doubles: null, probability: 0.05555555555555555 },
        { sum: 10, doubles: 5, probability: 0.027777777777777776 },
        { sum: 11, doubles: null, probability: 0.05555555555555555 },
        { sum: 12, doubles: 6, probability: 0.027777777777777776 },
    ];

    // minimax: () => number[];
    // staticEvaluation: () => number[];

    /** Produce a game state, or a node on the game tree */
    constructor(players: Player[], board: Board, probability = 1) {
        this.players = players;
        this.board = board;
        this.probability = probability;
    }

    /** Return the index of the player whose turn it is next. */
    getNextPlayer(): number {
        return (this.board.currentPlayer + 1) % this.players.length;
    }

    /**
     * Get child nodes of the current game state that can be
     * reached by rolling dice. This only affects properties of the
     * current player and not anything else about the game state.
     */
    getRollEffects(): GameState[] {
        let children: GameState[] = [];

        // Loop through all possible dice results
        for (let i = 0; i < GameState.significantRolls.length; i++) {
            // Clone the players
            let updatedPlayers = clone(this.players);

            // Update the current player's position
            updatedPlayers[this.board.currentPlayer].position +=
                GameState.significantRolls[i].sum;
            updatedPlayers[this.board.currentPlayer].position %= 36;

            // Clone the board
            let updatedBoard = clone(this.board);

            // Check if this roll got doubles
            if (GameState.significantRolls[i].doubles !== null) {
                // Increment the doublesRolled counter
                updatedPlayers[this.board.currentPlayer].doublesRolled += 1;

                // Go to jail after three consecutive doubles
                if (
                    updatedPlayers[this.board.currentPlayer].doublesRolled === 3
                ) {
                    // Set current player's position to jail
                    updatedPlayers[this.board.currentPlayer].position = 9;
                    updatedPlayers[this.board.currentPlayer].inJail = true;

                    // Reset counter
                    updatedPlayers[this.board.currentPlayer].doublesRolled = 0;

                    // It's the next player's turn now
                    updatedBoard.currentPlayer =
                        (updatedBoard.currentPlayer + 1) %
                        updatedPlayers.length;
                }
            } else {
                updatedBoard.currentPlayer =
                    (updatedBoard.currentPlayer + 1) % updatedPlayers.length;
            }

            // Push the new game state to children
            children.push(
                new GameState(
                    updatedPlayers,
                    updatedBoard,
                    GameState.significantRolls[i].probability
                )
            );
        }

        return children;
    }

    /** Get child nodes of the current game state on the game tree */
    getChildren(): GameState[] {
        let children: GameState[] = [];

        // The next move to be made is a dice roll
        if (this.board.moveIsChance) {
            children = this.getRollEffects();
        }

        return children;
    }

    toString(): string {
        let finalStr = `\x1b[33m${this.probability.toFixed(3)}\x1b[0m prob.\n`;
        finalStr += this.players.map((p) => `${p.toString()}`).join("\n");

        return "\n" + finalStr;
    }
}

// Initialise players
let playerCount = 2;
let players: Player[] = Array(playerCount);
while (playerCount--) players[playerCount] = PlayerFactory();

// Initialise game
let game: GameState = new GameState(players, {
    tiles: [],
    currentPlayer: 0,
    moveIsChance: true,
});

game = game.getChildren()[12];
game = game.getChildren()[12];

console.log(
    game
        .getChildren()
        .map((c) => c.toString())
        .join("\n")
);
