import { PropertyFactory, MakePlayers, assert } from './helpers';
import { GameState } from './minimax';
import { Player } from './types';

// Initialise players
const players = MakePlayers(2);

// Initialise game
let game = new GameState(players, {
    properties: {
        1: PropertyFactory('brown', 60, [70, 130, 220, 370, 750]),
        3: PropertyFactory('brown', 60, [70, 130, 220, 370, 750]),
        5: PropertyFactory('lightBlue', 100, [80, 140, 240, 410, 800]),
        6: PropertyFactory('lightBlue', 100, [80, 140, 240, 410, 800]),
        8: PropertyFactory('lightBlue', 120, [100, 160, 260, 440, 860]),
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
    nextMoveIsChance: true,
    activeChanceCard: null
});

// Play some turns
// for (let i = 0; i < 10; i++) {
//     const children = game.getChildren();
//     game = children[Math.round(Math.random() * (children.length - 1))];
// }

game = game.getChildren()[36];
const children = game.getChildren();

console.log(children.map((c) => c.toString()).join('\n'));
console.log(`\n${children.length} child states`);

// Log the total probability
if (children.every((p) => p.probability === null)) {
    console.log('No total probability');
} else {
    let totalProbability = children.reduce((p, c) => {
        return p + c.probability!;
    }, 0);

    // To ignore inaccuracies with floating-point math
    totalProbability = Math.round(totalProbability * 10 ** 10) / 10 ** 10;
    console.log(`Total probability: ${totalProbability}`);
}
