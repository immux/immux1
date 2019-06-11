import * as React from "react";
import { MapById } from "../../../utils";
import { Channel, ChannelId, User } from "../../datatypes";

interface Props {
    me: User | null;
    channels: MapById<Channel>;
    createChannel(name: string): void;
    pickChannel(channel: ChannelId): void;
}

interface State {
    channelNameInput: string;
}

class SideBar extends React.Component<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = {
            channelNameInput: ""
        };
    }

    render() {
        return (
            <div
                className="SideBar"
                style={{
                    background: "#ccc",
                    width: "200px",
                    height: "100vh",
                    padding: "1em"
                }}
            >
                Sidebar
                {this.props.me && (
                    <div
                        style={{
                            overflow: "hidden"
                        }}
                    >
                        Me:
                        {this.props.me.email}({this.props.me.id.slice(0, 5)})
                    </div>
                )}
                {!this.props.me && <div>No identity found</div>}
                <div>
                    {Object.values(this.props.channels).map(channel => (
                        <a
                            key={channel.id}
                            style={{
                                display: "block",
                                padding: "0.2em"
                            }}
                            onClick={() => {
                                this.props.pickChannel(channel.id);
                            }}
                        >
                            #{channel.name}
                        </a>
                    ))}
                </div>
                <div>
                    <input
                        placeholder="new channel name"
                        value={this.state.channelNameInput}
                        onChange={event => {
                            this.setState({
                                channelNameInput: event.target.value
                            });
                        }}
                    />
                    <button
                        onClick={() => {
                            this.props.createChannel(
                                this.state.channelNameInput
                            );
                            this.setState({
                                channelNameInput: ""
                            });
                        }}
                    >
                        Create channel
                    </button>
                </div>
            </div>
        );
    }
}

export default SideBar;
