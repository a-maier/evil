// thin wrapper around plotter font handling

use std::convert::From;
use std::default::Default;

use serde::{Deserialize, Serialize};
use strum::Display;

const DEFAULT_FONT_SIZE: f64 = 40.;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Deserialize, Serialize)]
pub enum FontFamily {
    Serif,
    SansSerif,
    Monospace,
    Name(String),
}

impl Default for FontFamily {
    fn default() -> Self {
        Self::Serif
    }
}

impl<'a> From<&'a FontFamily> for plotters::prelude::FontFamily<'a> {
    fn from(family: &'a FontFamily) -> Self {
        match family {
            FontFamily::Serif => plotters::prelude::FontFamily::Serif,
            FontFamily::SansSerif => plotters::prelude::FontFamily::SansSerif,
            FontFamily::Monospace => plotters::prelude::FontFamily::Monospace,
            FontFamily::Name(s) => plotters::prelude::FontFamily::Name(s.as_str()),
        }
    }
}

#[derive(Copy, Clone, Display, Eq, Ord, Hash, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum FontStyle {
    Normal,
    Oblique,
    Italic,
    Bold,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<FontStyle> for plotters::prelude::FontStyle {
    fn from(family: FontStyle) -> Self {
        match family {
            FontStyle::Normal => plotters::prelude::FontStyle::Normal,
            FontStyle::Oblique => plotters::prelude::FontStyle::Oblique,
            FontStyle::Italic => plotters::prelude::FontStyle::Italic,
            FontStyle::Bold => plotters::prelude::FontStyle::Bold,
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Font {
    pub family: FontFamily,
    pub size: f64,
    pub style: FontStyle,
}

impl Default for Font {
    fn default() -> Self {
        Self {
            size: DEFAULT_FONT_SIZE,
            family: Default::default(),
            style: Default::default(),
        }
    }
}

impl<'a> From<&'a Font> for plotters::prelude::FontDesc<'a> {
    fn from(font: &'a Font) -> Self {
        Self::new((&font.family).into(), font.size, font.style.into())
    }
}
