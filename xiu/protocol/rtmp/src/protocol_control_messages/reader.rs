use {
    super::errors::ProtocolControlMessageReaderError,
    crate::messages::define::SetPeerBandwidthProperties, byteorder::BigEndian,
    bytesio::bytes_reader::BytesReader,
};

pub struct ProtocolControlMessageReader {
    pub reader: BytesReader,
}

impl ProtocolControlMessageReader {
    pub fn new(reader: BytesReader) -> Self {
        Self { reader: reader }
    }
    pub fn read_set_chunk_size(&mut self) -> Result<u32, ProtocolControlMessageReaderError> {
        let chunk_size = self.reader.read_u32::<BigEndian>()?;
        return Ok(chunk_size);
    }

    pub fn read_abort_message(&mut self) -> Result<u32, ProtocolControlMessageReaderError> {
        let chunk_stream_id = self.reader.read_u32::<BigEndian>()?;
        return Ok(chunk_stream_id);
    }

    pub fn read_acknowledgement(&mut self) -> Result<u32, ProtocolControlMessageReaderError> {
        let sequence_number = self.reader.read_u32::<BigEndian>()?;
        return Ok(sequence_number);
    }

    pub fn read_window_acknowledgement_size(
        &mut self,
    ) -> Result<u32, ProtocolControlMessageReaderError> {
        let window_acknowledgement_size = self.reader.read_u32::<BigEndian>()?;
        return Ok(window_acknowledgement_size);
    }

    pub fn read_set_peer_bandwidth(
        &mut self,
    ) -> Result<SetPeerBandwidthProperties, ProtocolControlMessageReaderError> {
        let window_size = self.reader.read_u32::<BigEndian>()?;
        let limit_type = self.reader.read_u8()?;

        return Ok(SetPeerBandwidthProperties::new(window_size, limit_type));
    }
}
