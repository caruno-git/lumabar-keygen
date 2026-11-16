use std::env;
use std::ffi::c_void;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

#[link(name = "bcrypt")]
unsafe extern "system" {
    fn BCryptOpenAlgorithmProvider(
        algorithm: *mut *mut c_void,
        name: *const u16,
        implementation: *const u16,
        flags: u32,
    ) -> i32;
    fn BCryptCloseAlgorithmProvider(algorithm: *mut c_void, flags: u32) -> i32;
    fn BCryptCreateHash(
        algorithm: *mut c_void,
        hash: *mut *mut c_void,
        object: *mut u8,
        object_length: u32,
        secret: *mut u8,
        secret_length: u32,
        flags: u32,
    ) -> i32;
    fn BCryptDestroyHash(hash: *mut c_void) -> i32;
    fn BCryptHashData(hash: *mut c_void, data: *const u8, length: u32, flags: u32) -> i32;
    fn BCryptFinishHash(hash: *mut c_void, output: *mut u8, length: u32, flags: u32) -> i32;
}

struct BinaryPatch {
    offset: usize,
    original: &'static [u8],
    patched: &'static [u8],
}

struct Version {
    name: &'static str,
    file_size: usize,
    original_sha256: &'static str,
    patched_sha256: &'static str,
    output_name: &'static str,
    standalone: bool,
    patches: &'static [BinaryPatch],
}

const PATCHES_1219: &[BinaryPatch] = &[BinaryPatch {
    offset: 0xb0ae1d,
    original: &[0x48, 0xc7, 0x85, 0x00, 0x03],
    patched: &[0xe9, 0xa0, 0x04, 0x00, 0x00],
}];

const PATCHES_1220: &[BinaryPatch] = &[
    BinaryPatch {
        offset: 0x86942e,
        original: &[0x31, 0xff],
        patched: &[0xff, 0xc7],
    },
    BinaryPatch {
        offset: 0xb3bf40,
        original: &[0x4c, 0x89, 0xe9, 0x48, 0x8b],
        patched: &[0xe9, 0xed, 0x03, 0x00, 0x00],
    },
    BinaryPatch {
        offset: 0xb3c35e,
        original: &[0x00, 0x00, 0x00, 0x00],
        patched: &[0x01, 0x00, 0x00, 0x00],
    },
    BinaryPatch {
        offset: 0xacac72,
        original: &[0x0f, 0xb6, 0x05, 0x3f, 0x1e, 0xc9, 0x00],
        patched: &[0xb0, 0x01, 0x90, 0x90, 0x90, 0x90, 0x90],
    },
    BinaryPatch {
        offset: 0xc45736,
        original: &[0x0f, 0xb6, 0x05, 0x7b, 0x73, 0xb1, 0x00],
        patched: &[0xb0, 0x01, 0x90, 0x90, 0x90, 0x90, 0x90],
    },
];

const VERSIONS: &[Version] = &[
    Version {
        name: "1.2.19",
        file_size: 25060864,
        original_sha256: "341a6d791bd61fc48b02612a8922016ab99cb2038607ba6241cf1c5233e8c7b0",
        patched_sha256: "f0af6c1d770d45571c6b4f7729251da51f8396a5b83541a9ea1ce303a0c8a4db",
        output_name: "lumabar-1.2.19-patched.exe",
        standalone: false,
        patches: PATCHES_1219,
    },
    Version {
        name: "1.2.20",
        file_size: 25396736,
        original_sha256: "ecaadc7c0241244b8b19bdc836a213c2297ee269dab905f9fffda6607d991d9c",
        patched_sha256: "b29d9877079f31cf46b3a527ee8146a606ec5c06774d1e8d77d55934cc0ebd8c",
        output_name: "lumabar-1.2.20-standalone.exe",
        standalone: true,
        patches: PATCHES_1220,
    },
];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let input = args.next().map(PathBuf::from).ok_or_else(|| {
        "usage: lumabar-patcher.exe <original-lumabar.exe> [patched-output.exe]".to_owned()
    })?;
    let requested_output = args.next().map(PathBuf::from);
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let mut bytes =
        fs::read(&input).map_err(|error| format!("cannot read {}: {error}", input.display()))?;
    let digest = hex_encode(&sha256(&bytes)?);
    if let Some(version) = VERSIONS.iter().find(|item| item.patched_sha256 == digest) {
        return Err(format!("input is already patched LumaBar {}", version.name));
    }
    let version = VERSIONS
        .iter()
        .find(|item| item.original_sha256 == digest)
        .ok_or_else(|| format!("unsupported lumabar.exe SHA-256: {digest}"))?;
    if bytes.len() != version.file_size {
        return Err(format!(
            "unexpected file size {}; expected {} for LumaBar {}",
            bytes.len(),
            version.file_size,
            version.name
        ));
    }
    apply_patches(&mut bytes, version)?;
    let patched_digest = hex_encode(&sha256(&bytes)?);
    if patched_digest != version.patched_sha256 {
        return Err(format!("post-patch digest mismatch: {patched_digest}"));
    }
    let output = requested_output.unwrap_or_else(|| input.with_file_name(version.output_name));
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    fs::write(&output, &bytes)
        .map_err(|error| format!("cannot write {}: {error}", output.display()))?;
    println!("version: {}", version.name);
    println!("patched: {}", output.display());
    for patch in version.patches {
        println!(
            "offset: 0x{:x} ({} bytes)",
            patch.offset,
            patch.patched.len()
        );
    }
    println!(
        "mode: {}",
        if version.standalone {
            "standalone, no license.json required"
        } else {
            "offline signed license required"
        }
    );
    println!("sha256: {patched_digest}");
    Ok(())
}

fn apply_patches(bytes: &mut [u8], version: &Version) -> Result<(), String> {
    for patch in version.patches {
        let actual = bytes
            .get(patch.offset..patch.offset + patch.original.len())
            .ok_or_else(|| format!("patch offset 0x{:x} is outside the file", patch.offset))?;
        if actual != patch.original {
            return Err(format!(
                "unexpected bytes at 0x{:x}: {}",
                patch.offset,
                hex_encode(actual)
            ));
        }
    }
    for patch in version.patches {
        bytes[patch.offset..patch.offset + patch.patched.len()].copy_from_slice(patch.patched);
    }
    Ok(())
}

fn sha256(data: &[u8]) -> Result<[u8; 32], String> {
    let name: Vec<u16> = "SHA256".encode_utf16().chain(Some(0)).collect();
    let mut algorithm = std::ptr::null_mut();
    let open_status =
        unsafe { BCryptOpenAlgorithmProvider(&mut algorithm, name.as_ptr(), std::ptr::null(), 0) };
    if open_status < 0 {
        return Err(format!(
            "BCryptOpenAlgorithmProvider failed: 0x{:08x}",
            open_status as u32
        ));
    }
    let mut hash = std::ptr::null_mut();
    let create_status = unsafe {
        BCryptCreateHash(
            algorithm,
            &mut hash,
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            0,
            0,
        )
    };
    if create_status < 0 {
        unsafe { BCryptCloseAlgorithmProvider(algorithm, 0) };
        return Err(format!(
            "BCryptCreateHash failed: 0x{:08x}",
            create_status as u32
        ));
    }
    let data_status = unsafe { BCryptHashData(hash, data.as_ptr(), data.len() as u32, 0) };
    let mut output = [0u8; 32];
    let finish_status = unsafe { BCryptFinishHash(hash, output.as_mut_ptr(), 32, 0) };
    unsafe {
        BCryptDestroyHash(hash);
        BCryptCloseAlgorithmProvider(algorithm, 0);
    }
    if data_status < 0 {
        return Err(format!(
            "BCryptHashData failed: 0x{:08x}",
            data_status as u32
        ));
    }
    if finish_status < 0 {
        return Err(format!(
            "BCryptFinishHash failed: 0x{:08x}",
            finish_status as u32
        ));
    }
    Ok(output)
}

fn hex_encode(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[(byte >> 4) as usize] as char);
        output.push(DIGITS[(byte & 15) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bcrypt_sha256_known_vector() {
        assert_eq!(
            hex_encode(&sha256(b"abc").unwrap()),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn patch_tables_apply_exactly() {
        for version in VERSIONS {
            let size = version
                .patches
                .iter()
                .map(|patch| patch.offset + patch.patched.len())
                .max()
                .unwrap();
            let mut bytes = vec![0u8; size];
            for patch in version.patches {
                assert_eq!(patch.original.len(), patch.patched.len());
                assert_ne!(patch.original, patch.patched);
                bytes[patch.offset..patch.offset + patch.original.len()]
                    .copy_from_slice(patch.original);
            }
            apply_patches(&mut bytes, version).unwrap();
            for patch in version.patches {
                assert_eq!(
                    &bytes[patch.offset..patch.offset + patch.patched.len()],
                    patch.patched
                );
            }
            assert!(apply_patches(&mut bytes, version).is_err());
        }
    }
}
