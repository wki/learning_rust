#![no_std]
#![no_main]
#![allow(dead_code)]
use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::i2c::{self, Async, Config};
use embassy_rp::peripherals::{DMA_CH0, PIO0, I2C1};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};
// use embedded_hal_1::i2c::I2c;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

#[embassy_executor::task]
async fn cyw43_task(runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>) -> ! {
    runner.run().await
}

const LCD_CLEARDISPLAY : u8 = 0x01;
const LCD_RETURNHOME : u8 = 0x02;
const LCD_ENTRYMODESET : u8 = 0x04;
const LCD_DISPLAYCONTROL : u8 = 0x08;
const LCD_CURSORSHIFT : u8 = 0x10;
const LCD_FUNCTIONSET : u8 = 0x20;
const LCD_SETCGRAMADDR : u8 = 0x40;
const LCD_SETDDRAMADDR : u8 = 0x80;
const LCD_ENTRYSHIFTINCREMENT : u8 = 0x01;
const LCD_ENTRYLEFT : u8 = 0x02;
const LCD_BLINKON : u8 = 0x01;
const LCD_CURSORON : u8 = 0x02;
const LCD_DISPLAYON : u8 = 0x04;
const LCD_MOVERIGHT : u8 = 0x04;
const LCD_DISPLAYMOVE : u8 = 0x08;
const LCD_5X10DOTS: u8 = 0x04;
const LCD_2LINE : u8 = 0x08;
const LCD_8BITMODE : u8 = 0x10;
const LCD_CHARACTER : u8 = 1;
const LCD_COMMAND : u8 = 0;
const LCD_BACKLIGHT : u8 = 0x08;
const LCD_ENABLE_BIT: u8 = 0x04;

async fn write_byte(bus: &mut i2c::I2c<'_, I2C1, Async>, byte: u8) {
    bus.write_async(0x27u8, [byte]).await.unwrap();
}

async fn toggle_enable(bus: &mut i2c::I2c<'_, I2C1, Async>, byte: u8) {
    Timer::after_millis(1).await;
    write_byte(bus, byte | LCD_ENABLE_BIT).await;

    Timer::after_millis(1).await;
    write_byte(bus, byte & !LCD_ENABLE_BIT).await;

    Timer::after_millis(1).await;
}

async fn send_byte(bus: &mut i2c::I2c<'_, I2C1, Async>, byte: u8, mode: u8) {
    let h = mode | (byte & 0xf0) | LCD_BACKLIGHT;
    let l = mode | ((byte << 4) & 0xf0) | LCD_BACKLIGHT;

    write_byte(bus, h).await;
    toggle_enable(bus, h).await;

    write_byte(bus, l).await;
    toggle_enable(bus, l).await;
}

async fn lcd_init(bus: &mut i2c::I2c<'_, I2C1, Async>) {
    send_byte(bus, 0x03, LCD_COMMAND).await;
    send_byte(bus, 0x03, LCD_COMMAND).await;
    send_byte(bus, 0x03, LCD_COMMAND).await;
    send_byte(bus, 0x02, LCD_COMMAND).await;
    send_byte(bus, LCD_ENTRYMODESET | LCD_ENTRYLEFT, LCD_COMMAND).await;
    send_byte(bus, LCD_FUNCTIONSET | LCD_2LINE, LCD_COMMAND).await;
    send_byte(bus, LCD_DISPLAYCONTROL | LCD_DISPLAYON, LCD_COMMAND).await;
}

async fn lcd_clear(bus: &mut i2c::I2c<'_, I2C1, Async>) {
    send_byte(bus, LCD_CLEARDISPLAY, LCD_COMMAND).await
}

async fn lcd_char(bus: &mut i2c::I2c<'_, I2C1, Async>, byte: u8) {
    send_byte(bus, byte, LCD_CHARACTER).await;
}

async fn lcd_cursor(bus: &mut i2c::I2c<'_, I2C1, Async>, line: u8, pos: u8) {
    let val = if line == 0 { 0x80 + pos } else { 0xC0 + pos };
    send_byte(bus, val, LCD_COMMAND).await;
}

async fn lcd_message(bus: &mut i2c::I2c<'_, I2C1, Async>, message: &str) {
    for c in message.as_bytes().iter() {
        lcd_char(bus, *c).await
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Program start");

    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_22, Level::Low);

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
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // let wifi_ssid = env!("WIFI_SSID");
    // let wifi_password = env!("WIFI_PASSWORD");




    // Configure display
    let config = Config::default();

    let mut i2c = i2c::I2c::new_async(p.I2C1, p.PIN_3, p.PIN_2, Irqs, config);
    lcd_init(&mut i2c).await;
    lcd_clear(&mut i2c).await;
    // lcd_cursor(&mut i2c, 0, 0).await;
    lcd_message(&mut i2c, "Welcome to Rust").await;

    let mut i:u8 = 0;
    loop {
        lcd_cursor(&mut i2c, 1, 0).await;
        lcd_message(&mut i2c, "Counter: ").await;
        lcd_char(&mut i2c, 0x30 + i / 10).await;
        lcd_char(&mut i2c, 0x30 + i % 10).await;

        info!("external LED on, onboard LED off!");
        led.set_high();
        control.gpio_set(0, false).await;
        Timer::after(Duration::from_secs(1)).await;

        info!("external LED off, onboard LED on!");
        led.set_low();
        control.gpio_set(0, true).await;
        Timer::after(Duration::from_secs(1)).await;

        i = if i<99 {i+1} else {0};
    }
}