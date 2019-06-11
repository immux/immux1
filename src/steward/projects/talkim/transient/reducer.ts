import {
    Channel,
    ChannelId,
    Message,
    MessageId,
    User,
    UserId
} from "../datatypes";
import { TalkimAction } from "../actions/Action";
import { normalizeById } from "../../utils";

export interface TalkimTransientState {
    entities: {
        me: User | null;
        channels: { [id in ChannelId]: Channel };
        messages: { [id in MessageId]: Message };
        users: { [id in UserId]: User };
    };
    ui: {
        activeChannel: ChannelId | null;
    };
}

export const INITIAL_STATE: TalkimTransientState = {
    entities: {
        me: null,
        channels: {},
        messages: {},
        users: {}
    },
    ui: {
        activeChannel: null
    }
};

export function reducer(
    state: TalkimTransientState = INITIAL_STATE,
    action: TalkimAction
): TalkimTransientState {
    if (action.type === "composite-action") {
        let nextState = state;
        for (const subAction of action.actions) {
            nextState = reducer(nextState, subAction);
        }
        return nextState;
    }

    switch (action.type) {
        case "set-self": {
            return {
                ...state,
                entities: {
                    ...state.entities,
                    me: action.me
                }
            };
        }
        case "provide-users": {
            return {
                ...state,
                entities: {
                    ...state.entities,
                    users: {
                        ...state.entities.users,
                        ...normalizeById(action.users)
                    }
                }
            };
        }
        case "create-user-success": {
            return {
                ...state,
                entities: {
                    ...state.entities,
                    me: action.self
                }
            };
        }
        case "signin-success": {
            return {
                ...state,
                entities: {
                    ...state.entities,
                    me: action.self
                }
            };
        }
        case "get-self-success": {
            return {
                ...state,
                entities: {
                    ...state.entities,
                    me: action.self,
                    users: {
                        ...state.entities.users,
                        [action.self.id]: action.self
                    }
                }
            };
        }
        case "create-channel-success": {
            const channel = action.newChannel;
            return {
                ...state,
                entities: {
                    ...state.entities,
                    channels: {
                        ...state.entities.channels,
                        [channel.id]: channel
                    }
                }
            };
        }
        case "send-message-success": {
            return {
                ...state,
                entities: {
                    ...state.entities,
                    messages: {
                        ...state.entities.messages,
                        [action.message.id]: action.message
                    }
                }
            };
        }
        case "provide-channels": {
            return {
                ...state,
                entities: {
                    ...state.entities,
                    channels: {
                        ...state.entities.channels,
                        ...normalizeById(action.channels)
                    }
                }
            };
        }
        case "pick-channel": {
            return {
                ...state,
                ui: {
                    ...state.ui,
                    activeChannel: action.channel
                }
            };
        }
        case "provide-messages": {
            return {
                ...state,
                entities: {
                    ...state.entities,
                    messages: {
                        ...state.entities.messages,
                        ...normalizeById(action.messages)
                    }
                }
            };
        }
        default: {
            return state;
        }
    }
}
