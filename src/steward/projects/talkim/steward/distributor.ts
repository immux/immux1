import { ImmuxDbJS } from "../../../../connectors/typescript/immuxdb.types";
import { Distribution } from "../../meta";
import { CompositeAction, TalkimAction } from "../actions/Action";
import { User } from "../datatypes";
import { ProvideUsers } from "../actions/UserActions";

async function distributor(
    state: ImmuxDbJS,
    responseAction: TalkimAction
): Promise<Distribution[]> {
    console.info("distributor handling", responseAction);
    switch (responseAction.type) {
        case "send-message-success": {
            const authors: User[] = await state.users.find<User>({
                id: responseAction.message.sender
            });
            const provideUsers: ProvideUsers = {
                type: "provide-users",
                users: authors
            };
            const broadcastAction: CompositeAction = {
                type: "composite-action",
                actions: [responseAction, provideUsers]
            };
            return [
                {
                    type: "all-peers",
                    action: broadcastAction
                }
            ];
        }
        default: {
            console.info("distributor: none");
            return [];
        }
    }
}
