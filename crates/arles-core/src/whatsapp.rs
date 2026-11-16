//! Lo que WhatsApp tiene de distinto, y no es un detalle.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! ARLES NO MANDA EL TEXTO. MANDA EL NOMBRE DE UNA PLANTILLA Y SUS HUECOS.
//!
//! En correo, ARLES compone el mensaje entero y lo entrega. En WhatsApp **no
//! puede**: el texto de una plantilla lo aprueba Meta, palabra por palabra, y
//! lo que se manda por la API es el **nombre** de esa plantilla más los valores
//! de sus huecos.
//!
//! Esa diferencia es lo que gobierna este archivo, y tiene una consecuencia que
//! conviene celebrar en vez de sufrir: **desde ARLES no se puede alterar el
//! texto de un mensaje de WhatsApp**. Ni queriendo. Eso cierra por construcción
//! la puerta que la L-8 cierra por decisión —mutar texto para esquivar filtros
//! es evasión, y convertiría a ARLES en una herramienta de spam—, y la cierra
//! en el único canal donde de verdad se podría intentar.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LOS HUECOS DE META SON POSICIONALES, Y LOS DE ARLES TIENEN NOMBRE
//!
//! Meta numera: `{{1}}`, `{{2}}`. ARLES nombra: `{{nombre}}`, `{{empresa}}`.
//! Alguien tiene que decir cuál va en cuál, y ese alguien es quien da de alta
//! la plantilla —porque es quien la escribió en Meta y sabe qué pidió—.
//!
//! Se guarda **la lista ordenada** y no un mapa: el orden ES el significado.
//! Con un mapa, dos plantillas con los mismos huecos en distinto orden serían
//! indistinguibles, y el mensaje saldría con el apellido donde va el nombre —
//! sin que nada fallara, porque los dos son texto.

use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::plantilla::{Valores, Variable};

/// En qué modo trabaja un número (L-9).
///
/// ─────────────────────────────────────────────────────────────────────────
/// SON DOS COSAS SEPARADAS, NO UNA CON UNA CASILLA
///
/// La mayoría de quien usa esto sólo necesita el primero, y hoy la única forma
/// de ofrecérselo sería dentro de la misma función que lo expone al segundo.
///
/// **El modo vive en la cuenta, no en la campaña.** Es una propiedad del número
/// y de lo que su dueño aceptó, no de lo que se le ocurra a cada campaña: si
/// colgara de la campaña, cualquiera podría escribir en frío desde un número
/// dado de alta para seguimiento, y la advertencia que se aceptó al darlo de
/// alta no cubriría eso.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModoDelCanal {
    /// Sólo a quien dio permiso o escribió primero. **Sin riesgo.**
    Seguimiento,
    /// En frío. Es lo único de ARLES que puede acabar con un número
    /// inhabilitado, y eso no se recupera: se pierde el número.
    ProspeccionDirecta,
}

impl ModoDelCanal {
    /// Cómo se guarda en la columna `modo`. **Es un contrato con la base.**
    #[must_use]
    pub const fn como_se_guarda(self) -> &'static str {
        match self {
            Self::Seguimiento => "seguimiento",
            Self::ProspeccionDirecta => "prospeccion_directa",
        }
    }

    /// Lee el modo guardado.
    ///
    /// # Errors
    /// [`CoreError::ModoDeCanalDesconocido`] si la base trae algo que no
    /// reconocemos. **No se aproxima a `Seguimiento`**: leer mal un modo en
    /// frío como si fuera de seguimiento haría desaparecer todas las
    /// advertencias de la pantalla, y los mensajes saldrían igual.
    pub fn de_guardado(valor: &str) -> Result<Self, CoreError> {
        match valor {
            "seguimiento" => Ok(Self::Seguimiento),
            "prospeccion_directa" => Ok(Self::ProspeccionDirecta),
            _ => Err(CoreError::ModoDeCanalDesconocido),
        }
    }

    /// ¿Este modo escribe a quien no dio permiso?
    #[must_use]
    pub const fn escribe_en_frio(self) -> bool {
        matches!(self, Self::ProspeccionDirecta)
    }

    /// La clave del texto que lo describe (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::Seguimiento => "whatsapp.modo.seguimiento",
            Self::ProspeccionDirecta => "whatsapp.modo.prospeccionDirecta",
        }
    }
}

/// Qué dice Meta de una plantilla.
///
/// Los cinco estados son los de la API, y **ninguno se puede cambiar desde
/// ARLES**: se leen. Están todos porque cada uno lleva a algo distinto, y
/// fundirlos en «sirve / no sirve» quitaría justo lo que hay que hacer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstadoEnMeta {
    /// Se puede usar.
    Aprobada,
    /// Meta la está revisando. Días, no minutos: no se espera mirando.
    EnRevision,
    /// Meta la rechazó. Hay que reescribirla y volver a mandarla.
    Rechazada,
    /// Pausada por mala reacción de la gente. **Se recupera sola** si mejora,
    /// así que no es lo mismo que rechazada y no se arregla reescribiendo.
    Pausada,
    /// Deshabilitada del todo tras repetirse lo anterior. **Terminal.**
    Deshabilitada,
}

impl EstadoEnMeta {
    /// ¿Se puede mandar un mensaje con esta plantilla?
    ///
    /// **Sólo `Aprobada`**, y está escrito así a propósito: enumerando los
    /// estados que se rechazan, un estado nuevo de Meta pasaría a enviar hasta
    /// que alguien se acordara de prohibirlo. Es la misma forma que la
    /// comprobación de la cuenta de correo (§9 de `CLAUDE.md`).
    #[must_use]
    pub const fn se_puede_usar(self) -> bool {
        matches!(self, Self::Aprobada)
    }

    /// Cómo se guarda. **Contrato con la base.**
    #[must_use]
    pub const fn como_se_guarda(self) -> &'static str {
        match self {
            Self::Aprobada => "approved",
            Self::EnRevision => "pending",
            Self::Rechazada => "rejected",
            Self::Pausada => "paused",
            Self::Deshabilitada => "disabled",
        }
    }

    /// Lee el estado guardado, o el que contesta Meta.
    ///
    /// # Errors
    /// [`CoreError::EstadoDePlantillaDesconocido`] ante un valor que no
    /// reconocemos. **No se aproxima a `Aprobada`**: eso mandaría mensajes con
    /// una plantilla que Meta no aceptó, y cada rechazo cuenta contra el número.
    pub fn de_guardado(valor: &str) -> Result<Self, CoreError> {
        match valor {
            "approved" => Ok(Self::Aprobada),
            "pending" => Ok(Self::EnRevision),
            "rejected" => Ok(Self::Rechazada),
            "paused" => Ok(Self::Pausada),
            "disabled" => Ok(Self::Deshabilitada),
            _ => Err(CoreError::EstadoDePlantillaDesconocido),
        }
    }

    /// La clave del texto (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::Aprobada => "whatsapp.plantilla.aprobada",
            Self::EnRevision => "whatsapp.plantilla.enRevision",
            Self::Rechazada => "whatsapp.plantilla.rechazada",
            Self::Pausada => "whatsapp.plantilla.pausada",
            Self::Deshabilitada => "whatsapp.plantilla.deshabilitada",
        }
    }
}

/// Una plantilla que existe en Meta, tal como ARLES la conoce.
///
/// El `cuerpo` se guarda **para poder enseñarlo**, no para mandarlo: lo que
/// viaja a la API es el nombre y los huecos. Ver la nota de la cabecera.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantillaDeMeta {
    /// El `name` de Meta. Es el identificador real del mensaje.
    pub nombre: String,
    /// El idioma, en la forma de Meta: `es_MX`.
    pub idioma: String,
    pub estado: EstadoEnMeta,
    /// Qué variable de ARLES va en cada `{{n}}`, **en orden**.
    pub huecos: Vec<Variable>,
    /// El texto aprobado, para enseñarlo en pantalla.
    pub cuerpo: String,
}

/// Un mensaje listo para la API: el nombre de la plantilla y sus huecos ya
/// rellenados, en orden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MensajeDePlantilla {
    pub nombre: String,
    pub idioma: String,
    /// Los valores de `{{1}}`, `{{2}}`… en orden. Nunca vacíos: ver
    /// [`PlantillaDeMeta::preparar`].
    pub valores: Vec<String>,
    /// Qué variables no tenían valor para este contacto.
    ///
    /// Se devuelve igual que en las plantillas de correo, para que el preflight
    /// pueda contar a cuántos les falta un dato **antes** de mandar nada.
    pub faltaron: Vec<Variable>,
}

impl PlantillaDeMeta {
    /// Prepara el mensaje de un contacto.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// UN HUECO VACÍO NO SE MANDA VACÍO: SE RELLENA CON LA RESERVA
    ///
    /// Y aquí la razón no es de estilo, es que **la API lo rechaza**: Meta
    /// devuelve error ante un parámetro en blanco, y ese rechazo cuenta como
    /// intento fallido contra la calidad del número.
    ///
    /// Así que un hueco sin valor va con `reserva`, y **se apunta en
    /// `faltaron`** para que el preflight pueda decir a cuánta gente le va a
    /// tocar. Tragárselo en silencio mandaría «Hola cliente» a media lista sin
    /// que nadie lo supiera hasta leer los mensajes enviados.
    ///
    /// # Errors
    /// [`CoreError::PlantillaNoAprobada`] si Meta no la ha aprobado. Se
    /// comprueba **aquí**, en el único sitio por el que pasa todo mensaje de
    /// WhatsApp, y no en quien llama: una comprobación en el llamador se salta
    /// añadiendo otro llamador.
    pub fn preparar(
        &self,
        valores: &Valores,
        reserva: &str,
    ) -> Result<MensajeDePlantilla, CoreError> {
        if !self.estado.se_puede_usar() {
            return Err(CoreError::PlantillaNoAprobada);
        }

        let mut rellenos = Vec::with_capacity(self.huecos.len());
        let mut faltaron = Vec::new();

        for hueco in &self.huecos {
            match valores.de(*hueco) {
                Some(valor) => rellenos.push(valor.to_owned()),
                None => {
                    if !faltaron.contains(hueco) {
                        faltaron.push(*hueco);
                    }
                    rellenos.push(reserva.to_owned());
                }
            }
        }

        Ok(MensajeDePlantilla {
            nombre: self.nombre.clone(),
            idioma: self.idioma.clone(),
            valores: rellenos,
            faltaron,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aprobada() -> PlantillaDeMeta {
        PlantillaDeMeta {
            nombre: "bienvenida_distribuidores".to_owned(),
            idioma: "es_MX".to_owned(),
            estado: EstadoEnMeta::Aprobada,
            huecos: vec![Variable::Nombre, Variable::MiEmpresa],
            cuerpo: "Hola {{1}}, te escribe {{2}}.".to_owned(),
        }
    }

    /// Los huecos salen **en el orden de la plantilla**, no en el del mapa.
    ///
    /// Es la prueba que sostiene toda la traducción: Meta numera y ARLES nombra.
    /// Si el orden se perdiera, el mensaje saldría con el apellido donde va el
    /// nombre — y **no fallaría nada**, porque los dos son texto.
    ///
    /// Verificada rompiéndola el 29/09/2026: recorriendo el mapa de `Valores`
    /// en vez de `self.huecos`, los valores salen en orden alfabético de
    /// variable y esta prueba falla.
    #[test]
    fn los_huecos_salen_en_el_orden_de_la_plantilla() {
        let valores = Valores::nuevos()
            .con(Variable::MiEmpresa, "TELEMETRY")
            .con(Variable::Nombre, "Ana");

        let m = aprobada().preparar(&valores, "cliente").expect("prepara");
        assert_eq!(m.valores, vec!["Ana", "TELEMETRY"], "{{1}} es el nombre");
        assert!(m.faltaron.is_empty());
    }

    /// **Un hueco sin valor se rellena, y se dice que faltaba.**
    ///
    /// Las dos cosas. Mandarlo vacío hace que Meta rechace el mensaje, y ese
    /// rechazo cuenta contra la calidad del número. Y tragárselo en silencio
    /// mandaría «Hola cliente» a media lista sin que nadie lo supiera.
    ///
    /// Verificada rompiéndola el 29/09/2026: devolviendo `faltaron` siempre
    /// vacío, la segunda mitad de esta prueba falla.
    #[test]
    fn un_hueco_sin_valor_se_rellena_y_se_apunta() {
        let valores = Valores::nuevos().con(Variable::MiEmpresa, "TELEMETRY");

        let m = aprobada().preparar(&valores, "cliente").expect("prepara");
        assert_eq!(m.valores, vec!["cliente", "TELEMETRY"], "nada va en blanco");
        assert_eq!(m.faltaron, vec![Variable::Nombre], "y consta quién faltó");
    }

    /// **Una plantilla que Meta no aprobó no se manda.**
    ///
    /// Los cuatro estados que no son `Aprobada` van juntos porque el error es
    /// el mismo: no se manda. Lo que cambia es qué hacer, y eso lo dice la
    /// pantalla.
    ///
    /// Verificada rompiéndola el 29/09/2026: quitando la comprobación de
    /// `se_puede_usar`, los cuatro casos devuelven `Ok` y esta prueba falla en
    /// el primero.
    #[test]
    fn una_plantilla_sin_aprobar_no_se_manda() {
        for estado in [
            EstadoEnMeta::EnRevision,
            EstadoEnMeta::Rechazada,
            EstadoEnMeta::Pausada,
            EstadoEnMeta::Deshabilitada,
        ] {
            let p = PlantillaDeMeta {
                estado,
                ..aprobada()
            };
            assert!(
                matches!(
                    p.preparar(&Valores::nuevos(), "x"),
                    Err(CoreError::PlantillaNoAprobada)
                ),
                "{estado:?} no puede mandar"
            );
        }
    }

    /// **Sólo `Aprobada` sirve, y se comprueba por lo que es, no por lo que no.**
    ///
    /// El día que Meta añada un estado nuevo, `de_guardado` fallará al leerlo
    /// —que es lo correcto— y `se_puede_usar` diría que no aunque se colara.
    /// Escrito al revés —enumerando los que se rechazan— un estado nuevo
    /// enviaría mensajes hasta que alguien se acordara de prohibirlo.
    #[test]
    fn solo_aprobada_sirve() {
        assert!(EstadoEnMeta::Aprobada.se_puede_usar());
        for otro in [
            EstadoEnMeta::EnRevision,
            EstadoEnMeta::Rechazada,
            EstadoEnMeta::Pausada,
            EstadoEnMeta::Deshabilitada,
        ] {
            assert!(!otro.se_puede_usar(), "{otro:?}");
        }
    }

    /// **Un modo que no se reconoce no se lee como «seguimiento».**
    ///
    /// Es el error que no falla: leer mal un número en frío como si fuera de
    /// seguimiento haría desaparecer todas las advertencias de la pantalla, y
    /// los mensajes saldrían igual — en frío, y sin que nadie lo hubiera dicho.
    ///
    /// Verificada rompiéndola el 29/09/2026: devolviendo `Seguimiento` en el
    /// caso por defecto, esta prueba falla.
    #[test]
    fn un_modo_desconocido_no_se_aproxima_al_seguro() {
        assert_eq!(
            ModoDelCanal::de_guardado("seguimiento"),
            Ok(ModoDelCanal::Seguimiento)
        );
        assert_eq!(
            ModoDelCanal::de_guardado("prospeccion_directa"),
            Ok(ModoDelCanal::ProspeccionDirecta)
        );
        assert!(ModoDelCanal::de_guardado("modo_nuevo").is_err());
    }

    /// Los valores guardados son un contrato con la base: si cambian, las filas
    /// escritas hasta hoy dejan de leerse. Esta prueba los congela.
    #[test]
    fn los_valores_guardados_estan_congelados() {
        assert_eq!(ModoDelCanal::Seguimiento.como_se_guarda(), "seguimiento");
        assert_eq!(
            ModoDelCanal::ProspeccionDirecta.como_se_guarda(),
            "prospeccion_directa"
        );
        for e in [
            EstadoEnMeta::Aprobada,
            EstadoEnMeta::EnRevision,
            EstadoEnMeta::Rechazada,
            EstadoEnMeta::Pausada,
            EstadoEnMeta::Deshabilitada,
        ] {
            assert_eq!(
                EstadoEnMeta::de_guardado(e.como_se_guarda()),
                Ok(e),
                "{e:?} tiene que poder volver de la base"
            );
        }
    }

    /// Sólo uno de los dos modos escribe sin permiso, y se dice con una
    /// función en vez de con un `match` repartido por el programa.
    #[test]
    fn solo_la_prospeccion_escribe_en_frio() {
        assert!(!ModoDelCanal::Seguimiento.escribe_en_frio());
        assert!(ModoDelCanal::ProspeccionDirecta.escribe_en_frio());
    }
}
