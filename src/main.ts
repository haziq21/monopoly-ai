import { GameState, PlayerFactory } from './minimax';
import { Player } from './types';

// Initialise players
const PLAYER_COUNT = 3;
let players: Player[] = Array(PLAYER_COUNT);
for (let i = 0; i < PLAYER_COUNT; i++) {
    players[i] = PlayerFactory();
}

// Initialise game
let game = new GameState(players, {
    tiles: [],
    currentPlayer: 0,
    moveIsChance: true
});

// Play some turns
for (let i = 0; i < 100; i++) {
    const children = game.getChildren();
    game = children[Math.round(Math.random() * (children.length - 1))];
}

console.log(
    game
        .getChildren()
        .map((c) => c.toString())
        .join('\n')
);

let totalProbability = game
    .getChildren()
    .reduce((p, c) => p + c.probability, 0);

// To ignore inaccuracies with floating-point math
totalProbability = Math.round(totalProbability * 10 ** 10) / 10 ** 10;

console.assert(
    totalProbability === 1,
    `Total probability is not 1 (${totalProbability})`
);
