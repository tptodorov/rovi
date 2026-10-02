# ESP32-S3-WROOM-1 / -1U module datasheet (v1.8)

> Markdown transcription of [`module-datasheet.pdf`](module-datasheet.pdf) (source: espressif.com/sites/default/files/documentation/esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf). Machine-converted; figures/diagrams dropped and complex tables may be imperfect. **The PDF is authoritative.** **Level: module — not the dev/breakout board in hand; see this folder's README for board-level docs (pins, ports differ).**

# ESP32-S3-WROOM-1 ESP32-S3-WROOM-1U Datasheet Version 1.8 

- 2.4 GHz Wi-Fi (802.11b/g/n) and Bluetooth<sup>®</sup> 5 module 

- Built around ESP32-S3 series of SoCs, Xtensa<sup>®</sup> dual-core 32-bit LX7 microprocessor Flash up to 16 MB, PSRAM up to 16 MB 

Up to 36 GPIOs, rich set of peripherals 

On-board PCB antenna or external antenna connector 

ESP32-S3-WROOM-1 

ESP32-S3-WROOM-1U 

www.espressif.com 

1 Module Overview 

## 1 Module Overview 

Note: 

Check the link or the QR code to make sure that you use the latest version of this document: https://www.espressif.com/documentation/esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf 

### 1.1 Features 

#### CPU and On-Chip Memory 

- ESP32-S3 series of SoCs embedded, Xtensa<sup>®</sup> dual-core 32-bit LX7 microprocessor (with single precision FPU), up to 240 MHz 

- 384 KB ROM 

- 512 KB SRAM 

      - 4 strapping GPIOs 

   - SPI, LCD interface, Camera interface, UART, I2C, I2S, remote control, pulse counter, LED PWM, full-speed USB 2.0 OTG, USB Serial/JTAG controller, MCPWM, SD/MMC host controller, GDMA, TWAI<sup>®</sup> controller (compatible with ISO 11898-1), ADC, touch sensor, temperature sensor, timers and watchdogs 

- 16 KB SRAM in RTC 

- Up to 16 MB PSRAM 

#### Integrated Components on Module 

#### Wi-Fi 

   - 40 MHz crystal oscillator 

   - Up to 16 MB Quad SPI flash 

- 802.11b/g/n 

- Bit rate: 802.11n up to 150 Mbps 

#### Antenna Options 

   - ESP32-S3-WROOM-1: On-board PCB antenna 

- A-MPDU and A-MSDU aggregation 

- 0.4 _µ_ s guard interval support 

- Center frequency range of operating channel: 2412 ~ 2484 MHz 

- ESP32-S3-WROOM-1U: External antenna via a connector 

#### Operating Conditions 

#### Bluetooth 

   - Operating voltage/Power supply: 3.0 ~ 3.6 V 

   - Operating ambient temperature: 

- Bluetooth LE: Bluetooth 5, Bluetooth mesh 

- Speed: 125 Kbps, 500 Kbps, 1 Mbps, 2 Mbps 

- Advertising extensions 

   - 65 °C version: –40 ~ 65 °C 

   - 85 °C version: –40 ~ 85 °C 

   - 105 °C version: –40 ~ 105 °C 

- Multiple advertisement sets 

- Channel selection algorithm #2 

- Internal co-existence mechanism between Wi-Fi and Bluetooth to share the same antenna 

#### Certification 

- RF certification: See <u>certificates</u> 

- Green certification: RoHS/REACH 

#### Peripherals 

#### Test 

- 36 GPIOs 

- HTOL/HTSL/uHAST/TCT/ESD 

Espressif Systems 

2 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

1 Module Overview 

### 1.2 Series Comparison 

ESP32-S3-WROOM-1 and ESP32-S3-WROOM-1U are two powerful, generic Wi-Fi + Bluetooth LE MCU modules that are built around the ESP32-S3 series of SoCs. On top of a rich set of peripherals, the acceleration for neural network computing and signal processing workloads provided by the SoC make the modules an ideal choice for a wide variety of application scenarios related to AI and Artificial Intelligence of Things (AIoT), such as wake word detection, speech commands recognition, face detection and recognition, smart home, smart appliances, smart control panel, smart speaker, etc. 

ESP32-S3-WROOM-1 comes with a PCB antenna. ESP32-S3-WROOM-1U comes with an external antenna connector. A wide selection of module variants are available for customers as shown in Table 1-1 and 1-2. H4 series modules operate at –40 ~ 105 °C ambient temperature, R8 and R16V series modules operate at –40 ~ 65 °C ambient temperature, and other module variants operate at –40 ~ 85 °C ambient temperature. For R8 and R16V series modules with Octal SPI PSRAM, if the PSRAM ECC function is enabled, the maximum ambient temperature can be improved to 85 °C, while the usable size of PSRAM will be reduced by 1/16. 

Table 1-1. ESP32-S3-WROOM-1 Series Comparison<sup>1</sup> 

|<sup>2</sup>|<sup>3 4</sup>|<sup>4</sup>|Ambient Temp.<sup>5</sup>|Size<sup>6</sup>|
|---|---|---|---|---|
|Part Number|Flash<sup>,</sup>|PSRAM|(°C)|(mm)|
|ESP32-S3-WROOM-1-N4|4 MB (Quad SPI)|-|–40~85||
|ESP32-S3-WROOM-1-N8|8 MB (Quad SPI)|-|–40~85||
|ESP32-S3-WROOM-1-N16|16 MB (Quad SPI)|-|–40~85||
|ESP32-S3-WROOM-1-H4|4 MB (Quad SPI)|-|–40~105|18.0<br>|
|ESP32-S3-WROOM-1-N4R2|4 MB (Quad SPI)|2 MB (Quad SPI)|–40~85|×<br>|
|ESP32-S3-WROOM-1-N8R2|8 MB (Quad SPI)|2 MB (Quad SPI)|–40~85|25.5<br>|
|ESP32-S3-WROOM-1-N16R2|16 MB (Quad SPI)|2 MB (Quad SPI)|–40~85|×<br>|
|ESP32-S3-WROOM-1-N4R8|4 MB (Quad SPI)|8 MB (Octal SPI)|–40~65|3.1|
|ESP32-S3-WROOM-1-N8R8|8 MB (Quad SPI)|8 MB (Octal SPI)|–40~65||
|ESP32-S3-WROOM-1-N16R8|16 MB (Quad SPI)|8 MB (Octal SPI)|–40~65||
|ESP32-S3-WROOM-1-N16R16VA<sup>7</sup>|16 MB (Quad SPI)|16 MB (Octal SPI)|–40~65||

1 This table shares the same notes presented in Table 1-2 below. 

Table 1-2. ESP32-S3-WROOM-1U Series Comparison 

|Part Number<sup>2</sup>|Flash<sup>3, 4</sup>|PSRAM<sup>4</sup>|Ambient Temp.<sup>5</sup><br>(°C)|Size<sup>6</sup><br>(mm)|
|---|---|---|---|---|
|ESP32-S3-WROOM-1U-N4|4 MB (Quad SPI)|-|–40~85||
|ESP32-S3-WROOM-1U-N8|8 MB (Quad SPI)|-|–40~85||
|ESP32-S3-WROOM-1U-N16|16 MB (Quad SPI)|-|–40~85|180|
|ESP32-S3-WROOM-1U-H4|4 MB (Quad SPI)|-|–40~105|.<br>×|
|ESP32-S3-WROOM-1U-N4R2|4 MB (Quad SPI)|2 MB (Quad SPI)|–40~85|192|
|ESP32-S3-WROOM-1U-N8R2|8 MB (Quad SPI)|2 MB (Quad SPI)|–40~85|.<br>×|
|ESP32-S3-WROOM-1U-N16R2|16 MB (Quad SPI)|2 MB (Quad SPI)|–40~85|32|
|ESP32-S3-WROOM-1U-N4R8|4 MB (Quad SPI)|8 MB (Octal SPI)|–40~65|.|
|ESP32-S3-WROOM-1U-N8R8|8 MB (Quad SPI)|8 MB (Octal SPI)|–40~65||
|ESP32-S3-WROOM-1U-N16R8|16 MB (Quad SPI)|8 MB (Octal SPI)|–40~65||
|ESP32-S3-WROOM-1U-N16R16VA<sup>7</sup>|16 MB (Quad SPI)|16 MB (Octal SPI)|–40~65||

Espressif Systems 

3 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

1 Module Overview 

- 2 For customization of ESP32-S3-WROOM-1-H4, ESP32-S3-WROOM-1U-H4, and ESP32-S3-WROOM-1UN16R16VA, please <u>contact us.</u> 

- 3 By default, the SPI flash on the module operates at a maximum clock frequency of 80 MHz and does not support the auto suspend feature. If you have a requirement for a higher flash clock frequency of 120 MHz or if you need the flash auto suspend feature, please <u>contact us.</u> 

- 4 The modules use PSRAM integrated in the chip’s package. For specifications, refer to Section 6.5 _Memory Specifications_ . 

- 5 Ambient temperature specifies the recommended temperature range of the environment immediately outside the Espressif module. 

- 6 For details, refer to Section 10.1 _Module Dimensions_ . 

- 7 Please note that the VDD_SPI voltage is 1.8 V for ESP32-S3-WROOM-1-N16R16VA and ESP32-S3WROOM-1U-N16R16VA only. 

At the core of the modules is an ESP32-S3 series of SoC, an Xtensa<sup>®</sup> 32-bit LX7 CPU that operates at up to 240 MHz. You can power off the CPU and make use of the low-power co-processor to constantly monitor the peripherals for changes or crossing of thresholds. 

Note: 

For more information on ESP32-S3, please refer to _ESP32-S3 Series Datasheet_ . For chip revision identification, ESP-IDF release that supports a specific chip revision, and other information on chip revisions, please refer to _ESP32-S3 Series SoC Errata_ > Section _Chip Revision Identification_ . 

### 1.3 Applications 

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

Espressif Systems 

4 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

Contents 

## Contents 

|1|Module Overview|2|
|---|---|---|
|1.1|Features|2|
|1.2|Series Comparison|3|
|1.3|Applications|4|
|2|Block Diagram|9|
|3|Pin Definitions|10|
|3.1|Pin Layout|10|
|3.2|Pin Description|11|
|4|Boot Configurations|13|
|4.1|i<br>Chip Boot Mode Control|14|
|4.2|VDD_SPI Voltage Control|15|
|4.3|ROM Messages Printing Control|15|
|4.4|JTAG Signal Source Control|15|
|4.5|Chip Power-up and Reset|16|
|5|Peripherals|17|
|5.1|Peripheral Overview|17|
|5.2|Peripheral Description|17|
||5.2.1<br>Connectivity Interface|17|
||5.2.1.1<br>UART Controller|17|
||5.2.1.2<br>I2C Interface|18|
||5.2.1.3<br>I2S Interface|18|
||5.2.1.4<br>LCD and Camera Controller|19|
||5.2.1.5<br>Serial Peripheral Interface (SPI)|19|
||5.2.1.6<br>Two-Wire Automotive Interface (TWAI<sup>®</sup>)|21|
||5.2.1.7<br>USB 2.0 OTG Full-Speed Interface|21|
||5.2.1.8<br>USB Serial/JTAG Controller|22|
||5.2.1.9<br>SD/MMC Host Controller|23|
||5.2.1.10<br>Motor Control PWM (MCPWM)|24|
||5.2.1.11<br>Remote Control Peripheral (RMT)|24|
||5.2.1.12<br>Pulse Count Controller (PCNT)|25|
||5.2.2<br>Analog Signal Processing|25|
||5.2.2.1<br>SAR ADC|25|
||5.2.2.2<br>Temperature Sensor|25|
||5.2.2.3<br>Touch Sensor|26|
|6|Electrical Characteristics|27|
|6.1|Absolute Maximum Ratings|27|
|6.2|Recommended Operating Conditions|27|
|6.3|DC Characteristics (3.3 V, 25 °C)|27|

Espressif Systems 

5 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

Contents 

|6.4|Current Consumption Characteristics|28|
|---|---|---|
||6.4.1<br>Current Consumption in Active Mode|28|
||6.4.2<br>Current Consumption in Other Modes|29|
|6.5|Memory Specifications|30|
|7|RF Characteristics|32|
|7.1|Wi-Fi Radio|32|
||7.1.1<br>Wi-Fi RF Transmitter (TX) Characteristics|32|
||7.1.2<br>Wi-Fi RF Receiver (RX) Characteristics|33|
|7.2|Bluetooth LE Radio|34|
||7.2.1<br>Bluetooth LE RF Transmitter (TX) Characteristics|35|
||7.2.2<br>Bluetooth LE RF Receiver (RX) Characteristics|36|
|8|Module Schematics|39|
|9|Peripheral Schematics|41|
|10|Physical Dimensions|42|
|10.1|Module Dimensions|42|
|10.2|Dimensions of External Antenna Connector|43|
|11|PCB Layout Recommendations|45|
|11.1|PCB Land Pattern|45|
|11.2|Module Placement for PCB Design|46|
|12|Product Handling|47|
|12.1|Storage Conditions|47|
|12.2|Electrostatic Discharge (ESD)|47|
|12.3|Reflow Profile|47|
|12.4|Ultrasonic Vibration|48|
|Dat|asheet Versioning|49|
|Rel|ated Documentation and Resources|50|
|Re|vision History|51|

Espressif Systems 

6 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

List of Tables 

## List of Tables 

|1-1|ESP32-S3-WROOM-1 Series Comparison<sup>1</sup>|3|
|---|---|---|
|1-2|ESP32-S3-WROOM-1U Series Comparison|3|
|3-1|Pin Definitions|11|
|4-1|Default Configuration of Strapping Pins|13|
|4-2|Description of Timing Parameters for the Strapping Pins|14|
|4-3|Chip Boot Mode Control|14|
|4-4|VDD_SPI Voltage Control|15|
|4-5|JTAG Signal Source Control|16|
|4-6|Description of Timing Parameters for Power-up and Reset|16|
|6-1|Absolute Maximum Ratings|27|
|6-2|Recommended Operating Conditions|27|
|6-3|DC Characteristics (3.3 V, 25 °C)|27|
|6-4|Current Consumption for Wi-Fi (2.4 GHz) in Active Mode|28|
|6-5|Current Consumption for Bluetooth LE in Active Mode|28|
|6-6|Current Consumption in Modem-sleep Mode|29|
|6-7|Current Consumption in Low-Power Modes|30|
|6-8|Flash Specifications|30|
|6-9|PSRAM Specifications|31|
|7-1|Wi-Fi RF Characteristics|32|
|7-2|TX Power with Spectral Mask and EVM Meeting 802.11 Standards|32|
|7-3|TX EVM Test<sup>1</sup>|32|
|7-4|RX Sensitivity|33|
|7-5|Maximum RX Level|34|
|7-6|RX Adjacent Channel Rejection|34|
|7-7|Bluetooth LE RF Characteristics|34|
|7-8|Bluetooth LE - Transmitter Characteristics - 1 Mbps|35|
|7-9|Bluetooth LE - Transmitter Characteristics - 2 Mbps|35|
|7-10|Bluetooth LE - Transmitter Characteristics - 125 Kbps|35|
|7-11|Bluetooth LE - Transmitter Characteristics - 500 Kbps|36|
|7-12|Bluetooth LE - Receiver Characteristics - 1 Mbps|36|
|7-13|Bluetooth LE - Receiver Characteristics - 2 Mbps|36|
|7-14|Bluetooth LE - Receiver Characteristics - 125 Kbps|37|
|7-15|Bluetooth LE - Receiver Characteristics - 500 Kbps|37|

Espressif Systems 

7 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

List of Figures 

## List of Figures 

|2-1|ESP32-S3-WROOM-1 Block Diagram|9|
|---|---|---|
|2-2|ESP32-S3-WROOM-1U Block Diagram|9|
|3-1|Pin Layout (Top View)|10|
|4-1|Visualization of Timing Parameters for the Strapping Pins|14|
|4-2|Visualization of Timing Parameters for Power-up and Reset|16|
|8-1|ESP32-S3-WROOM-1 Schematics|39|
|8-2|ESP32-S3-WROOM-1U Schematics|40|
|9-1|Peripheral Schematics|41|
|10-1|ESP32-S3-WROOM-1 Physical Dimensions|42|
|10-2|ESP32-S3-WROOM-1U Physical Dimensions|42|
|10-3|Dimensions of External Antenna Connector|43|
|11-1|ESP32-S3-WROOM-1 Recommended PCB Land Pattern|45|
|11-2|ESP32-S3-WROOM-1U Recommended PCB Land Pattern|46|
|12-1|Reflow Profile|47|

Espressif Systems 

8 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

2 Block Diagram 

## 2 Block Diagram 

Figure 2-1. ESP32-S3-WROOM-1 Block Diagram 

Figure 2-2. ESP32-S3-WROOM-1U Block Diagram 

Note: 

For the pin mapping between the chip and the in-package PSRAM, please refer to _ESP32-S3 Series Datasheet_ > Table _Pin Mapping Between Chip and In-package Flash/PSRAM_ . 

Espressif Systems 

9 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

3 Pin Definitions 

## 3 Pin Definitions 

### 3.1 Pin Layout 

The pin diagram below shows the approximate location of pins on the module. For the actual diagram drawn to scale, please refer to Figure 10.1 _Module Dimensions_ . 

Figure 3-1. Pin Layout (Top View) 

###### Note: 

The pin diagram is applicable to ESP32-S3-WROOM-1 and ESP32-S3-WROOM-1U, but the latter has no antenna keepout zone. To learn more about the keepout zone for module’s antenna on the base board, please refer to _ESP32-S3 Hardware Design Guidelines_ > Section _General Principles of PCB Layout for Modules_ . 

Espressif Systems 

10 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

3 Pin Definitions 

### 3.2 Pin Description 

The module has 41 pins. See pin definitions in Table 3-1 _Pin Definitions_ . 

For explanations of pin names and function names, as well as configurations of peripheral pins, please refer to _<u>ESP32-S3 Series Datasheet</u>_ <u>.</u> 

Table 3-1. Pin Definitions 

|Name|No.|Type <sup>a</sup>|Function|
|---|---|---|---|
|GND|1|P|GND|
|3V3|2|P|Power supply|
||||High: on, enables the chip.|
|EN|3|I|Low: off, the chip powers off.<br>Note: Do not leave the EN pin floating.|
|IO4|4|I/O/T|RTC_GPIO4,GPIO4, TOUCH4, ADC1_CH3|
|IO5|5|I/O/T|RTC_GPIO5,GPIO5, TOUCH5, ADC1_CH4|
|IO6|6|I/O/T|RTC_GPIO6,GPIO6, TOUCH6, ADC1_CH5|
|IO7|7|I/O/T|RTC_GPIO7,GPIO7, TOUCH7, ADC1_CH6|
|IO15|8|I/O/T|RTC_GPIO15,GPIO15, U0RTS, ADC2_CH4, XTAL_32K_P|
|IO16|9|I/O/T|RTC_GPIO16,GPIO16, U0CTS, ADC2_CH5, XTAL_32K_N|
|IO17|10|I/O/T|RTC_GPIO17,GPIO17, U1TXD, ADC2_CH6|
|IO18|11|I/O/T|RTC_GPIO18,GPIO18, U1RXD, ADC2_CH7, CLK_OUT3|
|IO8|12|I/O/T|RTC_GPIO8,GPIO8, TOUCH8, ADC1_CH7, SUBSPICS1|
|IO19|13|I/O/T|RTC_GPIO19, GPIO19, U1RTS, ADC2_CH8, CLK_OUT2,USB_D-|
|IO20|14|I/O/T|RTC_GPIO20, GPIO20, U1CTS, ADC2_CH9, CLK_OUT1,USB_D+|
|IO3|15|I/O/T|RTC_GPIO3,GPIO3, TOUCH3, ADC1_CH2|
|IO46|16|I/O/T|GPIO46|
|IO9|17|I/O/T|RTC_GPIO9,GPIO9, TOUCH9, ADC1_CH8, FSPIHD, SUBSPIHD|
|IO10|18|I/O/T|RTC_GPIO10,GPIO10, TOUCH10, ADC1_CH9, FSPICS0, FSPIIO4,<br>SUBSPICS0|
|IO11|19|I/O/T|RTC_GPIO11,GPIO11, TOUCH11, ADC2_CH0, FSPID, FSPIIO5, SUBSPID|
|IO12|20|I/O/T|RTC_GPIO12,GPIO12, TOUCH12, ADC2_CH1, FSPICLK, FSPIIO6,<br>SUBSPICLK|
|IO13|21|I/O/T|RTC_GPIO13,GPIO13, TOUCH13, ADC2_CH2, FSPIQ, FSPIIO7, SUBSPIQ|
|IO14|22|I/O/T|RTC_GPIO14,GPIO14, TOUCH14, ADC2_CH3, FSPIWP, FSPIDQS,<br>SUBSPIWP|
|IO21|23|I/O/T|RTC_GPIO21,GPIO21|
|IO47<sup>c</sup>|24|I/O/T|SPICLK_P_DIFF,GPIO47, SUBSPICLK_P_DIFF|
|IO48<sup>c</sup>|25|I/O/T|SPICLK_N_DIFF,GPIO48, SUBSPICLK_N_DIFF|
|IO45|26|I/O/T|GPIO45|
|IO0|27|I/O/T|RTC_GPIO0,GPIO0|
|IO35<sup>b</sup>|28|I/O/T|SPIIO6,GPIO35, FSPID, SUBSPID|
|IO36<sup>b</sup><br>|29|I/O/T|SPIIO7,GPIO36, FSPICLK, SUBSPICLK|
|IO37<sup>b</sup>|30|I/O/T|SPIDQS,GPIO37, FSPIQ, SUBSPIQ|
|IO38|31|I/O/T|GPIO38, FSPIWP, SUBSPIWP|

Cont’d on next page 

Espressif Systems 

11 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

3 Pin Definitions 

Table 3-1 – cont’d from previous page 

|Name|No.|Type <sup>a</sup>|Function|
|---|---|---|---|
|IO39|32|I/O/T|MTCK, GPIO39, CLK_OUT3, SUBSPICS1|
|IO40|33|I/O/T|MTDO, GPIO40, CLK_OUT2|
|IO41|34|I/O/T|MTDI, GPIO41, CLK_OUT1|
|IO42|35|I/O/T|MTMS, GPIO42|
|RXD0|36|I/O/T|U0RXD, GPIO44, CLK_OUT2|
|TXD0|37|I/O/T|U0TXD, GPIO43, CLK_OUT1|
|IO2|38|I/O/T|RTC_GPIO2,GPIO2, TOUCH2, ADC1_CH1|
|IO1|39|I/O/T|RTC_GPIO1,GPIO1, TOUCH1, ADC1_CH0|
|GND|40|P|GND|
|EPAD|41|P|GND|

- a P: power supply; I: input; O: output; T: high impedance. Pin functions in bold font are the default pin functions. For pin 28 _∼_ 30, the default function is decided by eFuse bit. 

- b For modules with Octal SPI PSRAM, i.e., modules embedded with ESP32-S3R8 or ESP32-S3R16V, pins IO35, IO36, and IO37 are connected to the Octal SPI PSRAM and are not available for other uses. 

- c For modules embedded with ESP32-S3R16V, as the VDD_SPI voltage of the ESP32-S3R16V chip is set to 1.8 V, the working voltage for GPIO47 and GPIO48 is also 1.8 V, which is different from other GPIOs. 

Espressif Systems 

12 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

4 Boot Configurations 

## 4 Boot Configurations 

Note: 

The content below is excerpted from _ESP32-S3 Series Datasheet_ > Section _Boot Configurations_ . For the strapping pin mapping between the chip and modules, please refer to Chapter 8 _Module Schematics_ . 

The chip allows for configuring the following boot parameters through strapping pins and eFuse bits at power-up or a hardware reset, without microcontroller interaction. 

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

Table 4-1. Default Configuration of Strapping Pins 

|StrappingPin|Default Configuration|Bit Value|
|---|---|---|
|GPIO0|Weak pull-up|1|
|GPIO3|Floating|–|
|GPIO45|Weak pull-down|0|
|GPIO46|Weak pull-down|0|

To change the bit values, the strapping pins should be connected to external pull-down/pull-up resistances. If the ESP32-S3 is used as a device by a host MCU, the strapping pin voltage levels can also be controlled by the host MCU. 

All strapping pins have latches. At system reset, the latches sample the bit values of their respective strapping pins and store them until the chip is powered down or shut down. The states of latches cannot be changed in 

Espressif Systems 

13 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

4 Boot Configurations 

any other way. It makes the strapping pin values available during the entire chip operation, and the pins are freed up to be used as regular IO pins after reset. 

The timing of signals connected to the strapping pins should adhere to the _setup time_ and _hold time_ specifications in Table 4-2 and Figure 4-1. 

Table 4-2. Description of Timing Parameters for the Strapping Pins 

|Parameter|Description|Min (ms)|
|---|---|---|
|t_SU_|_Setup time_is the time reserved for the power rails to stabilize be-<br>fore the EN pin is pulled high to activate the chip.|0|
||_Hold time_ is the time reserved for the chip to read the strapping||
|t_H_|pin values after EN is already high and before these pins start op-<br>eratingas regular IO pins.|3|

Figure 4-1. Visualization of Timing Parameters for the Strapping Pins 

### 4.1 Chip Boot Mode Control 

GPIO0 and GPIO46 control the boot mode after the reset is released. See Table 4-3 _Chip Boot Mode Control_ . 

Table 4-3. Chip Boot Mode Control 

|Boot Mode|GPIO0|GPIO46|
|---|---|---|
|SPI Boot|1|Any value|
|Joint Download Boot<sup>2</sup>|0|0|

- 1 Bold marks the default value and configuration. 

- 2 Joint Download Boot mode supports the following download methods: 

   - USB Download Boot: 

      - USB-Serial-JTAG Download Boot 

      - USB-OTG Download Boot 

   - UART Download Boot 

Espressif Systems 

14 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

4 Boot Configurations 

In SPI Boot mode, the ROM bootloader loads and executes the program from SPI flash to boot the system. 

In Joint Download Boot mode, users can download binary files into flash using UART0 or USB interface. It is also possible to download binary files into SRAM and execute it from SRAM. 

In addition to SPI Boot and Joint Download Boot modes, ESP32-S3 also supports SPI Download Boot mode. For details, please see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Chip Boot Control_ . 

### 4.2 VDD_SPI Voltage Control 

Depending on the value of EFUSE_VDD_SPI_FORCE, the voltage can be controlled in two ways. 

Table 4-4. VDD_SPI Voltage Control 

|VDD_SPI power source <sup>2</sup>|Voltage|EFUSE_VDD_SPI_FORCE|GPIO45|EFUSE_VDD_SPI_TIEH|
|---|---|---|---|---|
|VDD3P3_RTC via R_SP I_|3.3 V||0||
|Flash Voltage Regulator|1.8 V|0|1|Ignored|
|Flash Voltage Regulator|1.8 V|||0|
|VDD3P3_RTC via R_SP I_|3.3 V|1|Ignored|1|

- 1 Bold marks the default value and configuration. 

- 2 See _<u>ESP32-S3 Series Datasheet</u>_ > Section _Power Scheme_ . 

### 4.3 ROM Messages Printing Control 

During boot process the messages by the ROM code can be printed to: 

- (Default) UART0 and USB Serial/JTAG controller 

- USB Serial/JTAG controller 

- UART0 

The ROM messages printing to UART or USB Serial/JTAG controller can be respectively disabled by configuring registers and eFuse. For detailed information, please refer to _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Chip Boot Control_ . 

### 4.4 JTAG Signal Source Control 

The strapping pin GPIO3 can be used to control the source of JTAG signals during the early boot process. This pin does not have any internal pull resistors and the strapping value must be controlled by the external circuit that cannot be in a high impedance state. 

As Table 4-5 shows, GPIO3 is used in combination with EFUSE_DIS_PAD_JTAG, EFUSE_DIS_USB_JTAG, and EFUSE_STRAP_JTAG_SEL. 

Espressif Systems 

15 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

4 Boot Configurations 

Table 4-5. JTAG Signal Source Control 

|JTAG Signal Source|EFUSE_DIS_PAD_JTAG<br>|EFUSE_DIS_USB_JTAG|EFUSE_STRAP_JTAG_|SEL<br>GPIO3|
|---|---|---|---|---|
||0|0|0|Ignored|
|USB Serial/JTAG Controller|0|0|1|1|
||1|0|Ignored|Ignored|
|JTAG ins<sup>2</sup>|0|0|1|0|
|p|0|1|Ignored|Ignored|
|JTAG is disabled|1|1|Ignored|Ignored|

- 1 Bold marks the default value and configuration. 

2 JTAG pins refer to MTDI, MTCK, MTMS, and MTDO. 

### 4.5 Chip Power-up and Reset 

Once the power is supplied to the chip, its power rails need a short time to stabilize. After that, EN – the pin used for power-up and reset – is pulled high to activate the chip. For information on EN as well as power-up and reset timing, see Figure 4-2 and Table 4-6. 

Figure 4-2. Visualization of Timing Parameters for Power-up and Reset 

Table 4-6. Description of Timing Parameters for Power-up and Reset 

|Parameter|Description|Min (_µ_s)|
|---|---|---|
||Time<br>reserved<br>for<br>the<br>power<br>rails<br>of<br>VDDA,<br>VDD3P3,||
|t_ST BL_|VDD3P3_RTC, and VDD3P3_CPU to stabilize before the EN|50|
||pin is pulled high to activate the chip||
|t_RST_|Time reserved for EN to stay below V_IL_nRST_ to reset the chip<br>(see Table6-3)|50|

Espressif Systems 

16 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

5 Peripherals 

## 5 Peripherals 

### 5.1 Peripheral Overview 

ESP32-S3 integrates a rich set of peripherals including SPI, LCD, Camera interface, UART, I2C, I2S, remote control, pulse counter, LED PWM, USB Serial/JTAG, MCPWM, SD/MMC host controller, TWAI<sup>®</sup> controller (compatible with ISO 11898-1, i.e., CAN Specification 2.0), ADC, touch sensor, and temperature sensor. It also includes a full-speed USB 2.0 On-The-Go (OTG) interface to enable USB communication. 

To learn more about on-chip components, please refer to _<u>ESP32-S3 Series Datasheet</u>_ > Section _Functional Description_ . 

###### Note: 

The content below is sourced from _ESP32-S3 Series Datasheet_ > Section _Peripherals_ . Some information may not be applicable to ESP32-S3-WROOM-1 and ESP32-S3-WROOM-1U as not all the IO signals are exposed on the module. To learn more about peripheral signals, please refer to _ESP32-S3 Technical Reference Manual_ > Section _Peripheral Signals via GPIO Matrix_ . 

### 5.2 Peripheral Description 

This section describes the chip’s peripheral capabilities, covering connectivity interfaces and on-chip sensors that extend its functionality. 

#### 5.2.1 Connectivity Interface 

This subsection describes the connectivity interfaces on the chip that enable communication and interaction with external devices and networks. 

#### 5.2.1.1 UART Controller 

ESP32-S3 has three UART (Universal Asynchronous Receiver Transmitter) controllers, i.e., UART0, UART1, and UART2, which support IrDA and asynchronous communication (RS232 and RS485) at a speed of up to 5 Mbps. 

##### Feature List 

- Three clock sources that can be divided 

- Programmable baud rate 

- 1024 x 8-bit RAM shared by TX FIFOs and RX FIFOs of the three UART controllers 

- Full-duplex asynchronous communication 

- Automatic baud rate detection of input signals 

- Data bits ranging from 5 to 8 

- Stop bits of 1, 1.5, 2, or 3 bits 

- Parity bit 

Espressif Systems 

17 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

5 Peripherals 

- Special character AT_CMD detection 

- RS485 protocol 

- IrDA protocol 

- High-speed data communication using GDMA 

- UART as wake-up source 

- Software and hardware flow control 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _UART Controller_ . 

##### Pin Assignment 

For details, see _<u>ESP32-S3 Series Datasheet</u>_ > Section _Peripheral Pin Assignment_ . 

#### 5.2.1.2 I2C Interface 

ESP32-S3 has two I2C bus interfaces which are used for I2C master mode or slave mode, depending on the user’s configuration. 

##### Feature List 

- Standard mode (100 kbit/s) 

- Fast mode (400 kbit/s) 

- Up to 800 kbit/s (constrained by SCL and SDA pull-up strength) 

- 7-bit and 10-bit addressing mode 

- Double addressing mode (slave addressing and slave register addressing) 

The hardware provides a command abstraction layer to simplify the usage of the I2C peripheral. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _I2C Controller_ . 

##### Pin Assignment 

For details, see _<u>ESP32-S3 Series Datasheet</u>_ > Section _Peripheral Pin Assignment_ . 

#### 5.2.1.3 I2S Interface 

ESP32-S3 includes two standard I2S interfaces. They can operate in master mode or slave mode, in full-duplex mode or half-duplex communication mode, and can be configured to operate with an 8-bit, 16-bit, 24-bit, or 32-bit resolution as an input or output channel. BCK clock frequency, from 10 kHz up to 40 MHz, is supported. 

The I2S interface has a dedicated DMA controller. It supports TDM PCM, TDM MSB alignment, TDM LSB alignment, TDM Phillips, and PDM interface. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _I2S Controller_ . 

Espressif Systems 

18 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

5 Peripherals 

##### Pin Assignment 

For details, see _<u>ESP32-S3 Series Datasheet</u>_ > Section _Peripheral Pin Assignment_ . 

#### 5.2.1.4 LCD and Camera Controller 

The LCD and Camera controller of ESP32-S3 consists of a LCD module and a camera module. 

The LCD module is designed to send parallel video data signals, and its bus supports 8-bit ~ 16-bit parallel RGB, I8080, and MOTO6800 interfaces. These interfaces operate at 40 MHz or lower, and support conversion among RGB565, YUV422, YUV420, and YUV411. 

The camera module is designed to receive parallel video data signals, and its bus supports an 8-bit ~ 16-bit DVP image sensor, with clock frequency of up to 40 MHz. The camera interface supports conversion among RGB565, YUV422, YUV420, and YUV411. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _LCD and Camera Controller_ . 

##### Pin Assignment 

For details, see _<u>ESP32-S3 Series Datasheet</u>_ > Section _Peripheral Pin Assignment_ . 

#### 5.2.1.5 Serial Peripheral Interface (SPI) 

ESP32-S3 has the following SPI interfaces: 

- SPI0 used by ESP32-S3’s GDMA controller and cache to access in-package or off-package flash/PSRAM 

- SPI1 used by the CPU to access in-package or off-package flash/PSRAM 

- SPI2 is a general purpose SPI controller with access to a DMA channel allocated by the GDMA controller 

- SPI3 is a general purpose SPI controller with access to a DMA channel allocated by the GDMA controller 

##### Feature List 

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

Espressif Systems 

19 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

5 Peripherals 

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

##### Pin Assignment 

For details, see _<u>ESP32-S3 Series Datasheet</u>_ > Section _Peripheral Pin Assignment_ . 

Espressif Systems 

20 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

5 Peripherals 

#### 5.2.1.6 Two-Wire Automotive Interface (TWAI<sup>®</sup> ) 

The Two-Wire Automotive Interface (TWAI<sup>®</sup> ) is a multi-master, multi-cast communication protocol with error detection and signaling as well as inbuilt message priorities and arbitration. 

##### Feature List 

- Compatible with ISO 11898-1 protocol (CAN Specification 2.0) 

- Standard frame format (11-bit ID) and extended frame format (29-bit ID) 

- Bit rates from 1 Kbit/s to 1 Mbit/s 

- Multiple modes of operation: 

   - Normal 

   - Listen Only 

   - Self-Test (no acknowledgment required) 

- 64-byte receive FIFO 

- Acceptance filter (single and dual filter modes) 

- Error detection and handling: 

   - Error counters 

   - Configurable error interrupt threshold 

   - Error code capture 

   - Arbitration lost capture 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Two-wire Automotive Interface_ . 

##### Pin Assignment 

For details, see _<u>ESP32-S3 Series Datasheet</u>_ > Section _Peripheral Pin Assignment_ . 

#### 5.2.1.7 USB 2.0 OTG Full-Speed Interface 

ESP32-S3 features a full-speed USB OTG interface along with an integrated transceiver. The USB OTG interface complies with the USB 2.0 specification. 

##### General Features 

- FS and LS data rates 

- HNP and SRP as A-device or B-device 

- Dynamic FIFO (DFIFO) sizing 

- Multiple modes of memory access 

   - Scatter/Gather DMA mode 

   - Buffer DMA mode 

Espressif Systems 

21 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

5 Peripherals 

   - Slave mode 

- Can choose integrated transceiver or external transceiver 

- Utilizing integrated transceiver with USB Serial/JTAG by time-division multiplexing when only integrated transceiver is used 

- Support USB OTG using one of the transceivers while USB Serial/JTAG using the other one when both integrated transceiver or external transceiver are used 

##### Device Mode Features 

- Endpoint number 0 always present (bi-directional, consisting of EP0 IN and EP0 OUT) 

- Six additional endpoints (endpoint numbers 1 to 6), configurable as IN or OUT 

- Maximum of five IN endpoints concurrently active at any time (including EP0 IN) 

- All OUT endpoints share a single RX FIFO 

- Each IN endpoint has a dedicated TX FIFO 

##### Host Mode Features 

- Eight channels (pipes) 

   - A control pipe consists of two channels (IN and OUT), as IN and OUT transactions must be handled separately. Only Control transfer type is supported. 

   - Each of the other seven channels is dynamically configurable to be IN or OUT, and supports Bulk, Isochronous, and Interrupt transfer types. 

- All channels share an RX FIFO, non-periodic TX FIFO, and periodic TX FIFO. The size of each FIFO is configurable. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _USB On-The-Go_ . 

##### Pin Assignment 

For details, see _<u>ESP32-S3 Series Datasheet</u>_ > Section _Peripheral Pin Assignment_ . 

#### 5.2.1.8 USB Serial/JTAG Controller 

ESP32-S3 integrates a USB Serial/JTAG controller. 

##### Feature List 

- USB Full-speed device. 

- Can be configured to either use internal USB PHY of ESP32-S3 or external PHY via GPIO matrix. 

- Fixed function device, hardwired for CDC-ACM (Communication Device Class - Abstract Control Model) and JTAG adapter functionality. 

- Two OUT Endpoints, three IN Endpoints in addition to Control Endpoint 0; Up to 64-byte data payload size. 

Espressif Systems 

22 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

5 Peripherals 

- Internal PHY, so no or very few external components needed to connect to a host computer. 

- CDC-ACM adherent serial port emulation is plug-and-play on most modern OSes. 

- JTAG interface allows fast communication with CPU debug core using a compact representation of JTAG instructions. 

- CDC-ACM supports host controllable chip reset and entry into download mode. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _USB Serial/JTAG Controller_ . 

##### Pin Assignment 

For details, see _<u>ESP32-S3 Series Datasheet</u>_ > Section _Peripheral Pin Assignment_ . 

#### 5.2.1.9 SD/MMC Host Controller 

ESP32-S3 has an SD/MMC Host controller. 

##### Feature List 

- Secure Digital (SD) memory version 3.0 and version 3.01 

- Secure Digital I/O (SDIO) version 3.0 

- Consumer Electronics Advanced Transport Architecture (CE-ATA) version 1.1 

- Multimedia Cards (MMC version 4.41, eMMC version 4.5 and version 4.51) 

- Up to 80 MHz clock output 

- Three data bus modes: 

   - 1-bit 

   - 4-bit (supports two SD/SDIO/MMC 4.41 cards, and one SD card operating at 1.8 V in 4-bit mode) 

   - 8-bit 

###### Note: 

When working at 80 MHz, the clock phase adjustment is limited and only phase 0° and 180° are supported. The PCB layout should be optimized accordingly to ensure timing closure. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _SD/MMC Host Controller_ . 

##### Pin Assignment 

For details, see _<u>ESP32-S3 Series Datasheet</u>_ > Section _Peripheral Pin Assignment_ . 

##### Feature List 

- Can generate a digital waveform with configurable periods and duty cycle. The duty cycle resolution can be up to 14 bits within a 1 ms period 

- Multiple clock sources, including APB clock and external main crystal clock 

- Can operate when the CPU is in Light-sleep mode 

Espressif Systems 

23 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

5 Peripherals 

_•_ Gradual increase or decrease of duty cycle, useful for the LED RGB color-fading generator 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _LED PWM Controller_ . 

##### Pin Assignment 

For details, see _<u>ESP32-S3 Series Datasheet</u>_ > Section _Peripheral Pin Assignment_ . 

#### 5.2.1.10 Motor Control PWM (MCPWM) 

ESP32-S3 integrates two MCPWMs that can be used to drive digital motors and smart light. Each MCPWM peripheral has one clock divider (prescaler), three PWM timers, three PWM operators, and a capture module. PWM timers are used for generating timing references. The PWM operators generate desired waveform based on the timing references. Any PWM operator can be configured to use the timing references of any PWM timers. Different PWM operators can use the same PWM timer’s timing references to produce related PWM signals. PWM operators can also use different PWM timers’ values to produce the PWM signals that work alone. Different PWM timers can also be synchronized together. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Motor Control PWM_ . 

##### Pin Assignment 

For details, see _<u>ESP32-S3 Series Datasheet</u>_ > Section _Peripheral Pin Assignment_ . 

#### 5.2.1.11 Remote Control Peripheral (RMT) 

The Remote Control Peripheral (RMT) is designed to send and receive infrared remote control signals. 

##### Feature List 

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

##### Pin Assignment 

For details, see _<u>ESP32-S3 Series Datasheet</u>_ > Section _Peripheral Pin Assignment_ . 

Espressif Systems 

24 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

5 Peripherals 

#### 5.2.1.12 Pulse Count Controller (PCNT) 

The pulse count controller (PCNT) captures pulse and counts pulse edges through multiple modes. 

##### Feature List 

- Four independent pulse counters (units) that count from 1 to 65535 

- Each unit consists of two independent channels sharing one pulse counter 

- All channels have input pulse signals (e.g. sig_ch0_u _n_ ) with their corresponding control signals (e.g. ctrl_ch0_u _n_ ) 

- Independently filter glitches of input pulse signals (sig_ch0_u _n_ and sig_ch1_u _n_ ) and control signals (ctrl_ch0_u _n_ and ctrl_ch1_u _n_ ) on each unit 

- Each channel has the following parameters: 

   1. Selection between counting on positive or negative edges of the input pulse signal 

   2. Configuration to Increment, Decrement, or Disable counter mode for control signal’s high and low states 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _Pulse Count Controller_ . 

##### Pin Assignment 

For details, see _<u>ESP32-S3 Series Datasheet</u>_ > Section _Peripheral Pin Assignment_ . 

#### 5.2.2 Analog Signal Processing 

This subsection describes components on the chip that sense and process real-world data. 

#### 5.2.2.1 SAR ADC 

ESP32-S3 integrates two 12-bit SAR ADCs and supports measurements on 20 channels (analog-enabled pins). For power-saving purpose, the ULP coprocessors in ESP32-S3 can also be used to measure voltage in sleep modes. By using threshold settings or other methods, we can awaken the CPU from sleep modes. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _On-Chip Sensors and Analog Signal Processing_ . 

##### Pin Assignment 

For details, see _<u>ESP32-S3 Series Datasheet</u>_ > Section _Peripheral Pin Assignment_ . 

#### 5.2.2.2 Temperature Sensor 

The temperature sensor generates a voltage that varies with temperature. The voltage is internally converted via an ADC into a digital value. 

The temperature sensor has a range of –40 °C to 125 °C. It is designed primarily to sense the temperature changes inside the chip. The temperature value depends on factors such as microcontroller clock frequency or I/O load. Generally, the chip’s internal temperature is higher than the ambient temperature. 

Espressif Systems 

25 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

5 Peripherals 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _On-Chip Sensors and Analog Signal Processing_ . 

#### 5.2.2.3 Touch Sensor 

ESP32-S3 has 14 capacitive-sensing GPIOs, which detect variations induced by touching or approaching the GPIOs with a finger or other objects. The low-noise nature of the design and the high sensitivity of the circuit allow relatively small pads to be used. Arrays of pads can also be used, so that a larger area or more points can be detected. The touch sensing performance can be further enhanced by the waterproof design and digital filtering feature. 

Note: 

ESP32-S3 touch sensor has not passed the Conducted Susceptibility (CS) test for now, and thus has limited application scenarios. 

For details, see _<u>ESP32-S3 Technical Reference Manual</u>_ > Chapter _On-Chip Sensors and Analog Signal Processing_ . 

Pin Assignment 

For details, see _<u>ESP32-S3 Series Datasheet</u>_ > Section _Peripheral Pin Assignment_ . 

Espressif Systems 

26 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

5 Peripherals 

## 6 Electrical Characteristics 

### 6.1 Absolute Maximum Ratings 

Stresses above those listed in Table 6-1 _Absolute Maximum Ratings_ may cause permanent damage to the device. These are stress ratings only and functional operation of the device at these or any other conditions beyond those indicated under Table 6-2 _Recommended Operating Conditions_ is not implied. Exposure to absolute-maximum-rated conditions for extended periods may affect device reliability. 

Table 6-1. Absolute Maximum Ratings 

|Symbol|Parameter|Min|Max|Unit|
|---|---|---|---|---|
|VDD33|Power supply voltage|–0.3|3.6|V|
|T_ST ORE_|Storage temperature|–40|105|°C|

### 6.2 Recommended Operating Conditions 

Table 6-2. Recommended Operating Conditions 

|Symbol|Parameter||Min|Typ|Max|Unit|
|---|---|---|---|---|---|---|
|VDD33|Power supply voltage||3.0|3.3|3.6|V|
|I_V DD_|Current delivered by external po|wer supply|0.5|—|—|A|
|||65 °C version|||65||
|T_A_|Operating ambient temperature|85 °C version|–40|—|85|°C|
|||105 °C version|||105||

### 6.3 DC Characteristics (3.3 V, 25 °C) 

Table 6-3. DC Characteristics (3.3 V, 25 °C) 

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

Espressif Systems 

27 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

5 Peripherals 

|V_IH_nRST_|Chip reset release voltage (EN voltage is<br>within the specified range)|0.75 × VDD<sup>1</sup>|—|VDD<sup>1 </sup>+ 0.3|V|
|---|---|---|---|---|---|
|V_IL_nRST_|Chip reset voltage (EN voltage is within the<br>specified range)|–0.3|—|0.25 × VDD<sup>1</sup>|V|

- 1 VDD – voltage from a power pin of a respective power domain. 

- 2 V _OH_ and V _OL_ are measured using high-impedance load. 

### 6.4 Current Consumption Characteristics 

#### 6.4.1 Current Consumption in Active Mode 

With the use of advanced power-management technologies, the module can switch between different power modes. For details on different power modes, please refer to _<u>ESP32-S3 Series Datasheet</u>_ > Section _Power Management Unit_ . 

The current consumption measurements are taken with a 3.3 V supply at 25 °C ambient temperature. 

TX current consumption is rated at a 100% duty cycle. 

RX current consumption is rated when the peripherals are disabled and the CPU idle. 

Table 6-4. Current Consumption for Wi-Fi (2.4 GHz) in Active Mode 

|Work Mode<br>RF Condition|Description|Peak (mA)|
|---|---|---|
||802.11b, 1 Mbps, @20.5 dBm|355|
|TX|802.11g, 54 Mbps, @18 dBm|297|
|k<br>|802.11n, HT20, MCS 7, @17.5 dBm|286|
|Active (RF woring)|802.11n, HT40, MCS 7, @17 dBm|285|
|RX|802.11b/g/n, HT20|95|
||802.11n, HT40|97|

Table 6-5. Current Consumption for Bluetooth LE in Active Mode 

|Work Mode|RF Condition|Description|Peak (mA)|
|---|---|---|---|
|||Bluetooth LE @ 20.0 dBm|344|
||TX|Bluetooth LE @ 9.0 dBm|202|
|Active (RF working)||Bluetooth LE @ 0 dBm|187|
|||Bluetooth LE @ –15.0 dBm|119|
||RX|Bluetooth LE|93|

Espressif Systems 

28 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

5 Peripherals 

Note: 

The content below is excerpted from Section _Current Consumption in Other Modes_ in _ESP32-S3 Series Datasheet_ . 

#### 6.4.2 Current Consumption in Other Modes 

Please note that if the chip embedded has in-package PSRAM, the current consumption of the module might be higher compared to the measurements below. 

Table 6-6. Current Consumption in Modem-sleep Mode 

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
|Mdl<sup>3</sup>||Dual core running128-bit data access instructions|41.8|56.3|
|oem-seep||WAITI|27.6|42.3|
|||Single core running 32-bit data access instructions, the<br>other core in idle state|39.9|54.6|
||160|Dual core running32-bit data access instructions|49.6|64.1|
|||Single core running 128-bit data access instructions, the<br>other core in idle state|54.4|69.2|
|||Dual core running128-bit data access instructions|66.7|81.1|
|||WAITI|32.9|47.6|
|||Single core running 32-bit data access instructions, the<br>other core in idle state|51.2|65.9|
||240|Dual core running32-bit data access instructions|66.2|81.3|
|||Single core running 128-bit data access instructions, the<br>other core in idle state|72.4|87.9|
|||Dual core running128-bit data access instructions|91.7|107.9|

> 1 Current consumption when all peripheral clocks are disabled. 

> 2 Current consumption when all peripheral clocks are enabled. In practice, the current consumption might be different depending on which peripherals are enabled. 

> 3 In Modem-sleep mode, Wi-Fi is clock gated, and the current consumption might be higher when accessing flash. For a flash rated at 80 Mbit/s, in SPI 2-line mode the consumption is 10 mA. 

Espressif Systems 

29 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

5 Peripherals 

Table 6-7. Current Consumption in Low-Power Modes 

|Work mode|Description|Typ (_µ_A)|
|---|---|---|
|Light-sleep<sup>1</sup>|VDD_SPI and Wi-Fi are powered down, and all GPIOs are high-impedance.|240|
||The ULP co-processor<br><br>ULP-FSM|170|
||is powered on<sup>2</sup><br>ULP-RISC-V<br>|190|
|Deep-sleep|ULP sensor-monitored pattern<sup>3</sup>|18|
||RTC memory and RTC peripherals are powered up.|8|
||RTC memory is powered up. RTC peripherals are powered down.|7|
|Power off|EN is set to low level. The chip is shut down.|1|

- 1 In Light-sleep mode, all related SPI pins are pulled up. For chips embedded with PSRAM, please add corresponding PSRAM consumption values, e.g., 140 _µ_ A for 8 MB 8-line PSRAM (3.3 V), 200 _µ_ A for 8 MB 8-line PSRAM (1.8 V) and 40 _µ_ A for 2 MB 4-line PSRAM (3.3 V). 

- 2 During Deep-sleep, when the ULP co-processor is powered on, peripherals such as GPIO and I2C are able to operate. 

- 3 The “ULP sensor-monitored pattern” refers to the mode where the ULP coprocessor or the sensor works periodically. When touch sensors work with a duty cycle of 1%, the typical current consumption is 18 _µ_ A. 

### 6.5 Memory Specifications 

The data below is sourced from the memory vendor datasheet. These values are guaranteed through design and/or characterization but are not fully tested in production. Devices are shipped with the memory erased. 

Table 6-8. Flash Specifications 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|VCC|Power supply voltage (1.8 V)|1.65|1.80|2.00|V|
||Power supply voltage (3.3 V)|2.7|3.3|3.6|V|
|F_C_|Maximum clock frequency|80|—|—|MHz|
|—|Program/erase cycles|100,000|—|—|cycles|
|T_RET_|Data retention time|20|—|—|years|
|T_P P_|Page program time|—|0.8|5|ms|
|T_SE_|Sector erase time (4 KB)|—|70|500|ms|
|T_BE_1|Block erase time (32 KB)|—|0.2|2|s|
|T_BE_2|Block erase time (64 KB)|—|0.3|3|s|
||Chip erase time (16 Mb)|—|7|20|s|
||Chip erase time (32 Mb)|—|20|60|s|
|T_CE_|Chip erase time (64 Mb)|—|25|100|s|
||Chip erase time (128 Mb)|—|60|200|s|
||Chip erase time (256 Mb)|—|70|300|s|

Espressif Systems 

30 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

5 Peripherals 

Table 6-9. PSRAM Specifications 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|VCC|Power supply voltage (1.8 V)|1.62|1.80|1.98|V|
||Power supply voltage (3.3 V)|2.7|3.3|3.6|V|
|F_C_|Maximum clock frequency|80|—|—|MHz|

Espressif Systems 

31 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

5 Peripherals 

## 7 RF Characteristics 

This section contains tables with RF characteristics of the Espressif product. 

The RF data is measured at the antenna port, where RF cable is connected, including the front-end loss. The external antennas used for the tests on the modules with external antenna connectors have an impedance of 50 Ω. 

Devices should operate in the center frequency range allocated by regional regulatory authorities. The target center frequency range and the target transmit power are configurable by software. See <u>ESP RF Test Tool and Test Guide</u> for instructions. 

Unless otherwise stated, the RF tests are conducted with a 3.3 V (±5%) supply at 25 ºC ambient temperature. 

### 7.1 Wi-Fi Radio 

Table 7-1. Wi-Fi RF Characteristics 

|Name|Description|
|---|---|
|Center frequency range of operatingchannel|2412~2484 MHz|
|Wi-Fi wireless standard|IEEE 802.11b/g/n|

#### 7.1.1 Wi-Fi RF Transmitter (TX) Characteristics 

Table 7-2. TX Power with Spectral Mask and EVM Meeting 802.11 Standards 

|Rt|Min|Typ|Max|
|---|---|---|---|
|ae|(dBm)|(dBm)|(dBm)|
|802.11b, 1 Mbps|—|20.5|—|
|802.11b, 11 Mbps|—|20.5|—|
|802.11g, 6 Mbps|—|20.0|—|
|802.11g, 54 Mbps|—|18.0|—|
|802.11n, HT20, MCS 0|—|19.0|—|
|802.11n, HT20, MCS 7|—|17.5|—|
|802.11n, HT40, MCS 0|—|18.5|—|
|802.11n, HT40, MCS 7|—|17.0|—|

Table 7-3. TX EVM Test<sup>1</sup> 

|Rate|Min|Typ|Limit|
|---|---|---|---|
||(dB)|(dB)|(dB)|
|802.11b, 1 Mbps, @20.5 dBm|—|–24.5|–10|
|802.11b, 11 Mbps, @20.5 dBm|—|–24.5|–10|
|802.11g, 6 Mbps, @20 dBm|—|–23.0|–5|
|802.11g, 54 Mbps, @18 dBm|—|–29.5|–25|

Cont’d on next page 

Espressif Systems 

32 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

5 Peripherals 

Table 7-3 – cont’d from previous page 

|Rate|Min<br>(dB)|Typ<br>(dB)|Limit<br>(dB)|
|---|---|---|---|
|802.11n, HT20, MCS 0, @19 dBm|—|–24.0|–5|
|802.11n, HT20, MCS 7, @17.5 dBm|—|–30.5|–27|
|802.11n, HT40, MCS 0, @18.5 dBm|—|–25.0|–5|
|802.11n, HT40, MCS 7, @17 dBm|—|–30.0|–27|

- 1 EVM is measured at the corresponding typical TX power provided in Table 7-2 _TX Power with Spectral Mask and EVM Meeting 802.11 Standards_ above. 

#### 7.1.2 Wi-Fi RF Receiver (RX) Characteristics 

For RX tests, the PER (packet error rate) limit is 8% for 802.11b, and 10% for 802.11g/n. 

Table 7-4. RX Sensitivity 

|Rate|Min<br>(dBm)|Typ<br>(dBm)|Max<br>(dBm)|
|---|---|---|---|
|802.11b, 1 Mbps|—|–98.2|—|
|802.11b, 2 Mbps|—|–95.6|—|
|802.11b, 5.5 Mbps|—|–92.8|—|
|802.11b, 11 Mbps|—|–88.5|—|
|802.11g, 6 Mbps|—|–93.0|—|
|802.11g, 9 Mbps|—|–92.0|—|
|802.11g, 12 Mbps|—|–90.8|—|
|802.11g, 18 Mbps|—|–88.5|—|
|802.11g, 24 Mbps|—|–85.5|—|
|802.11g, 36 Mbps|—|–82.2|—|
|802.11g, 48 Mbps|—|–78.0|—|
|802.11g, 54 Mbps|—|–76.2|—|
|802.11n, HT20, MCS 0|—|–93.0|—|
|802.11n, HT20, MCS 1|—|–90.6|—|
|802.11n, HT20, MCS 2|—|–88.4|—|
|802.11n, HT20, MCS 3|—|–84.8|—|
|802.11n, HT20, MCS 4|—|–81.6|—|
|802.11n, HT20, MCS 5|—|–77.4|—|
|802.11n, HT20, MCS 6|—|–75.6|—|
|802.11n, HT20, MCS 7|—|–74.2|—|
|802.11n, HT40, MCS 0|—|–90.0|—|
|802.11n, HT40, MCS 1|—|–87.5|—|
|802.11n, HT40, MCS 2|—|–85.0|—|
|802.11n, HT40, MCS 3|—|–82.0|—|
|802.11n, HT40, MCS 4|—|–78.5|—|
|802.11n, HT40, MCS 5|—|–74.4|—|

Cont’d on next page 

Espressif Systems 

33 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

5 Peripherals 

Table 7-4 – cont’d from previous page 

|Rate|Min<br>(dBm)|Typ<br>(dBm)|Max<br>(dBm)|
|---|---|---|---|
|802.11n, HT40, MCS 6|—|–72.5|—|
|802.11n, HT40, MCS 7|—|–71.2|—|

Table 7-5. Maximum RX Level 

|Rt|Min|Typ|Max|
|---|---|---|---|
|ae|(dBm)|(dBm)|(dBm)|
|802.11b, 1 Mbps|—|5|—|
|802.11b, 11 Mbps|—|5|—|
|802.11g, 6 Mbps|—|5|—|
|802.11g, 54 Mbps|—|0|—|
|802.11n, HT20, MCS 0|—|5|—|
|802.11n, HT20, MCS 7|—|0|—|
|802.11n, HT40, MCS 0|—|5|—|
|802.11n, HT40, MCS 7|—|0|—|

Table 7-6. RX Adjacent Channel Rejection 

|Rate|Min<br>(dB)|Typ<br>(dB)|Max<br>(dB)|
|---|---|---|---|
|802.11b, 1 Mbps|—|35|—|
|802.11b, 11 Mbps|—|35|—|
|802.11g, 6 Mbps|—|31|—|
|802.11g, 54 Mbps|—|14|—|
|802.11n, HT20, MCS 0|—|31|—|
|802.11n, HT20, MCS 7|—|13|—|
|802.11n, HT40, MCS 0|—|19|—|
|802.11n, HT40, MCS 7|—|8|—|

### 7.2 Bluetooth LE Radio 

Table 7-7. Bluetooth LE RF Characteristics 

|Name|Description|
|---|---|
|Center frequency range of operatingchannel|2402~2480 MHz|
|RF transmit power range|–24.0~20.0 dBm|

Espressif Systems 

34 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

5 Peripherals 

#### 7.2.1 Bluetooth LE RF Transmitter (TX) Characteristics 

Table 7-8. Bluetooth LE - Transmitter Characteristics - 1 Mbps 

|Parameter<br>Description|Min|Typ|Max|Unit|
|---|---|---|---|---|
|Max _|fn|n_=0_,_ 1_,_ 2_, ..k_|—|2.50|—|kHz|
|Carrier freenc offset and drift<br>Max _|f_0 _−fn|_|—|2.00|—|kHz|
|quy<br>Max _|fn −fn−_5_|_|—|1.40|—|kHz|
|_|f_1 _−f_0_|_|—|1.00|—|kHz|
|∆_f_1avg|—|249.00|—|kHz|
|Modulation characteristics<br>Min∆_f_2max (for at least<br>99.9% of all∆_f_2max)|—|198.00|—|kHz|
|∆_f_2avg/∆_f_1avg|—|0.86|—|—|
|±2 MHz offset|—|–37.00|—|dBm|
|In-band spurious emissions<br>±3 MHz offset|—|–42.00|—|dBm|
|>±3 MHz offset|—|–44.00|—|dBm|

Table 7-9. Bluetooth LE - Transmitter Characteristics - 2 Mbps 

|Parameter<br>Description|Min|Typ|Max|Unit|
|---|---|---|---|---|
|Max _|fn|n_=0_,_ 1_,_ 2_, ..k_|—|2.50|—|kHz|
|Ci f fft d dift<br>Max _|f_0 _−fn|_|—|2.00|—|kHz|
|arrer requency ose an r<br>Max _|fn −fn−_5_|_|—|1.40|—|kHz|
|_|f_1 _−f_0_|_|—|1.00|—|kHz|
|∆_f_1avg|—|499.00|—|kHz|
|Modulation characteristics<br>Min∆_f_2max (for at least<br>99.9% of all∆_f_2max)|—|416.00|—|kHz|
|∆_f_2avg/∆_f_1avg|—|0.89|—|—|
|±4 MHz offset|—|–42.00|—|dBm|
|In-band spurious emissions<br>±5 MHz offset|—|–44.00|—|dBm|
|>±5 MHz offset|—|–47.00|—|dBm|

Table 7-10. Bluetooth LE - Transmitter Characteristics - 125 Kbps 

|Parameter<br>Description|Min|Typ|Max|Unit|
|---|---|---|---|---|
|Max _|fn|n_=0_,_ 1_,_ 2_, ..k_|—|0.80|—|kHz|
|Ci f fft d dift<br>Max _|f_0 _−fn|_|—|1.00|—|kHz|
|arrer requency ose an r<br>_|fn −fn−_3_|_|—|0.30|—|kHz|
|_|f_0 _−f_3_|_|—|1.00|—|kHz|
|∆_f_1avg|—|248.00|—|kHz|
|Modulation characteristics<br>Min∆_f_1max (for at least<br>99.9% of all∆_f_1max)|—|222.00|—|kHz|
|±2 MHz offset|—|–37.00|—|dBm|
|In-band spurious emissions<br>±3 MHz offset|—|–42.00|—|dBm|
|>±3 MHz offset|—|–44.00|—|dBm|

Espressif Systems 

35 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

5 Peripherals 

Table 7-11. Bluetooth LE - Transmitter Characteristics - 500 Kbps 

|Parameter<br>|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
||Max _|fn|n_=0_,_ 1_,_ 2_, ..k_|—|0.80|—|kHz|
|Ci f fft d dift<br>|Max _|f_0 _−fn|_|—|1.00|—|kHz|
|arrer requency ose an r<br>|_|fn −fn−_3_|_|—|0.85|—|kHz|
||_|f_0 _−f_3_|_|—|0.34|—|kHz|
||∆_f_2avg|—|213.00|—|kHz|
|Modulation characteristics<br><br>|Min∆_f_2max (for at least<br>99.9% of all∆_f_2max)|—|196.00|—|kHz|
||±2 MHz offset|—|–37.00|—|dBm|
|In-band spurious emissions<br>|±3 MHz offset|—|–42.00|—|dBm|
||>±3 MHz offset|—|–44.00|—|dBm|

#### 7.2.2 Bluetooth LE RF Receiver (RX) Characteristics 

Table 7-12. Bluetooth LE - Receiver Characteristics - 1 Mbps 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|Sensitivity @30.8% PER|—|—|–96.5|—|dBm|
|Maximum received signal @30.8% PER|—|—|8|—|dBm|
|Co-channel C/I|F = F0 MHz|—|8|—|dB|
||F = F0 + 1 MHz|—|4|—|dB|
||F = F0 – 1 MHz|—|4|—|dB|
||F = F0 + 2 MHz|—|–23|—|dB|
|Adt hl ltiit C/I|F = F0 – 2 MHz|—|–23|—|dB|
|jacen canne seecvy|F = F0 + 3 MHz|—|–34|—|dB|
||F = F0 – 3 MHz|—|–34|—|dB|
||F_>_F0 + 3 MHz|—|–36|—|dB|
||F_>_F0 – 3 MHz|—|–37|—|dB|
|Image frequency|—|—|–36|—|dB|
|hl|F = F_image_ + 1 MHz|—|–39|—|dB|
|Adjacent canne to image frequency|F = F_image_ – 1 MHz|—|–34|—|dB|
||30 MHz~2000 MHz|—|–12|—|dBm|
||2003 MHz~2399 MHz|—|–18|—|dBm|
|Out-of-band blocking performance|2484 MHz~2997 MHz|—|–16|—|dBm|
||3000 MHz~12.75 GHz|—|–10|—|dBm|
|Intermodulation|—|—|–29|—|dBm|

Table 7-13. Bluetooth LE - Receiver Characteristics - 2 Mbps 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|Sensitivity @30.8% PER|—|—|–92|—|dBm|
|Maximum received signal @30.8% PER|—|—|3|—|dBm|
||||Con|t’d on ne|xt page|

Espressif Systems 

36 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

5 Peripherals 

Table 7-13 – cont’d from previous page 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|Co-channel C/I|F = F0 MHz|—|8|—|dB|
||F = F0 + 2 MHz|—|4|—|dB|
||F = F0 – 2 MHz|—|4|—|dB|
||F = F0 + 4 MHz|—|–27|—|dB|
|d hl l|F = F0 – 4 MHz|—|–27|—|dB|
|Ajacent canne seectivity C/I|F = F0 + 6 MHz|—|–38|—|dB|
||F = F0 – 6 MHz|—|–38|—|dB|
||F_>_F0 + 6 MHz|—|–41|—|dB|
||F_>_F0 – 6 MHz|—|–41|—|dB|
|Image frequency|—|—|–27|—|dB|
|Ad hl  i f|F = F_image_ + 2 MHz|—|–38|—|dB|
|jacent canne to mage requency|F = F_image_ – 2 MHz|—|4|—|dB|
||30 MHz~2000 MHz|—|–15|—|dBm|
|bd blk|2003 MHz~2399 MHz|—|–21|—|dBm|
|Out-of-an ocing performance|2484 MHz~2997 MHz|—|–21|—|dBm|
||3000 MHz~12.75 GHz|—|–9|—|dBm|
|Intermodulation|—|—|–29|—|dBm|

Table 7-14. Bluetooth LE - Receiver Characteristics - 125 Kbps 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|Sensitivity @30.8% PER|—|—|–103.5|—|dBm|
|Maximum received signal @30.8% PER|—|—|8|—|dBm|
|Co-channel C/I|F = F0 MHz|—|4|—|dB|
||F = F0 + 1 MHz|—|1|—|dB|
||F = F0 – 1 MHz|—|2|—|dB|
||F = F0 + 2 MHz|—|–26|—|dB|
|Ad hl lii C/I|F = F0 – 2 MHz|—|–26|—|dB|
|jacent canne seectvty|F = F0 + 3 MHz|—|–36|—|dB|
||F = F0 – 3 MHz|—|–39|—|dB|
||F_>_F0 + 3 MHz|—|–42|—|dB|
||F_>_F0 – 3 MHz|—|–43|—|dB|
|Image frequency|—|—|–42|—|dB|
|Adt hl t i f|F = F_image_ + 1 MHz|—|–43|—|dB|
|jacen canne o mage requency|F = F_image_ – 1 MHz|—|–36|—|dB|

Table 7-15. Bluetooth LE - Receiver Characteristics - 500 Kbps 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
|Sensitivity @30.8% PER|—|—|–100|—|dBm|
|Maximum received signal @30.8% PER|—|—|8|—|dBm|
|Co-channel C/I|F = F0 MHz|—|4|—|dB|

Cont’d on next page 

Espressif Systems 

37 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

5 Peripherals 

Table 7-15 – cont’d from previous page 

|Parameter|Description|Min|Typ|Max|Unit|
|---|---|---|---|---|---|
||F = F0 + 1 MHz|—|1|—|dB|
||F = F0 – 1 MHz|—|0|—|dB|
||F = F0 + 2 MHz|—|–24|—|dB|
|Adt hl ltiit C/I|F = F0 – 2 MHz|—|–24|—|dB|
|jacen canne seecvy|F = F0 + 3 MHz|—|–37|—|dB|
||F = F0 – 3 MHz|—|–39|—|dB|
||F_>_F0 + 3 MHz|—|–38|—|dB|
||F_>_F0 – 3 MHz|—|–42|—|dB|
|Image frequency|—|—|–38|—|dB|
|Adt hl t i f|F = F_image_ + 1 MHz|—|–42|—|dB|
|jacen canne o mage requency|F = F_image_ – 1 MHz|—|–37|—|dB|

Espressif Systems 

38 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

## 8 Module Schematics 

This is the reference design of the module. For modules with PSRAM, the VDD_SPI voltage is fixed to 3.3 V or 1.8 V via eFuse, so their VDD_SPI voltage will not be affected by the GPIO45 level. However, for other modules, please ensure that GPIO45 is not pulled high when the module is powered up by the external circuit. 

Figure 8-1. ESP32-S3-WROOM-1 Schematics 

Figure 8-2. ESP32-S3-WROOM-1U Schematics 

9 Peripheral Schematics 

## 9 Peripheral Schematics 

This is the typical application circuit of the module connected with peripheral components (for example, power supply, antenna, reset button, JTAG interface, and UART interface). 

Figure 9-1. Peripheral Schematics 

- Soldering the EPAD to the ground of the base board is not a must, however, it can optimize thermal performance. If you choose to solder it, please apply the correct amount of soldering paste. Too much soldering paste may increase the gap between the module and the baseboard. As a result, the adhesion between other pins and the baseboard may be poor. 

- To ensure that the power supply to the ESP32-S3 chip is stable during power-up, it is advised to add an RC delay circuit at the EN pin. The recommended setting for the RC delay circuit is usually R = 10 kΩ and C = 1 _µ_ F. However, specific parameters should be adjusted based on the power-up timing of the module and the power-up and reset sequence timing of the chip. For ESP32-S3’s power-up and reset sequence timing diagram, please refer Section 4.5 _Chip Power-up and Reset_ . 

Espressif Systems 

41 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

10 Physical Dimensions 

## 10 Physical Dimensions 

### 10.1 Module Dimensions 

~~Figure 1~~ 0-2. ESP32-S3-WROOM-1U Physical Dimensio ~~ns~~ 

Note: 

For information about tape, reel, and product marking, please refer to _ESP32-S3 Module Packaging Information_ . 

Espressif Systems 

42 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

10 Physical Dimensions 

### 10.2 Dimensions of External Antenna Connector 

ESP32-S3-WROOM-1U uses the first generation external antenna connector as shown in Figure 10-3 _Dimensions of External Antenna Connector_ . This connector is compatible with the following connectors: 

- U.FL Series connector from Hirose 

- MHF I connector from I-PEX 

- AMC connector from Amphenol 

Figure 10-3. Dimensions of External Antenna Connector 

Espressif Systems 

43 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

10 Physical Dimensions 

The external antenna used for ESP32-S3-WROOM-1U during certification testing is the first generation monopole antenna, with material code TFPD05H08750011. 

The module does not include an external antenna upon shipment. If needed, select a suitable external antenna based on the product’s usage environment and performance requirements. 

It is recommended to select an antenna that meets the following requirements: 

- 2.4 GHz band 

- 50 Ω impedance 

- The maximum gain does not exceed 2.33 dBi, the gain of the antenna used for certification 

- The connector matches the specifications shown in Figure 10-3 _Dimensions of External Antenna Connector_ 

###### Note: 

If you use an external antenna of a different type or gain, additional testing, such as EMC, may be required beyond the existing antenna test reports for Espressif modules. Specific requirements depend on the certification type. 

Espressif Systems 

44 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

11 PCB Layout Recommendations 

## 11 PCB Layout Recommendations 

### 11.1 PCB Land Pattern 

This section provides the following resources for your reference: 

- Figures for recommended PCB land patterns with all the dimensions needed for PCB design. See Figure 11-1 _ESP32-S3-WROOM-1 Recommended PCB Land Pattern_ and Figure 11-2 _ESP32-S3-WROOM-1U Recommended PCB Land Pattern_ . 

- Source files of recommended PCB land patterns to measure dimensions not covered in Figure 11-1 and Figure 11-2. You can view the source files for <u>ESP32-S3-WROOM-1</u> and <u>ESP32-S3-WROOM-1U</u> with <u>Autodesk Viewer.</u> 

- 3D models of <u>ESP32-S3-WROOM-1</u> and <u>ESP32-S3-WROOM-1U. Please make sure that you download the</u> 3D model file in .STEP format (beware that some browsers might add .txt). 

Figure 11-1. ESP32-S3-WROOM-1 Recommended PCB Land Pattern 

Espressif Systems 

45 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

11 PCB Layout Recommendations 

Figure 11-2. ESP32-S3-WROOM-1U Recommended PCB Land Pattern 

### 11.2 Module Placement for PCB Design 

If module-on-board design is adopted, attention should be paid while positioning the module on the base board. The interference of the base board on the module’s antenna performance should be minimized. 

For details about module placement for PCB design, please refer to _<u>ESP32-S3 Hardware Design Guidelines</u>_ > Section _General Principles of PCB Layout for Modules_ . 

Espressif Systems 

46 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

12 Product Handling 

## 12 Product Handling 

### 12.1 Storage Conditions 

The products sealed in moisture barrier bags (MBB) should be stored in a non-condensing atmospheric environment of < 40 °C and 90%RH. The module is rated at the moisture sensitivity level (MSL) of 3. 

After unpacking, the module must be soldered within 168 hours with the factory conditions 25±5 °C and 60%RH. If the above conditions are not met, the module needs to be baked. 

### 12.2 Electrostatic Discharge (ESD) 

- Human body model (HBM): ±2000 V 

- Charged-device model (CDM): ±500 V 

### 12.3 Reflow Profile 

Solder the module in a single reflow. 

Figure 12-1. Reflow Profile 

Espressif Systems 

47 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

12 Product Handling 

### 12.4 Ultrasonic Vibration 

Avoid exposing Espressif modules to vibration from ultrasonic equipment, such as ultrasonic welders or ultrasonic cleaners. This vibration may induce resonance in the in-module crystal and lead to its malfunction or even failure. As a consequence, the module may stop working or its performance may deteriorate. 

Espressif Systems 

48 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

<u>Submit Documentation Feedback</u> 

_Datasheet Versioning_ 

## Datasheet Versioning 

|Datasheet<br>Version|Status|Watermark|Definition|
|---|---|---|---|
|v0.1 ~ v0.5<br>(excluding v0.5)|Draft|Confidential|This datasheet is under development for products<br>in the design stage. Specifications may change<br>without prior notice.|
|v0.5 ~ v1.0<br>(excluding v1.0)|Preliminary<br>release|Preliminary|This datasheet is actively updated for products in<br>the verification stage. Specifications may change<br>before mass production, and the changes will be<br>documentation in the datasheet’s Revision History.|
|v1.0 and higher|Official release|—|This datasheet is publicly released for products in<br>mass production. Specifications are finalized, and<br>major changes will be communicated via<br>Product<br>Change<br>Notifications<br>(PCN).|
|Any version|—|Not<br>Recommended<br>for New Design<br>(NRND)<sup>1</sup>|This datasheet is updated less frequently for<br>products not recommended for new designs.|
|Any version|—|End of Life<br>(EOL)<sup>2</sup>|This datasheet is no longer mtained for products<br>that have reached end of life.|

> 1 Watermark will be added to the datasheet title page only when all the product variants covered by this datasheet are not recommended for new designs. 

> 2 Watermark will be added to the datasheet title page only when all the product variants covered by this datasheet have reached end of life. 

Espressif Systems 

49 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

_Related Documentation and Resources_ 

## Related Documentation and Resources 

### Related Documentation 

- <u>ESP32-S3 Series Datasheet</u> – Specifications of the ESP32-S3 hardware. 

- <u>ESP32-S3 Technical Reference Manual</u> – Detailed information on how to use the ESP32-S3 memory and peripherals. 

- <u>ESP32-S3 Hardware Design Guidelines</u> – Guidelines on how to integrate the ESP32-S3 into your hardware product. 

- <u>ESP32-S3 Series SoC Errata</u> – Descriptions of known errors in ESP32-S3 series of SoCs. 

- _Certificates_ 

<u>https://espressif.com/en/support/documents/certifcatesi</u> 

- _ESP32-S3 Product/Process Change Notifications (PCN)_ 

<u>https://espressif.com/en/support/documents/pcns?keys=ESP32-S3</u> 

- _ESP32-S3 Advisories_ – Information on security, bugs, compatibility, component reliability. <u>https://espressif.com/en/support/documents/advisories?keys=ESP32-S3</u> 

- _Documentation Updates and Update Notification Subscription_ <u>https://espressif.com/en/support/download/documents</u> 

### Developer Zone 

- <u>ESP-IDF Programming Guide for ESP32-S3</u> – Extensive documentation for the ESP-IDF development framework. 

- _ESP-IDF_ and other development frameworks on GitHub. 

<u>https://github.com/espressif</u> 

- _ESP32 BBS Forum_ – Engineer-to-Engineer (E2E) Community for Espressif products where you can post questions, share knowledge, explore ideas, and help solve problems with fellow engineers. <u>https://esp32.com/</u> 

- _ESP-FAQ_ – A summary document of frequently asked questions released by Espressif. <u>https://espressif.com/projects/esp-faq/en/latest/index.html</u> 

- _The ESP Journal_ – Best Practices, Articles, and Notes from Espressif folks. <u>https://blog.espressif.com/</u> 

- See the tabs _SDKs and Demos_ , _Apps_ , _Tools_ , _AT Firmware_ . <u>https://espressif.com/en/support/download/sdks-demos</u> 

### Products 

- _ESP32-S3 Series SoCs_ – Browse through all ESP32-S3 SoCs. 

- <u>https://espressif.com/en/products/socs?id=ESP32-S3</u> 

- _ESP32-S3 Series Modules_ – Browse through all ESP32-S3-based modules. <u>https://espressif.com/en/products/modules?id=ESP32-S3</u> 

- _ESP32-S3 Series DevKits_ – Browse through all ESP32-S3-based devkits. <u>https://espressif.com/en/products/devkits?id=ESP32-S3</u> 

- _ESP Product Selector_ – Find an Espressif hardware product suitable for your needs by comparing or applying filters. <u>https://products.espressif.com/#/product-selector?language=en</u> 

### Contact Us 

- See the tabs _Sales Questions_ , _Technical Enquiries_ , _Circuit Schematic & PCB Design Review_ , _Get Samples_ (Online stores), _Become Our Supplier_ , _Comments & Suggestions_ . <u>https://espressif.com/en/contact-us/sales-questions</u> 

Espressif Systems 

50 ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 <u>Submit Documentation Feedback</u> 

_Revision History_ 

## Revision History 

|Date|Version|Release notes|
|---|---|---|
|2026-03-02|v1.8|_•_ Updated “Ordering Code” to “Part Number” globally<br>_•_ Updated the_Pin Assignment_part for each subsection in Section5.2_Pe-_<br>_ripheral Description_<br>_•_ Added a note in Section5.2.1.9_SD/MMC Host Controller_<br>_•_ Updated Table6-7_Current Consumption in Low-Power Modes_|
|2025-11-18|v1.7|_•_ Added Table6-5_Current Consumption for Bluetooth LE in Active Mode_|
|2025-07-25|v1.6|_•_ Added Section4.5_Chip Power-up and Reset_<br>_•_ Added Section6.5_Memory Specifications_<br>_•_ Added the external antenna information for certification in Section 10.2<br>_Dimensions of External Antenna Connector_<br>_•_ Added Section_Datasheet Versioning_<br>_•_ Other minor changes|
|2025-06-10|v1.5|_•_ Added a note about the pin mapping between the chip and the in-<br>package flash/PSRAN in Section2_Block Diagram_|
|2024-11-14|v1.4|_•_ Renamed module variants ESP32-S3-WROOM-1-N16R16V and ESP32-S3-<br>WROOM-1U-N16R16V to ESP32-S3-WROOM-1-N16R16VA and ESP32-S3-<br>WROOM-1U-N16R16VA<br>_•_ Added a reference to the chip revision information in the note in Section<br>Section1.2_Series Comparison_<br>_•_ Updated Section1.3_Applications_<br>_•_ Restructured the previous Section_Strapping Pins_as Section4_Boot Con-_<br>_figurations_<br>_•_ Added Section5.2_Peripheral Description_<br>_•_ Divided Section _Electrical Characteristics_ into Section 6 _Electrical Char-_<br>_acteristics_and Section7_RF Characteristics_with updated formatting and<br>wording<br>_•_ Divided Section_Physical Dimensions and PCB Land Pattern_into Section<br>10_Physical Dimensions_and11_PCB Layout Recommendations_and added<br>Section11.2_Module Placement for PCB Design_<br>_•_ Added the 3D model link of ESP32-S3-WROOM-1U in Section 11.1 _PCB_<br>_Land Pattern_<br>_•_ Updated Figure12-1_Reflow Profile_<br>_•_ Other minor updates to formatting and wording|

Cont’d on next page 

Espressif Systems 

51 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

<u>Submit Documentation Feedback</u> 

_Revision History_ 

##### Cont’d from previous page 

|Date|Version|Release notes|
|---|---|---|
|2023-11-24|v1.3|_•_ Added the new module variants ESP32-S3-WROOM-1-N16R16VA and<br>ESP32-S3-WROOM-1U-N16R16VA, and updated the related information<br>_•_ Updated table notes to Table1-2_ESP32-S3-WROOM-1U Series Compari-_<br>_son_<br>_•_ Updated Section4.1_Chip Boot Mode Control_<br>_•_ Updated the module schematics in Section8_Module Schematics_<br>_•_ Updated the physical dimensions figure in Section 10.1 _Module Dimen-_<br>_sions_<br>_•_ Other minor updates|
|2023-03-07|v1.2|_•_ Update Section_Strapping Pins_<br>_•_ Update Section6.4_Current Consumption Characteristics_<br>_•_ Update the minimum value of RF transmit power in Section7.2.1_Bluetooth_<br>_LE RF Transmitter (TX) Characteristics_<br>_•_ Update descriptions in Section9_Peripheral Schematics_<br>_•_ Add descriptions in Section11.1_PCB Land Pattern_<br>_•_ Update Section12.4<br>_•_ Other minor changes|
|2022-07-22|v1.1|_•_ Update Table1-1and Table1-2<br>_•_ Other minor updates|
|2022-04-21|v1.0|_•_ Update Bluetooth LE RF data<br>_•_ Update power consumption data in Table6-7<br>_•_ Add certification and test information<br>_•_ Update Section_Strapping Pins_|
|2021-10-29|v0.6|Overall update for chip evision 1|
|2021-07-19|v0.5.1|Preliminary release, for chip revision 0|

Espressif Systems 

52 <u>Submit Documentation Feedback</u> 

ESP32-S3-WROOM-1 & WROOM-1U Datasheet v1.8 

#### Disclaimer and Copyright Notice 

Information in this document, including URL references, is subject to change without notice. 

ALL THIRD PARTY’S INFORMATION IN THIS DOCUMENT IS PROVIDED AS IS WITH NO WARRANTIES TO ITS AUTHENTICITY AND ACCURACY. 

NO WARRANTY IS PROVIDED TO THIS DOCUMENT FOR ITS MERCHANTABILITY, NON-INFRINGEMENT, FITNESS FOR ANY PARTICULAR PURPOSE, NOR DOES ANY WARRANTY OTHERWISE ARISING OUT OF ANY PROPOSAL, SPECIFICATION OR SAMPLE. All liability, including liability for infringement of any proprietary rights, relating to use of information in this document is disclaimed. No licenses express or implied, by estoppel or otherwise, to any intellectual property rights are granted herein. The Wi-Fi Alliance Member logo is a trademark of the Wi-Fi Alliance. The Bluetooth logo is a registered trademark of Bluetooth SIG. All trade names, trademarks and registered trademarks mentioned in this document are property of their respective owners, and are hereby acknowledged. 

Copyright © 2026 Espressif Systems (Shanghai) Co., Ltd. All rights reserved. <u>w.espressif.comww</u> 

