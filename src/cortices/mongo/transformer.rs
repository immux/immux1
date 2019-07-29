use bson::{Bson, Document};

use crate::config::ImmuxDBConfiguration;
use crate::cortices::mongo::ops::msg_header::MsgHeader;
use crate::cortices::mongo::ops::op::MongoOp;
use crate::cortices::mongo::ops::op_msg::{OpMsg, Section};

use crate::cortices::mongo::utils::{construct_single_doc_op_msg, is_1, make_bson_from_config};

use crate::declarations::commands::{
    Command, InsertCommand, InsertCommandSpec, Outcome, PickChainCommand, SelectCommand,
    SelectCondition,
};
use crate::declarations::errors::{ImmuxError, ImmuxResult};

#[derive(Debug)]
pub enum MongoTransformerError {
    GetObjectId,
    EncodeDocument,
    UnexpectedInputShape,
    UnimplementedWhereCondition(Bson),
    UnexpectedFilterDocument(Document),
    UnimplementedCommand,
    UnexpectedLastSection,
    NoSections,
    UnimplementedOp,
}

fn encode_document(doc: &Document) -> ImmuxResult<Vec<u8>> {
    let mut data: Vec<u8> = Vec::new();
    match bson::encode_document(&mut data, doc) {
        Err(_error) => Err(ImmuxError::MongoTransformer(
            MongoTransformerError::EncodeDocument,
        )),
        Ok(_) => Ok(data),
    }
}

fn get_obj_id(doc: &Document, key: &str) -> ImmuxResult<Vec<u8>> {
    match doc.get_object_id(key) {
        Err(_error) => Err(ImmuxError::MongoTransformer(
            MongoTransformerError::GetObjectId,
        )),
        Ok(object_id) => Ok(object_id.bytes().to_vec()),
    }
}

pub fn transform_mongo_op_to_command(op: &MongoOp) -> ImmuxResult<Command> {
    match op {
        MongoOp::Msg(op_msg) => {
            if let Some(last_section) = &op_msg.sections.last() {
                match last_section {
                    Section::Single(request_doc) => {
                        if is_1(request_doc.get_f64("isMaster")) {
                            if let Ok(target_db) = request_doc.get_str("$db") {
                                let command = PickChainCommand {
                                    new_chain_name: target_db.into(),
                                };
                                return Ok(Command::PickChain(command));
                            } else {
                                return Err(MongoTransformerError::UnexpectedInputShape.into());
                            }
                        } else if let Ok(collection) = request_doc.get_str("insert") {
                            if let Some(first_section) = op_msg.sections.first() {
                                match first_section {
                                    Section::Sequence(sequence) => {
                                        let mut targets: Vec<InsertCommandSpec> = Vec::new();
                                        for doc in &sequence.documents {
                                            let spec = InsertCommandSpec {
                                                id: get_obj_id(doc, "_id")?,
                                                value: encode_document(doc)?,
                                            };
                                            targets.push(spec)
                                        }
                                        let instruction = InsertCommand {
                                            targets,
                                            grouping: collection.as_bytes().to_vec(),
                                            insert_with_index: false,
                                        };
                                        Ok(Command::Insert(instruction))
                                    }
                                    Section::Single(_doc) => {
                                        Err(MongoTransformerError::UnexpectedInputShape.into())
                                    }
                                }
                            } else {
                                return Err(MongoTransformerError::UnexpectedInputShape.into());
                            }
                        } else if let Ok(collection) = request_doc.get_str("find") {
                            if let Ok(filter) = request_doc.get_document("filter") {
                                let grouping = collection.as_bytes().to_vec();
                                if filter.is_empty() {
                                    let command = SelectCommand {
                                        grouping,
                                        condition: SelectCondition::UnconditionalMatch,
                                    };
                                    Ok(Command::Select(command))
                                } else if let Some(where_condition) = filter.get("$where") {
                                    match where_condition {
                                        Bson::JavaScriptCode(code) => {
                                            let command = SelectCommand {
                                                grouping,
                                                condition: SelectCondition::JSCode(
                                                    code.to_string(),
                                                ),
                                            };
                                            Ok(Command::Select(command))
                                        }
                                        _ => {
                                            Err(MongoTransformerError::UnimplementedWhereCondition(
                                                where_condition.to_owned(),
                                            )
                                            .into())
                                        }
                                    }
                                } else {
                                    Err(MongoTransformerError::UnexpectedFilterDocument(
                                        filter.to_owned(),
                                    )
                                    .into())
                                }
                            } else {
                                Err(MongoTransformerError::UnexpectedInputShape.into())
                            }
                        } else {
                            Err(MongoTransformerError::UnimplementedCommand.into())
                        }
                    }
                    _ => Err(MongoTransformerError::UnexpectedLastSection.into()),
                }
            } else {
                Err(MongoTransformerError::NoSections.into())
            }
        }
        _ => Err(MongoTransformerError::UnimplementedOp.into()),
    }
}

fn get_header_from_op(op: &MongoOp) -> Result<MsgHeader, MongoTransformerError> {
    match op {
        MongoOp::Msg(msg) => Ok(msg.message_header.clone()),
        MongoOp::Reply(reply) => Ok(reply.message_header.clone()),
        MongoOp::Query(query) => Ok(query.message_header.clone()),
        _ => Err(MongoTransformerError::UnimplementedOp),
    }
}

pub fn transform_outcome_to_mongo_msg(
    outcome: &Outcome,
    config: &ImmuxDBConfiguration,
    incoming_op: &MongoOp,
) -> ImmuxResult<OpMsg> {
    let header = get_header_from_op(&incoming_op)?;
    match outcome {
        Outcome::PickChain(_ok) => {
            let result = make_bson_from_config(config);
            Ok(construct_single_doc_op_msg(result, &header))
        }
        Outcome::Insert(ok) => {
            let mut doc = Document::new();
            doc.insert("n", ok.count as i32);
            doc.insert("ok", 1.0);
            Ok(construct_single_doc_op_msg(doc, &header))
        }
        Outcome::Select(ok) => {
            let mut doc = Document::new();
            let mut cursor = Document::new();
            let documents: Vec<Bson> = ok
                .values
                .iter()
                .map(|value| {
                    let buffer = value.clone();
                    match bson::decode_document(&mut buffer.as_slice()) {
                        Err(_) => Bson::Document(Document::new()),
                        Ok(doc) => Bson::Document(doc),
                    }
                })
                .collect();
            cursor.insert("firstBatch", documents);
            cursor.insert("id", 0i64);
            cursor.insert("ns", ""); // Skipped actual implementation. See issue #82.
            doc.insert("cursor", cursor);
            doc.insert("ok", 1.0);
            Ok(construct_single_doc_op_msg(doc, &header))
        }
        Outcome::NameChain(_ok) => unimplemented!(),
        Outcome::CreateIndex(_ok) => unimplemented!(),
        Outcome::NameChain(_ok) => unimplemented!(),
        Outcome::Revert(_) => unimplemented!(),
        Outcome::RevertAll(_) => unimplemented!(),
        Outcome::Inspect(_) => unimplemented!(),
    }
}

#[cfg(test)]
mod mongo_command_transformer_tests {
    use bson::oid::ObjectId;
    use bson::spec::BinarySubtype;
    use bson::Bson;
    use bson::Document;

    use crate::cortices::mongo::ops::msg_header::MsgHeader;
    use crate::cortices::mongo::ops::op::MongoOp;
    use crate::cortices::mongo::ops::op_msg::Section::{Sequence, Single};
    use crate::cortices::mongo::ops::op_msg::{DocumentSequence, OpMsg, OpMsgFlags};
    use crate::cortices::mongo::ops::opcodes::MongoOpCode;
    use crate::cortices::mongo::transformer::transform_mongo_op_to_command;
    use crate::cortices::mongo::utils::construct_single_doc_op_msg;

    use crate::declarations::commands::{Command, SelectCondition};

    static HEADER: MsgHeader = MsgHeader {
        message_length: 0,
        request_id: 10,
        response_to: 0,
        op_code: MongoOpCode::OpMsg,
    };

    // use SOME_DB
    #[test]
    fn test_ismaster() {
        let mut doc = Document::new();
        let target_db_name = String::from("test");
        doc.insert("isMaster", 1.0);
        doc.insert("forShell", 1.0);
        doc.insert("$db", target_db_name.clone());

        let op = construct_single_doc_op_msg(doc, &HEADER);
        match transform_mongo_op_to_command(&MongoOp::Msg(op)) {
            Ok(Command::PickChain(pick_chain)) => {
                assert_eq!(pick_chain.new_chain_name, target_db_name.as_bytes());
            }
            Ok(_) => panic!("ismaster should be translated to pick_chain"),
            Err(error) => panic!("Failed to transform command {:#?}", error),
        }
    }

    // db.collection_name.insertMany([1, 2, 3])
    #[test]
    fn test_insert() {
        let data = vec![1u8, 2u8, 3u8];
        let collection = String::from("collection_name");
        let datum_key = "data";
        let op = OpMsg {
            message_header: HEADER.clone(),
            flags: OpMsgFlags {
                check_sum_present: false,
                more_to_come: false,
                exhaust_allowed: false,
            },
            sections: vec![
                Sequence(DocumentSequence {
                    section_size: 113,
                    identifier: String::from("documents"),
                    documents: data
                        .iter()
                        .map(|datum| {
                            let mut doc = Document::new();
                            doc.insert(
                                "_id",
                                bson::Bson::ObjectId(ObjectId::with_bytes([
                                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, *datum,
                                ])),
                            );
                            doc.insert(datum_key, *datum as i32);
                            doc
                        })
                        .collect(),
                }),
                Single({
                    let mut doc = Document::new();
                    doc.insert("insert", collection.clone());
                    doc.insert("ordered", true);
                    doc.insert("$db", "test");

                    let mut lsid = Document::new();
                    lsid.insert(
                        "id",
                        Bson::Binary(
                            BinarySubtype::Uuid,
                            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        ),
                    );
                    doc.insert("lsid", lsid);

                    doc
                }),
            ],
        };
        match transform_mongo_op_to_command(&MongoOp::Msg(op)) {
            Ok(Command::Insert(insert)) => {
                assert_eq!(data.len(), insert.targets.len());
                assert_eq!(insert.grouping, collection.as_bytes());
                let mut i = 0;
                while i < data.len() {
                    let datum = data[i];
                    let target = insert.targets[i].clone();
                    let doc_bytes = target.value.clone();
                    match bson::decode_document(&mut doc_bytes.as_slice()) {
                        Err(error) => panic!("Failed to parse document bytes: {:#?}", error),
                        Ok(doc) => match doc.get_i32(datum_key) {
                            Err(_error) => panic!("Missing datum on datum key {}", datum_key),
                            Ok(num) => assert_eq!(datum as i32, num),
                        },
                    }
                    i += 1;
                }
            }
            Ok(_) => panic!("Mongo insert should be translated to insert command"),
            Err(error) => panic!("Failed to transform command {:#?}", error),
        }
    }

    fn insert_adhoc_lsid(doc: &mut Document) {
        let mut lsid = Document::new();
        lsid.insert(
            "id",
            Bson::Binary(
                BinarySubtype::Uuid,
                vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ),
        );
        doc.insert("lsid", lsid);
    }

    // db.collection_name.find({})
    #[test]
    fn test_find_all() {
        let collection = String::from("Collection name");

        let mut doc = Document::new();
        doc.insert("find", collection.clone());
        let filter = Document::new();
        doc.insert("filter", filter);
        insert_adhoc_lsid(&mut doc);
        doc.insert("$db", "test");
        let op = construct_single_doc_op_msg(doc, &HEADER);
        match transform_mongo_op_to_command(&MongoOp::Msg(op)) {
            Ok(Command::Select(select)) => {
                assert_eq!(select.grouping, collection.as_bytes());
                match select.condition {
                    SelectCondition::UnconditionalMatch => (),
                    _ => panic!("select.condition is unexpected"),
                }
            }
            Ok(_) => panic!("Mongo find should be translated to select command"),
            Err(error) => panic!("Failed to transform command {:#?}", error),
        }
    }

    // db.collection_name.find(x => x == 1)
    #[test]
    fn test_find_by_javascript() {
        let collection = String::from("Collection name");
        let js_code = String::from("x => x == 1");

        let mut doc = Document::new();
        doc.insert("find", collection.clone());
        let mut filter = Document::new();
        filter.insert("$where", Bson::JavaScriptCode(js_code.clone()));
        doc.insert("filter", filter);
        insert_adhoc_lsid(&mut doc);
        doc.insert("$db", "test");
        let op = construct_single_doc_op_msg(doc, &HEADER);
        match transform_mongo_op_to_command(&MongoOp::Msg(op)) {
            Ok(Command::Select(select)) => {
                assert_eq!(select.grouping, collection.as_bytes());
                match select.condition {
                    SelectCondition::JSCode(transformed_code) => {
                        assert_eq!(transformed_code, js_code)
                    }
                    _ => panic!("select.condition is unexpected"),
                }
            }
            Ok(_) => panic!("Mongo find should be translated to select command"),
            Err(error) => panic!("Failed to transform command {:#?}", error),
        }
    }
}

#[cfg(test)]
mod mongo_outcome_transformer_tests {
    use bson::Document;

    use crate::config::ImmuxDBConfiguration;
    use crate::cortices::mongo::ops::msg_header::MsgHeader;
    use crate::cortices::mongo::ops::op::MongoOp;
    use crate::cortices::mongo::ops::op_msg::{OpMsg, OpMsgFlags, Section};
    use crate::cortices::mongo::ops::opcodes::MongoOpCode;
    use crate::cortices::mongo::transformer::transform_outcome_to_mongo_msg;

    use crate::declarations::commands::{InsertOutcome, Outcome, PickChainOutcome, SelectOutcome};

    #[test]
    fn test_pickchain() {
        let mock_config = ImmuxDBConfiguration::default();
        let mock_incoming_op = OpMsg {
            message_header: MsgHeader {
                message_length: 0,
                request_id: 0,
                response_to: 0,
                op_code: MongoOpCode::OpMsg,
            },
            flags: OpMsgFlags {
                check_sum_present: false,
                more_to_come: false,
                exhaust_allowed: false,
            },
            sections: vec![{
                let mut doc = Document::new();
                doc.insert("isMaster", 1.0);
                doc.insert("$db", String::from("target_db"));
                Section::Single(doc)
            }],
        };
        let outcome = PickChainOutcome {
            new_chain_name: "hello".as_bytes().to_vec(),
        };
        match transform_outcome_to_mongo_msg(
            &Outcome::PickChain(outcome),
            &mock_config,
            &MongoOp::Msg(mock_incoming_op),
        ) {
            Err(_error) => panic!("Cannot transform pickchain outcome"),
            Ok(op_msg) => {
                assert_eq!(op_msg.sections.len(), 1);
                let first_section = &op_msg.sections[0];
                match first_section {
                    Section::Single(doc) => {
                        assert_eq!(doc.get_bool("ismaster"), Ok(mock_config.is_master));
                        assert_eq!(
                            doc.get_i32("maxBsonObjectSize"),
                            Ok(mock_config.max_bson_object_size as i32),
                        );
                        assert_eq!(
                            doc.get_i32("maxMessageSizeBytes"),
                            Ok(mock_config.max_message_size_in_bytes as i32),
                        );
                        assert_eq!(
                            doc.get_i32("maxWriteBatchSize"),
                            Ok(mock_config.max_write_batch_size as i32)
                        );
                        assert!(doc.contains_key("localTime"));
                        assert_eq!(
                            doc.get_i32("logicalSessionTimeoutMinutes"),
                            Ok(mock_config.logical_session_timeout_minutes as i32),
                        );
                        assert_eq!(
                            doc.get_i32("minWireVersion"),
                            Ok(mock_config.min_mongo_wire_version as i32)
                        );
                        assert_eq!(
                            doc.get_i32("maxWireVersion"),
                            Ok(mock_config.max_mongo_wire_version as i32)
                        );
                        assert_eq!(doc.get_bool("readOnly"), Ok(mock_config.read_only));
                        assert_eq!(doc.get_f64("ok"), Ok(1.0));
                    }
                    _ => panic!("Unexpected section type"),
                }
            }
        }
    }

    #[test]
    fn test_insert() {
        let mock_config = ImmuxDBConfiguration::default();
        let mock_incoming_op = OpMsg {
            message_header: MsgHeader {
                message_length: 0,
                request_id: 0,
                response_to: 0,
                op_code: MongoOpCode::OpMsg,
            },
            flags: OpMsgFlags {
                check_sum_present: false,
                more_to_come: false,
                exhaust_allowed: false,
            },
            sections: vec![],
        };
        let outcome = InsertOutcome { count: 1024 };
        match transform_outcome_to_mongo_msg(
            &Outcome::Insert(outcome.clone()),
            &mock_config,
            &MongoOp::Msg(mock_incoming_op),
        ) {
            Err(_error) => panic!("Cannot transform insert outcome"),
            Ok(op_msg) => {
                assert_eq!(op_msg.sections.len(), 1);
                let first_section = &op_msg.sections[0];
                match first_section {
                    Section::Single(doc) => {
                        assert_eq!(doc.get_f64("ok"), Ok(1.0));
                        assert_eq!(doc.get_i32("n"), Ok(outcome.count as i32));
                    }
                    _ => panic!("Unexpected section type"),
                }
            }
        }
    }

    #[test]
    fn test_select() {
        let mock_config = ImmuxDBConfiguration::default();
        let mock_incoming_op = OpMsg {
            message_header: MsgHeader {
                message_length: 0,
                request_id: 0,
                response_to: 0,
                op_code: MongoOpCode::OpMsg,
            },
            flags: OpMsgFlags {
                check_sum_present: false,
                more_to_come: false,
                exhaust_allowed: false,
            },
            sections: vec![],
        };
        let outcome = SelectOutcome { values: vec![] };
        match transform_outcome_to_mongo_msg(
            &Outcome::Select(outcome.clone()),
            &mock_config,
            &MongoOp::Msg(mock_incoming_op),
        ) {
            Err(_error) => panic!("Cannot transform select outcome"),
            Ok(op_msg) => {
                assert_eq!(op_msg.sections.len(), 1);
                let first_section = &op_msg.sections[0];
                match first_section {
                    Section::Single(doc) => {
                        assert_eq!(doc.get_f64("ok"), Ok(1.0));
                    }
                    _ => panic!("Unexpected section type"),
                }
            }
        }
    }
}
