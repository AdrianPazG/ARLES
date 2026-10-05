//! El manifiesto del respaldo: qué tabla entra, en qué alcance, y por qué.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTO ES UNA DECLARACIÓN Y NO UNA CONSULTA SUELTA EN EL EXPORTADOR
//!
//! Un respaldo se escribe una vez y se mantiene durante años. El día que
//! alguien añada una tabla en la V11 y no la meta en el exportador, **nada
//! falla**: se exporta, se restaura, y los datos de esa tabla no están. Ni
//! excepción, ni aviso, ni una prueba en rojo. Lo descubre el usuario meses
//! después, cuando ya no hay de dónde sacarlos.
//!
//! Es el patrón del §9 —una pieza correcta que nadie usa— en su forma más cara,
//! porque aquí lo que se pierde no es una función: son datos de personas.
//!
//! Por eso la lista vive aquí, declarada y con su motivo, y la comprobación
//! `ninguna_tabla_se_queda_fuera_del_respaldo` del validador la compara contra
//! **el esquema** —no contra otra lista a mano—. Una tabla nueva sin clasificar
//! rompe la validación antes de llegar a ninguna parte.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LAS DOS DECISIONES DE DIRECCIÓN QUE ESTE ARCHIVO HACE CUMPLIR
//!
//! **D-8B · Dos alcances** (05/10/2026). Un respaldo que se lo lleva todo puede
//! tardar veinte minutos en una instalación de dos años, y un respaldo que tarda
//! veinte minutos no se hace nunca. Así que hay dos:
//!
//! - **Completo** — todo. Para cambiar de equipo.
//! - **Esencial** — lo que no se puede volver a conseguir, con el historial de
//!   envíos recortado. Para el día a día. **Es el que viene elegido.**
//!
//! **Y la regla que no se negocia:** las bajas van **enteras en los dos**.
//! Perder una baja significa volver a escribirle a quien pidió que no, y eso es
//! lo peor que este producto puede hacer. Lo vigila su propia comprobación,
//! aparte, porque una tabla mal clasificada pasaría las otras sin protestar.
//!
//! **D-8C · Restaurar reemplaza, nunca mezcla** (05/10/2026). Mezclar puede
//! resucitar una baja: alguien que pidió no recibir vuelve a la lista porque el
//! respaldo es anterior a su petición. La excepción es `suppression_entry`, que
//! **se une**: la lista final es la unión de las dos, porque una baja no se
//! deshace sola. Eso lo construye la 8.3; aquí queda declarado con qué dato.

/// Cuánto de una tabla entra en un respaldo.
///
/// No hay variante «depende»: cada tabla cae en una de las cuatro, y el
/// compilador obliga a elegir. Una tabla nueva no compila sin decidirlo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alcance {
    /// Entera en los dos respaldos. **Nunca se recorta por fecha.**
    ///
    /// Es para lo que no se puede volver a conseguir ni reconstruir: los
    /// contactos, su permiso y —sobre todo— las bajas.
    SiempreEntera,

    /// Entera en el completo; en el esencial, sólo los últimos doce meses.
    ///
    /// Para lo que crece sin techo y pierde valor con el tiempo sin perderlo
    /// del todo. Hoy sólo el historial de envíos y las respuestas caen aquí.
    ///
    /// **La variante lleva dentro la columna por la que se recorta**, y no es
    /// adorno: recortar por fecha necesita saber qué fecha, y las dos tablas
    /// que caen aquí la llaman distinto —`created_at` en el historial,
    /// `first_reply_at` en las respuestas—. Puesta en un campo aparte se podría
    /// dejar vacía, o rellenar en una tabla que no se recorta y creer que sí;
    /// puesta dentro de la variante, el compilador la exige justo cuando hace
    /// falta y la prohíbe donde no.
    RecortadaEnElEsencial {
        /// La columna de fecha por la que se corta. Que exista en el esquema lo
        /// comprueba `toda_columna_de_corte_existe_de_verdad` en `arles-db`,
        /// contra una base migrada de verdad y no contra esta lista.
        fecha: &'static str,
    },

    /// Sólo en el respaldo completo.
    ///
    /// Para lo que se puede reconstruir o lo que sólo importa al migrar de
    /// equipo. Que falte en el esencial no deja a nadie sin nada.
    SoloCompleto,

    /// No sale en ningún respaldo.
    ///
    /// Reservado para lo que describe **este equipo** y no **estos datos**, y
    /// para lo que restaurar en otra máquina sería incorrecto. El motivo va
    /// escrito al lado y el validador exige que sea una frase.
    Nunca,
}

impl Alcance {
    /// ¿Esta tabla entra en el respaldo esencial?
    #[must_use]
    pub const fn entra_en_el_esencial(self) -> bool {
        matches!(
            self,
            Self::SiempreEntera | Self::RecortadaEnElEsencial { .. }
        )
    }

    /// ¿Entra en el completo?
    #[must_use]
    pub const fn entra_en_el_completo(self) -> bool {
        !matches!(self, Self::Nunca)
    }

    /// Por qué columna se recorta en el esencial, si se recorta.
    #[must_use]
    pub const fn columna_de_corte(self) -> Option<&'static str> {
        match self {
            Self::RecortadaEnElEsencial { fecha } => Some(fecha),
            _ => None,
        }
    }
}

/// Una tabla del esquema, con su sitio en el respaldo y el motivo.
///
/// `porque` no es documentación decorativa: el validador exige que tenga
/// contenido. Una exclusión sin motivo es un olvido con mejor presentación.
#[derive(Debug, Clone, Copy)]
pub struct EnElRespaldo {
    pub tabla: &'static str,
    pub alcance: Alcance,
    pub porque: &'static str,
}

/// Cuántos meses de historial se guarda el respaldo esencial.
///
/// Doce y no seis porque un año cubre el ciclo completo de un cliente —las
/// campañas de temporada se comparan contra las del año anterior— y porque es
/// el plazo que la gente entiende sin que se lo expliquen.
pub const MESES_EN_EL_ESENCIAL: u32 = 12;

/// **Las 29 tablas del esquema, clasificadas.**
///
/// El orden es el del esquema, no alfabético: agrupa lo que se lee junto.
pub const MANIFIESTO: &[EnElRespaldo] = &[
    // ── La empresa y lo suyo ───────────────────────────────────────────────
    EnElRespaldo {
        tabla: "company",
        alcance: Alcance::SiempreEntera,
        porque: "Los datos de la empresa y su zona horaria. Sin esto, un respaldo restaurado no sabe de quién es.",
    },
    EnElRespaldo {
        tabla: "ui_preference",
        alcance: Alcance::Nunca,
        porque: "Describe ESTE equipo, no estos datos: el tema y si la barra está plegada. Restaurarlo en otra máquina impondría la pantalla de otra persona.",
    },
    EnElRespaldo {
        tabla: "audit_log",
        alcance: Alcance::SoloCompleto,
        porque: "La bitácora de decisiones críticas. Importa al migrar de equipo y para auditar; en el día a día no es lo que se pierde si falla un disco.",
    },
    // ── Los contactos, que son lo irreemplazable ───────────────────────────
    EnElRespaldo {
        tabla: "contact",
        alcance: Alcance::SiempreEntera,
        porque: "Es lo que el cliente importó y lo que más caro le sale volver a conseguir. Si algo justifica que exista un respaldo, es esto.",
    },
    EnElRespaldo {
        tabla: "contact_channel",
        alcance: Alcance::SiempreEntera,
        porque: "Las direcciones y teléfonos de cada contacto. Un contacto sin sus canales no sirve para nada.",
    },
    EnElRespaldo {
        tabla: "consent_entry",
        alcance: Alcance::SiempreEntera,
        porque: "El permiso de cada persona, y la prueba de que se tuvo. Perderlo deja al cliente sin con qué responder a una autoridad.",
    },
    EnElRespaldo {
        tabla: "suppression_entry",
        alcance: Alcance::SiempreEntera,
        porque: "Quién pidió no recibir. ENTERA y sin recorte en los dos alcances: perder una baja significa volver a escribirle, que es lo peor que ARLES puede hacer.",
    },
    EnElRespaldo {
        tabla: "import_batch",
        // ── RECLASIFICADA EN LA 8.2b, Y NO POR GUSTO ──
        //
        // Estaba como `SoloCompleto` con el motivo «es trazabilidad, no dato
        // operativo: sin ella los contactos siguen sirviendo». Suena razonable
        // y es falso: **`contact` tiene una clave ajena a esta tabla**. Un
        // respaldo esencial con los contactos y sin sus lotes no se puede
        // restaurar — cada fila de `contact` apunta a un padre que no está, y
        // con `PRAGMA foreign_keys=ON` la inserción se rechaza.
        //
        // No lo habría encontrado una prueba del manifiesto: lo encontró medir
        // el esquema. Ahora lo vigila `ningun_hijo_viaja_sin_su_padre`.
        alcance: Alcance::SiempreEntera,
        porque: "De dónde salió cada contacto. Entera en los dos porque `contact` tiene una clave ajena a esta tabla: sin ella los contactos no se pueden ni insertar. Y es una fila por archivo importado, así que no pesa.",
    },
    // ── Lo que la Fase 6 declaró fuera de alcance, y existe en el esquema ──
    EnElRespaldo {
        tabla: "tag",
        alcance: Alcance::SiempreEntera,
        porque: "Etiquetas del cliente. Hoy ninguna pantalla las escribe (§40), pero el esquema las guarda y lo que se guarde tiene que volver.",
    },
    EnElRespaldo {
        tabla: "contact_tag",
        alcance: Alcance::SiempreEntera,
        porque: "Qué etiqueta lleva cada contacto. Va con `tag` por la misma razón: si algún día se rellena, el respaldo ya lo contempla.",
    },
    EnElRespaldo {
        tabla: "contact_list",
        alcance: Alcance::SiempreEntera,
        porque: "Listas del cliente. Mismo caso que las etiquetas: el esquema las tiene y un respaldo no puede decidir ignorarlas.",
    },
    EnElRespaldo {
        tabla: "contact_list_member",
        alcance: Alcance::SiempreEntera,
        porque: "Quién está en cada lista. Sin esta tabla, las listas restauradas saldrían vacías sin que nada avisara.",
    },
    EnElRespaldo {
        tabla: "contact_field",
        alcance: Alcance::SiempreEntera,
        porque: "Campos propios que el cliente definió. Son suyos y nadie más los tiene: no hay de dónde reconstruirlos.",
    },
    // ── Las campañas y lo que se manda ─────────────────────────────────────
    EnElRespaldo {
        tabla: "campaign",
        alcance: Alcance::SiempreEntera,
        porque: "Las campañas creadas, con su estado. Rehacer una campaña entera a mano es trabajo de una tarde.",
    },
    EnElRespaldo {
        tabla: "campaign_stage",
        alcance: Alcance::SiempreEntera,
        porque: "Las etapas de cada campaña, con su ventana, sus límites y su remitente. Una campaña sin etapas no se puede activar.",
    },
    EnElRespaldo {
        tabla: "campaign_audience",
        alcance: Alcance::SiempreEntera,
        porque: "La audiencia congelada al activar. Se congela justo para que no cambie, así que recalcularla al restaurar daría otra lista.",
    },
    EnElRespaldo {
        tabla: "template",
        alcance: Alcance::SiempreEntera,
        porque: "Las plantillas que el cliente escribió. Es texto suyo y no está en ningún otro sitio.",
    },
    EnElRespaldo {
        tabla: "template_version",
        alcance: Alcance::SiempreEntera,
        porque: "Las versiones, que no se editan a propósito: son la prueba de qué texto se mandó en cada campaña.",
    },
    EnElRespaldo {
        tabla: "signature",
        alcance: Alcance::SiempreEntera,
        porque: "Las firmas de correo. Mismo caso que las plantillas: lo escribió el cliente.",
    },
    EnElRespaldo {
        tabla: "execution_window",
        alcance: Alcance::SiempreEntera,
        porque: "Las ventanas horarias con su zona IANA. Son una decisión del cliente sobre cuándo se le puede escribir a su gente.",
    },
    // ── El historial de envíos: lo que crece sin techo ─────────────────────
    EnElRespaldo {
        tabla: "message_attempt",
        alcance: Alcance::RecortadaEnElEsencial {
            fecha: "created_at",
        },
        porque: "Un renglón por destinatario y envío, y no se borra jamás (D-7C). Entero en el completo; en el esencial, los últimos doce meses, porque es lo único que crece sin techo.",
    },
    EnElRespaldo {
        tabla: "activity_event",
        alcance: Alcance::SoloCompleto,
        porque: "El registro de actividad, que YA se recorta cada dos meses en la propia base (7.4). Llevarlo al esencial sería guardar dos veces lo que el producto decidió no acumular.",
    },
    EnElRespaldo {
        tabla: "rate_budget",
        alcance: Alcance::Nunca,
        porque: "Cuánto cupo queda esta hora y este día. Es estado de ESTE momento en ESTE equipo: restaurarlo daría permiso para enviar un cupo que ya se gastó, o lo contrario.",
    },
    EnElRespaldo {
        tabla: "engine_control",
        alcance: Alcance::Nunca,
        porque: "La parada de emergencia y el estado del motor, de este equipo. Restaurar un «en marcha» de otra máquina arrancaría envíos que nadie pidió aquí.",
    },
    // ── Las cuentas: la referencia viaja, el secreto no ────────────────────
    EnElRespaldo {
        tabla: "email_account",
        alcance: Alcance::SiempreEntera,
        porque: "El servidor, el puerto y el remitente de cada cuenta — NUNCA la contraseña, que vive en el llavero (§75). Al restaurar, la cuenta aparece pidiendo reconectarse.",
    },
    EnElRespaldo {
        tabla: "whatsapp_account",
        alcance: Alcance::SiempreEntera,
        porque: "El número, su modo y las dos confirmaciones — NUNCA el token. La constancia de que a alguien se le avisó del riesgo tiene que sobrevivir a un cambio de equipo.",
    },
    EnElRespaldo {
        tabla: "whatsapp_template",
        alcance: Alcance::SiempreEntera,
        porque: "Las plantillas que Meta aprobó. Volver a registrarlas a mano es trabajo, y sin al menos una el canal no abre.",
    },
    EnElRespaldo {
        tabla: "whatsapp_reply",
        // Y no `created_at`, que esta tabla **no tiene**: la fecha que guarda es
        // cuándo llegó la primera respuesta de esa persona. Es el motivo de que
        // la columna viaje dentro de la variante y no se dé por supuesta.
        alcance: Alcance::RecortadaEnElEsencial {
            fecha: "first_reply_at",
        },
        porque: "Cuántas personas contestaron a cada campaña (D-7A, nunca qué dijeron). Crece con los envíos, así que sigue la misma regla que el historial.",
    },
    EnElRespaldo {
        tabla: "whatsapp_quality_event",
        alcance: Alcance::SoloCompleto,
        porque: "El histórico de calidad del número según Meta. Lo vuelve a decir Meta en cuanto se reconecte: se puede reconstruir, así que no ocupa sitio en el esencial.",
    },
];

/// El sitio de una tabla en el respaldo, si está declarada.
#[must_use]
pub fn alcance_de(tabla: &str) -> Option<Alcance> {
    MANIFIESTO
        .iter()
        .find(|e| e.tabla == tabla)
        .map(|e| e.alcance)
}

/// Las tablas que entran en un respaldo esencial.
#[must_use]
pub fn tablas_del_esencial() -> Vec<&'static str> {
    MANIFIESTO
        .iter()
        .filter(|e| e.alcance.entra_en_el_esencial())
        .map(|e| e.tabla)
        .collect()
}

/// Las tablas que entran en un respaldo completo.
#[must_use]
pub fn tablas_del_completo() -> Vec<&'static str> {
    MANIFIESTO
        .iter()
        .filter(|e| e.alcance.entra_en_el_completo())
        .map(|e| e.tabla)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ninguna tabla se declara dos veces.
    ///
    /// Dos entradas para la misma tabla con alcances distintos harían que
    /// `alcance_de` devolviera la primera y el exportador la obedeciera, sin
    /// que nada dijera que había otra opinión escrita diez líneas más abajo.
    #[test]
    fn ninguna_tabla_esta_declarada_dos_veces() {
        let mut vistas = std::collections::HashSet::new();
        for e in MANIFIESTO {
            assert!(
                vistas.insert(e.tabla),
                "la tabla {} está declarada dos veces en el manifiesto",
                e.tabla
            );
        }
    }

    /// **Las bajas enteras en los dos respaldos. D-8C hecha prueba.**
    ///
    /// Es la regla que no se negocia de toda la fase. Está comprobada aquí y
    /// también en el validador, a propósito: si alguien toca el manifiesto sin
    /// correr el validador, esta prueba lo caza igual.
    ///
    /// Verificada el 05/10/2026 bajándola a `SoloCompleto`: falla con el
    /// mensaje correcto.
    #[test]
    fn las_bajas_entran_enteras_en_los_dos() {
        assert_eq!(
            alcance_de("suppression_entry"),
            Some(Alcance::SiempreEntera),
            "las bajas no pueden recortarse ni quedarse fuera del esencial"
        );
        assert!(tablas_del_esencial().contains(&"suppression_entry"));
        assert!(tablas_del_completo().contains(&"suppression_entry"));
    }

    /// Lo que describe el equipo no viaja.
    ///
    /// Las tres `Nunca` lo son por el mismo motivo —son estado de ESTA máquina—
    /// y se congelan aquí porque meterlas por descuido es peor que un olvido:
    /// restaurar `engine_control` de otra máquina arrancaría envíos que nadie
    /// pidió en ésta.
    #[test]
    fn el_estado_de_este_equipo_no_viaja() {
        for tabla in ["ui_preference", "rate_budget", "engine_control"] {
            assert_eq!(
                alcance_de(tabla),
                Some(Alcance::Nunca),
                "{tabla} describe este equipo y no puede entrar en un respaldo"
            );
        }
    }

    /// El esencial es **más pequeño** que el completo, y no por poco.
    ///
    /// Si los dos alcances acabaran llevando lo mismo, la elección sería
    /// decorativa y el respaldo del día a día volvería a tardar lo que D-8B
    /// quería evitar. La prueba fija que la diferencia existe.
    #[test]
    fn el_esencial_lleva_menos_que_el_completo() {
        let esencial = tablas_del_esencial();
        let completo = tablas_del_completo();
        assert!(
            esencial.len() < completo.len(),
            "el esencial lleva {} tablas y el completo {}",
            esencial.len(),
            completo.len()
        );
        // Y todo lo del esencial está en el completo: un respaldo completo que
        // perdiera algo que el esencial sí lleva no tendría ningún sentido.
        for t in &esencial {
            assert!(
                completo.contains(t),
                "{t} está en el esencial y no en el completo"
            );
        }
    }

    /// Todo motivo es una frase, no un hueco.
    ///
    /// El compilador obliga a rellenar el campo, pero no a que diga algo. Esta
    /// prueba y la del validador miran lo mismo desde los dos lados.
    #[test]
    fn todo_motivo_esta_escrito() {
        for e in MANIFIESTO {
            assert!(
                e.porque.len() >= 25,
                "el motivo de {} no explica nada: {:?}",
                e.tabla,
                e.porque
            );
        }
    }

    /// **Lo que se recorta dice por qué columna, y lo que no, no la tiene.**
    ///
    /// El compilador ya obliga a poner la columna al elegir la variante. Lo que
    /// esta prueba fija es lo otro: que la columna **diga algo** —una cadena
    /// vacía compila— y que las dos tablas que se recortan no la llamen igual,
    /// porque la tentación de dar por supuesto `created_at` es exactamente el
    /// error que la variante con dato dentro impide.
    ///
    /// Verificada el 05/10/2026 poniendo `fecha: "created_at"` en
    /// `whatsapp_reply` —que no tiene esa columna—: la prueba de `arles-db`
    /// que compara contra el esquema falla nombrándola. Aquí falla la segunda
    /// aserción.
    #[test]
    fn lo_que_se_recorta_dice_por_que_columna() {
        let cortes: Vec<(&str, &str)> = MANIFIESTO
            .iter()
            .filter_map(|e| e.alcance.columna_de_corte().map(|c| (e.tabla, c)))
            .collect();

        assert_eq!(
            cortes.len(),
            2,
            "cambió qué tablas se recortan: {cortes:?} — revisa D-8B antes de tocar esto"
        );
        for (tabla, columna) in &cortes {
            assert!(
                !columna.is_empty(),
                "{tabla} se recorta y no dice por qué columna"
            );
        }
        assert_ne!(
            cortes.first().map(|c| c.1),
            cortes.get(1).map(|c| c.1),
            "si las dos se recortaran por la misma columna, la variante con dato \
             dentro no haría falta — y el día que una la llame distinto, nadie lo vería"
        );

        // Y nada más la tiene: lo que entra entero no se recorta por nada.
        for e in MANIFIESTO {
            if !matches!(e.alcance, Alcance::RecortadaEnElEsencial { .. }) {
                assert!(e.alcance.columna_de_corte().is_none(), "{}", e.tabla);
            }
        }
    }

    /// Las dos listas no se solapan con `Nunca`.
    #[test]
    fn lo_que_nunca_entra_no_sale_en_ninguna_lista() {
        let nunca: Vec<_> = MANIFIESTO
            .iter()
            .filter(|e| e.alcance == Alcance::Nunca)
            .map(|e| e.tabla)
            .collect();
        assert!(
            !nunca.is_empty(),
            "ninguna tabla marcada Nunca: revisa el manifiesto"
        );
        for t in nunca {
            assert!(!tablas_del_esencial().contains(&t));
            assert!(!tablas_del_completo().contains(&t));
        }
    }
}
