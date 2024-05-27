use {
    super::{
        define::{msg_type_id, RtmpMessageData},
        errors::{MessageError, MessageErrorValue},
    },
    crate::{
        amf0::{amf0_markers, amf0_reader::Amf0Reader},
        chunk::ChunkInfo,
        protocol_control_messages::reader::ProtocolControlMessageReader,
        user_control_messages::reader::EventMessagesReader,
        // utils,
    },
    bytesio::bytes_reader::BytesReader,
};

pub struct MessageParser {
    chunk_info: ChunkInfo,
}

impl MessageParser {
    pub fn new(chunk_info: ChunkInfo) -> Self {
        Self {
            chunk_info: chunk_info,
        }
    }
    pub fn parse(self) -> Result<RtmpMessageData, MessageError> {
        let mut reader = BytesReader::new(self.chunk_info.payload);

        match self.chunk_info.message_header.msg_type_id {
            msg_type_id::COMMAND_AMF0 | msg_type_id::COMMAND_AMF3 => {
                if self.chunk_info.message_header.msg_type_id == msg_type_id::COMMAND_AMF3 {
                    reader.read_u8()?;
                }
                let mut amf_reader = Amf0Reader::new(reader);

                let command_name = amf_reader.read_with_type(amf0_markers::STRING)?;
                let transaction_id = amf_reader.read_with_type(amf0_markers::NUMBER)?;

                // match command_name.clone() {
                //     Amf0ValueType::UTF8String(val) => {
                //         log::info!("command_name:{}", val);
                //     }
                //     _ => {}
                // }

                //The third value can be an object or NULL object
                let command_obj_raw = amf_reader.read_with_type(amf0_markers::OBJECT);
                let command_obj = match command_obj_raw {
                    Ok(val) => val,
                    Err(_) => amf_reader.read_with_type(amf0_markers::NULL)?,
                };

                let others = amf_reader.read_all()?;

                return Ok(RtmpMessageData::Amf0Command {
                    command_name: command_name,
                    transaction_id: transaction_id,
                    command_object: command_obj,
                    others,
                });
            }

            msg_type_id::AUDIO => {
                log::trace!(
                    "receive audio msg , msg length is{}\n",
                    self.chunk_info.message_header.msg_length
                );

                return Ok(RtmpMessageData::AudioData {
                    data: reader.extract_remaining_bytes(),
                });
            }
            msg_type_id::VIDEO => {
                log::trace!(
                    "receive video msg , msg length is{}\n",
                    self.chunk_info.message_header.msg_length
                );
                return Ok(RtmpMessageData::VideoData {
                    data: reader.extract_remaining_bytes(),
                });
            }
            msg_type_id::USER_CONTROL_EVENT => {
                log::trace!(
                    "receive user control event msg , msg length is{}\n",
                    self.chunk_info.message_header.msg_length
                );
                let data = EventMessagesReader::new(reader).parse_event()?;
                return Ok(data);
            }
            msg_type_id::SET_CHUNK_SIZE => {
                let chunk_size = ProtocolControlMessageReader::new(reader).read_set_chunk_size()?;
                return Ok(RtmpMessageData::SetChunkSize {
                    chunk_size: chunk_size,
                });
            }
            msg_type_id::ABORT => {
                let chunk_stream_id =
                    ProtocolControlMessageReader::new(reader).read_abort_message()?;
                return Ok(RtmpMessageData::AbortMessage {
                    chunk_stream_id: chunk_stream_id,
                });
            }
            msg_type_id::ACKNOWLEDGEMENT => {
                let sequence_number =
                    ProtocolControlMessageReader::new(reader).read_acknowledgement()?;
                return Ok(RtmpMessageData::Acknowledgement {
                    sequence_number: sequence_number,
                });
            }
            msg_type_id::WIN_ACKNOWLEDGEMENT_SIZE => {
                let size =
                    ProtocolControlMessageReader::new(reader).read_window_acknowledgement_size()?;
                return Ok(RtmpMessageData::WindowAcknowledgementSize { size: size });
            }
            msg_type_id::SET_PEER_BANDWIDTH => {
                let properties =
                    ProtocolControlMessageReader::new(reader).read_set_peer_bandwidth()?;
                return Ok(RtmpMessageData::SetPeerBandwidth {
                    properties: properties,
                });
            }
            msg_type_id::DATA_AMF0 | msg_type_id::DATA_AMF3 => {
                //let values = Amf0Reader::new(reader).read_all()?;
                return Ok(RtmpMessageData::AmfData {
                    raw_data: reader.extract_remaining_bytes(),
                });
            }

            msg_type_id::SHARED_OBJ_AMF3 | msg_type_id::SHARED_OBJ_AMF0 => {}

            msg_type_id::AGGREGATE => {}

            _ => {
                return Err(MessageError {
                    value: MessageErrorValue::UnknowMessageType,
                });
            }
        }
        return Err(MessageError {
            value: MessageErrorValue::UnknowMessageType,
        });
    }
}

#[cfg(test)]
mod tests {

    use super::MessageParser;
    use crate::chunk::unpacketizer::ChunkUnpacketizer;
    use crate::chunk::unpacketizer::UnpackResult;

    #[test]
    fn test_message_parse() {
        let mut unpacker = ChunkUnpacketizer::new();

        let data: [u8; 205] = [
            2, 0, 0, 0, 0, 0, 4, 1, 0, 0, 0, 0, 0, 0, 16, 0, //set chunk size
            //connect
            3, //|format+csid|
            0, 0, 0, //timestamp
            0, 0, 177, //msg_length
            20,  //msg_type_id 0x14
            0, 0, 0, 0, //msg_stream_id
            2, 0, 7, 99, 111, 110, 110, 101, 99, 116, 0, 63, 240, 0, 0, 0, 0, 0, 0, //body
            3, 0, 3, 97, 112, 112, 2, 0, 6, 104, 97, 114, 108, 97, 110, 0, 4, 116, 121, 112, 101,
            2, 0, 10, 110, 111, 110, 112, 114, 105, 118, 97, 116, 101, 0, 8, 102, 108, 97, 115,
            104, 86, 101, 114, 2, 0, 31, 70, 77, 76, 69, 47, 51, 46, 48, 32, 40, 99, 111, 109, 112,
            97, 116, 105, 98, 108, 101, 59, 32, 70, 77, 83, 99, 47, 49, 46, 48, 41, 0, 6, 115, 119,
            102, 85, 114, 108, 2, 0, 28, 114, 116, 109, 112, 58, 47, 47, 108, 111, 99, 97, 108,
            104, 111, 115, 116, 58, 49, 57, 51, 53, 47, 104, 97, 114, 108, 97, 110, 0, 5, 116, 99,
            85, 114, 108, 2, 0, 28, 114, 116, 109, 112, 58, 47, 47, 108, 111, 99, 97, 108, 104,
            111, 115, 116, 58, 49, 57, 51, 53, 47, 104, 97, 114, 108, 97, 110, 0, 0, 9,
        ];

        unpacker.extend_data(&data[..]);

        loop {
            let result = unpacker.read_chunk();

            let rv = match result {
                Ok(val) => val,
                Err(_) => {
                    print!("end-----------");
                    return;
                }
            };

            match rv {
                UnpackResult::ChunkInfo(chunk_info) => {
                    let _ = chunk_info.message_header.msg_streamd_id;
                    let _ = chunk_info.message_header.timestamp;

                    let mut message_parser = MessageParser::new(chunk_info);
                    let _ = message_parser.parse();
                }
                _ => {}
            }
        }
    }

    use uuid::Uuid;

    #[test]
    fn test_uuid() {
        let my_uuid = Uuid::new_v4();
        println!("{}", my_uuid);
    }
}
