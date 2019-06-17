import * as React from "react";
import { MapById } from "../../../utils";
import { Channel, ChannelId, Message, User } from "../../datatypes";

interface Props {
    me: User | null;
    channel: Channel | null;
    messages: MapById<Message>;
    users: MapById<User>;
    sendMessage(content: string): void;
}

interface State {
    textInput: string;
}

class TalkPanel extends React.Component<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = {
            textInput: ""
        };
    }

    render() {
        const { channel } = this.props;
        if (!channel) {
            return <div>Pick a channel</div>;
        }
        return (
            <div
                style={{
                    flex: 1
                }}
            >
                TalkPanel
                <div>#{channel.name}</div>
                <div>
                    {Object.values(this.props.messages)
                        .filter(
                            message =>
                                message.recipient.type === "channel" &&
                                message.recipient.id === channel.id
                        )
                        .map(message => {
                            const sender = this.props.users[message.sender];
                            const senderName = sender ? sender.email : "?";
                            return (
                                <div key={message.id}>
                                    <span>{senderName}:</span>
                                    {message.content}
                                    <span style={{ float: "right" }}>
                                        {new Date(
                                            message.time
                                        ).toLocaleString()}
                                    </span>
                                </div>
                            );
                        })}
                    <input
                        style={{
                            width: "80%"
                        }}
                        value={this.state.textInput}
                        onChange={event => {
                            this.setState({
                                textInput: event.target.value
                            });
                        }}
                    />
                    <button
                        onClick={() => {
                            this.props.sendMessage(this.state.textInput);
                            this.setState({
                                textInput: ""
                            });
                        }}
                    >
                        Send
                    </button>
                </div>
            </div>
        );
    }
}

export default TalkPanel;
