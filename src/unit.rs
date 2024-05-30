use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Copy, PartialOrd, Ord, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub enum SolidUnit {
    #[default]
    Grams,
    KiloGrams,
    Pinch,
}

#[derive(PartialEq, Copy, PartialOrd, Ord, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub enum LiquidUnit {
    #[default]
    MilliLiters,
    Liters,
}

#[derive(PartialEq, Copy, PartialOrd, Ord, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub enum AmbiguosUnit {
    TeaSpoon,
    TableSpoon,
    #[default]
    Count,
}

#[derive(PartialEq, Copy, PartialOrd, Ord, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum Unit {
    Solid(SolidUnit),
    Liquid(LiquidUnit),
    Ambigous(AmbiguosUnit),
}

impl Default for Unit {
    fn default() -> Self {
        Self::Solid(SolidUnit::default())
    }
}

pub static UNITS: &[Unit] = &[
    Unit::Solid(SolidUnit::Grams),
    Unit::Solid(SolidUnit::KiloGrams),
    Unit::Solid(SolidUnit::Pinch),
    Unit::Liquid(LiquidUnit::MilliLiters),
    Unit::Liquid(LiquidUnit::Liters),
    Unit::Ambigous(AmbiguosUnit::TeaSpoon),
    Unit::Ambigous(AmbiguosUnit::TableSpoon),
    Unit::Ambigous(AmbiguosUnit::Count),
];

impl SolidUnit {
    pub const fn in_grams(self) -> f64 {
        match self {
            Self::Grams => 1.0,
            Self::KiloGrams => 1000.0,
            // Self::TeaSpoon => 4.2,
            // Self::TableSpoon => 13.0,
            Self::Pinch => 0.3,
        }
    }
}

impl Unit {
    // pub fn to_grams(self, quantity: f64) -> f64 {
    //     quantity * self.in_grams()
    // }

    pub const fn is_liquid(self) -> bool {
        match self {
            Self::Solid(_) => false,
            _ => true,
        }
    }

    // pub fn grams(self, quantity: f64) -> f64 {
    //     quantity / self.in_grams()
    // }

    pub const fn abreviation(self) -> &'static str {
        match self {
            Self::Solid(SolidUnit::Grams) => "g",
            Self::Solid(SolidUnit::KiloGrams) => "kg",
            Self::Solid(SolidUnit::Pinch) => "pinch",

            Self::Liquid(LiquidUnit::MilliLiters) => "ml",
            Self::Liquid(LiquidUnit::Liters) => "L",

            Self::Ambigous(AmbiguosUnit::Count) => "x",
            Self::Ambigous(AmbiguosUnit::TeaSpoon) => "tsp",
            Self::Ambigous(AmbiguosUnit::TableSpoon) => "tbsp",
        }
    }
}

pub fn apropriate_unit(grams: f64, is_liquid: bool) -> (f64, Unit) {
    // if is_liquid {
    //     if grams < Unit::Liters.in_grams() {
    //         (grams, Unit::MilliLiters)
    //     } else {
    //         (Unit::Liters.grams(grams), Unit::Liters)
    //     }
    // } else if grams < Unit::KiloGrams.in_grams() {
    //     (grams, Unit::Grams)
    // } else {
    //     (Unit::KiloGrams.grams(grams), Unit::KiloGrams)
    // }
    todo!()
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abreviation())
    }
}
