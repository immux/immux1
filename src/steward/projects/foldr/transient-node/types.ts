import { FoldrProject } from "../../meta";
import { UserData } from "../types";

export interface FoldrClientState {
    projects: FoldrProject[];
    me: UserData | null;
}
