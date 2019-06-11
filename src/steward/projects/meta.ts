import { ActionfulAction } from "./foldr/types";

export interface FoldrProject {
    id: string;
    owner: string;
    name: string;
    index: string | null;
    responder: string | null;
    distributor: string | null;
}

export interface AllPeerDistribution {
    type: "all-peers";
    action: ActionfulAction;
}

export type Distribution = AllPeerDistribution;

export type Distributor = (state: any, action: any) => Promise<Distribution[]>;

export type StewardEnhancedIncoming<A extends ActionfulAction> = A & {
    steward: {
        cookie: string;
        host: string;
    };
};

export type StewardEnhancedOutgoing<A extends ActionfulAction> = A & {
    steward?: {
        cookie?: string;
    };
};
