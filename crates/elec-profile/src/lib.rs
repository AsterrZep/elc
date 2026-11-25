//! Perfil normativo como datos (INV-06).
//!
//! Ningún número de una norma vive en el código: tensión, frecuencia, cotas de montaje, curvas y
//! reglas vienen de un JSON. Añadir el segundo perfil (IEC) no debería tocar este crate.

use std::collections::BTreeMap;

use elec_core::{Millimeters, MountKind};
use serde::{Deserialize, Serialize};

/// Sistema de alimentación que define el perfil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupplySystem {
    /// Fase partida 120/240 V (residencial NEC).
    SplitPhase120_240,
    /// Monofásico 230 V (IEC).
    SinglePhase230,
    /// Trifásico 400 V; fuera del MVP.
    ThreePhase400,
}

/// Severidad de un hallazgo (D-11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Definición de una regla del perfil.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleDef {
    /// Identificador con edición, p. ej. `NEC-2023/210.8(A)/1`.
    pub rule_id: String,
    pub severity: Severity,
    /// Referencia parafraseada, nunca texto copiado de la norma.
    pub citation: String,
    /// `false` mientras no se haya contrastado con la edición oficial.
    pub verified: bool,
    /// Parámetros de la regla (distancias, límites, tablas).
    #[serde(default)]
    pub params: serde_json::Value,
}

/// Perfil normativo completo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormProfile {
    /// p. ej. `NEC-2023`.
    pub id: String,
    /// p. ej. `2023`.
    pub edition: String,
    pub supply: SupplySystem,
    pub frequency_hz: u16,
    #[serde(default)]
    pub rules: Vec<RuleDef>,
    /// Alturas de montaje por tipo de dispositivo, usadas para metrado vertical y (v1.1) para el 3D.
    #[serde(default)]
    pub standard_heights_mm: BTreeMap<MountKind, Millimeters>,
}

/// Error de carga de un perfil.
#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("perfil inválido: {0}")]
    Invalid(String),
    #[error("el perfil no declara reglas: {0}")]
    Empty(String),
}

impl NormProfile {
    /// Carga un perfil desde JSON.
    ///
    /// # Errors
    /// Devuelve [`ProfileError`] si el JSON no es válido o no declara al menos una regla.
    pub fn from_json(text: &str) -> Result<Self, ProfileError> {
        let profile: Self =
            serde_json::from_str(text).map_err(|err| ProfileError::Invalid(err.to_string()))?;
        if profile.rules.is_empty() {
            return Err(ProfileError::Empty(profile.id));
        }
        Ok(profile)
    }

    /// Número de reglas declaradas.
    #[must_use]
    pub fn total_rules(&self) -> usize {
        self.rules.len()
    }

    /// Reglas ya contrastadas con el texto oficial.
    #[must_use]
    pub fn verified_rules(&self) -> usize {
        self.rules.iter().filter(|rule| rule.verified).count()
    }

    /// Reglas que el informe debe declarar como no verificadas.
    #[must_use]
    pub fn unverified_rules(&self) -> usize {
        self.total_rules() - self.verified_rules()
    }

    /// Altura de montaje por defecto para un tipo de dispositivo.
    #[must_use]
    pub fn height_for(&self, kind: MountKind) -> Option<Millimeters> {
        self.standard_heights_mm.get(&kind).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "id": "TEST-2026",
        "edition": "2026",
        "supply": "SplitPhase120_240",
        "frequency_hz": 60,
        "rules": [
            {
                "rule_id": "TEST-2026/1",
                "severity": "Error",
                "citation": "regla de prueba",
                "verified": false
            }
        ],
        "standard_heights_mm": {
            "Switch": 1200,
            "Receptacle": 350,
            "Panel": 1700
        }
    }"#;

    #[test]
    fn req_drc_011_profile_reports_unverified_rules() {
        let profile = NormProfile::from_json(SAMPLE).expect("perfil válido");
        assert_eq!(profile.total_rules(), 1);
        assert_eq!(profile.verified_rules(), 0);
        assert_eq!(profile.unverified_rules(), 1);
    }

    #[test]
    fn heights_come_from_data() {
        let profile = NormProfile::from_json(SAMPLE).expect("perfil válido");
        assert_eq!(
            profile.height_for(MountKind::Switch),
            Some(Millimeters(1200))
        );
        assert_eq!(profile.height_for(MountKind::Luminaire), None);
    }

    #[test]
    fn profile_without_rules_is_rejected() {
        let text = r#"{
            "id": "X", "edition": "1", "supply": "SinglePhase230", "frequency_hz": 50
        }"#;
        assert!(NormProfile::from_json(text).is_err());
    }
}
