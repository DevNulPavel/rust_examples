use crate::types::ImageSize;
use byteorder::{LittleEndian, ReadBytesExt};
use eyre::Context;
use flate2::read::GzDecoder;
use std::{
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
    path::Path,
};

enum PVRVersion {
    Legacy,
    New,
}

fn detect_version(header: &[u8; 52]) -> Result<PVRVersion, eyre::Error> {
    // Пробуем сначала прочитать версию
    let new_header_version = (&header[0..4]).read_u32::<LittleEndian>().wrap_err("New version read")?;

    // В новом формате в начале конкретное число, значит новая версия
    if new_header_version == 0x50565203 {
        Ok(PVRVersion::New)
    } else {
        // Если в файлике в нужном месте magic-строка - старый тип
        if (&header[44..48]).eq(b"PVR!") {
            Ok(PVRVersion::Legacy)
        } else {
            Err(eyre::eyre!("Unknown pvr image format"))
        }
    }
}

pub fn pvrgz_image_size(pvrgz_file_path: &Path) -> Result<ImageSize, eyre::Error> {
    // Описание формата
    // https://github.com/powervr-graphics/WebGL_SDK/tree/4.0/Documentation/Specifications
    // NEW: http://cdn.imgtec.com/sdk-documentation/PVR+File+Format.Specification.pdf
    // LEGACY: http://cdn.imgtec.com/sdk-documentation/PVR+File+Format.Specification.Legacy.pdf

    // Заголовок файлика
    let header = {
        // Потоковое чтение из gzip
        let gz_file = File::open(pvrgz_file_path).wrap_err("Pvrgz open")?;
        let mut gz_decoder = GzDecoder::new(gz_file);

        // Читаем заголовок
        let mut header = [0_u8; 52];
        gz_decoder.read_exact(&mut header).wrap_err("Pvr header read failed")?;

        header
    };

    // Определяем версию
    let version = detect_version(&header).wrap_err("Version detect")?;

    // Курсор по заголовку
    let mut cursor = Cursor::new(header);

    // Смещаемся на нужную позицию
    match version {
        PVRVersion::Legacy => {
            cursor.seek(SeekFrom::Start(4)).wrap_err("Pvr header seek")?;
        }
        PVRVersion::New => {
            cursor.seek(SeekFrom::Start(24)).wrap_err("Pvr header seek")?;
        }
    }

    // Читаем
    let height = cursor.read_u32::<LittleEndian>().wrap_err("Pvr height read")?;
    let width = cursor.read_u32::<LittleEndian>().wrap_err("Pvr width read")?;

    // println!("Pvr {:?} size: {}x{}", pvrgz_file_path, width, height);

    Ok(ImageSize { width, height })
}
