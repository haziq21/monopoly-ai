import { PropertyFactory, MakePlayers } from './helpers';
import { GameState } from './minimax';

// Start the timer
console.time('Simulation time');

// Initialise players
const players = MakePlayers(2);

// Initialise game
let game = new GameState(players, {
    properties: [
        PropertyFactory(1, 'brown', 60, [70, 130, 220, 370, 750]),
        PropertyFactory(3, 'brown', 60, [70, 130, 220, 370, 750]),
        PropertyFactory(5, 'lightBlue', 100, [80, 140, 240, 410, 800]),
        PropertyFactory(6, 'lightBlue', 100, [80, 140, 240, 410, 800]),
        PropertyFactory(8, 'lightBlue', 120, [100, 160, 260, 440, 860]),
        PropertyFactory(10, 'pink', 140, [110, 180, 290, 460, 900]),
        PropertyFactory(12, 'pink', 140, [110, 180, 290, 460, 900]),
        PropertyFactory(13, 'pink', 160, [130, 200, 310, 490, 980]),
        PropertyFactory(14, 'orange', 180, [140, 210, 330, 520, 1000]),
        PropertyFactory(15, 'orange', 180, [140, 210, 330, 520, 1000]),
        PropertyFactory(17, 'orange', 200, [160, 230, 350, 550, 1100]),
        PropertyFactory(19, 'red', 220, [170, 250, 380, 580, 1160]),
        PropertyFactory(21, 'red', 220, [170, 250, 380, 580, 1160]),
        PropertyFactory(22, 'red', 240, [190, 270, 400, 610, 1200]),
        PropertyFactory(23, 'yellow', 260, [200, 280, 420, 640, 1300]),
        PropertyFactory(24, 'yellow', 260, [200, 280, 420, 640, 1300]),
        PropertyFactory(26, 'yellow', 280, [220, 300, 440, 670, 1340]),
        PropertyFactory(28, 'green', 300, [230, 320, 460, 700, 1400]),
        PropertyFactory(30, 'green', 300, [230, 320, 460, 700, 1400]),
        PropertyFactory(31, 'green', 320, [250, 340, 480, 730, 1440]),
        PropertyFactory(33, 'blue', 350, [270, 360, 510, 740, 1500]),
        PropertyFactory(35, 'blue', 400, [300, 400, 560, 810, 1600])
    ],
    currentPlayerIndex: 0,
    nextMoveIsChance: true,
    activeChanceCard: null,
    ccLvl1Rent: 0
});

/** Get all the child nodes reachable within `ply` ply. */
function aggregateChildren(state: GameState, ply: number): GameState[] {
    if (ply < 1) return [state];

    return state
        .getChildren()
        .map((child) => aggregateChildren(child, ply - 1))
        .flat();
}

const children = aggregateChildren(game, 4);

// Log the tree nodes
// console.log(children.map((c) => c.toString()).join('\n'));
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

// Stop the timer
console.timeEnd('Simulation time');
