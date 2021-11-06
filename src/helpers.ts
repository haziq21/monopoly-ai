import { Player, Property, PropertyColor } from './types';

export function assert(
    condition: boolean,
    message = 'Assertion failed'
): asserts condition {
    if (!condition) {
        throw message;
    }
}

export function PlayerFactory(
    position = 0,
    balance = 1500,
    inJail = false,
    doublesRolled = 0
): Player {
    return {
        position,
        balance,
        inJail,
        doublesRolled,

        toString: function () {
            let formattedPos = this.position.toLocaleString('en-US', {
                minimumIntegerDigits: 2
            });

            formattedPos = `\x1b[${
                this.inJail ? 31 : 36
            }m${formattedPos}\x1b[0m`;

            const formattedBalance = `\x1b[32m$${this.balance.toFixed(
                2
            )}\x1b[0m`;

            return `[${formattedPos}] \x1b[33m${this.doublesRolled}\x1b[0mdbls ${formattedBalance}`;
        }
    };
}

export function PropertyFactory(
    color: PropertyColor,
    price: number,
    rents: number[]
): Property {
    // Construct a property tile
    return {
        color,
        price,
        rents,
        rentLevel: null,
        owner: null
    };
}
