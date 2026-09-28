use crate::{filesystem::Directory, OfficialPackageKind, SignatureStatus};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use p384::{
    ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey},
    pkcs8::DecodePublicKey,
};
use sha2::{Digest, Sha256};
use std::{io::Read, path::Path};

// Pinned from XXMI-Launcher commit d56786b8dacb00c35204bff45ff5b8b83bd8962a.
const ZZMI_PUBLIC_KEY_DER_B64: &str = "MHYwEAYHKoZIzj0CAQYFK4EEACIDYgAEb11GjbKQS6SmRe8TcIc5VMu5Ob3moo5v2YeD+s53xEe4bVPGcToUNLu3Jgqo0OwWZ4RsNy1nR0HId6pR09HedyEMifxebsyPT3T5PH82QozEXHQlTDySklWUfGItoOdf";
const LIBRARIES_PUBLIC_KEY_DER_B64: &str = "MHYwEAYHKoZIzj0CAQYFK4EEACIDYgAEYac352uRGKZh6LOwK0fVDW/TpyECEfnRtUp+bP2PJPP63SWOkJ3a/d9pAnPfYezRVJ1hWjZtpRTT8HEAN/b4mWpJvqO43SAEV/1Q6vz9Rk/VvRV3jZ6B/tmqVnIeHKEb";

pub fn verify_release_signature(
    kind: &OfficialPackageKind,
    signature_base64: Option<&str>,
    payload: &[u8],
) -> SignatureStatus {
    let Some(signature) = signature_base64 else {
        return SignatureStatus::Missing;
    };
    let public_key = match kind {
        OfficialPackageKind::Zzmi => ZZMI_PUBLIC_KEY_DER_B64,
        OfficialPackageKind::XxmiLibraries => LIBRARIES_PUBLIC_KEY_DER_B64,
    };
    verify_payload(public_key, signature, payload)
}

pub fn verify_component_signature(path: &Path, signature_base64: &str) -> SignatureStatus {
    verify_file(LIBRARIES_PUBLIC_KEY_DER_B64, signature_base64, path)
}

pub(crate) fn verify_component_in_directory(
    directory: &Directory,
    filename: &str,
    signature_base64: &str,
) -> SignatureStatus {
    verify_directory_file(
        LIBRARIES_PUBLIC_KEY_DER_B64,
        signature_base64,
        directory,
        filename,
    )
}

pub fn verify_release_file(
    kind: &OfficialPackageKind,
    signature_base64: Option<&str>,
    path: &Path,
) -> SignatureStatus {
    let Some(signature) = signature_base64 else {
        return SignatureStatus::Missing;
    };
    let public_key = match kind {
        OfficialPackageKind::Zzmi => ZZMI_PUBLIC_KEY_DER_B64,
        OfficialPackageKind::XxmiLibraries => LIBRARIES_PUBLIC_KEY_DER_B64,
    };
    verify_file(public_key, signature, path)
}

pub fn verify_file(
    public_key_base64: &str,
    signature_base64: &str,
    path: &Path,
) -> SignatureStatus {
    let Some(parent) = path.parent() else {
        return SignatureStatus::Unsupported;
    };
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return SignatureStatus::Unsupported;
    };
    let directory = match Directory::open_absolute(parent) {
        Ok(directory) => directory,
        Err(_) => return SignatureStatus::Unsupported,
    };
    verify_directory_file(public_key_base64, signature_base64, &directory, name)
}

fn verify_directory_file(
    public_key_base64: &str,
    signature_base64: &str,
    directory: &Directory,
    name: &str,
) -> SignatureStatus {
    let key = match decode_key(public_key_base64) {
        Ok(key) => key,
        Err(()) => return SignatureStatus::Unsupported,
    };
    let signature = match decode_signature(signature_base64) {
        Ok(signature) => signature,
        Err(()) => return SignatureStatus::Invalid,
    };
    let mut file = match directory.open_file(name) {
        Ok(file) => file,
        Err(_) => return SignatureStatus::Unsupported,
    };
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = match file.read(&mut buffer) {
            Ok(count) => count,
            Err(_) => return SignatureStatus::Unsupported,
        };
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    verify_digest(&key, &signature, &digest.finalize())
}

fn verify_payload(
    public_key_base64: &str,
    signature_base64: &str,
    payload: &[u8],
) -> SignatureStatus {
    let key = match decode_key(public_key_base64) {
        Ok(key) => key,
        Err(()) => return SignatureStatus::Unsupported,
    };
    let signature = match decode_signature(signature_base64) {
        Ok(signature) => signature,
        Err(()) => return SignatureStatus::Invalid,
    };
    let digest = Sha256::digest(payload);
    verify_digest(&key, &signature, &digest)
}

fn verify_digest(key: &VerifyingKey, signature: &Signature, digest: &[u8]) -> SignatureStatus {
    match key.verify_prehash(digest, signature) {
        Ok(()) => SignatureStatus::Verified,
        Err(_) => SignatureStatus::Invalid,
    }
}

fn decode_key(encoded: &str) -> std::result::Result<VerifyingKey, ()> {
    let der = STANDARD.decode(encoded).map_err(|_| ())?;
    VerifyingKey::from_public_key_der(&der).map_err(|_| ())
}

fn decode_signature(encoded: &str) -> std::result::Result<Signature, ()> {
    let der = STANDARD.decode(encoded).map_err(|_| ())?;
    Signature::from_der(&der).map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::STANDARD;
    use p384::{
        ecdsa::{signature::hazmat::PrehashSigner, SigningKey},
        pkcs8::EncodePublicKey,
    };

    fn vector() -> (String, String, Vec<u8>) {
        // Fixed local-only test vector. The private key never authenticates upstream packages.
        let private = SigningKey::from_bytes((&[7_u8; 48]).into()).expect("fixed scalar is valid");
        let public_der = private.verifying_key().to_public_key_der().unwrap();
        let payload = b"LXMI local signature verification vector".to_vec();
        let digest = Sha256::digest(&payload);
        let signature: Signature = private.sign_prehash(&digest).unwrap();
        (
            STANDARD.encode(public_der.as_bytes()),
            STANDARD.encode(signature.to_der().as_bytes()),
            payload,
        )
    }

    #[test]
    fn verifies_valid_signature_and_rejects_modified_payload_and_wrong_key() {
        let (key, signature, payload) = vector();
        assert_eq!(
            verify_payload(&key, &signature, &payload),
            SignatureStatus::Verified
        );
        let mut changed = payload.clone();
        changed.push(b'!');
        assert_eq!(
            verify_payload(&key, &signature, &changed),
            SignatureStatus::Invalid
        );
        let (other_key, _, _) = vector();
        let wrong_key = STANDARD.encode([0_u8; 8]);
        assert_ne!(other_key, wrong_key);
        assert_eq!(
            verify_payload(&wrong_key, &signature, &payload),
            SignatureStatus::Unsupported
        );
    }

    #[test]
    fn missing_signature_is_never_verified() {
        assert_eq!(
            verify_release_signature(&OfficialPackageKind::Zzmi, None, b"x"),
            SignatureStatus::Missing
        );
    }

    #[test]
    fn verifies_component_from_descriptor_anchored_directory() {
        use std::{
            fs,
            time::{SystemTime, UNIX_EPOCH},
        };

        let (key, signature, payload) = vector();
        let tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("lxmi-signature-dir-{}-{tick}", std::process::id()));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("component.bin"), &payload).unwrap();
        let directory = Directory::open_absolute(&root).unwrap();
        assert_eq!(
            verify_directory_file(&key, &signature, &directory, "component.bin"),
            SignatureStatus::Verified
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn file_verification_rejects_symlinks_and_hardlinks() {
        use std::{
            fs,
            time::{SystemTime, UNIX_EPOCH},
        };

        let (key, signature, payload) = vector();
        let tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("lxmi-signature-path-{}-{tick}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let original = root.join("original.bin");
        fs::write(&original, &payload).unwrap();
        assert_eq!(
            verify_file(&key, &signature, &original),
            SignatureStatus::Verified
        );
        let symlink = root.join("symlink.bin");
        std::os::unix::fs::symlink(&original, &symlink).unwrap();
        let hardlink = root.join("hardlink.bin");
        fs::hard_link(&original, &hardlink).unwrap();

        assert_eq!(
            verify_file(&key, &signature, &symlink),
            SignatureStatus::Unsupported
        );
        assert_eq!(
            verify_file(&key, &signature, &hardlink),
            SignatureStatus::Unsupported
        );
        fs::remove_dir_all(root).unwrap();
    }
}
