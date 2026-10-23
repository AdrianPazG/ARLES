//! Reunir todo lo que ARLES tiene sobre una persona (entrega 3.4).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! EL DERECHO DE ACCESO, POR DENTRO
//!
//! Alguien pregunta «qué tienes sobre mí». Esto recorre **todas** las tablas
//! donde puede haber algo suyo y lo reúne. Redactarlo es de `arles_core::arco`;
//! aquí sólo se busca.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! SE BUSCA POR DIRECCIÓN, NO POR CONTACTO
//!
//! Y es la decisión que hace que esto funcione.
//!
//! Una persona puede **no tener contacto y aun así estar en ARLES**: pidió la
//! baja, alguien borró su ficha, y lo que queda es su entrada en la lista de
//! supresión —que sobrevive a propósito—. También puede quedar su
//! consentimiento registrado, o los envíos que se le hicieron, porque ésos
//! guardan la dirección y no el identificador.
//!
//! Si esto buscara por contacto, a esa persona se le contestaría «no tenemos
//! nada» mientras su dirección sigue en tres tablas. Y es justo la persona que
//! más motivos tiene para preguntar: la que pidió que la borraran y quiere
//! comprobar que se hizo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA CONSULTA QUEDA EN LA BITÁCORA
//!
//! Atender una solicitud de acceso es un acto que hay que poder demostrar. Se
//! registra en `audit_log` —que es append-only por disparador, no por
//! convención— con la dirección **cifrada con una huella**, no en claro.
//!
//! Por qué la huella: la bitácora sobrevive al borrado del contacto. Si
//! guardara la dirección en claro, cancelar a alguien dejaría su correo en una
//! tabla que nadie puede borrar — deshaciendo el borrado que acaba de pedir.
//! ─────────────────────────────────────────────────────────────────────────

use arles_core::arco::{
    CampoDelExpediente, DireccionDelExpediente, EnvioDelExpediente, Expediente,
    NegativaDelExpediente, OrigenDelExpediente, PermisoDelExpediente,
};
use arles_core::canal::Canal;
use arles_core::ids::CompanyId;
use rusqlite::{Connection, OptionalExtension, params};

use crate::db::Db;
use crate::error::DbError;

/// Cómo se lee cada base de consentimiento. Contrato con la `CHECK` de la V3.
fn base_legible(guardado: &str) -> &'static str {
    match guardado {
        "public_source" => "fuente de acceso público",
        "existing_relationship" => "relación comercial previa",
        "form_optin" => "lo pidió en un formulario",
        "import_affirmation" => "lo afirmó quien importó la lista",
        "verbal" => "lo dijo de viva voz",
        // No se inventa: si aparece una base que no conocemos, se dice. El
        // informe es una respuesta legal, y una base mal traducida afirmaría
        // algo que nadie declaró.
        _ => "base no reconocida",
    }
}

/// Cómo se lee el estado de un envío.
///
/// **Ninguno dice «entregado»** (§65). ARLES no puede saberlo: lo más que sabe
/// es que el proveedor lo aceptó.
fn estado_legible(guardado: &str) -> &'static str {
    match guardado {
        "queued" => "en cola",
        "claimed" | "sending" => "enviándose",
        "sent" => "aceptado por el proveedor",
        "failed" => "falló",
        "suppressed" => "no se envió: la dirección había pedido no recibir",
        "cancelled" => "cancelado",
        "presumed_sent" => "pudo haberse enviado (se interrumpió sin confirmar)",
        _ => "estado no reconocido",
    }
}

impl Db {
    /// Reúne todo lo que hay sobre una dirección.
    ///
    /// `direccion` tiene que venir **ya normalizada** por el núcleo: es la
    /// forma que guardan todas estas tablas, y buscar por lo que el usuario
    /// escribió no encontraría nada — `Ana@Empresa.MX` no es `ana@empresa.mx`.
    ///
    /// Devuelve un expediente aunque no haya nada. **Vacío es una respuesta
    /// válida**, no un error: «no tenemos nada sobre usted» es exactamente lo
    /// que alguien puede estar preguntando.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si alguna consulta falla. **No se devuelve un
    /// expediente a medias**: un informe de acceso incompleto es peor que
    /// ninguno, porque parece completo.
    pub fn expediente_de(
        &self,
        empresa: CompanyId,
        direccion: &str,
        nombre_de_la_empresa: &str,
        fecha: &str,
    ) -> Result<Expediente, DbError> {
        self.con_dominio(|conn| {
            let mut e = Expediente {
                se_pregunto_por: direccion.to_owned(),
                empresa: nombre_de_la_empresa.to_owned(),
                fecha: fecha.to_owned(),
                ..Default::default()
            };

            let emp = empresa.to_string();

            // ── ¿Hay un contacto vivo con esta dirección? ──────────────
            let contacto: Option<(String, String, String, String, String, String)> = conn
                .query_row(
                    "SELECT c.id,
                            COALESCE(c.first_name, ''),
                            COALESCE(c.last_name, ''),
                            -- La empresa del contacto vive en `source`: es
                            -- cómo la guarda `contactos.rs` desde la 3.2.
                            COALESCE(c.source, ''),
                            c.created_at,
                            c.updated_at
                       FROM contact_channel ch
                       JOIN contact c ON c.id = ch.contact_id
                      WHERE ch.company_id = ?1
                        AND ch.value_normalized = ?2
                        AND ch.deleted_at IS NULL
                        AND c.deleted_at IS NULL
                      LIMIT 1",
                    params![emp, direccion],
                    |f| {
                        Ok((
                            f.get(0)?,
                            f.get(1)?,
                            f.get(2)?,
                            f.get(3)?,
                            f.get(4)?,
                            f.get(5)?,
                        ))
                    },
                )
                .optional()?;

            // Las direcciones de esa persona: **todas**, no sólo por la que
            // preguntó. Quien pregunta por su correo tiene derecho a saber que
            // también guardamos su móvil.
            let mut suyas: Vec<String> = vec![direccion.to_owned()];

            if let Some((id, nombre, apellido, organizacion, alta, modificado)) = contacto {
                e.nombre = nombre;
                e.apellido = apellido;
                e.organizacion = organizacion;
                e.dado_de_alta = alta;
                e.ultima_modificacion = modificado;

                e.direcciones = direcciones_de(conn, &id)?;
                for d in &e.direcciones {
                    if !suyas.contains(&d.valor) {
                        suyas.push(d.valor.clone());
                    }
                }

                e.campos = campos_de(conn, &id)?;
                e.listas = listas_de(conn, &id)?;
                e.etiquetas = etiquetas_de(conn, &id)?;
                e.origen = origen_de(conn, &id)?;
            }

            // ── Lo que sobrevive al contacto ──────────────────────────
            //
            // Se busca por cada dirección suya, porque estas tres tablas
            // guardan la dirección y no el identificador — a propósito, para
            // que sobrevivan al borrado.
            for d in &suyas {
                e.negativas.extend(negativas_de(conn, &emp, d)?);
                e.permisos.extend(permisos_de(conn, &emp, d)?);
                e.envios.extend(envios_de(conn, &emp, d)?);
            }

            Ok(e)
        })
    }

    /// Deja constancia de que se atendió una solicitud de acceso.
    ///
    /// La dirección se guarda **como huella**, no en claro: ver la nota de
    /// cabecera del módulo.
    ///
    /// Se llama **después** de entregar el informe y no antes: registrar una
    /// entrega que no llegó a ocurrir convertiría la bitácora en una lista de
    /// intenciones.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn registrar_acceso(&self, empresa: CompanyId, direccion: &str) -> Result<(), DbError> {
        self.con_dominio(|conn| {
            crate::empresa::auditar(
                conn,
                &empresa.to_string(),
                "arco_acceso_entregado",
                "direccion",
                &arles_core::huella(direccion),
            )?;
            Ok(())
        })
    }
}

fn direcciones_de(
    conn: &Connection,
    contacto: &str,
) -> Result<Vec<DireccionDelExpediente>, DbError> {
    let mut consulta = conn.prepare(
        "SELECT channel, value_normalized, created_at,
                CASE WHEN id = (
                    SELECT p.id FROM contact_channel p
                     WHERE p.contact_id = contact_channel.contact_id
                       AND p.channel = contact_channel.channel
                       AND p.deleted_at IS NULL
                     ORDER BY p.created_at, p.id LIMIT 1
                ) THEN 1 ELSE 0 END
           FROM contact_channel
          WHERE contact_id = ?1 AND deleted_at IS NULL
          ORDER BY CASE channel WHEN 'email' THEN 0 ELSE 1 END, created_at, id",
    )?;
    let filas = consulta.query_map(params![contacto], |f| {
        Ok((
            f.get::<_, String>(0)?,
            f.get::<_, String>(1)?,
            f.get::<_, String>(2)?,
            f.get::<_, i64>(3)?,
        ))
    })?;

    let mut salida = Vec::new();
    for fila in filas {
        let (canal, valor, alta, principal) = fila?;
        salida.push(DireccionDelExpediente {
            canal: Canal::desde_texto(&canal).map_err(|_| DbError::DatoInvalido {
                motivo: "un canal guardado no es uno de los admitidos",
            })?,
            valor,
            principal: principal != 0,
            dada_de_alta: alta,
        });
    }
    Ok(salida)
}

fn campos_de(conn: &Connection, contacto: &str) -> Result<Vec<CampoDelExpediente>, DbError> {
    let mut consulta = conn.prepare(
        "SELECT field_key, COALESCE(field_value, '')
           FROM contact_field WHERE contact_id = ?1 ORDER BY field_key",
    )?;
    let filas = consulta.query_map(params![contacto], |f| {
        Ok(CampoDelExpediente {
            clave: f.get(0)?,
            valor: f.get(1)?,
        })
    })?;
    Ok(filas.collect::<Result<_, _>>()?)
}

fn listas_de(conn: &Connection, contacto: &str) -> Result<Vec<String>, DbError> {
    let mut consulta = conn.prepare(
        "SELECT l.name FROM contact_list_member m
           JOIN contact_list l ON l.id = m.list_id
          WHERE m.contact_id = ?1 ORDER BY l.name",
    )?;
    let filas = consulta.query_map(params![contacto], |f| f.get::<_, String>(0))?;
    Ok(filas.collect::<Result<_, _>>()?)
}

fn etiquetas_de(conn: &Connection, contacto: &str) -> Result<Vec<String>, DbError> {
    let mut consulta = conn.prepare(
        "SELECT t.name FROM contact_tag ct
           JOIN tag t ON t.id = ct.tag_id
          WHERE ct.contact_id = ?1 ORDER BY t.name",
    )?;
    let filas = consulta.query_map(params![contacto], |f| f.get::<_, String>(0))?;
    Ok(filas.collect::<Result<_, _>>()?)
}

fn origen_de(conn: &Connection, contacto: &str) -> Result<Option<OrigenDelExpediente>, DbError> {
    let fila: Option<(String, String, String, String)> = conn
        .query_row(
            "SELECT b.original_filename, b.origin, b.created_at,
                    COALESCE(b.consent_affirmation, '')
               FROM contact c
               JOIN import_batch b ON b.id = c.import_batch_id
              WHERE c.id = ?1",
            params![contacto],
            |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?, f.get(3)?)),
        )
        .optional()?;

    let Some((archivo, origen, cuando, texto)) = fila else {
        return Ok(None);
    };

    // El origen declarado se traduce a palabras. Un `evento_o_feria` en un
    // informe legal es jerga nuestra, no una respuesta.
    let legible = match arles_core::OrigenDeLaLista::desde_texto(&origen) {
        Ok(arles_core::OrigenDeLaLista::FormularioPropio) => "un formulario de la propia empresa",
        Ok(arles_core::OrigenDeLaLista::ClientesExistentes) => "clientes ya existentes",
        Ok(arles_core::OrigenDeLaLista::EventoOFeria) => "un evento o una feria",
        Ok(arles_core::OrigenDeLaLista::DirectorioPublico) => "un directorio público",
        Ok(arles_core::OrigenDeLaLista::Otro) => "otro origen",
        Err(_) => "origen no reconocido",
    };

    Ok(Some(OrigenDelExpediente {
        nombre_del_archivo: archivo,
        origen_declarado: legible.to_owned(),
        cuando,
        texto_aceptado: texto,
    }))
}

fn negativas_de(
    conn: &Connection,
    empresa: &str,
    direccion: &str,
) -> Result<Vec<NegativaDelExpediente>, DbError> {
    let mut consulta = conn.prepare(
        "SELECT channel, reason, created_at FROM suppression_entry
          WHERE company_id = ?1 AND address_normalized = ?2
          ORDER BY created_at",
    )?;
    let filas = consulta.query_map(params![empresa, direccion], |f| {
        Ok((
            f.get::<_, String>(0)?,
            f.get::<_, String>(1)?,
            f.get::<_, String>(2)?,
        ))
    })?;

    let mut salida = Vec::new();
    for fila in filas {
        let (canal, motivo, cuando) = fila?;
        // El motivo se traduce a palabras llanas aquí y no con la clave de
        // i18n: este informe no lo pinta la interfaz, lo escribe Rust en un
        // archivo, y `supresion.motivo.baja` no le dice nada a quien lo lea.
        let legible = match arles_core::MotivoDeSupresion::desde_texto(&motivo) {
            Ok(arles_core::MotivoDeSupresion::RebotePermanente) => {
                "el correo rebotó de forma definitiva"
            }
            Ok(arles_core::MotivoDeSupresion::Baja) => "pidió la baja",
            Ok(arles_core::MotivoDeSupresion::Manual) => "lo añadió la empresa a mano",
            Ok(arles_core::MotivoDeSupresion::Queja) => "marcó el correo como no deseado",
            Ok(arles_core::MotivoDeSupresion::NumeroInvalido) => {
                "el número no existe o no tiene WhatsApp"
            }
            Ok(arles_core::MotivoDeSupresion::FalloDelCanal) => {
                "el canal falló de forma definitiva"
            }
            Err(_) => "motivo no reconocido",
        };
        salida.push(NegativaDelExpediente {
            canal: Canal::desde_texto(&canal).map_err(|_| DbError::DatoInvalido {
                motivo: "una fila de supresión tiene un canal desconocido",
            })?,
            direccion: direccion.to_owned(),
            motivo: legible.to_owned(),
            desde: cuando,
        });
    }
    Ok(salida)
}

fn permisos_de(
    conn: &Connection,
    empresa: &str,
    direccion: &str,
) -> Result<Vec<PermisoDelExpediente>, DbError> {
    let mut consulta = conn.prepare(
        "SELECT channel, kind, basis, recorded_at FROM consent_entry
          WHERE company_id = ?1 AND address_normalized = ?2
          ORDER BY recorded_at",
    )?;
    let filas = consulta.query_map(params![empresa, direccion], |f| {
        Ok((
            f.get::<_, String>(0)?,
            f.get::<_, String>(1)?,
            f.get::<_, String>(2)?,
            f.get::<_, String>(3)?,
        ))
    })?;

    let mut salida = Vec::new();
    for fila in filas {
        let (canal, tipo, base, cuando) = fila?;
        salida.push(PermisoDelExpediente {
            canal: Canal::desde_texto(&canal).map_err(|_| DbError::DatoInvalido {
                motivo: "un permiso guardado tiene un canal desconocido",
            })?,
            direccion: direccion.to_owned(),
            tipo: if tipo == "granted" {
                "Permiso concedido".to_owned()
            } else {
                "Permiso retirado".to_owned()
            },
            base: base_legible(&base).to_owned(),
            cuando,
        });
    }
    Ok(salida)
}

fn envios_de(
    conn: &Connection,
    empresa: &str,
    direccion: &str,
) -> Result<Vec<EnvioDelExpediente>, DbError> {
    // Se une con `campaign` para poder nombrar la campaña. Un identificador
    // no le dice nada a quien lea el informe.
    let mut consulta = conn.prepare(
        "SELECT a.channel, COALESCE(c.name, ''), a.state,
                COALESCE(a.sent_at, a.created_at)
           FROM message_attempt a
           JOIN campaign c ON c.id = a.campaign_id
          WHERE c.company_id = ?1 AND a.contact_address = ?2
          ORDER BY COALESCE(a.sent_at, a.created_at)",
    )?;
    let filas = consulta.query_map(params![empresa, direccion], |f| {
        Ok((
            f.get::<_, String>(0)?,
            f.get::<_, String>(1)?,
            f.get::<_, String>(2)?,
            f.get::<_, String>(3)?,
        ))
    })?;

    let mut salida = Vec::new();
    for fila in filas {
        let (canal, campana, estado, cuando) = fila?;
        salida.push(EnvioDelExpediente {
            canal: Canal::desde_texto(&canal).map_err(|_| DbError::DatoInvalido {
                motivo: "un envío guardado tiene un canal desconocido",
            })?,
            direccion: direccion.to_owned(),
            campana,
            estado: estado_legible(&estado).to_owned(),
            cuando,
        });
    }
    Ok(salida)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arles_core::canal::ValorDeCanal;
    use arles_core::contacto::{BorradorDeCanal, BorradorDeContacto, DatosDeContacto};
    use arles_core::supresion::{
        AlcanceDeSupresion, MotivoDeSupresion, OrigenDeSupresion, PeticionDeSupresion,
    };

    use crate::conexion::ClaveMaestra;

    const FECHA: &str = "21 de septiembre de 2026";

    fn base() -> (tempfile::TempDir, Db, CompanyId) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        let empresa = db
            .guardar_empresa(
                &arles_core::DatosDeEmpresa::validar(&arles_core::BorradorDeEmpresa {
                    nombre_comercial: "TELEMETRY INSIGHT".into(),
                    pais: "MX".into(),
                    zona_horaria: "America/Mexico_City".into(),
                    correo_corporativo: "hola@telemetrymx.com".into(),
                    sitio_web: String::new(),
                })
                .expect("empresa válida"),
            )
            .expect("guarda empresa");
        (dir, db, empresa)
    }

    fn contacto(canales: &[(Canal, &str)]) -> DatosDeContacto {
        DatosDeContacto::validar(
            &BorradorDeContacto {
                nombre: "Ana".into(),
                apellido: "Ruiz".into(),
                empresa: "Empresa SA".into(),
                canales: canales
                    .iter()
                    .map(|(c, v)| BorradorDeCanal {
                        canal: *c,
                        valor: (*v).to_owned(),
                        principal: false,
                    })
                    .collect(),
            },
            "MX",
        )
        .expect("contacto válido")
    }

    fn cv(canal: Canal, valor: &str) -> arles_core::contacto::CanalValidado {
        let v = ValorDeCanal::parse(canal, valor, "MX").expect("valor válido");
        arles_core::contacto::CanalValidado {
            canal: v.canal(),
            valor_raw: v.raw().to_owned(),
            valor_normalizado: v.normalized().to_owned(),
            principal: true,
        }
    }

    fn expediente(db: &Db, empresa: CompanyId, direccion: &str) -> Expediente {
        db.expediente_de(empresa, direccion, "TELEMETRY INSIGHT", FECHA)
            .expect("reúne el expediente")
    }

    /// Preguntar por alguien de quien no hay nada **no es un error**.
    #[test]
    fn de_quien_no_esta_sale_un_expediente_vacio_y_no_un_error() {
        let (_d, db, empresa) = base();
        let e = expediente(&db, empresa, "nadie@empresa.mx");

        assert!(!e.hay_algo());
        assert_eq!(e.se_pregunto_por, "nadie@empresa.mx");
        assert_eq!(
            e.empresa, "TELEMETRY INSIGHT",
            "falta quién tiene los datos"
        );
    }

    #[test]
    fn de_un_contacto_vivo_sale_todo_lo_suyo() {
        let (_d, db, empresa) = base();
        db.crear_contacto(
            empresa,
            &contacto(&[
                (Canal::Correo, "Ana@Empresa.MX"),
                (Canal::WhatsApp, "81 1234 5678"),
            ]),
        )
        .expect("crea");

        let e = expediente(&db, empresa, "ana@empresa.mx");

        assert!(e.hay_algo());
        assert!(e.sigue_siendo_contacto());
        assert_eq!(e.nombre, "Ana");
        assert_eq!(e.apellido, "Ruiz");
        assert_eq!(e.organizacion, "Empresa SA");
        assert_eq!(
            e.direcciones.len(),
            2,
            "quien pregunta por su correo tiene derecho a saber que también \
             guardamos su móvil"
        );
    }

    /// **EL CASO QUE JUSTIFICA BUSCAR POR DIRECCIÓN.**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// Ana pide la baja, alguien borra su ficha, y meses después Ana escribe
    /// preguntando qué queda de ella.
    ///
    /// Si el expediente se buscara por contacto, la respuesta sería «no
    /// tenemos nada» — mientras su dirección sigue en la lista de supresión.
    /// Sería mentirle justo a la persona que más motivos tiene para preguntar.
    ///
    /// Se verificó rompiendo la garantía: haciendo que la búsqueda de
    /// negativas y permisos sólo mire cuando hay contacto vivo, esta prueba
    /// falla diciendo que a Ana se le contestó que no había nada.
    /// ─────────────────────────────────────────────────────────────────────
    #[test]
    fn de_quien_pidio_la_baja_y_fue_borrado_sigue_habiendo_expediente() {
        let (_d, db, empresa) = base();

        let ana = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");
        db.suprimir(
            empresa,
            &PeticionDeSupresion::nueva(
                vec![cv(Canal::Correo, "ana@empresa.mx")],
                AlcanceDeSupresion::Canal,
                MotivoDeSupresion::Baja,
                OrigenDeSupresion::Usuario,
                "",
            )
            .expect("petición"),
        )
        .expect("suprime");
        db.borrar_contacto(empresa, ana).expect("borra");

        let e = expediente(&db, empresa, "ana@empresa.mx");

        assert!(
            e.hay_algo(),
            "a Ana se le contestaría «no tenemos nada» teniendo su baja guardada"
        );
        assert!(!e.sigue_siendo_contacto(), "su ficha sí se borró");
        assert_eq!(e.negativas.len(), 1);
        assert_eq!(
            e.negativas.first().map(|n| n.motivo.as_str()),
            Some("pidió la baja"),
            "el motivo tiene que salir en palabras, no como clave de i18n"
        );

        // Y el informe redactado se lo dice con todas las letras.
        let texto = arles_core::arco::redactar(&e);
        assert!(!texto.contains("NO HAY NINGÚN DATO"), "{texto}");
        assert!(texto.contains("pidió la baja"));
    }

    /// El origen de la importación sale **en palabras**, y con el texto que se
    /// aceptó entero: es la prueba de qué se afirmó.
    #[test]
    fn el_origen_de_la_importacion_sale_en_palabras_y_con_su_texto() {
        let (_d, db, empresa) = base();
        let texto_aceptado = "Declaro que obtuve estos contactos de forma lícita.";

        db.importar_contactos(
            empresa,
            &crate::contactos::DatosDelLote {
                nombre_del_archivo: "feria-septiembre.csv".into(),
                huella_del_archivo: "sha256:x".into(),
                mapeo_en_json: "{}".into(),
                texto_del_consentimiento: texto_aceptado.into(),
                origen: arles_core::OrigenDeLaLista::EventoOFeria,
                total_de_filas: 1,
                choques: 0,
                invalidas: 0,
                lista: None,
                direcciones: Vec::new(),
            },
            &[contacto(&[(Canal::Correo, "ana@empresa.mx")])],
        )
        .expect("importa");

        let e = expediente(&db, empresa, "ana@empresa.mx");
        let o = e.origen.as_ref().expect("tiene origen");

        assert_eq!(o.nombre_del_archivo, "feria-septiembre.csv");
        assert_eq!(
            o.origen_declarado, "un evento o una feria",
            "el origen salió como jerga nuestra y no como una respuesta"
        );
        assert_eq!(o.texto_aceptado, texto_aceptado);
    }

    /// La consulta queda en la bitácora, y **con la dirección como huella**:
    /// si se guardara en claro, cancelar a alguien dejaría su correo en una
    /// tabla que nadie puede borrar — deshaciendo el borrado que pidió.
    #[test]
    fn el_acceso_queda_registrado_sin_la_direccion_en_claro() {
        let (_d, db, empresa) = base();
        db.registrar_acceso(empresa, "ana@empresa.mx")
            .expect("registra");

        let (accion, entidad_id) = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT action, entity_id FROM audit_log
                      WHERE action = 'arco_acceso_entregado'",
                    [],
                    |f| Ok((f.get::<_, String>(0)?, f.get::<_, String>(1)?)),
                )
            })
            .expect("hay una fila");

        assert_eq!(accion, "arco_acceso_entregado");
        assert!(
            !entidad_id.contains("ana@empresa.mx"),
            "la dirección quedó en claro en una tabla que nadie puede borrar: {entidad_id}"
        );
        assert!(entidad_id.starts_with("sha256:"), "{entidad_id}");
        // Y la huella es la de esa dirección, no una cualquiera.
        assert_eq!(entidad_id, arles_core::huella("ana@empresa.mx"));
    }

    /// Una dirección de otra empresa no aparece.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// LA SEGUNDA EMPRESA SE INSERTA POR SQL, Y NO ES UN ATAJO
    ///
    /// `guardar_empresa` **actualiza** la fila que ya hay: la v1.2.0 es de una
    /// sola empresa (D-4). La primera versión de esta prueba llamaba dos veces
    /// a `guardar_empresa` creyendo que salían dos, y lo que salía era la
    /// misma con otro nombre — así que la prueba no probaba nada y encima
    /// fallaba.
    ///
    /// Pero el filtro por `company_id` **sí es un invariante real**: el
    /// esquema admite varias empresas porque ARLES RELAY CLOUD lo necesitará,
    /// y el día que las haya, un informe de acceso que cruce de una a otra
    /// enseña datos ajenos a quien no tiene derecho a verlos.
    /// ─────────────────────────────────────────────────────────────────────
    #[test]
    fn el_expediente_no_cruza_de_una_empresa_a_otra() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");

        // Una segunda empresa, y un contacto suyo con su propia dirección.
        let otra = "01900000-0000-7000-8000-0000000000ff";
        db.ejecutar_en_pruebas(|c| {
            let tx = c.unchecked_transaction()?;
            tx.execute(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES (?1, 'OTRA SA', 'MX', 'America/Mexico_City',
                         'hola@otra.mx', ?2, ?2)",
                params![otra, "2026-09-21T00:00:00Z"],
            )?;
            tx.execute(
                "INSERT INTO contact (id, company_id, first_name, created_at, updated_at)
                 VALUES ('c-otra', ?1, 'Beto', ?2, ?2)",
                params![otra, "2026-09-21T00:00:00Z"],
            )?;
            tx.execute(
                "INSERT INTO contact_channel
                     (id, company_id, contact_id, channel, value_raw,
                      value_normalized, created_at, updated_at)
                 VALUES ('ch-otra', ?1, 'c-otra', 'email', 'beto@otra.mx',
                         'beto@otra.mx', ?2, ?2)",
                params![otra, "2026-09-21T00:00:00Z"],
            )?;
            tx.commit()
        })
        .expect("siembra la otra empresa");

        // Desde la empresa de siempre, Beto no existe.
        let e = expediente(&db, empresa, "beto@otra.mx");
        assert!(
            !e.hay_algo(),
            "el expediente enseñó a un contacto de otra empresa"
        );

        // Y la comprobación al revés, que es la que hace que la de arriba
        // valga algo: si nada se encontrara nunca, la primera pasaría igual.
        let e = expediente(&db, empresa, "ana@empresa.mx");
        assert!(
            e.hay_algo(),
            "tampoco encontró a la suya: la prueba sería vacua"
        );
    }
}
