# LumaBar 1.2.19 / 1.2.20 — standalone-патчер + офлайн-keygen

**[English README](README.md)**

Инструменты для Windows x64 и двух версий LumaBar:

- `lumabar-patcher.exe` определяет оригинальный бинарь по SHA-256 и создаёт патченную копию.
- `lumabar-keygen.exe` генерирует локально подписанный `license.json` для LumaBar 1.2.19 или 1.2.20.
- `lumabar-1.2.20-standalone.exe` — рекомендуемая готовая сборка: **ключ и `license.json` не нужны**.
- `lumabar-1.2.19-patched.exe` — старый вариант, для которого нужен keygen.

Автор инструментов: **carunodev**.

> Внимание: это обход лицензионного механизма. Он нарушает условия программы и может привести к отзыву лицензии или потере доступа. Поддерживаются только точные SHA-256 из таблицы ниже.

## LumaBar 1.2.20 — рекомендуемый standalone-режим

Скачай `lumabar-1.2.20-standalone.exe` из [последнего release](../../releases/latest) и запусти. Установка, keygen, файл лицензии и активация через Polar не нужны.

Чтобы пропатчить собственный оригинальный бинарь:

```powershell
.\lumabar-patcher.exe "C:\path\original-lumabar-1.2.20.exe" ".\lumabar-1.2.20-standalone.exe"
.\lumabar-1.2.20-standalone.exe
```

Патчер проверяет SHA-256 и все исходные байты перед записью. У результата должен быть этот SHA-256:

```text
b29d9877079f31cf46b3a527ee8146a606ec5c06774d1e8d77d55934cc0ebd8c
```

Standalone проверен двумя холодными запусками без `license.json` и `license.prev.json`. `license_get_status` и `license_revalidate` вернули `activated: true`, `stale: false`; при ревалидации запросов к Polar не было.

## Необязательный keygen для 1.2.20

По умолчанию keygen работает в режиме 1.2.20:

```powershell
.\lumabar-keygen.exe --install
```

Он создаёт `%APPDATA%\com.lumabar.dock\license.json` с двумя подписями:

- `sig` — прежняя HMAC-SHA256;
- `sig2` — вторая HMAC-SHA256, добавленная в LumaBar 1.2.20.

Обе подписи считаются от одного сообщения:

```text
license_key|instance_id|machine_id|customer_name|customer_email|activated_at_ms|last_validated_at_ms
```

Для 1.2.20 поле `last_validated_at_ms` содержит текущее время. Новая сборка отклоняет старый трюк с датой на 100 лет вперёд ещё при локальной проверке доверия.

**Оригинальный непропатченный 1.2.20 всё равно обращается к Polar и удаляет сгенерированный файл после HTTP 404.** Поэтому нужен патченный EXE. Standalone-режиму файл вообще не требуется.

## Старый режим LumaBar 1.2.19

Патч оригинала:

```powershell
.\lumabar-patcher.exe "C:\path\original-lumabar-1.2.19.exe" ".\lumabar-1.2.19-patched.exe"
```

Генерация лицензии:

```powershell
.\lumabar-keygen.exe --version 1.2.19 --install
```

Запусти `lumabar-1.2.19-patched.exe`. Для старой версии сохранено поведение с `last_validated_at_ms` далеко в будущем, которое использовалось её 14-дневной freshness-проверкой.

## Параметры keygen

```text
lumabar-keygen.exe [options]

--install             Записать license.json в конфиг LumaBar
--output <path>       Записать JSON в выбранный файл
--stdout              Только вывести JSON, ничего не записывать
--name <value>        Имя в статусе лицензии
--email <value>       Email в статусе лицензии
--machine-id <value>  MachineGuid другого компьютера
--version <version>   1.2.19 или 1.2.20 (по умолчанию: 1.2.20)
```

По умолчанию:

```text
name:    carunodev
email:   carunodev@localhost
version: 1.2.20
output:  .\license.json
```

Пример для другого ПК:

```powershell
.\lumabar-keygen.exe --machine-id "11111111-2222-3333-4444-555555555555" --name "User" --email "user@example.com" --output ".\license.json"
```

Посмотреть `MachineGuid` на целевом ПК:

```powershell
reg query "HKLM\SOFTWARE\Microsoft\Cryptography" /v MachineGuid
```

Сгенерированный UUID **не является серверной лицензией Polar** и не активируется на сайте. Это только локальное значение внутри подписанного JSON.

## Поддерживаемые бинарники

| Версия | Размер оригинала | SHA-256 оригинала | SHA-256 результата |
|---|---:|---|---|
| 1.2.19 | `25 060 864` | `341a6d791bd61fc48b02612a8922016ab99cb2038607ba6241cf1c5233e8c7b0` | `f0af6c1d770d45571c6b4f7729251da51f8396a5b83541a9ea1ce303a0c8a4db` |
| 1.2.20 | `25 396 736` | `ecaadc7c0241244b8b19bdc836a213c2297ee269dab905f9fffda6607d991d9c` | `b29d9877079f31cf46b3a527ee8146a606ec5c06774d1e8d77d55934cc0ebd8c` |

Неизвестные, частично патченные и уже патченные файлы отклоняются. В 1.2.20 изменяются пять offsets: `0x86942E`, `0xB3BF40`, `0xB3C35E`, `0xACAC72`, `0xC45736`. В 1.2.19 меняются пять байт по `0xB0AE1D`.

## Сборка из исходников

Нужен stable Rust/Cargo на Windows. Внешних crate-зависимостей нет.

```powershell
cargo test --all-targets
cargo build --release --all-targets
```

Результат появится в `target\release\`.

## Проверено

- 8 unit-тестов keygen и patcher прошли успешно.
- Оба результата patcher побайтово совпали с бинарями, сделанными через Rizin.
- Python `hmac` и `hashlib` независимо подтвердили `sig` и `sig2`.
- Патченный Tauri/WebView2 runtime 1.2.20 принял сгенерированный JSON с девятью полями и вернул `activated: true` с заданными именем и email.
- Standalone 1.2.20 прошёл холодные запуски, IPC, проверку paywall и отсутствие Polar-запросов вообще без файла лицензии.

## Лицензия

Исходный код keygen и patcher распространяется под [MIT](LICENSE). Права на оригинальный `lumabar.exe` принадлежат его разработчику; готовые патченные бинарники не перелицензируются под MIT.
