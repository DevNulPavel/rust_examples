use eyre::WrapErr;
use rcgen::generate_simple_self_signed;
use rustls::{Certificate, PrivateKey};

pub struct SertificateInfo {
    pub private_key: PrivateKey,
    pub certificate: Certificate,
}

pub fn generate_https_sertificate() -> Result<SertificateInfo, eyre::Error> {
    // Создаем самоподписной сертификат для HTTPS
    let generated_certificate = generate_simple_self_signed(vec!["localhost".to_owned()]).wrap_err("Certificate generation")?;

    let private_key = generated_certificate.serialize_private_key_der();
    let certificate = generated_certificate.serialize_der().wrap_err("Certificate serialize failed")?;

    Ok(SertificateInfo {
        private_key: PrivateKey(private_key),
        certificate: Certificate(certificate),
    })
}
