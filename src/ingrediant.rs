use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{generational_map::GenerationalKey, Unit};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ingrediant {
    pub name: Arc<str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct IngrediantQuantity {
    pub quantity: f64,
    pub unit: Unit,
}

pub type IngrediantKey = GenerationalKey<Ingrediant>;
