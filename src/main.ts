import { PropertyFactory, PlayerFactory, assert } from './helpers';
import { GameState } from './minimax';
import { Player } from './types';

// Initialise players
const PLAYER_COUNT = 3;
const players: Player[] = Array(PLAYER_COUNT);
for (let i = 0; i < PLAYER_COUNT; i++) {
    players[i] = PlayerFactory();
}

// Initialise game
let game = new GameState(players, {
    properties: {
        1: PropertyFactory('brown', 60, [70, 130, 220, 370, 750]),
        3: PropertyFactory('brown', 60, [70, 130, 220, 370, 750]),
        5: PropertyFactory('light blue', 100, [80, 140, 240, 410, 800]),
        6: PropertyFactory('light blue', 100, [80, 140, 240, 410, 800]),
        8: PropertyFactory('light blue', 120, [100, 160, 260, 440, 860]),
        10: PropertyFactory('pink', 140, [110, 180, 290, 460, 900]),
        12: PropertyFactory('pink', 140, [110, 180, 290, 460, 900]),
        13: PropertyFactory('pink', 160, [130, 200, 310, 490, 980]),
        14: PropertyFactory('orange', 180, [140, 210, 330, 520, 1000]),
        15: PropertyFactory('orange', 180, [140, 210, 330, 520, 1000]),
        17: PropertyFactory('orange', 200, [160, 230, 350, 550, 1100]),
        19: PropertyFactory('red', 220, [170, 250, 380, 580, 1160]),
        21: PropertyFactory('red', 220, [170, 250, 380, 580, 1160]),
        22: PropertyFactory('red', 240, [190, 270, 400, 610, 1200]),
        23: PropertyFactory('yellow', 260, [200, 280, 420, 640, 1300]),
        24: PropertyFactory('yellow', 260, [200, 280, 420, 640, 1300]),
        26: PropertyFactory('yellow', 280, [220, 300, 440, 670, 1340]),
        28: PropertyFactory('green', 300, [230, 320, 460, 700, 1400]),
        30: PropertyFactory('green', 300, [230, 320, 460, 700, 1400]),
        31: PropertyFactory('green', 320, [250, 340, 480, 730, 1440]),
        33: PropertyFactory('blue', 350, [270, 360, 510, 740, 1500]),
        35: PropertyFactory('blue', 400, [300, 400, 560, 810, 1600])
    },
    currentPlayerIndex: 0,
    nextMoveIsChance: true
});

// Play some turns
// for (let i = 0; i < 10; i++) {
//     const children = game.getChildren();
//     game = children[Math.round(Math.random() * (children.length - 1))];
// }

console.log(
    game
        .getChildren()
        .map((c) => c.toString())
        .join('\n')
);

// let totalProbability = game
//     .getChildren()
//     .reduce((p, c) => p + c.probability, 0);

// // To ignore inaccuracies with floating-point math
// totalProbability = Math.round(totalProbability * 10 ** 10) / 10 ** 10;

const children = game.getChildren();
if (children.every((p) => p.probability === null)) {
    console.log('No probability');
} else {
    console.log(
        `Probability: ${children.reduce((p, c) => {
            assert(c.probability !== null);
            return p + c.probability;
        }, 0)}`
    );
}

// console.assert(
//     totalProbability === 1,
//     `Total probability is not 1 (${totalProbability})`
// );
