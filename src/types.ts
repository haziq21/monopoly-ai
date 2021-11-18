/** Information about a dice roll. */
export interface DiceRoll {
    doubles: number | null;
    probability: number;
    sum: number;
}

/** The color of a property. */
export type PropertyColor =
    | 'brown'
    | 'lightBlue'
    | 'pink'
    | 'orange'
    | 'red'
    | 'yellow'
    | 'green'
    | 'blue';

/** A property tile on the game board */
export interface Property {
    position: number;
    color: PropertyColor;
    price: number;
    rents: number[];
    rentLevel: number | null;
    owner: number | null;
}

/** Every chance card that requires the player to make a choice. */
export type chanceCard =
    | 'rentLvlTo1'
    | 'rentLvlTo5'
    | 'rentLvlIncForSet'
    | 'rentLvlDecForSet'
    | 'rentLvlIncForBoardSide'
    | 'rentLvlDecForBoardSide'
    | 'rentLvlDecForNeighbours'
    | 'bonusForYouAndOpponent'
    | 'swapProperty'
    | 'sendOpponentToJail'
    | 'moveToAnyProperty';

/** The game board */
export interface Board {
    properties: Property[];
    currentPlayerIndex: number;
    nextMoveIsChance: boolean;
    activeChanceCard: null | chanceCard;
    /** Rounds to go until the "ccLvl1Rent" chance card's effect expires */
    ccLvl1Rent: number;
}

/** A player playing the game */
export interface Player {
    position: number;
    balance: number;
    inJail: boolean;
    doublesRolled: number;
    toString(): string;
}
