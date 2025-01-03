
#[cfg(feature = "_esp_ble")]
use rmk::ble::esp::initialize_esp_ble_keyboard_with_config_and_run;
#[cfg(feature = "_nrf_ble")]
use rmk::ble::nrf::initialize_nrf_ble_keyboard_and_run;
use rmk::config::RmkConfig;
#[cfg(not(feature = "rapid_debouncer"))]
use rmk::debounce::default_bouncer::DefaultDebouncer;
#[cfg(feature = "rapid_debouncer")]
use rmk::debounce::fast_debouncer::RapidDebouncer;

use rmk::action::KeyAction;
use rmk::initialize_usb_keyboard_and_run;
use rmk::debounce::DebouncerTrait;

use crate::custom::matrix::{SequentialMatrix, SequentialMatrixPins};

#[cfg(not(feature = "_esp_ble"))]
use embassy_executor::Spawner;
use embassy_usb::driver::Driver;
pub use embedded_hal;
use embedded_hal::digital::{InputPin, OutputPin};
#[cfg(feature = "async_matrix")]
use embedded_hal_async::digital::Wait;
#[cfg(any(feature = "_nrf_ble", not(feature = "_no_external_storage")))]
use embedded_storage_async::nor_flash::NorFlash as AsyncNorFlash;




/// Run RMK keyboard service. This function should never return.
///
/// # Arguments
///
/// * `input_pins` - input gpio pins, if `async_matrix` is enabled, the input pins should implement `embedded_hal_async::digital::Wait` trait
/// * `output_pins` - output gpio pins
/// * `usb_driver` - (optional) embassy usb driver instance. Some microcontrollers would enable the `_no_usb` feature implicitly, which eliminates this argument
/// * `flash` - (optional) async flash storage, which is used for storing keymap and keyboard configs. Some microcontrollers would enable the `_no_external_storage` feature implicitly, which eliminates this argument
/// * `default_keymap` - default keymap definition
/// * `keyboard_config` - other configurations of the keyboard, check [RmkConfig] struct for details
/// * `spawner`: (optional) embassy spawner used to spawn async tasks. This argument is enabled for non-esp microcontrollers
#[allow(unused_variables)]
#[allow(unreachable_code)]
pub async fn run_rmk_with_async_flash<
    #[cfg(feature = "async_matrix")] In: Wait + InputPin,
    #[cfg(not(feature = "async_matrix"))] In: InputPin,
    Out: OutputPin,
    #[cfg(not(feature = "_no_usb"))] D: Driver<'static>,
    #[cfg(not(feature = "_no_external_storage"))] F: AsyncNorFlash,
    const ROW: usize,
    const COL: usize,
    const NUM_LAYER: usize,
>(
    pins: SequentialMatrixPins<In, Out>,
    #[cfg(not(feature = "_no_usb"))] usb_driver: D,
    #[cfg(not(feature = "_no_external_storage"))] flash: F,
    default_keymap: &mut [[[KeyAction; COL]; ROW]; NUM_LAYER],
    keyboard_config: RmkConfig<'static, Out>,
    #[cfg(not(feature = "_esp_ble"))] spawner: Spawner,
) -> ! {
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

    // Dispatch according to chip and communication type
    #[cfg(feature = "_nrf_ble")]
    initialize_nrf_ble_keyboard_and_run(
        matrix,
        #[cfg(not(feature = "_no_usb"))]
        usb_driver,
        default_keymap,
        keyboard_config,
        None,
        spawner,
    )
    .await;

    #[cfg(feature = "_esp_ble")]
    initialize_esp_ble_keyboard_with_config_and_run(matrix, default_keymap, keyboard_config).await;

    #[cfg(all(
        not(feature = "_no_usb"),
        not(any(feature = "_nrf_ble", feature = "_esp_ble"))
    ))]
    initialize_usb_keyboard_and_run(
        matrix,
        usb_driver,
        #[cfg(not(feature = "_no_external_storage"))]
        flash,
        default_keymap,
        keyboard_config,
    )
    .await;

    // The fut should never return.
    // If there's no fut, the feature flags must not be correct.
    defmt::panic!("The run_rmk should never return");
}