# TB6612FNG datasheet (Toshiba, 2007-06-30)

> Markdown transcription of [`datasheet.pdf`](datasheet.pdf) (source: SparkFun mirror, `cdn.sparkfun.com/datasheets/Robotics/TB6612FNG.pdf`). Machine-converted; figures/diagrams dropped, H-SW truth table hand-corrected. **The PDF is authoritative.** **Level: chip — not the dev/breakout board in hand; see this folder's README for board-level docs (pins, ports differ).** Key limits: VM 4.5–13.5 V, Vcc 2.7–5.5 V, 1.0 A continuous per channel (VM ≥ 5 V), **fPWM max 100 kHz**, VIH = 0.7·Vcc.

TB6612FNG 

#### Toshiba Bi-CD Integrated Circuit  Silicon Monolithic 

# **T B 6 6 1 2 F N G** 

## Driver IC for Dual DC motor 

TB6612FNG is a driver IC for DC motor with output transistor in LD MOS structure with low ON-resistor. Two input signals, IN1 and IN2, can choose one of four modes such as CW, CCW, short brake, and stop mode. 

## **Features** 

- Power supply voltage ； VM=15V （ Max. ） 

- Output current ； Iout=1.2A(ave) / 3.2A (peak) 

- Output low ON resistor ； 0.5 Ω (upper ＋ lower Typ. @VM ≧ 5V) 

- Standby (Power save) system 

質量: 0.14 g (標準) 

- CW/CCW/short brake/stop function modes 

- Built-in thermal shutdown circuit and low voltage detecting circuit 

- Small faced package （ SSOP24 ： 0.65mm Lead pitch ） 

- Response to Pb free packaging 

- This product has a MOS structure and is sensitive to electrostatic discharge. When handling this product, ensure that the environment is protected against electrostatic discharge by using an earth strap, a conductive mat and an ionizer. Ensure also that the ambient temperature and relative humidity are maintained at reasonable levels. 

   - The TB6612FNG is a Pb-free product. The following conditions apply to solderability: *Solderability 1. Use of Sn-37Pb solder bath *solder bath temperature = 230°C *dipping time = 5 seconds *number of times = once *use of R-type flux 

   - 2. Use of Sn-3.0Ag-0.5Cu solder bath *solder bath temperature = 245°C *dipping time = 5 seconds 

1 

2007-06-30 

TB6612FNG 

## **Block Diagram** 

## **Pin Functions** 

|Pin<br>NO.<br>|Symbol<br>|I/O|Remarks|
|---|---|---|---|
|1<br>|AO1<br>|O|chA output1|
|2<br>|AO1<br>|||
|3<br>|PGND1<br>||Power GND 1|
|4<br>|PGND1<br>|⎯||
|5|AO2|||
|<br>6|<br>AO2|O|chA output2|
|7|BO2|O|chB output2|
|8<br>|BO2<br>|||
|9<br>|PGND2<br>||Power GND 2|
|10|PGND2|⎯||
|11<br>|BO1<br>|O|chB output1|
|12<br>|BO1<br>|||
|13|VM2||Motor supply（2.5V～13.5V）|
|14|VM3|⎯||
|15|PWMB|I|chB PWM input  /  200kΩ pull-down at internal|
|16<br>|BIN2<br>|I<br>|chB input2  /  200kΩ pull-down at internal<br>|
|17<br>|BIN1<br>|I<br>|chB input1  /  200kΩ pull-down at internal<br>|
|18|GND|⎯|Small signal GND|
|19|STBY|I|“L”=standby / 200kΩ pull-down at internal|
|20|Vcc|⎯|Small signal supply (2.7V～5.5V)|
|21|AIN1|I|chA input1  /  200kΩ pull-down at internal|
|22|AIN2|I|chA input2  /  200kΩ pull-down at internal|
|23<br>|PWMA<br>|I<br>|chA PWM input  /  200kΩ pull-down at internal<br>|
|24|VM1|⎯|Motor supply（2.5V～13.5V）|

2 

2007-06-30 

TB6612FNG 

## **Absolute Maximum Ratings (Ta** = **25°C)** 

|Characteristics|Symbol|Rating|Unit|Remarks|
|---|---|---|---|---|
|Supply voltage|VM|15|V||
||Vcc|6|||
|Input voltage|VIN|-0.2～6|V|IN1,IN2,STBY,PWM pins|
|Output voltage|Vout|15|V|O1,O2 pins|
||Iout|1.2||Per 1ch|
|Output current|Iout|2|A|tw=20ms Continuous pulse, Duty≦20%|
||<br>(peak)|3.2||tw=10ms Single pulse|
|||0.78||IC only|
|Power dissipation|PD|0.89|W|50×50 t=1.6(mm) Cu≧40% in PCB mounting|
|||1.36||76.2×114.3 t=1.6(mm) Cu≧30% in PCB monting|
|Operating temperature|Topr|-20～85|℃||
|Storage temperature|Tstg|-55～150|℃||

## **Operating Range** （ **Ta=-20** ～ **85** ℃） 

|Characteristics|Symbol|Min|Typ.|Max|Unit|Remarks|
|---|---|---|---|---|---|---|
||Vcc|2.7|3|5.5|V||
|Supply voltage|VM|4.5|5|13.5|V||
|Output current (H-SW)|Iout|---|---|1.0|A|VM≧5V|
|||---|---|0.4||5V＞VM≧4.5V|
|Switching frequency|fPWM|<br>---|<br>---|<br>100|kHz|<br>|

3 

2007-06-30 

TB6612FNG 

## H-SW Control Function

| IN1 | IN2 | PWM | STBY | OUT1 | OUT2 | Mode |
|---|---|---|---|---|---|---|
| H | H | H/L | H | L | L | Short brake |
| L | H | H | H | L | H | CCW |
| L | H | L | H | L | L | Short brake |
| H | L | H | H | H | L | CW |
| H | L | L | H | L | L | Short brake |
| L | L | H | H | OFF (Hi-Z) | OFF (Hi-Z) | Stop |
| H/L | H/L | H/L | L | OFF (Hi-Z) | OFF (Hi-Z) | Standby |

(Voltage-wave timing diagram omitted; see PDF p.4. Dead times t2/t4 are built in.)

## **H-SW Operating Description** 

・ To prevent penetrating current, dead time t2 and t4 is provided in switching to each mode in the IC. 

4 

2007-06-30 

TB6612FNG 

## **Electrical Characteristics (unless otherwise specified, Ta** = **25°C, Vcc** = **3V, VM=5V** ） 

|Characteristics|Symbol|Test Condition|Min|Typ.|Max|Unit|
|---|---|---|---|---|---|---|
||Icc(3V)|STBY=Vcc=3V,VM=5V|---|1.1|(1.8)|mA|
|Supply current|Icc(5.5V)|STBY=Vcc=5.5V,VM=5V|---|1.5|2.2||
||Icc(STB)|STBY=0V|---|---|1|μA|
||IM(STB)||---|---|1||
|Control input voltage|VIH||Vcc×0.7|---|Vcc+0.2|V|
||VIL||-0.2|---|Vcc×0.3||
|Control input current|IIH|VIN=3V|5|15|25|μA|
||IIL|VIN=0V|---|---|1||
|Standby input voltage|VIH(STB)||Vcc×0.7|---|Vcc+0.2|V|
||VIL(STB)||-0.2|---|Vcc×0.3||
|Standby input current|IIH(STB)|VIN=3V|5|15|25|μA|
||IIL(STB)|VIN=0V|---|---|1||
|Output saturating<br>|Vsat(U+L)1|Io=1A,Vcc=VM=5V|---|0.5|(0.7)|V|
|voltage|Vsat(U+L)2|Io=0.3A,Vcc=VM=5V||0.15|(0.21)||
|Output leakage current|IL(U)|VM=Vout=15V|---|---|1|μA|
||IL(L)|VM=15V,Vout=0V|-1|---|---||
|Regenerative diode VF|VF(U)|IF=1A|---|1|1.1|V|
|<br>|VF(L)||---|1|1.1||
|Low voltage detecting<br>voltage|UVLD|（Designed value）|---|1.9|---|V|
|Recovering voltage|UVLC||---|2.2|---||
||tr|（Designed value）|---|24|---||
||tf||---|41|---||
|Response speed|H to L<br>Dead|Penetration protect time<br>|---|(50)|---|ns|
||time<br>L to H|（Designed value）|---|(230)|---||
|Thermal shutdown<br>|||||||
|circuit operating<br>temperature<br>|TSD|（Designed value）|---|175|---|℃|
|Thermal shutdown<br>hysteresis|△TSD||---|20|---||

5 

2007-06-30 

TB6612FNG 

## **Target characteristics** 

6 

2007-06-30 

TB6612FNG 

## **Typical Application Diagram** 

Note: Condensers for noise absorption (C1, C2, C3, and C4) should be connected as close as possible to the IC. 

7 

2007-06-30 

TB6612FNG 

## **Package Dimennsions** 

Weght: 0.14 g (typ) 

8 

2007-06-30 

TB6612FNG 

## **Notes on Contents** 

### **1. Block Diagrams** 

Some of the functional blocks, circuits, or constants in the block diagram may be omitted or simplified for explanatory purposes. 

### **2. Equivalent Circuits** 

The equivalent circuit diagrams may be simplified or some parts of them may be omitted for explanatory purposes. 

### **3. Timing Charts** 

Timing charts may be simplified for explanatory purposes. 

### **4. Application Circuits** 

The application circuits shown in this document are provided for reference purposes only.  Thorough evaluation is required, especially at the mass production design stage. 

Toshiba does not grant any license to any industrial property rights by providing these examples of application circuits. 

### **5. Test Circuits** 

Components in the test circuits are used only to obtain and confirm the device characteristics. These components and circuits are not guaranteed to prevent malfunction or failure from occurring in the application equipment. 

## **IC Usage Considerations Notes on handling of ICs** 

- [1] The absolute maximum ratings of a semiconductor device are a set of ratings that must not be exceeded, even for a moment. Do not exceed any of these ratings. Exceeding the rating(s) may cause the device breakdown, damage or deterioration, and may result injury by explosion or combustion. 

- [2] Use an appropriate power supply fuse to ensure that a large current does not continuously flow in case of over current and/or IC failure. The IC will fully break down when used under conditions that exceed its absolute maximum ratings, when the wiring is routed improperly or when an abnormal pulse noise occurs from the wiring or load, causing a large current to continuously flow and the breakdown can lead smoke or ignition. To minimize the effects of the flow of a large current in case of breakdown, appropriate settings, such as fuse capacity, fusing time and insertion circuit location, are required. 

- [3] If your design includes an inductive load such as a motor coil, incorporate a protection circuit into the design to prevent device malfunction or breakdown caused by the current resulting from the inrush current at power ON or the negative current resulting from the back electromotive force at power OFF. IC breakdown may cause injury, smoke or ignition. Use a stable power supply with ICs with built-in protection functions. If the power supply is unstable, the protection function may not operate, causing IC breakdown. IC breakdown may cause injury, smoke or ignition. 

- [4] Do not insert devices in the wrong orientation or incorrectly. 

   - Make sure that the positive and negative terminals of power supplies are connected properly. Otherwise, the current or power consumption may exceed the absolute maximum rating, and exceeding the rating(s) may cause the device breakdown, damage or deterioration, and may result injury by explosion or combustion. 

In addition, do not use any device that is applied the current with inserting in the wrong orientation or incorrectly even just one time. 

9 

2007-06-30 

TB6612FNG 

### **Points to remember on handling of ICs** 

- (1) Thermal Shutdown Circuit 

Thermal shutdown circuits do not necessarily protect ICs under all circumstances. If the thermal shutdown circuits operate against the over temperature, clear the heat generation status immediately. 

Depending on the method of use and usage conditions, such as exceeding absolute maximum ratings can cause the thermal shutdown circuit to not operate properly or IC breakdown before operation. 

- (2) Heat Radiation Design 

In using an IC with large current flow such as power amp, regulator or driver, please design the device so that heat is appropriately radiated, not to exceed the specified junction temperature (TJ) at any time and condition. These ICs generate heat even during normal use. An inadequate IC heat radiation design can lead to decrease in IC life, deterioration of IC characteristics or IC breakdown. In addition, please design the device taking into considerate the effect of IC heat radiation with peripheral components. 

- (3) Back-EMF 

When a motor rotates in the reverse direction, stops or slows down abruptly, a current flow back to the motor’s power supply due to the effect of back-EMF. If the current sink capability of the power supply is small, the device’s motor power supply and output pins might be exposed to conditions beyond maximum ratings. To avoid this problem, take the effect of back-EMF into consideration in system design. 

10 

2007-06-30 

TB6612FNG 

### **RESTRICTIONS ON PRODUCT USE** 

070122EBA_R6 

- The information contained herein is subject to change without notice. 021023_D 

- TOSHIBA is continually working to improve the quality and reliability of its products. Nevertheless, semiconductor devices in general can malfunction or fail due to their inherent electrical sensitivity and vulnerability to physical stress. It is the responsibility of the buyer, when utilizing TOSHIBA products, to comply with the standards of safety in making a safe design for the entire system, and to avoid situations in which a malfunction or failure of such TOSHIBA products could cause loss of human life, bodily injury or damage to property. In developing your designs, please ensure that TOSHIBA products are used within specified operating ranges as set forth in the most recent TOSHIBA products specifications. Also, please keep in mind the precautions and conditions set forth in the “Handling Guide for Semiconductor Devices,” or “TOSHIBA Semiconductor Reliability Handbook” etc. 021023_A 

- The TOSHIBA products listed in this document are intended for usage in general electronics applications (computer, personal equipment, office equipment, measuring equipment, industrial robotics, domestic appliances, etc.). These TOSHIBA products are neither intended nor warranted for usage in equipment that requires extraordinarily high quality and/or reliability or a malfunction or failure of which may cause loss of human life or bodily injury (“Unintended Usage”). Unintended Usage include atomic energy control instruments, airplane or spaceship instruments, transportation instruments, traffic signal instruments, combustion control instruments, medical instruments, all types of safety devices, etc. Unintended Usage of TOSHIBA products listed in this document shall be made at the customer’s own risk. 021023_B 

- The products described in this document shall not be used or embedded to any downstream products of which manufacture, use and/or sale are prohibited under any applicable laws and regulations. 060106_Q 

- The information contained herein is presented only as a guide for the applications of our products. No responsibility is assumed by TOSHIBA for any infringements of patents or other rights of the third parties which may result from its use. No license is granted by implication or otherwise under any patents or other rights of TOSHIBA or the third parties. 070122_C 

- Please use this product in compliance with all applicable laws and regulations that regulate the inclusion or use of controlled substances. Toshiba assumes no liability for damage or losses occurring as a result of noncompliance with applicable laws and regulations. 060819_AF 

- The products described in this document are subject to foreign exchange and foreign trade control laws. 060925_E 

11 

2007-06-30 

