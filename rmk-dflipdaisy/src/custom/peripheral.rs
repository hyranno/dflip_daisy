#[cfg(not(feature = "rapid_debouncer"))]
use rmk::debounce::default_bouncer::DefaultDebouncer;
#[cfg(feature = "rapid_debouncer")]
use rmk::debounce::fast_debouncer::RapidDebouncer;
use rmk::debounce::DebouncerTrait;
#[cfg(feature = "_nrf_ble")]
use embassy_executor::Spawner;
use embedded_hal::digital::{InputPin, OutputPin};
#[cfg(feature = "async_matrix")]
use embedded_hal_async::digital::Wait;
#[cfg(not(feature = "_nrf_ble"))]
use embedded_io_async::{Read, Write};

use super::matrix::{SequentialMatrix, SequentialMatrixPins};


/// Run the split peripheral service.
///
/// # Arguments
///
/// * `input_pins` - input gpio pins, if `async_matrix` is enabled, the input pins should implement `embedded_hal_async::digital::Wait` trait
/// * `output_pins` - output gpio pins
/// * `central_addr` - (optional) central's BLE static address. This argument is enabled only for nRF BLE split now
/// * `peripheral_addr` - (optional) peripheral's BLE static address. This argument is enabled only for nRF BLE split now
/// * `serial` - (optional) serial port used to send peripheral split message. This argument is enabled only for serial split now
/// * `spawner`: (optional) embassy spawner used to spawn async tasks. This argument is enabled for non-esp microcontrollers
pub async fn run_rmk_split_peripheral<
    #[cfg(feature = "async_matrix")] In: Wait + InputPin,
    #[cfg(not(feature = "async_matrix"))] In: InputPin,
    Out: OutputPin,
    #[cfg(not(feature = "_nrf_ble"))] S: Write + Read,
    const ROW: usize,
    const COL: usize,
>(
    pins: SequentialMatrixPins<In, Out>,
    #[cfg(feature = "_nrf_ble")] central_addr: [u8; 6],
    #[cfg(feature = "_nrf_ble")] peripheral_addr: [u8; 6],
    #[cfg(not(feature = "_nrf_ble"))] serial: S,
    #[cfg(feature = "_nrf_ble")] spawner: Spawner,
) {
    #[cfg(feature = "rapid_debouncer")]
    let debouncer: RapidDebouncer<COL, ROW> = RapidDebouncer::new();
    #[cfg(not(feature = "rapid_debouncer"))]
    let debouncer: DefaultDebouncer<COL, ROW> = DefaultDebouncer::new();
    
    let matrix = SequentialMatrix::<
        In,
        Out,
        _,
        ROW,
        COL,
    >::new(pins, debouncer);

    #[cfg(not(feature = "_nrf_ble"))]
    rmk::split::serial::initialize_serial_split_peripheral_and_run::<_, S, ROW, COL>(
        matrix, serial,
    )
    .await;

    #[cfg(feature = "_nrf_ble")]
    rmk::split::nrf::peripheral::initialize_nrf_ble_split_peripheral_and_run::<_, ROW, COL>(
        matrix,
        central_addr,
        peripheral_addr,
        spawner,
    )
    .await;
}
