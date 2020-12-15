#![no_std]
#![no_main]

use common::{apa106led::Apa106Led, cube::Cube};
use cortex_m::singleton;
use firmware as _; // global logger + panicking-behavior + memory layout
use rtic::app;
use stm32f1xx_hal::{
    gpio::{gpioc::PC13, Output, PushPull, State},
    pac::{self, SPI2},
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

const FPS: u32 = 60;

#[app(device = stm32f1xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        status: PC13<Output<PushPull>>,
        spi_dma: &'static mut DmaInterface,
        timer: CountDownTimer<pac::TIM1>,
        cube: Cube,
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
            .into_push_pull_output_with_state(&mut gpioc.crh, State::High);

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

        let mut cube = Cube::new();

        cube.fill(Apa106Led {
            red: 2,
            green: 0,
            blue: 0,
        });

        defmt::info!("Config complete");

        init::LateResources {
            timer,
            status,
            spi_dma,
            cube,
        }
    }

    #[idle()]
    fn idle(_: idle::Context) -> ! {
        loop {
            // Fix default wfi() behaviour breaking debug probe
            core::sync::atomic::spin_loop_hint();
        }
    }

    #[task(binds = TIM1_UP, resources = [status, timer])]
    fn update(cx: update::Context) {
        let update::Resources { status, timer, .. } = cx.resources;

        status.toggle().unwrap();

        defmt::trace!("Ping");

        timer.clear_update_interrupt_flag();
    }
};
