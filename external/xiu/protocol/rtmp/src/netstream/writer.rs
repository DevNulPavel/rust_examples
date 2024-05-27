use {
    super::errors::NetStreamError,
    crate::{
        amf0::{amf0_writer::Amf0Writer, define::Amf0ValueType},
        chunk::{chunk::ChunkInfo, define as chunk_define, packetizer::ChunkPacketizer},
        messages::define as messages_define,
    },
    bytesio::{bytes_writer::BytesWriter, bytesio::BytesIO},
    std::{collections::HashMap, sync::Arc},
    tokio::sync::Mutex,
};

pub struct NetStreamWriter {
    amf0_writer: Amf0Writer,
    packetizer: ChunkPacketizer,
}

impl NetStreamWriter {
    pub fn new(io: Arc<Mutex<BytesIO>>) -> Self {
        Self {
            amf0_writer: Amf0Writer::new(BytesWriter::new()),
            packetizer: ChunkPacketizer::new(io),
        }
    }
    async fn write_chunk(&mut self) -> Result<(), NetStreamError> {
        let data = self.amf0_writer.extract_current_bytes();

        let mut chunk_info = ChunkInfo::new(
            chunk_define::csid_type::COMMAND_AMF0_AMF3,
            chunk_define::chunk_type::TYPE_0,
            0,
            data.len() as u32,
            messages_define::msg_type_id::COMMAND_AMF0,
            0,
            data,
        );

        self.packetizer.write_chunk(&mut chunk_info).await?;
        Ok(())
    }
    pub async fn write_play(
        &mut self,
        transaction_id: &f64,
        stream_name: &String,
        start: &f64,
        duration: &f64,
        reset: &bool,
    ) -> Result<(), NetStreamError> {
        self.amf0_writer.write_string(&String::from("play"))?;
        self.amf0_writer.write_number(transaction_id)?;
        self.amf0_writer.write_null()?;
        self.amf0_writer.write_string(stream_name)?;
        self.amf0_writer.write_number(start)?;
        self.amf0_writer.write_number(duration)?;
        self.amf0_writer.write_bool(reset)?;

        self.write_chunk().await
    }
    pub async fn write_delete_stream(
        &mut self,
        transaction_id: &f64,
        stream_id: &f64,
    ) -> Result<(), NetStreamError> {
        self.amf0_writer
            .write_string(&String::from("deleteStream"))?;
        self.amf0_writer.write_number(transaction_id)?;
        self.amf0_writer.write_null()?;
        self.amf0_writer.write_number(stream_id)?;

        self.write_chunk().await
    }

    pub async fn write_close_stream(
        &mut self,
        transaction_id: &f64,
        stream_id: &f64,
    ) -> Result<(), NetStreamError> {
        self.amf0_writer
            .write_string(&String::from("closeStream"))?;
        self.amf0_writer.write_number(transaction_id)?;
        self.amf0_writer.write_null()?;
        self.amf0_writer.write_number(stream_id)?;

        self.write_chunk().await
    }

    pub async fn write_release_stream(
        &mut self,
        transaction_id: &f64,
        stream_name: &String,
    ) -> Result<(), NetStreamError> {
        self.amf0_writer
            .write_string(&String::from("releaseStream"))?;
        self.amf0_writer.write_number(transaction_id)?;
        self.amf0_writer.write_null()?;
        self.amf0_writer.write_string(stream_name)?;

        self.write_chunk().await
    }

    pub async fn write_fcpublish(
        &mut self,
        transaction_id: &f64,
        stream_name: &String,
    ) -> Result<(), NetStreamError> {
        self.amf0_writer.write_string(&String::from("FCPublish"))?;
        self.amf0_writer.write_number(transaction_id)?;
        self.amf0_writer.write_null()?;
        self.amf0_writer.write_string(stream_name)?;

        self.write_chunk().await
    }

    #[allow(dead_code)]
    async fn write_receive_audio(
        &mut self,
        transaction_id: &f64,
        enable: &bool,
    ) -> Result<(), NetStreamError> {
        self.amf0_writer
            .write_string(&String::from("receiveAudio"))?;
        self.amf0_writer.write_number(transaction_id)?;
        self.amf0_writer.write_null()?;
        self.amf0_writer.write_bool(enable)?;

        self.write_chunk().await
    }
    #[allow(dead_code)]
    async fn write_receive_video(
        &mut self,
        transaction_id: &f64,
        enable: &bool,
    ) -> Result<(), NetStreamError> {
        self.amf0_writer
            .write_string(&String::from("receiveVideo"))?;
        self.amf0_writer.write_number(transaction_id)?;
        self.amf0_writer.write_null()?;
        self.amf0_writer.write_bool(enable)?;

        self.write_chunk().await
    }
    pub async fn write_publish(
        &mut self,
        transaction_id: &f64,
        stream_name: &String,
        stream_type: &String,
    ) -> Result<(), NetStreamError> {
        self.amf0_writer.write_string(&String::from("publish"))?;
        self.amf0_writer.write_number(transaction_id)?;
        self.amf0_writer.write_null()?;
        self.amf0_writer.write_string(stream_name)?;
        self.amf0_writer.write_string(stream_type)?;

        self.write_chunk().await
    }
    #[allow(dead_code)]
    async fn write_seek(&mut self, transaction_id: &f64, ms: &f64) -> Result<(), NetStreamError> {
        self.amf0_writer.write_string(&String::from("seek"))?;
        self.amf0_writer.write_number(transaction_id)?;
        self.amf0_writer.write_null()?;
        self.amf0_writer.write_number(ms)?;

        self.write_chunk().await
    }
    #[allow(dead_code)]
    async fn write_pause(
        &mut self,
        transaction_id: &f64,
        pause: &bool,
        ms: &f64,
    ) -> Result<(), NetStreamError> {
        self.amf0_writer.write_string(&String::from("pause"))?;
        self.amf0_writer.write_number(transaction_id)?;
        self.amf0_writer.write_null()?;
        self.amf0_writer.write_bool(pause)?;
        self.amf0_writer.write_number(ms)?;

        self.write_chunk().await
    }

    #[allow(dead_code)]
    async fn write_on_bw_done(
        &mut self,
        transaction_id: &f64,
        bandwidth: &f64,
    ) -> Result<(), NetStreamError> {
        self.amf0_writer.write_string(&String::from("onBWDone"))?;
        self.amf0_writer.write_number(transaction_id)?;
        self.amf0_writer.write_null()?;
        self.amf0_writer.write_number(bandwidth)?;

        self.write_chunk().await
    }

    pub async fn write_on_status(
        &mut self,
        transaction_id: &f64,
        level: &String,
        code: &String,
        description: &String,
    ) -> Result<(), NetStreamError> {
        self.amf0_writer.write_string(&String::from("onStatus"))?;
        self.amf0_writer.write_number(transaction_id)?;
        self.amf0_writer.write_null()?;

        let mut properties_map = HashMap::new();

        properties_map.insert(
            String::from("level"),
            Amf0ValueType::UTF8String(level.clone()),
        );
        properties_map.insert(
            String::from("code"),
            Amf0ValueType::UTF8String(code.clone()),
        );
        properties_map.insert(
            String::from("description"),
            Amf0ValueType::UTF8String(description.clone()),
        );

        self.amf0_writer.write_object(&properties_map)?;

        self.write_chunk().await
    }
}
