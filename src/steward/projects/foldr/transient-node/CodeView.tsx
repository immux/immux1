import * as React from "react";
import AceEditor from "react-ace";

import "brace/mode/javascript";
import "brace/mode/html";
import "brace/theme/monokai";

type LanguageOption = "javascript" | "html";

interface Props {
    code: string;
    language: LanguageOption;
    onChange(code: string): void;
}

interface State {
    editing: boolean;
    editingContent: string;
}

class CodeView extends React.Component<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = {
            editing: false,
            editingContent: ""
        };
    }

    render() {
        const { code } = this.props;
        return (
            <div>
                {code.length > 100 && (
                    <pre>
                        <code>{code.slice(0, 100)}</code>
                        ...
                    </pre>
                )}
                {code.length <= 100 && (
                    <pre>
                        <code>{code}</code>
                    </pre>
                )}
                {this.state.editing && (
                    <div
                        style={{
                            position: "fixed",
                            width: "95vw",
                            height: "95vh",
                            background: "#eee",
                            top: "2vh"
                        }}
                    >
                        <AceEditor
                            mode={this.props.language}
                            theme="monokai"
                            name="code"
                            fontSize={14}
                            showPrintMargin={true}
                            cursorStart={1}
                            highlightActiveLine={true}
                            style={{
                                width: "90%",
                                height: "90%",
                                margin: "0 auto"
                            }}
                            value={this.state.editingContent}
                            onChange={content =>
                                this.setState({
                                    editingContent: content
                                })
                            }
                        />
                        <button
                            onClick={() => {
                                this.setState({
                                    editing: false
                                });
                                this.props.onChange(this.state.editingContent);
                            }}
                        >
                            Confirm change
                        </button>
                        <button
                            onClick={() =>
                                this.setState({
                                    editing: false
                                })
                            }
                        >
                            Cancel
                        </button>
                    </div>
                )}
                <button
                    onClick={() => {
                        this.setState({
                            editing: true,
                            editingContent: code
                        });
                    }}
                >
                    Edit
                </button>
            </div>
        );
    }
}

export default CodeView;
