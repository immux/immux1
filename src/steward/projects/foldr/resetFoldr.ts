import * as fs from "fs";
import * as fetch from "isomorphic-fetch";
import { promisify } from "util";
import {
    createImmuxDbViaHttpsRestrictedAccess,
    makeImmuxDBHttp
} from "../../../connectors/immuxdb";
import { FOLDR_PROJECT_NAME, makeNamespaceForProject } from "../utils";
import { FoldrProject } from "../meta";
import { UserData } from "./types";
import { adminId, foldrId } from "../init";

const readFileAsync = promisify(fs.readFile);

async function inject() {
    const index = (await readFileAsync("build/index.html")).toString();
    const responder = (await readFileAsync("build/responder.js")).toString();

    const db = makeImmuxDBHttp("localhost:1991", fetch);
    await db.switchNamespace(makeNamespaceForProject(FOLDR_PROJECT_NAME));
    const state = createImmuxDbViaHttpsRestrictedAccess(db);

    const admin: UserData = {
        id: adminId,
        username: "admin",
        password: "admin"
    };
    await state.users.upsert(admin);
    console.info(
        `Inserted user ${admin.id}, username ${admin.username}, password ${
            admin.password
        }`
    );

    const project: FoldrProject = {
        id: foldrId,
        owner: admin.id,
        name: "foldr",
        index,
        responder,
        distributor: null
    };
    await state.projects.upsert(project);
    console.info(`Inserted project ${project.id}`);
}

!inject();
