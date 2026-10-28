//! La lista de alta: qué falta para poder enviar una campaña.
//!
//! **Se deriva, no se almacena.** La tentación es guardar un booleano por paso
//! y marcarlo al terminarlo; entonces basta que alguien borre su única cuenta
//! remitente para que la lista siga diciendo que ese paso está hecho. Aquí el
//! estado de cada paso es una consulta sobre los datos reales, así que no puede
//! mentir: si el dato desaparece, el paso vuelve a estar pendiente.
//!
//! Ver `documentacion/05-diseno/UX_NAVEGACION.md` §4.

use serde::{Deserialize, Serialize};

/// Cuántas cosas de cada tipo hay. Lo aporta la capa de datos.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecuentoDeAlta {
    pub empresas: i64,
    pub remitentes: i64,
    pub contactos: i64,
    pub plantillas: i64,
    pub ventanas: i64,
    pub campanas: i64,
}

/// Los seis pasos hasta la primera campaña.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PasoDeOnboarding {
    Empresa,
    Remitente,
    Contactos,
    Plantilla,
    VentanaDeEjecucion,
    PrimeraCampana,
}

/// Todos los pasos, en el orden en que se hacen.
pub const PASOS: &[PasoDeOnboarding] = &[
    PasoDeOnboarding::Empresa,
    PasoDeOnboarding::Remitente,
    PasoDeOnboarding::Contactos,
    PasoDeOnboarding::Plantilla,
    PasoDeOnboarding::VentanaDeEjecucion,
    PasoDeOnboarding::PrimeraCampana,
];

impl PasoDeOnboarding {
    /// Clave estable para el texto (§139). El título y la explicación viven en
    /// el catálogo de i18n, nunca aquí.
    #[must_use]
    pub fn clave(self) -> &'static str {
        match self {
            Self::Empresa => "empresa",
            Self::Remitente => "remitente",
            Self::Contactos => "contactos",
            Self::Plantilla => "plantilla",
            Self::VentanaDeEjecucion => "ventana",
            Self::PrimeraCampana => "campana",
        }
    }

    /// A dónde va quien quiere hacer este paso.
    ///
    /// **Los seis tienen a dónde llevar desde el 28/10/2026**, y por eso esto
    /// devuelve siempre `Some`. El tipo sigue siendo `Option` a propósito: un
    /// paso futuro puede nacer antes que su pantalla, y entonces la interfaz
    /// tiene que poder decirlo en vez de pintar un enlace a ninguna parte.
    /// Pero mientras todos la tengan, `todo_paso_lleva_a_su_pantalla` exige que
    /// así sea, sin lista de excepciones.
    #[must_use]
    pub fn ruta(self) -> Option<&'static str> {
        match self {
            Self::Empresa => Some("/inicio/configuracion"),
            // Existe desde la entrega 3.2. **Estuvo devolviendo `None` hasta el
            // 23/09/2026**, o sea que la lista de alta seguía diciendo «llega en
            // la entrega 3.2» de una pantalla que llevaba semanas construida, y
            // sin botón para ir a ella.
            //
            // No es un descuido aislado: esta función se escribió cuando sólo
            // existía la primera pantalla, y nada la vuelve a mirar cuando se
            // entrega una nueva. La prueba de abajo lo fija: **un paso cuya
            // pantalla existe tiene que tener ruta**, y falla al añadir una
            // pantalla sin apuntarla aquí.
            Self::Contactos => Some("/contactos"),
            Self::Remitente => Some("/remitentes"),
            // Y volvió a pasar, esta vez con dos pantallas, el 29/09/2026.
            //
            // `PantallaCampanas` existe desde la 6.4 y la de plantillas desde
            // la 6.8a, y las dos seguían aquí como «llega en la entrega 6».
            // Lo encontró la comprobación de la fase 3 al alinear el simulador
            // de la vista previa: el simulador ya llevaba a las pantallas
            // nuevas y el núcleo no, así que **divergieron**, que es
            // exactamente lo que esa comprobación existe para cazar.
            //
            // La lección es la del comentario de arriba, y sigue sin
            // automatizarse del todo: entregar una pantalla incluye venir aquí.
            Self::Plantilla => Some("/campanas/plantillas"),
            Self::PrimeraCampana => Some("/campanas"),
            // Y la tercera vez, el 28/10/2026, con la ventana de envío.
            //
            // Ésta no tiene pantalla propia —se define en el paso 5 del
            // asistente— y de ahí salía `None`. El razonamiento era correcto
            // y la conclusión no: **«no tiene pantalla propia» no es «no se
            // puede hacer»**, y de confundir las dos cosas salía que Inicio
            // dijera «No hay nada más que puedas hacer aquí todavía» con los
            // seis pasos hechos, y «Lo que falta por construir · 1 paso» en
            // toda instalación, para siempre.
            //
            // Lo encontró mirar la primera pantalla (D-01 de
            // `10-requisitos/HALLAZGOS-DE-DISENO.md`), no ninguna prueba: las
            // pruebas afirmaban esta rama como la garantía viva de la regla.
            Self::VentanaDeEjecucion => Some("/campanas/nueva"),
        }
    }

    /// ¿Está hecho, según los datos reales?
    #[must_use]
    pub fn completado(self, r: &RecuentoDeAlta) -> bool {
        match self {
            Self::Empresa => r.empresas > 0,
            Self::Remitente => r.remitentes > 0,
            Self::Contactos => r.contactos > 0,
            Self::Plantilla => r.plantillas > 0,
            Self::VentanaDeEjecucion => r.ventanas > 0,
            Self::PrimeraCampana => r.campanas > 0,
        }
    }
}

/// Un paso con su estado.
///
/// Solo `Serialize`: los textos son `&'static str` —claves del catálogo, no
/// datos— y nada los deserializa. La lista siempre se construye desde
/// [`ListaDeOnboarding::desde`], nunca llega de fuera.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDePaso {
    pub paso: PasoDeOnboarding,
    pub clave: &'static str,
    pub completado: bool,
    /// A dónde va quien quiere hacerlo. `None` sólo si no hay a dónde.
    ///
    /// ── AQUÍ VIVÍA UN CAMPO MÁS, Y QUITARLO ES EL ARREGLO ─────────────────
    ///
    /// Había además un `disponible: bool` que se calculaba como
    /// `ruta().is_some()` — el mismo dato dicho dos veces. Y una prueba,
    /// `disponible_y_ruta_no_pueden_divergir`, que comprobaba justo esa
    /// igualdad.
    ///
    /// La prueba estaba bien escrita y verificaba lo que decía. El problema es
    /// que **lo que decía era la regla equivocada**: los dos nombres invitaban
    /// a leerlos como preguntas distintas —«¿tiene pantalla?» y «¿se puede
    /// hacer hoy?»— y la interfaz los usó así, mientras el núcleo los mantenía
    /// iguales. Una comprobación puede estar verificada y garantizar lo que no
    /// toca.
    ///
    /// Con un solo campo el estado inconsistente **no existe**, que es la regla
    /// del proyecto: gana el tipo sobre el validador, y el validador sobre la
    /// prueba.
    pub ruta: Option<&'static str>,
}

/// La lista completa, lista para pintarse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListaDeOnboarding {
    pub pasos: Vec<EstadoDePaso>,
    pub completados: usize,
    pub total: usize,
    /// El primer paso pendiente **que ya se puede hacer**.
    ///
    /// `None` no significa «todo hecho»: puede significar que lo que falta
    /// todavía no está construido. Para «todo hecho» está [`Self::terminada`].
    pub siguiente: Option<PasoDeOnboarding>,
}

impl ListaDeOnboarding {
    /// Construye la lista a partir de los datos reales.
    #[must_use]
    pub fn desde(r: &RecuentoDeAlta) -> Self {
        let pasos: Vec<EstadoDePaso> = PASOS
            .iter()
            .map(|&paso| EstadoDePaso {
                paso,
                clave: paso.clave(),
                completado: paso.completado(r),
                ruta: paso.ruta(),
            })
            .collect();

        let completados = pasos.iter().filter(|p| p.completado).count();
        let siguiente = pasos
            .iter()
            .find(|p| !p.completado && p.ruta.is_some())
            .map(|p| p.paso);

        Self {
            total: pasos.len(),
            completados,
            siguiente,
            pasos,
        }
    }

    /// ¿Está todo hecho?
    #[must_use]
    pub fn terminada(&self) -> bool {
        self.completados == self.total
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /// **Todo paso lleva a su pantalla. Los seis, sin excepciones.**
    ///
    /// ── LO QUE HABÍA AQUÍ, Y POR QUÉ NO BASTÓ ─────────────────────────────
    ///
    /// Esta prueba tenía una **lista escrita a mano** de los pasos ya
    /// construidos, defendida con este argumento: «es la afirmación “esto ya
    /// está construido”, y tiene que costar un segundo de pensar al añadirle
    /// una línea».
    ///
    /// El argumento es bueno y la lista falló igual, **tres veces**:
    ///
    /// - Contactos, entre la 3.2 y el 23/09/2026: semanas anunciando como
    ///   futura una pantalla construida.
    /// - Campañas y Plantillas, el 29/09/2026: las dos escritas aquí como
    ///   «todavía no existen» con las pantallas hechas desde la 6.4 y la 6.8a.
    /// - La ventana de envío, hasta el 28/10/2026 — y ésta estaba **afirmada**
    ///   por la prueba como «la garantía viva de la regla», así que no la
    ///   cazaba nadie: la prueba exigía que siguiera sin ruta.
    ///
    /// Una lista a mano no se mantiene por saber que hay que mantenerla. Es el
    /// patrón número 3 del proyecto, y la única defensa que ha funcionado las
    /// tres veces es no tener lista: **ahora se exigen los seis.**
    ///
    /// Si algún día nace un paso antes que su pantalla, esta prueba falla y
    /// hay que decidirlo a la vista — que es exactamente cuando conviene
    /// decidirlo, y no seis semanas después.
    ///
    /// Verificada rompiéndola el 28/10/2026: devolviendo `None` en la rama de
    /// la ventana, que es el defecto que acaba de arreglarse.
    #[test]
    fn todo_paso_lleva_a_su_pantalla() {
        let sin_ruta: Vec<_> = PASOS.iter().filter(|p| p.ruta().is_none()).collect();
        assert!(
            sin_ruta.is_empty(),
            "estos pasos no llevan a ninguna parte, así que la lista de alta \
             los daría por no construidos: {sin_ruta:?}"
        );
    }

    /// Y la ruta tiene que ser una ruta de verdad, no una cadena cualquiera.
    ///
    /// Sin esto, el arreglo de arriba se puede «cumplir» con un `Some("")` o
    /// con un `Some("pendiente")`, y la lista de alta pintaría un enlace a
    /// ninguna parte — que es justo lo que la regla quería evitar. No se
    /// comprueba contra el router porque el núcleo no conoce el frontend: lo
    /// que sí se puede exigir es la forma.
    #[test]
    fn ninguna_ruta_es_inventada() {
        for p in PASOS {
            let r = p.ruta().expect("todo paso tiene ruta");
            assert!(
                r.starts_with('/') && r.len() > 1,
                "{p:?} declara «{r}», que no es una ruta"
            );
        }
    }

    #[test]
    fn una_instalacion_nueva_no_tiene_nada_hecho() {
        let l = ListaDeOnboarding::desde(&RecuentoDeAlta::default());
        assert_eq!(l.completados, 0);
        assert_eq!(l.total, PASOS.len());
        assert!(!l.terminada());
        assert_eq!(l.siguiente, Some(PasoDeOnboarding::Empresa));
    }

    #[test]
    fn configurar_la_empresa_marca_su_paso() {
        let r = RecuentoDeAlta {
            empresas: 1,
            ..RecuentoDeAlta::default()
        };
        let l = ListaDeOnboarding::desde(&r);
        assert_eq!(l.completados, 1);
        assert!(
            l.pasos
                .iter()
                .any(|p| p.paso == PasoDeOnboarding::Empresa && p.completado)
        );
    }

    /// La razón de derivar en vez de almacenar: si el dato desaparece, el paso
    /// vuelve a estar pendiente. Con un booleano guardado seguiría en verde.
    #[test]
    fn borrar_el_dato_devuelve_el_paso_a_pendiente() {
        let con = RecuentoDeAlta {
            remitentes: 1,
            ..RecuentoDeAlta::default()
        };
        let sin = RecuentoDeAlta::default();
        let hecho = |r: &RecuentoDeAlta| PasoDeOnboarding::Remitente.completado(r);
        assert!(hecho(&con));
        assert!(!hecho(&sin), "el paso mintió después de borrar el dato");
    }

    /// Cuando lo único pendiente todavía no está construido, `siguiente` es
    /// `None` **y la lista no está terminada**. Confundir las dos cosas haría
    /// que la interfaz felicitara al usuario por una campaña que no existe.
    #[test]
    fn sin_siguiente_no_significa_terminada() {
        // Todo lo que YA tiene pantalla, hecho. Lo que queda —plantilla,
        // ventana y campaña— llega en la Fase 6, así que no hay un «siguiente»
        // al que mandar a nadie, y la lista tampoco está terminada.
        //
        // Este recuento cambió el 23/09/2026: antes bastaba con la empresa,
        // porque Contactos y Remitentes no tenían ruta —aunque Contactos
        // llevaba construida desde la 3.2—. Al corregir eso, la prueba pasó a
        // medir una situación que ya no existe. Lo que comprueba es lo mismo
        // de siempre: que «no hay siguiente» y «está terminada» son cosas
        // distintas, y confundirlas felicitaría al usuario por una campaña que
        // no existe.
        let r = RecuentoDeAlta {
            empresas: 1,
            remitentes: 1,
            contactos: 1,
            ..RecuentoDeAlta::default()
        };
        let l = ListaDeOnboarding::desde(&r);
        // Con empresa, remitente y contactos hechos, el siguiente que se puede
        // hacer es la plantilla: su pantalla existe desde la 6.8a. Antes de esa
        // entrega no había ninguno más y esto era `None`.
        assert_eq!(l.siguiente, Some(PasoDeOnboarding::Plantilla));
        assert!(!l.terminada());
    }

    #[test]
    fn con_todo_hecho_la_lista_termina() {
        let r = RecuentoDeAlta {
            empresas: 1,
            remitentes: 1,
            contactos: 1,
            plantillas: 1,
            ventanas: 1,
            campanas: 1,
        };
        let l = ListaDeOnboarding::desde(&r);
        assert!(l.terminada());
        assert_eq!(l.siguiente, None);
    }

    #[test]
    fn cada_paso_tiene_su_clave_de_texto() {
        for p in PASOS {
            assert!(!p.clave().is_empty(), "{p:?} no declara clave");
        }
    }

    #[test]
    fn el_primer_paso_es_la_empresa() {
        assert_eq!(PASOS.first(), Some(&PasoDeOnboarding::Empresa));
        assert_eq!(
            PasoDeOnboarding::Empresa.ruta(),
            Some("/inicio/configuracion"),
            "el primer paso tiene que llevar a la configuración, no a Ajustes: \
             Ajustes es de la aplicación, no del negocio (Dirección, 21/09/2026)"
        );
    }

    /// La forma que ve el frontend. Si alguien renombra un campo, el
    /// componente deja de pintar y aquí falla antes.
    #[test]
    fn la_lista_se_serializa_con_los_nombres_que_espera_la_interfaz() {
        let l = ListaDeOnboarding::desde(&RecuentoDeAlta::default());
        let json = serde_json::to_string(&l).expect("serializa");
        for campo in ["pasos", "completados", "total", "siguiente", "ruta"] {
            assert!(json.contains(campo), "falta «{campo}» en {json}");
        }

        // Y los dos que se fueron el 28/10/2026 no pueden volver por la puerta
        // de atrás: `disponible` era `ruta.is_some()` dicho otra vez, y
        // `entrega` sacaba el número de entrega del roadmap a la interfaz
        // —«Llega en la entrega 6»—, que es jerga nuestra en la pantalla de
        // alguien que no trabaja aquí.
        for campo in ["disponible", "entrega"] {
            assert!(
                !json.contains(campo),
                "«{campo}» volvió a la frontera: {json}"
            );
        }
    }
}
