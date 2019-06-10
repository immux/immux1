import { FoldrClientState } from "./transient-node/types";
import { FoldrAction } from "./types";
import { INITIAL_STATE, reducer } from "./transient-node/reducer";
import { createStore } from "redux";
import { bindStoreToHttpPost } from "../../storeBind";

declare global {
    interface Window {
        __REDUX_DEVTOOLS_EXTENSION__?: Function;
    }
}

const store = createStore<FoldrClientState, FoldrAction, null, null>(
    reducer,
    INITIAL_STATE,
    window.__REDUX_DEVTOOLS_EXTENSION__ && window.__REDUX_DEVTOOLS_EXTENSION__()
);

const boundStore = bindStoreToHttpPost(store, `${location.origin}/http`);

export default boundStore;
