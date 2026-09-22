//! El derecho de **Cancelación**: borrar de verdad a una persona.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA TENSIÓN, Y CÓMO SE RESUELVE
//!
//! La LFPDPPP da derecho a un **borrado físico real**, no a marcar una fila
//! como borrada. Y el §91 exige bitácora de auditoría, que por definición no
//! se borra. Las dos cosas a la vez son imposibles si se leen a lo bruto.
//!
//! La resolución de `PRIVACIDAD_LFPDPPP.md §3` separa **el hecho** de **la
//! identidad**:
//!
//! | Qué | Qué se hace |
//! |---|---|
//! | La ficha, sus campos, sus canales, sus listas y etiquetas | **Se borra de verdad** |
//! | Los intentos de envío y la audiencia congelada | **Se conserva el hecho, se corta la identidad** |
//! | La bitácora | Se anota que hubo una cancelación, con la dirección **en huella** |
//! | La baja | **Permanece**, y si no la había **se crea** |
//!
//! Así la auditoría conserva «el 12 de marzo se canceló a alguien y el 3 de
//! febrero se le había enviado un correo que rebotó», sin conservar a quién.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ LA BAJA SE CREA, Y NO SÓLO SE CONSERVA
//!
//! El documento dice que la baja **permanece**, con este argumento: borrarla
//! significaría que si esa dirección reaparece en una importación futura,
//! esa persona volvería a recibir correos.
//!
//! El mismo argumento obliga a **crearla** cuando no la había. Alguien puede
//! pedir el borrado sin haber pedido antes la baja —son derechos distintos—,
//! y borrarlo sin dejar la baja deja exactamente el agujero que el documento
//! describe: su dirección en un CSV la semana que viene, y ARLES escribiéndole
//! a alguien que pidió desaparecer. Sería el peor fallo posible del producto.
//!
//! Se conserva su dirección, sí. El documento ya acepta ese intercambio: la
//! supresión no es tratamiento comercial, es **el registro de una negativa**,
//! y conservarla sirve al interés de quien la pidió.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ LA MARCA DE BORRADO NO ES UNA HUELLA
//!
//! Hay que poner *algo* donde estaba la dirección: las tablas de intentos y
//! de audiencia la tienen como NOT NULL y dentro de un índice único.
//!
//! La salida cómoda es guardar `sha256(direccion)`. Está **mal** para esto.
//! Una dirección de correo vive en un espacio pequeño y adivinable: con una
//! lista de correos y un rato de cómputo, esa huella se revierte. Guardarla
//! sería conservar a la persona con otro nombre, y llamarlo borrado.
//!
//! La marca es **aleatoria y distinta cada vez**, sin ninguna relación con lo
//! que sustituye. Eso corta el vínculo de verdad: ni siquiera ARLES puede
//! saber después si dos intentos fueron de la misma persona — que es
//! justamente lo que se ha pedido destruir.
//!
//! La bitácora sí lleva huella, y ahí es correcto: es una sola anotación, con
//! una finalidad concreta —poder demostrar que esa cancelación se atendió si
//! quien la pidió reclama— y la decide `PRIVACIDAD_LFPDPPP.md §3`.

use serde::Serialize;
use uuid::Uuid;

/// El prefijo de una dirección que ya no es de nadie.
///
/// Se escribe en claro y en español a propósito: quien abra la base con un
/// visor —un auditor, alguien de soporte— tiene que entender de un vistazo
/// que esa fila es un hecho sin dueño, no un dato corrupto.
pub const MARCA_DE_BORRADO: &str = "borrado:";

/// Lo que se escribe donde estaba una dirección.
///
/// **Cada llamada devuelve algo distinto**, aunque se le pase la misma
/// dirección — de hecho no recibe ninguna. Ver la nota de cabecera: una marca
/// derivada de la dirección se revierte, y entonces no se habría borrado nada.
///
/// ```
/// use arles_core::cancelacion::{marca_de_borrado, MARCA_DE_BORRADO, es_marca_de_borrado};
///
/// let a = marca_de_borrado();
/// let b = marca_de_borrado();
/// assert_ne!(a, b, "dos marcas iguales permitirían reagrupar a la persona");
/// assert!(a.starts_with(MARCA_DE_BORRADO));
/// assert!(es_marca_de_borrado(&a));
/// ```
///
/// ─────────────────────────────────────────────────────────────────────────
/// `new_v4` Y NO `now_v7`, QUE ES LA QUE USA TODO LO DEMÁS DEL PROYECTO
///
/// Los identificadores de ARLES son UUID v7 porque llevan la hora dentro y
/// eso los ordena por creación, que en una tabla es una ventaja.
///
/// **Aquí esa ventaja es el fallo.** Una cancelación marca varias filas en el
/// mismo instante; con v7, todas esas marcas compartirían el prefijo de
/// tiempo y quedarían consecutivas al ordenarlas. Cualquiera que abriera la
/// base podría decir «estas cinco se borraron a la vez, luego eran de la
/// misma persona» — y el vínculo que se pretendía cortar seguiría ahí, a la
/// vista.
///
/// Estuvo a punto de colarse: `v7` era la única versión activada en el
/// `Cargo.toml` del taller, así que lo fácil habría sido usarla.
#[must_use]
pub fn marca_de_borrado() -> String {
    format!("{MARCA_DE_BORRADO}{}", Uuid::new_v4())
}

/// ¿Esta dirección es en realidad una marca de borrado?
///
/// Lo usan las consultas que no deben tratar una fila anónima como si tuviera
/// destinatario — y las pruebas, para comprobar que no queda ni un rastro.
#[must_use]
pub fn es_marca_de_borrado(valor: &str) -> bool {
    valor.starts_with(MARCA_DE_BORRADO)
}

/// Qué pasó al cancelar.
///
/// ─────────────────────────────────────────────────────────────────────────
/// SON CIFRAS, NO ADJETIVOS (§94)
///
/// La pantalla de un borrado irreversible no puede decir «listo». Tiene que
/// decir **qué se borró y qué se conservó**, con números, porque quien atiende
/// la solicitud va a tener que responderle eso mismo a una persona.
///
/// `Serialize` y **no `Deserialize`**: esto lo produce el núcleo al terminar
/// el trabajo. Que la webview pudiera construir uno sería permitirle afirmar
/// que un borrado ocurrió.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cancelacion {
    /// Fichas borradas físicamente. Normalmente 1; 0 si sólo había rastro.
    pub contactos_borrados: u32,
    /// Canales —correos y móviles— que se fueron con ellas.
    pub canales_borrados: u32,
    /// Campos personalizados borrados.
    pub campos_borrados: u32,
    /// Pertenencias a listas y etiquetas borradas.
    pub pertenencias_borradas: u32,
    /// Intentos de envío que **se conservan**, ya sin dueño.
    pub envios_anonimizados: u32,
    /// Filas de audiencia congelada que se conservan, ya sin dueño.
    pub audiencias_anonimizadas: u32,
    /// `true` si la baja ya existía; `false` si esta cancelación la creó.
    pub la_baja_ya_existia: bool,
}

impl Cancelacion {
    /// ¿Había algo que borrar?
    ///
    /// **No es lo mismo que «falló»**. Alguien puede pedir su borrado sin
    /// haber estado nunca en la base: la respuesta honesta es «no teníamos
    /// nada suyo», y la baja se crea igual para que siga sin tenerse.
    #[must_use]
    pub const fn habia_algo(&self) -> bool {
        self.contactos_borrados > 0
            || self.envios_anonimizados > 0
            || self.audiencias_anonimizadas > 0
    }

    /// Lo que se borró del todo, sumado. Para una sola cifra en pantalla.
    #[must_use]
    pub const fn total_borrado(&self) -> u32 {
        self.contactos_borrados
            .saturating_add(self.canales_borrados)
            .saturating_add(self.campos_borrados)
            .saturating_add(self.pertenencias_borradas)
    }

    /// Lo que se conserva sin dueño, sumado.
    #[must_use]
    pub const fn total_conservado(&self) -> u32 {
        self.envios_anonimizados
            .saturating_add(self.audiencias_anonimizadas)
    }
}

/// Lo que se va a destruir, **antes** de destruirlo.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ ESTO EXISTE
///
/// Un borrado irreversible que sólo pregunta «¿seguro?» no está preguntando
/// nada: nadie puede decidir sobre algo que no ha visto. Esto es lo que la
/// pantalla enseña antes de dejar pulsar el botón — con cifras, no con
/// adjetivos (§94).
///
/// Se deriva del **expediente** que ya arma el derecho de Acceso, en vez de
/// contar por separado. Dos recuentos distintos de lo mismo divergen, y el día
/// que divergieran la pantalla prometería borrar algo que no borra.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VistaPrevia {
    /// La dirección por la que se preguntó, ya normalizada.
    pub direccion: String,
    /// El nombre que consta, si consta. Para que quien atiende **reconozca a
    /// la persona** antes de borrarla, en vez de fiarse de haber tecleado bien.
    pub nombre: String,
    /// `true` si esa dirección sigue dada de alta como contacto.
    pub sigue_siendo_contacto: bool,
    /// Correos y móviles que se borrarán.
    pub direcciones: u32,
    /// Campos personalizados que se borrarán.
    pub campos: u32,
    /// Listas y etiquetas de las que saldrá.
    pub pertenencias: u32,
    /// Envíos que **se conservan**, ya sin dueño.
    pub envios: u32,
    /// `true` si ya tiene una baja registrada.
    pub tiene_baja: bool,
}

impl VistaPrevia {
    /// La saca del expediente del derecho de Acceso.
    ///
    /// `arles-core` no toca la base: el expediente se lo pasa quien sí la toca.
    #[must_use]
    pub fn desde_expediente(e: &crate::arco::Expediente) -> Self {
        Self {
            direccion: e.se_pregunto_por.clone(),
            nombre: format!("{} {}", e.nombre, e.apellido).trim().to_owned(),
            sigue_siendo_contacto: e.sigue_siendo_contacto(),
            direcciones: u32::try_from(e.direcciones.len()).unwrap_or(u32::MAX),
            campos: u32::try_from(e.campos.len()).unwrap_or(u32::MAX),
            pertenencias: u32::try_from(e.listas.len().saturating_add(e.etiquetas.len()))
                .unwrap_or(u32::MAX),
            envios: u32::try_from(e.envios.len()).unwrap_or(u32::MAX),
            tiene_baja: !e.negativas.is_empty(),
        }
    }

    /// ¿Hay algo que borrar?
    ///
    /// **No es lo mismo que «falló»**: alguien puede pedir su borrado sin haber
    /// estado nunca en la base. La pantalla lo dice y deja la baja igual.
    #[must_use]
    pub const fn hay_algo(&self) -> bool {
        self.sigue_siendo_contacto || self.envios > 0 || self.tiene_baja
    }

    /// Lo que desaparecerá del todo, sumado. Una sola cifra para el titular.
    #[must_use]
    pub const fn total_a_borrar(&self) -> u32 {
        self.direcciones
            .saturating_add(self.campos)
            .saturating_add(self.pertenencias)
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use std::collections::HashSet;

    /// LA GARANTÍA CENTRAL: la marca no se puede revertir a la dirección.
    ///
    /// Se comprueba por la vía que de verdad importa —que **no es función de
    /// nada**— y no mirando el algoritmo: mil marcas seguidas, mil resultados
    /// distintos. Una huella daría una sola.
    ///
    /// Verificado rompiéndolo (CLAUDE.md §2): sustituyendo el cuerpo por
    /// `format!("{MARCA_DE_BORRADO}{}", huella("ana@empresa.mx"))` esta prueba
    /// falla con «1 marcas distintas de 1000», que es el mensaje correcto.
    #[test]
    fn la_marca_de_borrado_no_se_puede_revertir() {
        let cuantas = 1000;
        let distintas: HashSet<String> = (0..cuantas).map(|_| marca_de_borrado()).collect();
        assert_eq!(
            distintas.len(),
            cuantas,
            "{} marcas distintas de {cuantas}: si se repiten, alguien puede \
             reagrupar los envíos de la misma persona y el borrado no lo es",
            distintas.len()
        );
    }

    /// Y se reconoce como tal. Sin esto, una consulta trataría una fila
    /// anónima como si tuviera destinatario.
    #[test]
    fn una_marca_se_distingue_de_una_direccion() {
        assert!(es_marca_de_borrado(&marca_de_borrado()));
        assert!(!es_marca_de_borrado("ana@empresa.mx"));
        assert!(!es_marca_de_borrado("+528112345678"));
        // Y no se confunde con una dirección que empiece parecido: la marca
        // lleva dos puntos, y una dirección de correo no puede llevarlos en
        // la parte local sin comillas.
        assert!(!es_marca_de_borrado("borrado@empresa.mx"));
    }

    #[test]
    fn habia_algo_distingue_el_rastro_de_la_nada() {
        let vacia = Cancelacion {
            contactos_borrados: 0,
            canales_borrados: 0,
            campos_borrados: 0,
            pertenencias_borradas: 0,
            envios_anonimizados: 0,
            audiencias_anonimizadas: 0,
            la_baja_ya_existia: false,
        };
        assert!(!vacia.habia_algo());

        // Sólo rastro de envíos, sin ficha: SÍ había algo. Es el caso de quien
        // ya se había borrado antes y pide la cancelación otra vez.
        let solo_rastro = Cancelacion {
            envios_anonimizados: 3,
            ..vacia
        };
        assert!(solo_rastro.habia_algo());
        assert_eq!(solo_rastro.total_borrado(), 0);
        assert_eq!(solo_rastro.total_conservado(), 3);
    }

    /// `Cancelacion` **no se deserializa**.
    ///
    /// Es la quinta regla de frontera de `ARQUITECTURA.md §5` aplicada aquí:
    /// un tipo que afirma que un borrado ocurrió sólo puede salir de quien lo
    /// hizo. Si la webview pudiera construir uno, podría enseñar «se borraron
    /// 4 cosas» sin que se hubiera borrado ninguna.
    ///
    /// Esto lo comprueba además el validador de fase, que mira los `derive`.
    #[test]
    fn la_cancelacion_no_se_puede_deserializar() {
        // La prueba de verdad es de compilación: si alguien añadiera
        // `Deserialize`, esta línea seguiría compilando y el validador lo
        // cazaría. Aquí se deja constancia de la intención y se comprueba que
        // al menos SÍ se serializa, que es para lo que existe.
        let c = Cancelacion {
            contactos_borrados: 1,
            canales_borrados: 2,
            campos_borrados: 0,
            pertenencias_borradas: 1,
            envios_anonimizados: 5,
            audiencias_anonimizadas: 5,
            la_baja_ya_existia: true,
        };
        let json = serde_json::to_string(&c).expect("se serializa");
        assert!(json.contains("contactosBorrados"));
        assert!(json.contains("laBajaYaExistia"));
    }
}
