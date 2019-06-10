import {
    ImmuxDbCollection,
    ImmuxDbDocument,
    ImmuxDBFindCondition,
    ImmuxDbJS
} from "./immuxdb.types";

interface ImmuxDBHttp {
    host: string;
    simpleGet(collection: string, key: string): Promise<string>;
    select(collection: string, condition: string): Promise<string>;
    getAtHeight(
        collection: string,
        key: string,
        height: number
    ): Promise<string>;
    set(collection: string, key: string, value: string): Promise<string>;
    revertOne(collection: string, key: string, height: number): Promise<string>;
    revertAll(height: number): Promise<string>;
    readNamespace(): Promise<string>;
    switchNamespace(namespace: string): Promise<string>;
}

export function makeImmuxDBHttp(
    host: string,
    fetch: (path: string, options?: any) => Promise<any>
): ImmuxDBHttp {
    return {
        host,
        async simpleGet(collection: string, key: string) {
            const response = await fetch(
                `http://${this.host}/${collection}/${key}`
            );
            return await response.text();
        },
        async select(collection: string, condition: string) {
            const response = await fetch(
                `http://${this.host}/${collection}/?select=${condition}`
            );
            return await response.text();
        },
        async getAtHeight(collection: string, key: string, height: number) {
            const response = await fetch(
                `http://${this.host}/${collection}/${key}?height=${height}`
            );
            return await response.text();
        },
        async set(collection: string, key: string, value: string) {
            const response = await fetch(
                `http://${this.host}/${collection}/${key}`,
                {
                    method: "PUT",
                    body: value
                }
            );
            return await response.text();
        },
        async revertOne(collection: string, key: string, height: number) {
            const response = await fetch(
                `http://${this.host}/${collection}/${key}?revert=${height}`,
                {
                    method: "PUT"
                }
            );
            return await response.text();
        },
        async revertAll(height: number) {
            const response = await fetch(
                `http://${this.host}/?revert_all=${height}`,
                {
                    method: "PUT"
                }
            );
            return await response.text();
        },
        async readNamespace() {
            const response = await fetch(`http://${this.host}/?chain`);
            return await response.text();
        },
        async switchNamespace(namespace: string) {
            const response = await fetch(
                `http://${this.host}/?chain=${namespace}`,
                {
                    method: "PUT"
                }
            );
            return await response.text();
        }
    };
}

function getJsonReducer<T = any>(prev: T[], s: string): T[] {
    try {
        return [...prev, JSON.parse(s) as T];
    } catch {
        return prev;
    }
}

export function createImmuxDbViaHttpsRestrictedAccess(
    db: ImmuxDBHttp
): ImmuxDbJS {
    return new Proxy<ImmuxDbJS>(
        {},
        {
            get: (_, collectionName) => {
                const collectionObject: ImmuxDbCollection = {
                    upsert: async (doc: ImmuxDbDocument) => {
                        doc.id = doc.id || Math.random().toString();
                        await db.set(
                            collectionName.toString(),
                            doc.id,
                            JSON.stringify(doc)
                        );
                    },
                    find: async <T extends ImmuxDbDocument = ImmuxDbDocument>(
                        condition?: ImmuxDBFindCondition<T>
                    ) => {
                        const result = await db.select(
                            collectionName.toString(),
                            JSON.stringify(condition)
                        );
                        const rows = result.split("\r\n");
                        let data = rows.reduce<T[]>(getJsonReducer, []);
                        if (condition) {
                            data = data.filter(doc => {
                                for (const key in condition) {
                                    if (condition[key] !== doc[key]) {
                                        return false;
                                    }
                                }
                                return true;
                            });
                        }
                        return data;
                    },
                    findOne: async <
                        T extends ImmuxDbDocument = ImmuxDbDocument
                    >(
                        condition?: ImmuxDBFindCondition<T>
                    ) => {
                        const results = await collectionObject.find<T>(
                            condition
                        );
                        if (results[0]) {
                            return results[0];
                        } else {
                            return null;
                        }
                    }
                };
                return collectionObject;
            }
        }
    );
}
