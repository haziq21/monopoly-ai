/** Information about a dice roll, categorised by its sum */
export interface RollBySum {
    doubles: number | null;
    probability: number;
    sum: number;
}

/** A property tile on the game board */
export interface PropertyTile {
    type: 'property';
    color: 'green';
    price: number;
    rents: number[];
}

/** A tile on the game board that is not a property */
export interface NonPropertyTile {
    type: 'go' | 'jail' | 'free parking' | 'go to jail' | 'event' | 'location';
}

/** A tile on the game board */
export type Tile = PropertyTile | NonPropertyTile;

/** The game board */
export interface Board {
    tiles: Tile[];
    currentPlayer: number;
    moveIsChance: boolean;
}

/** A player playing the game */
export interface Player {
    position: number;
    balance: number;
    inJail: boolean;
    doublesRolled: number;
    toString(): string;
}
