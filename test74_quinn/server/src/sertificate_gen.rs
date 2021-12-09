use eyre::WrapErr;
use rcgen::generate_simple_self_signed;
use rustls::{Certificate, PrivateKey};
use std::{
    fs::{create_dir_all, File},
    io::{self, Read, Write},
    path::Path,
};

pub struct SertificateInfo {
    pub private_key: PrivateKey,
    pub certificate: Certificate,
}

fn create_required_dirs(path: &Path) -> Result<(), io::Error> {
    if let Some(dir) = path.parent() {
        create_dir_all(dir)?;
    }
    Ok(())
}

fn write_data_to_file(path: &Path, data: &[u8]) -> Result<(), eyre::Error> {
    create_required_dirs(path).wrap_err("Dirs create failed")?;
    let mut file = File::create(path).wrap_err("File create failed")?;
    file.write_all(data).wrap_err("File data write failed")?;
    Ok(())
}

fn read_all_file(path: &Path) -> Result<Vec<u8>, eyre::Error> {
    let mut cert_file = File::open(path).wrap_err("Certificate file open")?;
    let mut buf = Vec::new();
    cert_file.read_to_end(&mut buf).wrap_err("File read")?;
    Ok(buf)
}

pub fn read_or_generate_https_sertificate(certificate_path: &Path, key_path: &Path) -> Result<SertificateInfo, eyre::Error> {
    if certificate_path.exists() && key_path.exists() && certificate_path.is_file() && key_path.is_file() {
        let cert_data = read_all_file(certificate_path).wrap_err("Certificate file read")?;
        let key_data = read_all_file(key_path).wrap_err("Certificate file read")?;

        Ok(SertificateInfo {
            private_key: PrivateKey(key_data),
            certificate: Certificate(cert_data),
        })
    } else {
        // Создаем самоподписной сертификат для HTTPS
        let generated_certificate = generate_simple_self_signed(vec!["localhost".to_owned()]).wrap_err("Certificate generation")?;

        let private_key = generated_certificate.serialize_private_key_der();
        let certificate = generated_certificate.serialize_der().wrap_err("Certificate serialize failed")?;

        // Записываем сертификат и ключ в файлки, сертификат нужен будет еще клиенту для проверки
        write_data_to_file(certificate_path, certificate.as_slice()).wrap_err("Certificate")?;
        write_data_to_file(key_path, private_key.as_slice()).wrap_err("Private key")?;

        Ok(SertificateInfo {
            private_key: PrivateKey(private_key),
            certificate: Certificate(certificate),
        })
    }
}
