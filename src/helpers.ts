import { Player, Property, PropertyColor, DiceRoll } from './types';

export function assert(
    condition: boolean,
    message = 'Assertion failed'
): asserts condition {
    if (!condition) {
        throw message;
    }
}

/** A factory for players */
export function MakePlayers(
    amount: number,
    position = 0,
    balance = 1500,
    inJail = false,
    doublesRolled = 0
): Player[] {
    const players: Player[] = Array(amount);

    for (let i = 0; i < amount; i++) {
        players[i] = {
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

    return players;
}

/** A factory for properties (houses) */
export function PropertyFactory(
    position: number,
    color: PropertyColor,
    price: number,
    rents: number[]
): Property {
    // Construct a property tile
    return { position, color, price, rents, rentLevel: null, owner: null };
}

/** An array of all possible dice rolls. */
export const significantRolls: DiceRoll[] = (() => {
    const sigRolls: DiceRoll[] = [];

    // Loop through all possible dice results
    for (let d1 = 1; d1 <= 6; d1++) {
        for (let d2 = 1; d2 <= 6; d2++) {
            // Results of the current dice roll
            const roll = {
                sum: d1 + d2, // The sum of the dice
                doubles: d1 === d2 ? d1 : null, // The number on each dice if this roll is a double
                probability: 1 / 36 // The probability of this dice roll result
            };

            // Check if this roll was a double
            if (roll.doubles !== null) {
                // Push a new Roll object to sigRolls if a roll
                // with the same `doubles` doesn't already exist
                sigRolls.push(roll);
            } else {
                // Check if there already exists a roll in sigRolls with the same sum
                const sumPos = sigRolls.findIndex((r) => r.sum === roll.sum);

                if (sumPos !== -1) {
                    // If a roll with the same sum already exists in
                    // sigRolls, merge the probabilities of the two rolls
                    sigRolls[sumPos].probability += roll.probability;
                } else {
                    // Push a new Roll object to sigRolls if a roll
                    // with the same sum doesn't already exist
                    sigRolls.push(roll);
                }
            }
        }
    }

    return sigRolls;
})();

/** Probablity of NOT rolling a double in one try */
export const singleProbability = significantRolls
    .filter((r) => r.doubles === null)
    .reduce((p, c) => p + c.probability, 0);
