import * as React from "react";

interface DataProps {}

interface CallbackProps {
    register(email: string, password: string): void;
    signin(email: string, password: string): void;
}

type Props = DataProps & CallbackProps;

interface State {
    emailInput: string;
    passwordInput: string;
    mode: "register" | "signin";
}

class IdentityManager extends React.Component<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = {
            emailInput: "",
            passwordInput: "",
            mode: "register"
        };
    }

    render() {
        return (
            <div
                style={{
                    position: "fixed",
                    top: 0,
                    left: 0,
                    right: 0,
                    bottom: 0,
                    background: "rgba(0, 0, 0, 0.8)",
                    padding: "10em"
                }}
            >
                <div
                    style={{
                        background: "white",
                        minWidth: "50vw",
                        minHeight: "50vh"
                    }}
                >
                    <h2>
                        {this.state.mode === "register"
                            ? "Register"
                            : "Sign in"}
                    </h2>
                    <input
                        value={this.state.emailInput}
                        placeholder="Your name"
                        onChange={event =>
                            this.setState({
                                emailInput: event.target.value
                            })
                        }
                    />
                    <input
                        type="password"
                        value={this.state.passwordInput}
                        placeholder="Your name"
                        onChange={event =>
                            this.setState({
                                passwordInput: event.target.value
                            })
                        }
                    />
                    <button
                        onClick={() => {
                            if (this.state.mode === "register") {
                                this.props.register(
                                    this.state.emailInput,
                                    this.state.passwordInput
                                );
                            } else {
                                this.props.signin(
                                    this.state.emailInput,
                                    this.state.passwordInput
                                );
                            }
                        }}
                    >
                        {this.state.mode === "register"
                            ? "Register"
                            : "Sign in"}
                    </button>
                    <div>
                        <button
                            onClick={() =>
                                this.setState(prevState => ({
                                    ...prevState,
                                    mode:
                                        prevState.mode === "register"
                                            ? "signin"
                                            : "register"
                                }))
                            }
                        >
                            alternate mode
                        </button>
                    </div>
                </div>
            </div>
        );
    }
}

export default IdentityManager;
