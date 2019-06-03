import { FoldrAction } from "../types";
import { FoldrProject } from "../../meta";
import { dedupeById } from "./web";
import { FoldrClientState } from "./types";

export const INITIAL_STATE: FoldrClientState = {
    projects: [],
    me: null
};

export function reducer(
    state: FoldrClientState = INITIAL_STATE,
    action: FoldrAction
) {
    switch (action.type) {
        case "sign-in-success": {
            return {
                ...state,
                me: action.payload.user,
                projects: [
                    ...state.projects,
                    ...action.payload.projects
                ].reduce<FoldrProject[]>(dedupeById, [])
            };
        }
        case "create-project-success": {
            return {
                ...state,
                projects: [...state.projects, action.payload.project]
            };
        }
        default: {
            return state;
        }
    }
}
