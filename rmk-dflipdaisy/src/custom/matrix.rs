use rmk::{
  debounce::{DebounceState, DebouncerTrait},
  keyboard::KEY_EVENT_CHANNEL,
  event::KeyEvent,
  matrix::{MatrixTrait, KeyState},
};
use embassy_time::{Instant, Timer};
use embedded_hal::digital::{InputPin, OutputPin};
#[cfg(feature = "async_matrix")]
use embedded_hal_async::digital::Wait;


pub struct SequentialMatrixPins<
    #[cfg(feature = "async_matrix")] In: Wait + InputPin,
    #[cfg(not(feature = "async_matrix"))] In: InputPin,
    Out: OutputPin,
> {
    row_clock: Out,
    col_clock: Out,
    any_not: Out,
    reset_not: Out,
    input: In,
}

impl <
    #[cfg(feature = "async_matrix")] In: Wait + InputPin,
    #[cfg(not(feature = "async_matrix"))] In: InputPin,
    Out: OutputPin,
> SequentialMatrixPins<In, Out> {
    pub(crate) fn new(
        row_clock: Out,
        col_clock: Out,
        any_not: Out,
        reset_not: Out,
        input: In,
    ) -> Self {
        Self {
            row_clock,
            col_clock,
            any_not,
            reset_not,
            input,
        }
    }
}


pub struct SequentialMatrix<
    #[cfg(feature = "async_matrix")] In: Wait + InputPin,
    #[cfg(not(feature = "async_matrix"))] In: InputPin,
    Out: OutputPin,
    D: DebouncerTrait,
    const ROW: usize,
    const COL: usize,
> {
    pins: SequentialMatrixPins<In, Out>,
    /// Debouncer
    debouncer: D,
    /// Key state matrix
    key_states: [[KeyState; COL]; ROW],
    /// Start scanning
    scan_start: Option<Instant>,
}

impl<
    #[cfg(feature = "async_matrix")] In: Wait + InputPin,
    #[cfg(not(feature = "async_matrix"))] In: InputPin,
    Out: OutputPin,
    D: DebouncerTrait,
    const ROW: usize,
    const COL: usize,
> SequentialMatrix<In, Out, D, ROW, COL> {
    const PROPAGATION_DELAY: u64 = 50;

    pub(crate) fn new(
        pins: SequentialMatrixPins<In, Out>,
        debouncer: D,
    ) -> Self {
        Self {
            pins,
            debouncer,
            key_states: [[KeyState::new(); COL]; ROW],
            scan_start: None,
        }
    }
}

impl<
    #[cfg(feature = "async_matrix")] In: Wait + InputPin,
    #[cfg(not(feature = "async_matrix"))] In: InputPin,
    Out: OutputPin,
    D: DebouncerTrait,
    const ROW: usize,
    const COL: usize,
> MatrixTrait for SequentialMatrix<In, Out, D, ROW, COL> {
    const ROW: usize = ROW;
    const COL: usize = COL;

    #[cfg(feature = "async_matrix")]
    async fn wait_for_key(&mut self) {
        if let Some(start_time) = self.scan_start {
            // If no key press over 1ms, stop scanning and wait for interupt
            if start_time.elapsed().as_millis() <= 1 {
                return;
            } else {
                self.scan_start = None;
            }
        }
        // First, set any_not to low
        self.pins.reset_not.set_high().ok();
        self.pins.any_not.set_low().ok();
        Timer::after_nanos(Self::PROPAGATION_DELAY).await;

        let _ = self.pins.input.wait_for_high().await;

        // Set any_not pin back to high
        self.pins.any_not.set_high().ok();

        self.scan_start = Some(Instant::now());
    }

    async fn scan(&mut self) {
        defmt::info!("Matrix scanning");
        loop {
            #[cfg(feature = "async_matrix")]
            self.wait_for_key().await;

            // Reset
            self.pins.row_clock.set_low().ok();
            self.pins.col_clock.set_low().ok();
            self.pins.any_not.set_high().ok();
            self.pins.reset_not.set_low().ok();
            Timer::after_nanos(Self::PROPAGATION_DELAY).await;
            self.pins.reset_not.set_high().ok();
            Timer::after_nanos(Self::PROPAGATION_DELAY).await;

            // Scan matrix and send report
            for row in 0..ROW {
                for col in 0..COL {
                    // Check input pins and debounce
                    let debounce_state = self.debouncer.detect_change_with_debounce(
                        row,
                        col,
                        self.pins.input.is_high().ok().unwrap_or_default(),
                        &self.key_states[row][col],
                    );

                    match debounce_state {
                        DebounceState::Debounced => {
                            self.key_states[row][col].toggle_pressed();
                            let key_state = self.key_states[row][col];

                            KEY_EVENT_CHANNEL
                                .send(KeyEvent {
                                    row: row as u8,
                                    col: col as u8,
                                    pressed: key_state.pressed,
                                })
                                .await;
                        }
                        _ => (),
                    }

                    // If there's key still pressed, always refresh the self.scan_start
                    #[cfg(feature = "async_matrix")]
                    if self.key_states[row][col].pressed {
                        self.scan_start = Some(Instant::now());
                    }

                    // Clock
                    self.pins.col_clock.set_high().ok();
                    Timer::after_nanos(Self::PROPAGATION_DELAY).await;
                    self.pins.col_clock.set_low().ok();
                    Timer::after_nanos(Self::PROPAGATION_DELAY).await;
                }
                self.pins.row_clock.set_high().ok();
                Timer::after_nanos(Self::PROPAGATION_DELAY).await;
                self.pins.row_clock.set_low().ok();
                Timer::after_nanos(Self::PROPAGATION_DELAY).await;
            }

            embassy_time::Timer::after_micros(100).await;
        }
    }

    /// Read key state at position (row, col)
    fn get_key_state(&mut self, row: usize, col: usize) -> KeyState {
        return self.key_states[row][col];
    }

    fn update_key_state(&mut self, row: usize, col: usize, f: impl FnOnce(&mut KeyState)) {
        f(&mut self.key_states[row][col]);
    }
}



pub struct OffsettedMatrix<
    M: MatrixTrait,
    const ROW_OFFSET: usize,
    const COL_OFFSET: usize,
    const ROW: usize,
    const COL: usize,
> {
    matrix: M,
}

impl<
    M: MatrixTrait,
    const ROW_OFFSET: usize,
    const COL_OFFSET: usize,
    const ROW: usize,
    const COL: usize,
> OffsettedMatrix<M, ROW_OFFSET, COL_OFFSET, ROW, COL> {
    pub(crate) fn new(
        matrix: M
    ) -> Self {
        Self {
            matrix,
        }
    }
}

impl<
    M: MatrixTrait,
    const ROW_OFFSET: usize,
    const COL_OFFSET: usize,
    const ROW: usize,
    const COL: usize,
> MatrixTrait for OffsettedMatrix<M, ROW_OFFSET, COL_OFFSET, ROW, COL> {
    const ROW: usize = ROW;
    const COL: usize = COL;

    #[cfg(feature = "async_matrix")]
    async fn wait_for_key(&mut self) {
        self.matrix.wait_for_key().await
    }

    async fn scan(&mut self) {
        self.matrix.scan().await
    }

    /// Read key state at position (row, col)
    fn get_key_state(&mut self, row: usize, col: usize) -> KeyState {
        return self.matrix.get_key_state(row - ROW_OFFSET, col - COL_OFFSET);
    }

    fn update_key_state(&mut self, row: usize, col: usize, f: impl FnOnce(&mut KeyState)) {
        self.matrix.update_key_state(row - ROW_OFFSET, col - COL_OFFSET, f);
    }
}
