import { TalkimBaseAction } from "./base";
import { Channel } from "../datatypes";
import { MapById } from "../../utils";

export interface CreateChannelRequest extends TalkimBaseAction {
    type: "create-channel-request";
    name: string;
}

export interface CreateChannelSuccess extends TalkimBaseAction {
    type: "create-channel-success";
    newChannel: Channel;
}

export interface CreateChannelFailure extends TalkimBaseAction {
    type: "create-channel-failure";
}

export interface FetchChannelsRequest extends TalkimBaseAction {
    type: "fetch-channels-request";
}

export interface ProvideChannels extends TalkimBaseAction {
    type: "provide-channels";
    channels: Channel[];
}

export interface SubscribeChannel extends TalkimBaseAction {
    type: "subscribe-channel";
    channel: string;
}

export type ChannelAction =
    | CreateChannelRequest
    | CreateChannelSuccess
    | CreateChannelFailure
    | FetchChannelsRequest
    | ProvideChannels
    | SubscribeChannel;
