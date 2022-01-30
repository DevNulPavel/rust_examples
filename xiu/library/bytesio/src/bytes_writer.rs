use {
    super::{
        bytes_errors::{BytesWriteError, BytesWriteErrorValue},
        bytesio::BytesIO,
    },
    byteorder::{ByteOrder, WriteBytesExt},
    bytes::BytesMut,
    rand,
    rand::Rng,
    std::{io::Write, sync::Arc, time::Duration},
    tokio::{sync::Mutex, time::timeout},
};

pub struct BytesWriter {
    pub bytes: Vec<u8>,
}

impl BytesWriter {
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn write_u8(&mut self, byte: u8) -> Result<(), BytesWriteError> {
        self.bytes.write_u8(byte)?;
        Ok(())
    }

    pub fn or_u8_at(&mut self, position: usize, byte: u8) -> Result<(), BytesWriteError> {
        if position > self.bytes.len() {
            return Err(BytesWriteError {
                value: BytesWriteErrorValue::OutofIndex,
            });
        }
        self.bytes[position] |= byte;

        Ok(())
    }

    pub fn add_u8_at(&mut self, position: usize, byte: u8) -> Result<(), BytesWriteError> {
        if position > self.bytes.len() {
            return Err(BytesWriteError {
                value: BytesWriteErrorValue::OutofIndex,
            });
        }
        self.bytes[position] += byte;

        Ok(())
    }

    pub fn write_u8_at(&mut self, position: usize, byte: u8) -> Result<(), BytesWriteError> {
        if position > self.bytes.len() {
            return Err(BytesWriteError {
                value: BytesWriteErrorValue::OutofIndex,
            });
        }
        self.bytes[position] = byte;

        Ok(())
    }

    pub fn get(&mut self, position: usize) -> Option<&u8> {
        return self.bytes.get(position);
    }

    pub fn write_u16<T: ByteOrder>(&mut self, bytes: u16) -> Result<(), BytesWriteError> {
        self.bytes.write_u16::<T>(bytes)?;
        Ok(())
    }

    pub fn write_u24<T: ByteOrder>(&mut self, bytes: u32) -> Result<(), BytesWriteError> {
        self.bytes.write_u24::<T>(bytes)?;

        Ok(())
    }

    pub fn write_u32<T: ByteOrder>(&mut self, bytes: u32) -> Result<(), BytesWriteError> {
        self.bytes.write_u32::<T>(bytes)?;
        Ok(())
    }

    pub fn write_f64<T: ByteOrder>(&mut self, bytes: f64) -> Result<(), BytesWriteError> {
        self.bytes.write_f64::<T>(bytes)?;
        Ok(())
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), BytesWriteError> {
        self.bytes.write(buf)?;
        Ok(())
    }

    pub fn prepend(&mut self, buf: &[u8]) -> Result<(), BytesWriteError> {
        let tmp_bytes = self.bytes.clone();
        self.bytes.clear();
        self.bytes.write(buf)?;
        self.bytes.write(tmp_bytes.as_slice())?;
        Ok(())
    }

    pub fn append(&mut self, writer: &mut BytesWriter) {
        self.bytes.append(&mut writer.bytes);
    }

    pub fn write_random_bytes(&mut self, length: u32) -> Result<(), BytesWriteError> {
        let mut rng = rand::thread_rng();
        for _ in 0..length {
            self.bytes.write_u8(rng.gen())?;
        }
        Ok(())
    }
    pub fn extract_current_bytes(&mut self) -> BytesMut {
        let mut rv_data = BytesMut::new();
        rv_data.extend_from_slice(&self.bytes.clone()[..]);
        self.bytes.clear();

        rv_data
    }

    pub fn clear(&mut self) {
        self.bytes.clear();
    }

    pub fn get_current_bytes(&mut self) -> BytesMut {
        let mut rv_data = BytesMut::new();
        rv_data.extend_from_slice(&self.bytes[..]);
        rv_data
    }

    pub fn pop_bytes(&mut self, size: usize) {
        for _ in 0..size {
            self.bytes.pop();
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }
}

pub struct AsyncBytesWriter {
    pub bytes_writer: BytesWriter,
    pub io: Arc<Mutex<BytesIO>>,
}

impl AsyncBytesWriter {
    pub fn new(io: Arc<Mutex<BytesIO>>) -> Self {
        Self {
            bytes_writer: BytesWriter::new(),
            io: io,
        }
    }

    pub fn write_u8(&mut self, byte: u8) -> Result<(), BytesWriteError> {
        self.bytes_writer.write_u8(byte)
    }

    pub fn write_u16<T: ByteOrder>(&mut self, bytes: u16) -> Result<(), BytesWriteError> {
        self.bytes_writer.write_u16::<T>(bytes)
    }

    pub fn write_u24<T: ByteOrder>(&mut self, bytes: u32) -> Result<(), BytesWriteError> {
        self.bytes_writer.write_u24::<T>(bytes)
    }

    pub fn write_u32<T: ByteOrder>(&mut self, bytes: u32) -> Result<(), BytesWriteError> {
        self.bytes_writer.write_u32::<T>(bytes)
    }

    pub fn write_f64<T: ByteOrder>(&mut self, bytes: f64) -> Result<(), BytesWriteError> {
        self.bytes_writer.write_f64::<T>(bytes)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), BytesWriteError> {
        self.bytes_writer.write(buf)
    }

    pub fn write_random_bytes(&mut self, length: u32) -> Result<(), BytesWriteError> {
        self.bytes_writer.write_random_bytes(length)
    }

    pub fn extract_current_bytes(&mut self) -> BytesMut {
        self.bytes_writer.extract_current_bytes()
    }

    pub async fn flush(&mut self) -> Result<(), BytesWriteError> {
        self.io
            .lock()
            .await
            .write(self.bytes_writer.bytes.clone().into())
            .await?;
        self.bytes_writer.bytes.clear();
        Ok(())
    }

    pub async fn flush_timeout(&mut self, duration: Duration) -> Result<(), BytesWriteError> {
        let message = timeout(
            duration,
            self.io
                .lock()
                .await
                .write(self.bytes_writer.bytes.clone().into()),
        )
        .await;

        match message {
            Ok(_) => {
                self.bytes_writer.bytes.clear();
            }
            Err(_) => {
                return Err(BytesWriteError {
                    value: BytesWriteErrorValue::Timeout,
                })
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    #[test]
    fn test_write_vec() {
        let mut v: Vec<u8> = Vec::new();

        v.push(0x01);
        assert_eq!(1, v.len());
        assert_eq!(0x01, v[0]);

        v[0] = 0x02;
        assert_eq!(0x02, v[0]);

        const FLV_HEADER: [u8; 9] = [
            0x46, // 'F'
            0x4c, //'L'
            0x56, //'V'
            0x01, //version
            0x05, //00000101  audio tag  and video tag
            0x00, 0x00, 0x00, 0x09, //flv header size
        ];

        let rv = v.write(&FLV_HEADER);

        match rv {
            Ok(val) => {
                print!("{} ", val);
            }
            _ => {}
        }

        assert_eq!(10, v.len());
    }

    #[test]
    fn test_bit_opertion() {
        let pts: i64 = 1627702096;

        let val = ((pts << 1) & 0xFE) as u8;

        print!("======={}=======\n", pts << 1);
        print!("======={}=======\n", val);
    }

    #[test]
    fn test_bit_opertion2() {
        let flags = 0xC0;
        let pts: i64 = 1627702096;

        let b9 = ((flags >> 2) & 0x30)/* 0011/0010 */ | (((pts >> 30) & 0x07) << 1) as u8 /* PTS 30-32 */ | 0x01 /* marker_bit */;
        print!("=======b9{}=======\n", b9);

        let b10 = (pts >> 22) as u8 & 0xFF; /* PTS 22-29 */
        print!("=======b10{}=======\n", b10);

        let b11 = ((pts >> 14) & 0xFE) as u8 /* PTS 15-21 */ | 0x01; /* marker_bit */
        print!("=======b11{}=======\n", b11);

        let b12 = (pts >> 7) as u8 & 0xFF; /* PTS 7-14 */
        print!("=======b12{}=======\n", b12);

        let b13 = ((pts << 1) & 0xFE) as u8 /* PTS 0-6 */ | 0x01; /* marker_bit */
        print!("=======b13{}=======\n", b13);
    }

    #[test]
    fn test_bit_opertion3() {
        //let flags = 0xC0;
        let pts: i64 = 1627702096;

        let b12 = ((pts & 0x7fff) << 1) | 1; /* PTS 7-14 */
        print!("=======b12{}=======\n", b12 >> 8 as u8);
        print!("=======b13{}=======\n", b12 as u8);
    }
}
