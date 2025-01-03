macro_rules! config_output_pin_rp {
    ($p:ident, $out_pin:ident) => {
        {
            Output::new(AnyPin::from($p.$out_pin), embassy_rp::gpio::Level::Low)
        }
    };
}

macro_rules! config_input_pin_rp {
    ($p:ident, $in_pin:ident) => {
        {
            Input::new(AnyPin::from($p.$in_pin), embassy_rp::gpio::Pull::Down)
        }
    };
}

macro_rules! config_sequential_matrix_pins_rp {
    (
        peripherals: $p:ident,
        row_clock: $row:ident,
        col_clock: $col:ident,
        any_not: $any_not:ident,
        reset_not: $reset_not:ident,
        input: $input:ident,
    ) => {
        {
            SequentialMatrixPins::new(
                config_output_pin_rp!($p, $row),
                config_output_pin_rp!($p, $col),
                config_output_pin_rp!($p, $any_not),
                config_output_pin_rp!($p, $reset_not),
                config_input_pin_rp!($p, $input),
            )
        }
    }
}