#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_semihosting as _;
use stm32f1xx_hal::{
    delay::Delay,
    pac,
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi},
};

const ON_BYTE: u8 = 0b1111_1100;
const OFF_BYTE: u8 = 0b1100_0000;

#[entry]
fn main() -> ! {
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .freeze(&mut flash.acr);

    // Acquire the GPIOA peripheral
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
    let miso = gpiob.pb14;
    let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);

    let spi_mode = Mode {
        polarity: Polarity::IdleHigh,
        phase: Phase::CaptureOnFirstTransition,
    };
    let mut spi = Spi::spi2(
        dp.SPI2,
        (sck, miso, mosi),
        spi_mode,
        // 4x
        // Uses a divisor of 16 to get an actual frequency of 2_225_000 which is 3.85% off this
        // value. See `freq-calc.xlsx`.
        // 2_340_000.hz(),

        // 8x (1 byte per bit value)
        // Uses a divisor of 8 to get an actual frequency of 4_500_000 which is -3.85% off this
        // value. See `freq-calc.xlsx`.
        4_680_000.hz(),
        clocks,
        &mut rcc.apb1,
    );

    // Set up the DMA device
    let dma = dp.DMA1.split(&mut rcc.ahb);

    // Connect the SPI device to the DMA
    let spi_dma = spi.with_tx_dma(dma.5);

    // 64 LEDs, 8 bits per LED plus one final stop bit to set the line low
    let data: [u8; (64 * 8 * 3) + 1] = [OFF_BYTE; (64 * 8 * 3) + 1];
    // data[data.len() - 1] = 0x00;

    // All LEDs dim red
    // for i in (0..64) {
    //     data[i * 8 * 3 + 8] = ON_BYTE;
    // }

    // Start a DMA transfer
    // let transfer = spi_dma.write(&[
    //     OFF_BYTE, OFF_BYTE, OFF_BYTE, OFF_BYTE, OFF_BYTE, OFF_BYTE, OFF_BYTE, ON_BYTE, //
    //     OFF_BYTE, OFF_BYTE, OFF_BYTE, OFF_BYTE, OFF_BYTE, OFF_BYTE, OFF_BYTE, OFF_BYTE, //
    //     OFF_BYTE, OFF_BYTE, OFF_BYTE, OFF_BYTE, OFF_BYTE, OFF_BYTE, OFF_BYTE, OFF_BYTE, //
    //     0x00,
    // ]);

    let transfer = spi_dma.write(&data);
    let (_spi_dma, _buffer) = transfer.wait();

    let mut delay = Delay::new(cp.SYST, clocks);

    loop {}
}
