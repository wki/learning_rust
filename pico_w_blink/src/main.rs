#![no_std]
#![no_main]
#![allow(dead_code)]
/*
Wiring

 GP2, GP3 -> SDA & SCL für I2C Display
 GP26 -> Joystick h/v Analog in
 GP27 -> Joystick h/v Analog in

 */


use heapless::String;
use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, Peripheral};
use embassy_rp::gpio::{Level, Output, Pull};
use embassy_rp::adc::{Adc, Channel as AdcChannel, Config as AdcConfig, InterruptHandler as AdcInterruptHandler};
use embassy_rp::i2c::{self, Config as I2cConfig, InterruptHandler as I2cInterruptHandler};
use embassy_rp::peripherals::{DMA_CH0, PIO0, I2C1};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
    primitives::{Rectangle, Circle},
};
use embedded_graphics::primitives::{Line, PrimitiveStyle};
use sh1106::Builder;
use sh1106::prelude::GraphicsMode;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    I2C1_IRQ => I2cInterruptHandler<I2C1>;
    ADC_IRQ_FIFO => AdcInterruptHandler;
});

#[embassy_executor::task]
async fn cyw43_task(runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Program start");

    let mut p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_15, Level::Low);

    // Configure PIO and CYW43
    let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");
    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    control.init(clm).await;
    control.set_power_management(cyw43::PowerManagementMode::PowerSave).await;

    // let wifi_ssid = env!("WIFI_SSID");
    // let wifi_password = env!("WIFI_PASSWORD");

    // Configure display
    let config = I2cConfig::default();
    let i2c = i2c::I2c::new_async(p.I2C1, p.PIN_3, p.PIN_2, Irqs, config);
    let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();
    display.init().unwrap();
    display.flush().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let reverse_text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::Off)
        .build();

    Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    Line::new(Point::new(8, 32 + 16), Point::new(8 + 16, 32 + 16))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();

    Line::new(Point::new(8, 32 + 16), Point::new(8 + 8, 32))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();

    Line::new(Point::new(8 + 16, 32 + 16), Point::new(8 + 8, 32))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();

    Rectangle::with_corners(Point::new(48, 32), Point::new(48 + 16, 32 + 16))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();

    Circle::new(Point::new(88, 32), 16)
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    // Configure ADC for Joystick reading
    let mut adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let mut joystick_h = AdcChannel::new_pin(&mut p.PIN_26, Pull::None);
    let mut joystick_v = AdcChannel::new_pin(&mut p.PIN_27, Pull::None);


    let mut i:u8 = 0;
    loop {
        // BLINK LEDs
        info!("blinking...");
        led.set_high();
        control.gpio_set(0, false).await;
        Timer::after(Duration::from_secs(1)).await;

        led.set_low();
        control.gpio_set(0, true).await;
        Timer::after(Duration::from_secs(1)).await;

        // READ ADC
        let x = adc.read(&mut joystick_h).await.unwrap();
        let y = adc.read(&mut joystick_v).await.unwrap();
        info!("X: {}, Y: {}", x, y);
        let xline = convert(x);
        let yline = convert(y);



        // print things...
        // info!("i: {}", i);
        let mut line:String<5> = String::new();
        line.push((0x30 + i / 10) as char).unwrap();
        line.push((0x30 + i % 10) as char).unwrap();

        Rectangle::with_corners(Point::new(77, 14), Point::new(77 + 16, 14 + 12))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(&mut display)
            .unwrap();

        Text::with_baseline(&line, Point::new(80, 16), reverse_text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        Rectangle::with_corners(Point::new(0, 50), Point::new(0 + 127, 50+13))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
            .draw(&mut display)
            .unwrap();

        Text::with_baseline(&xline, Point::new(0, 50), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        Text::with_baseline(&yline, Point::new(40, 50), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        display.flush().unwrap();
        // info!("draw done"); // Info: Zeichen-Operation braucht ca. 0,1s

        i = if i<99 {i+1} else {0};
    }
}

fn convert(nr: u16) -> String<7> {
    let mut xline:String<7> = String::new();
    let mut div = 10000;
    let mut omit_zeros = true;
    while div >= 10 {
        let digit = (nr % div) / (div / 10);
        if div == 10 || !omit_zeros || digit > 0 {
            xline.push((0x30 + digit) as u8 as char).unwrap();
            omit_zeros = false;
        }

        div /= 10;
    }

    xline
}