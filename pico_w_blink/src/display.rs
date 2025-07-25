#![no_std]

use embassy_rp::i2c;
use embassy_rp::peripherals::I2C1;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
    // primitives::{Rectangle, Circle},
};
use embedded_graphics::mono_font::MonoTextStyle;
// use embedded_graphics::primitives::{Line, PrimitiveStyle};
use sh1106::Builder;
// use sh1106::interface::DisplayInterface;
use sh1106::prelude::{GraphicsMode, I2cInterface};

pub struct Display<'a> {
    pub display: GraphicsMode<I2cInterface<i2c::I2c<'a, I2C1, i2c::Async>>>,
    pub positive_text_style: MonoTextStyle<'a, BinaryColor>,
    pub reverse_text_style: MonoTextStyle<'a, BinaryColor>,
}

pub enum TextStyle {
    Positive,
    Negative
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
        self.display.flush().unwrap();
    }

    pub fn write_text(&mut self, text: &str, position: Point, text_style: TextStyle) {
        let color = match text_style {
            TextStyle::Positive => BinaryColor::On,
            TextStyle::Negative => BinaryColor::Off
        };
        let style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(color)
            .build();

        Text::with_baseline(text, position, style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
    }
}

/*
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
*/
