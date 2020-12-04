use crate::DmaInterface;
use common::{apa106led::Apa106Led, cube::Cube};
use core::ptr;
use stm32f1xx_hal::device::SPI2;

const ON_BYTE: u8 = 0b1111_1100;
const OFF_BYTE: u8 = 0b1100_0000;

const DATA_LEN: usize = (64 * 8 * 3) + 1;
static mut DATA: [u8; DATA_LEN] = [OFF_BYTE; DATA_LEN];

pub fn flush(cube: &Cube, spi_dma: &mut DmaInterface) {
    use core::sync::atomic::{self, Ordering};
    use stm32f1xx_hal::dma::TransferPayload;

    for (led_idx, light) in self.cube_frame.iter().enumerate() {
        let start = led_idx * (8 * 3);
        //
        for (byte_idx, bit) in colour_to_raw(&light).iter().enumerate() {
            unsafe { DATA[start + byte_idx] = *bit }
        }
    }

    // let mut spi_dma: &mut DmaInterface = cx.resources.spi_dma;

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
}

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
