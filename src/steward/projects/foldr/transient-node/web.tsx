import * as React from "react";
import { Dispatch } from "redux";
import * as ReactDom from "react-dom";
import { Provider, connect } from "react-redux";
import {
    CreateUserAction,
    FoldrAction,
    SigninAction,
    UserData
} from "../types";
import { FoldrProject } from "../../meta";
import { FoldrClientState } from "./types";
import ProjectPanel from "./ProjectPanel";
import store from "../store";

export function dedupeById<T extends { id: string }>(prev: T[], next: T): T[] {
    if (prev.some(element => element.id === next.id)) {
        return prev;
    } else {
        return [...prev, next];
    }
}

interface RootDataProps {
    projects: FoldrProject[];
    me: UserData | null;
}

interface RootCallbackProps {
    action(action: FoldrAction): void;
}

type RootProps = RootDataProps & RootCallbackProps;

interface RootState {
    signinMode: boolean;
    usernameInput: string;
    passwordInput: string;
}

class Root extends React.Component<RootProps, RootState> {
    constructor(props: RootProps) {
        super(props);
        this.state = {
            signinMode: true,
            usernameInput: "",
            passwordInput: ""
        };
    }

    render() {
        return (
            <div
                style={{
                    fontFamily: "sans-serif"
                }}
            >
                <h1>
                    Foldr<span style={{ fontSize: "0.5em" }}>/Prototype</span>
                </h1>
                {!this.props.me && !this.state.signinMode && (
                    <div className="panel-register">
                        <h2>Register</h2>
                        <input
                            placeholder="Username"
                            value={this.state.usernameInput}
                            onChange={event =>
                                this.setState({
                                    usernameInput: event.target.value
                                })
                            }
                        />
                        <input
                            placeholder="Password"
                            value={this.state.passwordInput}
                            type="password"
                            onChange={event =>
                                this.setState({
                                    passwordInput: event.target.value
                                })
                            }
                        />
                        <button
                            className="register"
                            onClick={async () => {
                                const action: CreateUserAction = {
                                    type: "create-user",
                                    payload: {
                                        username: this.state.usernameInput,
                                        password: this.state.passwordInput
                                    }
                                };
                                this.props.action(action);
                            }}
                        >
                            Register
                        </button>
                        <div>
                            <a
                                href="#"
                                onClick={() =>
                                    this.setState({
                                        signinMode: true
                                    })
                                }
                            >
                                Sign in instead
                            </a>
                        </div>
                    </div>
                )}
                {!this.props.me && this.state.signinMode && (
                    <div className="panel-signin">
                        <h2>Sign in</h2>
                        <input
                            placeholder="Username"
                            value={this.state.usernameInput}
                            onChange={event =>
                                this.setState({
                                    usernameInput: event.target.value
                                })
                            }
                        />
                        <input
                            placeholder="Password"
                            value={this.state.passwordInput}
                            type="password"
                            onChange={event =>
                                this.setState({
                                    passwordInput: event.target.value
                                })
                            }
                        />
                        <button
                            className="register"
                            onClick={async () => {
                                const action: SigninAction = {
                                    type: "sign-in",
                                    payload: {
                                        username: this.state.usernameInput,
                                        password: this.state.passwordInput
                                    }
                                };
                                this.props.action(action);
                            }}
                        >
                            Sign in
                        </button>
                        <div>
                            <a
                                href="#"
                                onClick={() =>
                                    this.setState({
                                        signinMode: false
                                    })
                                }
                            >
                                Register instead
                            </a>
                        </div>
                    </div>
                )}
                {this.props.me && (
                    <div className="panel-self">
                        <div className="username" />
                        <ProjectPanel
                            projects={this.props.projects}
                            onIndexChange={(code, project) => {
                                this.props.action({
                                    type: "update-project",
                                    payload: {
                                        projectName: project.name,
                                        index: code
                                    },
                                    meta: {
                                        username: this.props.me
                                            ? this.props.me.username
                                            : "",
                                        password: this.props.me
                                            ? this.props.me.password
                                            : ""
                                    }
                                });
                            }}
                            onResponderChange={(code, project) => {
                                this.props.action({
                                    type: "update-project",
                                    payload: {
                                        projectName: project.name,
                                        responder: code
                                    },
                                    meta: {
                                        username: this.props.me
                                            ? this.props.me.username
                                            : "",
                                        password: this.props.me
                                            ? this.props.me.password
                                            : ""
                                    }
                                });
                            }}
                            onDistributorChange={(code, project) => {
                                this.props.action({
                                    type: "update-project",
                                    payload: {
                                        projectName: project.name,
                                        distributor: code
                                    },
                                    meta: {
                                        username: this.props.me
                                            ? this.props.me.username
                                            : "",
                                        password: this.props.me
                                            ? this.props.me.password
                                            : ""
                                    }
                                });
                            }}
                            onCreateProject={name => {
                                this.props.action({
                                    type: "create-project",
                                    payload: {
                                        name
                                    },
                                    meta: {
                                        username: this.props.me
                                            ? this.props.me.username
                                            : "",
                                        password: this.props.me
                                            ? this.props.me.password
                                            : ""
                                    }
                                });
                            }}
                        />
                    </div>
                )}
            </div>
        );
    }
}

function mapStateToProps(state: FoldrClientState): RootDataProps {
    return {
        projects: state.projects,
        me: state.me
    };
}

function mapDispatchToProps(dispatch: Dispatch): RootCallbackProps {
    return {
        action(action) {
            dispatch(action);
        }
    };
}

const ConnectedRoot = connect(
    mapStateToProps,
    mapDispatchToProps
)(Root);

ReactDom.render(
    <Provider store={store}>
        <ConnectedRoot />
    </Provider>,
    document.getElementById("react-root")
);
