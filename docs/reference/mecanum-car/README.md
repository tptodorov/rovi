# Mecanum car chassis and motors

Reference for the chassis kit used in the Rovi car. This records the seller listing separately from measurements of the received assembly. The listing is not a motor datasheet.

## Product identity

| Field | Listing information |
| --- | --- |
| Product | Mecanum-wheel omnidirectional robot-car chassis kit |
| AliExpress item ID | `1005002904454989` |
| Listing title | “Mecanum-Rad Omnidirektionales Roboter-Auto-Chassis-Kit mit 4 TT-Motoren für Arduino und Raspberry Pi DIY-Spielzeugteile” (German storefront title) |
| Seller-stated included drive components | Four TT motors and four mecanum wheels |
| Listing | [AliExpress product page](https://de.aliexpress.com/item/1005002904454989.html) |
| Page checked | 2026-10-03 |

The listing photos show a silver, two-level chassis plate assembly, four yellow mecanum wheels with black rollers, and four yellow TT-style geared DC motor housings. Photos are saved in [`listing-images/`](listing-images/) as a visual record of the listing, not proof that every detail matches the received kit.

## Specifications confirmed from the listing

* Chassis type: four-wheel mecanum / omnidirectional car chassis.
* Motor count and listing description: four TT motors.
* The listing title mentions Arduino and Raspberry Pi as intended DIY platforms.

The product endpoint returned an AliExpress 502 page when checked; only its listing title and image list were available. No detailed SKU/specification section could be retrieved. No voltage, no-load speed, rated or stall current, gearbox ratio, chassis dimensions, mass, or motor part number could be confirmed. Do not infer those values from the generic “TT motor” description or product photos.

## To verify on the received hardware

Record these when measured or found on packaging/labels:

* Chassis plate dimensions, material/thickness, and assembled mass.
* Motor markings/model and gearbox ratio, if present.
* Motor voltage range, no-load speed/current, and stall current from a datasheet or controlled measurement.
* Wheel diameter, roller count, roller orientation, and wheel-to-motor mapping.
* Whether the four motors and all chassis hardware match the listing photos.
* Battery source and wiring/power distribution intended for the four motors.

These values determine whether the two TB6612FNG modules in the current parts list are suitable for four-motor operation. See the separate [TB6612FNG board reference](../tb6612fng/README.md); M1 validated one motor channel only.

## Preliminary measurement

On 2026-10-03, Todor reported **120 mA** for a single motor connected to a **5 V** power bank with a stated capacity of **6500 mAh**. The motor identity, load, measurement method, and whether the reading was steady-state or startup current were not recorded. This is an observation, not a motor rating, no-load rating, or stall-current measurement.

### Conditional arithmetic only

* At the observed 5 V and 0.12 A: `P = V × I = 0.60 W` for that unrecorded operating condition.
* If all four motors each drew the same 0.12 A under the same conditions: `I₄ = 4 × 0.12 A = 0.48 A` and `P₄ = 5 V × 0.48 A = 2.4 W`. This is a linear scenario, not a measured four-motor load or a design value.
* Only if 6500 mAh represented usable capacity at the 5 V output would nominal energy be `6.5 Ah × 5 V = 32.5 Wh`; ideal arithmetic runtime would then be `32.5 Wh ÷ 0.60 W ≈ 54 h` for one motor or `32.5 Wh ÷ 2.4 W ≈ 13.5 h` for four motors at the same assumed draw. The capacity reference voltage/output rating is unknown, and conversion losses, motor load, startup current, cutoff behavior, and other car electronics are excluded. Do not use these figures to promise runtime or select the power source.

Repeat the measurements under documented conditions during [M4](../../../experiments/rovi-m4-four-motor-safety/MILESTONE.md), and confirm whether this power bank is the intended car supply.

## Source images

Images are the six product photos supplied in the AliExpress page's image list on 2026-10-03. AliExpress may change or remove listing content. They are retained here to identify the chassis style and apparent assembly, not as dimensional drawings.

* [Listing image 1](listing-images/listing-01.jpg)
* [Listing image 2](listing-images/listing-02.jpg)
* [Listing image 3](listing-images/listing-03.jpg)
* [Listing image 4](listing-images/listing-04.jpg)
* [Listing image 5](listing-images/listing-05.jpg)
* [Listing image 6](listing-images/listing-06.jpg)
