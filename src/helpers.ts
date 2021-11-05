import { Property, PropertyColor } from './types';

export function assert(
    condition: boolean,
    message = 'Assertion failed'
): asserts condition {
    if (!condition) {
        throw message;
    }
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
