import * as http from "http";
import * as fetch from "isomorphic-fetch";
import {
    createImmuxDbViaHttpsRestrictedAccess,
    makeImmuxDBHttp
} from "../connectors/immuxdb";
import { FoldrProject } from "./projects/meta";
import {
    FOLDR_PROJECT_NAME,
    makeNamespaceForProject
} from "./projects/foldr/utils";

const hostname = "127.0.0.1";
const port = 10000;

function getProjectName(host?: string): string | null {
    if (!host) {
        return null;
    }
    const segments = host.split(".");
    if (segments.length < 3) {
        return null;
    }
    return segments[0];
}

const server = http.createServer(async (request, response) => {
    console.info(request.headers);
    const targetProjectName = getProjectName(request.headers.host);
    if (!targetProjectName) {
        response.end(
            JSON.stringify({
                type: "no-project-specified"
            })
        );
        return;
    }
    const db = makeImmuxDBHttp("localhost:1991", fetch);
    await db.switchNamespace(makeNamespaceForProject(FOLDR_PROJECT_NAME));
    const state = createImmuxDbViaHttpsRestrictedAccess(db);
    const project = await state.projects.findOne<FoldrProject>({
        name: targetProjectName
    });
    if (!project) {
        response.end(
            JSON.stringify({
                type: "no-project-found"
            })
        );
        return;
    }
    switch (request.method) {
        case "GET": {
            response.setHeader("Content-Type", "text/html; charset=utf-8");
            response.end(project.index);
            return;
        }
        case "POST": {
            let body = "";
            request.on("data", data => {
                body += data;
            });
            request.on("end", async () => {
                if (project.responder) {
                    await db.switchNamespace(
                        makeNamespaceForProject(targetProjectName)
                    );
                    const action = JSON.parse(body);
                    let result = await eval(`
                        (async function code() {
                          ${project.responder}
                          const action = ${JSON.stringify(action)};
                          const response = await responder(state, action);
                          return response;
                        })()
                        `);
                    response.end(JSON.stringify(result));
                } else {
                    response.end(
                        JSON.stringify({
                            type: "no-responder",
                            error: true
                        })
                    );
                }
            });
            return;
        }
        default: {
            response.end(
                JSON.stringify({
                    type: "unexpected-method",
                    error: true,
                    payload: {
                        method: request.method
                    }
                })
            );
            return;
        }
    }
});

server.listen(port, hostname, () => {
    console.log(`Server running at http://${hostname}:${port}/`);
});
