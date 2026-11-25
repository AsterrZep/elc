//! Motor de reglas (DRC).
//!
//! Cada hallazgo cita su regla, su severidad y los elementos implicados (INV-07). Las nueve
//! familias se implementan en M5; aquí vive el contrato de salida y el contexto de evaluación.

use elec_core::Project;
use elec_profile::{NormProfile, Severity};
use serde::{Deserialize, Serialize};

/// Referencia a un elemento del proyecto implicado en un hallazgo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElementRef {
    /// Tipo de elemento: `wall`, `device`, `circuit`, `net`…
    pub kind: String,
    /// Identificador del elemento, en su forma textual.
    pub id: String,
}

/// Hallazgo de una regla.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    #[serde(default)]
    pub elements: Vec<ElementRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_hint: Option<String>,
}

/// Contexto de evaluación de reglas.
pub struct RuleContext<'a> {
    pub project: &'a Project,
    pub profile: &'a NormProfile,
}

/// Evalúa las reglas del perfil sobre el proyecto.
///
/// Devuelve una lista vacía hasta M5: no hay reglas implementadas todavía, y el informe declara
/// cuántas reglas existen y cuántas están verificadas.
#[must_use]
pub fn evaluate(_ctx: &RuleContext<'_>) -> Vec<Finding> {
    Vec::new()
}

/// Número de reglas que el perfil declara, para el encabezado legal del informe (§3.4).
#[must_use]
pub fn declared_rule_count(profile: &NormProfile) -> usize {
    profile.total_rules()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluation_starts_empty_until_rules_exist() {
        let project = Project::new("Casa");
        let profile = NormProfile {
            id: "TEST".into(),
            edition: "1".into(),
            supply: elec_profile::SupplySystem::SplitPhase120_240,
            frequency_hz: 60,
            rules: Vec::new(),
            standard_heights_mm: Default::default(),
        };
        let ctx = RuleContext {
            project: &project,
            profile: &profile,
        };
        assert!(evaluate(&ctx).is_empty());
        assert_eq!(declared_rule_count(&profile), 0);
    }
}
