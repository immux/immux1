import { TalkimBaseAction } from "./base";
import { User } from "../datatypes";

export interface CreateUserRequest extends TalkimBaseAction {
    type: "create-user-request";
    email: string;
    password: string;
}

export interface CreateUserSuccess extends TalkimBaseAction {
    type: "create-user-success";
    self: User;
}

export interface CreateUserFailure extends TalkimBaseAction {
    type: "create-user-failure";
}

export interface SigninRequest extends TalkimBaseAction {
    type: "signin-request";
    email: string;
    password: string;
}

export interface SigninSuccess extends TalkimBaseAction {
    type: "signin-success";
    self: User;
}

export interface SigninFailure extends TalkimBaseAction {
    type: "signin-failure";
}

export interface GetSelfRequest extends TalkimBaseAction {
    type: "get-self-request";
}

export interface GetSelfSuccess extends TalkimBaseAction {
    type: "get-self-success";
    self: User;
}

export interface GetSelfFailure extends TalkimBaseAction {
    type: "get-self-failure";
}

export interface ProvideUsers extends TalkimBaseAction {
    type: "provide-users";
    users: User[];
}

export type UserAction =
    | CreateUserRequest
    | CreateUserSuccess
    | CreateUserFailure
    | SigninRequest
    | SigninSuccess
    | SigninFailure
    | GetSelfRequest
    | GetSelfSuccess
    | GetSelfFailure
    | ProvideUsers;
