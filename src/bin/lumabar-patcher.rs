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

const ORIGINAL_SHA256: &str = "341a6d791bd61fc48b02612a8922016ab99cb2038607ba6241cf1c5233e8c7b0";
const PATCHED_SHA256: &str = "f0af6c1d770d45571c6b4f7729251da51f8396a5b83541a9ea1ce303a0c8a4db";
const PATCH_OFFSET: usize = 0xb0ae1d;
const ORIGINAL_PATCH_BYTES: &[u8; 5] = &[0x48, 0xc7, 0x85, 0x00, 0x03];
const PATCH_BYTES: &[u8; 5] = &[0xe9, 0xa0, 0x04, 0x00, 0x00];

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
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| input.with_file_name("lumabar-1.2.19-patched.exe"));
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let mut bytes =
        fs::read(&input).map_err(|error| format!("cannot read {}: {error}", input.display()))?;
    if bytes.len() != 25060864 {
        return Err(format!(
            "unexpected file size {}; expected 25060864",
            bytes.len()
        ));
    }
    let digest = hex_encode(&sha256(&bytes)?);
    if digest == PATCHED_SHA256 {
        return Err("input is already patched".into());
    }
    if digest != ORIGINAL_SHA256 {
        return Err(format!("unsupported lumabar.exe SHA-256: {digest}"));
    }
    let actual = bytes
        .get(PATCH_OFFSET..PATCH_OFFSET + 5)
        .ok_or("patch offset is outside the file")?;
    if actual != ORIGINAL_PATCH_BYTES {
        return Err(format!(
            "unexpected bytes at 0x{PATCH_OFFSET:x}: {}",
            hex_encode(actual)
        ));
    }
    bytes[PATCH_OFFSET..PATCH_OFFSET + 5].copy_from_slice(PATCH_BYTES);
    let patched_digest = hex_encode(&sha256(&bytes)?);
    if patched_digest != PATCHED_SHA256 {
        return Err(format!("post-patch digest mismatch: {patched_digest}"));
    }
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    fs::write(&output, &bytes)
        .map_err(|error| format!("cannot write {}: {error}", output.display()))?;
    println!("patched: {}", output.display());
    println!("offset: 0x{PATCH_OFFSET:x}");
    println!("sha256: {patched_digest}");
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
}
