import * as React from "react";
import * as ReactDom from "react-dom";
import { Provider } from "react-redux";
import store from "./store";
import ConnectedRoot from "./components/Root";

ReactDom.render(
    <Provider store={store}>
        <ConnectedRoot />
    </Provider>,
    document.getElementById("root")
);
