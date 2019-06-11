import { TalkimBaseAction } from "../actions/base";
import { ChannelId, User } from "../datatypes";

export interface SetSelf extends TalkimBaseAction {
    type: "set-self";
    me: User;
}

export interface PickChannel extends TalkimBaseAction {
    type: "pick-channel";
    channel: ChannelId;
}

export type TransientAction = SetSelf | PickChannel;
