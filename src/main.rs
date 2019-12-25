#![no_std]
#![no_main]

mod apa106led;
mod colour_functions;
mod cube;
mod embedded_rand;
mod patterns;

use crate::{
    apa106led::{Apa106Led, WARM_WHITE},
    colour_functions::fade,
    cube::Cube4,
    patterns::MAX_BRIGHTNESS,
};
use core::ptr;
use cortex_m::{asm::wfi, singleton};
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use panic_semihosting as _;
use rtfm::app;
use stm32f1xx_hal::{
    delay::Delay,
    device::SPI2,
    dma,
    gpio::{gpiob, gpioc::PC13, Output, PushPull, State},
    pac,
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi},
    timer::{CountDownTimer, Event, Timer},
};

type DmaInterface = stm32f1xx_hal::dma::TxDma<
    stm32f1xx_hal::spi::SpiPayload<
        SPI2,
        stm32f1xx_hal::spi::Spi2NoRemap,
        (
            stm32f1xx_hal::gpio::gpiob::PB13<
                stm32f1xx_hal::gpio::Alternate<stm32f1xx_hal::gpio::PushPull>,
            >,
            stm32f1xx_hal::gpio::gpiob::PB14<
                stm32f1xx_hal::gpio::Input<stm32f1xx_hal::gpio::Floating>,
            >,
            stm32f1xx_hal::gpio::gpiob::PB15<
                stm32f1xx_hal::gpio::Alternate<stm32f1xx_hal::gpio::PushPull>,
            >,
        ),
    >,
    stm32f1xx_hal::dma::dma1::C5,
>;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies
    // in `clocks`
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .freeze(&mut flash.acr);

    // Acquire the GPIOC peripheral
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let mut delay = Delay::new(cp.SYST, clocks);

    // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the
    // function in order to configure the port. For pins 0-7, crl should be passed instead
    let led = gpioc
        .pc13
        .into_push_pull_output_with_state(&mut gpioc.crh, State::High);

    let mut timer = Timer::tim1(dp.TIM1, &clocks, &mut rcc.apb2).start_count_down(10.hz());
    timer.listen(Event::Update);

    // let mut timer2 =
    //     Timer::tim2(dp.TIM2, &clocks, &mut rcc.apb1).start_count_down(1.hz());
    // timer2.listen(Event::Update);

    let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
    let miso = gpiob.pb14.into_floating_input(&mut gpiob.crh);
    let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);

    let mut spi = Spi::spi2(
        dp.SPI2,
        (sck, miso, mosi),
        Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        },
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

    // Start a DMA transfer
    // let (_buffer, spi_dma) = unsafe { spi_dma.write(&DATA).wait() };

    let mut spi_dma = singleton!(: DmaInterface = spi_dma).unwrap();

    // let buf = singleton!(: [u8; DATA_LEN] = [OFF_BYTE; DATA_LEN]).unwrap();
    // unsafe { DATA[DATA.len() - 1] = 0x00 };

    let mut cube = Cube4::new();

    cube.fill(Apa106Led {
        red: 2,
        green: 0,
        blue: 0,
    });

    let raindrop_colour = fade(WARM_WHITE, MAX_BRIGHTNESS as f32 / 255.0);

    let mut counter = 0;

    loop {
        // Rainbow
        for _ in 0..4 {
            patterns::christmas_rainbow(&mut cube, &mut spi_dma, &mut delay);
        }

        // Fadey slices thing
        for _ in 0..4 {
            patterns::animated_slices(&mut cube, &mut spi_dma, &mut delay);
        }

        // Rain
        for _ in 0..16 {
            patterns::rain(&mut cube, &mut spi_dma, &mut delay, raindrop_colour);
        }

        // Blender
        for _ in 0..16 {
            patterns::blender(&mut cube, &mut spi_dma, &mut delay, raindrop_colour);
        }
    }
}
