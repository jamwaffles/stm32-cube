#![no_std]
#![no_main]

use panic_semihosting as _;

use core::ptr;
use cortex_m::{asm::wfi, singleton};
use embedded_hal::digital::v2::OutputPin;
use rtfm::app;
use stm32f1xx_hal::{
    device::SPI2,
    dma,
    gpio::{gpiob, gpioc::PC13, Output, PushPull, State},
    pac,
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi},
    timer::{CountDownTimer, Event, Timer},
};

const ON_BYTE: u8 = 0b1111_1100;
const OFF_BYTE: u8 = 0b1100_0000;

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

const DATA_LEN: usize = (64 * 8 * 3) + 1;
static mut DATA: [u8; DATA_LEN] = [OFF_BYTE; DATA_LEN];

#[derive(Copy, Clone)]
pub struct Apa106Led {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[app(device = stm32f1xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        led: PC13<Output<PushPull>>,
        spi_dma: &'static mut DmaInterface,
        timer_handler: CountDownTimer<pac::TIM1>,
        timer2_handler: CountDownTimer<pac::TIM2>,
        // buf: &'static mut [u8; DATA_LEN],
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

        // Zero last data byte so line goes low after transmission
        unsafe { DATA[DATA.len() - 1] = 0x00 };

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

        // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the
        // function in order to configure the port. For pins 0-7, crl should be passed instead
        let led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, State::High);

        let mut timer =
            Timer::tim1(cx.device.TIM1, &clocks, &mut rcc.apb2).start_count_down(10.hz());
        timer.listen(Event::Update);

        let mut timer2 =
            Timer::tim2(cx.device.TIM2, &clocks, &mut rcc.apb1).start_count_down(1.hz());
        timer2.listen(Event::Update);

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
        unsafe { DATA[DATA.len() - 1] = 0x00 };

        // Init the static resources to use them later through RTFM
        init::LateResources {
            led: led,
            spi_dma: spi_dma,
            timer_handler: timer,
            timer2_handler: timer2,
            // buf,
        }
    }

    #[idle(resources = [ spi_dma])]
    fn idle(cx: idle::Context) -> ! {
        loop {
            // Waits for interrupt
            wfi();
        }
    }

    #[task(binds = TIM1_UP, resources = [spi_dma, led, timer_handler])]
    fn frame_update(cx: frame_update::Context) {
        // cx.resources.led.toggle();

        use core::sync::atomic::{self, Ordering};
        use stm32f1xx_hal::dma::TransferPayload;

        let mut spi_dma: &mut DmaInterface = cx.resources.spi_dma;

        unsafe {
            spi_dma
                .channel
                .set_peripheral_address(unsafe { &(*SPI2::ptr()).dr as *const _ as u32 }, false);
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

        // Clears the update flag
        cx.resources.timer_handler.clear_update_interrupt_flag();
    }

    #[task(binds = TIM2, resources = [led, timer2_handler])]
    fn animation_update(cx: animation_update::Context) {
        cx.resources.led.toggle();
        unsafe { DATA[7] = ON_BYTE };

        let lights = [
            Apa106Led {
                red: 0,
                green: 10,
                blue: 0,
            },
            Apa106Led {
                red: 10,
                green: 0,
                blue: 0,
            },
            Apa106Led {
                red: 0,
                green: 0,
                blue: 10,
            },
        ];

        for (led_idx, light) in lights.iter().enumerate() {
            let start = led_idx * (8 * 3);

            for (byte_idx, bit) in colour_to_raw(&light).iter().enumerate() {
                unsafe { DATA[start + byte_idx] = *bit }
            }
        }

        // Clears the update flag
        cx.resources.timer2_handler.clear_update_interrupt_flag();
    }
};

fn bit_is_set(byte: u8, bit_index: u8) -> bool {
    (byte & (1 << bit_index)) != 0
}

fn colour_to_raw(input: &Apa106Led) -> [u8; 24] {
    let mut bytes: [u8; 24] = [0; 24];

    // Gamma correct colours
    // let gamma_corrected_input = Apa106Led {
    //  red: GAMMA_MAP[input.red as usize],
    //  green: GAMMA_MAP[input.green as usize],
    //  blue: GAMMA_MAP[input.blue as usize],
    // };

    // SPI transmits MSB first
    // Gamma correction
    // for pos in 0..8 {
    //  bytes[7 - pos as usize] = if bit_is_set(gamma_corrected_input.red, pos as u8) { ON_BYTE } else { OFF_BYTE };

    //  bytes[8 + (7 - pos as usize)] = if bit_is_set(gamma_corrected_input.green, pos as u8) { ON_BYTE } else { OFF_BYTE };

    //  bytes[16 + (7 - pos as usize)] = if bit_is_set(gamma_corrected_input.blue, pos as u8) { ON_BYTE } else { OFF_BYTE };
    // }

    // No gamma correction
    for pos in 0..8 {
        bytes[7 - pos as usize] = if bit_is_set(input.red, pos as u8) {
            ON_BYTE
        } else {
            OFF_BYTE
        };

        bytes[8 + (7 - pos as usize)] = if bit_is_set(input.green, pos as u8) {
            ON_BYTE
        } else {
            OFF_BYTE
        };

        bytes[16 + (7 - pos as usize)] = if bit_is_set(input.blue, pos as u8) {
            ON_BYTE
        } else {
            OFF_BYTE
        };
    }

    bytes
}
