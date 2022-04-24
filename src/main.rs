#![no_std]
#![no_main]

use cortex_m_rt::{entry, exception, ExceptionFrame};
use cortex_m_semihosting::hprintln;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use panic_halt as _;
use ssd1309::{prelude::*, Builder};
use tm4c123x_hal::{self as hal, prelude::*};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let p = hal::Peripherals::take().unwrap();

    let mut sc = p.SYSCTL.constrain();
    let mut porta = p.GPIO_PORTA.split(&sc.power_control);

    sc.clock_setup.oscillator = hal::sysctl::Oscillator::Main(
        hal::sysctl::CrystalFrequency::_16mhz,
        hal::sysctl::SystemClock::UsePll(hal::sysctl::PllOutputFrequency::_80_00mhz),
    );
    let clocks = sc.clock_setup.freeze();

    let mut res = porta.pa7.into_push_pull_output();
    let dc = porta.pa6.into_push_pull_output();
    let scl = porta.pa2.into_af_push_pull(&mut porta.control);
    let mosi = porta.pa5.into_af_push_pull(&mut porta.control);
    let miso = porta
        .pa4
        .into_af_push_pull::<hal::gpio::AF2>(&mut porta.control);

    let mode = hal::spi::MODE_0;
    let freq = 400000_u32.hz();
    let mut delay = hal::delay::Delay::new(cp.SYST, &clocks);

    let pins = (scl, miso, mosi);

    let spi = hal::spi::Spi::spi0(p.SSI0, pins, mode, freq, &clocks, &sc.power_control);

    let spi_interface = SPIInterfaceNoCS::new(spi, dc);

    let mut disp: GraphicsMode<_> = Builder::new().connect(spi_interface).into();

    disp.reset(&mut res, &mut delay).unwrap();

    disp.init().unwrap();
    disp.flush().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Eureka", Point::zero(), text_style, Baseline::Top)
        .draw(&mut disp)
        .unwrap();

    Text::with_baseline(
        "works!",
        Point::new(0, 16),
        text_style,
        Baseline::Top,
    )
    .draw(&mut disp)
    .unwrap();

    disp.flush().unwrap();

    hprintln!("done").unwrap();

    loop {}
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
