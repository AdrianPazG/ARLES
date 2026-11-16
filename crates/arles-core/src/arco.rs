//! El derecho de **Acceso**: todo lo que ARLES tiene sobre una persona.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ ES ESTO, EN UNA FRASE
//!
//! Alguien escribe y dice «enséñame qué tienes sobre mí». Esto arma la
//! respuesta.
//!
//! Es la **A** de ARCO —Acceso, Rectificación, Cancelación, Oposición—, los
//! cuatro derechos que cualquier persona puede ejercer sobre sus datos. La ley
//! no se los exige a ARLES: se los exige a **quien usa ARLES**, que es quien
//! decide a quién escribirle. Si la herramienta no permitiera atender una
//! solicitud sin llamar a un programador, el que incumpliría sería el cliente.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ EL INFORME ES TEXTO Y NO UN CSV NI UN JSON
//!
//! Esto lo lee **la persona que preguntó**, no un sistema. Un JSON con llaves y
//! corchetes no responde a «qué tienes sobre mí»: responde a «cómo lo guardas».
//! Y un CSV de un solo registro es una fila de celdas que hay que abrir en
//! Excel para entender.
//!
//! Sale texto corrido, con sus apartados y sus fechas. Se puede leer en
//! pantalla, imprimir, o pegar en un correo de respuesta.
//!
//! Hay una consecuencia de seguridad **que es la razón de fondo**: un CSV
//! exportado es un vector de inyección de fórmulas —una celda que empieza por
//! `=` se ejecuta al abrirla en Excel—. En texto corrido eso no existe. Si
//! algún día hace falta exportar en CSV, la neutralización ya está escrita en
//! `arles_import::neutralizar_formula` y **hay que usarla**.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! ESTE MÓDULO NO LEE NADA
//!
//! Recibe el expediente ya reunido y lo redacta. Es lo que permite probar el
//! informe entero —incluido el caso de «no tenemos nada»— sin montar una base
//! de datos, y lo que mantiene este crate sin I/O.
//! ─────────────────────────────────────────────────────────────────────────

use crate::canal::Canal;

/// Una dirección de esa persona, con lo que sepamos de ella.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DireccionDelExpediente {
    pub canal: Canal,
    pub valor: String,
    /// `true` si es la principal de su canal.
    pub principal: bool,
    pub dada_de_alta: String,
}

/// Un dato propio que la empresa guardó sobre esa persona.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampoDelExpediente {
    pub clave: String,
    pub valor: String,
}

/// De dónde salió esa persona, si entró por una importación.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrigenDelExpediente {
    pub nombre_del_archivo: String,
    /// Cómo se declaró el origen de la lista, ya en texto legible.
    pub origen_declarado: String,
    pub cuando: String,
    /// El texto que quien importó aceptó en ese momento.
    ///
    /// Se guarda íntegro y se enseña íntegro: es la prueba de qué se afirmó,
    /// y resumirlo la destruiría.
    pub texto_aceptado: String,
}

/// Una negativa registrada: esa dirección pidió no recibir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegativaDelExpediente {
    pub canal: Canal,
    pub direccion: String,
    /// Ya en texto legible, no la clave de i18n.
    pub motivo: String,
    pub desde: String,
}

/// Un permiso registrado sobre una dirección.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermisoDelExpediente {
    pub canal: Canal,
    pub direccion: String,
    /// «concedido» o «retirado», ya legible.
    pub tipo: String,
    /// En qué se apoya, ya legible.
    pub base: String,
    pub cuando: String,
}

/// Un envío que se le hizo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvioDelExpediente {
    pub canal: Canal,
    pub direccion: String,
    pub campana: String,
    /// En qué acabó. **Nunca dice «entregado»** (§65): ARLES no puede saberlo.
    pub estado: String,
    pub cuando: String,
}

/// Todo lo que ARLES tiene sobre una persona.
///
/// **Vacío es una respuesta válida**, no un error. «No tenemos nada sobre
/// usted» es exactamente lo que alguien puede estar preguntando, y un informe
/// que lo diga vale tanto como uno lleno.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Expediente {
    /// La dirección por la que se preguntó, ya normalizada.
    pub se_pregunto_por: String,
    /// Cómo se llama la empresa que tiene los datos. Quien lea esto tiene
    /// derecho a saber **quién** los tiene.
    pub empresa: String,
    /// Cuándo se sacó el informe.
    pub fecha: String,
    pub nombre: String,
    pub apellido: String,
    pub organizacion: String,
    pub dado_de_alta: String,
    pub ultima_modificacion: String,
    pub direcciones: Vec<DireccionDelExpediente>,
    pub campos: Vec<CampoDelExpediente>,
    pub listas: Vec<String>,
    pub etiquetas: Vec<String>,
    pub origen: Option<OrigenDelExpediente>,
    pub negativas: Vec<NegativaDelExpediente>,
    pub permisos: Vec<PermisoDelExpediente>,
    pub envios: Vec<EnvioDelExpediente>,
}

impl Expediente {
    /// ¿Hay algo de esta persona en ARLES?
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// NO BASTA CON MIRAR SI HAY CONTACTO
    ///
    /// Una persona puede **no tener contacto y aun así estar en ARLES**: si
    /// pidió la baja y después alguien borró su ficha, lo que queda es su
    /// entrada en la lista de supresión. Y eso es un dato sobre ella, así que
    /// tiene derecho a saber que existe.
    ///
    /// De hecho es el caso más delicado de todos: alguien que pidió que lo
    /// borraran y quiere comprobar que se hizo. Contestarle «no tenemos nada»
    /// mientras su dirección sigue en una tabla sería mentirle.
    /// ─────────────────────────────────────────────────────────────────────
    #[must_use]
    pub fn hay_algo(&self) -> bool {
        !self.nombre.is_empty()
            || !self.apellido.is_empty()
            || !self.organizacion.is_empty()
            || !self.direcciones.is_empty()
            || !self.campos.is_empty()
            || !self.listas.is_empty()
            || !self.etiquetas.is_empty()
            || self.origen.is_some()
            || !self.negativas.is_empty()
            || !self.permisos.is_empty()
            || !self.envios.is_empty()
    }

    /// ¿Existe todavía como contacto, o sólo queda su rastro?
    #[must_use]
    pub fn sigue_siendo_contacto(&self) -> bool {
        !self.direcciones.is_empty()
    }
}

/// Cómo se llama un canal en el informe. En palabras, no en jerga.
fn canal_legible(canal: Canal) -> &'static str {
    match canal {
        Canal::Correo => "correo electrónico",
        Canal::WhatsApp => "WhatsApp",
    }
}

/// Redacta el informe que se le entrega a la persona.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LO QUE ESTE TEXTO TIENE QUE CONSEGUIR
///
/// Que alguien que no sabe nada de ARLES —ni de bases de datos— lea el archivo
/// y entienda tres cosas: **qué se tiene sobre él, de dónde salió, y qué se ha
/// hecho con ello**.
///
/// Por eso no hay identificadores internos, ni nombres de tabla, ni claves de
/// traducción. Un `a1b2c3-d4e5` en un informe de acceso no informa de nada: es
/// ruido que además sugiere que hay más de lo que se está enseñando.
/// ─────────────────────────────────────────────────────────────────────────
#[must_use]
pub fn redactar(e: &Expediente) -> String {
    let mut t = String::with_capacity(2048);

    t.push_str("INFORME DE DATOS PERSONALES\n");
    t.push_str("===========================\n\n");
    t.push_str(&format!("Se preguntó por: {}\n", e.se_pregunto_por));
    if !e.empresa.is_empty() {
        t.push_str(&format!("Datos en poder de: {}\n", e.empresa));
    }
    t.push_str(&format!("Fecha del informe: {}\n\n", e.fecha));

    if !e.hay_algo() {
        // El caso que parece que no hace falta y es el que más se va a usar.
        t.push_str(
            "NO HAY NINGÚN DATO SOBRE ESTA DIRECCIÓN\n\
             ---------------------------------------\n\n\
             No consta ningún contacto, ninguna lista, ningún permiso ni ninguna\n\
             solicitud de baja asociada a esta dirección.\n\n\
             Este informe se entrega igualmente: que no haya datos también es una\n\
             respuesta, y queda constancia de que se consultó.\n",
        );
        return t;
    }

    if !e.sigue_siendo_contacto() {
        // Justo el caso de quien pidió que lo borraran y quiere comprobarlo.
        t.push_str(
            "ESTA PERSONA YA NO ESTÁ DADA DE ALTA COMO CONTACTO\n\
             -------------------------------------------------\n\n\
             Su ficha fue eliminada. Lo que sigue es el rastro que queda de ella,\n\
             y se enseña entero porque también son datos suyos.\n\n",
        );
    }

    // ── Quién es ───────────────────────────────────────────────────────
    let nombre = format!("{} {}", e.nombre, e.apellido);
    let nombre = nombre.trim();
    if !nombre.is_empty() || !e.organizacion.is_empty() {
        t.push_str("SUS DATOS\n---------\n\n");
        if !nombre.is_empty() {
            t.push_str(&format!("Nombre: {nombre}\n"));
        }
        if !e.organizacion.is_empty() {
            t.push_str(&format!("Empresa u organización: {}\n", e.organizacion));
        }
        if !e.dado_de_alta.is_empty() {
            t.push_str(&format!("Dado de alta el: {}\n", e.dado_de_alta));
        }
        if !e.ultima_modificacion.is_empty() {
            t.push_str(&format!("Última modificación: {}\n", e.ultima_modificacion));
        }
        t.push('\n');
    }

    // ── Por dónde se le puede escribir ─────────────────────────────────
    if !e.direcciones.is_empty() {
        t.push_str("DIRECCIONES REGISTRADAS\n-----------------------\n\n");
        for d in &e.direcciones {
            t.push_str(&format!("- {} ({}", d.valor, canal_legible(d.canal)));
            if d.principal {
                t.push_str(", principal");
            }
            t.push(')');
            if !d.dada_de_alta.is_empty() {
                t.push_str(&format!(" · desde el {}", d.dada_de_alta));
            }
            t.push('\n');
        }
        t.push('\n');
    }

    // ── Datos propios de la empresa ────────────────────────────────────
    if !e.campos.is_empty() {
        t.push_str("OTROS DATOS GUARDADOS\n---------------------\n\n");
        for c in &e.campos {
            t.push_str(&format!("- {}: {}\n", c.clave, c.valor));
        }
        t.push('\n');
    }

    if !e.listas.is_empty() || !e.etiquetas.is_empty() {
        t.push_str("LISTAS Y ETIQUETAS\n------------------\n\n");
        for l in &e.listas {
            t.push_str(&format!("- Lista: {l}\n"));
        }
        for et in &e.etiquetas {
            t.push_str(&format!("- Etiqueta: {et}\n"));
        }
        t.push('\n');
    }

    // ── De dónde salieron sus datos ────────────────────────────────────
    //
    // Es lo que más suele preguntar quien ejerce este derecho: no «qué
    // tienes», sino «de dónde lo sacaste».
    t.push_str("DE DÓNDE SALIERON ESTOS DATOS\n-----------------------------\n\n");
    match &e.origen {
        Some(o) => {
            t.push_str(&format!(
                "Se importaron desde un archivo llamado «{}» el {}.\n",
                o.nombre_del_archivo, o.cuando
            ));
            t.push_str(&format!("Origen declarado: {}\n\n", o.origen_declarado));
            if !o.texto_aceptado.is_empty() {
                t.push_str("Quien hizo esa importación aceptó lo siguiente:\n\n");
                t.push_str(&format!("    {}\n\n", o.texto_aceptado));
            }
        }
        None => {
            t.push_str(
                "No proceden de ninguna importación: se escribieron a mano en la\n\
                 aplicación.\n\n",
            );
        }
    }

    // ── Permisos ───────────────────────────────────────────────────────
    if !e.permisos.is_empty() {
        t.push_str("PERMISOS REGISTRADOS\n--------------------\n\n");
        for p in &e.permisos {
            t.push_str(&format!(
                "- {} para {} ({}) · {} · {}\n",
                p.tipo,
                p.direccion,
                canal_legible(p.canal),
                p.base,
                p.cuando
            ));
        }
        t.push('\n');
    }

    // ── Negativas ──────────────────────────────────────────────────────
    t.push_str("SOLICITUDES DE NO RECIBIR\n-------------------------\n\n");
    if e.negativas.is_empty() {
        t.push_str("No consta ninguna.\n\n");
    } else {
        for n in &e.negativas {
            t.push_str(&format!(
                "- {} ({}) · {} · desde el {}\n",
                n.direccion,
                canal_legible(n.canal),
                n.motivo,
                n.desde
            ));
        }
        t.push_str(
            "\nMientras estas solicitudes existan, ARLES no envía nada a esas\n\
             direcciones. Se conservan precisamente para que eso siga siendo cierto:\n\
             si se borraran, la dirección podría volver a entrar en una lista futura\n\
             y recibir mensajes otra vez.\n\n",
        );
    }

    // ── Qué se le ha enviado ───────────────────────────────────────────
    t.push_str("MENSAJES ENVIADOS\n-----------------\n\n");
    if e.envios.is_empty() {
        t.push_str("No consta ningún envío a estas direcciones.\n\n");
    } else {
        for env in &e.envios {
            t.push_str(&format!(
                "- {} · {} ({}) · {} · {}\n",
                env.cuando,
                env.direccion,
                canal_legible(env.canal),
                env.campana,
                env.estado
            ));
        }
        // §65: aceptado no es entregado, y decirlo aquí importa más que en
        // ninguna otra pantalla — quien lee esto puede estar comprobando si
        // recibió algo que no recuerda.
        t.push_str(
            "\n«Aceptado por el proveedor» significa que el servidor de correo lo\n\
             admitió para su entrega. No significa que llegara a su bandeja: eso\n\
             ARLES no puede saberlo.\n\n",
        );
    }

    t.push_str(
        "─────────────────────────────────────────────────────────────────\n\
         Este informe recoge todos los datos que la aplicación ARLES tiene sobre\n\
         esta dirección en la fecha indicada arriba.\n",
    );

    t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Expediente {
        Expediente {
            se_pregunto_por: "ana@empresa.mx".into(),
            empresa: "TELEMETRY INSIGHT".into(),
            fecha: "21 de septiembre de 2026".into(),
            ..Default::default()
        }
    }

    /// **Vacío es una respuesta válida.** Y el informe se entrega igual.
    #[test]
    fn sin_datos_el_informe_lo_dice_y_no_queda_vacio() {
        let texto = redactar(&base());

        assert!(texto.contains("NO HAY NINGÚN DATO"), "{texto}");
        assert!(texto.contains("ana@empresa.mx"));
        assert!(
            texto.contains("TELEMETRY INSIGHT"),
            "falta quién tiene los datos"
        );
        assert!(
            texto.len() > 200,
            "el informe de «no hay nada» salió demasiado corto para entregarse"
        );
    }

    /// **El caso delicado:** alguien pidió la baja, después le borraron la
    /// ficha, y ahora quiere comprobar qué queda. No se le puede decir que no
    /// hay nada mientras su dirección siga en una tabla.
    #[test]
    fn una_persona_sin_contacto_pero_con_baja_no_sale_como_vacia() {
        let mut e = base();
        e.negativas.push(NegativaDelExpediente {
            canal: Canal::Correo,
            direccion: "ana@empresa.mx".into(),
            motivo: "pidió la baja".into(),
            desde: "14 de agosto de 2026".into(),
        });

        assert!(e.hay_algo(), "una baja registrada es un dato sobre ella");
        assert!(!e.sigue_siendo_contacto());

        let texto = redactar(&e);
        assert!(
            !texto.contains("NO HAY NINGÚN DATO"),
            "se le dijo que no había nada teniendo su baja guardada:\n{texto}"
        );
        assert!(texto.contains("YA NO ESTÁ DADA DE ALTA"), "{texto}");
        assert!(texto.contains("pidió la baja"));
        // Y se le explica por qué se conserva, que es lo que va a preguntar.
        assert!(texto.contains("si se borraran"), "{texto}");
    }

    /// El informe no enseña identificadores internos ni jerga de la base.
    ///
    /// Un `a1b2c3-d4e5` no informa de nada y sugiere que hay más de lo que se
    /// está enseñando.
    #[test]
    fn el_informe_no_lleva_jerga_ni_identificadores() {
        let mut e = base();
        e.nombre = "Ana".into();
        e.apellido = "Ruiz".into();
        e.direcciones.push(DireccionDelExpediente {
            canal: Canal::Correo,
            valor: "ana@empresa.mx".into(),
            principal: true,
            dada_de_alta: "3 de marzo de 2026".into(),
        });

        let texto = redactar(&e);
        for jerga in [
            "contact_id",
            "company_id",
            "suppression_entry",
            "email_normalized",
            "supresion.motivo",
            "error.",
            "null",
            "None",
        ] {
            assert!(
                !texto.contains(jerga),
                "el informe enseña «{jerga}», que no significa nada para quien lo lee:\n{texto}"
            );
        }
        // Y los canales se nombran en palabras.
        assert!(texto.contains("correo electrónico"), "{texto}");
        assert!(!texto.contains("email"), "{texto}");
    }

    /// El texto del consentimiento se enseña **íntegro**: es la prueba de qué
    /// se afirmó al importar, y resumirlo la destruiría.
    #[test]
    fn el_texto_que_se_acepto_al_importar_sale_entero() {
        let largo = "Declaro que obtuve estos contactos de forma lícita y que puedo \
                     escribirles con fines comerciales. [TEXTO PENDIENTE DE REVISIÓN \
                     JURÍDICA · P-09]";
        let mut e = base();
        e.origen = Some(OrigenDelExpediente {
            nombre_del_archivo: "feria-septiembre.csv".into(),
            origen_declarado: "evento o feria".into(),
            cuando: "18 de septiembre de 2026".into(),
            texto_aceptado: largo.into(),
        });

        let texto = redactar(&e);
        assert!(
            texto.contains(largo),
            "el texto aceptado se recortó:\n{texto}"
        );
        assert!(texto.contains("feria-septiembre.csv"));
    }

    /// Sin importación, se dice que se escribieron a mano. El apartado no
    /// desaparece: «de dónde salieron» es lo que más pregunta quien ejerce
    /// este derecho, y un hueco no responde.
    #[test]
    fn sin_importacion_se_dice_que_se_escribieron_a_mano() {
        let mut e = base();
        e.nombre = "Ana".into();

        let texto = redactar(&e);
        assert!(texto.contains("DE DÓNDE SALIERON"), "{texto}");
        assert!(texto.contains("a mano"), "{texto}");
    }

    /// §65 · aceptado no es entregado, y aquí importa más que en ninguna otra
    /// parte: quien lee esto puede estar comprobando si recibió algo.
    #[test]
    fn los_envios_no_prometen_entrega() {
        let mut e = base();
        e.nombre = "Ana".into();
        e.envios.push(EnvioDelExpediente {
            canal: Canal::Correo,
            direccion: "ana@empresa.mx".into(),
            campana: "Boletín de marzo".into(),
            estado: "aceptado por el proveedor".into(),
            cuando: "3 de marzo de 2026".into(),
        });

        let texto = redactar(&e);
        assert!(texto.contains("No significa que llegara"), "{texto}");
        assert!(
            !texto.to_lowercase().contains("entregado el"),
            "el informe afirma una entrega que ARLES no puede saber:\n{texto}"
        );
    }

    /// Los apartados que no aplican **no dejan un hueco mudo**: se dice que no
    /// consta nada. Un apartado vacío se lee como un fallo del informe.
    #[test]
    fn los_apartados_sin_datos_dicen_que_no_hay_nada() {
        let mut e = base();
        e.nombre = "Ana".into();

        let texto = redactar(&e);
        assert!(
            texto.contains("No consta ninguna."),
            "faltan las negativas:\n{texto}"
        );
        assert!(
            texto.contains("No consta ningún envío"),
            "faltan los envíos:\n{texto}"
        );
    }
}
