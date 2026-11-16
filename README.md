# LumaBar 1.2.19 — offline keygen + patcher

Инструменты для офлайн-активации **LumaBar 1.2.19 x64** на Windows:

- `lumabar-keygen.exe` — генерирует UUID-ключ и подписанный `license.json`.
- `lumabar-patcher.exe` — проверяет оригинальный `lumabar.exe` по SHA-256 и создаёт патченную копию.
- `lumabar-1.2.19-patched.exe` — готовый проверенный бинарь для замены оригинального.

Автор исходного кода инструментов: **carunodev**.

> Внимание: это обход лицензионного механизма. Использование может привести к отзыву лицензии или блокировке доступа и нарушает условия программы. Поддерживается только указанная версия LumaBar.

## Быстрый старт

1. Полностью закрой LumaBar через иконку в трее → **Quit**.
2. Скачай три `.exe` из [последнего release](../../releases/latest).
3. Сделай резервную копию оригинального `lumabar.exe`.
4. Замени оригинальный файл на `lumabar-1.2.19-patched.exe`, переименовав его обратно в `lumabar.exe`.
5. Запусти `lumabar-keygen.exe --install`.
6. Запусти LumaBar.

Типовые команды для PowerShell **от администратора** (из папки со скачанными файлами):

```powershell
Copy-Item "$env:ProgramFiles\lumabar\lumabar.exe" "$env:ProgramFiles\lumabar\lumabar.exe.bak"
Copy-Item ".\lumabar-1.2.19-patched.exe" "$env:ProgramFiles\lumabar\lumabar.exe" -Force
.\lumabar-keygen.exe --install
```

Не удаляй штатный `WebView2Loader.dll` из папки установки.

Keygen выведет новый `license_key`, `instance_id`, `machine_id` и путь к созданному файлу. По умолчанию файл записывается в:

```text
%APPDATA%\com.lumabar.dock\license.json
```

## Если хочешь пропатчить оригинал сам

Необязательно скачивать готовый патченный бинарь. Положи рядом `lumabar-patcher.exe` и оригинальный `lumabar.exe`, затем:

```powershell
.\lumabar-patcher.exe "C:\Program Files\lumabar\lumabar.exe" ".\lumabar-1.2.19-patched.exe"
```

Patcher принимает только оригинальный файл LumaBar **1.2.19 x64** с этими параметрами:

| Проверка | Значение |
|---|---|
| Размер | `25 060 864` байта |
| SHA-256 оригинала | `341a6d791bd61fc48b02612a8922016ab99cb2038607ba6241cf1c5233e8c7b0` |
| File offset патча | `0xB0AE1D` |
| SHA-256 результата | `f0af6c1d770d45571c6b4f7729251da51f8396a5b83541a9ea1ce303a0c8a4db` |

После успешного патча выполни:

```powershell
.\lumabar-keygen.exe --install
```

## Параметры keygen

```text
lumabar-keygen.exe [options]

--install             Записать license.json в конфиг LumaBar
--output <path>       Записать JSON в выбранный файл
--stdout              Только вывести JSON, ничего не записывать
--name <value>        Имя в статусе лицензии
--email <value>       Email в статусе лицензии
--machine-id <value>  MachineGuid другого компьютера
```

По умолчанию используются:

```text
name:  carunodev
email: carunodev@localhost
output: .\license.json
```

Пример для другого ПК:

```powershell
.\lumabar-keygen.exe --machine-id "11111111-2222-3333-4444-555555555555" --name "User" --email "user@example.com" --output ".\license.json"
```

`MachineGuid` на целевом ПК можно посмотреть так:

```powershell
reg query "HKLM\SOFTWARE\Microsoft\Cryptography" /v MachineGuid
```

Затем скопируй созданный `license.json` в `%APPDATA%\com.lumabar.dock\` на целевом ПК.

## Важно про сгенерированный ключ

Сгенерированный UUID **не является серверной лицензией Polar** и не активируется на сайте. Он используется локально в подписанном `license.json`.

Обязательны обе части:

1. keygen создаёт файл с корректной подписью;
2. патченный `lumabar.exe` принимает локальный статус вместо повторной проверки через Polar.

Оригинальный непропатченный бинарь при повторной серверной проверке удалит такой файл.

## Как это работает

Подпись файла:

```text
base64(HMAC-SHA256(secret, license_key|instance_id|machine_id|customer_name|customer_email|activated_at_ms|last_validated_at_ms))
```

`last_validated_at_ms` выставляется в будущем, поэтому 14-дневная проверка freshness остаётся свежей.

Бинарный патч находится в `license_revalidate`: сразу после успешной загрузки `StoredLicense` управление передаётся существующей ветке `machine_id → LicenseStatus → cleanup`, до создания HTTP-запроса. Изменено 5 байт по file offset `0xB0AE1D`.

## Сборка из исходников

Нужен stable Rust/Cargo на Windows. Внешних crate-зависимостей нет.

```powershell
cargo test --all-targets
cargo build --release --all-targets
```

Результат появится в `target\release\`.

## Проверено

- 6 unit-тестов: SHA-256, HMAC-SHA256, Base64, JSON escaping.
- Независимая сверка подписи через Python `hmac` + `hashlib`.
- Patcher побайтово совпал с патчем, сделанным через Rizin.
- Оригинальный Tauri/WebView2 runtime вернул для `license_get_status` и `license_revalidate`:

```json
{
  "activated": true,
  "stale": false,
  "customerName": "carunodev",
  "customerEmail": "carunodev@localhost"
}
```

- Проверен холодный перезапуск: `license.json` сохраняется, статус остаётся активным.

## Лицензия

Исходный код keygen и patcher распространяется под [MIT](LICENSE). Права на оригинальный `lumabar.exe` принадлежат его разработчику; готовый патченный бинарь не перелицензируется под MIT.
