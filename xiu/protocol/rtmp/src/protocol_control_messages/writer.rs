use {
    super::errors::ControlMessagesError, crate::messages::define::msg_type_id,
    byteorder::BigEndian, bytesio::bytes_writer::AsyncBytesWriter,
};

pub struct ProtocolControlMessagesWriter {
    writer: AsyncBytesWriter,
    //amf0_writer: Amf0Writer,
}

impl ProtocolControlMessagesWriter {
    pub fn new(writer: AsyncBytesWriter) -> Self {
        Self { writer: writer }
    }
    pub fn write_control_message_header(
        &mut self,
        msg_type_id: u8,
        len: u32,
    ) -> Result<(), ControlMessagesError> {
        //0 1 2 3 4 5 6 7
        //+-+-+-+-+-+-+-+-+
        //|fmt|  cs id  |
        //+-+-+-+-+-+-+-+-+
        // 0x0     0x02
        self.writer.write_u8(0x0 << 6 | 0x02)?; //fmt 0 and csid 2
        self.writer.write_u24::<BigEndian>(0)?; //timestamp 3 bytes and value 0
        self.writer.write_u24::<BigEndian>(len)?; //msg length
        self.writer.write_u8(msg_type_id)?; //msg type id
        self.writer.write_u32::<BigEndian>(0)?; //msg stream ID 0

        Ok(())
    }
    pub async fn write_set_chunk_size(
        &mut self,
        chunk_size: u32,
    ) -> Result<(), ControlMessagesError> {
        self.write_control_message_header(msg_type_id::SET_CHUNK_SIZE, 4)?;
        self.writer
            .write_u32::<BigEndian>(chunk_size & 0x7FFFFFFF)?; //first bit must be 0

        self.writer.flush().await?;
        Ok(())
    }

    pub async fn write_abort_message(
        &mut self,
        chunk_stream_id: u32,
    ) -> Result<(), ControlMessagesError> {
        self.write_control_message_header(msg_type_id::ABORT, 4)?;
        self.writer.write_u32::<BigEndian>(chunk_stream_id)?;

        self.writer.flush().await?;
        Ok(())
    }

    pub async fn write_acknowledgement(
        &mut self,
        sequence_number: u32,
    ) -> Result<(), ControlMessagesError> {
        self.write_control_message_header(msg_type_id::ACKNOWLEDGEMENT, 4)?;
        self.writer.write_u32::<BigEndian>(sequence_number)?;

        self.writer.flush().await?;
        Ok(())
    }

    pub async fn write_window_acknowledgement_size(
        &mut self,
        window_size: u32,
    ) -> Result<(), ControlMessagesError> {
        self.write_control_message_header(msg_type_id::WIN_ACKNOWLEDGEMENT_SIZE, 4)?;
        self.writer.write_u32::<BigEndian>(window_size)?;

        self.writer.flush().await?;
        Ok(())
    }

    pub async fn write_set_peer_bandwidth(
        &mut self,
        window_size: u32,
        limit_type: u8,
    ) -> Result<(), ControlMessagesError> {
        self.write_control_message_header(msg_type_id::SET_PEER_BANDWIDTH, 5)?;
        self.writer.write_u32::<BigEndian>(window_size)?;
        self.writer.write_u8(limit_type)?;

        self.writer.flush().await?;

        Ok(())
    }
}
