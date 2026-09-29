# ESP32-S3-N16R8 CORE board

The specific dev board in hand: ESP32-S3-N16R8 module (16 MB flash / 8 MB PSRAM), CORE-style board, USB Type-C ×2, onboard WS2812 RGB LED, 22-pin headers each side. Pinout and dimensions below are transcribed from the vendor's own datasheet PDF (not a generic ESP32-S3 reference) — see [`pinout.png`](pinout.png) and [`dimensions.png`](dimensions.png).

* Board size: 57 × 28 mm (63.3 mm including the antenna connector overhang)
* Working voltage: 3.3–5 V · 36 IO pins · 2× I2C, 4× SPI

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
| 37 | GPIO37 |
| 36 | GPIO36 |
| 35 | GPIO35 |
| 0 | GPIO0, **BOOT** (strapping pin) |
| 45 | GPIO45 (strapping pin) |
| 48 | GPIO48 — drives the **onboard WS2812 RGB LED** |
| 47 | GPIO47 |
| 21 | GPIO21, USB_D+, RTC |
| 20 | GPIO20, USB_D-, RTC |
| 19 | GPIO19, RTC |
| GND | Ground |
| GND | Ground |

Strapping pins (GPIO0/3/45/46) affect boot mode — avoid using them for general I/O unless you know what you're doing. GPIO48 is already committed to the onboard RGB LED. GPIO19-21 are the USB D+/D- lines.

## Used so far

* **M1** (`experiments/rovi-m1-tb6612/`): GPIO4 (STBY), GPIO5 (AIN1), GPIO6 (AIN2), GPIO7 (PWMA) — none of these conflict with strapping, USB, or the RGB LED pin. See [`WIRING.md`](../../../experiments/rovi-m1-tb6612/WIRING.md).

## Source

Vendor-supplied datasheet PDF (product listing documentation), pages 32 (pinout) and 30 (dimensions). Not committed to this repo (large, vendor-owned); the extracted images here are the durable copy.
