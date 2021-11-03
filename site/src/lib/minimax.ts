import clone from 'just-clone';
import { generateSignificantRolls } from '$lib/precalculatedRolls';

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

			formattedPos = `\x1b[${this.inJail ? 31 : 36}m${formattedPos}\x1b[0m`;

			const formattedBalance = `\x1b[32m$${this.balance.toFixed(2)}\x1b[0m`;

			return `[${formattedPos}] \x1b[33m${this.doublesRolled}\x1b[0mdbls ${formattedBalance}`;
		}
	};
}

export class GameState {
	players: Player[];
	board: Board;
	probability: number;

	static readonly significantRolls = generateSignificantRolls();

	// minimax: () => number[];
	// staticEvaluation: () => number[];

	/** Produce a game state, or a node on the game tree */
	constructor(players: Player[], board: Board, probability = 1) {
		this.players = players;
		this.board = board;
		this.probability = probability;
	}

	/** Return player whose turn it currently is. */
	get currentPlayer(): Player {
		return this.players[this.board.currentPlayer];
	}

	/**
	 * Changes `this.board.currentPlayer` to the
	 * index of the player whose turn it is next.
	 */
	nextPlayer(): void {
		this.board.currentPlayer = (this.board.currentPlayer + 1) % this.players.length;
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
	 * Get child nodes of the current game state that can be
	 * reached by rolling dice. This only affects properties of the
	 * current player and not anything else about the game state.
	 */
	getRollEffects(): GameState[] {
		const children: GameState[] = [];

		// Loop through all possible dice results
		for (let i = 0; i < GameState.significantRolls.length; i++) {
			// Clone the players
			const updatedPlayers = clone(this.players);

			// Update the current player's position
			updatedPlayers[this.board.currentPlayer].position += GameState.significantRolls[i].sum;
			updatedPlayers[this.board.currentPlayer].position %= 36;

			// Get next game state
			const nextState = new GameState(
				updatedPlayers,
				clone(this.board),
				GameState.significantRolls[i].probability
			);

			// Check if the player landed on 'go to jail'
			if (nextState.currentPlayer.position === 27) {
				nextState.sendToJail();
			}
			// Check if this roll got doubles
			else if (GameState.significantRolls[i].doubles !== null) {
				// Increment the doublesRolled counter
				nextState.currentPlayer.doublesRolled += 1;

				// Go to jail after three consecutive doubles
				if (nextState.currentPlayer.doublesRolled === 3) {
					nextState.sendToJail();
				}
			} else {
				// It's the next player's turn now
				nextState.nextPlayer();
			}

			// Push the new game state to children
			children.push(nextState);
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
		while ((lastId = playerIds.pop()) !== undefined && (lastChild = children.pop()) !== undefined) {
			if (lastId in seen) {
				// Merge their probabilities
				seen[lastId].probability += lastChild.probability;
			} else {
				// This is the first child encountered with this id
				seen[lastId] = lastChild;
			}
		}

		return Object.values(seen);
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
		let finalStr = `\x1b[33m${this.probability.toFixed(3)}\x1b[0m probability:\n`;
		finalStr += this.players.map((p) => `${p.toString()}`).join('\n');

		return '\n' + finalStr;
	}
}
