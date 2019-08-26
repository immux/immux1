import { FoldrAction, UserData } from "../types";
import { FoldrProject } from "../../meta";
import { ImmuxDbJS } from "../../../../connectors/typescript/immuxdb.types";

function escapeHtml(unsafe: string): string {
    return unsafe
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
}

async function authenticate(
    state: ImmuxDbJS,
    username: string,
    password: string
): Promise<UserData | null> {
    const claimedUser = await state.users.findOne<UserData>({ username });
    if (!claimedUser) {
        return null;
    } else if (claimedUser.password !== password) {
        return null;
    } else {
        return claimedUser;
    }
}

async function responder(
    state: ImmuxDbJS,
    action: FoldrAction
): Promise<FoldrAction> {
    switch (action.type) {
        case "create-user": {
            const existingUser = await state.users.findOne<UserData>({
                username: action.payload.username
            });
            if (existingUser) {
                return {
                    type: "create-user-failure",
                    payload: {
                        reason: "duplicated"
                    }
                };
            } else {
                const newUser: UserData = {
                    id: Math.random().toString(),
                    username: action.payload.username,
                    password: action.payload.password
                };
                await state.users.upsert({
                    id: Math.random().toString(),
                    ...newUser
                });
                return {
                    type: "create-user-success",
                    payload: {
                        user: newUser
                    }
                };
            }
        }
        case "sign-in": {
            const existingUser = await state.users.findOne<UserData>({
                username: action.payload.username
            });
            if (!existingUser) {
                return {
                    type: "user-not-found"
                };
            } else if (existingUser.password !== action.payload.password) {
                return {
                    type: "authentication-failure"
                };
            } else {
                return {
                    type: "sign-in-success",
                    payload: {
                        user: existingUser,
                        projects: await state.projects.find<FoldrProject>({
                            owner: existingUser.id
                        })
                    }
                };
            }
        }
        case "create-project": {
            const authenticatedUser = await authenticate(
                state,
                action.meta.username,
                action.meta.password
            );
            if (!authenticatedUser) {
                return {
                    type: "authentication-failure"
                };
            }
            const existing = await state.projects.findOne<FoldrProject>({name: action.payload.name})
            if (existing) {
                return {
                    type: "duplicated-project"
                }
            }
            const project: FoldrProject = {
                id: Math.random().toString(),
                owner: authenticatedUser.id,
                name: action.payload.name,
                index: null,
                responder: null,
                distributor: null
            };
            await state.projects.upsert(project);
            return {
                type: "create-project-success",
                payload: {
                    project
                }
            };
        }
        case "update-project": {
            const authenticatedUser = await authenticate(
                state,
                action.meta.username,
                action.meta.password
            );
            if (!authenticatedUser) {
                return {
                    type: "authentication-failure"
                };
            }
            const claimedProject = await state.projects.findOne<FoldrProject>({
                name: action.payload.projectName
            });
            if (!claimedProject) {
                return {
                    type: "project-not-found"
                };
            } else if (claimedProject.owner !== authenticatedUser.id) {
                return {
                    type: "authorization-failure"
                };
            }
            if (action.payload.index !== undefined) {
                claimedProject.index = action.payload.index;
            }
            if (action.payload.responder !== undefined) {
                claimedProject.responder = action.payload.responder;
            }
            if (action.payload.distributor !== undefined) {
                claimedProject.distributor = action.payload.distributor;
            }
            await state.projects.upsert(claimedProject);
            return {
                type: "ok"
            };
        }
        case "get-self": {
            const authenticatedUser = await authenticate(
                state,
                action.meta.username,
                action.meta.password
            );
            if (!authenticatedUser) {
                return {
                    type: "authentication-failure"
                };
            }
            return {
                type: "provide-user",
                payload: {
                    user: authenticatedUser
                }
            };
        }
        default: {
            return {
                type: "unknown-action",
                error: true
            };
        }
    }
}
