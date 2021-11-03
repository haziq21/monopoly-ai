getRollEffects(
    rollNumber = 1,
    startingPlayers: Player[] = this.players
): GameState[] {
    let children: GameState[] = [];

    // Loop through all possible dice results
    for (let i = 0; i < GameState.significantRolls.length; i++) {
        // Clone the players
        let updatedPlayers = clone(startingPlayers);

        // Update the current player's position
        updatedPlayers[this.board.currentPlayer].position +=
            GameState.significantRolls[i].sum;
        updatedPlayers[this.board.currentPlayer].position %= 36;

        // Check if this roll got doubles
        if (GameState.significantRolls[i].doubles !== null) {
            // Go to jail after three consecutive doubles
            if (rollNumber === 3) {
                // Set current player's position to jail
                updatedPlayers[this.board.currentPlayer].position = 9;
                updatedPlayers[this.board.currentPlayer].inJail = true;

                // Push the new game state to children
                children.push(
                    new GameState(
                        updatedPlayers,
                        clone(this.board),
                        GameState.significantRolls[i].probability
                    )
                );
            }

            // Re-roll if it's doubles (but not three consecutive ones)
            else {
                children = [
                    ...children,
                    // Recurse to re-roll, then concatenate the new game states to children
                    ...this.getRollEffects(rollNumber + 1, updatedPlayers),
                ];
            }
        } else {
            // Push the new game state to children
            children.push(
                new GameState(
                    updatedPlayers,
                    clone(this.board),
                    GameState.significantRolls[i].probability
                )
            );
        }
    }

    return children;
}