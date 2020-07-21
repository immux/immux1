import * as http from "http";
import * as ws from "ws";
import * as fetch from "isomorphic-fetch";
import {
    Distribution,
    FoldrProject,
    StewardEnhancedIncoming,
    StewardEnhancedOutgoing
} from "./projects/meta";
import {
    ActionfulAction,
    FoldrAction,
    NoResponder,
    NotFound,
    ProjectNotFound,
    URLParsingError,
} from "./projects/foldr/types";
import { ImmuxDbJS } from "../connectors/typescript/immuxdb.types";

import { FOLDR_PROJECT_NAME, makeNamespaceForProject } from "./projects/utils";
import { IncomingMessage } from "http";
import { createImmuxDbViaHttpsRestrictedAccess, ImmuxDBHttp, makeImmuxDBHttp } from "../connectors/typescript/immuxdb";

interface StewardSocket extends ws {
    host?: string;
}

const hostname = "127.0.0.1";
const ports = {
    http: 10000,
    ws: 11000
};

function getProjectNameFromUrl(host?: string): string | null {
    if (!host) {
        return null;
    }
    const segments = host.split(".");
    if (segments.length < 3) {
        return null;
    }
    return segments[0];
}

async function execute<Action extends ActionfulAction = ActionfulAction>(
    state: ImmuxDbJS,
    action: Action,
    project: FoldrProject,
    db: ImmuxDBHttp,
    request: IncomingMessage,
    peers?: Set<StewardSocket>
): Promise<Action> {
    const enhancedAction: StewardEnhancedIncoming<Action> = {
        ...action,
        steward: {
            cookie: request.headers.cookie || "",
            host: request.headers.host || ""
        }
    };
    console.info(project)
    if (project.responder) {
        await db.switchNamespace(makeNamespaceForProject(project.name));
        let result: StewardEnhancedOutgoing<Action> = await eval(`
                        (async function code() {
                          ${project.responder}
                          const action = ${JSON.stringify(enhancedAction)};
                          const response = await responder(state, action);
                          return response;
                        })()
                        `);

        if (project.distributor) {
            let distributions: Distribution[] = await eval(`
                        (async function code() {
                          ${project.distributor}
                          const distributions = await distributor(state, result);
                          return distributions;
                        })()
                        `);
            for (const distribution of distributions) {
                switch (distribution.type) {
                    case "all-peers": {
                        if (peers) {
                            for (const peer of peers) {
                                if (peer.host === enhancedAction.steward.host) {
                                    peer.send(
                                        JSON.stringify(distribution.action)
                                    );
                                }
                            }
                        }
                        break;
                    }
                    default: {
                        // Noop
                    }
                }
            }
        }

        return result;
    } else {
        const result: NoResponder = {
            type: "no-responder",
            project: project.name
        }
        return result as unknown as Action;
    }
}

async function getProject(
    db: ImmuxDBHttp,
    state: ImmuxDbJS,
    targetProjectName: string
): Promise<FoldrProject | null> {
    await db.switchNamespace(makeNamespaceForProject(FOLDR_PROJECT_NAME));
    const project = await state.projects.findOne<FoldrProject>({
        name: targetProjectName
    });
    return project;
}

const projectNotFound: ProjectNotFound = {
    type: "project-not-found"
};

function urlParsingError(url: string): URLParsingError {
    return {
        type: "url-parsing-error",
        url,
    }
}

function initializeHttpServer(db: ImmuxDBHttp, state: ImmuxDbJS) {
    const httpServer = http.createServer(async (request, response) => {
        const targetProjectName = getProjectNameFromUrl(request.headers.host);
        if (!targetProjectName) {
            response.end(JSON.stringify(urlParsingError(request.headers.host || "")));
            return;
        }
        await db.switchNamespace(makeNamespaceForProject(FOLDR_PROJECT_NAME));
        const project = await getProject(db, state, targetProjectName);
        if (!project) {
            response.end(JSON.stringify(projectNotFound));
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
                    const action = JSON.parse(body);
                    const result: StewardEnhancedOutgoing<
                        ActionfulAction
                    > = await execute(state, action, project, db, request);
                    if (result.steward && result.steward.cookie) {
                        response.setHeader("Set-Cookie", result.steward.cookie);
                        delete result.steward;
                    }
                    response.end(JSON.stringify(result));
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

    httpServer.listen(ports.http, hostname, () => {
        console.log(`http server running at http://${hostname}:${ports.http}/`);
    });
}

function initializeWebSocketServer(db: ImmuxDBHttp, state: ImmuxDbJS) {
    const ALLOWED_WEBSOCKET_HOSTS = ["foldr.test:8888", "foldr.site"];

    const verifyClient: ws.VerifyClientCallbackSync = info => {

        // Preventing a.foldr.site connecting to ws://b.foldr.site/api/ws
        const sourceTargetMatch =
            !!info.req.headers.host &&
            info.origin
                .replace("http://", "")
                .replace("https://", "")
                .startsWith(info.req.headers.host);
        const hostAllowed = ALLOWED_WEBSOCKET_HOSTS.some(host =>
            info.origin.endsWith(host)
        );

        return sourceTargetMatch && hostAllowed;
    };

    const wsServer = new ws.Server(
        {
            port: ports.ws,
            perMessageDeflate: {
                threshold: 4096
            },
            verifyClient
        },
        () => {
            console.log(`ws server running at ws://${hostname}:${ports.ws}/`);
        }
    );

    wsServer.on("connection", (socket: StewardSocket, request) => {
        console.info("connection", request.url, request.headers.host);
        socket.on("message", async message => {
            const targetProjectName = getProjectNameFromUrl(
                request.headers.host
            );
            socket.host = request.headers.host;
            let result: StewardEnhancedOutgoing<FoldrAction>;
            if (!targetProjectName) {
                result = urlParsingError(request.headers.host || "");
            } else {
                await db.switchNamespace(
                    makeNamespaceForProject(FOLDR_PROJECT_NAME)
                );
                const project = await getProject(db, state, targetProjectName);
                if (!project) {
                    result = projectNotFound;
                } else {
                    const action = JSON.parse(message.toString());
                    result = await execute(
                        state,
                        action,
                        project,
                        db,
                        request,
                        wsServer.clients
                    );
                }
            }
            delete result.steward;
            socket.send(JSON.stringify(result));
        });
    });

    wsServer.on("error", error => {
        console.info("ws server error", error);
    });
}

function setup() {
    const dbViaHttp = makeImmuxDBHttp("localhost:1991", fetch);
    const state = createImmuxDbViaHttpsRestrictedAccess(dbViaHttp);
    initializeHttpServer(dbViaHttp, state);
    initializeWebSocketServer(dbViaHttp, state);
}

setup();
