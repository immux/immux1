import * as fs from "fs";
import * as fetch from "isomorphic-fetch";
import { promisify } from "util";
import { FoldrProject } from "../meta";
import { FOLDR_PROJECT_NAME, makeNamespaceForProject } from "../utils";
import { adminId, foldrId } from "../init";
import { createImmuxDbViaHttpsRestrictedAccess, makeImmuxDBHttp } from "../../../connectors/typescript/immuxdb";

const readFileAsync = promisify(fs.readFile);

async function update() {
    const index = (await readFileAsync("build/index.html")).toString();
    const responder = (await readFileAsync("build/responder.js")).toString();

    const db = makeImmuxDBHttp("localhost:1991", fetch);
    await db.switchNamespace(makeNamespaceForProject(FOLDR_PROJECT_NAME));
    const state = createImmuxDbViaHttpsRestrictedAccess(db);

    const currentFoldr = await state.projects.findOne({ id: foldrId });

    if (currentFoldr) {
        const nextProject = {
            ...currentFoldr,
            index,
            responder
        };
        await state.projects.upsert(nextProject);
        console.info(`Found old foldr project, replaced`);
    } else {
        const project: FoldrProject = {
            id: foldrId,
            owner: adminId,
            name: "foldr",
            index,
            responder,
            distributor: null
        };
        await state.projects.upsert(project);
        console.info(
            `Existing foldr not found, inserted project ${project.id}`
        );
    }
}

!update();
