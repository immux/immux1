import * as React from "react";
import { Dispatch } from "redux";
import { connect } from "react-redux";

import { MapById } from "../../../utils";
import { Channel, ChannelId, Message, User } from "../../datatypes";
import { TalkimAction } from "../../actions/Action";
import { TalkimTransientState } from "../reducer";

import SideBar from "./SideBar";
import TalkPanel from "./TalkPanel";
import IdentityManager from "./IdentityManager";

interface RootState {}

interface RootDataProps {
    me: User | null;
    channels: MapById<Channel>;
    activeChannel: ChannelId | null;
    messages: MapById<Message>;
    users: MapById<User>;
}

interface RootCallbackProps {
    action(action: TalkimAction): void;
}

type RootProps = RootDataProps & RootCallbackProps;

class Root extends React.Component<RootProps, RootState> {
    componentDidMount() {
        this.props.action({
            type: "get-self-request"
        });
        this.props.action({
            type: "fetch-channels-request"
        });
    }

    render() {
        return (
            <div
                id="root"
                style={{
                    display: "flex"
                }}
            >
                <SideBar
                    me={this.props.me}
                    channels={this.props.channels}
                    createChannel={name => {
                        this.props.action({
                            type: "create-channel-request",
                            name
                        });
                    }}
                    pickChannel={channelId => {
                        this.props.action({
                            type: "pick-channel",
                            channel: channelId
                        });
                        this.props.action({
                            type: "fetch-messages-request",
                            recipient: {
                                type: "channel",
                                id: channelId
                            }
                        });
                    }}
                />
                <TalkPanel
                    messages={this.props.messages}
                    users={this.props.users}
                    me={this.props.me}
                    channel={
                        this.props.activeChannel
                            ? this.props.channels[this.props.activeChannel]
                            : null
                    }
                    sendMessage={text => {
                        if (this.props.activeChannel) {
                            this.props.action({
                                type: "send-message-request",
                                content: text,
                                recipient: {
                                    type: "channel",
                                    id: this.props.activeChannel
                                }
                            });
                        }
                    }}
                />
                {!this.props.me && (
                    <IdentityManager
                        register={(email, password) => {
                            this.props.action({
                                type: "create-user-request",
                                email,
                                password
                            });
                        }}
                        signin={(email, password) => {
                            this.props.action({
                                type: "signin-request",
                                email,
                                password
                            });
                        }}
                    />
                )}
            </div>
        );
    }
}

function mapStateToProps(state: TalkimTransientState): RootDataProps {
    return {
        me: state.entities.me,
        users: state.entities.users,
        channels: state.entities.channels,
        activeChannel: state.ui.activeChannel,
        messages: state.entities.messages
    };
}

function mapDispatchToProps(dispatch: Dispatch): RootCallbackProps {
    return {
        action(action: TalkimAction) {
            dispatch(action);
        }
    };
}

const ConnectedRoot = connect(
    mapStateToProps,
    mapDispatchToProps
)(Root);

export default ConnectedRoot;
