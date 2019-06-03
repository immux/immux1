import * as React from "react";
import { FoldrProject } from "../../meta";
import CodeView from "./CodeView";

function ProjectView(props: {
    project: FoldrProject;
    onIndexChange(code: string): void;
    onResponderChange(code: string): void;
}) {
    const url = getProjectUrl(
        location.protocol,
        location.origin,
        props.project.name
    );
    return (
        <div
            key={props.project.id}
            style={{
                border: "1px solid #999",
                padding: "1em"
            }}
        >
            <h2
                style={{
                    borderBottom: "1px solid black"
                }}
            >
                {props.project.name}
            </h2>
            <div>
                <a href={url}>{url}</a>
            </div>
            <div>
                <h3>Index</h3>
                <CodeView
                    language="html"
                    code={props.project.index || ""}
                    onChange={props.onIndexChange}
                />
            </div>
            <div>
                <h3>Responder</h3>
                <CodeView
                    language="javascript"
                    code={props.project.responder || ""}
                    onChange={props.onResponderChange}
                />
            </div>
        </div>
    );
}

function getProjectUrl(
    protocol: string,
    host: string,
    projectName: string
): string {
    const newOrigin = [projectName, ...host.split(".").slice(1)].join(".");
    return `${protocol}//${newOrigin}`;
}

interface Props {
    projects: FoldrProject[];
    onIndexChange(code: string, project: FoldrProject): void;
    onResponderChange(code: string, project: FoldrProject): void;
    onCreateProject(name: string): void;
}

interface State {
    newProjectName: string;
}

class ProjectPanel extends React.PureComponent<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = {
            newProjectName: ""
        };
    }

    render() {
        return (
            <div className="projects">
                <h3>Your projects</h3>
                {this.props.projects.map(project => (
                    <ProjectView
                        key={project.id}
                        project={project}
                        onIndexChange={code =>
                            this.props.onIndexChange(code, project)
                        }
                        onResponderChange={code =>
                            this.props.onResponderChange(code, project)
                        }
                    />
                ))}
                <div>
                    <h3>Create new project</h3>
                    <div>
                        <input
                            placeholder="Project name"
                            value={this.state.newProjectName}
                            onChange={event =>
                                this.setState({
                                    newProjectName: event.target.value
                                })
                            }
                        />
                        <button
                            onClick={() =>
                                this.props.onCreateProject(
                                    this.state.newProjectName
                                )
                            }
                        >
                            Create
                        </button>
                    </div>
                </div>
            </div>
        );
    }
}

export default ProjectPanel;
