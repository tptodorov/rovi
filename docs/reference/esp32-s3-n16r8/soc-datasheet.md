# ESP32-S3 Series SoC datasheet (v2.2)

> Markdown transcription of [`soc-datasheet.pdf`](soc-datasheet.pdf) (source: espressif.com/sites/default/files/documentation/esp32-s3_datasheet_en.pdf). Machine-converted; figures/diagrams dropped and complex tables may be imperfect. **The PDF is authoritative.** **Level: chip — not the dev/breakout board in hand; see this folder's README for board-level docs (pins, ports differ).**

ESP32-S3 Series Datasheet Version 2.2 

Xtensa<sup>®</sup> 32-bit LX7 dual-core microprocessor 2.4 GHz Wi-Fi (IEEE 802.11b/g/n) and Bluetooth<sup>®</sup> 5 (LE) Optional 1.8 V or 3.3 V flash and PSRAM in the chip’s package 45 GPIOs 

QFN56 (7×7 mm) Package 

## Including: 

ESP32-S3 

ESP32-S3FN8 ESP32-S3RH2 ESP32-S3R8 ESP32-S3R16V ESP32-S3FH4R2 ESP32-S3R8V – <u>End of life (EOL)</u> ESP32-S3R2 – <u>End of life (EOL), upgraded to ESP32-S3RH2</u> 

www.espressif.com 

# Product Overview 

ESP32-S3 is a low-power MCU-based system on a chip (SoC) with integrated 2.4 GHz Wi-Fi and Bluetooth<sup>®</sup> Low Energy (Bluetooth LE). It consists of high-performance dual-core microprocessor (Xtensa<sup>®</sup> 32-bit LX7), a ULP coprocessor, a Wi-Fi baseband, a Bluetooth LE baseband, RF module, and numerous peripherals. 

The functional block diagram of the SoC is shown below. 

Power consumption 

Normal 

Low power consumption components capable of working in Deep-sleep mode 

#### ESP32-S3 Functional Block Diagram 

For more information on power consumption, see Section 4.1.3.5 _Power Management Unit (PMU)_ . 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

2 <u>Submit Documentation Feedback</u> 

## Features 

### Wi-Fi 

- Complies with IEEE 802.11b/g/n 

- Supports 20 MHz and 40 MHz bandwidth in 2.4 GHz band 

- 1T1R mode with data rate up to 150 Mbps 

- Wi-Fi Multimedia (WMM) 

- TX/RX A-MPDU, TX/RX A-MSDU 

- Immediate Block ACK 

- Fragmentation and defragmentation 

- Automatic Beacon monitoring (hardware TSF) 

- Four virtual Wi-Fi interfaces 

- Simultaneous support for Infrastructure BSS in Station, SoftAP, or Station + SoftAP modes Note that when ESP32-S3 scans in Station mode, the SoftAP channel will change along with the Station channel 

- Antenna diversity 

- 802.11mc FTM 

### Bluetooth<sup>®</sup> 

- Bluetooth LE: Bluetooth 5, Bluetooth Mesh 

- High-power mode with up to 20 dBm transmission power 

- Speed: 125 Kbps, 500 Kbps, 1 Mbps, 2 Mbps 

- LE Advertising Extensions 

- Multiple Advertising Sets 

- LE Channel Selection Algorithm #2 

- Internal co-existence mechanism between Wi-Fi and Bluetooth to share the same antenna 

### CPU and Memory 

- Xtensa<sup>®</sup> dual-core 32-bit LX7 microprocessor 

- Clock speed: up to 240 MHz 

- CoreMark<sup>®</sup> score: 

   - Two cores at 240 MHz: 1329.92 CoreMark; 5.54 CoreMark/MHz 

- Five-stage pipeline 

- 128-bit data bus and dedicated SIMD instructions 

- Single precision floating point unit (FPU) 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

3 

<u>Submit Documentation Feedback</u> 

- Ultra-Low-Power (ULP) coprocessors: 

   - ULP-RISC-V coprocessor 

   - ULP-FSM coprocessor 

- General DMA controller, with 5 transmit channels and 5 receive channels 

- L1 cache 

- ROM: 384 KB 

- SRAM: 512 KB 

- SRAM in RTC: 16 KB 

- 4096-bit eFuse memory, up to 1792 bits for users 

- Supported SPI protocols: SPI, Dual SPI, Quad SPI, Octal SPI, QPI and OPI interfaces that allow connection to flash, external RAM, and other SPI devices 

- Flash controller with cache is supported 

- Flash in-Circuit Programming (ICP) is supported 

### Peripherals 

- 45 programmable GPIOs 

   - 4 strapping GPIOs 

   - GPIOs allocated for in-package memory: 

      - 6 GPIOs for either in-package flash or PSRAM 

      - 7 GPIOs when both in-package flash and PSRAM are integrated 

- Connectivity interfaces: 

   - Three UART interfaces 

   - Two I2C interfaces 

   - Two I2S interfaces 

   - LCD interface 

   - 8-bit ~ 16-bit DVP camera interface 

   - Two SPI ports for communication with flash and RAM 

   - Two general-purpose SPI ports 

   - TWAI<sup>®</sup> controller, compatible with ISO 11898-1 (CAN Specification 2.0) 

   - Full-speed USB OTG 

   - USB Serial/JTAG controller 

   - SD/MMC host controller with 2 slots 

   - LED PWM controller, up to 8 channels 

   - Two Motor Control PWM (MCPWM) 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

4 <u>Submit Documentation Feedback</u> 

   - RMT (TX/RX) 

   - Pulse count controller 

- Analog signal processing: 

   - Two 12-bit SAR ADCs, up to 20 channels 

   - Temperature sensor 

   - 14 capacitive touch sensing IOs 

- Timers: 

   - Four 54-bit general-purpose timers 

   - 52-bit system timer 

   - Three watchdog timers 

### Power Management 

- Fine-resolution power control, including clock frequency, duty cycle, Wi-Fi operating modes, and individual internal component control 

- Four power modes designed for typical scenarios: Active, Modem-sleep, Light-sleep, Deep-sleep 

- Power consumption in Deep-sleep mode is 7 _µ_ A 

- RTC memory remains powered on in Deep-sleep mode 

### Security 

- Secure boot - permission control on accessing internal and external memory 

- Flash encryption - memory encryption and decryption 

- Cryptographic hardware acceleration: 

   - SHA Accelerator (FIPS PUB 180-4) 

   - AES Accelerator (FIPS PUB 197) 

   - RSA Accelerator 

   - HMAC Accelerator 

   - RSA Digital Signature Peripheral (RSA_DS) 

   - Random Number Generator (RNG) 

### RF Module 

- Antenna switches, RF balun, power amplifier, low-noise receive amplifier 

- Up to +21 dBm of power for an 802.11b transmission 

- Up to +19.5 dBm of power for an 802.11n transmission 

- Up to -104.5 dBm of sensitivity for Bluetooth LE receiver (125 Kbps) 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

5 <u>Submit Documentation Feedback</u> 

## Applications 

With low power consumption, ESP32-S3 is an ideal choice for IoT devices in the following areas: 

- Smart Home 

- Industrial Automation 

- Health Care 

- Consumer Electronics 

   - Generic Low-power IoT Sensor Hubs 

   - Generic Low-power IoT Data Loggers 

   - Cameras for Video Streaming 

   - USB Devices 

- Smart Agriculture 

- POS Machines 

- Service Robot 

- Audio Devices 

- Speech Recognition 

- Image Recognition 

- Wi-Fi + Bluetooth Networking Card 

- Touch and Proximity Sensing 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

6 

<u>Submit Documentation Feedback</u> 

Contents 

Note: Check the link or the QR code to make sure that you use the latest version of this document: https://www.espressif.com/documentation/esp32-s3_datasheet_en.pdf 

|Co|ntents||
|---|---|---|
|Pro|duct Overview|2|
|Featu|res|3|
|Appli|cations|6|
|1|ESP32-S3 Series Comparison|13|
|1.1|Nomenclature|13|
|1.2|Comparison|13|
|1.3|Chip Revision|14|
|2|Pins|15|
|2.1|Pin Layout|15|
|2.2|Pin Overview|16|
|2.3|IO Pins|20|
||2.3.1<br>IO MUX Functions|20|
||2.3.2<br>RTC Functions|23|
||2.3.3<br>Analog Functions|24|
||2.3.4<br>Restrictions for GPIOs and RTC_GPIOs|25|
||2.3.5<br>Peripheral Pin Assignment|26|
|2.4|Analog Pins|28|
|2.5|Power Supply|29|
||2.5.1<br>Power Pins|29|
||2.5.2<br>Power Scheme|29|
||2.5.3<br>Chip Power-up and Reset|30|
|2.6|Pin Mapping Between Chip and Flash/PSRAM|31|
|3|Boot Configurations|32|
|3.1|Chip Boot Mode Control|33|
|3.2|VDD_SPI Voltage Control|34|
|3.3|ROM Messages Printing Control|34|
|3.4|JTAG Signal Source Control|34|
|4|Functional Description|36|
|4.1|<br>System|36|
||4.1.1<br>Microprocessor and Master|36|
||4.1.1.1<br>CPU|36|
||4.1.1.2<br>Processor Instruction Extensions (PIE)|36|

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

7 <u>Submit Documentation Feedback</u> 

Contents 

||4.1.1.3|Ultra-Low-Power Coprocessor (ULP)|37|
|---|---|---|---|
||4.1.1.4|GDMA Controller (GDMA)|37|
|4.1.2|Memory|Organization|38|
||4.1.2.1|Internal Memory|38|
||4.1.2.2|External Flash and RAM|39|
||4.1.2.3|Cache|39|
||4.1.2.4|eFuse Controller|40|
|4.1.3|System|Components|40|
||4.1.3.1|IO MUX and GPIO Matrix|40|
||4.1.3.2|Reset|41|
||4.1.3.3|Clock|41|
||4.1.3.4|Interrupt Matrix|42|
||4.1.3.5|Power Management Unit (PMU)|42|
||4.1.3.6|System Timer|44|
||4.1.3.7|General Purpose Timers|44|
||4.1.3.8|Watchdog Timers|45|
||4.1.3.9|XTAL32K Watchdog Timers|45|
||4.1.3.10|Permission Control|45|
||4.1.3.11|World Controller|46|
||4.1.3.12|System Registers|47|
|4.1.4|Cryptogr|aphy and Security Component|47|
||4.1.4.1|SHA Accelerator|47|
||4.1.4.2|AES Accelerator|48|
||4.1.4.3|RSA Accelerator|48|
||4.1.4.4|Secure Boot|48|
||4.1.4.5|HMAC Accelerator|49|
||4.1.4.6|RSA Digital Signature Peripheral (RSA_DS)|49|
||4.1.4.7|External Memory Encryption and Decryption|49|
||4.1.4.8|Clock Glitch Detection|50|
||4.1.4.9|Random Number Generator|50|
|4.2<br>Periph|erals||51|
|4.2.1|Connect|ivity Interface|51|
||4.2.1.1|UART Controller|51|
||4.2.1.2|I2C Interface|51|
||4.2.1.3|I2S Interface|52|
||4.2.1.4|LCD and Camera Controller|52|
||4.2.1.5|Serial Peripheral Interface (SPI)|53|
||4.2.1.6|Two-Wire Automotive Interface (TWAI<sup>®</sup>)|54|
||4.2.1.7|USB 2.0 OTG Full-Speed Interface|55|
||4.2.1.8|USB Serial/JTAG Controller|56|
||4.2.1.9|SD/MMC Host Controller|56|
||4.2.1.10|Motor Control PWM (MCPWM)|57|
||4.2.1.11|Remote Control Peripheral (RMT)|58|
||4.2.1.12|Pulse Count Controller (PCNT)|58|
|4.2.2|Analog S|ignal Processing|59|
||4.2.2.1|SAR ADC|59|

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

8 <u>Submit Documentation Feedback</u> 

Contents 

||4.2.2.2<br>Temperature Sensor|59|
|---|---|---|
||4.2.2.3<br>Touch Sensor|59|
|4.3|Wireless Communication|61|
||4.3.1<br>Radio|61|
||4.3.1.1<br>2.4 GHz Receiver|61|
||4.3.1.2<br>2.4 GHz Transmitter|61|
||4.3.1.3<br>Clock Generator|61|
||4.3.2<br>Wi-Fi|61|
||4.3.2.1<br>Wi-Fi Radio and Baseband|62|
||4.3.2.2<br>Wi-Fi MAC|62|
||4.3.2.3<br>Networking Features|62|
||4.3.3<br>Bluetooth LE|62|
||4.3.3.1<br>Bluetooth LE PHY|63|
||4.3.3.2<br>Bluetooth LE Link Controller|63|
|5|Electrical Characteristics|64|
|5.1|Absolute Maximum Ratings|64|
|5.2|Recommended Operating Conditions|64|
|5.3|VDD_SPI Output Characteristics|65|
|5.4|DC Characteristics (3.3 V, 25 °C)|65|
|5.5|ADC Characteristics|66|
|5.6|Current Consumption|66|
||5.6.1<br>Current Consumption in Active Mode|66|
||5.6.2<br>Current Consumption in Other Modes|67|
|5.7|Memory Specifications|68|
|5.8|Reliability|69|
|6|RF Characteristics|70|
|6.1|Wi-Fi Radio|70|
||6.1.1<br>Wi-Fi RF Transmitter (TX) Characteristics|70|
||6.1.2<br>Wi-Fi RF Receiver (RX) Characteristics|71|
|6.2|Bluetooth LE Radio|72|
||6.2.1<br>Bluetooth LE RF Transmitter (TX) Characteristics|73|
||6.2.2<br>Bluetooth LE RF Receiver (RX) Characteristics|74|
|7|Packaging|77|
|ES|P32-S3 Consolidated Pin Overview|79|
|Dat|asheet Versioning|80|
|Glo|ssary|81|
|Rel|ated Documentation and Resources|82|
|Re|vision History|83|

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

9 <u>Submit Documentation Feedback</u> 

List of Tables 

# List of Tables 

|1-1|ESP32-S3 Series Comparison|13|
|---|---|---|
|2-1|Pin Overview|16|
|2-2|Power-Up Glitches on Pins|18|
|2-3|Peripheral Signals Routed via IO MUX|20|
|2-4|IO MUX Functions|21|
|2-5|RTC Peripheral Signals Routed via RTC IO MUX|23|
|2-6|RTC Functions|23|
|2-7|Analog Signals Routed to Analog Functions|24|
|2-8|Analog Functions|24|
|2-9|Peripheral Pin Assignment|27|
|2-10|Analog Pins|28|
|2-11|Power Pins|29|
|2-12|Voltage Regulators|29|
|2-13|Description of Timing Parameters for Power-up and Reset|30|
|2-14|Pin Mapping Between Chip and Flash or PSRAM|31|
|3-1|Default Configuration of Strapping Pins|32|
|3-2|Description of Timing Parameters for the Strapping Pins|33|
|3-3|Chip Boot Mode Control|33|
|3-4|VDD_SPI Voltage Control|34|
|3-5|JTAG Signal Source Control|35|
|4-1|Components and Power Domains|44|
|5-1|Absolute Maximum Ratings|64|
|5-2|Recommended Operating Conditions|64|
|5-3|VDD_SPI Internal and Output Characteristics|65|
|5-4|DC Characteristics (3.3 V, 25 °C)|65|
|5-5|ADC Characteristics|66|
|5-6|ADC Calibration Results|66|
|5-7|Current Consumption for Wi-Fi (2.4 GHz) in Active Mode|66|
|5-8|Current Consumption for Bluetooth LE in Active Mode|67|
|5-9|Current Consumption in Modem-sleep Mode|67|
|5-10|Current Consumption in Low-Power Modes|68|
|5-11|Flash Specifications|68|
|5-12|PSRAM Specifications|69|
|5-13|Reliability Qualifications|69|
|6-1|Wi-Fi RF Characteristics|70|
|6-2|TX Power with Spectral Mask and EVM Meeting 802.11 Standards|70|
|6-3|TX EVM Test<sup>1</sup>|70|
|6-4|RX Sensitivity|71|
|6-5|Maximum RX Level|72|
|6-6|RX Adjacent Channel Rejection|72|
|6-7|Bluetooth LE Frequency|72|
|6-8|Transmitter Characteristics - Bluetooth LE 1 Mbps|73|
|6-9|Transmitter Characteristics - Bluetooth LE 2 Mbps|73|

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

10 <u>Submit Documentation Feedback</u> 

List of Tables 

|6-10|Transmitter Characteristics - Bluetooth LE 125 Kbps|73|
|---|---|---|
|6-11|Transmitter Characteristics - Bluetooth LE 500 Kbps|74|
|6-12|Receiver Characteristics - Bluetooth LE 1 Mbps|74|
|6-13|Receiver Characteristics - Bluetooth LE 2 Mbps|75|
|6-14|Receiver Characteristics - Bluetooth LE 125 Kbps|75|
|6-15|Receiver Characteristics - Bluetooth LE 500 Kbps|76|
|7-1|Consolidated Pin Overview|79|

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

11 

<u>Submit Documentation Feedback</u> 

List of Figures 

# List of Figures 

|1-1|ESP32-S3 Series Nomenclature|13|
|---|---|---|
|2-1|ESP32-S3 Pin Layout (Top View)|15|
|2-2|ESP32-S3 Power Scheme|30|
|2-3|Visualization of Timing Parameters for Power-up and Reset|30|
|3-1|Visualization of Timing Parameters for the Strapping Pins|33|
|4-1|Address Mapping Structure|38|
|4-2|Components and Power Domains|43|
|7-1|QFN56 (7×7 mm) Package|77|
|7-2|QFN56 (7×7 mm) Package (Only for ESP32-S3FH4R2)|78|

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

12 

<u>Submit Documentation Feedback</u> 

1 ESP32-S3 Series Comparison 

# 1 ESP32-S3 Series Comparison 

## 1.1 Nomenclature 

Figure 1-1. ESP32-S3 Series Nomenclature 

## 1.2 Comparison 

Table 1-1. ESP32-S3 Series Comparison 

|Part Number<sup>1</sup>|In-Package Flash <sup>2</sup>|In-Package PSRAM|Ambient Temp. <sup>3</sup>|VDD_SPI Voltage <sup>4</sup>|Chip Revision|
|---|---|---|---|---|---|
|ESP32-S3|—|—|–40_∼_105 °C|3.3 V/1.8 V|v0.1/v0.2|
|ESP32-S3FN8|8 MB (Quad SPI)<sup>5</sup>|—|–40_∼_85 °C|3.3 V|v0.1/v0.2|
|ESP32-S3RH2|—|2 MB (Quad SPI)|–40_∼_105 °C|3.3 V|v0.2|
|ESP32-S3R8|—|8 MB (Octal SPI)|–40_∼_65 °C|3.3 V|v0.1/v0.2|
|ESP32-S3R16V|—|16 MB (Octal SPI)|–40_∼_65 °C|1.8 V|v0.2|
|ESP32-S3FH4R2|4 MB (Quad SPI)|2 MB (Quad SPI)|–40_∼_85 °C|3.3 V|v0.1/v0.2|
|ESP32-S3R8V (EOL)|—|8 MB (Octal SPI)|–40_∼_65 °C|1.8 V|v0.1/v0.2|
|ESP32-S3R2 (EOL)<sup>6</sup>|—|2 MB (Quad SPI)|–40_∼_85 °C|3.3 V|v0.1/v0.2|

- 1 For details on chip marking and packing, see Section 7 _Packaging_ . 

- 2 For information about in-package flash, see also Section 4.1.2.1 _Internal Memory_ . By default, the SPI flash on the chip operates at a maximum clock frequency of 80 MHz and does not support the auto suspend feature. If you have a requirement for a higher flash clock frequency of 120 MHz or if you need the flash auto suspend feature, please <u>contact us.</u> 

- 3 Ambient temperature specifies the recommended temperature range of the environment immediately outside an Espressif chip. For chips with Octal SPI PSRAM (ESP32-S3R8, ESP32-S3R8V, and ESP32-S3R16V), if the PSRAM ECC function is enabled, the maximum ambient temperature can be improved to 85 °C, while the usable size of PSRAM will be reduced by 1/16. 

- 4 For more information on VDD_SPI, see Section 2.5 _Power Supply_ . 

- 5 For details about SPI modes, see Section 2.6 _Pin Mapping Between Chip and Flash/PSRAM_ . 

- 6 ESP32-S3R2 has been upgraded to ESP32-S3RH2. For more information, see <u>PCN.</u> 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

13 <u>Submit Documentation Feedback</u> 

1 ESP32-S3 Series Comparison 

## 1.3 Chip Revision 

As shown in Table 1-1 _ESP32-S3 Series Comparison_ , ESP32-S3 now has multiple chip revisions available on the market using the same part number. 

For chip revision identification, ESP-IDF release that supports a specific chip revision, and errors fixed in each chip revision, please refer to _<u>ESP32-S3 Series SoC Errata</u>_ <u>.</u> 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

14 

<u>Submit Documentation Feedback</u> 

2 Pins 

# 2 Pins 

## 2.1 Pin Layout 

Figure 2-1. ESP32-S3 Pin Layout (Top View) 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

15 <u>Submit Documentation Feedback</u> 

2 Pins 

## 2.2 Pin Overview 

The ESP32-S3 chip integrates multiple peripherals that require communication with the outside world. To keep the chip package size reasonably small, the number of available pins has to be limited. So the only way to route all the incoming and outgoing signals is through pin multiplexing. Pin muxing is controlled via software programmable registers (see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _IO MUX and GPIO Matrix_ ). 

All in all, the ESP32-S3 chip has the following types of pins: 

- IO pins with the following predefined sets of functions to choose from: 

   - Each IO pin has predefined IO MUX functions – see Table 2-4 _IO MUX Functions_ 

   - Some IO pins have predefined RTC functions – see Table 2-6 _RTC Functions_ 

   - Some IO pins have predefined analog functions – see Table 2-8 _Analog Functions_ 

_Predefined functions_ means that each IO pin has a set of direct connections to certain on-chip peripherals. During run-time, the user can configure which peripheral from a predefined set to connect to a certain pin at a certain time via memory mapped registers (see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _IO MUX and GPIO pins_ ). 

- Analog pins that have exclusively-dedicated analog functions – see Table 2-10 _Analog Pins_ 

- Power pins that supply power to the chip components and non-power pins – see Table 2-11 _Power Pins_ 

Table 2-1 _Pin Overview_ gives an overview of all the pins. For more information, see the respective sections for each pin type below, or _ESP32-S3 Consolidated Pin Overview_ . 

Table 2-1. Pin Overview 

|||||Pin S|ettings <sup>6</sup>|Pi|n Function Sets|<sup>1</sup>|
|---|---|---|---|---|---|---|---|---|
|Pin No.|Pin Name|Pin Type|Pin Providing Power <sup>2-5</sup>|At Reset|After Reset|IO MUX|RTC IO MUX|Analog|
|1|LNA_IN|Analog|||||||
|2|VDD3P3|Power|||||||
|3|VDD3P3|Power|||||||
|4|CHIP_PU|Analog|VDD3P3_RTC||||||
|5|GPIO0|IO|VDD3P3_RTC|WPU, IE|WPU, IE|IO MUX|RTC IO MUX||
|6|GPIO1|IO|VDD3P3_RTC|IE|IE|IO MUX|RTC IO MUX|Analog|
|7|GPIO2|IO|VDD3P3_RTC|IE|IE|IO MUX|RTC IO MUX|Analog|
|8|GPIO3|IO|VDD3P3_RTC|IE|IE|IO MUX|RTC IO MUX|Analog|
|9|GPIO4|IO|VDD3P3_RTC|||IO MUX|RTC IO MUX|Analog|
|10|GPIO5|IO|VDD3P3_RTC|||IO MUX|RTC IO MUX|Analog|
|11|GPIO6|IO|VDD3P3_RTC|||IO MUX|RTC IO MUX|Analog|
|12|GPIO7|IO|VDD3P3_RTC|||IO MUX|RTC IO MUX|Analog|
|13|GPIO8|IO|VDD3P3_RTC|||IO MUX|RTC IO MUX|Analog|
|14|GPIO9|IO|VDD3P3_RTC||IE|IO MUX|RTC IO MUX|Analog|
|15|GPIO10|IO|VDD3P3_RTC||IE|IO MUX|RTC IO MUX|Analog|
|16|GPIO11|IO|VDD3P3_RTC||IE|IO MUX|RTC IO MUX|Analog|
|17|GPIO12|IO|VDD3P3_RTC||IE|IO MUX|RTC IO MUX|Analog|
|18|GPIO13|IO|VDD3P3_RTC||IE|IO MUX|RTC IO MUX|Analog|

Cont’d on next page 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

16 <u>Submit Documentation Feedback</u> 

2 Pins 

Cont’d from previous page 

|||||Pin S|ettings <sup>6</sup>|Pi|n Function Sets|<sup>1</sup>|
|---|---|---|---|---|---|---|---|---|
|Pin No.|Pin Name|Pin Type|Pin Providing Power <sup>2-5</sup>|At Reset|After Reset|IO MUX|RTC IO MUX|Analog|
|19|GPIO14|IO|VDD3P3_RTC||IE|IO MUX|RTC IO MUX|Analog|
|20|VDD3P3_RTC|Power|||||||
|21|XTAL_32K_P|IO|VDD3P3_RTC|||IO MUX|RTC IO MUX|Analog|
|22|XTAL_32K_N|IO|VDD3P3_RTC|||IO MUX|RTC IO MUX|Analog|
|23|GPIO17|IO|VDD3P3_RTC||IE|IO MUX|RTC IO MUX|Analog|
|24|GPIO18|IO|VDD3P3_RTC||IE|IO MUX|RTC IO MUX|Analog|
|25|GPIO19|IO|VDD3P3_RTC|||IO MUX|RTC IO MUX|Analog|
|26|GPIO20|IO|VDD3P3_RTC|USB_PU|USB_PU|IO MUX|RTC IO MUX|Analog|
|27|GPIO21|IO|VDD3P3_RTC|||IO MUX|RTC IO MUX||
|28|SPICS1|IO|VDD_SPI|WPU, IE|WPU, IE|IO MUX|||
|29|VDD_SPI|Power|||||||
|30|SPIHD|IO|VDD_SPI|WPU, IE|WPU, IE|IO MUX|||
|31|SPIWP|IO|VDD_SPI|WPU, IE|WPU, IE|IO MUX|||
|32|SPICS0|IO|VDD_SPI|WPU, IE|WPU, IE|IO MUX|||
|33|SPICLK|IO|VDD_SPI|WPU, IE|WPU, IE|IO MUX|||
|34|SPIQ|IO|VDD_SPI|WPU, IE|WPU, IE|IO MUX|||
|35|SPID|IO|VDD_SPI|WPU, IE|WPU, IE|IO MUX|||
|36|SPICLK_N|IO|VDD_SPI/VDD3P3_CPU|IE|IE|IO MUX|||
|37|SPICLK_P|IO|VDD_SPI/VDD3P3_CPU|IE|IE|IO MUX|||
|38|GPIO33|IO|VDD_SPI/VDD3P3_CPU||IE|IO MUX|||
|39|GPIO34|IO|VDD_SPI/VDD3P3_CPU||IE|IO MUX|||
|40|GPIO35|IO|VDD_SPI/VDD3P3_CPU||IE|IO MUX|||
|41|GPIO36|IO|VDD_SPI/VDD3P3_CPU||IE|IO MUX|||
|42|GPIO37|IO|VDD_SPI/VDD3P3_CPU||IE|IO MUX|||
|43|GPIO38|IO|VDD3P3_CPU||IE|IO MUX|||
|44|MTCK|IO|VDD3P3_CPU||IE<sup>7</sup>|IO MUX|||
|45|MTDO|IO|VDD3P3_CPU||IE|IO MUX|||
|46|VDD3P3_CPU|Power|||||||
|47|MTDI|IO|VDD3P3_CPU||IE|IO MUX|||
|48|MTMS|IO|VDD3P3_CPU||IE|IO MUX|||
|49|U0TXD|IO|VDD3P3_CPU|WPU, IE|WPU, IE|IO MUX|||
|50|U0RXD|IO|VDD3P3_CPU|WPU, IE|WPU, IE|IO MUX|||
|51|GPIO45|IO|VDD3P3_CPU|WPD, IE|WPD, IE|IO MUX|||
|52|GPIO46|IO|VDD3P3_CPU|WPD, IE|WPD, IE|IO MUX|||
|53|XTAL_N|Analog|||||||
|54|XTAL_P|Analog|||||||
|55|VDDA|Power|||||||
|56|VDDA|Power|||||||
|57|GND|Power|||||||

1. Bold marks the pin function set in which a pin has its default function in the default boot mode. For more information about the boot mode，see Section 3.1 _Chip Boot Mode Control_ . 

2. In column Pin Providing Power, regarding pins powered by VDD_SPI: 

   - Power actually comes from the internal power rail supplying power to VDD_SPI. For details, see Section 2.5.2 _Power Scheme_ . 

> 3. In column Pin Providing Power, regarding pins powered by VDD3P3_CPU / VDD_SPI: 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

17 <u>Submit Documentation Feedback</u> 

2 Pins 

   - Pin Providing Power (either VDD3P3_CPU or VDD_SPI) is decided by eFuse bit EFUSE_PIN_POWER_SELECTION (see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _eFuse Controller_ ) and can be configured via the IO_MUX_PAD_POWER_CTRL bit (see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _IO MUX and GPIO pins_ ). 

4. For ESP32-S3R8V and ESP32-S3R16V chip, as the VDD_SPI voltage has been set to 1.8 V, the working voltage for pins SPICLK_N and SPICLK_P (GPIO47 and GPIO48) would also be 1.8 V, which is different from other GPIOs. 

5. The default drive strengths for each pin are as follows: 

   - GPIO17 and GPIO18: 10 mA 

   - GPIO19 and GPIO20: 40 mA 

   - All other pins: 20 mA 

6. Column Pin Settings shows predefined settings at reset and after reset with the following abbreviations: 

   - IE – input enabled 

   - WPU – internal weak pull-up resistor enabled 

   - WPD – internal weak pull-down resistor enabled 

   - USB_PU – USB pull-up resistor enabled 

      - By default, the USB function is enabled for USB pins (i.e., GPIO19 and GPIO20), and the pin pull-up is decided by the USB pull-up. The USB pull-up is controlled by USB_SERIAL_JTAG_DP/DM_PULLUP and the pull-up resistor value is controlled by USB_SERIAL_JTAG_PULLUP_VALUE. For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _USB Serial/JTAG Controller_ ). 

      - When the USB function is disabled, USB pins are used as regular GPIOs and the pin’s internal weak pull-up and pull-down resistors are disabled by default (configurable by IO_MUX_FUN_ WPU/WPD). For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _IO MUX and GPIO Matrix_ . 

7. Depends on the value of EFUSE_DIS_PAD_JTAG 

   - 0 - WPU is enabled 

   - 1 - pin floating 

Some pins have glitches during power-up. See details in Table 2-2. 

Table 2-2. Power-Up Glitches on Pins 

|Pin|Glitch<sup>1</sup>|Typical Time Period (_µ_s)|
|---|---|---|
|GPIO1|Low-levelglitch|60|
|GPIO2|Low-levelglitch|60|
|GPIO3|Low-levelglitch|60|
|GPIO4|Low-levelglitch|60|
|GPIO5|Low-levelglitch|60|
|GPIO6|Low-levelglitch|60|
|GPIO7|Low-levelglitch|60|
|GPIO8|Low-levelglitch|60|
|GPIO9|Low-levelglitch|60|
|GPIO10|Low-levelglitch|60|
|GPIO11|Low-levelglitch|60|
|GPIO12|Low-levelglitch|60|
|GPIO13|Low-levelglitch|60|
|GPIO14|Low-levelglitch|60|
|XTAL_32K_P|Low-levelglitch|60|
|XTAL_32K_N|Low-levelglitch|60|
|GPIO17|Low-levelglitch|60|

Cont’d on next page 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

18 <u>Submit Documentation Feedback</u> 

2 Pins 

Table 2-2 – cont’d from previous page 

|Pin|Glitch<sup>1</sup>|Typical Time Period (_µ_s)|
|---|---|---|
||Low-levelglitch|60|
|GPIO18|High-levelglitch|60|
||Low-levelglitch|60|
|GPIO19|High-levelglitch<sup>2</sup>|60|
||Pull-downglitch|60|
|GPIO20|High-levelglitch<sup>2</sup>|60|

- 1 Low-level glitch: the pin is at a low level output status during the time period; High-level glitch: the pin is at a high level output status during the time period; Pull-down glitch: the pin is at an internal weak pulled-down status during the time period; Pull-up glitch: the pin is at an internal weak pulled-up status during the time period. Please refer to Table 5-4 _DC Characteristics (3.3 V, 25 °C)_ for detailed parameters about low/high-level and pull-down/up. 

- 2 GPIO19 and GPIO20 pins both have two high-level glitches during chip power-up, each lasting for about 60 _µ_ s. The total duration for the glitches and the delay are 3.2 ms and 2 ms respectively for GPIO19 and GPIO20. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

19 

<u>Submit Documentation Feedback</u> 

2 Pins 

## 2.3 IO Pins 

### 2.3.1 IO MUX Functions 

The IO MUX allows multiple input/output signals to be connected to a single input/output pin. Each IO pin of ESP32-S3 can be connected to one of the five signals (IO MUX functions, i.e., F0-F4), as listed in Table 2-4 _IO MUX Functions_ . 

Among the five sets of signals: 

- Some are routed via the GPIO Matrix (GPIO0, GPIO1, etc.), which incorporates internal signal routing circuitry for mapping signals programmatically. It gives the pin access to almost any peripheral signals. However, the flexibility of programmatic mapping comes at a cost as it might affect the latency of routed signals. For details about connecting to peripheral signals via GPIO Matrix, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _IO MUX and GPIO Matrix_ . 

- Some are directly routed from certain peripherals (U0TXD, MTCK, etc.), including UART0/1, JTAG, SPI0/1, and SPI2 - see Table 2-3 _Peripheral Signals Routed via IO MUX_ . 

Table 2-3. Peripheral Signals Routed via IO MUX 

|Pin Function|Signal|Description|
|---|---|---|
|U…TXD|Transmit data||
|U…RXD<br>U…RTS<br>U…CTS|Receive data<br>Request to send<br>Clear to send|UART0/1 interface|
|MTCK|Test clock||
|MTDO<br>MTDI|Test Data Out<br>Test Data In|JTAG interface for debugging|
|MTMS|Test Mode Select||
|SPIQ<br>SPID<br>SPIHD<br>SPIWP<br>SPICLK<br>SPICS…|Master in, slave out<br>Master out, slave in<br>Hold<br>Write protect<br>Clock<br>Chip select|SPI0/1 interface (powered by VDD_SPI) for connection to in-package or<br>off-package flash/PSRAM via the SPI bus. It supports 1-, 2-, 4-line SPI<br>modes. See also Section2.6_Pin Mapping Between Chip and_<br>_Flash/PSRAM_|
|SPIIO…<br>SPIDQS|Data<br>Data strobe/data mask|SPI0/1 interface (powered by VDD_SPI or VDD3P3_CPU) for the higher<br>4 bits data line interface and DQS interface in 8-line SPI mode|
|SPICLK_N_DIFF|Negative clock signal|Differential clock negative/positive for the SPI bus|
|SPICLK_P_DIFF|Positive clock signal||
|SUBSPIQ<br>SUBSPID<br>SUBSPIHD<br>SUBSPIWP<br>SUBSPICLK<br>SUBSPICS…|Master in, slave out<br>Master out, slave in<br>Hold<br>Write protect<br>Clock<br>Chip select|SPI0/1 interface (powered by VDD3P3_RTC or VDD3V3_CPU) for<br>connection to in-package oroff-package flash/PSRAM via the SUBSPI<br>bus. It supports 1-, 2-, 4-line SPI modes|
|SUBSPICLK_N_DIFF<br>SUBSPICLK_P_DIFF|Negative clock signal<br>Positive clock signal|Differential clock negative/positive for the SUBSPI bus|

Cont’d on next page 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

20 <u>Submit Documentation Feedback</u> 

2 Pins 

Table 2-3 – cont’d from previous page 

|Pin Function|Signal|Description|
|---|---|---|
|FSPIQ|Master in, slave out||
|FSPID|Master out, slave in||
|FSPIHD|Hold|SPI2 interface for fast SPI connection. It supports 1-, 2-, 4-line SPI|
|FSPIWP|Write protect|modes|
|FSPICLK|Clock||
|FSPICS0|Chip select||
|FSPIIO…|Data|The higher 4 bits data line interface and DQS interface for SPI2 interface|
|FSPIDQS|Data strobe/data mask|in 8-line SPI mode|
|CLK_OUT…|Clock output|Output clock signals generated by the chip’s internal components|

Table 2-4 _IO MUX Functions_ shows the IO MUX functions of IO pins. 

Table 2-4. IO MUX Functions 

|||||||IO MUX Function <sup>1,</sup>|<sup>2, 3</sup>|||||
|---|---|---|---|---|---|---|---|---|---|---|---|
|Pin No.|GPIO <sup>2</sup>|F0|Type <sup>3</sup>|F1|Type|F2|Type|F3|Type|F4|Type|
|5|GPIO0|GPIO0|I/O/T|GPIO0|I/O/T|||||||
|6|GPIO1|GPIO1|I/O/T|GPIO1|I/O/T|||||||
|7|GPIO2|GPIO2|I/O/T|GPIO2|I/O/T|||||||
|8|GPIO3|GPIO3|I/O/T|GPIO3|I/O/T|||||||
|9|GPIO4|GPIO4|I/O/T|GPIO4|I/O/T|||||||
|10|GPIO5|GPIO5|I/O/T|GPIO5|I/O/T|||||||
|11|GPIO6|GPIO6|I/O/T|GPIO6|I/O/T|||||||
|12|GPIO7|GPIO7|I/O/T|GPIO7|I/O/T|||||||
|13|GPIO8|GPIO8|I/O/T|GPIO8|I/O/T|||SUBSPICS1|O/T|||
|14|GPIO9|GPIO9|I/O/T|GPIO9|I/O/T|||SUBSPIHD|I1/O/T|FSPIHD|I1/O/T|
|15|GPIO10|GPIO10|I/O/T|GPIO10|I/O/T|FSPIIO4|I1/O/T|SUBSPICS0|O/T|FSPICS0|I1/O/T|
|16|GPIO11|GPIO11|I/O/T|GPIO11|I/O/T|FSPIIO5|I1/O/T|SUBSPID|I1/O/T|FSPID|I1/O/T|
|17|GPIO12|GPIO12|I/O/T|GPIO12|I/O/T|FSPIIO6|I1/O/T|SUBSPICLK|O/T|FSPICLK|I1/O/T|
|18|GPIO13|GPIO13|I/O/T|GPIO13|I/O/T|FSPIIO7|I1/O/T|SUBSPIQ|I1/O/T|FSPIQ|I1/O/T|
|19|GPIO14|GPIO14|I/O/T|GPIO14|I/O/T|FSPIDQS|O/T|SUBSPIWP|I1/O/T|FSPIWP|I1/O/T|
|21|GPIO15|GPIO15|I/O/T|GPIO15|I/O/T|U0RTS|O|||||
|22|GPIO16|GPIO16|I/O/T|GPIO16|I/O/T|U0CTS|I1|||||
|23|GPIO17|GPIO17|I/O/T|GPIO17|I/O/T|U1TXD|O|||||
|24|GPIO18|GPIO18|I/O/T|GPIO18|I/O/T|U1RXD|I1|CLK_OUT3|O|||
|25|GPIO19|GPIO19|I/O/T|GPIO19|I/O/T|U1RTS|O|CLK_OUT2|O|||
|26|GPIO20|GPIO20|I/O/T|GPIO20|I/O/T|U1CTS|I1|CLK_OUT1|O|||
|27|GPIO21|GPIO21|I/O/T|GPIO21|I/O/T|||||||
|28|GPIO26|SPICS1|O/T|GPIO26|I/O/T|||||||
|30|GPIO27|SPIHD|I1/O/T|GPIO27|I/O/T|||||||
|31|GPIO28|SPIWP|I1/O/T|GPIO28|I/O/T|||||||
|32|GPIO29|SPICS0|O/T|GPIO29|I/O/T|||||||
|33|GPIO30|SPICLK|O/T|GPIO30|I/O/T|||||||
|34|GPIO31|SPIQ|I1/O/T|GPIO31|I/O/T|||||||

Cont’d on next page 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

21 <u>Submit Documentation Feedback</u> 

2 Pins 

Cont’d from previous page 

|||||||IO MUX Function <sup>1,</sup>|<sup>2, 3</sup>|||||
|---|---|---|---|---|---|---|---|---|---|---|---|
|Pin No.|GPIO <sup>2</sup>|F0|Type <sup>3</sup>|F1|Type|F2|Type|F3|Type|F4|Type|
|35|GPIO32|SPID|I1/O/T|GPIO32|I/O/T|||||||
|36|GPIO48|SPICLK_N_DIFF|O/T|GPIO48|I/O/T|SUBSPICLK_N_DIFF|O/T|||||
|37|GPIO47|SPICLK_P_DIFF|O/T|GPIO47|I/O/T|SUBSPICLK_P_DIFF|O/T|||||
|38|GPIO33|GPIO33|I/O/T|GPIO33|I/O/T|FSPIHD|I1/O/T|SUBSPIHD|I1/O/T|SPIIO4|I1/O/T|
|39|GPIO34|GPIO34|I/O/T|GPIO34|I/O/T|FSPICS0|I1/O/T|SUBSPICS0|O/T|SPIIO5|I1/O/T|
|40|GPIO35|GPIO35|I/O/T|GPIO35|I/O/T|FSPID|I1/O/T|SUBSPID|I1/O/T|SPIIO6|I1/O/T|
|41|GPIO36|GPIO36|I/O/T|GPIO36|I/O/T|FSPICLK|I1/O/T|SUBSPICLK|O/T|SPIIO7|I1/O/T|
|42|GPIO37|GPIO37|I/O/T|GPIO37|I/O/T|FSPIQ|I1/O/T|SUBSPIQ|I1/O/T|SPIDQS|I0/O/T|
|43|GPIO38|GPIO38|I/O/T|GPIO38|I/O/T|FSPIWP|I1/O/T|SUBSPIWP|I1/O/T|||
|44|GPIO39|MTCK|I1|GPIO39|I/O/T|CLK_OUT3|O|SUBSPICS1|O/T|||
|45|GPIO40|MTDO|O/T|GPIO40|I/O/T|CLK_OUT2|O|||||
|47|GPIO41|MTDI|I1|GPIO41|I/O/T|CLK_OUT1|O|||||
|48|GPIO42|MTMS|I1|GPIO42|I/O/T|||||||
|49|GPIO43|U0TXD|O|GPIO43|I/O/T|CLK_OUT1|O|||||
|50|GPIO44|U0RXD|I1|GPIO44|I/O/T|CLK_OUT2|O|||||
|51|GPIO45|GPIO45|I/O/T|GPIO45|I/O/T|||||||
|52|GPIO46|GPIO46|I/O/T|GPIO46|I/O/T|||||||

> 1 Bold marks the default pin functions in the default boot mode. For more information about the boot mode，see Section 3.1 _Chip Boot Mode Control_ . 

- 2 Regarding highlighted cells, see Section 2.3.4 _Restrictions for GPIOs and RTC_GPIOs_ . 

- 3 Each IO MUX function (F _n_ , _n_ = 0 ~ 4) is associated with a _type_ . The description of _type_ is as follows: 

   - I – input. O – output. T – high impedance. 

   - I1 – input; if the pin is assigned a function other than F _n_ , the input signal of F _n_ is always 1. 

   - I0 – input; if the pin is assigned a function other than F _n_ , the input signal of F _n_ is always 0. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

22 <u>Submit Documentation Feedback</u> 

2 Pins 

### 2.3.2 RTC Functions 

When the chip is in Deep-sleep mode, the IO MUX described in Section 2.3.1 _IO MUX Functions_ will not work. That is where the RTC IO MUX comes in. It allows multiple input/output signals to be a single input/output pin in Deep-sleep mode, as the pin is connected to the RTC system and powered by VDD3P3_RTC. 

RTC IO pins can be assigned to RTC functions. They can 

- Either work as RTC GPIOs (RTC_GPIO0, RTC_GPIO1, etc.), connected to the ULP coprocessor 

- Or connect to RTC peripheral signals (sar_i2c_scl_0, sar_i2c_sda_0, etc.) - see Table 2-5 _RTC Peripheral Signals Routed via RTC IO MUX_ 

Table 2-5. RTC Peripheral Signals Routed via RTC IO MUX 

|Pin Function|Signal|Description|
|---|---|---|
|sar_i2c_scl…<br>sar_i2c_sda…|Serial clock<br>Serial data|RTC I2C0/1 interface|

Table 2-6 _RTC Functions_ shows the RTC functions of RTC IO pins. 

Table 2-6. RTC Functions 

|Pin|RTC|R|TC F|unctio|n <sup>2</sup>|
|---|---|---|---|---|---|
|No.|IO Name <sup>1</sup>|F0|F1|F2|F3|
|5|RTC_GPIO0|RTC_GPIO0|||sar_i2c_scl_0|
|6|RTC_GPIO1|RTC_GPIO1|||sar_i2c_sda_0|
|7|RTC_GPIO2|RTC_GPIO2|||sar_i2c_scl_1|
|8|RTC_GPIO3|RTC_GPIO3|||sar_i2c_sda_1|
|9|RTC_GPIO4|RTC_GPIO4||||
|10|RTC_GPIO5|RTC_GPIO5||||
|11|RTC_GPIO6|RTC_GPIO6||||
|12|RTC_GPIO7|RTC_GPIO7||||
|13|RTC_GPIO8|RTC_GPIO8||||
|14|RTC_GPIO9|RTC_GPIO9||||
|15|RTC_GPIO10|RTC_GPIO10||||
|16|RTC_GPIO11|RTC_GPIO11||||
|17|RTC_GPIO12|RTC_GPIO12||||
|18|RTC_GPIO13|RTC_GPIO13||||
|19|RTC_GPIO14|RTC_GPIO14||||
|21|RTC_GPIO15|RTC_GPIO15||||
|22|RTC_GPIO16|RTC_GPIO16||||
|23|RTC_GPIO17|RTC_GPIO17||||
|24|RTC_GPIO18|RTC_GPIO18||||
|25|RTC_GPIO19|RTC_GPIO19||||
|26|RTC_GPIO20|RTC_GPIO20||||
|27|RTC_GPIO21|RTC_GPIO21||||

> 1 This column lists the RTC GPIO names, since RTC functions are configured with RTC GPIO registers that use RTC GPIO numbering. 

- 2 Regarding highlighted cells, see Section 2.3.4 _Restrictions for GPIOs and RTC_GPIOs_ . 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

23 

<u>Submit Documentation Feedback</u> 

2 Pins 

### 2.3.3 Analog Functions 

Some IO pins also have analog functions, for analog peripherals (such as ADC) in any power mode. Internal analog signals are routed to these analog functions, see Table 2-7 _Analog Signals Routed to Analog Functions_ . 

Table 2-7. Analog Signals Routed to Analog Functions 

|Pin Function|Signal|Description|
|---|---|---|
|TOUCH…|Touch sensor channel … signal|Touch sensor interface|
|ADC…_CH…|ADC1/2 channel … signal|ADC1/2 interface|
|XTAL_32K_N|Negative clock signal|32 kHz external clock input/output|
|XTAL_32K_P|Positive clock signal|connected to ESP32-S3’s oscillator|
|USB_D-<br>USB_D+|Data -<br>Data +|USB OTG and USB Serial/JTAG function|

Table 2-8 _Analog Functions_ shows the analog functions of IO pins. 

Table 2-8. Analog Functions 

|||Analog Fun|ction <sup>1, 2</sup>|
|---|---|---|---|
|Pin No.|GPIO <sup>3</sup>|F0|F1|
|6|RTC_GPIO1|TOUCH1|ADC1_CH0|
|7|RTC_GPIO2|TOUCH2|ADC1_CH1|
|8|RTC_GPIO3|TOUCH3|ADC1_CH2|
|9|RTC_GPIO4|TOUCH4|ADC1_CH3|
|10|RTC_GPIO5|TOUCH5|ADC1_CH4|
|11|RTC_GPIO6|TOUCH6|ADC1_CH5|
|12|RTC_GPIO7|TOUCH7|ADC1_CH6|
|13|RTC_GPIO8|TOUCH8|ADC1_CH7|
|14|RTC_GPIO9|TOUCH9|ADC1_CH8|
|15|RTC_GPIO10|TOUCH10|ADC1_CH9|
|16|RTC_GPIO11|TOUCH11|ADC2_CH0|
|17|RTC_GPIO12|TOUCH12|ADC2_CH1|
|18|RTC_GPIO13|TOUCH13|ADC2_CH2|
|19|RTC_GPIO14|TOUCH14|ADC2_CH3|
|21|RTC_GPIO15|XTAL_32K_P|ADC2_CH4|
|22|RTC_GPIO16|XTAL_32K_N|ADC2_CH5|
|23|RTC_GPIO17||ADC2_CH6|
|24|RTC_GPIO18||ADC2_CH7|
|25|RTC_GPIO19|USB_D-|ADC2_CH8|
|26|RTC_GPIO20|USB_D+|ADC2_CH9|

- 1 Bold marks the default pin functions in the default boot mode. For more information about the boot mode，see Section 3.1 _Chip Boot Mode Control_ . 

- 2 This column lists the RTC GPIO names, since analog functions are configured with RTC GPIO registers that use RTC GPIO numbering. 

- 3 Regarding highlighted cells, see Section 2.3.4 _Restrictions for GPIOs and RTC_GPIOs_ . 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

24 <u>Submit Documentation Feedback</u> 

2 Pins 

### 2.3.4 Restrictions for GPIOs and RTC_GPIOs 

All IO pins of ESP32-S3 have GPIO and some have RTC_GPIO pin functions. However, the IO pins are multiplexed and can be configured for different purposes based on the requirements. Some IOs have restrictions for usage. It is essential to consider the multiplexed nature and the limitations when using these IO pins. 

In tables of this chapter, some pin functions are in <mark>red</mark> or yellow . These functions indicate pins that require extra caution when used as <mark>GPIO / GPIO :</mark> 

- <mark>IO Pins</mark> – allocated for communication with in-package flash/PSRAM and NOT recommended for other 

- uses. For details, see Section 2.6 _Pin Mapping Between Chip and Flash/PSRAM_ . 

- <mark>IO Pins</mark> – have one of the following important functions: 

   - Strapping pins – need to be at certain logic levels at startup. See Section 3 _Boot Configurations_ . 

Note: 

Strapping pins are highlighted by _Pin Name_ or configurations _At Reset_ , instead of the pin functions. 

- USB_D+/- – by default, connected to the USB Serial/JTAG Controller. To function as GPIOs, these pins need to be reconfigured via the IO_MUX_MCU_SEL bit (see 

   - _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _IO MUX and GPIO Matrix_ for details). 

- JTAG interface – often used for debugging. See Table 2-4 _IO MUX Functions_ . To free these pins up, the pin functions USB_D+/- of the USB Serial/JTAG Controller can be used instead. See also Section 3.4 _JTAG Signal Source Control_ . 

- UART0 interface – often used for debugging. See Table 2-4 _IO MUX Functions_ . 

- 8-line SPI interface – no restrictions, unless the chip is connected to flash/PSRAM using 8-line SPI mode. 

For more information about assigning pins, please see Section 2.3.5 _Peripheral Pin Assignment_ and ESP32-S3 Consolidated Pin Overview. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

25 <u>Submit Documentation Feedback</u> 

2 Pins 

### 2.3.5 Peripheral Pin Assignment 

Table 2-9 _Peripheral Pin Assignment_ highlights which pins can be assigned to each peripheral interface according to the following priorities: 

- Priority 1 (P1) : Fixed pins connected directly to peripheral signals via IO MUX or RTC IO MUX. 

- If a peripheral interface does not have priority 1 pins, such as UART2, it can be assigned to any GPIO pins from priority 2 to priority 4. 

- Any GPIO pins mapping to peripheral signals via GPIO Matrix, can be priority 2, 3, or 4. 

   - Priority 2 (P2) : GPIO pins can be freely used without restrictions. 

   - Priority 3 (P3) : GPIO pins should be used with caution, as they may conflict with the following 

   - important functions described in Section 2.3.4 _Restrictions for GPIOs and RTC_GPIOs_ : 

      - <mark>GPIO0, GPIO3, GPIO45, GPIO46 :</mark> Strapping pins. 

      - <mark>GPIO19, GPIO20 :</mark> USB Serial/JTAG interface. 

      - <mark>GPIO39, GPIO40, GPIO41, GPIO42</mark> : JTAG interface. 

      - <mark>GPIO43, GPIO44</mark> : UART0 interface. 

      - <mark>GPIO33, GPIO34, GPIO35, GPIO36, GPIO37 :</mark> The higher 4 bits data line interface and DQS 

      - interface for the SPI0/1 interface in 8-line SPI mode, and can be GPIO pins if the chip is not connected to flash or PSRAM in 8-line SPI mode. 

   - Priority 4 (P4) : GPIO pins already allocated or not recommended for use, as described in Section 

   - 2.3.4 _Restrictions for GPIOs and RTC_GPIOs_ : 

      - <mark>GPIO26, GPIO27, GPIO28, GPIO29, GPIO30, GPIO31, GPIO32 :</mark> SPI0/1 interface connected to 

      - the in-package flash and PSRAM, or recommended for the off-package flash and PSRAM. 

If a peripheral interface does not have priority 2 to 4 pins, such as USB Serial/JTAG, it means it can be assigned only to priority 1 pins. 

Note: 

- For details about which peripheral signals are connected to IO MUX or RTC IO MUX pins, please refer to Section 2.3.1 _IO MUX Functions_ or Section 2.3.2 _RTC Functions_ . 

- For details about which peripheral signals can be assigned to GPIO pins, please refer to _ESP32-S3 Technical Reference Manual_ > Chapter IO MUX and GPIO Matrix > Section Peripheral Signal List. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

26 <u>Submit Documentation Feedback</u> 

Table 2-9. Peripheral Pin Assignment 

2 Signals of UART0, UART1, SPI0/1, and SPI2 interfaces can be mapped to any GPIO pins through the GPIO Matrix, regardless of whether they are directly routed to fixed pins via IO MUX. 

2 Pins 

## 2.4 Analog Pins 

Table 2-10. Analog Pins 

|Pin|Pin|Pin|Pin|
|---|---|---|---|
|No.|Name|Type|Function|
|1|LNA_IN|I/O|Low Noise Amplifier (RF LNA) input/output signals|
||||High: on, enables the chip (powered up).|
|4|CHIP_PU|I|Low: off, disables the chip (powered down).|
||||Note: Do not leave the CHIP_PU pin floating.|
|53|XTAL_N|—|External clock input/output connected to chip’s crystal or oscillator.|
|54|XTAL_P|—|P/N means differential clock positive/negative.|

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

28 

<u>Submit Documentation Feedback</u> 

2 Pins 

## 2.5 Power Supply 

### 2.5.1 Power Pins 

The chip is powered via the power pins described in Table 2-11 _Power Pins_ . 

Table 2-11. Power Pins 

||||Power Supply <sup>1, 2</sup>||
|---|---|---|---|---|
|Pin No.|Pin Name|Direction|Power Domain/Other|IO Pins <sup>5</sup>|
|2|VDD3P3|Input|Analogpower domain||
|3|VDD3P3|Input|Analogpower domain||
|20|VDD3P3_RTC|Input|RTC and part of Digital power domains|RTC IO|
|29|VDDSPI<sup>3,4</sup>|Input|In-package memory (backup power line)||
||_|Output|In-package and off-package flash/PSRAM|SPI IO|
|46|VDD3P3_CPU|Input|Digital power domain|Digital IO|
|55|VDDA|Input|Analogpower domain||
|56|VDDA|Input|Analogpower domain||
|57|GND|–|Externalground connection||

- 1 See in conjunction with Section 2.5.2 _Power Scheme_ . 

- 2 For recommended and maximum voltage and current, see Section 5.1 _Absolute Maximum Ratings_ and Section 5.2 _Recommended Operating Conditions_ . 

- 3 To configure VDD_SPI as input or output, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Low-power Management_ . 

- 4 To configure output voltage, see Section 3.2 _VDD_SPI Voltage Control_ and Section 5.3 _VDD_SPI Output Characteristics_ . 

- 5 RTC IO pins are those powered by VDD3P3_RTC and so on, as shown in Figure 2-2 _ESP32-S3 Power Scheme_ . See also Table 2-1 _Pin Overview_ > Column _Pin Providing Power_ . 

### 2.5.2 Power Scheme 

The power scheme is shown in Figure 2-2 _ESP32-S3 Power Scheme_ . 

The components on the chip are powered via voltage regulators. 

Table 2-12. Voltage Regulators 

|Voltage Regulator|Output|Power Supply|
|---|---|---|
|Digital|1.1 V|Digital power domain|
|Low-power|1.1 V|RTC power domain|
|||Can be configured to power|
|Flash|1.8 V|in-package flash/PSRAM or|
|||off-package memory|

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

29 <u>Submit Documentation Feedback</u> 

2 Pins 

Figure 2-2. ESP32-S3 Power Scheme 

### 2.5.3 Chip Power-up and Reset 

Once the power is supplied to the chip, its power rails need a short time to stabilize. After that, CHIP_PU – the pin used for power-up and reset – is pulled high to activate the chip. For information on CHIP_PU as well as power-up and reset timing, see Figure 2-3 and Table 2-13. 

Figure 2-3. Visualization of Timing Parameters for Power-up and Reset 

Table 2-13. Description of Timing Parameters for Power-up and Reset 

|Parameter|Description|Min (_µ_s)|
|---|---|---|
||Time<br>reserved<br>for<br>the<br>power<br>rails<br>of<br>VDDA,<br>VDD3P3,||
|t_ST BL_|VDD3P3_RTC, and VDD3P3_CPU to stabilize before the CHIP_PU|50|
||pin is pulled high to activate the chip||
|t_RST_|Time reserved for CHIP_PU to stay below V_IL_nRST_ to reset the<br>chip (see Table5-4)|50|

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

30 <u>Submit Documentation Feedback</u> 

2 Pins 

## 2.6 Pin Mapping Between Chip and Flash/PSRAM 

Table 2-14 lists the pin mapping between the chip and flash/PSRAM for all SPI modes. 

For chip variants with in-package flash/PSRAM (see Table 1-1 _ESP32-S3 Series Comparison_ ), the pins allocated for communication with in-package flash/PSRAM can be identified depending on the SPI mode used. 

For off-package flash/PSRAM, these are the recommended pin mappings. 

For more information on SPI controllers, see also Section 4.2.1.5 _Serial Peripheral Interface (SPI)_ . 

Notice: Do not use the pins connected to in-package flash/PSRAM for any other purposes. 

Table 2-14. Pin Mapping Between Chip and Flash or PSRAM 

|||Singl|e SPI|Dua|l SPI|Quad|SPI/QPI|Octal|SPI/OPI|
|---|---|---|---|---|---|---|---|---|---|
|Pin No.|Pin Name|Flash|PSRAM|Flash|PSRAM|Flash|PSRAM|Flash|PSRAM|
|28|SPICS1<sup>2</sup>||CE#||CE#||CE#||CE#|
|30|SPIHD|HOLD#|SIO3|HOLD#|SIO3|HOLD#|SIO3|DQ3|DQ3|
|31|SPIWP|WP#|SIO2|WP#|SIO2|WP#|SIO2|DQ2|DQ2|
|32|SPICS0<sup>1</sup>|CS#||CS#||CS#||CS#||
|33|SPICLK|CLK|CLK|CLK|CLK|CLK|CLK|CLK|CLK|
|34|SPIQ|DO|SO/SIO1|DO|SO/SIO1|DO|SO/SIO1|DQ1|DQ1|
|35|SPID|DI|SI/SIO0|DI|SI/SIO0|DI|SI/SIO0|DQ0|DQ0|
|38|GPIO33|||||||DQ4|DQ4|
|39|GPIO34|||||||DQ5|DQ5|
|40|GPIO35|||||||DQ6|DQ6|
|41|GPIO36|||||||DQ7|DQ7|
|42|GPIO37|||||||DQS/DM|DQS/DM|

> 1 CS0 is for in-package flash 

> 2 CS1 is for in-package PSRAM 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

31 <u>Submit Documentation Feedback</u> 

3 Boot Configurations 

# 3 Boot Configurations 

The chip allows for configuring the following boot parameters through strapping pins and eFuse parameters at power-up or a hardware reset, without microcontroller interaction. 

- Chip boot mode 

   - Strapping pin: GPIO0 and GPIO46 

- VDD_SPI voltage 

   - Strapping pin: GPIO45 

   - eFuse parameter: EFUSE_VDD_SPI_FORCE and EFUSE_VDD_SPI_TIEH 

- ROM message printing 

   - Strapping pin: GPIO46 

   - eFuse parameter: EFUSE_UART_PRINT_CONTROL and EFUSE_DIS_USB_SERIAL_JTAG_ROM_PRINT 

- JTAG signal source 

   - Strapping pin: GPIO3 

   - eFuse parameter: EFUSE_DIS_PAD_JTAG, EFUSE_DIS_USB_JTAG, and EFUSE_STRAP_JTAG_SEL 

The default values of all the above eFuse parameters are 0, which means that they are not burnt. Given that eFuse is one-time programmable, once programmed to 1, it can never be reverted to 0. For how to program eFuse parameters, please refer to _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _eFuse Controller_ . 

The default values of the strapping pins, namely the logic levels, are determined by pins’ internal weak pull-up/pull-down resistors at reset if the pins are not connected to any circuit, or connected to an external high-impedance circuit. 

Table 3-1. Default Configuration of Strapping Pins 

|StrappingPin|Default Configuration|Bit Value|
|---|---|---|
|GPIO0|Weak pull-up|1|
|GPIO3|Floating|–|
|GPIO45|Weak pull-down|0|
|GPIO46|Weak pull-down|0|

To change the bit values, the strapping pins should be connected to external pull-down/pull-up resistances. If the ESP32-S3 is used as a device by a host MCU, the strapping pin voltage levels can also be controlled by the host MCU. 

All strapping pins have latches. At Chip Reset, the latches sample the bit values of their respective strapping pins and store them until the chip is powered down or shut down. The states of latches cannot be changed in any other way. It makes the strapping pin values available during the entire chip operation, and the pins are freed up to be used as regular IO pins after reset. For details on Chip Reset, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Reset and Clock_ . 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

32 <u>Submit Documentation Feedback</u> 

3 Boot Configurations 

The timing of signals connected to the strapping pins should adhere to the _setup time_ and _hold time_ specifications in Table 3-2 and Figure 3-1. 

Table 3-2. Description of Timing Parameters for the Strapping Pins 

|Parameter|Description|Min (ms)|
|---|---|---|
|t_SU_|_Setup time_is the time reserved for the power rails to stabilize be-<br>fore the CHIP_PU pin is pulled high to activate the chip.|0|
||_Hold time_ is the time reserved for the chip to read the strapping||
|t_H_|pin values after CHIP_PU is already high and before these pins|3|
||start operatingas regular IO pins.||

Figure 3-1. Visualization of Timing Parameters for the Strapping Pins 

## 3.1 Chip Boot Mode Control 

GPIO0 and GPIO46 control the boot mode after the reset is released. See Table 3-3 _Chip Boot Mode Control_ . 

Table 3-3. Chip Boot Mode Control 

|Boot Mode|GPIO0|GPIO46|
|---|---|---|
|SPI boot mode|1|Any value|
|Joint download boot mode <sup>2</sup>|0|0|

- 1 Bold marks the default value and configuration. 

- 2 Joint Download Boot mode supports the following download methods: 

   - USB Download Boot: 

      - USB-Serial-JTAG Download Boot 

      - USB-OTG Download Boot 

   - UART Download Boot 

In addition to SPI Boot and Joint Download Boot modes, ESP32-S3 also supports SPI Download Boot mode. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

33 <u>Submit Documentation Feedback</u> 

3 Boot Configurations 

For details, please see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Chip Boot Control_ . 

## 3.2 VDD_SPI Voltage Control 

The required VDD_SPI voltage for the chips of the ESP32-S3 Series can be found in Table 1-1 _ESP32-S3 Series Comparison_ . 

The VDD_SPI voltage can be: 

- (Default) 3.3 V supplied by VDD3P3_RTC via R _SP I_ 

- 1.8V supplied by the Flash Voltage Regulator 

The voltage is determined by EFUSE_VDD_SPI_FORCE, GPIO45, and EFUSE_VDD_SPI_TIEH. 

Table 3-4. VDD_SPI Voltage Control 

|VDD_SPI power source <sup>2</sup>|Voltage|EFUSE_VDD_SPI_FORCE|GPIO45|EFUSE_VDD_SPI_TIEH|
|---|---|---|---|---|
|VDD3P3RTC i R|33 V|0|0|Ignored|
|_ va_SP I_|.|1|Ignored|1|
|Flh Vl Rl|V|0|1|Ignored|
|as otage eguator|1.8|1|Ignored|0|

- 1 Bold marks the default value and configuration. 

- 2 See Section 2.5.2 _Power Scheme_ . 

## 3.3 ROM Messages Printing Control 

During the boot process, the messages by the ROM code can be printed to: 

- (Default) UART0 and USB Serial/JTAG controller 

- USB Serial/JTAG controller 

- UART0 

The ROM messages printing to UART or USB Serial/JTAG controller can be respectively disabled by configuring registers and eFuse. For detailed information, please refer to _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Chip Boot Control_ . 

## 3.4 JTAG Signal Source Control 

The strapping pin GPIO3 can be used to control the source of JTAG signals during the early boot process. This pin does not have any internal pull resistors and the strapping value must be controlled by the external circuit that cannot be in a high impedance state. 

As Table 3-5 _JTAG Signal Source Control_ shows, GPIO3 is used in combination with EFUSE_DIS_PAD_JTAG, EFUSE_DIS_USB_JTAG, and EFUSE_STRAP_JTAG_SEL. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

34 <u>Submit Documentation Feedback</u> 

3 Boot Configurations 

Table 3-5. JTAG Signal Source Control 

|JTAG Signal Source|EFUSE_DIS_PAD_JTAG|EFUSE_DIS_USB_JTAG|EFUSE_STRAP_JTAG_SEL|GPIO3|
|---|---|---|---|---|
||0|0|0|Ignored|
|USB Serieal/JTAG Controller|0|0|1|1|
||1|0|Ignored|Ignored|
|JTAG ins<sup>2</sup>|0|0|1|0|
|p|0|1|Ignored|Ignored|
|JTAG is disabled|1|1|Ignored|Ignored|

> 1 Bold marks the default value and configuration. 

> 2 JTAG pins refer to MTDI, MTCK, MTMS, and MTDO. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

35 

<u>Submit Documentation Feedback</u> 

4 Functional Description 

# 4 Functional Description 

## 4.1 System 

This section describes the core of the chip’s operation, covering its microprocessor, memory organization, system components, and security features. 

### 4.1.1 Microprocessor and Master 

This subsection describes the core processing units within the chip and their capabilities. 

### 4.1.1.1 CPU 

ESP32-S3 has a low-power Xtensa<sup>®</sup> dual-core 32-bit LX7 microprocessor. 

#### Feature List 

- Five-stage pipeline that supports the clock frequency of up to 240 MHz 

- 16-bit/24-bit instruction set providing high code density 

- 32-bit customized instruction set and 128-bit data bus that provide high computing performance 

- Support for single-precision floating-point unit (FPU) 

- 32-bit multiplier and 32-bit divider 

- Unbuffered GPIO instructions 

- 32 interrupts at six levels 

- Windowed ABI with 64 physical general registers 

- Trace function with TRAX compressor, up to 16 KB trace memory 

- JTAG for debugging 

For information about the Xtensa<sup>®</sup> Instruction Set Architecture, please refer to 

_<u>Xtensa</u>_<sup>®</sup> _<u>Instruction Set Architecture (ISA) Summary</u>_ <u>.</u> 

### 4.1.1.2 Processor Instruction Extensions (PIE) 

ESP32-S3 contains a series of new extended instruction set in order to improve the operation efficiency of specific AI and DSP (Digital Signal Processing) algorithms. 

#### Feature List 

- 128-bit new general-purpose registers 

- 128-bit vector operations, e.g., complex multiplication, addition, subtraction, multiplication, shifting, comparison, etc 

- Data handling instructions and load/store operation instructions combined 

- Non-aligned 128-bit vector data 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

36 <u>Submit Documentation Feedback</u> 

4 Functional Description 

- Saturation operation 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Processor Instruction Extensions_ . 

### 4.1.1.3 Ultra-Low-Power Coprocessor (ULP) 

The ULP coprocessor is designed as a simplified, low-power replacement of CPU in sleep modes. It can be also used to supplement the functions of the CPU in normal working mode. The ULP coprocessor and RTC memory remain powered up during the Deep-sleep mode. Hence, the developer can store a program for the ULP coprocessor in the RTC slow memory to access RTC GPIO, RTC peripheral devices, RTC timers and internal sensors in Deep-sleep mode. 

ESP32-S3 has two ULP coprocessors, one based on RISC-V instruction set architecture (ULP-RISC-V) and the other on finite state machine (ULP-FSM). The clock of the coprocessors is the internal fast RC oscillator. 

#### Feature List 

- ULP-RISC-V: 

   - Support for <u>RV32IMC</u> instruction set 

   - Thirty-two 32-bit general-purpose registers 

   - 32-bit multiplier and divider 

   - Support for interrupts 

   - Booted by the CPU, its dedicated timer, or RTC GPIO 

- ULP-FSM: 

   - Support for common instructions including arithmetic, jump, and program control instructions 

   - Support for on-board sensor measurement instructions 

   - Booted by the CPU, its dedicated timer, or RTC GPIO 

Note: 

Note that these two coprocessors cannot work simultaneously. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _ULP Coprocessor_ . 

### 4.1.1.4 GDMA Controller (GDMA) 

ESP32-S3 has a general-purpose DMA controller (GDMA) with five independent channels for transmitting and another five independent channels for receiving. These ten channels are shared by peripherals that have DMA feature, and support dynamic priority. 

The GDMA controller controls data transfer using linked lists. It allows peripheral-to-memory and memory-to-memory data transfer at a high speed. All channels can access internal and external RAM. 

The ten peripherals on ESP32-S3 with DMA feature are SPI2, SPI3, UHCI0, I2S0, I2S1, LCD/CAM, AES, SHA, ADC, and RMT. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _GDMA Controller_ . 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

37 <u>Submit Documentation Feedback</u> 

4 Functional Description 

### 4.1.2 Memory Organization 

This subsection describes the memory arrangement to explain how data is stored, accessed, and managed for efficient operation. 

Figure 4-1 illustrates the address mapping structure of ESP32-S3. 

Figure 4-1. Address Mapping Structure 

### 4.1.2.1 Internal Memory 

The internal memory of ESP32-S3 refers to the memory integrated on the chip die or in the chip package, including ROM, SRAM, eFuse, and flash. 

#### Feature List 

- 384 KB ROM: for booting and core functions 

- 512 KB on-chip SRAM: for data and instructions, running at a configurable frequency of up to 240 MHz 

- RTC FAST memory: 8 KB SRAM that supports read/write/instruction fetch by the main CPU (LX7 dual-core processor). It can retain data in Deep-sleep mode 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

38 <u>Submit Documentation Feedback</u> 

4 Functional Description 

- RTC SLOW Memory: 8 KB SRAM that supports read/write/instruction fetch by the main CPU (LX7 dual-core processor) or coprocessors. It can retain data in Deep-sleep mode 

- 4096-bit eFuse memory: 1792 bits are available for users, such as encryption key and device ID. See also Section 4.1.2.4 _eFuse Controller_ 

- In-package flash and PSRAM: 

   - See flash and PSRAM size in Chapter 1 _ESP32-S3 Series Comparison_ 

   - For specifications, refer to Section 5.7 _Memory Specifications_ . 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _System and Memory_ . 

### 4.1.2.2 External Flash and RAM 

ESP32-S3 supports SPI, Dual SPI, Quad SPI, Octal SPI, QPI, and OPI interfaces that allow connection to multiple external flash and RAM. 

The external flash and RAM can be mapped into the CPU instruction memory space and read-only data 

memory space. The external RAM can also be mapped into the CPU data memory space. ESP32-S3 supports up to 1 GB of external flash and RAM, and hardware encryption/decryption based on XTS-AES to protect users’ programs and data in flash and external RAM. 

Through high-speed caches, ESP32-S3 can support at a time up to: 

- External flash or RAM mapped into 32 MB instruction space as individual blocks of 64 KB 

- External RAM mapped into 32 MB data space as individual blocks of 64 KB. 8-bit, 16-bit, 32-bit, and 128-bit reads and writes are supported. External flash can also be mapped into 32 MB data space as individual blocks of 64 KB, but only supporting 8-bit, 16-bit, 32-bit and 128-bit reads. 

Note: 

After ESP32-S3 is initialized, firmware can customize the mapping of external RAM or flash into the CPU address space. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _System and Memory_ . 

### 4.1.2.3 Cache 

ESP32-S3 has an instruction cache and a data cache shared by the two CPU cores. Each cache can be partitioned into multiple banks. 

#### Feature List 

- Instruction cache: 16 KB (one bank) or 32 KB (two banks) Data cache: 32 KB (one bank) or 64 KB (two banks) 

- Instruction cache: four-way or eight-way set associative Data cache: four-way set associative 

- Block size of 16 bytes or 32 bytes for both instruction cache and data cache 

- Pre-load function 

- Lock function 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

39 <u>Submit Documentation Feedback</u> 

4 Functional Description 

- Critical word first and early restart 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _System and Memory_ . 

### 4.1.2.4 eFuse Controller 

ESP32-S3 contains a 4-Kbit eFuse to store parameters, which are burned and read by an eFuse controller. 

#### Feature List 

- 4 Kbits in total, with 1792 bits reserved for users, e.g., encryption key and device ID 

- One-time programmable storage 

- Configurable write protection 

- Configurable read protection 

- Various hardware encoding schemes to protect against data corruption 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _eFuse Controller_ . 

### 4.1.3 System Components 

This subsection describes the essential components that contribute to the overall functionality and control of the system. 

### 4.1.3.1 IO MUX and GPIO Matrix 

The IO MUX and GPIO Matrix in the ESP32-S3 chip provide flexible routing of peripheral input and output signals to the GPIO pins. These peripherals enhance the functionality and performance of the chip by allowing the configuration of I/O, support for multiplexing, and signal synchronization for peripheral inputs. 

#### Feature List 

- GPIO Matrix: 

   - A full-switching matrix between the peripheral input/output signals and the GPIO pins 

   - 175 digital peripheral input signals can be sourced from the input of any GPIO pins 

   - The output of any GPIO pins can be from any of the 184 digital peripheral output signals 

   - Supports signal synchronization for peripheral inputs based on APB clock bus 

   - Provides input signal filter 

   - Supports sigma delta modulated output 

   - Supports GPIO simple input and output 

- IO MUX: 

   - Provides one configuration register IO_MUX_GPIO _n_ _REG for each GPIO pin. The pin can be configured to 

      - perform GPIO function routed by GPIO matrix 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

40 <u>Submit Documentation Feedback</u> 

4 Functional Description 

      - or perform direct connection bypassing GPIO matrix 

   - Supports some high-speed digital signals (SPI, JTAG, UART) bypassing GPIO matrix for better high-frequency digital performance (IO MUX is used to connect these pins directly to peripherals) 

- RTC IO MUX: 

   - Controls low power feature of 22 RTC GPIO pins 

   - Controls analog functions of 22 RTC GPIO pins 

   - Redirects 22 RTC input/output signals to RTC system 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _IO MUX and GPIO Matrix_ . 

### 4.1.3.2 Reset 

ESP32-S3 provides four reset levels, namely CPU Reset, Core Reset, System Reset, and Chip Reset. 

#### Feature List 

- Support four reset levels: 

   - CPU Reset: only resets CPU _x_ core. CPU _x_ can be CPU0 or CPU1 here. Once such reset is released, programs will be executed from CPU _x_ reset vector. Each CPU core has its own reset logic. If CPU Reset is from CPU0, the <u>sensitive registers</u> will be reset, too. 

   - Core Reset: resets the whole digital system except RTC, including CPU0, CPU1, peripherals, Wi-Fi, Bluetooth<sup>®</sup> LE (BLE), and digital GPIOs. 

   - System Reset: resets the whole digital system, including RTC. 

   - Chip Reset: resets the whole chip. 

- Support software reset and hardware reset: 

   - Software reset is triggered by CPU _x_ configuring its corresponding registers. Refer to _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Low-power Management_ for more details. 

   - Hardware reset is directly triggered by the circuit. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Reset and Clock_ . 

### 4.1.3.3 Clock 

#### CPU Clock 

The CPU clock has three possible sources: 

- External main crystal clock 

- Internal fast RC oscillator (typically about 17.5 MHz, adjustable) 

- PLL clock 

The application can select the clock source from the three clocks above. The selected clock source drives the CPU clock directly, or after division, depending on the application. Once the CPU is reset, the default clock source would be the external main crystal clock divided by 2. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

41 <u>Submit Documentation Feedback</u> 

4 Functional Description 

Note: 

ESP32-S3 is unable to operate without an external main crystal clock. 

#### RTC Clock 

The RTC slow clock is used for RTC counter, RTC watchdog and low-power controller. It has three possible sources: 

- External low-speed (32 kHz) crystal clock 

- Internal slow RC oscillator (typically about 136 kHz, adjustable) 

- Internal fast RC oscillator divided clock (derived from the internal fast RC oscillator divided by 256) 

The RTC fast clock is used for RTC peripherals and sensor controllers. It has two possible sources: 

- External main crystal clock divided by 2 

- Internal fast RC oscillator (typically about 17.5 MHz, adjustable) 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Reset and Clock_ . 

### 4.1.3.4 Interrupt Matrix 

The interrupt matrix embedded in ESP32-S3 independently allocates peripheral interrupt sources to the two CPUs’ peripheral interrupts, to timely inform CPU0 or CPU1 to process the interrupts once the interrupt signals are generated. 

#### Feature List 

- 99 peripheral interrupt sources as input 

- Generate 26 peripheral interrupts to CPU0 and 26 peripheral interrupts to CPU1 as output. 

Note that the remaining six CPU0 interrupts and six CPU1 interrupts are internal interrupts. 

- Disable CPU non-maskable interrupt (NMI) sources 

- Query current interrupt status of peripheral interrupt sources 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Interrupt Matrix_ . 

### 4.1.3.5 Power Management Unit (PMU) 

ESP32-S3 has an advanced Power Management Unit (PMU). It can be flexibly configured to power up different power domains of the chip to achieve the best balance between chip performance, power consumption, and wakeup latency. 

The integrated Ultra-Low-Power (ULP) coprocessors allow ESP32-S3 to operate in Deep-sleep mode with most of the power domains turned off, thus achieving extremely low-power consumption. 

Configuring the PMU is a complex procedure. To simplify power management for typical scenarios, there are the following predefined power modes that power up different combinations of power domains: 

- Active mode – The CPU, RF circuits, and all peripherals are on. The chip can process data, receive, transmit, and listen. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

42 

<u>Submit Documentation Feedback</u> 

4 Functional Description 

- Modem-sleep mode – The CPU is on, but the clock frequency can be reduced. The wireless connections can be configured to remain active as RF circuits are periodically switched on when required. 

- Light-sleep mode – The CPU stops running, and can be optionally powered on. The RTC peripherals, as well as the ULP coprocessor can be woken up periodically by the timer. The chip can be woken up via all wake up mechanisms: MAC, RTC timer, or external interrupts. Wireless connections can remain active. Some groups of digital peripherals can be optionally powered off. 

- Deep-sleep mode – Only RTC is powered on. Wireless connection data is stored in RTC memory. 

For power consumption in different power modes, see Section 5.6 _Current Consumption_ . 

Figure 4-2 _Components and Power Domains_ and the following Table 4-1 show the distribution of chip components between power domains and power subdomains . 

Figure 4-2. Components and Power Domains 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

43 <u>Submit Documentation Feedback</u> 

4 Functional Description 

Table 4-1. Components and Power Domains 

||Power|RTC||Digita|l|||Anal|og||||
|---|---|---|---|---|---|---|---|---|---|---|---|---|
|Power<br>Mode|Domain||Optional<br>RTC<br>Periph||CPU|Optional<br>Digital<br>Periph|Wireless<br>Digital<br>Circuits||RC_<br>FAST_<br>CLK|XTAL_<br>CLK|PLL|RF<br>Circuits|
|Active||ON|ON|ON|ON|ON|ON|ON|ON|ON|ON|ON|
|Modem|-sleep|ON|ON|ON|ON|ON|ON<sup>1</sup>|ON|ON|ON|ON|OFF<sup>2</sup>|
|Light-sl|eep|ON|ON|ON|OFF<sup>1</sup>|ON<sup>1</sup>|OFF<sup>1</sup>|ON|OFF|OFF|OFF|OFF<sup>2</sup>|
|Deep-s|leep|ON|ON<sup>1</sup>|OFF|OFF|OFF|OFF|ON|OFF|OFF|OFF|OFF|

- 1 Configurable. See _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Low-power Management_ for more details. 

- 2 If Wireless Digital Circuits are on, RF circuits are periodically switched on when required by internal operation to keep active wireless connections running. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Low Power Management_ . 

### 4.1.3.6 System Timer 

ESP32-S3 integrates a 52-bit system timer, which has two 52-bit counters and three comparators. 

#### Feature List 

- Counters with a clock frequency of 16 MHz 

- Three types of independent interrupts generated according to alarm value 

- Two alarm modes: target mode and period mode 

- 52-bit target alarm value and 26-bit periodic alarm value 

- Read sleep time from RTC timer when the chip is awaken from Deep-sleep or Light-sleep mode 

- Counters can be stalled if the CPU is stalled or in OCD mode 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _System Timer_ . 

### 4.1.3.7 General Purpose Timers 

ESP32-S3 is embedded with four 54-bit general-purpose timers, which are based on 16-bit prescalers and 54-bit auto-reload-capable up/down-timers. 

#### Feature List 

- 16-bit clock prescaler, from 2 to 65536 

- 54-bit time-base counter programmable to be incrementing or decrementing 

- Able to read real-time value of the time-base counter 

- Halting and resuming the time-base counter 

- Programmable alarm generation 

- Timer value reload (Auto-reload at alarm or software-controlled instant reload) 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

44 

<u>Submit Documentation Feedback</u> 

4 Functional Description 

- Level interrupt generation 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Timer Group_ . 

### 4.1.3.8 Watchdog Timers 

ESP32-S3 contains three watchdog timers: one in each of the two timer groups (called Main System Watchdog Timers, or MWDT) and one in the RTC Module (called the RTC Watchdog Timer, or RWDT). 

During the flash boot process, RWDT and the first MWDT are enabled automatically in order to detect and recover from booting errors. 

#### Feature List 

- Four stages: 

   - Each with a programmable timeout value 

   - Each stage can be configured, enabled and disabled separately 

- Upon expiry of each stage: 

   - Interrupt, CPU reset, or core reset occurs for MWDT 

   - Interrupt, CPU reset, core reset, or system reset occurs for RWDT 

- 32-bit expiry counter 

- Write protection, to prevent RWDT and MWDT configuration from being altered inadvertently 

- Flash boot protection: If the boot process from an SPI flash does not complete within a predetermined period of time, the watchdog will reboot the entire main system 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Watchdog Timers_ . 

### 4.1.3.9 XTAL32K Watchdog Timers 

#### Interrupt and Wake-Up 

When the XTAL32K watchdog timer detects the oscillation failure of XTAL32K_CLK, an oscillation failure interrupt RTC_XTAL32K_DEAD_INT (for interrupt description, please refer to 

_<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Low-power Management_ ) is generated. At this point, the CPU will be woken up if in Light-sleep mode or Deep-sleep mode. 

#### BACKUP32K_CLK 

Once the XTAL32K watchdog timer detects the oscillation failure of XTAL32K_CLK, it replaces XTAL32K_CLK with BACKUP32K_CLK (with a frequency of 32 kHz or so) derived from RTC_CLK as RTC’s SLOW_CLK, so as to ensure proper functioning of the system. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _XTAL32K Watchdog Timers_ . 

### 4.1.3.10 Permission Control 

In ESP32-S3, the Permission Control module is used to control access to the slaves (including internal memory, peripherals, external flash, and RAM). The host can access its slave only if it has the right permission. In this way, data and instructions are protected from illegitimate read or write. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

45 <u>Submit Documentation Feedback</u> 

4 Functional Description 

The ESP32-S3 CPU can run in both Secure World and Non-secure World where independent permission controls are adopted. The Permission Control module is able to identify which World the host is running and then proceed with its normal operations. 

#### Feature List 

- Manage access to internal memory by: 

   - CPU 

   - CPU trace module 

   - GDMA 

- Manage access to external flash and RAM by: 

   - MMU 

   - SPI1 

   - GDMA 

   - CPU through Cache 

- Manage access to peripherals, supporting 

   - independent permission control for each peripheral 

   - monitoring non-aligned access 

   - access control for customized address range 

- Integrate permission lock register 

   - All permission registers can be locked with the permission lock register. Once locked, the permission register and the lock register cannot be modified, unless the CPU is reset. 

- Integrate permission monitor interrupt 

   - In case of illegitimate access, the permission monitor interrupt will be triggered and the CPU will be informed to handle the interrupt. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Permission Control_ . 

### 4.1.3.11 World Controller 

ESP32-S3 can divide the hardware and software resources into a Secure World and a Non-Secure World to prevent sabotage or access to device information. Switching between the two worlds is performed by the World Controller. 

#### Feature List 

- Control of the CPU switching between secure and non-secure worlds 

- Control of 15 DMA peripherals switching between secure and non-secure worlds 

- Record of CPU’s world switching logs 

- Shielding of the CPU’s NMI interrupt 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

46 <u>Submit Documentation Feedback</u> 

4 Functional Description 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _World Controller_ . 

### 4.1.3.12 System Registers 

ESP32-S3 system registers can be used to control the following peripheral blocks and core modules: 

- System and memory 

- Clock 

- Software Interrupt 

- Low-power management 

- Peripheral clock gating and reset 

- CPU Control 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _System Registers_ . 

### 4.1.4 Cryptography and Security Component 

This subsection describes the security features incorporated into the chip, which safeguard data and operations. 

### 4.1.4.1 SHA Accelerator 

ESP32-S3 integrates an SHA accelerator, which is a hardware device that speeds up SHA algorithm significantly. 

#### Feature List 

- All the hash algorithms introduced in <u>FIPS PUB 180-4 Spec.</u> 

   - SHA-1 

   - SHA-224 

   - SHA-256 

   - SHA-384 

   - SHA-512 

   - SHA-512/224 

   - SHA-512/256 

   - SHA-512/ _t_ 

- Two working modes 

   - Typical SHA 

   - DMA-SHA 

- interleaved function when working in Typical SHA working mode 

- Interrupt function when working in DMA-SHA working mode 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _SHA Accelerator_ . 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

47 <u>Submit Documentation Feedback</u> 

4 Functional Description 

### 4.1.4.2 AES Accelerator 

ESP32-S3 integrates an Advanced Encryption Standard (AES) Accelerator, which is a hardware device that speeds up AES algorithm significantly. 

#### Feature List 

- Typical AES working mode 

   - AES-128/AES-256 encryption and decryption 

- DMA-AES working mode 

   - AES-128/AES-256 encryption and decryption 

   - Block cipher mode 

      - ECB (Electronic Codebook) 

      - CBC (Cipher Block Chaining) 

      - OFB (Output Feedback) 

      - CTR (Counter) 

      - CFB8 (8-bit Cipher Feedback) 

      - CFB128 (128-bit Cipher Feedback) 

   - Interrupt on completion of computation 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _AES Accelerator_ . 

### 4.1.4.3 RSA Accelerator 

The RSA Accelerator provides hardware support for high precision computation used in various RSA asymmetric cipher algorithms. 

#### Feature List 

- Large-number modular exponentiation with two optional acceleration options 

- Large-number modular multiplication, up to 4096 bits 

- Large-number multiplication, with operands up to 2048 bits 

- Operands of different lengths 

- Interrupt on completion of computation 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _RSA Accelerator_ . 

### 4.1.4.4 Secure Boot 

Secure Boot feature uses a hardware root of trust to ensure only signed firmware (with RSA-PSS signature) can be booted. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

48 <u>Submit Documentation Feedback</u> 

4 Functional Description 

### 4.1.4.5 HMAC Accelerator 

The Hash-based Message Authentication Code (HMAC) module computes Message Authentication Codes (MACs) using Hash algorithm and keys as described in RFC 2104. 

#### Feature List 

- Standard HMAC-SHA-256 algorithm 

- Hash result only accessible by configurable hardware peripheral (in downstream mode) 

- Compatible to challenge-response authentication algorithm 

- Generates required keys for the RSA Digital Signature Peripheral (RSA_DS) (in downstream mode) 

- Re-enables soft-disabled JTAG (in downstream mode) 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _HMAC Accelerator_ . 

### 4.1.4.6 RSA Digital Signature Peripheral (RSA_DS) 

An RSA Digital Signature Peripheral (RSA_DS) is used to verify the authenticity and integrity of a message using a cryptographic algorithm. 

#### Feature List 

- RSA_DS with key length up to 4096 bits 

- Encrypted private key data, only decryptable by RSA_DS 

- SHA-256 digest to protect private key data against tampering by an attacker 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _RSA Digital Signature Peripheral (RSA_DS)_ . 

### 4.1.4.7 External Memory Encryption and Decryption 

ESP32-S3 integrates an External Memory Encryption and Decryption module that complies with the XTS-AES standard. 

#### Feature List 

- General XTS-AES algorithm, compliant with IEEE Std 1619-2007 

- Software-based manual encryption 

- High-speed auto encryption, without software’s participation 

- High-speed auto decryption, without software’s participation 

- Encryption and decryption functions jointly determined by registers configuration, eFuse parameters, and boot mode 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _External Memory Encryption and Decryption_ . 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

49 

<u>Submit Documentation Feedback</u> 

4 Functional Description 

### 4.1.4.8 Clock Glitch Detection 

The Clock Glitch Detection module on ESP32-S3 monitors input clock signals from XTAL_CLK. If it detects a glitch with a width shorter than 3 ns, input clock signals from XTAL_CLK are blocked. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Clock Glitch Detection_ . 

### 4.1.4.9 Random Number Generator 

The random number generator (RNG) in ESP32-S3 generates true random numbers, which means random number generated from a physical process, rather than by means of an algorithm. No number generated within the specified range is more or less likely to appear than any other number. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Random Number Generator_ . 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

50 

<u>Submit Documentation Feedback</u> 

4 Functional Description 

## 4.2 Peripherals 

This section describes the chip’s peripheral capabilities, covering connectivity interfaces and on-chip sensors that extend its functionality. 

### 4.2.1 Connectivity Interface 

This subsection describes the connectivity interfaces on the chip that enable communication and interaction with external devices and networks. 

### 4.2.1.1 UART Controller 

ESP32-S3 has three UART (Universal Asynchronous Receiver Transmitter) controllers, i.e., UART0, UART1, and UART2, which support IrDA and asynchronous communication (RS232 and RS485) at a speed of up to 5 Mbps. 

#### Feature List 

- Three clock sources that can be divided 

- Programmable baud rate 

- 1024 x 8-bit RAM shared by TX FIFOs and RX FIFOs of the three UART controllers 

- Full-duplex asynchronous communication 

- Automatic baud rate detection of input signals 

- Data bits ranging from 5 to 8 

- Stop bits of 1, 1.5, 2, or 3 bits 

- Parity bit 

- Special character AT_CMD detection 

- RS485 protocol 

- IrDA protocol 

- High-speed data communication using GDMA 

- UART as wake-up source 

- Software and hardware flow control 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _UART Controller_ . 

#### Pin Assignment 

For details, see Section 2.3.5 _Peripheral Pin Assignment_ . 

### 4.2.1.2 I2C Interface 

ESP32-S3 has two I2C bus interfaces which are used for I2C master mode or slave mode, depending on the user’s configuration. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

51 

<u>Submit Documentation Feedback</u> 

4 Functional Description 

#### Feature List 

- Standard mode (100 kbit/s) 

- Fast mode (400 kbit/s) 

- Up to 800 kbit/s (constrained by SCL and SDA pull-up strength) 

- 7-bit and 10-bit addressing mode 

- Double addressing mode (slave addressing and slave register addressing) 

The hardware provides a command abstraction layer to simplify the usage of the I2C peripheral. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _I2C Controller_ . 

#### Pin Assignment 

For details, see Section 2.3.5 _Peripheral Pin Assignment_ . 

### 4.2.1.3 I2S Interface 

ESP32-S3 includes two standard I2S interfaces. They can operate in master mode or slave mode, in full-duplex mode or half-duplex communication mode, and can be configured to operate with an 8-bit, 16-bit, 24-bit, or 32-bit resolution as an input or output channel. BCK clock frequency, from 10 kHz up to 40 MHz, is supported. 

The I2S interface has a dedicated DMA controller. It supports TDM PCM, TDM MSB alignment, TDM LSB alignment, TDM Phillips, and PDM interface. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _I2S Controller_ . 

#### Pin Assignment 

For details, see Section 2.3.5 _Peripheral Pin Assignment_ . 

### 4.2.1.4 LCD and Camera Controller 

The LCD and Camera controller of ESP32-S3 consists of a LCD module and a camera module. 

The LCD module is designed to send parallel video data signals, and its bus supports 8-bit ~ 16-bit parallel RGB, I8080, and MOTO6800 interfaces. These interfaces operate at 40 MHz or lower, and support conversion among RGB565, YUV422, YUV420, and YUV411. 

The camera module is designed to receive parallel video data signals, and its bus supports an 8-bit ~ 16-bit DVP image sensor, with clock frequency of up to 40 MHz. The camera interface supports conversion among RGB565, YUV422, YUV420, and YUV411. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _LCD and Camera Controller_ . 

#### Pin Assignment 

For details, see Section 2.3.5 _Peripheral Pin Assignment_ . 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

52 <u>Submit Documentation Feedback</u> 

4 Functional Description 

### 4.2.1.5 Serial Peripheral Interface (SPI) 

ESP32-S3 has the following SPI interfaces: 

- SPI0 used by ESP32-S3’s GDMA controller and cache to access in-package or off-package flash/PSRAM 

- SPI1 used by the CPU to access in-package or off-package flash/PSRAM 

- SPI2 is a general purpose SPI controller with access to a DMA channel allocated by the GDMA controller 

- SPI3 is a general purpose SPI controller with access to a DMA channel allocated by the GDMA controller 

#### Feature List 

- SPI0 and SPI1: 

   - Supports Single SPI, Dual SPI, Quad SPI, Octal SPI, QPI, and OPI modes 

   - 8-line SPI mode supports single data rate (SDR) and double data rate (DDR) 

   - Configurable clock frequency with a maximum of 120 MHz for 8-line SPI SDR/DDR modes 

   - Data transmission is in bytes 

- SPI2: 

   - Supports operation as a master or slave 

   - Connects to a DMA channel allocated by the GDMA controller 

   - Supports Single SPI, Dual SPI, Quad SPI, Octal SPI, QPI, and OPI modes 

   - Configurable clock polarity (CPOL) and phase (CPHA) 

   - Configurable clock frequency 

   - Data transmission is in bytes 

   - Configurable read and write data bit order: most-significant bit (MSB) first, or least-significant bit (LSB) first 

   - As a master 

      - Supports 2-line full-duplex communication with clock frequency up to 80 MHz 

      - Full-duplex 8-line SPI mode supports single data rate (SDR) only 

      - Supports 1-, 2-, 4-, 8-line half-duplex communication with clock frequency up to 80 MHz 

      - Half-duplex 8-line SPI mode supports both single data rate (up to 80 MHz) and double data rate (up to 40 MHz) 

      - Provides six SPI_CS pins for connection with six independent SPI slaves 

      - Configurable CS setup time and hold time 

   - As a slave 

      - Supports 2-line full-duplex communication with clock frequency up to 60 MHz 

      - Supports 1-, 2-, 4-line half-duplex communication with clock frequency up to 60 MHz 

      - Full-duplex and half-duplex 8-line SPI mode supports single data rate (SDR) only 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

53 <u>Submit Documentation Feedback</u> 

4 Functional Description 

- SPI3: 

   - Supports operation as a master or slave 

   - Connects to a DMA channel allocated by the GDMA controller 

   - Supports Single SPI, Dual SPI, Quad SPI, and QPI modes 

   - Configurable clock polarity (CPOL) and phase (CPHA) 

   - Configurable clock frequency 

   - Data transmission is in bytes 

   - Configurable read and write data bit order: most-significant bit (MSB) first, or least-significant bit (LSB) first 

   - As a master 

      - Supports 2-line full-duplex communication with clock frequency up to 80 MHz 

      - Supports 1-, 2-, 4-line half-duplex communication with clock frequency up to 80 MHz 

      - Provides three SPI_CS pins for connection with three independent SPI slaves 

      - Configurable CS setup time and hold time 

   - As a slave 

      - Supports 2-line full-duplex communication with clock frequency up to 60 MHz 

      - Supports 1-, 2-, 4-line half-duplex communication with clock frequency up to 60 MHz 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _SPI Controller_ . 

#### Pin Assignment 

For details, see Section 2.3.5 _Peripheral Pin Assignment_ . 

### 4.2.1.6 Two-Wire Automotive Interface (TWAI<sup>®</sup> ) 

The Two-Wire Automotive Interface (TWAI<sup>®</sup> ) is a multi-master, multi-cast communication protocol with error detection and signaling as well as inbuilt message priorities and arbitration. 

#### Feature List 

- Compatible with ISO 11898-1 protocol (CAN Specification 2.0) 

- Standard frame format (11-bit ID) and extended frame format (29-bit ID) 

- Bit rates from 1 Kbit/s to 1 Mbit/s 

- Multiple modes of operation: 

   - Normal 

   - Listen Only 

   - Self-Test (no acknowledgment required) 

- 64-byte receive FIFO 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

54 <u>Submit Documentation Feedback</u> 

4 Functional Description 

- Acceptance filter (single and dual filter modes) 

- Error detection and handling: 

   - Error counters 

   - Configurable error interrupt threshold 

   - Error code capture 

   - Arbitration lost capture 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Two-wire Automotive Interface_ . 

#### Pin Assignment 

For details, see Section 2.3.5 _Peripheral Pin Assignment_ . 

### 4.2.1.7 USB 2.0 OTG Full-Speed Interface 

ESP32-S3 features a full-speed USB OTG interface along with an integrated transceiver. The USB OTG interface complies with the USB 2.0 specification. 

#### General Features 

- FS and LS data rates 

- HNP and SRP as A-device or B-device 

- Dynamic FIFO (DFIFO) sizing 

- Multiple modes of memory access 

   - Scatter/Gather DMA mode 

   - Buffer DMA mode 

   - Slave mode 

- Can choose integrated transceiver or external transceiver 

- Utilizing integrated transceiver with USB Serial/JTAG by time-division multiplexing when only integrated transceiver is used 

- Support USB OTG using one of the transceivers while USB Serial/JTAG using the other one when both integrated transceiver or external transceiver are used 

#### Device Mode Features 

- Endpoint number 0 always present (bi-directional, consisting of EP0 IN and EP0 OUT) 

- Six additional endpoints (endpoint numbers 1 to 6), configurable as IN or OUT 

- Maximum of five IN endpoints concurrently active at any time (including EP0 IN) 

- All OUT endpoints share a single RX FIFO 

- Each IN endpoint has a dedicated TX FIFO 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

55 <u>Submit Documentation Feedback</u> 

4 Functional Description 

#### Host Mode Features 

- Eight channels (pipes) 

   - A control pipe consists of two channels (IN and OUT), as IN and OUT transactions must be handled separately. Only Control transfer type is supported. 

   - Each of the other seven channels is dynamically configurable to be IN or OUT, and supports Bulk, Isochronous, and Interrupt transfer types. 

- All channels share an RX FIFO, non-periodic TX FIFO, and periodic TX FIFO. The size of each FIFO is configurable. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _USB On-The-Go_ . 

#### Pin Assignment 

For details, see Section 2.3.5 _Peripheral Pin Assignment_ . 

### 4.2.1.8 USB Serial/JTAG Controller 

ESP32-S3 integrates a USB Serial/JTAG controller. 

#### Feature List 

- USB Full-speed device. 

- Can be configured to either use internal USB PHY of ESP32-S3 or external PHY via GPIO matrix. 

- Fixed function device, hardwired for CDC-ACM (Communication Device Class - Abstract Control Model) and JTAG adapter functionality. 

- Two OUT Endpoints, three IN Endpoints in addition to Control Endpoint 0; Up to 64-byte data payload size. 

- Internal PHY, so no or very few external components needed to connect to a host computer. 

- CDC-ACM adherent serial port emulation is plug-and-play on most modern OSes. 

- JTAG interface allows fast communication with CPU debug core using a compact representation of JTAG instructions. 

- CDC-ACM supports host controllable chip reset and entry into download mode. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _USB Serial/JTAG Controller_ . 

#### Pin Assignment 

For details, see Section 2.3.5 _Peripheral Pin Assignment_ . 

### 4.2.1.9 SD/MMC Host Controller 

ESP32-S3 has an SD/MMC Host controller. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

56 <u>Submit Documentation Feedback</u> 

4 Functional Description 

#### Feature List 

- Secure Digital (SD) memory version 3.0 and version 3.01 

- Secure Digital I/O (SDIO) version 3.0 

- Consumer Electronics Advanced Transport Architecture (CE-ATA) version 1.1 

- Multimedia Cards (MMC version 4.41, eMMC version 4.5 and version 4.51) 

- Up to 80 MHz clock output 

- Three data bus modes: 

   - 1-bit 

   - 4-bit (supports two SD/SDIO/MMC 4.41 cards, and one SD card operating at 1.8 V in 4-bit mode) 

   - 8-bit 

##### Note: 

When working at 80 MHz, the clock phase adjustment is limited and only phase 0° and 180° are supported. The PCB layout should be optimized accordingly to ensure timing closure. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _SD/MMC Host Controller_ . 

#### Pin Assignment 

For details, see Section 2.3.5 _Peripheral Pin Assignment_ . 

#### Feature List 

- Can generate a digital waveform with configurable periods and duty cycle. The duty cycle resolution can be up to 14 bits within a 1 ms period 

- Multiple clock sources, including APB clock and external main crystal clock 

- Can operate when the CPU is in Light-sleep mode 

- Gradual increase or decrease of duty cycle, useful for the LED RGB color-fading generator 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _LED PWM Controller_ . 

#### Pin Assignment 

For details, see Section 2.3.5 _Peripheral Pin Assignment_ . 

### 4.2.1.10 Motor Control PWM (MCPWM) 

ESP32-S3 integrates two MCPWMs that can be used to drive digital motors and smart light. Each MCPWM peripheral has one clock divider (prescaler), three PWM timers, three PWM operators, and a capture module. PWM timers are used for generating timing references. The PWM operators generate desired waveform based on the timing references. Any PWM operator can be configured to use the timing references of any PWM timers. Different PWM operators can use the same PWM timer’s timing references to produce related PWM 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

57 <u>Submit Documentation Feedback</u> 

4 Functional Description 

signals. PWM operators can also use different PWM timers’ values to produce the PWM signals that work alone. Different PWM timers can also be synchronized together. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Motor Control PWM_ . 

#### Pin Assignment 

For details, see Section 2.3.5 _Peripheral Pin Assignment_ . 

### 4.2.1.11 Remote Control Peripheral (RMT) 

The Remote Control Peripheral (RMT) is designed to send and receive infrared remote control signals. 

#### Feature List 

- Four TX channels 

- Four RX channels 

- Support multiple channels (programmable) transmitting data simultaneously 

- Eight channels share a 384 x 32-bit RAM 

- Support modulation on TX pulses 

- Support filtering and demodulation on RX pulses 

- Wrap TX mode 

- Wrap RX mode 

- Continuous TX mode 

- DMA access for TX mode on channel 3 

- DMA access for RX mode on channel 7 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Remote Control Peripheral_ . 

#### Pin Assignment 

For details, see Section 2.3.5 _Peripheral Pin Assignment_ . 

### 4.2.1.12 Pulse Count Controller (PCNT) 

The pulse count controller (PCNT) captures pulse and counts pulse edges through multiple modes. 

#### Feature List 

- Four independent pulse counters (units) that count from 1 to 65535 

- Each unit consists of two independent channels sharing one pulse counter 

- All channels have input pulse signals (e.g. sig_ch0_u _n_ ) with their corresponding control signals (e.g. ctrl_ch0_u _n_ ) 

- Independently filter glitches of input pulse signals (sig_ch0_u _n_ and sig_ch1_u _n_ ) and control signals (ctrl_ch0_u _n_ and ctrl_ch1_u _n_ ) on each unit 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

58 

<u>Submit Documentation Feedback</u> 

4 Functional Description 

- Each channel has the following parameters: 

   1. Selection between counting on positive or negative edges of the input pulse signal 

   2. Configuration to Increment, Decrement, or Disable counter mode for control signal’s high and low states 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Pulse Count Controller_ . 

#### Pin Assignment 

For details, see Section 2.3.5 _Peripheral Pin Assignment_ . 

### 4.2.2 Analog Signal Processing 

This subsection describes components on the chip that sense and process real-world data. 

### 4.2.2.1 SAR ADC 

ESP32-S3 integrates two 12-bit SAR ADCs and supports measurements on 20 channels (analog-enabled pins). For power-saving purpose, the ULP coprocessors in ESP32-S3 can also be used to measure voltage in sleep modes. By using threshold settings or other methods, we can awaken the CPU from sleep modes. 

Note: 

Please note that the ADC2_CH… analog functions (see Table 2-8 _Analog Functions_ ) cannot be used with Wi-Fi simultaneously. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _On-Chip Sensors and Analog Signal Processing_ . 

#### Pin Assignment 

For details, see Section 2.3.5 _Peripheral Pin Assignment_ . 

### 4.2.2.2 Temperature Sensor 

The temperature sensor generates a voltage that varies with temperature. The voltage is internally converted via an ADC into a digital value. 

The temperature sensor has a range of –40 °C to 125 °C. It is designed primarily to sense the temperature changes inside the chip. The temperature value depends on factors such as microcontroller clock frequency or I/O load. Generally, the chip’s internal temperature is higher than the ambient temperature. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _On-Chip Sensors and Analog Signal Processing_ . 

### 4.2.2.3 Touch Sensor 

ESP32-S3 has 14 capacitive-sensing GPIOs, which detect variations induced by touching or approaching the GPIOs with a finger or other objects. The low-noise nature of the design and the high sensitivity of the circuit allow relatively small pads to be used. Arrays of pads can also be used, so that a larger area or more points 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

59 <u>Submit Documentation Feedback</u> 

4 Functional Description 

can be detected. The touch sensing performance can be further enhanced by the waterproof design and digital filtering feature. 

Note: 

ESP32-S3 touch sensor has not passed the Conducted Susceptibility (CS) test for now, and thus has limited application scenarios. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _On-Chip Sensors and Analog Signal Processing_ . 

#### Pin Assignment 

For details, see Section 2.3.5 _Peripheral Pin Assignment_ . 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

60 

<u>Submit Documentation Feedback</u> 

4 Functional Description 

## 4.3 Wireless Communication 

This section describes the chip’s wireless communication capabilities, spanning radio technology, Wi-Fi, Bluetooth, and 802.15.4. 

### 4.3.1 Radio 

This subsection describes the fundamental radio technology embedded in the chip that facilitates wireless communication and data exchange. 

### 4.3.1.1 2.4 GHz Receiver 

The 2.4 GHz receiver demodulates the 2.4 GHz RF signal to quadrature baseband signals and converts them to the digital domain with two high-resolution, high-speed ADCs. To adapt to varying signal channel conditions, ESP32-S3 integrates RF filters, Automatic Gain Control (AGC), DC offset cancelation circuits, and baseband filters. 

### 4.3.1.2 2.4 GHz Transmitter 

The 2.4 GHz transmitter modulates the quadrature baseband signals to the 2.4 GHz RF signal, and drives the antenna with a high-powered CMOS power amplifier. The use of digital calibration further improves the linearity of the power amplifier. 

To compensate for receiver imperfections, additional calibration methods are built into the chip, including: 

- Carrier leakage compensation 

- I/Q amplitude/phase matching 

- Baseband nonlinearities suppression 

- RF nonlinearities suppression 

- Antenna matching 

These built-in calibration routines reduce the cost and time to the market for your product, and eliminate the need for specialized testing equipment. 

### 4.3.1.3 Clock Generator 

The clock generator produces quadrature clock signals of 2.4 GHz for both the receiver and the transmitter. All components of the clock generator are integrated into the chip, including inductors, varactors, filters, regulators, and dividers. 

The clock generator has built-in calibration and self-test circuits. Quadrature clock phases and phase noise are optimized on chip with patented calibration algorithms which ensure the best performance of the receiver and the transmitter. 

### 4.3.2 Wi-Fi 

This subsection describes the chip’s Wi-Fi capabilities, which facilitate wireless communication at a high data rate. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

61 

<u>Submit Documentation Feedback</u> 

4 Functional Description 

### 4.3.2.1 Wi-Fi Radio and Baseband 

The ESP32-S3 Wi-Fi radio and baseband support the following features: 

- 802.11b/g/n 

- 802.11n MCS0-7 that supports 20 MHz and 40 MHz bandwidth 

- 802.11n MCS32 

- 802.11n 0.4 _µ_ s guard-interval 

- Data rate up to 150 Mbps 

- RX STBC (single spatial stream) 

- Adjustable transmitting power 

- Antenna diversity: 

ESP32-S3 supports antenna diversity with an external RF switch. This switch is controlled by one or more GPIOs, and used to select the best antenna to minimize the effects of channel imperfections. 

### 4.3.2.2 Wi-Fi MAC 

ESP32-S3 implements the full 802.11b/g/n Wi-Fi MAC protocol. It supports the Basic Service Set (BSS) STA and SoftAP operations under the Distributed Control Function (DCF). Power management is handled automatically with minimal host interaction to minimize the active duty period. 

The ESP32-S3 Wi-Fi MAC applies the following low-level protocol functions automatically: 

- Four virtual Wi-Fi interfaces 

- Simultaneous Infrastructure BSS Station mode, SoftAP mode, and Station + SoftAP mode 

- RTS protection, CTS protection, Immediate Block ACK 

- Fragmentation and defragmentation 

- TX/RX A-MPDU, TX/RX A-MSDU 

- TXOP 

- WMM 

- GCMP, CCMP, TKIP, WAPI, WEP, BIP, WPA2-PSK/WPA2-Enterprise, and WPA3-PSK/WPA3-Enterprise 

- Automatic beacon monitoring (hardware TSF) 

- 802.11mc FTM 

### 4.3.2.3 Networking Features 

Users are provided with libraries for TCP/IP networking, ESP-WIFI-MESH networking, and other networking protocols over Wi-Fi. TLS 1.2 support is also provided. 

### 4.3.3 Bluetooth LE 

This subsection describes the chip’s Bluetooth capabilities, which facilitate wireless communication for low-power, short-range applications. ESP32-S3 includes a Bluetooth Low Energy subsystem that integrates a 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

62 <u>Submit Documentation Feedback</u> 

4 Functional Description 

hardware link layer controller, an RF/modem block and a feature-rich software protocol stack. It supports the core features of Bluetooth 5 and Bluetooth Mesh. 

### 4.3.3.1 Bluetooth LE PHY 

Bluetooth Low Energy radio and PHY in ESP32-S3 support: 

- 1 Mbps PHY 

- 2 Mbps PHY for high transmission speed and high data throughput 

- Coded PHY for high RX sensitivity and long range (125 Kbps and 500 Kbps) 

- Class 1 transmit power without external PA 

- HW Listen Before Talk (LBT) 

### 4.3.3.2 Bluetooth LE Link Controller 

Bluetooth Low Energy Link Layer Controller in ESP32-S3 supports: 

- LE Advertising Extensions, to enhance broadcasting capacity and broadcast more intelligent data 

- Multiple Advertising Sets 

- Simultaneous Advertising and Scanning 

- Multiple connections in simultaneous central and peripheral roles 

- Adaptive Frequency Hopping (AFH) and Channel Assessment 

- LE Channel Selection Algorithm #2 

- Connection Parameter Update 

- High Duty Cycle Non-Connectable Advertising 

- LE Privacy v1.2 

- LE Data Packet Length Extension 

- Link Layer Extended Scanner Filter Policies 

- Low Duty Cycle Directed Advertising 

- Link Layer Encryption 

- LE Ping 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

63 

<u>Submit Documentation Feedback</u> 

5 Electrical Characteristics 

# 5 Electrical Characteristics 

## 5.1 Absolute Maximum Ratings 

Stresses above those listed in Table 5-1 _Absolute Maximum Ratings_ may cause permanent damage to the device. These are stress ratings only and normal operation of the device at these or any other conditions beyond those indicated in Section 5.2 _Recommended Operating Conditions_ is not implied. Exposure to absolute-maximum-rated conditions for extended periods may affect device reliability. 

Table 5-1. Absolute Maximum Ratings 

|Parameter|Description|Min|Max|Unit|
|---|---|---|---|---|
|Input power pins<sup>1</sup>|Allowed input voltage|–0.3|3.6|V|
|I_output_<sup>2</sup>|Cumulative IO output current|—|1500|mA|
|T_ST ORE_|Storage temperature|–40|150|°C|

- 1 For more information on input power pins, see Section 2.5.1 _Power Pins_ . 

- 2 The product proved to be fully functional after all its IO pins were pulled high while being connected to ground for 24 consecutive hours at ambient temperature of 25 °C. 

## 5.2 Recommended Operating Conditions 

For recommended ambient temperature, see Section 1 _ESP32-S3 Series Comparison_ . 

Table 5-2. Recommended Operating Conditions 

|Parameter <sup>1</sup>|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|VDDA, VDD3P3|Recommended input voltage|3.0|3.3|3.6|V|
|VDD3P3_RTC<sup>2</sup>|Recommended input voltage|3.0|3.3|3.6|V|
|VDD_SPI (as input)|—|1.8|3.3|3.6|V|
|VDD3P3_CPU<sup>3</sup>|Recommended input voltage|3.0|3.3|3.6|V|
|I_V DD_ <sup>4</sup>|Cumulative input current|0.5|—|—|A|

- 1 See in conjunction with Section 2.5 _Power Supply_ . 

- 2 If VDD3P3_RTC is used to power VDD_SPI (see Section 2.5.2 _Power Scheme_ ), the voltage drop on R _SP I_ should be accounted for. See also Section 5.3 _VDD_SPI Output Characteristics_ . 

- 3 If writing to eFuses, the voltage on VDD3P3_CPU should not exceed 3.3 V as the circuits responsible for burning eFuses are sensitive to higher voltages. 

- 4 If you use a single power supply, the recommended output current is 500 mA or more. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

64 <u>Submit Documentation Feedback</u> 

5 Electrical Characteristics 

## 5.3 VDD_SPI Output Characteristics 

Table 5-3. VDD_SPI Internal and Output Characteristics 

|Parameter|Description <sup>1</sup>|Typ|Unit|
|---|---|---|---|
|R_SP I_|VDD_SPI powered by VDD3P3_RTC via R_SP I_<br>for 3.3 V flash/PSRAM<sup>2</sup>|14|Ω|
|I_SP I_|Output current when VDD_SPI is powered by<br>Flash Voltage Regulator for 1.8 V flash/PSRAM|40|mA|

- 1 See in conjunction with Section 2.5.2 _Power Scheme_ . 

- 2 VDD3P3_RTC must be more than _VDD_flash_min + I_flash_max * RSP I_ ; where 

   - _VDD_flash_min_ – minimum operating voltage of flash/PSRAM 

   - _I_flash_max_ – maximum operating current of flash/PSRAM 

## 5.4 DC Characteristics (3.3 V, 25 °C) 

Table 5-4. DC Characteristics (3.3 V, 25 °C) 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|C_IN_|Pin capacitance|—|2|—|pF|
|V_IH_|High-level input voltage|0.75 × VDD<sup>1</sup>|—|VDD<sup>1 </sup>+ 0.3|V|
|V_IL_|Low-level input voltage|–0.3|—|0.25 × VDD<sup>1</sup>|V|
|I_IH_|High-level input current|—|—|50|nA|
|I_IL_|Low-level input current|—|—|50|nA|
|V_OH_ <sup>2</sup>|High-level output voltage|0.8 × VDD<sup>1</sup>|—|—|V|
|V_OL_ <sup>2</sup>|Low-level output voltage|—|—|0.1 × VDD<sup>1</sup>|V|
|I_OH_|High-level source current (VDD<sup>1 </sup>= 3.3 V,<br>V_OH_ >= 2.64 V, PAD_DRIVER = 3)|—|40|—|mA|
|I_OL_|Low-level sink current (VDD<sup>1 </sup>= 3.3 V, V_OL_ =<br>0.495 V, PAD_DRIVER = 3)|—|28|—|mA|
|R_P U_|Internal weak pull-up resistor|—|45|—|kΩ|
|R_P D_|Internal weak pull-down resistor|—|45|—|kΩ|
|V_IH_nRST_|Chip reset release voltage (CHIP_PU voltage<br>is within the specified range)|0.75 × VDD<sup>1</sup>|—|VDD<sup>1 </sup>+ 0.3|V|
|V_IL_nRST_|Chip reset voltage (CHIP_PU voltage is within<br>the specified range)|–0.3|—|0.25 × VDD<sup>1</sup>|V|

> 1 VDD – voltage from a power pin of a respective power domain. 

> 2 V _OH_ and V _OL_ are measured using high-impedance load. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

65 

<u>Submit Documentation Feedback</u> 

5 Electrical Characteristics 

## 5.5 ADC Characteristics 

The measurements in this section are taken with an external 100 nF capacitor connected to the ADC, using DC signals as input, and at an ambient temperature of 25 °C with disabled Wi-Fi. 

Table 5-5. ADC Characteristics 

|Symbol|Min|Max|Unit|
|---|---|---|---|
|DNL (Differential nonlinearity)<sup>1</sup>|–4|4|LSB|
|INL (Integral nonlinearity)|–8|8|LSB|
|Samplingrate|—|100|kSPS<sup>2</sup>|

- 1 To get better DNL results, you can sample multiple times and apply a filter, or calculate the average value. 

- 2 kSPS means kilo samples-per-second. 

The calibrated ADC results after hardware calibration and <u>software calibration</u> are shown in Table 5-6. For higher accuracy, you may implement your own calibration methods. 

Table 5-6. ADC Calibration Results 

|Parameter|Description|Min|Max|Unit|
|---|---|---|---|---|
||ATTEN0, effective measurement range of 0~850|–5|5|mV|
|Ttl|ATTEN1, effective measurement range of 0~1100|–6|6|mV|
|oa error|ATTEN2, effective measurement range of 0~1600|–10|10|mV|
||ATTEN3, effective measurement range of 0~2900|–50|50|mV|

## 5.6 Current Consumption 

### 5.6.1 Current Consumption in Active Mode 

The current consumption measurements are taken with a 3.3 V supply at 25 °C ambient temperature. 

TX current consumption is rated at a 100% duty cycle. 

RX current consumption is rated when the peripherals are disabled and the CPU idle. 

Table 5-7. Current Consumption for Wi-Fi (2.4 GHz) in Active Mode 

|Work Mode|RF Condition|Description|Peak|(mA)|
|---|---|---|---|---|
|||802.11b, 1 Mbps, @21 dBm||340|
||TX|802.11g, 54 Mbps, @19 dBm||291|
|Ati RF ki||802.11n, HT20, MCS7, @18.5 dBm||283|
|cve ( worng)||802.11n, HT40, MCS7, @18 dBm||286|
||RX|802.11b/g/n, HT20||88|
|||802.11n, HT40||91|

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

66 <u>Submit Documentation Feedback</u> 

5 Electrical Characteristics 

Table 5-8. Current Consumption for Bluetooth LE in Active Mode 

|Work Mode|RF Condition|Description|Peak (mA)|
|---|---|---|---|
|||Bluetooth LE @ 21.0 dBm|335|
||TX|Bluetooth LE @ 9.0 dBm|193|
|Active (RF working)||Bluetooth LE @ 0 dBm|176|
|||Bluetooth LE @–15.0 dBm|116|
||RX|Bluetooth LE|93|

### 5.6.2 Current Consumption in Other Modes 

The measurements below are applicable to ESP32-S3 and ESP32-S3FH8. Since ESP32-S3R2, ESP32-S3RH2, ESP32-S3R8, ESP32-S3R8V, ESP32-S3R16V, and ESP32-S3FN4R2 are embedded with PSRAM, their current consumption might be higher. 

Table 5-9. Current Consumption in Modem-sleep Mode 

|Work mode|Frequency<br>(MHz)|Description|Typ<sup>1</sup><br>(mA)|Typ<sup>2</sup><br>(mA)|
|---|---|---|---|---|
|||WAITI (Dual core in idle state)|13.2|18.8|
|||Single core running 32-bit data access instructions, the<br>other core in idle state|16.2|21.8|
||40|Dual core running32-bit data access instructions|18.7|24.4|
|||Single core running 128-bit data access instructions, the<br>other core in idle state|19.9|25.4|
|||Dual core running128-bit data access instructions|23.0|28.8|
|||WAITI|22.0|36.1|
|||Single core running 32-bit data access instructions, the<br>other core in idle state|28.4|42.6|
||80|Dual core running32-bit data access instructions|33.1|47.3|
|||Single core running 128-bit data access instructions, the<br>other core in idle state|35.1|49.6|
|Modem-slee<sup>3</sup>||Dual core running128-bit data access instructions|41.8|56.3|
|p||WAITI|27.6|42.3|
|||Single core running 32-bit data access instructions, the<br>other core in idle state|39.9|54.6|
||160|Dual core running32-bit data access instructions|49.6|64.1|
|||Single core running 128-bit data access instructions, the<br>other core in idle state|54.4|69.2|
|||Dual core running128-bit data access instructions|66.7|81.1|
|||WAITI|32.9|47.6|
|||Single core running 32-bit data access instructions, the<br>other core in idle state|51.2|65.9|
||240|Dual core running32-bit data access instructions|66.2|81.3|
|||Single core running 128-bit data access instructions, the<br>other core in idle state|72.4|87.9|

Cont’d on next page 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

67 <u>Submit Documentation Feedback</u> 

5 Electrical Characteristics 

Table 5-9 – cont’d from previous page 

|Work mode|Frequency<br>(MHz)|Description|Typ<sup>1</sup><br>(mA)|Typ<sup>2</sup><br>(mA)|
|---|---|---|---|---|
|||Dual core running128-bit data access instructions|91.7|107.9|

- 1 Current consumption when all peripheral clocks are disabled. 

- 2 Current consumption when all peripheral clocks are enabled. In practice, the current consumption might be different depending on which peripherals are enabled. 

- 3 In Modem-sleep mode, Wi-Fi is clock gated, and the current consumption might be higher when accessing flash. For a flash rated at 80 Mbit/s, in SPI 2-line mode the consumption is 10 mA. 

Table 5-10. Current Consumption in Low-Power Modes 

|Work mode|Description|Typ (_µ_A)|
|---|---|---|
|Light-sleep<sup>1</sup>|VDD_SPI and Wi-Fi are powered down, and all GPIOs are high-impedance.|240|
||The ULP co-processor<br>ULP-FSM|170|
||is powered on<sup>2</sup><br>ULP-RISC-V<br>|190|
|Deep-sleep|ULP sensor-monitored pattern<sup>3</sup>|18|
||RTC memory and RTC peripherals are powered up.|8|
||RTC memory is powered up. RTC peripherals are powered down.|7|
|Power off|CHIP_PU is set to low level. The chip is shut down.|1|

- 1 In Light-sleep mode, all related SPI pins are pulled up. For chips embedded with PSRAM, please add corresponding PSRAM consumption values, e.g., 140 _µ_ A for 8 MB 8-line PSRAM (3.3 V), 200 _µ_ A for 8 MB 8-line PSRAM (1.8 V) and 40 _µ_ A for 2 MB 4-line PSRAM (3.3 V). 

- 2 During Deep-sleep, when the ULP co-processor is powered on, peripherals such as GPIO and I2C are able to operate. 

- 3 The “ULP sensor-monitored pattern” refers to the mode where the ULP coprocessor or the sensor works periodically. When touch sensors work with a duty cycle of 1%, the typical current consumption is 18 _µ_ A. 

## 5.7 Memory Specifications 

The data below is sourced from the memory vendor datasheet. These values are guaranteed through design and/or characterization but are not fully tested in production. Devices are shipped with the memory erased. 

Table 5-11. Flash Specifications 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|VCC|Power supply voltage (1.8 V)|1.65|1.80|2.00|V|
||Power supply voltage (3.3 V)|2.7|3.3|3.6|V|
|F_C_|Maximum clock frequency|80|—|—|MHz|
|—|Program/erase cycles|100,000|—|—|cycles|
|T_RET_|Data retention time|20|—|—|years|
|T_P P_|Page program time|—|0.8|5|ms|
|T_SE_|Sector erase time (4 KB)|—|70|500|ms|

Cont’d on next page 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

68 <u>Submit Documentation Feedback</u> 

5 Electrical Characteristics 

Table 5-11 – cont’d from previous page 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|T_BE_1|Block erase time (32 KB)|—|0.2|2|s|
|T_BE_2|Block erase time (64 KB)|—|0.3|3|s|
||Chip erase time (16 Mb)|—|7|20|s|
||Chip erase time (32 Mb)|—|20|60|s|
|T_CE_|Chip erase time (64 Mb)|—|25|100|s|
||Chip erase time (128 Mb)|—|60|200|s|
||Chip erase time (256 Mb)|—|70|300|s|

Table 5-12. PSRAM Specifications 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|VCC|Power supply voltage (1.8 V)|1.62|1.80|1.98|V|
||Power supply voltage (3.3 V)|2.7|3.3|3.6|V|
|F_C_|Maximum clock frequency|80|—|—|MHz|

## 5.8 Reliability 

Table 5-13. Reliability Qualifications 

|Test Item|Test Conditions|Test Standard|
|---|---|---|
|HTOL (High Temperature<br>OperatingLife)|125 °C, 1000 hours<br>|JESD22-A108|
|ESD (Electro-Static|HBM (Human Body Mode)<sup>1 </sup>± 2000 V<br>|JS-001|
|Discharge Sensitivity)|CDM (Charge Device Mode)<sup>2 </sup>± 1000 V|JS-002|
|Latch u|Current trigger ± 200 mA|JESD78|
|p|Voltage trigger 1.5 × VDD_max_||
|Preconditioning|Bake 24 hours @125 °C<br>Moisture soak (level 3: 192 hours @30 °C, 60% RH)<br>IR reflow solder: 260 + 0 °C, 20 seconds, three times|J-STD-020, JESD47,<br>JESD22-A113|
|TCT (Temperature Cycling<br>Test)|–65 °C / 150 °C, 500 cycles|JESD22-A104|
|uHAST (Highly<br>Accelerated Stress Test,<br>unbiased)|130 °C, 85% RH, 96 hours|JESD22-A118|
|HTSL (High Temperature<br>Storage Life)|150 °C, 1000 hours|JESD22-A103|
|LTSL (Low Temperature<br>Storage Life)|–40 °C, 1000 hours|JESD22-A119|

1 JEDEC document JEP155 states that 500 V HBM allows safe manufacturing with a standard ESD control process. 2 JEDEC document JEP157 states that 250 V CDM allows safe manufacturing with a standard ESD control process. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

69 <u>Submit Documentation Feedback</u> 

6 RF Characteristics 

# 6 RF Characteristics 

This section contains tables with RF characteristics of the Espressif product. 

The RF data is measured at the antenna port, where RF cable is connected, including the front-end loss. The front-end circuit is a 0 Ω resistor. 

Devices should operate in the center frequency range allocated by regional regulatory authorities. The target center frequency range and the target transmit power are configurable by software. See <u>ESP RF Test Tool and Test Guide</u> for instructions. 

Unless otherwise stated, the RF tests are conducted with a 3.3 V (±5%) supply at 25 ºC ambient temperature. 

## 6.1 Wi-Fi Radio 

Table 6-1. Wi-Fi RF Characteristics 

|Name|Description|
|---|---|
|Center frequency range of operatingchannel|2412~2484 MHz|
|Wi-Fi wireless standard|IEEE 802.11b/g/n|

### 6.1.1 Wi-Fi RF Transmitter (TX) Characteristics 

Table 6-2. TX Power with Spectral Mask and EVM Meeting 802.11 Standards 

|Rate|Min<br>(dBm)|Typ<br>(dBm)|Max<br>(dBm)|
|---|---|---|---|
|802.11b, 1 Mbps|—|21.0|—|
|802.11b, 11 Mbps|—|21.0|—|
|802.11g, 6 Mbps|—|20.5|—|
|802.11g, 54 Mbps|—|19.0|—|
|802.11n, HT20, MCS0|—|19.5|—|
|802.11n, HT20, MCS7|—|18.5|—|
|802.11n, HT40, MCS0|—|19.5|—|
|802.11n, HT40, MCS7|—|18.0|—|

Table 6-3. TX EVM Test<sup>1</sup> 

||Min|Typ|Limit|
|---|---|---|---|
|Rate|(dB)|(dB)|(dB)|
|802.11b, 1 Mbps, @21 dBm|—|–24.5|–10|
|802.11b, 11 Mbps, @21 dBm|—|–24.5|–10|
|802.11g, 6 Mbps, @20.5 dBm|—|–21.5|–5|
|802.11g, 54 Mbps, @19 dBm|—|–28.0|–25|

Cont’d on next page 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

70 <u>Submit Documentation Feedback</u> 

6 RF Characteristics 

Table 6-3 – cont’d from previous page 

||Min|Typ|Limit|
|---|---|---|---|
|Rate|(dB)|(dB)|(dB)|
|802.11n, HT20, MCS0, @19.5 dBm|—|–23.0|–5|
|802.11n, HT20, MCS7, @18.5 dBm|—|–29.5|–27|
|802.11n, HT40, MCS0, @19.5 dBm|—|–23.0|–5|
|802.11n, HT40, MCS7, @18 dBm|—|–29.5|–27|

- 1 EVM is measured at the corresponding typical TX power provided in Table 6-2 _TX Power with Spectral Mask and EVM Meeting 802.11 Standards_ above. 

### 6.1.2 Wi-Fi RF Receiver (RX) Characteristics 

For RX tests, the PER (packet error rate) limit is 8% for 802.11b, and 10% for 802.11g/n. 

Table 6-4. RX Sensitivity 

|Rate|Min<br>(dBm)|Typ<br>(dBm)|Max<br>(dBm)|
|---|---|---|---|
|802.11b, 1 Mbps|—|–98.4|—|
|802.11b, 2 Mbps|—|–95.4|—|
|802.11b, 5.5 Mbps|—|–93.0|—|
|802.11b, 11 Mbps|—|–88.6|—|
|802.11g, 6 Mbps|—|–93.2|—|
|802.11g, 9 Mbps|—|–91.8|—|
|802.11g, 12 Mbps|—|–91.2|—|
|802.11g, 18 Mbps|—|–88.6|—|
|802.11g, 24 Mbps|—|–86.0|—|
|802.11g, 36 Mbps|—|–82.4|—|
|802.11g, 48 Mbps|—|–78.2|—|
|802.11g, 54 Mbps|—|–76.5|—|
|802.11n, HT20, MCS0|—|–92.6|—|
|802.11n, HT20, MCS1|—|–91.0|—|
|802.11n, HT20, MCS2|—|–88.2|—|
|802.11n, HT20, MCS3|—|–85.0|—|
|802.11n, HT20, MCS4|—|–81.8|—|
|802.11n, HT20, MCS5|—|–77.4|—|
|802.11n, HT20, MCS6|—|–75.8|—|
|802.11n, HT20, MCS7|—|–74.2|—|
|802.11n, HT40, MCS0|—|–90.0|—|
|802.11n, HT40, MCS1|—|–88.0|—|
|802.11n, HT40, MCS2|—|–85.2|—|
|802.11n, HT40, MCS3|—|–82.0|—|
|802.11n, HT40, MCS4|—|–79.0|—|
|802.11n, HT40, MCS5|—|–74.4|—|

Cont’d on next page 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

71 <u>Submit Documentation Feedback</u> 

6 RF Characteristics 

Table 6-4 – cont’d from previous page 

||Min|Typ|Max|
|---|---|---|---|
|Rate|(dBm)|(dBm)|(dBm)|
|802.11n, HT40, MCS6|—|–72.8|—|
|802.11n, HT40, MCS7|—|–71.4|—|

Table 6-5. Maximum RX Level 

|Rate|Min<br>(dBm)|Typ<br>(dBm)|Max<br>(dBm)|
|---|---|---|---|
|802.11b, 1 Mbps|—|5|—|
|802.11b, 11 Mbps|—|5|—|
|802.11g, 6 Mbps|—|5|—|
|802.11g, 54 Mbps|—|0|—|
|802.11n, HT20, MCS0|—|5|—|
|802.11n, HT20, MCS7|—|0|—|
|802.11n, HT40, MCS0|—|5|—|
|802.11n, HT40, MCS7|—|0|—|

Table 6-6. RX Adjacent Channel Rejection 

|Rate|Min<br>(dB)|Typ<br>(dB)|Max<br>(dB)|
|---|---|---|---|
|802.11b, 1 Mbps|—|35|—|
|802.11b, 11 Mbps|—|35|—|
|802.11g, 6 Mbps|—|31|—|
|802.11g, 54 Mbps|—|20|—|
|802.11n, HT20, MCS0|—|31|—|
|802.11n, HT20, MCS7|—|16|—|
|802.11n, HT40, MCS0|—|25|—|
|802.11n, HT40, MCS7|—|11|—|

## 6.2 Bluetooth LE Radio 

Table 6-7. Bluetooth LE Frequency 

||Min|Typ|Max|
|---|---|---|---|
|Parameter|(MHz)|(MHz)|(MHz)|
|Center frequency of operatingchannel|2402|—|2480|

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

72 <u>Submit Documentation Feedback</u> 

6 RF Characteristics 

### 6.2.1 Bluetooth LE RF Transmitter (TX) Characteristics 

Table 6-8. Transmitter Characteristics - Bluetooth LE 1 Mbps 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|RF transmit ower|RF power control range|–24.00|0|20.00|dBm|
|p|Gain control step|—|3.00|—|dB|
||Max _|fn|n_=0_,_ 1_,_ 2_, ..k_|—|2.50|—|kHz|
|Carrier freuenc offset and d|ift<br>Max _|f_0 _−fn|_|—|2.00|—|kHz|
|qy|<br>Max _|fn −fn−_5_|_|—|1.39|—|kHz|
||_|f_1 _−f_0_|_|—|0.80|—|kHz|
||∆_f_1avg|—|249.00|—|kHz|
|Modulation characteristics|Min∆_f_2max (for at least<br>99.9% of all∆_f_2max)|—|198.00|—|kHz|
||∆_f_2avg/∆_f_1avg|—|0.86|—|—|
||±2 MHz offset|—|–37.00|—|dBm|
|In-band spurious emissions|±3 MHz offset|—|–42.00|—|dBm|
||>±3 MHz offset|—|–44.00|—|dBm|

Table 6-9. Transmitter Characteristics - Bluetooth LE 2 Mbps 

|Parameter<br>Description|Min|Typ|Max|Unit|
|---|---|---|---|---|
|RF transmit ower<br>RF power control range|–24.00|0|20.00|dBm|
|p<br>Gain control step|—|3.00|—|dB|
|Max _|fn|n_=0_,_ 1_,_ 2_, ..k_|—|2.50|—|kHz|
|Ci fn fft nd dift<br>Max _|f_0 _−fn|_|—|1.90|—|kHz|
|arrer requecy ose a r<br>Max _|fn −fn−_5_|_|—|1.40|—|kHz|
|_|f_1 _−f_0_|_|—|1.10|—|kHz|
|∆_f_1avg|—|499.00|—|kHz|
|Modulation characteristics<br>Min∆_f_2max (for at least<br>99.9% of all∆_f_2max)|—|416.00|—|kHz|
|∆_f_2avg/∆_f_1avg|—|0.89|—|—|
|±4 MHz offset|—|–43.80|—|dBm|
|In-band spurious emissions<br>±5 MHz offset|—|–45.80|—|dBm|
|>±5 MHz offset|—|–47.00|—|dBm|

Table 6-10. Transmitter Characteristics - Bluetooth LE 125 Kbps 

|Parameter<br>|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|RF trnmit r<br>|RF power control range|–24.00|0|20.00|dBm|
|as powe<br>|Gain control step|—|3.00|—|dB|
||Max _|fn|n_=0_,_ 1_,_ 2_, ..k_|—|0.80|—|kHz|
|Ci f fft d dift<br>|Max _|f_0 _−fn|_|—|0.98|—|kHz|
|arrer requency ose an r<br>_|_|_fn −fn−_3_|_|—|0.30|—|kHz|
|_|_|_f_0 _−f_3_|_|—|1.00|—|kHz|

Cont’d on next page 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

73 <u>Submit Documentation Feedback</u> 

6 RF Characteristics 

Table 6-10 – cont’d from previous page 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
||∆_f_1avg|—|248.00|—|kHz|
|Modulation characteristics|Min∆_f_1max (for at least<br>99.9% of all∆_f_1max)|—|222.00|—|kHz|
||±2 MHz offset|—|–37.00|—|dBm|
|In-band spurious emissions|±3 MHz offset|—|–42.00|—|dBm|
||>±3 MHz offset|—|–44.00|—|dBm|

Table 6-11. Transmitter Characteristics - Bluetooth LE 500 Kbps 

|Parameter<br>Description|Min|Typ|Max|Unit|
|---|---|---|---|---|
|RF transmit ower<br>RF power control range|–24.00|0|20.00|dBm|
|p<br>Gain control step|—|3.00|—|dB|
|Max _|fn|n_=0_,_ 1_,_ 2_, ..k_|—|0.70|—|kHz|
|Ci f fft d dift<br>Max _|f_0 _−fn|_|—|0.90|—|kHz|
|arrer requency ose an r<br>_|fn −fn−_3_|_|—|0.85|—|kHz|
|_|f_0 _−f_3_|_|—|0.34|—|kHz|
|∆_f_2avg|—|213.00|—|kHz|
|Modulation characteristics<br>Min∆_f_2max (for at least<br>99.9% of all∆_f_2max)|—|196.00|—|kHz|
|±2 MHz offset|—|–37.00|—|dBm|
|In-band spurious emissions<br>±3 MHz offset|—|–42.00|—|dBm|
|>±3 MHz offset|—|–44.00|—|dBm|

### 6.2.2 Bluetooth LE RF Receiver (RX) Characteristics 

Table 6-12. Receiver Characteristics - Bluetooth LE 1 Mbps 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|Sensitivity @30.8% PER|—|—|–97.5|—|dBm|
|Maximum received signal @30.8% PER|—|—|8|—|dBm|
|Co-channel C/I|F = F0 MHz|—|9|—|dB|
||F = F0 + 1 MHz|—|–3|—|dB|
||F = F0 – 1 MHz|—|–3|—|dB|
||F = F0 + 2 MHz|—|–28|—|dB|
|Adnt hnnl ltiit C/I|F = F0 – 2 MHz|—|–30|—|dB|
|jace cae seecvy|F = F0 + 3 MHz|—|–31|—|dB|
||F = F0 – 3 MHz|—|–33|—|dB|
||F_>_F0 + 3 MHz|—|–32|—|dB|
||F_>_F0 – 3 MHz|—|–36|—|dB|
|Image frequency|—|—|–32|—|dB|
|Adnt hnnl t im frn|F = F_image_ + 1 MHz|—|–39|—|dB|
|jace cae o age equecy|F = F_image_ – 1 MHz|—|–31|—|dB|

Cont’d on next page 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

74 <u>Submit Documentation Feedback</u> 

6 RF Characteristics 

Table 6-12 – cont’d from previous page 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
||30 MHz~2000 MHz|—|–9|—|dBm|
|lk|2003 MHz~2399 MHz|—|–19|—|dBm|
|Out-of-band bocing performance|2484 MHz~2997 MHz|—|–16|—|dBm|
||3000 MHz~12.75 GHz|—|–5|—|dBm|
|Intermodulation|—|—|–31|—|dBm|

Table 6-13. Receiver Characteristics - Bluetooth LE 2 Mbps 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|Sensitivity @30.8% PER|—|—|–93.5|—|dBm|
|Maximum received signal @30.8% PER|—|—|3|—|dBm|
|Co-channel C/I|F = F0 MHz|—|10|—|dB|
||F = F0 + 2 MHz|—|–8|—|dB|
||F = F0 – 2 MHz|—|–5|—|dB|
||F = F0 + 4 MHz|—|–31|—|dB|
|Adt hl ltiit C/I|F = F0 – 4 MHz|—|–33|—|dB|
|jacen canne seecvy|F = F0 + 6 MHz|—|–37|—|dB|
||F = F0 – 6 MHz|—|–37|—|dB|
||F_>_F0 + 6 MHz|—|–40|—|dB|
||F_>_F0 – 6 MHz|—|–40|—|dB|
|Image frequency|—|—|–31|—|dB|
||F = F_image_ + 2 MHz|—|–37|—|dB|
|Adjacent channel to image frequency|F = F_image_ – 2 MHz|—|–8|—|dB|
||30 MHz~2000 MHz|—|–16|—|dBm|
||2003 MHz~2399 MHz|—|–20|—|dBm|
|Out-of-band blocking performance|2484 MHz~2997 MHz|—|–16|—|dBm|
||3000 MHz~12.75 GHz|—|–16|—|dBm|
|Intermodulation|—|—|–30|—|dBm|

Table 6-14. Receiver Characteristics - Bluetooth LE 125 Kbps 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|Sensitivity @30.8% PER|—|—|–104.5|—|dBm|
|Maximum received signal @30.8% PER|—|—|8|—|dBm|
|Co-channel C/I|F = F0 MHz|—|6|—|dB|
||F = F0 + 1 MHz|—|–6|—|dB|
||F = F0 – 1 MHz|—|–5|—|dB|
||F = F0 + 2 MHz|—|–32|—|dB|
|Adt hl ltiit C/I|F = F0 – 2 MHz|—|–39|—|dB|
|jacen canne seecvy|F = F0 + 3 MHz|—|–35|—|dB|
||F = F0 – 3 MHz|—|–45|—|dB|
||F_>_F0 + 3 MHz|—|–35|—|dB|

Cont’d on next page 

75 ESP32-S3 Series Datasheet v2.2 <u>Submit Documentation Feedback</u> 

Espressif Systems 

6 RF Characteristics 

Table 6-14 – cont’d from previous page 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
||F_>_F0 – 3 MHz|—|–48|—|dB|
|Image frequency|—|—|–35|—|dB|
|Ad hl  i f|F = F_image_ + 1 MHz|—|–49|—|dB|
|jacent canne to mage requency|F = F_image_ – 1 MHz|—|–32|—|dB|

Table 6-15. Receiver Characteristics - Bluetooth LE 500 Kbps 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|Sensitivity @30.8% PER|—|—|–101|—|dBm|
|Maximum received signal @30.8% PER|—|—|8|—|dBm|
|Co-channel C/I|F = F0 MHz|—|4|—|dB|
||F = F0 + 1 MHz|—|–5|—|dB|
||F = F0 – 1 MHz|—|–5|—|dB|
||F = F0 + 2 MHz|—|–28|—|dB|
|Adt hl ltiit C/I|F = F0 – 2 MHz|—|–36|—|dB|
|jacen canne seecvy|F = F0 + 3 MHz|—|–36|—|dB|
||F = F0 – 3 MHz|—|–38|—|dB|
||F_>_F0 + 3 MHz|—|–37|—|dB|
||F_>_F0 – 3 MHz|—|–41|—|dB|
|Image frequency|—|—|–37|—|dB|
|Adt hl t i f|F = F_image_ + 1 MHz|—|–44|—|dB|
|jacen canne o mage requency|F = F_image_ – 1 MHz|—|–28|—|dB|

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

76 <u>Submit Documentation Feedback</u> 

7 Packaging 

# 7 Packaging 

- For information about tape, reel, and product marking, please refer to _<u>ESP32-S3 Chip Packaging Information</u>_ <u>.</u> 

- The pins of the chip are numbered in anti-clockwise order starting from Pin 1 in the top view. For pin numbers and pin names, see also Figure 2-1 _ESP32-S3 Pin Layout (Top View)_ . 

- The recommended land pattern <u>source flei (asc)</u> is available for download. You can import the file with software such as PADS and Altium Designer. 

- All ESP32-S3 chip variants have identical land pattern (see Figure 7-1) except ESP32-S3FH4R2 has a bigger EPAD (see Figure 7-2). The <u>source flei (asc)</u> may be adopted for ESP32-S3FH4R2 by altering the size of the EPAD (see dimensions D2 and E2 in Figure 7-2). 

Figure 7-1. QFN56 (7×7 mm) Package 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

77 <u>Submit Documentation Feedback</u> 

7 Packaging 

Figure 7-2. QFN56 (7×7 mm) Package (Only for ESP32-S3FH4R2) 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

78 <u>Submit Documentation Feedback</u> 

# ESP32-S3 Consolidated Pin Overview 

#### Table 7-1. Consolidated Pin Overview 

|Pin No.<br>Pin Name|Pin Type|Pin ProvidingPower|Pin Se<br>At Reset|ttings<br>After Reset|RTC IO M<br>F0|UX Function<br>F3|AnalogFu<br>F0|nction<br>F1|F0|Type<br>F1<br>Type|IO MUX Function<br>F2<br>Type|F3<br>Type|F4<br>Type|
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
|1<br>LNA_IN|Analog|||||||||||||
|2<br>VDD3P3|Power|||||||||||||
|3<br>VDD3P3|Power|||||||||||||
|4<br>CHIP_PU|Analog|VDD3P3_RTC||||||||||||
|5<br>GPIO0|IO|VDD3P3_RTC|WPU, IE|WPU, IE|RTC_GPIO0|sar_i2c_scl_0|||GPIO0|I/O/T<br>GPIO0<br>I/O/T||||
|6<br>GPIO1|IO|VDD3P3_RTC|IE|IE|RTC_GPIO1|sar_i2c_sda_0|TOUCH1|ADC1_CH0|GPIO1|I/O/T<br>GPIO1<br>I/O/T||||
|7<br>GPIO2|IO|VDD3P3_RTC|IE|IE|RTC_GPIO2|sar_i2c_scl_1|TOUCH2|ADC1_CH1|GPIO2|I/O/T<br>GPIO2<br>I/O/T||||
|8<br>GPIO3|IO|VDD3P3_RTC|IE|IE|RTC_GPIO3|sar_i2c_sda_1|TOUCH3|ADC1_CH2|GPIO3|I/O/T<br>GPIO3<br>I/O/T||||
|9<br>GPIO4|IO|VDD3P3_RTC|||RTC_GPIO4||TOUCH4|ADC1_CH3|GPIO4|I/O/T<br>GPIO4<br>I/O/T||||
|10<br>GPIO5|IO|VDD3P3_RTC|||RTC_GPIO5||TOUCH5|ADC1_CH4|GPIO5|I/O/T<br>GPIO5<br>I/O/T||||
|11<br>GPIO6|IO|VDD3P3_RTC|||RTC_GPIO6||TOUCH6|ADC1_CH5|GPIO6|I/O/T<br>GPIO6<br>I/O/T||||
|12<br>GPIO7|IO|VDD3P3_RTC|||RTC_GPIO7||TOUCH7|ADC1_CH6|GPIO7|I/O/T<br>GPIO7<br>I/O/T||||
|13<br>GPIO8|IO|VDD3P3_RTC|||RTC_GPIO8||TOUCH8|ADC1_CH7|GPIO8|I/O/T<br>GPIO8<br>I/O/T||SUBSPICS1<br>O/T||
|14<br>GPIO9|IO|VDD3P3_RTC||IE|RTC_GPIO9||TOUCH9|ADC1_CH8|GPIO9|I/O/T<br>GPIO9<br>I/O/T||SUBSPIHD<br>I1/O/T|FSPIHD<br>I1/O/T|
|15<br>GPIO10|IO|VDD3P3_RTC||IE|RTC_GPIO10||TOUCH10|ADC1_CH9|GPIO10|I/O/T<br>GPIO10<br>I/O/T|FSPIIO4<br>I1/O/T|SUBSPICS0<br>O/T|FSPICS0<br>I1/O/T|
|16<br>GPIO11|IO|VDD3P3_RTC||IE|RTC_GPIO11||TOUCH11|ADC2_CH0|GPIO11|I/O/T<br>GPIO11<br>I/O/T|FSPIIO5<br>I1/O/T|SUBSPID<br>I1/O/T|FSPID<br>I1/O/T|
|17<br>GPIO12|IO|VDD3P3_RTC||IE|RTC_GPIO12||TOUCH12|ADC2_CH1|GPIO12|I/O/T<br>GPIO12<br>I/O/T|FSPIIO6<br>I1/O/T|SUBSPICLK<br>O/T|FSPICLK<br>I1/O/T|
|18<br>GPIO13|IO|VDD3P3_RTC||IE|RTC_GPIO13||TOUCH13|ADC2_CH2|GPIO13|I/O/T<br>GPIO13<br>I/O/T|FSPIIO7<br>I1/O/T|SUBSPIQ<br>I1/O/T|FSPIQ<br>I1/O/T|
|19<br>GPIO14|IO|VDD3P3_RTC||IE|RTC_GPIO14||TOUCH14|ADC2_CH3|GPIO14|I/O/T<br>GPIO14<br>I/O/T|FSPIDQS<br>O/T|SUBSPIWP<br>I1/O/T|FSPIWP<br>I1/O/T|
|20<br>VDD3P3_RTC|Power|||||||||||||
|21<br>XTAL_32K_P|IO|VDD3P3_RTC|||RTC_GPIO15||XTAL_32K_P|ADC2_CH4|GPIO15|I/O/T<br>GPIO15<br>I/O/T|U0RTS<br>O|||
|22<br>XTAL_32K_N|IO|VDD3P3_RTC|||RTC_GPIO16||XTAL_32K_N|ADC2_CH5|GPIO16|I/O/T<br>GPIO16<br>I/O/T|U0CTS<br>I1|||
|23<br>GPIO17|IO|VDD3P3_RTC||IE|RTC_GPIO17|||ADC2_CH6|GPIO17|I/O/T<br>GPIO17<br>I/O/T|U1TXD<br>O|||
|24<br>GPIO18|IO|VDD3P3_RTC||IE|RTC_GPIO18|||ADC2_CH7|GPIO18|I/O/T<br>GPIO18<br>I/O/T|U1RXD<br>I1|CLK_OUT3<br>O||
|25<br>GPIO19|IO|VDD3P3_RTC|||RTC_GPIO19||USB_D-|ADC2_CH8|GPIO19|I/O/T<br>GPIO19<br>I/O/T|U1RTS<br>O|CLK_OUT2<br>O||
|26<br>GPIO20|IO|VDD3P3_RTC|USB_PU|USB_PU|RTC_GPIO20||USB_D+|ADC2_CH9|GPIO20|I/O/T<br>GPIO20<br>I/O/T|U1CTS<br>I1|CLK_OUT1<br>O||
|27<br>GPIO21|IO|VDD3P3_RTC|||RTC_GPIO21||||GPIO21|I/O/T<br>GPIO21<br>I/O/T||||
|28<br>SPICS1<br><br>|IO<br>|VDD_SPI|WPU, IE|WPU, IE|||||SPICS1|O/T<br>GPIO26<br>I/O/T||||
|29<br>VDD_SPI|Power|||||||||||||
|30<br>SPIHD|IO|VDD_SPI|WPU, IE|WPU, IE|||||SPIHD|I1/O/T<br>GPIO27<br>I/O/T||||
|31<br>SPIWP<br><br>|IO<br>|VDD_SPI<br>|WPU, IE<br>|WPU, IE<br>|||||SPIWP<br>|I1/O/T<br>GPIO28<br>I/O/T<br><br><br>||||
|32<br>SPICS0|IO|VDD_SPI|WPU, IE|WPU, IE|||||SPICS0|O/T<br>GPIO29<br>I/O/T||||
|33<br>SPICLK|IO|VDD_SPI|WPU, IE|WPU, IE|||||SPICLK|O/T<br>GPIO30<br>I/O/T||||
|34<br>SPIQ|IO|VDDSPI|WPU IE|WPU IE|||||SPIQ|I1/O/T<br>GPIO31<br>I/O/T||||
|35<br>SPID|IO|_<br>VDD_SPI|,<br>WPU, IE|,<br>WPU, IE|||||SPID|I1/O/T<br>GPIO32<br>I/O/T||||
|36<br>SPICLK_N|IO|VDD_SPI/VDD3P3_CPU|IE|IE|||||SPICLK_P_DI|FF<br>O/T<br>GPIO48<br>I/O/T|SUBSPICLK_P_DIFF<br>O/T|||
|37<br>SPICLKP|IO|VDDSPI/VDD3P3CPU|IE|IE|||||SPICLKNDI|FF<br>O/T<br>GPIO47<br>I/O/T|SUBSPICLKNDIFF<br>O/T|||
|_<br>38<br>GPIO33|IO|__<br>VDD_SPI/VDD3P3_CPU||IE|||||__<br>GPIO33|I/O/T<br>GPIO33<br>I/O/T|__<br><br>FSPIHD<br>I1/O/T|SUBSPIHD<br>I1/O/T|SPIIO4<br>I1/O/T|
|39<br>GPIO34|IO|VDD_SPI/VDD3P3_CPU||IE|||||GPIO34|I/O/T<br>GPIO34<br>I/O/T|FSPICS0<br>I1/O/T|SUBSPICS0<br>O/T|SPIIO5<br>I1/O/T|
|40<br>GPIO35|IO|VDD_SPI/VDD3P3_CPU||IE|||||GPIO35|I/O/T<br>GPIO35<br>I/O/T|FSPID<br>I1/O/T|SUBSPID<br>I1/O/T|SPIIO6<br>I1/O/T|
|41<br>GPIO36|IO|VDDSPI/VDD3P3CPU||IE|||||GPIO36|I/O/T<br>GPIO36<br>I/O/T|FSPICLK<br>I1/O/T|SUBSPICLK<br>O/T|SPIIO7<br>I1/O/T|
|42<br>GPIO37|IO|__<br>VDD_SPI/VDD3P3_CPU||IE|||||GPIO37|I/O/T<br>GPIO37<br>I/O/T|FSPIQ<br>I1/O/T|SUBSPIQ<br>I1/O/T|SPIDQS<br>I0/O/T|
|43<br>GPIO38|IO|VDD3P3_CPU||IE|||||GPIO38|I/O/T<br>GPIO38<br>I/O/T|FSPIWP<br>I1/O/T|SUBSPIWP<br>I1/O/T||
|44<br>MTCK<br><br>|IO<br>|VDD3P3_CPU<br>||IE<br>|||||MTCK<br>|I1<br>GPIO39<br>I/O/T<br><br><br>|CLK_OUT3<br>O<br><br>|SUBSPICS1<br>O/T||
|45<br>MTDO|IO|VDD3P3_CPU||IE|||||MTDO|O/T<br>GPIO40<br>I/O/T|CLK_OUT2<br>O|||
|46<br>VDD3P3_CPU|Power|||||||||||||
|47<br>MTDI|IO|VDD3P3CPU||IE|||||MTDI|I1<br>GPIO41<br>I/O/T|CLKOUT1<br>O|||
|48<br>MTMS|IO|_<br>VDD3P3CPU||IE|||||MTMS|I1<br>GPIO42<br>I/O/T|_<br>|||
|49<br>U0TXD|IO|_<br>VDD3P3_CPU|WPU, IE|WPU, IE|||||U0TXD|O<br>GPIO43<br>I/O/T|CLK_OUT1<br>O|||
|50<br>U0RXD|IO|VDD3P3CPU|WPU IE|WPU IE|||||U0RXD|I1<br>GPIO44<br>I/O/T|CLKOUT2<br>O|||
|51<br>GPIO45|IO|_<br>VDD3P3CPU|,<br>WPD, IE|,<br>WPD, IE|||||GPIO45|I/O/T<br>GPIO45<br>I/O/T|_<br>|||
|52<br>GPIO46|IO|_<br>VDD3P3_CPU|<br>WPD, IE|<br>WPD, IE|||||GPIO46|I/O/T<br>GPIO46<br>I/O/T||||
|53<br>XTAL_N|Analog|||||||||||||
|54<br>XTAL_P<br><br>|Analog<br>|||||||||||||
|55<br>VDDA<br><br>|Power<br>|||||||||||||
|56<br>VDDA<br>57<br>GND|Power<br>Power|||||||||||||

* For details, see Section 2 _Pins_ . Regarding highlighted cells, see Section 2.3.4 _Restrictions for GPIOs and RTC_GPIOs_ . 

_Datasheet Versioning_ 

# Datasheet Versioning 

|Datasheet<br>Version|Status|Watermark|Definition|
|---|---|---|---|
|v0.1 ~ v0.5<br>(excluding v0.5)|Draft|Confidential|This datasheet is under development for products<br>in the design stage. Specifications may change<br>without prior notice.|
|v0.5 ~ v1.0<br>(excluding v1.0)|Preliminary<br>release|Preliminary|This datasheet is actively updated for products in<br>the verification stage. Specifications may change<br>before mass production, and the changes will be<br>documentation in the datasheet’s Revision History.|
|v1.0 and higher|Official release|—|This datasheet is publicly released for products in<br>mass production. Specifications are finalized, and<br>major changes will be communicated via<br>Product<br>Change<br>Notifications<br>(PCN).|
|Any version|—|Not<br>Recommended<br>for New Design<br>(NRND)<sup>1</sup>|This datasheet is updated less frequently for<br>products not recommended for new designs.|
|Any version|—|End of Life<br>(EOL)<sup>2</sup>|This datasheet is no longer mtained for products<br>that have reached end of life.|

> 1 Watermark will be added to the datasheet title page only when all the product variants covered by this datasheet are not recommended for new designs. 

> 2 Watermark will be added to the datasheet title page only when all the product variants covered by this datasheet have reached end of life. 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

80 <u>Submit Documentation Feedback</u> 

_Glossary_ 

# Glossary 

#### module 

A self-contained unit integrated within the chip to extend its capabilities, such as cryptographic modules, RF modules 2 

#### peripheral 

A hardware component or subsystem within the chip to interface with the outside world 2 

#### in-package flash 

Flash integrated directly into the chip’s package, and external to the chip die 4, 13 

#### off-package flash 

Flash external to the chip’s package 20 

#### strapping pin 

A type of GPIO pin used to configure certain operational settings during the chip’s power-up, and can be reconfigured as normal GPIO after the chip’s reset 32 

#### eFuse parameter 

A parameter stored in an electrically programmable fuse (eFuse) memory within a chip. The parameter can be set by programming EFUSE_PGM_DATA _n_ _REG registers, and read by reading a register field named after the parameter 32 

#### SPI boot mode 

A boot mode in which users load and execute the existing code from SPI flash 33 

#### joint download boot mode 

A boot mode in which users can download code into flash via the UART or other interfaces (see Table 3-3 _Chip Boot Mode Control_ > Note), and load and execute the downloaded code from the flash or SRAM 33 

#### eFuse 

A one-time programmable (OTP) memory which stores system and user parameters, such as MAC address, chip revision number, flash encryption key, etc. Value 0 indicates the default state, and value 1 indicates the eFuse has been programmed 39 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

81 

<u>Submit Documentation Feedback</u> 

_Related Documentation and Resources_ 

# Related Documentation and Resources 

## Related Documentation 

- _<u>ESP32-S3 Technical Reference Manual</u>_ – Detailed information on how to use the ESP32-S3 memory and peripherals. 

- _<u>ESP32-S3 Hardware Design Guidelines</u>_ – Guidelines on how to integrate the ESP32-S3 into your hardware product. 

- _<u>ESP32-S3 Series SoC Errata</u>_ – Descriptions of known errors in ESP32-S3 series of SoCs. 

- _Certificates_ 

<u>https://espressif.com/en/support/documents/certifcatesi</u> 

- _ESP32-S3 Product/Process Change Notifications (PCN)_ <u>https://espressif.com/en/support/documents/pcns?keys=ESP32-S3</u> 

- _ESP32-S3 Advisories_ – Information on security, bugs, compatibility, component reliability. <u>https://espressif.com/en/support/documents/advisories?keys=ESP32-S3</u> 

- _Documentation Updates and Update Notification Subscription_ 

<u>https://espressif.com/en/support/download/documents</u> 

## Developer Zone 

- <u>ESP-IDF Programming Guide for ESP32-S3</u> – Extensive documentation for the ESP-IDF development framework. 

- _ESP-IDF_ and other development frameworks on GitHub. 

<u>https://github.com/espressif</u> 

- _ESP32 BBS Forum_ – Engineer-to-Engineer (E2E) Community for Espressif products where you can post questions, share knowledge, explore ideas, and help solve problems with fellow engineers. 

- <u>https://esp32.com/</u> 

- _ESP-FAQ_ – A summary document of frequently asked questions released by Espressif. <u>https://espressif.com/projects/esp-faq/en/latest/index.html</u> 

- _The ESP Journal_ – Best Practices, Articles, and Notes from Espressif folks. <u>https://blog.espressif.com/</u> 

- See the tabs _SDKs and Demos_ , _Apps_ , _Tools_ , _AT Firmware_ . <u>https://espressif.com/en/support/download/sdks-demos</u> 

## Products 

- _ESP32-S3 Series SoCs_ – Browse through all ESP32-S3 SoCs. <u>https://espressif.com/en/products/socs?id=ESP32-S3</u> 

- _• ESP32-S3 Series Modules_ – Browse through all ESP32-S3-based modules. <u>https://espressif.com/en/products/modules?id=ESP32-S3</u> 

- _ESP32-S3 Series DevKits_ – Browse through all ESP32-S3-based devkits. <u>https://espressif.com/en/products/devkits?id=ESP32-S3</u> 

- _ESP Product Selector_ – Find an Espressif hardware product suitable for your needs by comparing or applying filters. <u>https://products.espressif.com/#/product-selector?language=en</u> 

## Contact Us 

- See the tabs _Sales Questions_ , _Technical Enquiries_ , _Circuit Schematic & PCB Design Review_ , _Get Samples_ (Online stores), _Become Our Supplier_ , _Comments & Suggestions_ . <u>https://espressif.com/en/contact-us/sales-questions</u> 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

82 

<u>Submit Documentation Feedback</u> 

_Revision History_ 

# Revision History 

|Date<br>2026-03-05|Version<br>v2.2|Release notes<br>_•_ Renamed the Digital Signature module to “RSA Digital Signature Peripheral<br>(RSA_DS)”<br>_•_ Updated Figure4-1_Address Mapping Structure_<br>_•_ Added a note in Section4.2.1.9_SD/MMC Host Controller_<br>_•_ Updated table5-10_Current Consumption in Low-Power Modes_|
|---|---|---|
|2025-11-28|v2.1|_•_ Updated the status of ESP32-S3R2 to End of Life and added chip variant<br>ESP32-S3RH2<br>_•_ Updated “Ordering Code” to “Part Number” in Table1-1_ESP32-S3 Series_<br>_Comparison_<br>_•_ Added Section1.3_Chip Revision_and chip version information in Table1-1<br>_ESP32-S3 Series Comparison_<br>_•_ Added Section2.3.5_Peripheral Pin Assignment_and updated the_Pin As-_<br>_signment_part for each subsection in Section4.2_Peripherals_<br>_•_ Updated Figure 3-1 _Visualization of Timing Parameters for the Strapping_<br>_Pins_<br>_•_ Added Section5.7_Memory Specifications_<br>_•_ Added Table 5-8 _Current Consumption for Bluetooth LE in Active Mode_<br>in Section5.6_Current Consumption_<br>_•_ Added Appendix_Datasheet Status Definitions_and_Glossary_<br>_•_ Other structural, formatting, and content improvements|
|2025-04-24|v2.0|_•_ Updated the status of ESP32-S3R8V to End of Life<br>_•_ Updated the CoreMark<sup>® </sup>score in Section_CPU and Memory_<br>_•_ Updated Figure 4.1.2 _Memory Organization_ in Section 4-1 _Address Map-_<br>_ping Structure_<br>_•_ Updated the temperature sensor’s measurement range in Section4.2.2.2<br>_Temperature Sensor_<br>_•_ Added some notes in Chapter6_RF Characteristics_<br>_•_ Updated the source file link for the recommended land pattern in Chapter<br>7_Packaging_|

Cont’d on next page 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

83 <u>Submit Documentation Feedback</u> 

_Revision History_ 

#### Cont’d from previous page 

|Date|Version|Release notes|
|---|---|---|
|2024-09-11|v1.9|_•_ Updated descriptions on the title page<br>_•_ Updated feature descriptions in Section_Features_and adjusted the format<br>_•_ Updated the pin introduction in Section 2.2 _Pin Overview_ and adjusted<br>the format<br>_•_ Updated descriptions in Section2.3_IO Pins_, and divided Section_RTC and_<br>_Analog Pin Functions_ into Section 2.3.3 _Analog Functions_ and Section<br>2.3.2_RTC Functions_<br>_•_ Updated Section_Strapping Pins_to Section3_Boot Configurations_<br>_•_ Adjusted the structure and section order in Section4_Functional Descrip-_<br>_tion_, deleted Section _Peripheral Pin Configurations_, and added the _Pin_<br>_Assignment_part in each subsection in Section4.2_Peripherals_|
|2023-11-24|v1.8|_•_ Added chip variant ESP32-S3R16V and updated related information<br>_•_ Added the second and third table notes in Table 1-1 _ESP32-S3 Series_<br>_Comparison_<br>_•_ Updated Section3.1_Chip Boot Mode Control_<br>_•_ Updated Section5.5_ADC Characteristics_<br>_•_ Other minor updates|
|2023-06|v1.7|_•_ Removed the sample status for ESP32-S3FH4R2<br>_•_ Updated Figure_ESP32-S3 Functional Block Diagram_and Figure4-2_Com-_<br>_ponents and Power Domains_<br>_•_ Added the predefined settings at reset and after reset for GPIO20 in Table<br>2-1_Pin Overview_<br>_•_ Updated notes for Table2-4_IO MUX Functions_<br>_•_ Updated the clock name “FOSC_CLK” to “RC_FAST_CLK” in Section<br>4.1.3.5_Power Management Unit (PMU)_<br>_•_ Updated descriptions in Section 4.2.1.5 _Serial Peripheral Interface (SPI)_<br>and Section4.1.4.3_RSA Accelerator_<br>_•_ Other minor updates|

Cont’d on next page 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

84 <u>Submit Documentation Feedback</u> 

_Revision History_ 

#### Cont’d from previous page 

|Date|Version|Release notes|
|---|---|---|
|2023-02|v1.6|_•_ Improved the content in the following sections:<br>– Section_Product Overview_<br>– Section2_Pins_<br>– Section4.1.3.5_Power Management Unit (PMU)_<br>– Section4.2.1.5_Serial Peripheral Interface (SPI)_<br>– Section5.1_Absolute Maximum Ratings_<br>– Section5.2_Recommended Operating Conditions_<br>– Section5.3_VDD_SPI Output Characteristics_<br>– Section5.5_ADC Characteristics_<br>_•_ Added_ESP32-S3 Consolidated Pin Overview_<br>_•_ Updated the notes in Section1_ESP32-S3 Series Comparison_and Section<br>7_Packaging_<br>_•_ Updated the effective measurement range in Table5-5_ADC Characteris-_<br>_tics_<br>_•_ Updated the Bluetooth maximum transmit power<br>_•_ Other minor updates|
|2022-12|v1.5|_•_ Removed the “External PA is supported” feature from Section_Features_<br>_•_ Updated the ambient temperature for ESP32-S3FH4R2 from –40 _∼_105<br>°C to–40_∼_85 °C<br>_•_ Added two notes in Section7|
|2022-11|v1.4|_•_ Added the package information for ESP32-S3FH4R2 in Section7<br>_•_ Added<br>ESP32-S3<br>Series<br>SoC<br>Errata in Section<br>_•_ Other minor updates|
|2022-09|v1.3|_•_ Added a note about the maximum ambient temperature of R8 series chips<br>to Table1-1and Table5-2<br>_•_ Added information about power-up glitches for some pins in Section2.2<br>_•_ Added the information about VDD3P3 power pins to Table2.2and Sec-<br>tion2.5.2<br>_•_ Updated section4.3.3.1<br>_•_ Added the fourth note in Table2-1<br>_•_ Updated the minimum and maximum values of Bluetooth LE RF transmit<br>power in Section6.2.1<br>_•_ Other minor updates|
|2022-07|v1.2|_•_ Updated description of ROM code printing in Section3<br>_•_ Updated Figure_ESP32-S3 Functional Block Diagram_<br>_•_ Update Section5.6<br>_•_ Deleted the hyperlinks in_Application_|

Cont’d on next page 

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

85 <u>Submit Documentation Feedback</u> 

_Revision History_ 

#### Cont’d from previous page 

|Date|Version|Release notes|
|---|---|---|
|2022-04|v1.1|_•_ Synchronized eFuse size throughout<br>_•_ Updated pin description in Table2-1<br>_•_ Updated SPI resistance in Table5-3<br>_•_ Added information about chip ESP32-S3FH4R2|
|2022-01|v1.0|_•_ Added wake-up sources for Deep-sleep mode<br>_•_ Added Table3-4for default configurations of VDD_SPI<br>_•_ Added ADC calibration results in Table5-5<br>_•_ Added typical values when all peripherals and peripheral clocks are en-<br>abled to Table5-9<br>_•_ Added more descriptions of modules/peripherals in Section4<br>_•_ Updated Figure_ESP32-S3 Functional Block Diagram_<br>_•_ Updated JEDEC specification<br>_•_ Updated Wi-Fi RF data in Section5.6<br>_•_ Updated temperature for ESP32-S3R8 and ESP32-S3R8V<br>_•_ Updated description of Deep-sleep mode in Table5-10<br>_•_ Updated wording throughout|
|2021-10-12|v0.6.1|Updated text description|
|2021-09-30|v0.6|_•_ Updated to chip revision 1 by swapping pin 53 and pin 54 (XTAL_P and<br>XTAL_N)<br>_•_ Updated Figure_ESP32-S3 Functional Block Diagram_<br>_•_ Added CoreMark score in section Features<br>_•_ Updated Section3<br>_•_ Added data for cumulative IO output current in Table5-1<br>_•_ Added data for Modem-sleep current consumption in Table5-9<br>_•_ Updated data in section5.6,6.1, and6.2<br>_•_ Updated wording throughout|
|2021-07-19|v0.5.1|_•_ Added “for chip revision 0” on cover, in footer and watermark to indicate<br>that the current and previous versions of this datasheet are for chip ver-<br>sion 0<br>_•_ Corrected a few typos|
|2021-07-09|v0.5|Preliminary version|

ESP32-S3 Series Datasheet v2.2 

Espressif Systems 

86 <u>Submit Documentation Feedback</u> 

### Disclaimer and Copyright Notice 

Information in this document, including URL references, is subject to change without notice. 

ALL THIRD PARTY’S INFORMATION IN THIS DOCUMENT IS PROVIDED AS IS WITH NO WARRANTIES TO ITS AUTHENTICITY AND ACCURACY. 

NO WARRANTY IS PROVIDED TO THIS DOCUMENT FOR ITS MERCHANTABILITY, NON-INFRINGEMENT, FITNESS FOR ANY PARTICULAR PURPOSE, NOR DOES ANY WARRANTY OTHERWISE ARISING OUT OF ANY PROPOSAL, SPECIFICATION OR SAMPLE. All liability, including liability for infringement of any proprietary rights, relating to use of information in this document is disclaimed. No licenses express or implied, by estoppel or otherwise, to any intellectual property rights are granted herein. The Wi-Fi Alliance Member logo is a trademark of the Wi-Fi Alliance. The Bluetooth logo is a registered trademark of Bluetooth SIG. All trade names, trademarks and registered trademarks mentioned in this document are property of their respective owners, and are hereby acknowledged. 

Copyright © 2026 Espressif Systems (Shanghai) Co., Ltd. All rights reserved. <u>w.espressif.comww</u> 

