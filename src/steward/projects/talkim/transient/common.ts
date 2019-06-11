export const ME_KEY = "me";

interface ObjectWithId {
    id: string;
}

function normalize<T extends ObjectWithId>(array: T[]): { [id: string]: T } {
    const result: { [id: string]: T } = {};
    for (const element of array) {
        result[element.id] = element;
    }
    return result;
}
