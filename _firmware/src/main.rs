#![no_std]
#![no_main]

mod cube;
mod embedded_rand;
mod patterns;

use crate::{flush, patterns::MAX_BRIGHTNESS};
use common::{
    apa106led::{Apa106Led, WARM_WHITE},
    colour_functions::fade,
    cube::Cube,
};
use core::ptr;
use cortex_m::{asm::wfi, singleton};
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

#[app(device = stm32f1xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        led: PC13<Output<PushPull>>,
        spi_dma: &'static mut DmaInterface,
        timer_handler: CountDownTimer<pac::TIM1>,
        cube: Cube,
        delay: Delay,
        // timer2_handler: CountDownTimer<pac::TIM2>,
        // buf: &'static mut [u8; DATA_LEN],
    }
    // static mut SPI_DEVICE: PC13<Output<PushPull>> = ();
    // static mut TIMER_HANDLER: CountDownTimer<pac::TIM1> = ();
    // static mut DATA: [u8; (64 * 8 * 3) + 1] = [OFF_BYTE; (64 * 8 * 3) + 1];

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let cp = cortex_m::Peripherals::take().unwrap();

        // Take ownership over the raw flash and rcc devices and convert them into the corresponding
        // HAL structs
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();

        // Freeze the configuration of all the clocks in the system and store the frozen frequencies
        // in `clocks`
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .freeze(&mut flash.acr);

        // Acquire the GPIOC peripheral
        let mut gpioc = cx.device.GPIOC.split(&mut rcc.apb2);
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.apb2);

        let mut delay = Delay::new(cp.SYST, clocks);

        // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the
        // function in order to configure the port. For pins 0-7, crl should be passed instead
        let led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, State::High);

        let mut timer =
            Timer::tim1(cx.device.TIM1, &clocks, &mut rcc.apb2).start_count_down(10.hz());
        timer.listen(Event::Update);

        // let mut timer2 =
        //     Timer::tim2(cx.device.TIM2, &clocks, &mut rcc.apb1).start_count_down(1.hz());
        // timer2.listen(Event::Update);

        let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
        let miso = gpiob.pb14.into_floating_input(&mut gpiob.crh);
        let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);

        let mut spi = Spi::spi2(
            cx.device.SPI2,
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
        let dma = cx.device.DMA1.split(&mut rcc.ahb);

        // Connect the SPI device to the DMA
        let spi_dma = spi.with_tx_dma(dma.5);

        // Start a DMA transfer
        // let (_buffer, spi_dma) = unsafe { spi_dma.write(&DATA).wait() };

        let spi_dma = singleton!(: DmaInterface = spi_dma).unwrap();

        // let buf = singleton!(: [u8; DATA_LEN] = [OFF_BYTE; DATA_LEN]).unwrap();
        // unsafe { DATA[DATA.len() - 1] = 0x00 };

        let mut cube = Cube::new();

        cube.fill(Apa106Led {
            red: 2,
            green: 0,
            blue: 0,
        });

        // Init the static resources to use them later through RTFM
        init::LateResources {
            led: led,
            spi_dma: spi_dma,
            timer_handler: timer,
            cube,
            delay
            // timer2_handler: timer2,
            // buf,
        }
    }

    #[idle(resources = [ spi_dma, delay, cube ])]
    fn idle(cx: idle::Context) -> ! {
        let raindrop_colour = fade(WARM_WHITE, MAX_BRIGHTNESS as f32 / 255.0);

        let mut counter = 0;

        loop {
            // Rainbow
            for _ in 0..4 {
                patterns::christmas_rainbow(
                    cx.resources.cube,
                    cx.resources.spi_dma,
                    cx.resources.delay,
                );
            }

            // Fadey slices thing
            for _ in 0..4 {
                patterns::animated_slices(
                    cx.resources.cube,
                    cx.resources.spi_dma,
                    cx.resources.delay,
                );
            }

            // Rain
            for _ in 0..16 {
                patterns::rain(
                    cx.resources.cube,
                    cx.resources.spi_dma,
                    cx.resources.delay,
                    raindrop_colour,
                );
            }

            // Blender
            for _ in 0..16 {
                patterns::blender(
                    cx.resources.cube,
                    cx.resources.spi_dma,
                    cx.resources.delay,
                    raindrop_colour,
                );
            }
        }
    }
};

// #[task(binds = TIM1_UP, resources = [spi_dma, led, timer_handler])]
// fn frame_update(cx: frame_update::Context) {
//     // cx.resources.led.toggle();

//     use core::sync::atomic::{self, Ordering};
//     use stm32f1xx_hal::dma::TransferPayload;

//     let mut spi_dma: &mut DmaInterface = cx.resources.spi_dma;

//     unsafe {
//         spi_dma
//             .channel
//             .set_peripheral_address(unsafe { &(*SPI2::ptr()).dr as *const _ as u32 }, false);
//         spi_dma
//             .channel
//             .set_memory_address(DATA.as_ptr() as u32, true);
//         spi_dma.channel.set_transfer_length(DATA.len());
//     }
//     atomic::compiler_fence(Ordering::Release);
//     spi_dma.channel.ch().cr.modify(|_, w| {
//         w.mem2mem()
//             .clear_bit()
//             .pl()
//             .medium()
//             .msize()
//             .bits8()
//             .psize()
//             .bits8()
//             .circ()
//             .clear_bit()
//             .dir()
//             .set_bit()
//     });
//     spi_dma.start();
//     while spi_dma.channel.in_progress() {}

//     atomic::compiler_fence(Ordering::Acquire);

//     spi_dma.stop();

//     // we need a read here to make the Acquire fence effective
//     // we do *not* need this if `dma.stop` does a RMW operation
//     unsafe {
//         ptr::read_volatile(&0);
//     }

//     // we need a fence here for the same reason we need one in `Transfer.wait`
//     atomic::compiler_fence(Ordering::Acquire);

//     // Clears the update flag
//     cx.resources.timer_handler.clear_update_interrupt_flag();
// }

// #[task(binds = TIM2, resources = [led, timer2_handler])]
// fn animation_update(cx: animation_update::Context) {
//     cx.resources.led.toggle();
//     unsafe { DATA[7] = ON_BYTE };

//     let lights = [
//         Apa106Led {
//             red: 0,
//             green: 10,
//             blue: 0,
//         },
//         Apa106Led {
//             red: 10,
//             green: 0,
//             blue: 0,
//         },
//         Apa106Led {
//             red: 0,
//             green: 0,
//             blue: 10,
//         },
//     ];

//     for (led_idx, light) in lights.iter().enumerate() {
//         let start = led_idx * (8 * 3);

//         for (byte_idx, bit) in colour_to_raw(&light).iter().enumerate() {
//             unsafe { DATA[start + byte_idx] = *bit }
//         }
//     }

//     // Clears the update flag
//     cx.resources.timer2_handler.clear_update_interrupt_flag();
// }
