import clone from 'just-clone';
import { significantRolls, singleProbability } from './precalculatedRolls';
import { Player, Board, RollBySum } from './types';

export function PlayerFactory(
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
            let formattedPos = this.position.toLocaleString('en-US', {
                minimumIntegerDigits: 2
            });

            formattedPos = `\x1b[${
                this.inJail ? 31 : 36
            }m${formattedPos}\x1b[0m`;

            const formattedBalance = `\x1b[32m$${this.balance.toFixed(
                2
            )}\x1b[0m`;

            return `[${formattedPos}] \x1b[33m${this.doublesRolled}\x1b[0mdbls ${formattedBalance}`;
        }
    };
}

export class GameState {
    players: Player[];
    board: Board;
    probability: number;

    // minimax: () => number[];
    // staticEvaluation: () => number[];

    /** Produce a game state, or a node on the game tree */
    constructor(players: Player[], board: Board, probability = 1) {
        this.players = players;
        this.board = board;
        this.probability = probability;
    }

    /** The player whose turn it currently is. */
    get currentPlayer(): Player {
        return this.players[this.board.currentPlayer];
    }

    /**
     * Move the current player by the specified amount of tiles.
     * Also awards the player $200 for passing 'Go'.
     */
    moveBy(amount: number) {
        const newPosition = (this.currentPlayer.position + amount) % 36;

        // Give the player $200 if they pass 'Go'
        if (newPosition < this.currentPlayer.position) {
            this.currentPlayer.balance += 200;
        }

        this.currentPlayer.position = newPosition;
    }

    /**
     * Changes `this.board.currentPlayer` to the
     * index of the player whose turn it is next.
     */
    nextPlayer(): void {
        this.board.currentPlayer =
            (this.board.currentPlayer + 1) % this.players.length;
    }

    /**
     * Send the current player to jail. Modifies the
     * current player object and the current player index.
     */
    sendToJail(): void {
        // Set current player's position to jail
        this.players[this.board.currentPlayer].position = 9;
        this.players[this.board.currentPlayer].inJail = true;

        // Reset doubles counter
        this.players[this.board.currentPlayer].doublesRolled = 0;

        // It's the next player's turn now
        this.nextPlayer();
    }

    /**
     * Return a clone of the current game state.
     */
    clone(probability = 1): GameState {
        return new GameState(
            clone(this.players),
            clone(this.board),
            probability
        );
    }

    /** Roll either `tries` times or until we get a double, whichever comes first. */
    static rollForDoubles(tries: number): RollBySum[] {
        return significantRolls.map((roll) => {
            let totalProbability: number;

            // Rolled a double
            if (roll.doubles !== null) {
                // The probability of getting this specific double
                let doubleProbability = 0;

                for (let i = 0; i < tries; i++) {
                    doubleProbability +=
                        roll.probability * singleProbability ** i;
                }

                totalProbability = doubleProbability;
            }

            // Didn't roll a double
            else {
                totalProbability =
                    roll.probability * singleProbability ** (tries - 1);
            }

            return {
                doubles: roll.doubles,
                sum: roll.sum,
                probability: totalProbability
            };
        });
    }

    /**
     * Get child nodes of the current game state that can be
     * reached by rolling dice. This only affects properties of the
     * current player and not anything else about the game state.
     */
    getRollEffects(): GameState[] {
        const children: GameState[] = [];

        // Try getting out of jail if the player is in jail
        if (this.currentPlayer.inJail) {
            // Try rolling doubles to get out of jail
            const doubleProbabilities = GameState.rollForDoubles(3);

            // Loop through all possible dice results
            for (let dbl = 0; dbl < doubleProbabilities.length; dbl++) {
                // Derive a new game state from the current game state
                const newState = this.clone(
                    doubleProbabilities[dbl].probability
                );

                // Update the current player's position
                newState.moveBy(doubleProbabilities[dbl].sum);
                newState.currentPlayer.inJail = false;
                // Now the player has to do something according to the tile they're on
                newState.board.moveIsChance = false;

                // We didn't manage to roll doubles
                if (doubleProbabilities[dbl].doubles === null) {
                    // $100 penalty for not rolling doubles
                    newState.currentPlayer.balance -= 100;
                }

                // Push the updated state to children
                children.push(newState);
            }
        }

        // Otherwise, play as normal
        else {
            // Loop through all possible dice results
            for (let i = 0; i < significantRolls.length; i++) {
                // Derive a new game state from the current game state
                const nextState = this.clone(significantRolls[i].probability);

                // Update the current player's position
                nextState.moveBy(significantRolls[i].sum);

                // Check if the player landed on 'go to jail'
                if (nextState.currentPlayer.position === 27) {
                    nextState.sendToJail();
                }
                // Check if this roll got doubles
                else if (significantRolls[i].doubles !== null) {
                    // Increment the doublesRolled counter
                    nextState.currentPlayer.doublesRolled += 1;

                    // Go to jail after three consecutive doubles
                    if (nextState.currentPlayer.doublesRolled === 3) {
                        nextState.sendToJail();
                    }
                } else {
                    // Reset doubles counter
                    nextState.currentPlayer.doublesRolled = 0;

                    // Now the player has to do something according to the tile they're on
                    nextState.board.moveIsChance = false;
                }

                // Push the new game state to children
                children.push(nextState);
            }
        }

        // Get an id representing each player. Players with
        // the exact same state should have the exact same id.
        const playerIds = children.map((gs) =>
            // `this.board.currentPlayer` is the index of the player who *just* moved.
            // `gs.board.currentPlayer` could be the index of the next player to move
            // (after the one that just moved), meaning they could have not yet moved.
            // Hence, we use `this.board.currentPlayer` to get the id of the player
            // who just moved (instead of `gs.board.currentPlayer`) because we want to
            // sieve out / merge the players who moved to the exact same place with the
            // exact same state using different means / methods.
            JSON.stringify(gs.players[this.board.currentPlayer])
        );

        // Store non-duplicate nodes here
        const seen: Record<string, GameState> = {};
        let lastId: string | undefined;
        let lastChild: GameState | undefined;

        // Merge duplicate nodes
        while (
            (lastId = playerIds.pop()) !== undefined &&
            (lastChild = children.pop()) !== undefined
        ) {
            if (lastId in seen) {
                // Merge their probabilities
                seen[lastId].probability += lastChild.probability;
            } else {
                // This is the first child encountered with this id
                seen[lastId] = lastChild;
            }
        }

        // Return de-duplicated nodes
        return Object.values(seen);
    }

    getChoiceEffects(): GameState[] {
        let child = this.clone();
        child.nextPlayer();
        child.board.moveIsChance = true;

        return [child];
    }

    /** Get child nodes of the current game state on the game tree */
    getChildren(): GameState[] {
        return this.board.moveIsChance
            ? this.getRollEffects()
            : this.getChoiceEffects();
    }

    toString(): string {
        let finalStr = `Probability: \x1b[33m${(this.probability * 100).toFixed(
            2
        )}%\x1b[0m\n`;
        finalStr += `Next move: ${
            this.board.moveIsChance ? 'chance' : 'choice'
        }\n`;
        finalStr += this.players.map((p) => `${p.toString()}`).join('\n');

        return '\n' + finalStr;
    }
}
