//! Pausar, Reanudar y Detener: tres acciones que no se pueden confundir.
//!
//! Ver `MOTOR_DE_EJECUCION.md` §9 y `VISION_Y_ALCANCE.md` §62.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA DIFERENCIA QUE ESTE ARCHIVO EXISTE PARA PROTEGER
//!
//! | Acción | Qué le pasa a la cola | ¿Se puede deshacer? |
//! |---|---|---|
//! | **Pausar** | Se conserva **intacta** | Sí |
//! | **Reanudar** | Sigue donde estaba | — |
//! | **Detener** | Los pendientes pasan a `cancelled` | **No** |
//!
//! Parecen tres botones y son dos cosas muy distintas. Pausar es «espera un
//! momento»; detener es «esta campaña se acabó». Y detener **no se puede
//! deshacer**: los intentos cancelados no vuelven, porque volver a encolarlos
//! significaría reconstruir una audiencia que se congeló al activar y que
//! quizá ya no exista igual.
//!
//! Por eso la pantalla pide confirmación explícita al detener, **y dice
//! cuántos mensajes van a quedarse sin enviar**. «¿Seguro?» no es una
//! confirmación informada; «esto cancela 4 312 correos que no han salido» sí
//! (§94).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ LAS REGLAS VIVEN AQUÍ Y NO EN EL BOTÓN
//!
//! Si «no se puede reanudar una campaña detenida» viviera en la pantalla,
//! cualquier otro camino hasta la base —un comando nuevo, una herramienta de
//! soporte, una importación— se la saltaría sin enterarse. Es el mismo motivo
//! por el que el permiso para levantar una supresión lo decide el núcleo.

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// En qué punto está una campaña.
///
/// Los identificadores son contrato con la `CHECK` de `campaign.status` en la
/// migración V1. **No se cambian**: hay datos escritos con ellos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstadoDeCampana {
    /// Se está escribiendo. No tiene audiencia congelada ni nada que enviar.
    Borrador,
    /// Pasó el preflight (§63) y se puede activar.
    Lista,
    /// Activada con fecha futura. La audiencia ya está congelada.
    Programada,
    /// Enviando.
    Enviando,
    /// En pausa. **La cola está intacta.**
    EnPausa,
    /// Detenida. Lo que no había salido se canceló. **Terminal.**
    Detenida,
    /// Se envió todo lo que había. **Terminal.**
    Terminada,
}

impl EstadoDeCampana {
    /// Cómo se escribe en la base. Contrato con la `CHECK` de la V1.
    #[must_use]
    pub const fn como_texto(self) -> &'static str {
        match self {
            Self::Borrador => "draft",
            Self::Lista => "ready",
            Self::Programada => "scheduled",
            Self::Enviando => "running",
            Self::EnPausa => "paused",
            Self::Detenida => "stopped",
            Self::Terminada => "completed",
        }
    }

    /// Los siete, para recorrerlos sin olvidarse de ninguno.
    pub const TODOS: &'static [Self] = &[
        Self::Borrador,
        Self::Lista,
        Self::Programada,
        Self::Enviando,
        Self::EnPausa,
        Self::Detenida,
        Self::Terminada,
    ];

    /// Lee lo que hay guardado en la base.
    ///
    /// # Errores
    ///
    /// [`CoreError::EstadoDeCampanaDesconocido`] si no es de los siete. **No se
    /// adivina**: una campaña cuyo estado no se entiende no se puede pausar ni
    /// detener con seguridad, y suponer «borrador» dejaría pasar acciones sobre
    /// una campaña que podría estar enviando.
    pub fn desde_texto(texto: &str) -> Result<Self, CoreError> {
        Self::TODOS
            .iter()
            .copied()
            .find(|e| e.como_texto() == texto)
            .ok_or(CoreError::EstadoDeCampanaDesconocido)
    }

    /// Un estado terminal no admite ninguna acción más.
    #[must_use]
    pub const fn es_terminal(self) -> bool {
        matches!(self, Self::Detenida | Self::Terminada)
    }

    /// ¿Hay una cola viva detrás de este estado?
    ///
    /// Es lo que decide si detener tiene algo que cancelar, y si la pantalla
    /// debe avisar al cerrar la ventana (§51).
    #[must_use]
    pub const fn tiene_cola_viva(self) -> bool {
        matches!(self, Self::Programada | Self::Enviando | Self::EnPausa)
    }

    /// La clave de texto del estado (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::Borrador => "campana.estado.borrador",
            Self::Lista => "campana.estado.lista",
            Self::Programada => "campana.estado.programada",
            Self::Enviando => "campana.estado.enviando",
            Self::EnPausa => "campana.estado.enPausa",
            Self::Detenida => "campana.estado.detenida",
            Self::Terminada => "campana.estado.terminada",
        }
    }
}

/// Lo que una persona pulsa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AccionDeCampana {
    /// «Espera un momento.» La cola se queda como está.
    Pausar,
    /// «Sigue.» Continúa donde estaba, respetando los límites.
    Reanudar,
    /// «Se acabó.» **Irreversible**: cancela lo que no ha salido.
    Detener,
}

impl AccionDeCampana {
    /// Las tres.
    pub const TODAS: &'static [Self] = &[Self::Pausar, Self::Reanudar, Self::Detener];

    /// La clave de texto de la acción (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::Pausar => "campana.accion.pausar",
            Self::Reanudar => "campana.accion.reanudar",
            Self::Detener => "campana.accion.detener",
        }
    }

    /// ¿Hace falta que la persona confirme antes?
    ///
    /// Sólo detener, y no por prudencia genérica: porque **no se puede
    /// deshacer**. Pedir confirmación para todo enseña a confirmar sin leer, y
    /// entonces la confirmación que importa tampoco se lee.
    #[must_use]
    pub const fn exige_confirmacion(self) -> bool {
        matches!(self, Self::Detener)
    }

    /// ¿Esta acción cancela los envíos que quedaban?
    #[must_use]
    pub const fn cancela_lo_pendiente(self) -> bool {
        matches!(self, Self::Detener)
    }

    /// A qué estado lleva esta acción, o por qué no procede.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// LAS REGLAS, Y LO QUE CADA UNA IMPIDE
    ///
    /// **Pausar** sólo tiene sentido sobre algo que iba a enviar: una campaña
    /// programada o enviando. Pausar un borrador no significa nada, y pausar
    /// una detenida daría a entender que se puede reanudar.
    ///
    /// **Reanudar** sólo desde pausa. Reanudar una detenida es la acción que
    /// este archivo existe para impedir: sus intentos están cancelados, así que
    /// «reanudar» encontraría una cola vacía y la campaña quedaría marcada como
    /// enviando **sin enviar nada**, sin error y sin explicación.
    ///
    /// **Detener** desde cualquier estado que tenga cola viva. Desde un
    /// borrador o una lista no hay nada que detener; desde una terminada,
    /// tampoco — y permitirlo convertiría «terminada» en «detenida» y borraría
    /// la diferencia entre «se envió todo» y «se canceló».
    ///
    /// # Errores
    ///
    /// [`AccionImposible`] con el estado en el que está y la acción que se
    /// pidió, para que el mensaje pueda decir las tres cosas del §95.
    pub const fn aplicar(
        self,
        estado: EstadoDeCampana,
    ) -> Result<EstadoDeCampana, AccionImposible> {
        use EstadoDeCampana as E;
        let destino = match (self, estado) {
            (Self::Pausar, E::Programada | E::Enviando) => E::EnPausa,
            (Self::Reanudar, E::EnPausa) => E::Enviando,
            (Self::Detener, E::Programada | E::Enviando | E::EnPausa) => E::Detenida,
            _ => {
                return Err(AccionImposible {
                    estaba_en: estado,
                    se_pidio: self,
                });
            }
        };
        Ok(destino)
    }
}

/// Esa acción no procede desde ese estado.
///
/// Lleva los dos datos —dónde estaba y qué se pidió— porque el §95 exige que
/// el error diga **qué pasó**, **cómo arreglarlo** y **qué está a salvo**, y
/// sin el estado de partida no se puede decir ninguna de las tres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, thiserror::Error)]
#[serde(rename_all = "camelCase")]
#[error("no se puede {se_pidio:?} una campaña en estado {estaba_en:?}")]
pub struct AccionImposible {
    pub estaba_en: EstadoDeCampana,
    pub se_pidio: AccionDeCampana,
}

impl AccionImposible {
    /// La clave de texto (§139).
    ///
    /// Hay una clave **propia** para el caso de reanudar una campaña detenida,
    /// y no es un lujo: es la confusión concreta que hay que deshacer. El
    /// mensaje genérico diría «no se puede reanudar desde ahí»; el específico
    /// puede decir lo que la persona necesita oír — que detener canceló los
    /// envíos y que para volver a escribir a esa gente hay que crear una
    /// campaña nueva.
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match (self.se_pidio, self.estaba_en) {
            (AccionDeCampana::Reanudar, EstadoDeCampana::Detenida) => {
                "campana.error.detenerNoSeDeshace"
            }
            (AccionDeCampana::Pausar, _) => "campana.error.noSePuedePausar",
            (AccionDeCampana::Reanudar, _) => "campana.error.noSePuedeReanudar",
            (AccionDeCampana::Detener, _) => "campana.error.noHayNadaQueDetener",
        }
    }
}

/// La parada de emergencia: «que no salga nada más, de ninguna campaña».
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ NO ES LO MISMO QUE PAUSAR TODAS LAS CAMPAÑAS
///
/// Pausar campaña por campaña es una lista de acciones que hay que acertar,
/// que tarda, y que deja fuera la que se activó hace diez segundos. La parada
/// de emergencia es **una** y vale para todo: el motor la consulta antes de
/// cada envío, no entre lotes.
///
/// **Y sobrevive a cerrar el programa.** Guardarla sólo en memoria significaría
/// que arrancar ARLES la levanta sola, y quien la pulsó no estaría delante para
/// verlo. Guardada en la base, arrancar encuentra la parada puesta y lo dice.
///
/// Se guarda **cuándo** se puso y no un sí/no, porque la pantalla tiene que
/// poder decir desde cuándo: «detenido desde las 14:32» es un hecho, «detenido»
/// a secas invita a preguntarse si se quedó puesto de ayer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParadaDeEmergencia {
    /// No hay parada puesta. El motor puede enviar.
    Levantada,
    /// Está puesta desde este instante, y la puso alguien.
    Puesta {
        /// Cuándo, en el formato que guarda la base.
        desde: String,
        /// Quién, si consta.
        por: Option<String>,
    },
}

impl ParadaDeEmergencia {
    /// ¿Hay que parar?
    #[must_use]
    pub const fn hay_que_parar(&self) -> bool {
        matches!(self, Self::Puesta { .. })
    }

    /// La clave de texto (§139).
    #[must_use]
    pub const fn clave_i18n(&self) -> &'static str {
        match self {
            Self::Levantada => "campana.parada.levantada",
            Self::Puesta { .. } => "campana.parada.puesta",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use AccionDeCampana::{Detener, Pausar, Reanudar};
    use EstadoDeCampana as E;

    /// **Una campaña detenida no se reanuda. Nunca.**
    ///
    /// Es la regla que da sentido a este archivo. Si se pudiera, la campaña
    /// quedaría marcada como enviando sobre una cola que se canceló entera:
    /// **no enviaría nada, no fallaría nada**, y la pantalla diría «enviando»
    /// durante días.
    ///
    /// El error tiene clave propia para poder explicar la salida —crear una
    /// campaña nueva— en vez de decir sólo que no se puede.
    #[test]
    fn una_campana_detenida_no_se_reanuda() {
        let error = Reanudar.aplicar(E::Detenida).expect_err("no procede");
        assert_eq!(error.estaba_en, E::Detenida);
        assert_eq!(error.clave_i18n(), "campana.error.detenerNoSeDeshace");
    }

    /// Pausar y reanudar es ida y vuelta, y vuelve al mismo sitio.
    #[test]
    fn pausar_y_reanudar_devuelven_la_campana_a_enviando() {
        let pausada = Pausar.aplicar(E::Enviando).expect("se puede pausar");
        assert_eq!(pausada, E::EnPausa);
        assert_eq!(Reanudar.aplicar(pausada), Ok(E::Enviando));
    }

    /// Una campaña programada se puede pausar antes de que empiece.
    #[test]
    fn una_campana_programada_se_puede_pausar_antes_de_arrancar() {
        assert_eq!(Pausar.aplicar(E::Programada), Ok(E::EnPausa));
    }

    /// Detener funciona desde los tres estados que tienen cola viva.
    #[test]
    fn detener_vale_desde_todo_lo_que_tiene_cola() {
        for estado in [E::Programada, E::Enviando, E::EnPausa] {
            assert!(
                estado.tiene_cola_viva(),
                "«{}» debería contar como cola viva",
                estado.como_texto()
            );
            assert_eq!(Detener.aplicar(estado), Ok(E::Detenida));
        }
    }

    /// **Terminada no se convierte en detenida.**
    ///
    /// Son dos cosas distintas que la pantalla enseña distinto: una campaña que
    /// envió todo lo que tenía y una que se canceló a medias. Permitir el paso
    /// borraría esa diferencia en el histórico, y ese histórico es lo que
    /// responde a «¿a quién le llegó esto?».
    #[test]
    fn una_campana_terminada_no_se_convierte_en_detenida() {
        let error = Detener.aplicar(E::Terminada).expect_err("no procede");
        assert_eq!(error.clave_i18n(), "campana.error.noHayNadaQueDetener");
    }

    /// Desde un borrador no se puede ni pausar ni detener: no hay nada.
    #[test]
    fn sobre_un_borrador_no_hay_nada_que_parar() {
        assert!(Pausar.aplicar(E::Borrador).is_err());
        assert!(Detener.aplicar(E::Borrador).is_err());
        assert!(Reanudar.aplicar(E::Borrador).is_err());
    }

    /// **Ninguna acción sale de un estado terminal.**
    ///
    /// Se recorren las tres acciones contra los dos estados terminales en vez
    /// de escribir los seis casos: así, el día que se añada una acción, esta
    /// prueba la cubre sola.
    #[test]
    fn de_un_estado_terminal_no_sale_ninguna_accion() {
        for estado in E::TODOS.iter().copied().filter(|e| e.es_terminal()) {
            for accion in AccionDeCampana::TODAS.iter().copied() {
                assert!(
                    accion.aplicar(estado).is_err(),
                    "«{}» es terminal y aceptó «{}»",
                    estado.como_texto(),
                    accion.clave_i18n()
                );
            }
        }
    }

    /// **Sólo detener cancela, y sólo detener pide confirmación.**
    ///
    /// Las dos propiedades van juntas a propósito: lo que se confirma es
    /// justamente lo que no se puede deshacer. Si alguna vez dejaran de
    /// coincidir, o habría una acción irreversible sin confirmar o una
    /// confirmación que sobra — y las confirmaciones que sobran son las que
    /// enseñan a pulsar «sí» sin leer.
    #[test]
    fn se_confirma_exactamente_lo_que_no_se_puede_deshacer() {
        for accion in AccionDeCampana::TODAS.iter().copied() {
            assert_eq!(
                accion.exige_confirmacion(),
                accion.cancela_lo_pendiente(),
                "«{}» confirma y cancela cosas distintas",
                accion.clave_i18n()
            );
        }
        assert!(Detener.exige_confirmacion());
        assert!(!Pausar.exige_confirmacion());
    }

    /// **Pausar no toca la cola.** Es la diferencia entera con detener.
    #[test]
    fn pausar_no_cancela_nada() {
        assert!(!Pausar.cancela_lo_pendiente());
        assert!(!Reanudar.cancela_lo_pendiente());
    }

    /// Los identificadores que van a la base no cambian: hay datos con ellos,
    /// y una `CHECK` en la V1 que los exige.
    #[test]
    fn los_identificadores_persistidos_son_estables() {
        assert_eq!(E::Borrador.como_texto(), "draft");
        assert_eq!(E::EnPausa.como_texto(), "paused");
        assert_eq!(E::Detenida.como_texto(), "stopped");
        assert_eq!(E::Terminada.como_texto(), "completed");
    }

    /// Y se leen de vuelta.
    #[test]
    fn lo_que_se_escribe_se_vuelve_a_leer() {
        for estado in E::TODOS.iter().copied() {
            assert_eq!(
                EstadoDeCampana::desde_texto(estado.como_texto()),
                Ok(estado)
            );
        }
    }

    /// Un estado que no se entiende **no se adivina**.
    ///
    /// Suponer «borrador» dejaría pasar acciones sobre una campaña que podría
    /// estar enviando.
    #[test]
    fn un_estado_desconocido_no_se_adivina() {
        assert!(EstadoDeCampana::desde_texto("running ").is_err());
        assert!(EstadoDeCampana::desde_texto("").is_err());
        assert!(EstadoDeCampana::desde_texto("halted").is_err());
    }

    /// La parada de emergencia dice desde cuándo, no sólo que sí.
    #[test]
    fn la_parada_de_emergencia_dice_desde_cuando() {
        let puesta = ParadaDeEmergencia::Puesta {
            desde: "2026-09-23T14:32:00Z".to_owned(),
            por: Some("adrian".to_owned()),
        };
        assert!(puesta.hay_que_parar());
        assert!(!ParadaDeEmergencia::Levantada.hay_que_parar());
    }
}
