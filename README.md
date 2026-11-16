# LumaBar 1.2.19 / 1.2.20 — standalone patcher + offline keygen

**[Русская версия README](README.ru.md)**

Windows x64 tools for two LumaBar versions:

- `lumabar-patcher.exe` identifies an original binary by SHA-256 and creates a patched copy.
- `lumabar-keygen.exe` generates a locally signed `license.json` for LumaBar 1.2.19 or 1.2.20.
- `lumabar-1.2.20-standalone.exe` is the recommended ready-to-run build: **no key and no `license.json` required**.
- `lumabar-1.2.19-patched.exe` is the legacy build and requires the keygen.

Tooling author: **carunodev**.

> Warning: these tools bypass the application's licensing mechanism. This violates the program's terms and may result in license revocation or loss of access. Only the exact binary hashes listed below are supported.

## LumaBar 1.2.20 — recommended standalone mode

Download `lumabar-1.2.20-standalone.exe` from the [latest release](../../releases/latest) and run it. No installer, keygen, license file, or Polar activation is needed.

To patch your own original binary instead:

```powershell
.\lumabar-patcher.exe "C:\path\original-lumabar-1.2.20.exe" ".\lumabar-1.2.20-standalone.exe"
.\lumabar-1.2.20-standalone.exe
```

The patcher verifies the input hash and every original byte before writing. The resulting file must have this SHA-256:

```text
b29d9877079f31cf46b3a527ee8146a606ec5c06774d1e8d77d55934cc0ebd8c
```

Standalone mode was verified with two cold starts without `license.json` or `license.prev.json`. `license_get_status` and `license_revalidate` returned `activated: true`, `stale: false`, and revalidation produced no Polar requests.

## Optional keygen for 1.2.20

The keygen defaults to LumaBar 1.2.20:

```powershell
.\lumabar-keygen.exe --install
```

It creates `%APPDATA%\com.lumabar.dock\license.json` with both signatures:

- `sig`: the original HMAC-SHA256 signature;
- `sig2`: the second HMAC-SHA256 signature introduced in LumaBar 1.2.20.

Both signatures cover the same message:

```text
license_key|instance_id|machine_id|customer_name|customer_email|activated_at_ms|last_validated_at_ms
```

For 1.2.20, `last_validated_at_ms` is set to the current time. The new build rejects the old keygen's far-future timestamp during its local trust check.

**The original, unpatched 1.2.20 binary still contacts Polar and deletes a generated file when the server returns 404.** Use the patched build. Standalone mode does not need this file at all.

## Legacy LumaBar 1.2.19 mode

Patch the original binary:

```powershell
.\lumabar-patcher.exe "C:\path\original-lumabar-1.2.19.exe" ".\lumabar-1.2.19-patched.exe"
```

Generate its license:

```powershell
.\lumabar-keygen.exe --version 1.2.19 --install
```

Start `lumabar-1.2.19-patched.exe`. The legacy keygen keeps the far-future `last_validated_at_ms` behavior required by the old 14-day freshness logic.

## Keygen options

```text
lumabar-keygen.exe [options]

--install             Write license.json into the LumaBar config directory
--output <path>       Write JSON to a selected file
--stdout              Print JSON without writing a file
--name <value>        Customer name shown in the license status
--email <value>       Customer email shown in the license status
--machine-id <value>  MachineGuid of another computer
--version <version>   1.2.19 or 1.2.20 (default: 1.2.20)
```

Defaults:

```text
name:    carunodev
email:   carunodev@localhost
version: 1.2.20
output:  .\license.json
```

Example for another PC:

```powershell
.\lumabar-keygen.exe --machine-id "11111111-2222-3333-4444-555555555555" --name "User" --email "user@example.com" --output ".\license.json"
```

Read `MachineGuid` on the target PC with:

```powershell
reg query "HKLM\SOFTWARE\Microsoft\Cryptography" /v MachineGuid
```

The generated UUID is **not a Polar server license** and cannot be activated on the website. It is only a local value inside the signed JSON.

## Supported binaries

| Version | Original size | Original SHA-256 | Patched SHA-256 |
|---|---:|---|---|
| 1.2.19 | `25,060,864` | `341a6d791bd61fc48b02612a8922016ab99cb2038607ba6241cf1c5233e8c7b0` | `f0af6c1d770d45571c6b4f7729251da51f8396a5b83541a9ea1ce303a0c8a4db` |
| 1.2.20 | `25,396,736` | `ecaadc7c0241244b8b19bdc836a213c2297ee269dab905f9fffda6607d991d9c` | `b29d9877079f31cf46b3a527ee8146a606ec5c06774d1e8d77d55934cc0ebd8c` |

Unknown, partially patched, and already patched inputs are rejected. For 1.2.20, five file offsets are patched: `0x86942E`, `0xB3BF40`, `0xB3C35E`, `0xACAC72`, and `0xC45736`. The legacy 1.2.19 patch changes five bytes at `0xB0AE1D`.

## Build from source

Install stable Rust/Cargo on Windows. No external crates are required.

```powershell
cargo test --all-targets
cargo build --release --all-targets
```

Build outputs are written to `target\release\`.

## Verification performed

- 8 unit tests passed across the keygen and patcher.
- Both patcher outputs matched the Rizin-produced binaries byte for byte.
- Python `hmac` and `hashlib` independently verified `sig` and `sig2`.
- The patched 1.2.20 Tauri/WebView2 runtime accepted the generated nine-field license and returned `activated: true` with the configured customer name and email.
- Standalone 1.2.20 passed cold-start, IPC, paywall, and Polar-request checks without any license file.

## License

The keygen and patcher source code is available under [MIT](LICENSE). The original `lumabar.exe` belongs to its developer; pre-patched binaries are not relicensed under MIT.
