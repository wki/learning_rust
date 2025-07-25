#![no_std]
#![no_main]
#![allow(dead_code)]
/*
Wiring
 GP2, GP3 -> SDA & SCL for I2C Display
 GP26 -> Joystick h Analog in
 GP27 -> Joystick v Analog in
 ?? -> joystick button
 */
mod display;

use embedded_graphics::prelude::Point;
use heapless::String;
use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts};
use embassy_rp::gpio::{Level, Output, Pull};
use embassy_rp::adc::{Adc, Channel as AdcChannel, Config as AdcConfig, InterruptHandler as AdcInterruptHandler};
use embassy_rp::i2c::{self, Config as I2cConfig, InterruptHandler as I2cInterruptHandler};
use embassy_rp::peripherals::{DMA_CH0, PIO0, I2C1};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_time::{Duration, Instant, Timer};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use bt_hci::cmd::le::*;
use bt_hci::controller::ControllerCmdSync;
use cyw43::bluetooth::BtDriver;
use embassy_futures::join::join;
use trouble_host::prelude::*;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    I2C1_IRQ => I2cInterruptHandler<I2C1>;
    ADC_IRQ_FIFO => AdcInterruptHandler;
});

// Use your company ID (register for free with Bluetooth SIG)
const COMPANY_ID: u16 = 0xBEEF;

fn make_adv_payload(start: Instant, update_count: u32) -> [u8; 8] {
    let mut data = [0u8; 8];
    let elapsed_ms = Instant::now().duration_since(start).as_millis() as u32;
    data[0..4].copy_from_slice(&update_count.to_be_bytes());
    data[4..8].copy_from_slice(&elapsed_ms.to_be_bytes());
    data
}

pub async fn run<C>(controller: C)
where
    C: Controller
    + for<'t> ControllerCmdSync<LeSetExtAdvData<'t>>
    + ControllerCmdSync<LeClearAdvSets>
    + ControllerCmdSync<LeSetExtAdvParams>
    + ControllerCmdSync<LeSetAdvSetRandomAddr>
    + ControllerCmdSync<LeReadNumberOfSupportedAdvSets>
    + for<'t> ControllerCmdSync<LeSetExtAdvEnable<'t>>
    + for<'t> ControllerCmdSync<LeSetExtScanResponseData<'t>>,
{
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    //info!("Our address = {:?}", address);

    let mut resources: HostResources<DefaultPacketPool, 0, 0, 27> = HostResources::new();
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);
    let Host {
        mut peripheral,
        mut runner,
        ..
    } = stack.build();

    let mut adv_data = [0; 64];
    let mut update_count = 0u32;
    let start = Instant::now();
    let len = AdStructure::encode_slice(
        &[
            AdStructure::CompleteLocalName(b"JoyStickBeacon"),
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ManufacturerSpecificData {
                company_identifier: COMPANY_ID,
                payload: &make_adv_payload(start, update_count),
            },
        ],
        &mut adv_data[..],
    )
        .unwrap();

    info!("Starting advertising");
    let _ = join(runner.run(), async {
        loop {
            let mut params = AdvertisementParameters::default();
            params.interval_min = Duration::from_millis(25);
            params.interval_max = Duration::from_millis(150);
            let _advertiser = peripheral
                .advertise(
                    &params,
                    Advertisement::NonconnectableNonscannableUndirected {
                        adv_data: &adv_data[..len],
                    },
                )
                .await
                .unwrap();
            loop {
                Timer::after(Duration::from_millis(100)).await;
                update_count = update_count.wrapping_add(1);

                let len = AdStructure::encode_slice(
                    &[
                        AdStructure::CompleteLocalName(b"JoyStickBeaconx"),
                        AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                        AdStructure::ManufacturerSpecificData {
                            company_identifier: COMPANY_ID,
                            payload: &make_adv_payload(start, update_count),
                        },
                    ],
                    &mut adv_data[..],
                )
                    .unwrap();

                peripheral
                    .update_adv_data(Advertisement::NonconnectableNonscannableUndirected {
                        adv_data: &adv_data[..len],
                    })
                    .await
                    .unwrap();

                if update_count % 100 == 0 {
                    info!("Still running: Updated the beacon {} times", update_count);
                }
            }
        }
    })
        .await;
}

#[embassy_executor::task]
async fn cyw43_task(runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn beacon_task(bt_device: BtDriver<'static>) -> () {
    let controller: ExternalController<_, 10> = ExternalController::new(bt_device);
    run(controller).await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Program start");

    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_15, Level::Low);

    // Configure PIO and CYW43
    let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");
    let btfw = include_bytes!("../cyw43-firmware/43439A0_btfw.bin");
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
    // let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    let (_net_device, bt_device, mut control, runner) = cyw43::new_with_bluetooth(state, pwr, spi, fw, btfw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    control.init(clm).await;
    control.set_power_management(cyw43::PowerManagementMode::PowerSave).await;

    // bluetooth-LE controller
    unwrap!(spawner.spawn(beacon_task(bt_device)));

    // let wifi_ssid = env!("WIFI_SSID");
    // let wifi_password = env!("WIFI_PASSWORD");

    // Configure display
    let config = I2cConfig::default();
    let i2c = i2c::I2c::new_async(p.I2C1, p.PIN_3, p.PIN_2, Irqs, config);
    let mut screen = display::init(i2c);
    screen.write_text("Hello Rust", Point::zero(), display::TextStyle::Positive);
    screen.flush();

    // Configure ADC for Joystick reading
    let mut adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let mut joystick_h = AdcChannel::new_pin(p.PIN_26, Pull::None);
    let mut joystick_v = AdcChannel::new_pin(p.PIN_27, Pull::None);

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
        let mut line:String<5> = String::new();
        line.push((0x30 + i / 10) as char).unwrap();
        line.push((0x30 + i % 10) as char).unwrap();

        screen.write_text(line.as_str(), Point::new(80,16), display::TextStyle::PositiveClear);
        screen.write_text(xline.as_str(), Point::new(0,50), display::TextStyle::NegativeClear);
        screen.write_text(yline.as_str(), Point::new(40,50), display::TextStyle::NegativeClear);
        screen.flush();

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