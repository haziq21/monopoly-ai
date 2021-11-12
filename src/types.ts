/** Information about a dice roll. */
export interface DiceRoll {
    doubles: number | null;
    probability: number;
    sum: number;
}

/** The color of a property. */
export type PropertyColor =
    | 'brown'
    | 'light blue'
    | 'pink'
    | 'orange'
    | 'red'
    | 'yellow'
    | 'green'
    | 'blue';

/** A property tile on the game board */
export interface Property {
    color: PropertyColor;
    price: number;
    rents: number[];
    rentLevel: number | null;
    owner: number | null;
}

/** Every chance card */
export type chanceCard = 'rentLevelTo1' | 'rentLevelTo5';

/** The game board */
export interface Board {
    properties: Record<number, Property>;
    currentPlayerIndex: number;
    nextMoveIsChance: boolean;
    activeChanceCard: null | chanceCard;
}

/** A player playing the game */
export interface Player {
    position: number;
    balance: number;
    inJail: boolean;
    doublesRolled: number;
    toString(): string;
}
