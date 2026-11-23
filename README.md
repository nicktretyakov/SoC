# esp32-ml-activity

Распознавание активности (покой / ходьба / бег) на **ESP32 + MPU6050** на Rust:
данные с акселерометра по I2C → извлечение признаков → классификация
(k-NN или дерево решений из `smartcore`) → отправка JSON по Wi-Fi через HTTP POST.

Проект демонстрирует на одном небольшом встраиваемом примере сразу несколько
техник: ООП через трейты, принципы SOLID, многопоточность (`std::thread::scope`
+ bounded `mpsc`), набор сторонних крейтов, собственные декларативные макросы и
базовое машинное обучение «на устройстве».

## Архитектура

```
┌──────────────────────────────────────────────────────────────────────────┐
│                              ESP32 (SoC)                                 │
│  ┌──────────┐   I2C    ┌─────────────┐                                   │
│  │ MPU6050  │◄─SDA/SCL─►│ sensor thd  │── Vec<AccelSample> ─┐            │
│  │ accel    │  GPIO21/22│ WindowedSam │   mpsc sync_channel │            │
│  └──────────┘           │  pler       │                     ▼            │
│                         └─────────────┘            ┌──────────────┐      │
│                                                    │  ml thread   │      │
│                                                    │  FeatureExtr │      │
│                                                    │  + Classifier│      │
│                                                    └──────┬───────┘      │
│                                         Classification │              │
│                                           mpsc sync_chan│              │
│                                                           ▼             │
│                                                  ┌────────────────┐     │
│                                                  │  net thread    │     │
│                                                  │  JSON → HTTP   │     │
│                                                  │  POST over Wi-Fi      │
│                                                  └────┬───────────┘     │
└───────────────────────────────────────────────────────┼─────────────────┘
                                                        ▼
                                          ┌──────────────────────────┐
                                          │  удалённый сервер (HTTP) │
                                          └──────────────────────────┘
```

## Что где реализовано

| Требование            | Где                                                                 |
|-----------------------|---------------------------------------------------------------------|
| ООП                   | трейты `Sensor`, `SampleSource`, `Classifier`, `DataSink` + структуры-реализации |
| SOLID                 | SRP — по одной ответственности на модуль; OCP — новый сенсор/классификатор/sink добавляется реализацией трейта без правки пайплайна; LSP — любой `Classifier` взаимозаменяем; ISP — маленькие целевые трейты; DIP — пайплайн зависит от трейтов, не от конкретных типов |
| Многопоточность       | `src/pipeline/threads.rs` — три scoped-потока + bounded `mpsc::sync_channel` |
| Крейты                | `esp-idf-hal`, `esp-idf-svc`, `esp-idf-sys`, `smartcore`, `serde`, `serde_json`, `log`, `anyhow`, `num-traits` |
| Макросы               | `src/macros.rs` — `activity_label!`, `features!`, `sample!`, `tag_log!`; плюс `serde_json::json!` |
| ML                    | `src/ml/knn.rs` — собственный k-NN (Евклид + голосование); `src/ml/decision_tree.rs` — `DecisionTreeClassifier` из `smartcore` |
| SoC / I2C             | `src/sensor/mpu6050.rs` — драйвер MPU6050 поверх `I2cDriver` из `esp-idf-hal` |
| Wi-Fi                 | `src/transport/wifi.rs` — `BlockingWifi`/`EspWifi` из `esp-idf-svc` |
| HTTP POST             | `src/transport/http.rs` — `HttpClient` из `esp-idf-svc` |

## Структура проекта

```
esp32-ml-activity/
├── .cargo/config.toml      # таргет xtensa-esp32-espidf, ldproxy, build-std
├── rust-toolchain.toml     # channel = "esp"
├── Cargo.toml              # зависимости (esp-idf 5.1.x aligned)
├── build.rs                # проброс конфигурации ESP-IDF
├── sdkconfig.defaults      # стек/heap/Wi-Fi/LWIP под std + HTTP
├── partitions.csv          # nvs + factory 1 МБ
├── README.md
└── src/
    ├── main.rs             # точки входа: I2C → Wi-Fi → ML → пайплайн
    ├── lib.rs              # корень библиотеки, re-export
    ├── config.rs           # SSID/пароль/URL/пины/гиперпараметры (РЕДАКТИРОВАТЬ ЗДЕСЬ)
    ├── error.rs            # AppError / AppResult
    ├── util.rs             # now_ms()
    ├── macros.rs           # activity_label!, features!, sample!, tag_log!
    ├── sensor/
    │   ├── traits.rs       # Sensor, SampleSource
    │   ├── types.rs        # AccelSample, Activity
    │   ├── mpu6050.rs      # драйвер I2C
    │   └── sampler.rs      # WindowedSampler
    ├── features/mod.rs     # FeatureExtractor (mean/std/max/min/range)
    ├── ml/
    │   ├── traits.rs       # Classifier, Prediction, Classification
    │   ├── knn.rs          # собственный k-NN
    │   ├── decision_tree.rs# smartcore DecisionTreeClassifier
    │   └── dataset.rs      # захардкоженный датасет (6×3 = 18 семплов)
    ├── transport/
    │   ├── traits.rs       # DataSink
    │   ├── wifi.rs         # connect()
    │   └── http.rs         # HttpSink, LoggingSink
    └── pipeline/threads.rs # run() — три потока + каналы
```

## Подключение MPU6050 → ESP32

| MPU6050 | ESP32 pin | Примечание                 |
|---------|-----------|----------------------------|
| VCC     | 3V3       | не 5V — чип 3.3 В          |
| GND     | GND       |                            |
| SDA     | GPIO21    | `config::SDA_PIN`          |
| SCL     | GPIO22    | `config::SCL_PIN`          |
| AD0     | GND       | адрес I2C = 0x68 (`config::I2C_MASTER_ADDR`) |
| INT     | —         | не используется            |

Линии SDA/SCL уже подтянуты на плате ESP32, но внешние подтяжки 4.7 кОм на 3V3
не помешают при длинных проводах.

## Установка инструментов

### Шаг 1 — Xtensa Rust-форк (espup)

```bash
rustup toolchain list | grep esp            # проверить, установлен ли
# если пусто — установить:
cargo install espup
espup install                                # скачивает Xtensa Rust-форк (~600 МБ)
source ~/export-esp.sh                       # активирует окружение в текущей оболочке
```

### Шаг 2 — проверить, что тулчейн виден

```bash
rustup toolchain list
# должна быть строка вида:  esp (override)   (или esp-1.xx.x.x)
```

`rust-toolchain.toml` (`channel = "esp"`) заставляет cargo в этой папке всегда
использовать именно ESP-форк.

### Шаг 3 — инструменты прошивки

```bash
cargo install ldproxy
cargo install espflash cargo-espflash
```

> **Замечание про `build-std`.** Утверждение «build-std не нужен, потому что в
> ESP-форке уже есть core/std/alloc» верно **для `core` и `alloc`** — их
> ESP-форк действительно поставляет прекомпилированными. Но `std` для таргета
> `xtensa-esp32-espidf` **не** прекомпилирован: он собирается из исходников
> поверх конкретной версии ESP-IDF (её тянет `esp-idf-sys` через `embuild`).
> Поэтому `.cargo/config.toml` всё-таки содержит
> `build-std = ["std", "panic_abort"]` — это стандартная рабочая конфигурация
> `esp-idf-template`, без неё бинарник не соберётся. Это не противоречит
> `channel = "esp"`: форк даёт компилятор и `core`/`alloc`, а `build-std`
> достраивает только `std`.

## Конфигурация

Откройте `src/config.rs` и замените три значения:

```rust
pub const WIFI_SSID: &str  = "your_ssid";        // ← имя сети 2.4 ГГц
pub const WIFI_PASS: &str  = "your_password";    // ← пароль
pub const SERVER_URL: &str = "http://your_server_url/data"; // ← endpoint
```

Если `SERVER_URL` всё ещё содержит подстроку `your_server_url`, при старте
вместо HTTP используется `LoggingSink` — результаты просто пишутся в лог, чтобы
можно было проверить ML без сервера.

Прочие настройки (частота дискретизации, размер окна, `k`, параметры дерева)
находятся там же и снабжены комментариями.

## Сборка и прошивка

```bash
source ~/export-esp.sh
cargo build --release
cargo espflash flash --release --monitor /dev/ttyUSB0   # порт подставить свой
```

`--monitor` открывает последовательный лог; `tag_log!`-сообщения отмечены
тегами `sensor` / `ml` / `net` / `wifi`.

## Формат payload

Каждые ~0.5 с (при `SAMPLE_HZ=50`, `WINDOW_SAMPLES=25`) серверу уходит:

```json
{
  "device": "esp32-001",
  "activity": "walk",
  "confidence": 0.6,
  "features": {
    "mean_mag": 1.10,
    "std_mag": 0.30,
    "max_mag": 1.80,
    "min_mag": 0.40,
    "range_mag": 1.40
  },
  "samples": 25,
  "timestamp_ms": 12345
}
```

`confidence` для k-NN — доля `k` соседей, проголосовавших за победителя;
для дерева решений — `1.0` (smartcore 0.3 не открывает per-class вероятности).

## Выбор модели

В `src/config.rs`:

```rust
pub const ML_MODEL: &str = "knn";           // собственный k-NN
// pub const ML_MODEL: &str = "decision_tree"; // smartcore
```

## Тесты (хостовые, без ESP-IDF)

Чисто-логические модули (`features`, `knn`, `dataset`, `macros`, `pipeline`
payload) покрыты `#[cfg(test)]`-тестами. Запуск на хосте требует gating'а
esp-idf-импортов — поэтому тесты запускаются внутри ESP-сборки:

```bash
cargo test --target xtensa-esp32-espidf
```

## Расширение

- **Новый сенсор** — реализуйте `Sensor` и оберните в `WindowedSampler`; пайплайн не изменится.
- **Новая модель ML** — реализуйте `Classifier` (`classify`, `name`) и добавьте ветку в `main.rs`.
- **Новый транспорт** (MQTT, LoRa) — реализуйте `DataSink` и подставьте в `pipeline::run`.

## Замечания о версиях API

Точные сигнатуры `esp-idf-hal` / `esp-idf-svc` / `smartcore` меняются между
минорами. Код написан под зафиксированный в `Cargo.toml` набор
(`esp-idf-sys 0.34` / `esp-idf-hal 0.43` / `esp-idf-svc 0.48` /
`smartcore 0.3.2`). При обновлении проверьте:

- порядок type-параметров `DecisionTreeClassifier<T, Y, X>` в `smartcore`;
- поля `DecisionTreeClassifierParameters`;
- сигнатуры `I2cDriver::new` / `I2cConfig::new` / `BlockingWifi::wrap`;
- тип поля `HttpClient<'static>` в `transport/http.rs`.
