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

/** The game board */
export interface Board {
    properties: Record<number, Property>;
    currentPlayerIndex: number;
    nextMoveIsChance: boolean;
}

/** A player playing the game */
export interface Player {
    position: number;
    balance: number;
    inJail: boolean;
    doublesRolled: number;
    toString(): string;
}
