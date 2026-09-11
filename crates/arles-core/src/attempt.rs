//! Máquina de estados de un intento de envío.
//!
//! Ver `documentacion/03-arquitectura/MOTOR_DE_EJECUCION.md` §1 y ADR-0004.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Estado de un `message_attempt`.
///
/// Un intento es la unidad atómica del motor: **uno por contacto y campaña**.
/// La restricción `UNIQUE(campaign_id, contact_id)` hace que el duplicado sea
/// imposible a nivel de esquema, no una esperanza del código.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptState {
    /// En cola, esperando turno.
    Queued,
    /// Un worker lo tomó mediante compare-and-swap.
    Claimed,
    /// Llamada al proveedor en curso.
    Sending,
    /// El proveedor **aceptó** el mensaje.
    ///
    /// No significa «entregado»: significa que el proveedor se hizo cargo.
    /// La interfaz dice «Aceptado» (§65).
    Sent,
    /// Fallo transitorio. Se reintentará con backoff.
    Failed,
    /// Fallo definitivo. No se reintenta.
    PermanentlyFailed,
    /// Bloqueado por la lista de supresión. No es un fallo.
    Suppressed,
    /// Cancelado por una parada.
    Cancelled,
    /// **Estado ambiguo**: la aplicación murió con el intento en [`Self::Sending`].
    ///
    /// El correo pudo salir o no, y no hay forma de saberlo sin leer la bandeja
    /// de enviados — permiso que v1.2.0 no solicita (§69).
    ///
    /// **Nunca se reenvía automáticamente.** Ver [`AttemptState::recuperar_al_arrancar`].
    PresumedSent,
}

impl AttemptState {
    /// Identificador estable para persistir. No cambiar: hay datos con él.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Claimed => "claimed",
            Self::Sending => "sending",
            Self::Sent => "sent",
            Self::Failed => "failed",
            Self::PermanentlyFailed => "permanently_failed",
            Self::Suppressed => "suppressed",
            Self::Cancelled => "cancelled",
            Self::PresumedSent => "presumed_sent",
        }
    }

    /// Un estado terminal no admite más transiciones.
    pub fn es_terminal(self) -> bool {
        matches!(
            self,
            Self::Sent
                | Self::PermanentlyFailed
                | Self::Suppressed
                | Self::Cancelled
                | Self::PresumedSent
        )
    }

    /// ¿Este estado consumió un envío real hacia el proveedor?
    ///
    /// [`Self::PresumedSent`] cuenta: pudo haber salido, y el presupuesto de
    /// envío debe asumir lo peor para no exceder el límite del proveedor.
    pub fn consumio_envio(self) -> bool {
        matches!(self, Self::Sent | Self::PresumedSent)
    }

    /// Transiciones permitidas.
    pub fn puede_ir_a(self, destino: Self) -> bool {
        use AttemptState::*;
        match (self, destino) {
            (Queued, Claimed | Suppressed | Cancelled) => true,
            (Claimed, Sending | Suppressed | Cancelled) => true,
            (Sending, Sent | Failed | PermanentlyFailed | PresumedSent) => true,
            // Un fallo transitorio vuelve a la cola tras el backoff.
            (Failed, Queued | PermanentlyFailed | Cancelled) => true,
            _ => false,
        }
    }

    /// Aplica una transición, o explica por qué no procede.
    ///
    /// # Errores
    ///
    /// [`TransitionError`] si la transición no está permitida.
    pub fn transicionar(self, destino: Self) -> Result<Self, TransitionError> {
        if self.puede_ir_a(destino) {
            Ok(destino)
        } else {
            Err(TransitionError {
                desde: self,
                hasta: destino,
            })
        }
    }

    /// Qué hacer con un intento al arrancar la aplicación.
    ///
    /// Un intento que quedó en [`Self::Sending`] es **genuinamente ambiguo**: la
    /// aplicación murió entre que el proveedor aceptó el mensaje y que pudimos
    /// registrar el commit.
    ///
    /// Pasa a [`Self::PresumedSent`] y **no se reenvía**. Los dos errores
    /// posibles no son simétricos:
    ///
    /// | Si elegimos | Y sí salió | Y no salió |
    /// |---|---|---|
    /// | Reenviar | **Duplicado.** Irreversible, invisible para nosotros | Correcto |
    /// | No reenviar | Correcto | **Visible y recuperable** |
    ///
    /// Fallamos del lado recuperable. Ver ADR-0004.
    ///
    /// Un intento en [`Self::Claimed`] sí vuelve a la cola: nadie llamó todavía
    /// al proveedor, así que no hay ambigüedad.
    pub fn recuperar_al_arrancar(self) -> Self {
        match self {
            Self::Sending => Self::PresumedSent,
            Self::Claimed => Self::Queued,
            otro => otro,
        }
    }
}

impl fmt::Display for AttemptState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for AttemptState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AttemptState::{}", self.as_str())
    }
}

/// Transición rechazada por la máquina de estados.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("transición de intento no permitida: {desde} → {hasta}")]
pub struct TransitionError {
    pub desde: AttemptState,
    pub hasta: AttemptState,
}

#[cfg(test)]
mod tests {
    use super::AttemptState::*;
    use super::*;

    const TODOS: [AttemptState; 9] = [
        Queued,
        Claimed,
        Sending,
        Sent,
        Failed,
        PermanentlyFailed,
        Suppressed,
        Cancelled,
        PresumedSent,
    ];

    #[test]
    fn el_camino_feliz() {
        let s = Queued
            .transicionar(Claimed)
            .and_then(|s| s.transicionar(Sending))
            .and_then(|s| s.transicionar(Sent));
        assert_eq!(s, Ok(Sent));
    }

    #[test]
    fn los_estados_terminales_no_admiten_transiciones() {
        for estado in TODOS.into_iter().filter(|e| e.es_terminal()) {
            for destino in TODOS {
                assert!(
                    estado.transicionar(destino).is_err(),
                    "{estado} es terminal y no debería ir a {destino}"
                );
            }
        }
    }

    #[test]
    fn no_se_puede_enviar_sin_pasar_por_claimed() {
        assert!(Queued.transicionar(Sending).is_err());
    }

    #[test]
    fn no_se_puede_marcar_enviado_desde_la_cola() {
        assert!(Queued.transicionar(Sent).is_err());
    }

    #[test]
    fn un_fallo_transitorio_vuelve_a_la_cola() {
        assert_eq!(Sending.transicionar(Failed), Ok(Failed));
        assert_eq!(Failed.transicionar(Queued), Ok(Queued));
    }

    /// ADR-0004: el caso que define la política de idempotencia.
    #[test]
    fn un_intento_ambiguo_nunca_se_reenvia() {
        assert_eq!(Sending.recuperar_al_arrancar(), PresumedSent);
        assert!(
            PresumedSent.es_terminal(),
            "presumed_sent debe ser terminal: si admitiera volver a la cola, \
             un reinicio podría producir el duplicado que queremos evitar"
        );
        assert!(PresumedSent.transicionar(Queued).is_err());
    }

    #[test]
    fn un_intento_tomado_pero_no_enviado_si_vuelve_a_la_cola() {
        // Nadie llamó al proveedor todavía: no hay ambigüedad que preservar.
        assert_eq!(Claimed.recuperar_al_arrancar(), Queued);
    }

    #[test]
    fn la_recuperacion_no_toca_los_estados_terminales() {
        for estado in TODOS.into_iter().filter(|e| e.es_terminal()) {
            assert_eq!(estado.recuperar_al_arrancar(), estado);
        }
    }

    /// El presupuesto de envío debe asumir lo peor con un intento ambiguo:
    /// si pudo salir, cuenta contra el límite del proveedor.
    #[test]
    fn un_intento_ambiguo_cuenta_como_envio_consumido() {
        assert!(PresumedSent.consumio_envio());
        assert!(Sent.consumio_envio());
        assert!(!Failed.consumio_envio());
        assert!(!Suppressed.consumio_envio());
        assert!(!Cancelled.consumio_envio());
    }

    #[test]
    fn la_supresion_puede_ganar_antes_de_llamar_al_proveedor() {
        // La comprobación autoritativa ocurre justo antes del envío.
        assert_eq!(Claimed.transicionar(Suppressed), Ok(Suppressed));
        assert_eq!(Queued.transicionar(Suppressed), Ok(Suppressed));
        // Pero no después: ya se llamó al proveedor.
        assert!(Sending.transicionar(Suppressed).is_err());
    }

    #[test]
    fn los_identificadores_persistidos_son_estables() {
        // Si este test falla, hay datos en producción que dejarán de leerse.
        assert_eq!(Queued.as_str(), "queued");
        assert_eq!(PermanentlyFailed.as_str(), "permanently_failed");
        assert_eq!(PresumedSent.as_str(), "presumed_sent");
    }

    #[test]
    fn serde_usa_los_mismos_identificadores() {
        let json = serde_json::to_string(&PresumedSent).expect("serializa");
        assert_eq!(json, "\"presumed_sent\"");
    }
}
