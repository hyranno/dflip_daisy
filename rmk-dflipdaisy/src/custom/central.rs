use embassy_executor::Spawner;
use embassy_usb::driver::Driver;
use embedded_hal::digital::{InputPin, OutputPin};
#[cfg(feature = "async_matrix")]
use embedded_hal_async::digital::Wait;
#[cfg(any(feature = "_nrf_ble", not(feature = "_no_external_storage")))]
use embedded_storage_async::nor_flash::NorFlash;

use rmk::action::KeyAction;
#[cfg(feature = "_nrf_ble")]
use rmk::ble::nrf::initialize_nrf_ble_keyboard_and_run;
use rmk::config::RmkConfig;
#[cfg(not(feature = "rapid_debouncer"))]
use rmk::debounce::default_bouncer::DefaultDebouncer;
#[cfg(feature = "rapid_debouncer")]
use rmk::debounce::fast_debouncer::RapidDebouncer;
use rmk::debounce::DebouncerTrait;
use rmk::split::central::initialize_usb_split_central_and_run;

use super::matrix::{SequentialMatrix, SequentialMatrixPins, OffsettedMatrix};

/// Run RMK split central keyboard service. This function should never return.
///
/// # Arguments
///
/// * `input_pins` - input gpio pins, if `async_matrix` is enabled, the input pins should implement `embedded_hal_async::digital::Wait` trait
/// * `output_pins` - output gpio pins
/// * `usb_driver` - (optional) embassy usb driver instance. Some microcontrollers would enable the `_no_usb` feature implicitly, which eliminates this argument
/// * `flash` - (optional) flash storage, which is used for storing keymap and keyboard configs. Some microcontrollers would enable the `_no_external_storage` feature implicitly, which eliminates this argument
/// * `default_keymap` - default keymap definition
/// * `keyboard_config` - other configurations of the keyboard, check [RmkConfig] struct for details
/// * `central_addr` - (optional) central's BLE static address. This argument is enabled only for nRF BLE split central now
/// * `spawner`: (optional) embassy spawner used to spawn async tasks. This argument is enabled for non-esp microcontrollers
#[allow(unused_variables)]
#[allow(unreachable_code)]
pub async fn run_rmk_split_central<
    #[cfg(feature = "async_matrix")] In: Wait + InputPin,
    #[cfg(not(feature = "async_matrix"))] In: InputPin,
    Out: OutputPin,
    #[cfg(not(feature = "_no_usb"))] D: Driver<'static>,
    #[cfg(not(feature = "_no_external_storage"))] F: NorFlash,
    const TOTAL_ROW: usize,
    const TOTAL_COL: usize,
    const CENTRAL_ROW: usize,
    const CENTRAL_COL: usize,
    const CENTRAL_ROW_OFFSET: usize,
    const CENTRAL_COL_OFFSET: usize,
    const NUM_LAYER: usize,
>(
    pins: SequentialMatrixPins<In, Out>,
    #[cfg(not(feature = "_no_usb"))] usb_driver: D,
    #[cfg(not(feature = "_no_external_storage"))] flash: F,
    default_keymap: &mut [[[KeyAction; TOTAL_COL]; TOTAL_ROW]; NUM_LAYER],
    keyboard_config: RmkConfig<'static, Out>,
    #[cfg(feature = "_nrf_ble")] central_addr: [u8; 6],
    #[cfg(not(feature = "_esp_ble"))] spawner: Spawner,
) -> ! {
    #[cfg(feature = "rapid_debouncer")]
    let debouncer: RapidDebouncer<CENTRAL_COL, CENTRAL_ROW> = RapidDebouncer::new();
    #[cfg(not(feature = "rapid_debouncer"))]
    let debouncer: DefaultDebouncer<CENTRAL_COL, CENTRAL_ROW> = DefaultDebouncer::new();

    let inner_matrix = SequentialMatrix::<
        In,
        Out,
        _,
        CENTRAL_ROW,
        CENTRAL_COL,
    >::new(pins, debouncer);
    let matrix = OffsettedMatrix::<
        _,
        CENTRAL_ROW_OFFSET,
        CENTRAL_COL_OFFSET,
        CENTRAL_ROW,
        CENTRAL_COL,
    >::new(inner_matrix);

    #[cfg(feature = "_nrf_ble")]
    let fut = initialize_nrf_ble_keyboard_and_run::<_, _, D, TOTAL_ROW, TOTAL_COL, NUM_LAYER>(
        matrix,
        usb_driver,
        default_keymap,
        keyboard_config,
        Some(central_addr),
        spawner,
    )
    .await;

    #[cfg(not(any(feature = "_nrf_ble", feature = "_esp_ble")))]
    let fut = initialize_usb_split_central_and_run::<_, _, D, F, TOTAL_ROW, TOTAL_COL, NUM_LAYER>(
        matrix,
        usb_driver,
        flash,
        default_keymap,
        keyboard_config,
    )
    .await;

    fut
}


