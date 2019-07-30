export type ImmuxDBFindCondition<T> = { [key in keyof T]?: T[key] };

export interface ImmuxDbDocument {
    id?: string;
}

export interface ImmuxDbCollection {
    upsert: (doc: ImmuxDbDocument) => Promise<void>;
    find: <T extends ImmuxDbDocument = ImmuxDbDocument>(
        condition?: ImmuxDBFindCondition<T>
    ) => Promise<T[]>;
    findOne: <T extends ImmuxDbDocument = ImmuxDbDocument>(
        condition?: ImmuxDBFindCondition<T>
    ) => Promise<T | null>;
}

export type ImmuxDbJS = { [collection in string]: ImmuxDbCollection };
