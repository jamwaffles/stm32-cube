# Blue Pill Cube

## Wiring

Power the cube's +5v from a different supply - the Blue Pill/USB can't handle it.

Connect the data cable (USB green) to B15. Job done.

## Running firmware

Make sure OpenOCD is running, e.g.

```bash
openocd -f interface/stlink.cfg -f target/stm32f1x.cfg
```

then:

```bash
cd firmware
cargo run --release
```

## Simulator

Two options are available:

- `cargo run` to run the 3D version using KISS3D.
- `cargo run --bin eg` to run the `embedded-graphics` 2D simulator.

e-g-sim requires SDL2:

```bash
sudo apt install libsdl2-dev
```

## APA106 timing

The APA106 is similar to the WS2812 in that is uses a single wire, timing-based bus to operate,
however the timings are slightly different. Cycle time is 1.71us per bit, and a 1 or 0 is dictated
by the duty cycle as below:

| Description    | Time   |
| -------------- | ------ |
| 0 bit on time  | 0.35us |
| 0 bit off time | 1.36us |
| 1 bit on time  | 1.36us |
| 1 bit off time | 0.35us |

This equates to a roughly 20% duty cycle for 0 bits and an 80% duty cycle for on bits.

There are a lot of libraries out there that use finely tuned assembly routines to generate the
correct signalling, however I took the same approach as [Espruino](http://www.espruino.com/WS2811)
and used the SPI bus on the TM4C123GH6PM micro. To generate the correct waveform I use two different
nibbles (MSB sent first); `0b1000` is an "off" pulse and `0b1110` is an "on" pulse. The duty cycles
here are 25% and 75% respectively, which is close enough to the permissible timing characteristics
of the APA106 (±150ns, 11%) to not be an issue.

The cycle time for one bit is 1.71us, or ~585KHz. Because I use 4 SPI bits to transfer one APA106
data bit, that clock rate needs to be multiplied by 4, resulting in an SPI bus frequency of
**~2.33MHz**. The exact frequency is determined by the values of the SSI clock registers, which are
calculated with the formula

> SysClk / (CPSDVSR \* (1 + SCR))

In my case I'm using a `SysClk` of 80MHz so the closest CPSDVSR and SCR values I get are 2 and 16
respectively, resulting in a frequency of **2.35MHz**. This is close enough to the target frequency
that it should work fine and does for me in testing.

## Pattern ideas

Start with a blank cube. Each LED quickly fades up to full brightness, then back to a baseline
brightness, then sloooowly fades to black again after a short delay. Kind of like super stylised
non-distracting fireworks/fireflies. The pattern ends by letting every LED fade slowly to black
without lighting any new ones.
