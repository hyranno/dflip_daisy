#![no_main]
#![no_std]

#[macro_use]
mod macros;

mod custom;
use crate::custom::peripheral::run_rmk_split_peripheral;
use rmk_custom_device::matrix::SequentialMatrixPins;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{AnyPin, Input, Output},
    peripherals::{UART0, USB},
    uart::{self, BufferedUart},
    usb::InterruptHandler,
};
use panic_probe as _;
use rmk::split::SPLIT_MESSAGE_MAX_SIZE;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    UART0_IRQ => uart::BufferedInterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("RMK start!");
    // Initialize peripherals
    let p = embassy_rp::init(Default::default());

    // Pin config
    let pins = config_sequential_matrix_pins_rp!(
        peripherals: p,
        row_clock: PIN_9,
        col_clock: PIN_10,
        any_not: PIN_11,
        reset_not: PIN_12,
        input: PIN_13,
    );

    static TX_BUF: StaticCell<[u8; SPLIT_MESSAGE_MAX_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; SPLIT_MESSAGE_MAX_SIZE])[..];
    static RX_BUF: StaticCell<[u8; SPLIT_MESSAGE_MAX_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; SPLIT_MESSAGE_MAX_SIZE])[..];
    let uart_instance = BufferedUart::new(
        p.UART0,
        Irqs,
        p.PIN_0,
        p.PIN_1,
        tx_buf,
        rx_buf,
        uart::Config::default(),
    );

    // Start serving
    run_rmk_split_peripheral::<Input<'_>, Output<'_>, _, 2, 2>(
        pins,
        uart_instance,
    )
    .await;
}