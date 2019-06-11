import { createStore } from "redux";
import { INITIAL_STATE, reducer, TalkimTransientState } from "./reducer";
import { bindStore } from "../../../actionful/storeBind";
import { TalkimAction } from "../actions/Action";

declare global {
    interface Window {
        __REDUX_DEVTOOLS_EXTENSION__?: Function;
    }
}

const store = createStore<TalkimTransientState, TalkimAction, null, null>(
    reducer,
    INITIAL_STATE,
    window.__REDUX_DEVTOOLS_EXTENSION__ && window.__REDUX_DEVTOOLS_EXTENSION__()
);

const boundStore = bindStore(store, location.origin, {
    preferredChannel: "ws",
    forceChannelDict: {
        "signin-request": "http",
        "get-self-request": "http"
    }
});

export default boundStore;
