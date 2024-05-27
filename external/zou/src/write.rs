use std::{
    fs::{
        File
    },
    io::{
        Write,
        Seek, 
        SeekFrom
    },
    sync::{
        Arc, 
        Mutex
    }
};

/////////////////////////////////////////////////////////////////////////////////

/// Структура, которая содержит шареный файлик в виде OutputFileWriter + смещение для записи в файл
pub struct OutputChunkWriter {
    output: OutputFileWriter,
    offset: u64,
}

impl OutputChunkWriter {
    pub fn write(&mut self, done_offset: u64, buf: &[u8]) {
        self.output.write(self.offset + done_offset, buf)
    }
}

/////////////////////////////////////////////////////////////////////////////////

/// Структура, которая содержит шареный файлик
pub struct OutputFileWriter {
    file: Arc<Mutex<File>>,
}

impl Clone for OutputFileWriter {
    fn clone(&self) -> OutputFileWriter {
        OutputFileWriter { file: self.file.clone() }
    }
}

impl OutputFileWriter {
    pub fn new(file: File) -> OutputFileWriter {
        OutputFileWriter { file: Arc::new(Mutex::new(file)) }
    }

    /// Пишем в файлик, к внутреннему смещению добавляем еще смещение 
    /// в виде параметра
    pub fn write(&mut self, offset: u64, buf: &[u8]) {
        let mut out_file = self
            .file
            .lock()
            .unwrap();
        out_file
            .seek(SeekFrom::Start(offset))
            .expect("Error while seeking in file.");
        out_file
            .write_all(buf)
            .expect("Error while writing to file.");
    }

    /// Создаем писателя для чанка с указанным смещением
    pub fn create_chunk_writer(&mut self, offset: u64) -> OutputChunkWriter {
        OutputChunkWriter {
            output: self.clone(),
            offset: offset,
        }
    }
}
