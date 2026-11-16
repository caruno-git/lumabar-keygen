# LumaBar 1.2.19 — offline keygen + patcher

**[Русская версия README](README.ru.md)**

Windows tools for offline activation of **LumaBar 1.2.19 x64**:

- `lumabar-keygen.exe` — generates a UUID key and a signed `license.json`.
- `lumabar-patcher.exe` — verifies the original `lumabar.exe` by SHA-256 and creates a patched copy.
- `lumabar-1.2.19-patched.exe` — a ready-to-use, runtime-verified binary that can replace the original.

Tooling author: **carunodev**.

> Warning: this bypasses the application's licensing mechanism. It violates the program's terms and may result in license revocation or loss of access. Only the exact LumaBar version listed below is supported.

## Quick start

1. Fully exit LumaBar through its tray icon → **Quit**.
2. Download all three `.exe` files from the [latest release](../../releases/latest), or download `lumabar-offline-bundle.zip`.
3. Back up the original `lumabar.exe`.
4. Replace it with `lumabar-1.2.19-patched.exe`, renaming the patched file back to `lumabar.exe`.
5. Run `lumabar-keygen.exe --install`.
6. Start LumaBar.

Typical **elevated PowerShell** commands, run from the download directory:

```powershell
Copy-Item "$env:ProgramFiles\lumabar\lumabar.exe" "$env:ProgramFiles\lumabar\lumabar.exe.bak"
Copy-Item ".\lumabar-1.2.19-patched.exe" "$env:ProgramFiles\lumabar\lumabar.exe" -Force
.\lumabar-keygen.exe --install
```

Keep the original `WebView2Loader.dll` in the installation directory.

The keygen prints the new `license_key`, `instance_id`, `machine_id`, and output path. By default, the file is written to:

```text
%APPDATA%\com.lumabar.dock\license.json
```

## Patch the original binary yourself

You do not have to download the pre-patched binary. Put `lumabar-patcher.exe` next to the original `lumabar.exe`, then run:

```powershell
.\lumabar-patcher.exe "C:\Program Files\lumabar\lumabar.exe" ".\lumabar-1.2.19-patched.exe"
```

The patcher accepts only the original LumaBar **1.2.19 x64** binary with these properties:

| Check | Value |
|---|---|
| File size | `25,060,864` bytes |
| Original SHA-256 | `341a6d791bd61fc48b02612a8922016ab99cb2038607ba6241cf1c5233e8c7b0` |
| Patch file offset | `0xB0AE1D` |
| Patched SHA-256 | `f0af6c1d770d45571c6b4f7729251da51f8396a5b83541a9ea1ce303a0c8a4db` |

After patching, run:

```powershell
.\lumabar-keygen.exe --install
```

## Keygen options

```text
lumabar-keygen.exe [options]

--install             Write license.json into the LumaBar config directory
--output <path>       Write JSON to a selected file
--stdout              Print JSON without writing a file
--name <value>        Customer name shown in the license status
--email <value>       Customer email shown in the license status
--machine-id <value>  MachineGuid of another computer
```

Defaults:

```text
name:  carunodev
email: carunodev@localhost
output: .\license.json
```

Example for another PC:

```powershell
.\lumabar-keygen.exe --machine-id "11111111-2222-3333-4444-555555555555" --name "User" --email "user@example.com" --output ".\license.json"
```

Read `MachineGuid` on the target PC with:

```powershell
reg query "HKLM\SOFTWARE\Microsoft\Cryptography" /v MachineGuid
```

Then copy the generated `license.json` to `%APPDATA%\com.lumabar.dock\` on that PC.

## Important: what the generated key is

The generated UUID **is not a Polar server license** and cannot be activated on the website. It is used locally inside the signed `license.json`.

Both parts are required:

1. The keygen creates a file with the correct signature.
2. The patched `lumabar.exe` accepts the local status instead of revalidating it through Polar.

The original, unpatched binary will delete this file during server revalidation.

## How it works

License-file signature:

```text
base64(HMAC-SHA256(secret, license_key|instance_id|machine_id|customer_name|customer_email|activated_at_ms|last_validated_at_ms))
```

`last_validated_at_ms` is set in the future, so the 14-day freshness check remains fresh.

The binary patch is located in `license_revalidate`. Immediately after `StoredLicense` is loaded, execution is transferred to the existing `machine_id → LicenseStatus → cleanup` path before an HTTP request is constructed. Only 5 bytes are changed at file offset `0xB0AE1D`.

## Build from source

Install stable Rust/Cargo on Windows. No external crates are required.

```powershell
cargo test --all-targets
cargo build --release --all-targets
```

Build outputs are written to `target\release\`.

## Verification performed

- 6 unit tests covering SHA-256, HMAC-SHA256, Base64, and JSON escaping.
- Independent signature verification with Python `hmac` and `hashlib`.
- The patcher's output matched the Rizin-produced patch byte for byte.
- The original Tauri/WebView2 runtime returned this result for both `license_get_status` and `license_revalidate`:

```json
{
  "activated": true,
  "stale": false,
  "customerName": "carunodev",
  "customerEmail": "carunodev@localhost"
}
```

- Cold restart verified: `license.json` is preserved and the status remains activated.

## License

The keygen and patcher source code is available under [MIT](LICENSE). The original `lumabar.exe` belongs to its developer; the pre-patched binary is not relicensed under MIT.
