//! Las etapas de una campaña, y las reglas que las ordenan.
//!
//! Decisión **L-1** de `documentacion/01-producto/LOGISTICA_DE_CAMPANAS.md`.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ ES UNA ETAPA
//!
//! Una campaña tiene **una o dos etapas**, y cada etapa es de **un solo canal**.
//! Lo que Dirección pidió es una secuencia sobre la misma tabla de contactos:
//! sale el correo, y quien cumpla la condición recibe después el WhatsApp.
//!
//! Lo que no se mezcla es el **envío**, no la campaña. Cada etapa tiene su
//! remitente, su plantilla, su ritmo, su ventana y **su propio estado**: eso es
//! lo que permite apagar WhatsApp sin apagar el correo (L-6), dar un número de
//! preflight por etapa en vez de una suma sin sentido, y registrar el permiso
//! por canal, que es lo que pide Meta si alguien reclama.
//! ─────────────────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

use crate::canal::Canal;

/// Cuántas etapas caben. Dos: el correo y el WhatsApp.
///
/// No es una limitación técnica sino una decisión de producto. Una secuencia de
/// cinco pasos es una herramienta de automatización, y eso es otro producto —
/// con otro precio, otra curva de aprendizaje y otra conversación sobre riesgo.
/// Si algún día se amplía, que sea por una decisión escrita y no porque el
/// número estaba suelto.
pub const MAX_ETAPAS: usize = 2;

/// Tope de la espera entre etapas: 30 días.
///
/// Por encima, la segunda etapa llega cuando el destinatario ya no recuerda la
/// primera, y una campaña que sigue «en curso» un mes después es una campaña
/// que nadie va a revisar. No es un límite del motor: es el punto donde la
/// secuencia deja de ser una secuencia.
pub const MAX_ESPERA_HORAS: u32 = 24 * 30;

/// Qué tiene que cumplirse para que a alguien le llegue la segunda etapa.
///
/// ── Lo que NO puede estar aquí, y por qué ──
///
/// La condición natural sería «sólo a quien no contestó el correo». **No se
/// puede saber**: detectarlo exige leer el buzón, un permiso que v1.2.0 no pide
/// y que no queremos pedir (§69). Lo mismo con «sólo a quien lo abrió»: la
/// apertura está descartada porque Apple precarga las imágenes desde 2021 y el
/// dato es ruido (ver `FUERA_DE_ALCANCE.md`).
///
/// Y tampoco puede estar «sólo si el número existe en WhatsApp»: no hay forma
/// legítima de comprobarlo (L-7).
///
/// => Queda lo que ARLES **sí** sabe de primera mano: qué pasó con su propio
/// intento de la etapa anterior. Ofrecer una condición que no se puede evaluar
/// sería peor que no ofrecerla: la campaña se activaría y el filtro no filtraría
/// nada, sin que nadie se entere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CondicionDeEtapa {
    /// A todo el que tenga el canal y pase supresión y consentimiento.
    Siempre,
    /// Sólo a quien el intento anterior **no** le falló de forma definitiva.
    ///
    /// Un correo rebotado con 5xx dice que esa persona no existe en ese canal;
    /// insistirle por WhatsApp después de eso es exactamente el patrón que hace
    /// que a uno lo reporten.
    SoloSiLaAnteriorNoFallo,
}

impl CondicionDeEtapa {
    pub const fn como_texto(self) -> &'static str {
        match self {
            Self::Siempre => "always",
            Self::SoloSiLaAnteriorNoFallo => "previous_not_failed",
        }
    }

    pub const TODAS: &'static [Self] = &[Self::Siempre, Self::SoloSiLaAnteriorNoFallo];

    /// Lee lo que hay guardado en la base.
    ///
    /// # Errores
    ///
    /// [`ErrorDeSecuencia::CondicionDesconocida`] si la cadena no es de las dos.
    pub fn desde_texto(texto: &str) -> Result<Self, ErrorDeSecuencia> {
        match texto {
            "always" => Ok(Self::Siempre),
            "previous_not_failed" => Ok(Self::SoloSiLaAnteriorNoFallo),
            _ => Err(ErrorDeSecuencia::CondicionDesconocida),
        }
    }
}

/// Una etapa tal y como la plantea quien construye la campaña.
///
/// Es sólo lo que hace falta para decidir si la **secuencia** es válida. La
/// plantilla, el remitente y la ventana viven en la base y no intervienen en
/// esta comprobación: una secuencia mal ordenada lo está aunque las tres estén
/// bien elegidas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EtapaPlanificada {
    /// 1 para la primera, 2 para la segunda.
    pub posicion: u8,
    pub canal: Canal,
    pub condicion: CondicionDeEtapa,
    /// Horas de espera desde que termina la etapa anterior.
    pub espera_horas: u32,
}

/// Por qué una secuencia de etapas no se sostiene.
///
/// Una por motivo, no un «campaña inválida»: quien la está construyendo tiene
/// que saber **qué** corregir (§95).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ErrorDeSecuencia {
    #[error("una campaña necesita al menos una etapa")]
    SinEtapas,

    #[error("una campaña admite como mucho dos etapas")]
    DemasiadasEtapas,

    #[error("las etapas tienen que numerarse 1 y 2, sin huecos ni repeticiones")]
    PosicionesMal,

    #[error("las dos etapas son del mismo canal")]
    CanalRepetido,

    #[error("la primera etapa no puede depender de una anterior")]
    LaPrimeraNoEspera,

    #[error("la espera entre etapas supera los 30 días")]
    EsperaDesmesurada,

    #[error("la condición de la etapa no es una de las admitidas")]
    CondicionDesconocida,
}

impl ErrorDeSecuencia {
    /// Clave estable para que la interfaz traduzca el error con i18n (§139).
    pub fn clave_i18n(self) -> &'static str {
        match self {
            Self::SinEtapas => "error.etapa.sin_etapas",
            Self::DemasiadasEtapas => "error.etapa.demasiadas",
            Self::PosicionesMal => "error.etapa.posiciones",
            Self::CanalRepetido => "error.etapa.canal_repetido",
            Self::LaPrimeraNoEspera => "error.etapa.primera_no_espera",
            Self::EsperaDesmesurada => "error.etapa.espera_desmesurada",
            Self::CondicionDesconocida => "error.etapa.condicion_desconocida",
        }
    }
}

/// Comprueba que un conjunto de etapas forma una secuencia que se puede ejecutar.
///
/// Se valida **aquí y no en la pantalla**, por la misma razón que la empresa: dos
/// validaciones son dos reglas que mantener iguales, y el día que divergen el
/// formulario aprueba lo que el motor rechaza.
///
/// # Errores
///
/// El primer [`ErrorDeSecuencia`] que se encuentre, en el orden en que están
/// escritas las comprobaciones: primero cuántas hay, después cómo están
/// numeradas, y sólo entonces lo que dicen.
pub fn validar_secuencia(etapas: &[EtapaPlanificada]) -> Result<(), ErrorDeSecuencia> {
    if etapas.is_empty() {
        return Err(ErrorDeSecuencia::SinEtapas);
    }
    if etapas.len() > MAX_ETAPAS {
        return Err(ErrorDeSecuencia::DemasiadasEtapas);
    }

    // Las posiciones son 1 y 2, cada una una vez. Se comprueban como conjunto y
    // no por el orden del vector: quien construye la campaña puede mandarlas en
    // cualquier orden, y aceptar un vector desordenado pero bien numerado es
    // más robusto que exigir que llegue ordenado.
    let mut posiciones: Vec<u8> = etapas.iter().map(|e| e.posicion).collect();
    posiciones.sort_unstable();
    let esperadas: Vec<u8> = (1..=u8::try_from(etapas.len()).unwrap_or(u8::MAX)).collect();
    if posiciones != esperadas {
        return Err(ErrorDeSecuencia::PosicionesMal);
    }

    // Dos etapas del mismo canal no son una secuencia: son dos envíos iguales,
    // y la unicidad de los intentos rechazaría el segundo contacto a contacto.
    // La campaña se activaría y la segunda etapa no enviaría nada, sin error.
    //
    // Se comparan **todos contra todos** en vez de la pareja 0-1: escrito con
    // índices, ampliar `MAX_ETAPAS` dejaría la comprobación mirando sólo las
    // dos primeras y la tercera repetida pasaría sin que nada fallara.
    let mut canales: Vec<&'static str> = etapas.iter().map(|e| e.canal.como_texto()).collect();
    canales.sort_unstable();
    canales.dedup();
    if canales.len() != etapas.len() {
        return Err(ErrorDeSecuencia::CanalRepetido);
    }

    for etapa in etapas {
        if etapa.posicion == 1
            && (etapa.condicion != CondicionDeEtapa::Siempre || etapa.espera_horas != 0)
        {
            return Err(ErrorDeSecuencia::LaPrimeraNoEspera);
        }
        if etapa.espera_horas > MAX_ESPERA_HORAS {
            return Err(ErrorDeSecuencia::EsperaDesmesurada);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn primera(canal: Canal) -> EtapaPlanificada {
        EtapaPlanificada {
            posicion: 1,
            canal,
            condicion: CondicionDeEtapa::Siempre,
            espera_horas: 0,
        }
    }

    fn segunda(canal: Canal, espera_horas: u32) -> EtapaPlanificada {
        EtapaPlanificada {
            posicion: 2,
            canal,
            condicion: CondicionDeEtapa::SoloSiLaAnteriorNoFallo,
            espera_horas,
        }
    }

    /// Lo que Dirección pidió, escrito como prueba.
    #[test]
    fn correo_y_despues_whatsapp_es_una_secuencia_valida() {
        let etapas = [primera(Canal::Correo), segunda(Canal::WhatsApp, 48)];
        assert_eq!(validar_secuencia(&etapas), Ok(()));
    }

    /// Y una campaña de un solo canal sigue siendo lo normal.
    #[test]
    fn una_sola_etapa_es_valida() {
        assert_eq!(validar_secuencia(&[primera(Canal::Correo)]), Ok(()));
        assert_eq!(validar_secuencia(&[primera(Canal::WhatsApp)]), Ok(()));
    }

    #[test]
    fn una_campana_sin_etapas_no_se_puede_ejecutar() {
        assert_eq!(validar_secuencia(&[]), Err(ErrorDeSecuencia::SinEtapas));
    }

    #[test]
    fn tres_etapas_no_caben() {
        let etapas = [
            primera(Canal::Correo),
            segunda(Canal::WhatsApp, 24),
            EtapaPlanificada {
                posicion: 3,
                canal: Canal::Correo,
                condicion: CondicionDeEtapa::Siempre,
                espera_horas: 24,
            },
        ];
        assert_eq!(
            validar_secuencia(&etapas),
            Err(ErrorDeSecuencia::DemasiadasEtapas)
        );
    }

    /// **El error que se activaría sin dar error.** Dos etapas de correo pasan
    /// el preflight, la campaña arranca, y la segunda no envía ni un mensaje:
    /// la unicidad de los intentos rechaza cada fila contacto a contacto,
    /// porque para la base es el mismo envío.
    #[test]
    fn dos_etapas_del_mismo_canal_no_son_una_secuencia() {
        let etapas = [primera(Canal::Correo), segunda(Canal::Correo, 24)];
        assert_eq!(
            validar_secuencia(&etapas),
            Err(ErrorDeSecuencia::CanalRepetido)
        );
    }

    #[test]
    fn las_posiciones_no_admiten_huecos_ni_repeticiones() {
        let repetida = [primera(Canal::Correo), {
            let mut e = segunda(Canal::WhatsApp, 24);
            e.posicion = 1;
            e
        }];
        assert_eq!(
            validar_secuencia(&repetida),
            Err(ErrorDeSecuencia::PosicionesMal)
        );

        let con_hueco = [primera(Canal::Correo), {
            let mut e = segunda(Canal::WhatsApp, 24);
            e.posicion = 3;
            e
        }];
        assert_eq!(
            validar_secuencia(&con_hueco),
            Err(ErrorDeSecuencia::PosicionesMal)
        );
    }

    /// El orden del vector no decide nada: lo decide el número de la etapa.
    #[test]
    fn las_etapas_pueden_llegar_desordenadas() {
        let etapas = [segunda(Canal::WhatsApp, 48), primera(Canal::Correo)];
        assert_eq!(validar_secuencia(&etapas), Ok(()));
    }

    /// La primera etapa no espera a nadie. Si esperara, la campaña quedaría
    /// activada sin enviar nada y sin que la pantalla pudiera explicar por qué.
    #[test]
    fn la_primera_etapa_ni_espera_ni_depende() {
        let mut e = primera(Canal::Correo);
        e.espera_horas = 24;
        assert_eq!(
            validar_secuencia(&[e]),
            Err(ErrorDeSecuencia::LaPrimeraNoEspera)
        );

        let mut e = primera(Canal::Correo);
        e.condicion = CondicionDeEtapa::SoloSiLaAnteriorNoFallo;
        assert_eq!(
            validar_secuencia(&[e]),
            Err(ErrorDeSecuencia::LaPrimeraNoEspera)
        );
    }

    #[test]
    fn una_espera_de_meses_no_es_una_secuencia() {
        let etapas = [
            primera(Canal::Correo),
            segunda(Canal::WhatsApp, MAX_ESPERA_HORAS + 1),
        ];
        assert_eq!(
            validar_secuencia(&etapas),
            Err(ErrorDeSecuencia::EsperaDesmesurada)
        );

        // Justo en el tope sí entra: el límite es el límite, no uno menos.
        let etapas = [
            primera(Canal::Correo),
            segunda(Canal::WhatsApp, MAX_ESPERA_HORAS),
        ];
        assert_eq!(validar_secuencia(&etapas), Ok(()));
    }

    /// Enviar la segunda etapa de inmediato es legítimo y hay que permitirlo:
    /// es lo que hace una campaña que sale por los dos canales el mismo día.
    #[test]
    fn la_segunda_etapa_puede_no_esperar() {
        let etapas = [primera(Canal::Correo), segunda(Canal::WhatsApp, 0)];
        assert_eq!(validar_secuencia(&etapas), Ok(()));
    }

    #[test]
    fn la_condicion_va_y_vuelve_de_su_texto() {
        for c in CondicionDeEtapa::TODAS {
            assert_eq!(CondicionDeEtapa::desde_texto(c.como_texto()), Ok(*c));
        }
        assert_eq!(
            CondicionDeEtapa::desde_texto("si_no_contesto"),
            Err(ErrorDeSecuencia::CondicionDesconocida)
        );
    }

    /// Las cadenas están dentro de las `CHECK` de la migración V4.
    #[test]
    fn los_textos_de_la_condicion_son_los_del_esquema() {
        assert_eq!(CondicionDeEtapa::Siempre.como_texto(), "always");
        assert_eq!(
            CondicionDeEtapa::SoloSiLaAnteriorNoFallo.como_texto(),
            "previous_not_failed"
        );
    }

    #[test]
    fn cada_error_tiene_clave_i18n() {
        for e in [
            ErrorDeSecuencia::SinEtapas,
            ErrorDeSecuencia::DemasiadasEtapas,
            ErrorDeSecuencia::PosicionesMal,
            ErrorDeSecuencia::CanalRepetido,
            ErrorDeSecuencia::LaPrimeraNoEspera,
            ErrorDeSecuencia::EsperaDesmesurada,
            ErrorDeSecuencia::CondicionDesconocida,
        ] {
            assert!(e.clave_i18n().starts_with("error.etapa."));
        }
    }
}
