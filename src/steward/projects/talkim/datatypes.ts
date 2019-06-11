type UUID = string;

export type UserId = UUID;

export interface User {
    id: UserId;
    email: string;
}

export type UserSecretId = string;

export interface UserSecret {
    id: UserSecretId;
    userId: UserId;
    password: string;
}

export type ChannelId = UUID;

export interface Channel {
    id: ChannelId;
    name: string;
    owner: string;
}

interface UserDesignator {
    type: "user";
    id: UserId;
}

interface ChannelDesignator {
    type: "channel";
    id: ChannelId;
}

export type Designator = UserDesignator | ChannelDesignator;

export type MessageId = UUID;

export interface Message {
    id: MessageId;
    sender: UserId;
    recipient: Designator;
    content: string;
    time: number;
}

export type SessionTokenId = UUID;

export interface SessionToken {
    id: SessionTokenId;
    holder: UserId;
    valid: boolean;
    secret: string;
    issueTime: number;
}
