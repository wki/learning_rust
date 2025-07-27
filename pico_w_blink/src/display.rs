#![no_std]

use defmt::{info, warn};
use embassy_rp::i2c;
use embassy_rp::peripherals::I2C1;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
    primitives::{Rectangle},
};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::primitives::{PrimitiveStyle};
use sh1106::Builder;
// use sh1106::interface::DisplayInterface;
use sh1106::prelude::{GraphicsMode, I2cInterface};

pub struct Display<'a> {
    display: GraphicsMode<I2cInterface<i2c::I2c<'a, I2C1, i2c::Async>>>,
    positive_text_style: MonoTextStyle<'a, BinaryColor>,
    reverse_text_style: MonoTextStyle<'a, BinaryColor>,
}

pub enum TextStyle {
    Positive, PositiveClear,
    Negative, NegativeClear
}

// FIXME: type definition still wrong.
pub fn init(i2c: i2c::I2c<I2C1, i2c::Async>) -> Display {
    let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();
    display.init().unwrap();
    display.flush().unwrap();

    let positive_text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let reverse_text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::Off)
        .build();

    Display {
        display,
        positive_text_style,
        reverse_text_style,
    }
}

impl Display<'_> {
    pub fn clear(&mut self) {
        self.display.clear();
        self.display.flush().unwrap();
    }

    pub fn flush(&mut self) {
        // sometimes panics here
        self.display.flush().unwrap();
    }

    pub fn write_text(&mut self, text: &str, position: Point, text_style: TextStyle) {
        let (style, clear, color) = match text_style {
            TextStyle::Positive =>      (self.positive_text_style, false, BinaryColor::Off),
            TextStyle::PositiveClear => (self.positive_text_style, true,  BinaryColor::Off),
            TextStyle::Negative =>      (self.reverse_text_style,  false, BinaryColor::On),
            TextStyle::NegativeClear => (self.reverse_text_style,  true,  BinaryColor::On),
        };

        if clear {
            let top_left = Point {
                x: position.x - 1,
                y: position.y - 1
            };
            let bottom_right = Point {
                x: position.x + (text.len() as i32) * 6 + 1,
                y: position.y + 11
            };
            Rectangle::with_corners(top_left, bottom_right)
                .into_styled(PrimitiveStyle::with_fill(color))
                .draw(&mut self.display)
                .unwrap();
        }

        Text::with_baseline(text, position, style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
    }
}
