// This file generates an array of all possible dice rolls

interface Roll {
    sum: number;
    doubles: number | null;
    probability: number;
}

let sigRolls: Roll[] = [];

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
            // Get the index of the roll in sigRolls with the same `doubles`
            const doublePos = sigRolls.findIndex(
                (r) => r.doubles === roll.doubles
            );

            if (doublePos !== -1) {
                // If a roll with the same `doubles` already exists in
                // sigRolls, add the current roll probability (1/36) to
                // the probability of that corresponding roll in sigRoll
                sigRolls[doublePos].probability += roll.probability;
            } else {
                // Push a new Roll object to sigRolls if a roll
                // with the same `doubles` doesn't already exist
                sigRolls.push(roll);
            }
        } else {
            // Get the index of the roll in sigRolls with the same sum
            const sumPos = sigRolls.findIndex((r) => r.sum === roll.sum);

            if (sumPos !== -1) {
                // If a roll with the same sum already exists in
                // sigRolls, add the current roll probability (1/36) to
                // the probability of that corresponding roll in sigRoll
                sigRolls[sumPos].probability += roll.probability;
            } else {
                // Push a new Roll object to sigRolls if a roll
                // with the same sum doesn't already exist
                sigRolls.push(roll);
            }
        }
    }
}

console.log(sigRolls);

// Total probability should be 1
console.log(
    "Total probability:",
    sigRolls.reduce((p, c) => p + c.probability, 0)
);
