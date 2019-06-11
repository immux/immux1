export const FOLDR_PROJECT_NAME = "foldr";

export type MapById<EntityType> = {
    [id: string]: EntityType;
};

interface ObjectWithId {
    id: string;
}

export function makeNamespaceForProject(projectName: string): string {
    return `project-${projectName}`;
}

export function normalizeById<T extends ObjectWithId>(arr: T[]): MapById<T> {
    const result: MapById<T> = {};
    for (const element of arr) {
        result[element.id] = element;
    }
    return result;
}

export function mergeArray<T extends ObjectWithId>(
    base: MapById<T>,
    data: T[]
): MapById<T> {
    if (data.length > 0) {
        return {
            ...base,
            ...normalizeById(data)
        };
    } else {
        return base;
    }
}
