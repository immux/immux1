import { FoldrProject } from "../meta";

export interface ActionfulAction {
    type: string;
}

export interface UnknownAction {
    type: "unknown-action";
    error: true;
}

export interface UserData {
    id: number;
    username: string;
    password: string;
}

export interface CreateUserAction extends ActionfulAction {
    type: "create-user";
    payload: {
        username: string;
        password: string;
    };
}

export interface CreateUserSuccess extends ActionfulAction {
    type: "create-user-success";
    payload: {
        user: UserData;
    };
}

export interface CreateUserFailure extends ActionfulAction {
    type: "create-user-failure";
    payload: {
        reason: string;
    };
}

export interface SigninAction extends ActionfulAction {
    type: "sign-in";
    payload: {
        username: string;
        password: string;
    };
}

export interface SigninSuccess extends ActionfulAction {
    type: "sign-in-success";
    payload: {
        user: UserData;
        projects: FoldrProject[];
    };
}

export interface SigninFailure extends ActionfulAction {
    type: "sign-in-failure";
    payload: {
        reason: string;
    };
}

export interface CreateProject extends ActionfulAction {
    type: "create-project";
    payload: {
        name: string;
    };
    meta: {
        username: string;
        password: string;
    };
}

export interface CreateProjectSuccess extends ActionfulAction {
    type: "create-project-success";
    payload: {
        project: FoldrProject;
    };
}

export interface UpdateProject extends ActionfulAction {
    type: "update-project";
    payload: {
        projectName: string;
        index?: string | null;
        responder?: string | null;
        distributor?: string | null;
    };
    meta: {
        username: string;
        password: string;
    };
}

export interface GetSelf extends ActionfulAction {
    type: "get-self";
    meta: {
        username: string;
        password: string;
    };
}

export interface AuthenticationFailure {
    type: "authentication-failure";
}

export interface NotFound {
    type: "not-found";
}

export interface UserNotFound {
    type: "user-not-found";
}

export interface ProjectNotFound {
    type: "project-not-found"
}

export interface URLParsingError {
    type: "url-parsing-error",
    url: string
}

export interface DuplicatedProject {
    type: "duplicated-project"
}

export interface AuthorizationFailure {
    type: "authorization-failure";
}

export interface ProvideUser {
    type: "provide-user";
    payload: {
        user: UserData;
    };
}

export interface Ok {
    type: "ok";
}

export interface NoResponder {
    type: "no-responder";
    project: string
}

export type FoldrAction =
    | UnknownAction
    | Ok
    | NotFound
    | UserNotFound
    | ProjectNotFound
    | DuplicatedProject
    | URLParsingError
    | NoResponder
    | AuthenticationFailure
    | AuthorizationFailure
    | CreateUserAction
    | CreateUserSuccess
    | CreateUserFailure
    | SigninAction
    | SigninSuccess
    | SigninFailure
    | ProvideUser
    | CreateProject
    | CreateProjectSuccess
    | UpdateProject
    | GetSelf;
