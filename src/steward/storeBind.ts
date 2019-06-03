import { Action, AnyAction, Dispatch, Store } from "redux";

export function hijackStore<S = any, A extends Action = AnyAction>(
    store: Store<S, A>,
    preDispatch: (action: any) => any,
    whitelist?: string[]
): Store<S, A> {
    const newDispatch: Dispatch<A> = function<T extends A>(action: T): T {
        console.info("triggered hijack");
        if (!whitelist || whitelist.includes(action.type)) {
            preDispatch(action);
        }
        return store.dispatch(action);
    }.bind(store);
    return {
        ...store,
        dispatch: newDispatch
    };
}

export function bindStoreToSocket(store: Store, path: string): Store {
    let socket: WebSocket | null = null;
    let pendingActions: any[] = [];

    const newStore = hijackStore(store, action => {
        if (socket && socket.readyState === socket.OPEN) {
            console.info("send", action);
            socket.send(JSON.stringify(action));
        } else {
            console.info("keep", action);
            pendingActions.push(action);
        }
    });

    function connect() {
        socket = new WebSocket(path);
        socket.onopen = () => {
            for (const action of pendingActions) {
                socket && socket.send(JSON.stringify(action));
            }
            pendingActions = [];
        };
        socket.onmessage = event => {
            newStore.dispatch(JSON.parse(event.data));
        };
        socket.onclose = () => {
            setTimeout(connect, 1000);
        };
    }

    connect();

    return newStore;
}

export function bindStoreToHttpPost(store: Store, path: string): Store {
    const newStore = hijackStore(store, async action => {
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
    });

    return newStore;
}
