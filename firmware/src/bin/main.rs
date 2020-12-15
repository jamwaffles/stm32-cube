#![no_std]
#![no_main]

use core::ptr;

use common::{
    apa106led::Apa106Led,
    cube::Cube,
    patterns::{Pattern, Rainbow},
};
use cortex_m::singleton;
use firmware as _; // global logger + panicking-behavior + memory layout
use rtic::app;
use stm32f1xx_hal::{
    dma::WriteDma,
    gpio::{self, gpioc::PC13, Output, PushPull},
    pac::{self, SPI2},
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi, SpiTxDma},
    timer::{CountDownTimer, Event, Timer},
};

type DmaInterface = SpiTxDma<
    SPI2,
    stm32f1xx_hal::spi::Spi2NoRemap,
    (
        stm32f1xx_hal::gpio::gpiob::PB13<
            stm32f1xx_hal::gpio::Alternate<stm32f1xx_hal::gpio::PushPull>,
        >,
        stm32f1xx_hal::gpio::gpiob::PB14<stm32f1xx_hal::gpio::Input<stm32f1xx_hal::gpio::Floating>>,
        stm32f1xx_hal::gpio::gpiob::PB15<
            stm32f1xx_hal::gpio::Alternate<stm32f1xx_hal::gpio::PushPull>,
        >,
    ),
    stm32f1xx_hal::dma::dma1::C5,
>;

// 1000 / FPS should produce an integer for better accuracy.
const FPS: u32 = 30;

#[app(device = stm32f1xx_hal::stm32, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        status: PC13<Output<PushPull>>,
        spi_dma: &'static mut DmaInterface,
        timer: CountDownTimer<pac::TIM1>,
        cube: Cube,
        state: common::State,
        #[init(0)]
        time: u32,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let dp = cx.device;

        let mut flash = dp.FLASH.constrain();
        let mut rcc = dp.RCC.constrain();

        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .freeze(&mut flash.acr);

        let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
        let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

        // let mut delay = Delay::new(core.SYST, clocks);

        let status = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, gpio::State::High);

        let mut timer = Timer::tim1(dp.TIM1, &clocks, &mut rcc.apb2).start_count_down(FPS.hz());
        timer.listen(Event::Update);

        let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
        let miso = gpiob.pb14.into_floating_input(&mut gpiob.crh);
        let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);

        let spi = Spi::spi2(
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

        let spi_dma = singleton!(: DmaInterface = spi_dma).unwrap();

        // let buf = singleton!(: [u8; DATA_LEN] = [OFF_BYTE; DATA_LEN]).unwrap();
        // unsafe { DATA[DATA.len() - 1] = 0x00 };

        let mut cube = Cube::new(8);

        cube.fill(Apa106Led {
            red: 2,
            green: 0,
            blue: 0,
        });

        let state = common::State::new(Pattern::Rainbow(Rainbow::default()));

        defmt::info!("Config complete");

        init::LateResources {
            timer,
            status,
            spi_dma,
            cube,
            state,
        }
    }

    #[idle()]
    fn idle(_: idle::Context) -> ! {
        loop {
            // Fix default wfi() behaviour breaking debug probe
            core::sync::atomic::spin_loop_hint();
        }
    }

    #[task(priority = 1, resources = [ cube, spi_dma, status ])]
    fn flush(cx: flush::Context) {
        const DATA_LEN: usize = (64 * 8 * 3) + 1;
        static mut DATA: [u8; DATA_LEN] = [0x00; DATA_LEN];

        let flush::Resources {
            mut cube,
            spi_dma,
            status,
            ..
        } = cx.resources;

        use core::sync::atomic::{self, Ordering};
        use stm32f1xx_hal::dma::TransferPayload;

        cube.lock(|cube| {
            for (led_idx, colour) in cube.frame().iter().enumerate() {
                let start = led_idx * (8 * 3);

                for (byte_idx, bit) in colour
                    .divide_by(cube.brightness_divider)
                    .as_bitbang_data()
                    .iter()
                    .enumerate()
                {
                    unsafe { DATA[start + byte_idx] = *bit }
                }
            }
        });

        // The following code is ripped straight out of the STM32F1xx lib, without all the ownership
        // stuff.

        unsafe {
            spi_dma
                .channel
                .set_peripheral_address(&(*SPI2::ptr()).dr as *const _ as u32, false);
            spi_dma
                .channel
                .set_memory_address(DATA.as_ptr() as u32, true);
            spi_dma.channel.set_transfer_length(DATA.len());
        }
        atomic::compiler_fence(Ordering::Release);
        spi_dma.channel.ch().cr.modify(|_, w| {
            w.mem2mem()
                .clear_bit()
                .pl()
                .medium()
                .msize()
                .bits8()
                .psize()
                .bits8()
                .circ()
                .clear_bit()
                .dir()
                .set_bit()
        });
        spi_dma.start();
        while spi_dma.channel.in_progress() {}

        atomic::compiler_fence(Ordering::Acquire);

        spi_dma.stop();

        // we need a read here to make the Acquire fence effective
        // we do *not* need this if `dma.stop` does a RMW operation
        unsafe {
            ptr::read_volatile(&0);
        }

        // we need a fence here for the same reason we need one in `Transfer.wait`
        atomic::compiler_fence(Ordering::Acquire);

        // status.toggle().unwrap();
    }

    #[task(binds = TIM1_UP, priority = 2, spawn = [ flush ], resources = [ timer, state, cube, time])]
    fn update(cx: update::Context) {
        let update::Resources {
            timer,
            state,
            cube,
            time,
            ..
        } = cx.resources;

        timer.clear_update_interrupt_flag();

        *time += 1000 / FPS;

        cx.spawn.flush().expect("Failed to spawn");

        state.drive(*time, cube);
    }

    extern "C" {
        fn EXTI0();
    }
};
