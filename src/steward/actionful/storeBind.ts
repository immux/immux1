import { Action, AnyAction, Dispatch, Store } from "redux";
import { ActionfulAction } from "../projects/foldr/types";
import { HTTP_SUBPATH, WS_SUBPATH } from "../paths";

interface HijackedStore<S = any, A extends Action = AnyAction>
    extends Store<S, A> {
    originalDispatch<T extends A>(action: T): T;
}

interface HijackStoreOptions {
    preDispatch: (action: any) => any;
    whitelist?: string[];
}

export function hijackStore<S = any, A extends Action = AnyAction>(
    store: Store<S, A>,
    options: HijackStoreOptions
): HijackedStore<S, A> {
    const { whitelist, preDispatch } = options;
    const newDispatch: Dispatch<A> = function<T extends A>(action: T): T {
        if (!whitelist || whitelist.includes(action.type)) {
            preDispatch(action);
        }
        return store.dispatch(action);
    }.bind(store);
    return {
        ...store,
        dispatch: newDispatch,
        originalDispatch: store.dispatch
    };
}

export type BidirectionalTransportChannel = "ws"; // | "webrtc"
export type TransportChannel = BidirectionalTransportChannel | "http";

interface BindStoreOptions {
    preferredChannel?: BidirectionalTransportChannel | null;
    whitelist?: string[];
    forceChannelDict?: { [type in string]?: TransportChannel };
}

export function bindStore<S = any, A extends ActionfulAction = AnyAction>(
    store: Store<S, A>,
    origin: string = window.location.origin,
    options: BindStoreOptions = {}
): Store<S, A> {
    const { preferredChannel, whitelist, forceChannelDict } = options;
    const currentTransport: TransportChannel = preferredChannel || "http";

    let pendingActions: any[] = [];
    function connect(): WebSocket {
        const protocol = origin.startsWith("https") ? "wss" : "ws";
        const host = origin.replace("http://", "").replace("https://", "");
        const path = `${protocol}://${host}${WS_SUBPATH}`;

        const socket = new WebSocket(path);
        socket.onopen = () => {
            for (const action of pendingActions) {
                console.info("send pending", action);
                socket.send(JSON.stringify(action));
            }
            pendingActions = [];
        };
        socket.onmessage = event => {
            newStore.originalDispatch(JSON.parse(event.data));
        };
        socket.onclose = () => {
            setTimeout(connect, 1000);
        };
        return socket;
    }
    const socket = connect();

    function getTransport(type: string): TransportChannel {
        if (!forceChannelDict) {
            return currentTransport;
        }
        return forceChannelDict[type] || currentTransport;
    }

    const newStore = hijackStore(store, {
        async preDispatch(action: A) {
            const transport = getTransport(action.type);
            switch (transport) {
                case "http": {
                    const path = `${origin}${HTTP_SUBPATH}`;
                    console.info("using path", path);
                    const response = await fetch(path, {
                        method: "POST",
                        body: JSON.stringify(action)
                    });
                    try {
                        const action = await response.json();
                        store.dispatch(action);
                    } catch (error) {
                        console.error(error);
                    }
                    break;
                }
                case "ws": {
                    if (socket && socket.readyState === socket.OPEN) {
                        console.info("send", action);
                        socket.send(JSON.stringify(action));
                    } else {
                        console.info("keep", action);
                        pendingActions.push(action);
                    }
                    break;
                }
            }
        },
        whitelist
    });

    return newStore;
}
