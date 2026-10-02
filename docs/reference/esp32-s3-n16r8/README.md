# ESP32-S3-N16R8 CORE board

The specific dev board in hand: ESP32-S3-N16R8 module (16 MB flash / 8 MB PSRAM), CORE-style board, USB Type-C ×2, onboard WS2812 RGB LED, 22-pin headers each side. Pinout and dimensions below are transcribed from the vendor's own datasheet PDF (not a generic ESP32-S3 reference) — see [`pinout.png`](pinout.png) and [`dimensions.png`](dimensions.png).

* Board size: 57 × 28 mm (63.3 mm including the antenna connector overhang)
* Working voltage: 3.3–5 V · 36 IO pins · 2× I2C, 4× SPI

## Docs by level

| Level | Files | Notes |
| --- | --- | --- |
| **Board** (this CORE dev board) | [`pinout.png`](pinout.png), [`dimensions.png`](dimensions.png), the pin tables below | The only docs that describe the headers, the two USB-C ports, the RGB LED and the 5Vin/3V3 pins. Header labels are **GPIO numbers**, not package pin numbers. |
| **Module** (ESP32-S3-WROOM-1-N16R8) | [`module-datasheet.md`](module-datasheet.md) | Module pin numbers/names differ from the board header order; shows which GPIOs flash/PSRAM consume. No USB-C ports, LED or board regulator here. |
| **Chip** (ESP32-S3 SoC) | [`soc-datasheet.md`](soc-datasheet.md), [`technical-reference-manual.md`](technical-reference-manual.md), [`errata.md`](errata.md), [`hardware-design-guidelines.md`](hardware-design-guidelines.md) | QFN56 package pin numbers, electrical limits, peripherals/registers. Says nothing about this board's connectors. |

**Ports differ by level.** Chip docs only know the native USB D+/D- on GPIO19/20 and UART0 on GPIO43/44. The board has **two USB Type-C ports** and their routing (which goes to native USB, which to a USB–UART bridge, if any) is board-level and **not documented in any file here** — verify against the physical board or vendor schematic before relying on either port for flashing/serial/power.

## Pin header (left)

| Pin | Function |
| --- | --- |
| 3V3 | 3.3 V out |
| 3V3 | 3.3 V out (second pin, same rail) |
| RST | Reset |
| 4 | RTC GPIO4 — **used by M1: STBY** |
| 5 | RTC GPIO5 — **used by M1: AIN1** |
| 6 | RTC GPIO6 — **used by M1: AIN2** |
| 7 | RTC GPIO7 — **used by M1: PWMA** |
| 15 | RTC GPIO15 |
| 16 | RTC GPIO16 |
| 17 | RTC GPIO17 |
| 18 | RTC GPIO18 |
| 8 | RTC GPIO8 |
| 3 | RTC GPIO3 (strapping pin) |
| 46 | GPIO46 (strapping pin) |
| 9 | RTC GPIO9 |
| 10 | RTC GPIO10 |
| 11 | RTC GPIO11 |
| 12 | RTC GPIO12 |
| 13 | RTC GPIO13 |
| 14 | RTC GPIO14 |
| 5Vin | 5 V in (USB or external) |
| GND | Ground |

## Pin header (right)

| Pin | Function |
| --- | --- |
| GND | Ground |
| 43 | GPIO43, U0TXD |
| 44 | GPIO44, U0RXD |
| 1 | GPIO1, RTC |
| 2 | GPIO2, RTC |
| 42 | GPIO42, MTMS |
| 41 | GPIO41, MTDI |
| 40 | GPIO40, MTDO |
| 39 | GPIO39, MTCK |
| 38 | GPIO38 |
| 37 | GPIO37 (**reserved: octal PSRAM SPIIO7**) |
| 36 | GPIO36 (**reserved: octal PSRAM SPIIO6**) |
| 35 | GPIO35 (**reserved: octal PSRAM SPIIO5**) |
| 0 | GPIO0, **BOOT** (strapping pin) |
| 45 | GPIO45 (strapping pin) |
| 48 | GPIO48 — drives the **onboard WS2812 RGB LED** |
| 47 | GPIO47 |
| 21 | GPIO21, USB_D+, RTC |
| 20 | GPIO20, USB_D-, RTC |
| 19 | GPIO19, RTC |
| GND | Ground |
| GND | Ground |

Strapping pins (GPIO0/3/45/46) affect boot mode — avoid using them for general I/O unless you know what you're doing. GPIO48 is already committed to the onboard RGB LED. GPIO19-21 are the USB D+/D- lines. This module is the **R8 (octal PSRAM)** variant, so GPIO33–37 are wired internally to the PSRAM SPI bus (`SPIIO4`–`SPIIO7`/`SPIDQS`) and must not be reused as general I/O even though GPIO35–37 are broken out on the header — see [Espressif's ESP32-S3 GPIO docs](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/peripherals/gpio.html). GPIO33/34 aren't broken out on this header at all.

## Used so far

* **M1** (`experiments/rovi-m1-tb6612/`): GPIO4 (STBY), GPIO5 (AIN1), GPIO6 (AIN2), GPIO7 (PWMA) — none of these conflict with strapping, USB, or the RGB LED pin. See [`WIRING.md`](../../../experiments/rovi-m1-tb6612/WIRING.md).

## Datasheets (chip and module level — not the board)

Full-text, searchable Markdown transcriptions (machine-converted from Espressif's PDFs; the PDFs are authoritative):

* [`module-datasheet.md`](module-datasheet.md) / [`.pdf`](module-datasheet.pdf) — ESP32-S3-WROOM-1 / -1U v1.8. The board carries the **-N16R8** variant (16 MB quad flash, 8 MB **octal** PSRAM).
* [`soc-datasheet.md`](soc-datasheet.md) / [`.pdf`](soc-datasheet.pdf) — ESP32-S3 series SoC v2.2 (peripherals incl. MCPWM, electrical characteristics, pin/IO MUX tables).
* [`hardware-design-guidelines.md`](hardware-design-guidelines.md) / [`.pdf`](hardware-design-guidelines.pdf) — ESP32-S3 hardware design guidelines (power, strapping, flash/PSRAM pin reservations, layout).
* [`errata.md`](errata.md) / [`.pdf`](errata.pdf) — ESP32-S3 SoC errata.
* [`technical-reference-manual.md`](technical-reference-manual.md) — Technical Reference Manual (1531 pp., register-level MCPWM etc.). Markdown only; the 15 MB PDF is not vendored, fetch it from the URL in the file header.

## Source

Pinout/dimensions images: vendor product-listing datasheet, pages 32 (pinout) and 30 (dimensions). Datasheets above: `espressif.com/sites/default/files/documentation/` (`esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf`, `esp32-s3_datasheet_en.pdf`), fetched 2026-09-30.
