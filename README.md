# OneWire

This crate is an OneWire-Bus implementation ontop of generic `Input-` and `OutputPins` from the [embedded-hal](https://crates.io/crates/embedded-hal).

[![Build Status](https://github.com/kellerkindt/onewire/workflows/Rust/badge.svg)](https://github.com/kellerkindt/onewire/actions?query=workflow%3ARust)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/kellerkindt/onewire)
[![Crates.io](https://img.shields.io/crates/v/onewire.svg)](https://crates.io/crates/onewire)
[![Documentation](https://docs.rs/onewire/badge.svg)](https://docs.rs/onewire)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/kellerkindt/onewire/issues/new)

# How to use

Below is an example how to create a new `Driver` instance, search for devices and read the temperature from a [DS18B20](https://datasheets.maximintegrated.com/en/ds/ds18b20.pdf).

```rust,no_run
use onewire::{Driver, ds18b20::Ds18b20, DeviceSearch, Device};
use stm32f1xx_hal::prelude::*;

fn main() -> ! {
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let mut dp = stm32f1xx_hal::pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut gpioc = dp.GPIOC.split();

    let mut delay = cp.SYST.delay(&clocks);
    let pin = gpioc.pc15.into_open_drain_output(&mut gpioc.crh);
    let mut driver = Driver::new((pin,), false);

    driver.reset(&mut delay).unwrap();

    // search for devices
    let mut search = DeviceSearch::new();
    while let Some(address) = driver.search_next(&mut search, &mut delay).unwrap() {
        match address.family_code() {
            Ds18b20::FAMILY_CODE => {
                let mut ds18b20 = Ds18b20::from_address::<()>(address).unwrap();

                // request sensor to measure temperature
                let resolution = ds18b20.measure_temperature(&mut driver, &mut delay).unwrap();

                // wait for compeltion, depends on resolution
                delay.delay_ms(resolution.time_ms());

                // read temperature
                let temperature = ds18b20.read_temperature(&mut driver, &mut delay).unwrap();
            },
            _ => {
                // unknown device type
            }
        }
    }

    loop {}
}
```

The code from the example is copy&pasted from a working project, but not tested in this specific combination.
