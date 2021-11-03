// This file generates an array of all possible dice rolls

import { RollByDoubles, RollBySum } from "./types";

/**
 * Generate an array of all possible
 * dice rolls, categorised by their sums.
 */
export function generateSignificantRolls(): RollBySum[] {
    let sigRolls: RollBySum[] = [];

    // Loop through all possible dice results
    for (let d1 = 1; d1 <= 6; d1++) {
        for (let d2 = 1; d2 <= 6; d2++) {
            // Results of the current dice roll
            const roll = {
                sum: d1 + d2, // The sum of the dice
                doubles: d1 === d2 ? d1 : null, // The number on each dice if this roll is a double
                probability: 1 / 36, // The probability of this dice roll result
            };

            // Check if this roll was a double
            if (roll.doubles !== null) {
                // Check if there already exists a roll in sigRolls with the same `doubles`
                const doublePos = sigRolls.findIndex(
                    (r) => r.doubles === roll.doubles
                );

                if (doublePos !== -1) {
                    // If a roll with the same `doubles` already exists in
                    // sigRolls, merge the probabilities of the two rolls
                    sigRolls[doublePos].probability += roll.probability;
                } else {
                    // Push a new Roll object to sigRolls if a roll
                    // with the same `doubles` doesn't already exist
                    sigRolls.push(roll);
                }
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
}

/**
 * Generate an array of all possible dice rolls,
 * categorised by whether the rolls are doubles.
 */
export function generateDoubleRolls(): RollByDoubles[] {
    const sigRolls = generateSignificantRolls();

    // An array of significant rolls that are doubles
    const dblRolls: RollByDoubles[] = sigRolls
        .filter((r) => r.doubles !== null)
        .map((r) => ({
            doubles: r.doubles,
            probability: r.probability,
        }));

    // A single object to store all the non-double rolls
    const nonDblRolls: RollByDoubles = sigRolls
        .filter((r) => r.doubles === null)
        .reduce(
            (p, c) => ({
                doubles: null,
                probability: p.probability + c.probability,
            }),
            { doubles: null, probability: 0 }
        );

    return [...dblRolls, nonDblRolls];
}
