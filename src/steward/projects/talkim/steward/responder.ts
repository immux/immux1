import { TalkimAction } from "../actions/Action";
import {
    Channel,
    Message,
    SessionToken,
    SessionTokenId,
    User,
    UserSecret
} from "../datatypes";
import { StewardEnhancedIncoming, StewardEnhancedOutgoing } from "../../meta";
import { ProvideMessages } from "../actions/MessageAction";
import { ProvideUsers } from "../actions/UserActions";
import { ImmuxDbJS } from "../../../../connectors/typescript/immuxdb.types";

function dedupe<T>(value: T, index: number, array: T[]) {
    return index === array.indexOf(value);
}

type WithId<T> = { id: string } & T;

const COOKIE_SEPARATOR = `#`;

interface SessionMark {
    tokenId: SessionTokenId;
    tokenSecret: string;
}

function formCookie(mark: SessionMark): string {
    return [mark.tokenId, mark.tokenSecret].join(COOKIE_SEPARATOR);
}

function parseCookie(cookie: string): SessionMark | null {
    const segments = cookie.split(COOKIE_SEPARATOR);
    if (segments.length < 2) {
        return null;
    } else {
        return {
            tokenId: segments[0],
            tokenSecret: segments[1]
        };
    }
}

async function authenticate(
    cookie: string,
    state: ImmuxDbJS
): Promise<User | null> {
    if (!cookie) {
        console.warn("authenticate: no cookie");
        return null;
    }
    const mark = parseCookie(cookie);
    if (!mark) {
        console.warn("authenticate: cannot parse mark");
        return null;
    }
    const { tokenId, tokenSecret } = mark;
    const token = await state.sessionTokens.findOne<SessionToken>({
        id: tokenId
    });
    if (!token) {
        console.warn("authenticate: no token");
        return null;
    }
    if (token.secret !== tokenSecret) {
        console.warn("authenticate: no secret");
        return null;
    }
    const user = await state.users.findOne<User>({ id: token.holder });
    if (!user) {
        console.warn("authenticate: no user");
        return null;
    }
    return user;
}

async function responder(
    state: ImmuxDbJS,
    action: StewardEnhancedIncoming<TalkimAction>
): Promise<StewardEnhancedOutgoing<TalkimAction>> {
    switch (action.type) {
        case "create-channel-request": {
            const authenticatedActor = await authenticate(
                action.steward.cookie,
                state
            );
            if (!authenticatedActor) {
                return {
                    type: "authentication-failure"
                };
            }
            const existing = await state.channels.findOne<Channel>({
                name: action.name
            });
            if (existing) {
                return {
                    type: "create-channel-failure"
                };
            }
            const channel: WithId<Channel> = {
                id: Math.random().toString(),
                name: action.name,
                owner: action.steward.cookie
            };
            await state.channels.upsert(channel);
            return {
                type: "create-channel-success",
                newChannel: channel
            };
        }
        case "send-message-request": {
            const authenticatedActor = await authenticate(
                action.steward.cookie,
                state
            );
            if (!authenticatedActor) {
                return {
                    type: "authentication-failure"
                };
            }
            const message: Message = {
                id: Math.random().toString(),
                sender: authenticatedActor.id,
                recipient: action.recipient,
                content: action.content,
                time: Date.now()
            };
            await state.messages.upsert(message);
            return {
                type: "send-message-success",
                message
            };
        }
        case "fetch-channels-request": {
            const channels = await state.channels.find<Channel>();
            return {
                type: "provide-channels",
                channels: channels
            };
        }
        case "fetch-messages-request": {
            const data = await state.messages.find<WithId<Message>>({});
            const messages = data.filter(
                message =>
                    message.recipient.id === action.recipient.id &&
                    message.recipient.type === action.recipient.type
            );
            const users = (await Promise.all(
                messages
                    .map(messsage => messsage.sender)
                    .filter(dedupe)
                    .map(id => state.users.findOne<User>({ id }))
            )).filter(Boolean) as User[];
            const provideMessage: ProvideMessages = {
                type: "provide-messages",
                messages
            };
            const provideUsers: ProvideUsers = {
                type: "provide-users",
                users
            };
            return {
                type: "composite-action",
                actions: [provideUsers, provideMessage]
            };
        }
        case "create-user-request": {
            const { email, password } = action;
            const existing = await state.users.findOne<User>({ email });
            if (existing) {
                return {
                    type: "create-user-failure"
                };
            }
            const user: User = {
                id: Math.random().toString(),
                email
            };
            await state.users.upsert(user);
            const secret: UserSecret = {
                id: Math.random().toString(),
                userId: user.id,
                password
            };
            await state.userSecrets.upsert(secret);
            const token: SessionToken = {
                id: Math.random().toString(),
                holder: user.id,
                valid: true,
                secret: Math.random().toString() + Math.random().toString(),
                issueTime: Date.now()
            };
            await state.sessionTokens.upsert(token);
            return {
                type: "create-user-success",
                self: user,
                steward: {
                    cookie: formCookie({
                        tokenId: token.id,
                        tokenSecret: token.secret
                    })
                }
            };
        }
        case "signin-request": {
            const { email, password } = action;
            const existing = await state.users.findOne<User>({ email });
            if (!existing) {
                return {
                    type: "signin-failure"
                };
            }
            const userSecret = await state.userSecrets.findOne<UserSecret>({
                userId: existing.id
            });
            if (!userSecret) {
                return {
                    type: "signin-failure"
                };
            }
            if (userSecret.password !== password) {
                return {
                    type: "signin-failure"
                };
            }

            const token: SessionToken = {
                id: Math.random().toString(),
                holder: existing.id,
                valid: true,
                secret: Math.random().toString() + Math.random().toString(),
                issueTime: Date.now()
            };
            await state.sessionTokens.upsert(token);
            return {
                type: "signin-success",
                self: existing,
                steward: {
                    cookie: formCookie({
                        tokenId: token.id,
                        tokenSecret: token.secret
                    })
                }
            };
        }
        case "get-self-request": {
            const authenticatedActor = await authenticate(
                action.steward.cookie,
                state
            );
            if (!authenticatedActor) {
                return {
                    type: "get-self-failure"
                };
            }
            return {
                type: "get-self-success",
                self: authenticatedActor
            };
        }
        default: {
            return {
                type: "no-op"
            };
        }
    }
}
