/** Information about a dice roll, categorised by whether it's a double */
interface RollByDoubles {
    doubles: number | null;
    probability: number;
}

/** Information about a dice roll, categorised by its sum */
interface RollBySum extends RollByDoubles {
    sum: number;
}

/** A property tile on the game board */
interface PropertyTile {
    type: 'property';
    color: 'green';
    price: number;
    rents: number[];
}

/** A tile on the game board that is not a property */
interface NonPropertyTile {
    type: 'go' | 'jail' | 'free parking' | 'go to jail' | 'event' | 'location';
}

/** A tile on the game board */
type Tile = PropertyTile | NonPropertyTile;

/** The game board */
interface Board {
    tiles: Tile[];
    currentPlayer: number;
    moveIsChance: boolean;
}

/** A player playing the game */
interface Player {
    position: number;
    balance: number;
    inJail: boolean;
    doublesRolled: number;
    toString(): string;
}
