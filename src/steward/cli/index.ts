#!/usr/bin/env node

import * as fs from "fs"
import * as path from "path"
import { exec } from 'child_process'
import { promisify } from "util"
import * as readline from "readline-sync"
import * as fetch from "isomorphic-fetch"

import { copy } from "fs-extra"

const mkdirAsync = promisify(fs.mkdir)
const writeFileAsync = promisify(fs.writeFile)
const readFileAsync = promisify(fs.readFile)
const copyAsync = promisify(copy)
const execAsync = promisify(exec)

const TEMPLATE_NAME = "skeleton"

const PROJECT_CONFIG_NAME = 'foldr.json'

import { CreateProject, FoldrAction, UpdateProject } from "../projects/foldr/types";
import { HTTP_SUBPATH } from "../paths";

async function readTextFile(name: string): Promise<string | null> {
    try {
        return (await readFileAsync(name)).toString();
    }
    catch (e) {
        return null
    }
}

interface CreateProjectOptions {
    projectName: string
}

interface ProjectConfig {
    name: string
    remote: string
}

async function create(options: CreateProjectOptions) {
    const { projectName } = options
    await mkdirAsync(projectName)
    await copyAsync(path.join(__dirname, TEMPLATE_NAME), projectName)

    const config: ProjectConfig = {
        name: projectName,
        remote: "foldr.foldr.site"
    }

    await writeFileAsync(`${projectName}/${PROJECT_CONFIG_NAME}`, JSON.stringify(config, null, 4))
}

async function build() {
    return await execAsync("./build.sh")
}

interface UploadProjectOptions {
    username: string
    password: string
    project: ProjectConfig
}

async function upload(options: UploadProjectOptions) {

    const {username, password, project} = options

    const index = await readTextFile("build/index.html");
    const responder = await readTextFile("build/responder.js");
    const distributor = await readTextFile("build/distributor.js");

    const endpoint = `http://${options.project.remote}${HTTP_SUBPATH}`
    console.info(`Uploading to ${endpoint}`)

    const createProjectAction: CreateProject = {
        type: "create-project",
        payload: {
            name: project.name
        },
        meta: {
            username,
            password,
        }
    }
    const createProjectResponse = await fetch(endpoint, {
        method: "POST",
        body: JSON.stringify(createProjectAction)
    })
    console.info('create', await createProjectResponse.text())

    const updateProjectAction: UpdateProject = {
        type: "update-project",
        payload: {
            projectName: project.name,
            index,
            responder,
            distributor,
        },
        meta: {
            username,
            password,
        }
    }
    const updateProjectResponse = await fetch(endpoint, {
        method: "POST",
        body: JSON.stringify(updateProjectAction)
    })
    console.info('update', await updateProjectResponse.text())
}

async function main() {
    console.info(__dirname)
    const command = process.argv[2];
    if (!command) {
        return
    }
    switch (command) {
        case "create": {
            const projectName = process.argv[3];
            if (!projectName) {
                console.info("Missing project name")
            }
            await create({
                projectName
            })

            break;
        }
        case "build": {
            await build();
            break;
        }
        case "upload": {
            const project: ProjectConfig = JSON.parse((await readFileAsync(PROJECT_CONFIG_NAME)).toString())
            const username = readline.question("Username: ")
            const password = readline.question("Password: ", {
                hideEchoBack: true,
            })
            await upload({
                username,
                password,
                project,
            })
            break;
        }
        default: {
            console.info(`Unknown command ${command}`)
        }
    }
    console.info("End!")
}

!!main()
