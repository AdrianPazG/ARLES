//! La lista de supresión: quién ha dicho que no, y por qué (entrega 3.4).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA LISTA DE SUPRESIÓN ES LO ÚNICO QUE NO SE PUEDE ROMPER
//!
//! Todo lo demás de ARLES tiene arreglo. Una campaña mal configurada se
//! cancela, un contacto mal importado se corrige, una plantilla fea se
//! reescribe. **Escribirle a alguien que pidió que no le escribieran no tiene
//! arreglo**: el correo ya salió, y quien lo recibe no tiene por qué creerse la
//! segunda promesa.
//!
//! Por eso este módulo es corto y cerrado. No hay motivos libres, no hay
//! orígenes libres, y no hay forma de que una fila de supresión llegue a la
//! base sin haber pasado por aquí.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ LA SUPRESIÓN SOBREVIVE AL BORRADO DEL CONTACTO
//!
//! Parece contradictorio: alguien pide «bórrame» y ARLES se queda con su
//! dirección. La contradicción se deshace al preguntar **a quién sirve** cada
//! opción.
//!
//! Si se borrara la supresión junto al contacto, bastaría con que su dirección
//! reapareciera en una importación —la misma lista comprada, el mismo archivo
//! de la feria— para que volviera a recibir correos. El borrado se habría
//! respetado durante seis meses y se habría deshecho solo.
//!
//! La entrada de supresión **no es tratamiento comercial**: es el registro de
//! una negativa. Conservarla es lo que hace que la negativa dure. Está
//! argumentado en `04-seguridad/PRIVACIDAD_LFPDPPP.md §3`, y es exactamente lo
//! que mide la puerta de salida de esta entrega: borrar un contacto,
//! reimportarlo, y comprobar que **sigue** sin escribírsele.
//! ─────────────────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

use crate::canal::Canal;
use crate::contacto::CanalValidado;
use crate::error::CoreError;

/// Por qué una dirección dejó de recibir.
///
/// **Lista cerrada**, y las cadenas de [`MotivoDeSupresion::como_texto`] están
/// escritas literalmente dentro de la `CHECK` de la migración V3. Hay una
/// prueba en `arles-db` que lee la migración y lo comprueba: si alguien añade
/// una variante aquí y no toca el esquema, la fila se rechazaría al escribir —
/// y el momento de enterarse no puede ser cuando alguien pulse «no escribirle
/// más».
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
// Los `rename` fijan el nombre de la base como el único (H-05): ver la nota
// de `arles_core::canal::Canal`.
pub enum MotivoDeSupresion {
    /// El servidor del destinatario rechazó el correo de forma definitiva.
    ///
    /// **En la v1.2.0 sólo llega por el 5xx síncrono de SMTP.** Los rebotes
    /// asíncronos —los que llegan como un correo de vuelta— necesitan leer un
    /// buzón, y eso se difirió a la v1.3 (ADR-0009/0014). Que el motivo exista
    /// no significa que se detecte todo: lo honesto es decirlo.
    #[serde(rename = "hard_bounce")]
    RebotePermanente,
    /// La persona pidió la baja.
    #[serde(rename = "unsubscribe")]
    Baja,
    /// Alguien de la empresa la añadió a mano.
    #[serde(rename = "manual")]
    Manual,
    /// La persona la marcó como correo no deseado.
    #[serde(rename = "complaint")]
    Queja,
    /// El número no existe o no tiene WhatsApp.
    #[serde(rename = "invalid_number")]
    NumeroInvalido,
    /// El canal falló de forma que no se va a arreglar reintentando.
    #[serde(rename = "channel_failure")]
    FalloDelCanal,
}

impl MotivoDeSupresion {
    /// Cómo se escribe en la base. Contrato con la `CHECK` de la V3.
    pub const fn como_texto(self) -> &'static str {
        match self {
            Self::RebotePermanente => "hard_bounce",
            Self::Baja => "unsubscribe",
            Self::Manual => "manual",
            Self::Queja => "complaint",
            Self::NumeroInvalido => "invalid_number",
            Self::FalloDelCanal => "channel_failure",
        }
    }

    pub const TODOS: &'static [Self] = &[
        Self::RebotePermanente,
        Self::Baja,
        Self::Manual,
        Self::Queja,
        Self::NumeroInvalido,
        Self::FalloDelCanal,
    ];

    /// Clave de i18n de su nombre visible (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::RebotePermanente => "supresion.motivo.rebotePermanente",
            Self::Baja => "supresion.motivo.baja",
            Self::Manual => "supresion.motivo.manual",
            Self::Queja => "supresion.motivo.queja",
            Self::NumeroInvalido => "supresion.motivo.numeroInvalido",
            Self::FalloDelCanal => "supresion.motivo.falloDelCanal",
        }
    }

    /// ¿Lo pidió la persona, o lo dedujo una máquina?
    ///
    /// La distinción gobierna [`MotivoDeSupresion::se_puede_levantar`] y no es
    /// cosmética: una baja la decidió alguien, un rebote lo dedujo un servidor.
    #[must_use]
    pub const fn lo_pidio_la_persona(self) -> bool {
        matches!(self, Self::Baja | Self::Queja)
    }

    /// ¿Se puede sacar de la lista desde la aplicación?
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// **NO, SI LA PERSONA LO PIDIÓ.** Y ésta es la regla del módulo.
    ///
    /// Una dirección que entró por un rebote o por un número inválido puede
    /// salir: el servidor pudo equivocarse, el buzón pudo volver, el número
    /// pudo darse de alta en WhatsApp. Es una deducción técnica, y las
    /// deducciones se revisan.
    ///
    /// Una dirección que entró porque **la persona pidió la baja o marcó el
    /// correo como no deseado** no sale con un botón. Si saliera, el trabajo de
    /// respetar esa negativa dependería de que nadie la deshaga por error, por
    /// prisa, o porque «esa lista era buena». La única forma de que esa persona
    /// vuelva a recibir es que vuelva a darse de alta ella.
    ///
    /// Esto **se puede sortear** editando la base a mano, y no se pretende otra
    /// cosa: el objetivo no es impedirlo, es que no pase sin querer.
    /// ─────────────────────────────────────────────────────────────────────
    #[must_use]
    pub const fn se_puede_levantar(self) -> bool {
        !self.lo_pidio_la_persona()
    }

    /// Lee lo que hay guardado en la base.
    ///
    /// # Errores
    ///
    /// [`CoreError::MotivoDeSupresionDesconocido`] si la cadena no es de las
    /// seis. **No se repara cayendo en `Manual`**: el motivo decide si la
    /// supresión se puede levantar, así que adivinarlo mal es decidir mal sobre
    /// si alguien vuelve a recibir correos.
    pub fn desde_texto(texto: &str) -> Result<Self, CoreError> {
        Self::TODOS
            .iter()
            .copied()
            .find(|m| m.como_texto() == texto)
            .ok_or(CoreError::MotivoDeSupresionDesconocido)
    }
}

/// Quién metió la fila.
///
/// Separado del motivo a propósito. El motivo dice **por qué** dejó de recibir;
/// el origen dice **quién lo decidió**. Un `Manual` que viene de `Usuario` y un
/// `Manual` que viene de `Importacion` son la misma decisión tomada por dos
/// manos distintas, y el día que haya que revisar una lista importa cuál fue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
// Los `rename` fijan el nombre de la base como el único (H-05).
pub enum OrigenDeSupresion {
    /// Lo dijo el servidor de correo, en la misma conversación SMTP.
    #[serde(rename = "smtp_5xx")]
    RespuestaSmtp,
    /// Lo hizo una persona desde ARLES.
    #[serde(rename = "user")]
    Usuario,
    /// Venía marcado en un archivo importado.
    #[serde(rename = "import")]
    Importacion,
    /// Lo avisó Meta por el canal de WhatsApp.
    #[serde(rename = "whatsapp_webhook")]
    AvisoDeWhatsApp,
}

impl OrigenDeSupresion {
    /// Contrato con la `CHECK` de la V3.
    pub const fn como_texto(self) -> &'static str {
        match self {
            Self::RespuestaSmtp => "smtp_5xx",
            Self::Usuario => "user",
            Self::Importacion => "import",
            Self::AvisoDeWhatsApp => "whatsapp_webhook",
        }
    }

    pub const TODOS: &'static [Self] = &[
        Self::RespuestaSmtp,
        Self::Usuario,
        Self::Importacion,
        Self::AvisoDeWhatsApp,
    ];

    /// Lee lo que hay guardado en la base.
    ///
    /// # Errores
    ///
    /// [`CoreError::OrigenDeSupresionDesconocido`] si no es de los cuatro.
    pub fn desde_texto(texto: &str) -> Result<Self, CoreError> {
        Self::TODOS
            .iter()
            .copied()
            .find(|o| o.como_texto() == texto)
            .ok_or(CoreError::OrigenDeSupresionDesconocido)
    }
}

/// ¿Dijo que no a este canal, o a todos?
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ HAY DOS, Y NO UNA FILA «GLOBAL»
///
/// «No me escribas por WhatsApp» y «no me escribas» son peticiones distintas, y
/// la segunda no se puede guardar como una sola fila: la unicidad de la tabla
/// es por (empresa, canal, dirección), y una persona tiene **una dirección por
/// canal** — un correo y un móvil que no se parecen en nada.
///
/// Así que una petición global escribe **una fila por canal**, todas con el
/// mismo `request_id`. Ese identificador común es lo único que permite después
/// deshacer la petición entera sin adivinar cuáles de sus filas eran.
/// ─────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
// Los `rename` fijan el nombre de la base como el único (H-05).
pub enum AlcanceDeSupresion {
    /// Sólo este canal. El resto sigue abierto.
    #[serde(rename = "channel")]
    Canal,
    /// Todos los canales de esa persona. Se escribe una fila por cada uno.
    #[serde(rename = "global")]
    Global,
}

impl AlcanceDeSupresion {
    /// Contrato con la `CHECK` de la V3.
    pub const fn como_texto(self) -> &'static str {
        match self {
            Self::Canal => "channel",
            Self::Global => "global",
        }
    }

    pub const TODOS: &'static [Self] = &[Self::Canal, Self::Global];

    /// Lee lo que hay guardado en la base.
    ///
    /// # Errores
    ///
    /// [`CoreError::AlcanceDeSupresionDesconocido`] si no es de los dos.
    pub fn desde_texto(texto: &str) -> Result<Self, CoreError> {
        Self::TODOS
            .iter()
            .copied()
            .find(|a| a.como_texto() == texto)
            .ok_or(CoreError::AlcanceDeSupresionDesconocido)
    }
}

/// Una petición de supresión, ya validada y lista para escribirse.
///
/// ─────────────────────────────────────────────────────────────────────────
/// ESTE TIPO NO SE PUEDE DESERIALIZAR, Y ES DELIBERADO
///
/// Es la quinta regla de frontera de `ARQUITECTURA.md §5`: un tipo validado no
/// entra por JSON. Si la webview pudiera construir una `PeticionDeSupresion`
/// con las direcciones que quisiera, podría escribir en la lista de supresión
/// una dirección **normalizada a su gusto** — y la comprobación que impide
/// enviar compara justo esa forma normalizada.
///
/// El resultado sería una supresión que existe en la tabla, se ve en la
/// pantalla, y **no bloquea nada**. La peor clase de fallo: uno que se ve
/// arreglado.
/// ─────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PeticionDeSupresion {
    /// Los canales a los que afecta, ya validados por el núcleo.
    ///
    /// Con [`AlcanceDeSupresion::Canal`] es exactamente uno. Con `Global` son
    /// todos los que tenga esa persona.
    pub canales: Vec<CanalValidado>,
    pub alcance: AlcanceDeSupresion,
    pub motivo: MotivoDeSupresion,
    pub origen: OrigenDeSupresion,
    /// Nota libre de quien la añadió. Se enseña tal cual.
    pub notas: String,
}

impl PeticionDeSupresion {
    /// Arma una petición, comprobando lo único que se puede comprobar sin base.
    ///
    /// # Errores
    ///
    /// - [`CoreError::SupresionSinCanales`] si no viene ningún canal. Una
    ///   supresión sin dirección no bloquea a nadie; escribirla daría una
    ///   pantalla con una fila vacía y la sensación de que algo se hizo.
    /// - [`CoreError::AlcanceNoCoincideConLosCanales`] si el alcance dice
    ///   «este canal» y vienen varios. **No se arregla solo pasándolo a
    ///   global**: quien pidió una cosa no pidió la otra, y ampliar en silencio
    ///   una supresión es cambiar lo que una persona decidió.
    pub fn nueva(
        canales: Vec<CanalValidado>,
        alcance: AlcanceDeSupresion,
        motivo: MotivoDeSupresion,
        origen: OrigenDeSupresion,
        notas: &str,
    ) -> Result<Self, CoreError> {
        if canales.is_empty() {
            return Err(CoreError::SupresionSinCanales);
        }
        if alcance == AlcanceDeSupresion::Canal && canales.len() > 1 {
            return Err(CoreError::AlcanceNoCoincideConLosCanales);
        }
        Ok(Self {
            canales,
            alcance,
            motivo,
            origen,
            notas: notas.trim().to_owned(),
        })
    }

    /// Los canales distintos que toca, sin repetir.
    ///
    /// Una persona puede traer dos correos; la supresión es por dirección, pero
    /// saber qué canales quedan cerrados es lo que la pantalla enseña.
    #[must_use]
    pub fn canales_afectados(&self) -> Vec<Canal> {
        let mut vistos: Vec<Canal> = Vec::new();
        for c in &self.canales {
            if !vistos.contains(&c.canal) {
                vistos.push(c.canal);
            }
        }
        vistos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canal::ValorDeCanal;

    /// Un canal validado por el camino de verdad: se parsea con el validador
    /// del núcleo, no se construye a mano. Construirlo a mano dejaría pasar un
    /// `valor_normalizado` que el validador nunca produciría, y toda la
    /// supresión compara justo esa forma.
    fn canal(c: Canal, valor: &str) -> CanalValidado {
        let v = ValorDeCanal::parse(c, valor, "MX").expect("el valor de prueba es válido");
        CanalValidado {
            canal: v.canal(),
            valor_raw: v.raw().to_owned(),
            valor_normalizado: v.normalized().to_owned(),
            principal: true,
        }
    }

    fn correo(valor: &str) -> CanalValidado {
        canal(Canal::Correo, valor)
    }

    fn movil(valor: &str) -> CanalValidado {
        canal(Canal::WhatsApp, valor)
    }

    /// Ida y vuelta por las tres listas cerradas. Sin esto, una variante nueva
    /// puede escribirse en la base y no volver a leerse.
    #[test]
    fn las_tres_listas_van_y_vuelven() {
        for m in MotivoDeSupresion::TODOS {
            assert_eq!(MotivoDeSupresion::desde_texto(m.como_texto()), Ok(*m));
        }
        for o in OrigenDeSupresion::TODOS {
            assert_eq!(OrigenDeSupresion::desde_texto(o.como_texto()), Ok(*o));
        }
        for a in AlcanceDeSupresion::TODOS {
            assert_eq!(AlcanceDeSupresion::desde_texto(a.como_texto()), Ok(*a));
        }
    }

    /// Una cadena que no reconocemos es un dato corrupto, **no** un caso a
    /// reparar en silencio. Reparar un motivo desconocido como `Manual`
    /// convertiría una baja pedida por una persona en algo que se puede
    /// levantar con un botón.
    #[test]
    fn lo_desconocido_da_error_y_no_cae_en_un_valor_por_defecto() {
        assert_eq!(
            MotivoDeSupresion::desde_texto("porque si"),
            Err(CoreError::MotivoDeSupresionDesconocido)
        );
        assert_eq!(
            MotivoDeSupresion::desde_texto(""),
            Err(CoreError::MotivoDeSupresionDesconocido)
        );
        // Y no se normaliza la caja: la base guarda exactamente estas cadenas.
        assert_eq!(
            MotivoDeSupresion::desde_texto("MANUAL"),
            Err(CoreError::MotivoDeSupresionDesconocido)
        );
        assert_eq!(
            OrigenDeSupresion::desde_texto("otro"),
            Err(CoreError::OrigenDeSupresionDesconocido)
        );
        assert_eq!(
            AlcanceDeSupresion::desde_texto("todos"),
            Err(CoreError::AlcanceDeSupresionDesconocido)
        );
    }

    /// **La regla del módulo.** Lo que pidió una persona no se deshace con un
    /// botón; lo que dedujo una máquina, sí.
    #[test]
    fn solo_se_levanta_lo_que_dedujo_una_maquina() {
        assert!(!MotivoDeSupresion::Baja.se_puede_levantar());
        assert!(!MotivoDeSupresion::Queja.se_puede_levantar());

        assert!(MotivoDeSupresion::RebotePermanente.se_puede_levantar());
        assert!(MotivoDeSupresion::NumeroInvalido.se_puede_levantar());
        assert!(MotivoDeSupresion::FalloDelCanal.se_puede_levantar());
        assert!(MotivoDeSupresion::Manual.se_puede_levantar());

        // Y las dos caras son la misma moneda: si esto dejara de cumplirse,
        // habría un motivo que la persona pidió y que aun así se levanta.
        for m in MotivoDeSupresion::TODOS {
            assert_eq!(
                m.se_puede_levantar(),
                !m.lo_pidio_la_persona(),
                "{} dice una cosa y la contraria",
                m.como_texto()
            );
        }
    }

    #[test]
    fn una_supresion_sin_canales_no_se_puede_armar() {
        assert_eq!(
            PeticionDeSupresion::nueva(
                Vec::new(),
                AlcanceDeSupresion::Global,
                MotivoDeSupresion::Baja,
                OrigenDeSupresion::Usuario,
                "",
            ),
            Err(CoreError::SupresionSinCanales)
        );
    }

    /// Pedir «este canal» con varios canales no se arregla ampliando a global:
    /// ampliar en silencio una supresión cambia lo que una persona decidió.
    #[test]
    fn un_alcance_de_un_canal_con_varios_canales_se_rechaza() {
        assert_eq!(
            PeticionDeSupresion::nueva(
                vec![correo("ana@empresa.mx"), movil("8110000000")],
                AlcanceDeSupresion::Canal,
                MotivoDeSupresion::Manual,
                OrigenDeSupresion::Usuario,
                "",
            ),
            Err(CoreError::AlcanceNoCoincideConLosCanales)
        );
    }

    #[test]
    fn una_peticion_global_con_dos_canales_se_arma_y_los_cuenta() {
        let p = PeticionDeSupresion::nueva(
            vec![correo("ana@empresa.mx"), movil("8110000000")],
            AlcanceDeSupresion::Global,
            MotivoDeSupresion::Baja,
            OrigenDeSupresion::Usuario,
            "  lo pidió por teléfono  ",
        )
        .expect("se arma");

        assert_eq!(p.canales_afectados().len(), 2);
        assert_eq!(
            p.notas, "lo pidió por teléfono",
            "las notas no se recortaron"
        );
    }

    /// Dos correos de la misma persona son un solo canal afectado. La pantalla
    /// dice «no recibe por correo», no «no recibe por correo y por correo».
    #[test]
    fn dos_direcciones_del_mismo_canal_cuentan_como_un_canal() {
        let p = PeticionDeSupresion::nueva(
            vec![correo("ana@empresa.mx"), correo("ana.torres@empresa.mx")],
            AlcanceDeSupresion::Global,
            MotivoDeSupresion::Manual,
            OrigenDeSupresion::Usuario,
            "",
        )
        .expect("se arma");

        assert_eq!(p.canales_afectados(), vec![Canal::Correo]);
        assert_eq!(p.canales.len(), 2, "las dos direcciones sí se conservan");
    }
}
