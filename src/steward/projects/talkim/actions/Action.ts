import { ChannelAction } from "./ChannelAction";
import { TransientAction } from "../transient/Action";
import { TalkimBaseAction } from "./base";
import { MessageAction } from "./MessageAction";
import { UserAction } from "./UserActions";

export interface Noop extends TalkimBaseAction {
    type: "no-op";
}

export interface CompositeAction extends TalkimBaseAction {
    type: "composite-action";
    actions: TalkimAction[];
}

export interface AuthenticationFailure extends TalkimBaseAction {
    type: "authentication-failure";
}

export type TalkimAction =
    | Noop
    | CompositeAction
    | AuthenticationFailure
    | TransientAction
    | UserAction
    | ChannelAction
    | MessageAction;
