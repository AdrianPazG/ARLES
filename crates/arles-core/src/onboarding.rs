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

    /// A dónde lleva el paso, si ya se puede hacer.
    ///
    /// `None` significa que la pantalla **todavía no existe**. Se dice, no se
    /// esconde: un paso que lleva a una pantalla vacía es peor que uno que
    /// avisa de que llega más adelante.
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
            // Las demás pantallas llegan en sus entregas. Ver `entrega`.
            _ => None,
        }
    }

    /// Entrega del roadmap que construye este paso.
    ///
    /// Se muestra en la interfaz para que la lista sea honesta sobre lo que
    /// todavía no está: «Contactos · llega en la entrega 3.2».
    #[must_use]
    pub fn entrega(self) -> &'static str {
        match self {
            Self::Empresa => "3.1",
            Self::Contactos => "3.2",
            Self::Remitente => "5",
            Self::Plantilla | Self::VentanaDeEjecucion | Self::PrimeraCampana => "6",
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
    /// La pantalla existe y se puede ir a ella.
    pub disponible: bool,
    pub ruta: Option<&'static str>,
    pub entrega: &'static str,
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
                disponible: paso.ruta().is_some(),
                ruta: paso.ruta(),
                entrega: paso.entrega(),
            })
            .collect();

        let completados = pasos.iter().filter(|p| p.completado).count();
        let siguiente = pasos
            .iter()
            .find(|p| !p.completado && p.disponible)
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

    /// **Un paso cuya pantalla ya existe tiene que tener a dónde llevar.**
    ///
    /// La lista de alta enseña «llega en la entrega X» y esconde el botón
    /// cuando un paso no tiene ruta. Sin esta prueba, entregar una pantalla y
    /// olvidarse de apuntarla aquí deja la lista diciendo que está por llegar
    /// — que es lo que pasó con Contactos entre la 3.2 y la 5.3: semanas
    /// anunciando como futura una pantalla construida, y sin forma de llegar a
    /// ella desde la lista.
    ///
    /// La lista de abajo se escribe a mano a propósito: es la afirmación «esto
    /// ya está construido», y tiene que costar un segundo de pensar al
    /// añadirle una línea.
    #[test]
    fn los_pasos_ya_construidos_llevan_a_su_pantalla() {
        let construidos = [
            (PasoDeOnboarding::Empresa, "/inicio/configuracion"),
            (PasoDeOnboarding::Contactos, "/contactos"),
            (PasoDeOnboarding::Remitente, "/remitentes"),
        ];
        for (paso, ruta) in construidos {
            assert_eq!(
                paso.ruta(),
                Some(ruta),
                "{paso:?} ya está construido: la lista de alta tiene que poder llevar ahí"
            );
        }

        // Y los que todavía no existen no inventan una ruta: un botón que
        // lleva a una pantalla vacía es peor que no tener botón.
        for paso in [
            PasoDeOnboarding::Plantilla,
            PasoDeOnboarding::VentanaDeEjecucion,
            PasoDeOnboarding::PrimeraCampana,
        ] {
            assert_eq!(paso.ruta(), None, "{paso:?} todavía no tiene pantalla");
        }
    }
    use super::*;

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
        assert_eq!(l.siguiente, None, "lo construido ya está hecho");
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
    fn cada_paso_dice_en_que_entrega_llega() {
        for p in PASOS {
            assert!(!p.entrega().is_empty(), "{p:?} no declara entrega");
            assert!(!p.clave().is_empty());
        }
    }

    /// Un paso disponible tiene ruta y uno sin ruta no está disponible: son la
    /// misma afirmación, y si divergen la interfaz pinta un enlace a ninguna
    /// parte.
    #[test]
    fn disponible_y_ruta_no_pueden_divergir() {
        let l = ListaDeOnboarding::desde(&RecuentoDeAlta::default());
        for p in &l.pasos {
            assert_eq!(p.disponible, p.ruta.is_some(), "{:?} incoherente", p.paso);
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
        for campo in [
            "pasos",
            "completados",
            "total",
            "siguiente",
            "disponible",
            "entrega",
        ] {
            assert!(json.contains(campo), "falta «{campo}» en {json}");
        }
    }
}
