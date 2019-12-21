#![no_std]
#![no_main]

use panic_semihosting as _;

use cortex_m::asm::wfi;
use rtfm::app;

use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::{
    gpio::{gpioc::PC13, Output, PushPull, State},
    pac,
    prelude::*,
    timer::{CountDownTimer, Event, Timer},
};

const ON_BYTE: u8 = 0b1111_1100;
const OFF_BYTE: u8 = 0b1100_0000;

#[app(device = stm32f1xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        spi: PC13<Output<PushPull>>,
        data: [u8; (64 * 8 * 3) + 1],
        timer_handler: CountDownTimer<pac::TIM1>,
    }
    // static mut SPI_DEVICE: PC13<Output<PushPull>> = ();
    // static mut TIMER_HANDLER: CountDownTimer<pac::TIM1> = ();
    // static mut DATA: [u8; (64 * 8 * 3) + 1] = [OFF_BYTE; (64 * 8 * 3) + 1];

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        // Take ownership over the raw flash and rcc devices and convert them into the corresponding
        // HAL structs
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();

        // Freeze the configuration of all the clocks in the system and store the frozen frequencies
        // in `clocks`
        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        // Acquire the GPIOC peripheral
        let mut gpioc = cx.device.GPIOC.split(&mut rcc.apb2);

        // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the
        // function in order to configure the port. For pins 0-7, crl should be passed instead
        let led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, State::High);
        // Configure the syst timer to trigger an update every second and enables interrupt
        let mut timer =
            Timer::tim1(cx.device.TIM1, &clocks, &mut rcc.apb2).start_count_down(1.hz());
        timer.listen(Event::Update);

        // Init the static resources to use them later through RTFM
        init::LateResources {
            spi: led,
            data: [OFF_BYTE; (64 * 8 * 3) + 1],
            timer_handler: timer,
        }
    }

    #[idle]
    fn idle(cx: idle::Context) -> ! {
        loop {
            // Waits for interrupt
            wfi();
        }
    }

    #[task(binds = TIM1_UP, resources = [spi, data, timer_handler])]
    fn tick(cx: tick::Context) {
        cx.resources.spi.toggle();

        // Clears the update flag
        cx.resources.timer_handler.clear_update_interrupt_flag();
    }
};
