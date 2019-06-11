import { TalkimBaseAction } from "./base";
import { ChannelId, Designator, Message, UserId } from "../datatypes";

export interface SendMessageRequest extends TalkimBaseAction {
    type: "send-message-request";
    content: string;
    recipient: Designator;
}

export interface SendMessageSuccess extends TalkimBaseAction {
    type: "send-message-success";
    message: Message;
}

export interface SendMessageFailure extends TalkimBaseAction {
    type: "send-message-failure";
}

export interface FetchMessagesRequest extends TalkimBaseAction {
    type: "fetch-messages-request";
    recipient: Designator;
}

export interface ProvideMessages extends TalkimBaseAction {
    type: "provide-messages";
    messages: Message[];
}

export interface FetchMessagesFailure extends TalkimBaseAction {
    type: "fetch-message-failure";
}

export type MessageAction =
    | SendMessageRequest
    | SendMessageSuccess
    | SendMessageFailure
    | FetchMessagesRequest
    | ProvideMessages
    | FetchMessagesFailure;
