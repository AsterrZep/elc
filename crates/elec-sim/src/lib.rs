//! Simulación eléctrica (INV-08).
//!
//! Estado separado del diseño: este crate **nunca** muta el `Project`. El solver real (MNA
//! lineal, curvas y fallas) se implementa en M3/M4; aquí vive el contrato y el reloj.

use elec_core::Project;
use elec_profile::NormProfile;
use serde::{Deserialize, Serialize};

/// Tiempo simulado en milisegundos.
///
/// Entero a propósito: el reloj debe ser determinista y reproducible (INV-10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SimTime(pub u64);

impl SimTime {
    #[must_use]
    pub const fn from_millis(ms: u64) -> Self {
        Self(ms)
    }

    #[must_use]
    pub const fn millis(self) -> u64 {
        self.0
    }
}

/// Reloj acelerable de la simulación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimClock {
    pub elapsed: SimTime,
    /// Factor de aceleración: 1×, 60×, 3600×.
    pub speed: u32,
}

impl SimClock {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            elapsed: SimTime(0),
            speed: 1,
        }
    }

    #[must_use]
    pub const fn with_speed(speed: u32) -> Self {
        Self {
            elapsed: SimTime(0),
            speed,
        }
    }

    /// Avanza el reloj el tiempo real transcurrido, multiplicado por la velocidad.
    pub fn advance(&mut self, real_elapsed_ms: u64) {
        self.elapsed.0 = self
            .elapsed
            .0
            .saturating_add(real_elapsed_ms.saturating_mul(u64::from(self.speed)));
    }
}

impl Default for SimClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Causa del disparo de una protección.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TripCause {
    Overload,
    ShortCircuit,
    EarthLeak,
    Manual,
}

/// Estado de una protección.
///
/// `band` es un intervalo, no un instante: la norma define bandas de disparo y el simulador no
/// debe inventar un punto dentro de ellas (ver `INVESTIGACIÓN-v1` §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtectionState {
    Closed,
    Tripped {
        band: (SimTime, SimTime),
        cause: TripCause,
    },
}

/// Estado completo de una simulación.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SimState {
    pub clock: SimClock,
    pub protections: Vec<ProtectionState>,
}

/// Error de simulación.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SimError {
    #[error("el solver nodal se implementa en M3")]
    NotImplemented,
    #[error("el proyecto no tiene netlist: define las conexiones antes de simular")]
    EmptyNetlist,
}

/// Ejecuta la simulación de un escenario.
///
/// # Errors
/// Devuelve [`SimError::NotImplemented`] hasta M3.
pub fn simulate(_project: &Project, _profile: &NormProfile) -> Result<SimState, SimError> {
    Err(SimError::NotImplemented)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_scales_with_speed() {
        let mut clock = SimClock::with_speed(60);
        clock.advance(1000);
        assert_eq!(clock.elapsed.millis(), 60_000);
    }

    #[test]
    fn clock_defaults_to_realtime() {
        let mut clock = SimClock::new();
        clock.advance(250);
        assert_eq!(clock.elapsed, SimTime::from_millis(250));
    }

    #[test]
    fn saturated_clock_does_not_overflow() {
        let mut clock = SimClock::with_speed(u32::MAX);
        clock.advance(u64::MAX);
        assert_eq!(clock.elapsed.millis(), u64::MAX);
    }
}
