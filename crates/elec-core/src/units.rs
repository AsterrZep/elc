//! Magnitudes físicas con tipos propios (INV-03) y longitud entera (INV-04).
//!
//! La API pública no expone `f64` suelto con significado físico: cada magnitud tiene su tipo,
//! su constructor validado y su unidad en el `Display`.

use std::fmt;
use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

/// Longitud en milímetros enteros.
///
/// Toda la geometría del proyecto usa esta unidad: no hay comparaciones de decimales para
/// decidir topología (INV-04).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Millimeters(pub i64);

impl Millimeters {
    pub const ZERO: Self = Self(0);

    #[must_use]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }

    #[must_use]
    pub fn to_meters(self) -> f64 {
        self.0 as f64 / 1000.0
    }

    #[must_use]
    pub const fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }
}

impl Add for Millimeters {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Millimeters {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl fmt::Display for Millimeters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} mm", self.0)
    }
}

/// Error al construir una magnitud física.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum UnitError {
    /// El valor no es finito o es negativo.
    #[error("la magnitud debe ser finita y no negativa")]
    Invalid,
}

macro_rules! quantity {
    ($name:ident, $unit:literal, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(f64);

        impl $name {
            #[must_use]
            pub const fn zero() -> Self {
                Self(0.0)
            }

            /// Construye la magnitud rechazando valores no finitos o negativos.
            pub fn new(value: f64) -> Result<Self, UnitError> {
                if value.is_finite() && value >= 0.0 {
                    Ok(Self(value))
                } else {
                    Err(UnitError::Invalid)
                }
            }

            #[must_use]
            pub const fn get(self) -> f64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{} {}", self.0, $unit)
            }
        }
    };
}

quantity!(Amps, "A", "Corriente eléctrica en amperios.");
quantity!(Milliamps, "mA", "Corriente residual en miliamperios.");
quantity!(Volts, "V", "Tensión eléctrica en voltios.");
quantity!(Watts, "W", "Potencia en vatios.");
quantity!(Ohms, "Ω", "Resistencia en ohmios.");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn req_geo_001_millimeters_are_exact_integers() {
        let width = Millimeters::new(3000);
        let height = Millimeters::new(150);
        assert_eq!(width.get() * height.get(), 450_000);
    }

    #[test]
    fn physical_quantities_reject_invalid_values() {
        assert!(Amps::new(-1.0).is_err());
        assert!(Amps::new(f64::NAN).is_err());
        assert!(Volts::new(230.0).is_ok());
    }

    #[test]
    fn millimeters_display_with_unit() {
        assert_eq!(Millimeters::new(1200).to_string(), "1200 mm");
    }
}
