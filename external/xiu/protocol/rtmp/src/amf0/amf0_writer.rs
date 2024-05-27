use {
    super::{amf0_markers, errors::Amf0WriteErrorValue, Amf0ValueType, Amf0WriteError},
    byteorder::BigEndian,
    bytes::BytesMut,
    bytesio::bytes_writer::BytesWriter,
    std::collections::HashMap,
};

pub struct Amf0Writer {
    writer: BytesWriter,
}

impl Amf0Writer {
    pub fn new(writer: BytesWriter) -> Self {
        Self { writer: writer }
    }
    pub fn write_anys(&mut self, values: &Vec<Amf0ValueType>) -> Result<(), Amf0WriteError> {
        for val in values {
            self.write_any(val)?;
        }

        Ok(())
    }
    pub fn write_any(&mut self, value: &Amf0ValueType) -> Result<(), Amf0WriteError> {
        match *value {
            Amf0ValueType::Boolean(ref val) => self.write_bool(&val),
            Amf0ValueType::Null => self.write_null(),
            Amf0ValueType::Number(ref val) => self.write_number(&val),
            Amf0ValueType::UTF8String(ref val) => self.write_string(&val),
            Amf0ValueType::Object(ref val) => self.write_object(&val),
            _ => Ok(()),
        }
    }

    pub fn write_number(&mut self, value: &f64) -> Result<(), Amf0WriteError> {
        self.writer.write_u8(amf0_markers::NUMBER)?;
        self.writer.write_f64::<BigEndian>(value.clone())?;
        Ok(())
    }

    pub fn write_bool(&mut self, value: &bool) -> Result<(), Amf0WriteError> {
        self.writer.write_u8(amf0_markers::BOOLEAN)?;
        self.writer.write_u8(value.clone() as u8)?;
        Ok(())
    }

    pub fn write_string(&mut self, value: &String) -> Result<(), Amf0WriteError> {
        if value.len() > (u16::max_value() as usize) {
            return Err(Amf0WriteError {
                value: Amf0WriteErrorValue::NormalStringTooLong,
            });
        }

        self.writer.write_u8(amf0_markers::STRING)?;
        self.writer.write_u16::<BigEndian>(value.len() as u16)?;
        self.writer.write(value.as_bytes())?;

        Ok(())
    }

    pub fn write_null(&mut self) -> Result<(), Amf0WriteError> {
        self.writer.write_u8(amf0_markers::NULL)?;
        Ok(())
    }

    pub fn write_object_eof(&mut self) -> Result<(), Amf0WriteError> {
        self.writer
            .write_u24::<BigEndian>(amf0_markers::OBJECT_END as u32)?;
        Ok(())
    }

    pub fn write_object(
        &mut self,
        properties: &HashMap<String, Amf0ValueType>,
    ) -> Result<(), Amf0WriteError> {
        self.writer.write_u8(amf0_markers::OBJECT)?;

        for (key, value) in properties {
            self.writer.write_u16::<BigEndian>(key.len() as u16)?;
            self.writer.write(key.as_bytes())?;
            self.write_any(value)?;
        }

        self.write_object_eof()?;
        Ok(())
    }

    // pub async fn flush(&mut self) -> Result<(), Amf0WriteError> {
    //     self.writer.flush()?;
    // }

    pub fn extract_current_bytes(&mut self) -> BytesMut {
        self.writer.extract_current_bytes()
    }

    pub fn get_current_bytes(&mut self) -> BytesMut {
        self.writer.get_current_bytes()
    }

    pub fn len(&self) -> usize {
        self.writer.len()
    }
}
