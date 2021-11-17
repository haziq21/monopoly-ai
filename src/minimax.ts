import clone from 'just-clone';
import { assert } from './helpers';
import { significantRolls, singleProbability } from './precalculatedRolls';
import { Player, Board, DiceRoll, Property, chanceCard } from './types';

const positions = {
    properties: [
        1, 3, 5, 6, 8, 10, 12, 13, 14, 15, 17, 19, 21, 22, 23, 24, 26, 28, 30,
        31, 33, 35
    ],
    locations: [7, 16, 25, 34],
    chanceCards: [2, 4, 11, 20, 29, 32]
};

export class GameState {
    players: Player[];
    board: Board;
    probability: number | null;

    // minimax: () => number[];
    // staticEvaluation: () => number[];

    /** Produce a game state, or a node on the game tree */
    constructor(
        players: Player[],
        board: Board,
        probability: number | null = 1
    ) {
        this.players = players;
        this.board = board;
        this.probability = probability;
    }

    /** The player whose turn it currently is. */
    get currentPlayer(): Player {
        return this.players[this.board.currentPlayerIndex];
    }

    /** The property which the current player is on. */
    get currentProperty(): Property {
        return this.board.properties[this.currentPlayer.position];
    }

    /**
     * Move the current player by the specified amount of tiles.
     * Sets their `inJail` flag to false if appropriate.
     * Also awards the player $200 for passing 'Go'.
     */
    moveBy(amount: number): void {
        const newPosition = (this.currentPlayer.position + amount) % 36;

        // Set the player's `inJail` flag to false if appropriate
        if (this.currentPlayer.inJail && amount !== 0) {
            this.currentPlayer.inJail = false;
        }

        // Give the player $200 if they pass 'Go'
        if (newPosition < this.currentPlayer.position) {
            this.currentPlayer.balance += 200;
        }

        this.currentPlayer.position = newPosition;
    }

    /**
     * Changes the `currentPlayerIndex` to the index of the
     * player whose turn it is next, but only if the current
     * player didn't roll doubles (in which case it would be
     * their turn again).
     */
    nextPlayer(): void {
        if (this.currentPlayer.doublesRolled === 0) {
            // Change the currentPlayer index to the index of the next player
            this.board.currentPlayerIndex =
                (this.board.currentPlayerIndex + 1) % this.players.length;
        }

        // The player whose turn it is next rolls the dice
        this.board.nextMoveIsChance = true;
    }

    /**
     * Send the current player to jail. Modifies the
     * current player object and the current player index.
     */
    sendToJail(): void {
        // Set current player's position to jail
        this.players[this.board.currentPlayerIndex].position = 9;
        this.players[this.board.currentPlayerIndex].inJail = true;

        // Reset doubles counter
        this.players[this.board.currentPlayerIndex].doublesRolled = 0;

        // It's the next player's turn now
        this.nextPlayer();
    }

    /** Return a clone of the current game state. */
    clone(probability: number | null = null): GameState {
        return new GameState(
            clone(this.players),
            clone(this.board),
            probability
        );
    }

    /** Roll either `tries` times or until we get a double, whichever comes first. */
    static rollForDoubles(tries: number): DiceRoll[] {
        /* 
        Let P(S) be the probability that a double is not attained in one roll.
        Let P(r) be the probability of obtaining this specific dice configuration `r` after one roll.
        The return value of `significantRolls()` demonstrates all possible "specific dice configurations".

        When rolling the dice for maximum of `n` times, or stopping
        when we get doubles, the probabilities work out as follows:

        The probability of the final roll being any double `d` (where the sum
        of the dice is `2d`) is given by `sum_(i=0)^(n-1) P(r) * P(S)^i`.
        
        The probability of all `n` rolls being non-doubles is given by `P(r) * P(S)^(n - 1)`.
        
        The following code implements this.
        */
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
     * reached by rolling to a chance card tile.
     */
    getRollToChanceCardEffects(): [GameState[], number] {
        const children: GameState[] = [];

        // Chance card: -$50 per property owned
        const propertyPenalty = this.clone(this.probability! / 21);
        let propertyPenaltyIsDifferent = false;

        // Deduct $50 per property owned
        for (let i in propertyPenalty.board.properties) {
            if (
                propertyPenalty.board.properties[i].owner ===
                propertyPenalty.board.currentPlayerIndex
            ) {
                propertyPenalty.currentPlayer.balance -= 50;
                propertyPenaltyIsDifferent = true;
            }
        }

        // Check if the chance card had any effect
        if (propertyPenaltyIsDifferent) {
            propertyPenalty.nextPlayer();
            children.push(propertyPenalty);
        }

        // Chance card: Pay level 1 rent for 2 rounds
        // TODO

        // Chance card: Move all players not in jail to free parking
        const allToParking = this.clone(this.probability! / 21);
        let allToParkingIsDifferent = false;

        for (let i = 0; i < allToParking.players.length; i++) {
            if (!allToParking.players[i].inJail) {
                allToParking.players[i].position = 18;
                allToParkingIsDifferent = true;
            }
        }

        // Check if the chance card had any effect
        if (allToParkingIsDifferent) {
            allToParking.nextPlayer();
            children.push(allToParking);
        }

        // Chance cards that require the player to make a choice
        const choicefulChanceCards: [number, chanceCard][] = [
            [3, 'rentLvlTo1'],
            [1, 'rentLvlTo5']
        ];

        // Push the child states for all the choiceful chance cards
        for (const [amount, id] of choicefulChanceCards) {
            const card = this.clone(this.probability! * (amount / 21));
            card.board.activeChanceCard = id;
            card.board.nextMoveIsChance = false;
            children.push(card);
        }

        const totalChildrenProbability = children
            .map((c) => c.probability)
            .reduce((p, c) => p! + c!, 0)!;

        return [children, this.probability! - totalChildrenProbability];
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
                // Now the player has to do something according to the tile they're on
                newState.board.nextMoveIsChance = false;

                // TODO: Refactor this to compress possibilities of choice-less chance cards

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
                }

                // Check if the player landed on a chance card tile
                if (
                    positions.chanceCards.includes(
                        nextState.currentPlayer.position
                    )
                ) {
                    const [childStates, choicefulProbability] =
                        nextState.getRollToChanceCardEffects();
                    children.push(...childStates);
                    nextState.probability = choicefulProbability;
                } else {
                    // Now the player has to do something according to the tile they're on
                    nextState.board.nextMoveIsChance = false;
                }

                // Push the new game state to children
                children.push(nextState);
            }
        }

        // Get an id representing each player. Players with
        // the exact same state should have the exact same id.
        const playerIds = children.map(
            (gs) =>
                // `this.board.currentPlayerIndex` is the index of the player who *just* moved.
                // `gs.board.currentPlayerIndex` could be the index of the next player to move
                // (after the one that just moved), meaning they could have not yet moved.
                // Hence, we use `this.board.currentPlayerIndex` to get the id of the player
                // who just moved (instead of `gs.board.currentPlayer`) because we want to
                // sieve out / merge the players who moved to the exact same place with the
                // exact same state using different means / methods.
                JSON.stringify(gs.players[this.board.currentPlayerIndex]) +
                JSON.stringify(gs.board.properties) +
                JSON.stringify(gs.board.activeChanceCard)
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
                const temp: GameState = seen[lastId];

                // Merge their probabilities
                temp.probability! += lastChild.probability!;
            } else {
                // This is the first child encountered with this id
                seen[lastId] = lastChild;
            }
        }

        // Return de-duplicated nodes
        return Object.values(seen);
    }

    getPropertyChoiceEffects(): GameState[] {
        // The player can choose to buy the property
        if (this.currentProperty.owner === null) {
            // Choose not to buy this property
            const noBuy = this.clone();
            // TODO: Implement auctioning

            // Choose to buy this property
            const buyProp = this.clone();
            buyProp.currentProperty.owner = buyProp.board.currentPlayerIndex;
            buyProp.currentProperty.rentLevel = 1;
            buyProp.currentPlayer.balance -= buyProp.currentProperty.price;

            return [noBuy, buyProp];
        }
        // The rent level increases because the property is owned by this player
        else if (this.currentProperty.owner === this.board.currentPlayerIndex) {
            const newState = this.clone();

            newState.currentProperty.rentLevel = Math.min(
                newState.currentProperty.rentLevel! + 1,
                5
            );

            return [newState];
        }
        // The player has to pay rent because it's someone else's property
        else {
            const newState = this.clone();

            const balanceDue =
                newState.currentProperty.rents[
                    newState.currentProperty.rentLevel!
                ];

            // Pay the owner...
            newState.players[newState.currentProperty.owner!].balance +=
                balanceDue;

            // ...using the current player's money
            newState.currentPlayer.balance -= balanceDue;

            // Then increase the rent level
            newState.currentProperty.rentLevel = Math.min(
                newState.currentProperty.rentLevel! + 1,
                5
            );

            return [newState];
        }
    }

    getLocationChoiceEffects(): GameState[] {
        const children: GameState[] = [];

        for (let i = 0; i < positions.properties.length; i++) {
            const newState = this.clone();

            // Player can teleport to any property on the board
            newState.currentPlayer.position = positions.properties[i];

            // Effects of landing on the property
            children.push(...newState.getPropertyChoiceEffects());
        }

        return children;
    }

    /**
     * Effects of the chance cards that require the player to make a choice.
     * This is an object because namespaces don't work in classes.
     */
    chanceCards: Record<chanceCard, () => GameState[]> = {
        rentLvlTo1: (): GameState[] => {
            const children: GameState[] = [];

            for (let i in this.board.properties) {
                const rentLevel = this.board.properties[i].rentLevel;

                // Don't need to add another child node if the rent level is already 1
                if (rentLevel !== null && rentLevel > 1) {
                    const child = this.clone();
                    child.board.properties[i].rentLevel = 1;
                    children.push(child);
                }
            }

            return children.length ? children : [this.clone()];
        },

        rentLvlTo5: (): GameState[] => {
            const children: GameState[] = [];

            for (let i in this.board.properties) {
                const rentLevel = this.board.properties[i].rentLevel;

                // Don't need to add another child node if the rent level is already 5
                if (rentLevel !== null && rentLevel < 5) {
                    const child = this.clone();
                    child.board.properties[i].rentLevel = 5;
                    children.push(child);
                }
            }

            return children.length ? children : [this.clone()];
        },

        rentLvlIncForSet: (): GameState[] => {
            return [];
        },

        rentLvlDecForSet: (): GameState[] => {
            return [];
        },

        rentLvlIncForBoardSide: (): GameState[] => {
            return [];
        },

        rentLvlDecForBoardSide: (): GameState[] => {
            return [];
        },

        rentLvlDecForNeighbours: (): GameState[] => {
            return [];
        },

        swapProperty: (): GameState[] => {
            return [];
        },

        sendOpponentToJail: (): GameState[] => {
            return [];
        },

        moveToAnyProperty: (): GameState[] => {
            return [];
        }
    };

    getChanceCardEffects(): GameState[] {
        // Get child states according to currently active chance card
        const children = this.chanceCards[this.board.activeChanceCard!]();
        // Reset active chance card
        children.forEach((c) => (c.board.activeChanceCard = null));

        return children;
    }

    getChoiceEffects(): GameState[] {
        let children: GameState[] = [];

        // The player landed on a location tile
        if (positions.locations.includes(this.currentPlayer.position)) {
            children = this.getLocationChoiceEffects();
        }
        // The player landed on a property tile
        else if (positions.properties.includes(this.currentPlayer.position)) {
            children = this.getPropertyChoiceEffects();
        }
        // The player landed on a chance card tile
        else if (positions.chanceCards.includes(this.currentPlayer.position)) {
            children = this.getChanceCardEffects();
        }
        // TODO: Implement the rest
        else {
            let child = this.clone();
            child.nextPlayer();

            children = [child];
        }

        // It's the next player's turn (if this player didn't roll doubles)
        for (let i = 0; i < children.length; i++) {
            children[i].nextPlayer();
        }

        return children;
    }

    /** Get child nodes of the current game state on the game tree */
    getChildren(): GameState[] {
        return this.board.nextMoveIsChance
            ? this.getRollEffects()
            : this.getChoiceEffects();
    }

    toString(): string {
        const probabilityStr =
            this.probability !== null
                ? (this.probability * 100).toFixed(2) + '%'
                : 'null';
        const activeChanceCard = `Active CC: ${this.board.activeChanceCard}\n`;
        const nextMove = this.board.nextMoveIsChance ? 'chance' : 'choice';
        const playerStrTemplate = (p: Player) =>
            this.currentPlayer === p
                ? `${String(p)} < next (\x1b[36m${nextMove}\x1b[0m)`
                : `${String(p)}`;
        const playersStr = this.players.map(playerStrTemplate).join('\n');

        let finalStr = `\nProbability: \x1b[33m${probabilityStr}\x1b[0m\n`;
        if (this.board.activeChanceCard) {
            finalStr += activeChanceCard;
        }
        finalStr += playersStr;

        return finalStr;
    }
}
