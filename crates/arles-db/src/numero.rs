//! El número de WhatsApp guardado, y las dos cosas que ARLES no puede
//! comprobar por sí mismo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! AQUÍ NO HAY NINGÚN TOKEN, IGUAL QUE EN `correo.rs` NO HAY CONTRASEÑAS
//!
//! Lo que se guarda es `credential_ref`: una referencia **opaca** al llavero
//! del sistema. No sirve para nada sin el llavero, y nada de lo que sale de
//! este módulo la puede convertir en un token.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LAS DOS CONFIRMACIONES SON MARCAS DE TIEMPO, NO CASILLAS
//!
//! Coexistencia (L-11) y la aceptación del riesgo del modo en frío se guardan
//! con **cuándo** se confirmaron, no con un sí. La diferencia es lo que hace
//! que sirvan para algo:
//!
//! · Un booleano contesta «alguien marcó una casilla alguna vez». Una fecha
//!   contesta «el 14 de octubre, y desde entonces han pasado tres meses».
//! · El preflight de cada campaña vuelve a preguntar por Coexistencia, y para
//!   poder decir «lo confirmaste hace tres meses» hace falta la fecha.
//! · Y el día que caiga un número, la pregunta va a ser «¿le advirtieron?».
//!   Un sí sin fecha no responde a eso.
//!
//! ARLES **no puede comprobar ninguna de las dos**: no ve dentro de Meta. Las
//! pregunta, las registra aquí y en `audit_log`, y no finge que las sabe.

use arles_core::ids::{CompanyId, WhatsappAccountId};
use arles_core::whatsapp::ModoDelCanal;
use rusqlite::params;

use crate::db::Db;
use crate::empresa::{ahora, auditar};
use crate::error::DbError;

/// Lo que hace falta para dar de alta un número.
#[derive(Debug, Clone)]
pub struct NumeroNuevo<'a> {
    /// Cómo se llama en la pantalla. No sale en ningún mensaje.
    pub nombre_visible: &'a str,
    /// El número en E.164, tal como lo produce `arles_core::PhoneNumber`.
    pub telefono_e164: &'a str,
    /// El identificador de la cuenta de empresa en Meta. **No es un secreto**:
    /// es una dirección, y Meta la enseña en su propio panel.
    pub waba_id: &'a str,
    /// El identificador del número en Cloud API. Tampoco es un secreto.
    pub phone_number_id: &'a str,
    /// Seguimiento o prospección directa (L-9).
    pub modo: ModoDelCanal,
}

/// Un número tal como se enseña. **Sin token.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumeroGuardado {
    pub id: WhatsappAccountId,
    pub nombre_visible: String,
    pub telefono_e164: String,
    pub waba_id: String,
    pub phone_number_id: String,
    pub modo: ModoDelCanal,
    /// La referencia opaca al llavero. **No es el token** y no sirve sin él.
    pub referencia_de_credencial: String,
    /// `connected`, `needs_reauth`, `circuit_open` o `disabled`.
    pub estado: String,
    /// Cuándo se confirmó Coexistencia. `None` = no se ha confirmado.
    pub coexistencia_confirmada: Option<String>,
    /// Cuándo se aceptó el riesgo del modo en frío. `None` = no se ha aceptado.
    pub riesgo_aceptado: Option<String>,
    /// Lo que Meta dice de la calidad: `green`, `yellow`, `red` o `unknown`.
    pub calidad: Option<String>,
    /// El tope diario que Meta impone. Lo decide Meta, no nosotros.
    pub limite_de_meta: Option<u32>,
}

impl NumeroGuardado {
    /// ¿Este número puede mandar una campaña **hoy**?
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// LA PUERTA DEL CANAL, EN UN SOLO SITIO
    ///
    /// Tres condiciones, y la tercera sólo aplica a un modo:
    ///
    /// 1. La cuenta está conectada.
    /// 2. Coexistencia está confirmada (L-11). Sin ella, quien atienda no
    ///    puede responder desde su teléfono y las respuestas no llegan — así
    ///    que la cifra que más protege al número sería siempre cero.
    /// 3. Y **si el modo escribe en frío**, el riesgo está aceptado.
    ///
    /// Escrito por lo que se exige y no por lo que se rechaza, igual que la
    /// comprobación de la cuenta de correo: un estado nuevo en el esquema no
    /// debe empezar a enviar solo.
    #[must_use]
    pub fn puede_enviar(&self) -> bool {
        self.estado == "connected"
            && self.coexistencia_confirmada.is_some()
            && (!self.modo.escribe_en_frio() || self.riesgo_aceptado.is_some())
    }
}

impl Db {
    /// Da de alta un número.
    ///
    /// Nace **sin ninguna de las dos confirmaciones**, y por eso no puede
    /// enviar todavía. Darlas por supuestas al crear sería justo lo que estas
    /// marcas existen para impedir: que alguien pueda decir que nadie le avisó.
    ///
    /// La contraseña —el token de Meta— no entra aquí: la guarda `arles-app` en
    /// el llavero, y lo único que llega a esta tabla es la referencia opaca.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla.
    /// `id` se genera **fuera**, y no aquí, porque la referencia del llavero lo
    /// lleva dentro: `llavero://…/whatsapp-number/{id}`. Generándolo dentro no
    /// habría forma de construir esa referencia antes de llamar, y la fila
    /// acabaría apuntando a una entrada del llavero que no es la suya.
    ///
    /// Es el mismo reparto que `alta_de_cuenta_de_correo`. Se corrigió el
    /// 29/09/2026, al enchufar los comandos de la W.6: la firma original
    /// generaba el identificador dentro y hacía imposible la llamada.
    pub fn guardar_numero(
        &self,
        empresa: CompanyId,
        id: WhatsappAccountId,
        nuevo: &NumeroNuevo<'_>,
        referencia_de_credencial: &str,
    ) -> Result<WhatsappAccountId, DbError> {
        self.con(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();

            tx.execute(
                "INSERT INTO whatsapp_account (id, company_id, display_name, phone_e164,
                                               waba_id, phone_number_id, credential_ref,
                                               modo, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'connected', ?9, ?9)",
                params![
                    id.to_string(),
                    empresa.to_string(),
                    nuevo.nombre_visible.trim(),
                    nuevo.telefono_e164,
                    nuevo.waba_id.trim(),
                    nuevo.phone_number_id.trim(),
                    referencia_de_credencial,
                    nuevo.modo.como_se_guarda(),
                    cuando
                ],
            )?;

            auditar(
                &tx,
                &empresa.to_string(),
                "crear",
                "whatsapp_account",
                &id.to_string(),
            )?;
            tx.commit()?;
            Ok(id)
        })
    }

    /// Los números de la empresa.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla, o [`DbError::Nucleo`] si una
    /// fila trae un modo que el núcleo no reconoce. **No se salta**: ver la
    /// nota de `ModoDeCanalDesconocido`.
    pub fn numeros(&self, empresa: CompanyId) -> Result<Vec<NumeroGuardado>, DbError> {
        self.con_dominio(|conn| {
            let mut consulta = conn.prepare(
                "SELECT id, display_name, phone_e164, waba_id, phone_number_id,
                        modo, credential_ref, status,
                        coexistencia_confirmada_at, riesgo_aceptado_at,
                        calidad, limite_diario_meta
                   FROM whatsapp_account
                  WHERE company_id = ?1
                  ORDER BY created_at",
            )?;

            let filas = consulta.query_map(params![empresa.to_string()], |f| {
                Ok((
                    f.get::<_, String>(0)?,
                    f.get::<_, String>(1)?,
                    f.get::<_, String>(2)?,
                    f.get::<_, String>(3)?,
                    f.get::<_, String>(4)?,
                    f.get::<_, String>(5)?,
                    f.get::<_, String>(6)?,
                    f.get::<_, String>(7)?,
                    f.get::<_, Option<String>>(8)?,
                    f.get::<_, Option<String>>(9)?,
                    f.get::<_, Option<String>>(10)?,
                    f.get::<_, Option<i64>>(11)?,
                ))
            })?;

            let mut todos = Vec::new();
            for fila in filas {
                let (id, nombre, tel, waba, pn, modo, cred, estado, coex, riesgo, cal, tope) =
                    fila?;
                todos.push(NumeroGuardado {
                    id: id.parse().map_err(DbError::Nucleo)?,
                    nombre_visible: nombre,
                    telefono_e164: tel,
                    waba_id: waba,
                    phone_number_id: pn,
                    modo: ModoDelCanal::de_guardado(&modo).map_err(DbError::Nucleo)?,
                    referencia_de_credencial: cred,
                    estado,
                    coexistencia_confirmada: coex,
                    riesgo_aceptado: riesgo,
                    calidad: cal,
                    limite_de_meta: tope.and_then(|v| u32::try_from(v).ok()),
                });
            }
            Ok(todos)
        })
    }

    /// Un número por su identificador.
    ///
    /// # Errores
    ///
    /// Los de [`Db::numeros`].
    pub fn numero(
        &self,
        empresa: CompanyId,
        id: WhatsappAccountId,
    ) -> Result<Option<NumeroGuardado>, DbError> {
        Ok(self.numeros(empresa)?.into_iter().find(|n| n.id == id))
    }

    /// Anota que se confirmó Coexistencia (L-11).
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// SE VUELVE A ESCRIBIR CADA VEZ, Y ESO ES DELIBERADO
    ///
    /// Si ya estaba confirmada, la fecha se actualiza. Podría parecer que hay
    /// que conservar la primera, pero lo que el preflight pregunta es «¿sigue
    /// activa **hoy**?», y la respuesta útil es la última vez que alguien lo
    /// miró — no la primera vez que alguien lo dijo.
    ///
    /// La bitácora sí guarda las dos: cada confirmación deja su línea, así que
    /// el historial completo está en `audit_log`, que no se edita.
    ///
    /// # Errores
    ///
    /// [`DbError::DatoInvalido`] si ese número no existe o no es de esta
    /// empresa. [`DbError::Sqlite`] si la escritura falla.
    pub fn confirmar_coexistencia(
        &self,
        empresa: CompanyId,
        id: WhatsappAccountId,
        quien: &str,
    ) -> Result<(), DbError> {
        self.marcar(
            empresa,
            id,
            quien,
            "coexistencia_confirmada_at",
            "coexistencia_confirmada_by",
            "confirmar_coexistencia",
        )
    }

    /// Anota que se aceptó el riesgo del modo en frío.
    ///
    /// Es la otra mitad de la puerta, y sólo hace falta para
    /// [`ModoDelCanal::ProspeccionDirecta`]. Deja su línea en la bitácora por
    /// la misma razón que el aviso de más de 50 diarios: el día que un número
    /// caiga, la pregunta será si alguien lo advirtió.
    ///
    /// # Errores
    ///
    /// Los de [`Db::confirmar_coexistencia`].
    pub fn aceptar_el_riesgo(
        &self,
        empresa: CompanyId,
        id: WhatsappAccountId,
        quien: &str,
    ) -> Result<(), DbError> {
        self.marcar(
            empresa,
            id,
            quien,
            "riesgo_aceptado_at",
            "riesgo_aceptado_by",
            "aceptar_riesgo_de_whatsapp",
        )
    }

    /// Lo común de las dos confirmaciones.
    ///
    /// Los nombres de columna son **constantes del código**, nunca texto de
    /// fuera: esta función es privada y sus dos únicos llamadores están arriba.
    /// Aun así conviene decirlo, porque una función que compone SQL con un
    /// nombre de columna es exactamente la forma que tiene una inyección el día
    /// que alguien la haga pública.
    fn marcar(
        &self,
        empresa: CompanyId,
        id: WhatsappAccountId,
        quien: &str,
        columna_cuando: &'static str,
        columna_quien: &'static str,
        accion: &'static str,
    ) -> Result<(), DbError> {
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();

            let filas = tx.execute(
                &format!(
                    "UPDATE whatsapp_account
                        SET {columna_cuando} = ?3, {columna_quien} = ?4, updated_at = ?3
                      WHERE id = ?1 AND company_id = ?2"
                ),
                params![id.to_string(), empresa.to_string(), cuando, quien],
            )?;

            if filas == 0 {
                return Err(DbError::DatoInvalido {
                    motivo: "ese número de WhatsApp no existe",
                });
            }

            auditar(
                &tx,
                &empresa.to_string(),
                accion,
                "whatsapp_account",
                &id.to_string(),
            )?;
            tx.commit()?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conexion::ClaveMaestra;

    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";

    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('{EMPRESA}', 'TELEMETRY', 'MX', 'America/Mexico_City',
                         'hola@t.mx', '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');"
            ))
        })
        .expect("siembra");
        (dir, db)
    }

    fn empresa() -> CompanyId {
        EMPRESA.parse().expect("id")
    }

    fn alta(db: &Db, modo: ModoDelCanal) -> WhatsappAccountId {
        // El identificador se genera aquí y la referencia lo nombra, que es
        // justamente el reparto que la firma exige. Antes del 29/09/2026 esta
        // prueba pasaba una referencia acabada en `/x` —un identificador que no
        // existía— y nadie lo notaba, porque el método generaba el suyo dentro
        // y la referencia no se volvía a leer.
        let id = WhatsappAccountId::nuevo();
        db.guardar_numero(
            empresa(),
            id,
            &NumeroNuevo {
                nombre_visible: "Ventas",
                telefono_e164: "+528112345678",
                waba_id: "waba-1",
                phone_number_id: "pn-1",
                modo,
            },
            &format!("llavero://mx.telemetryinsight.arlesrelay/whatsapp-number/{id}"),
        )
        .expect("da de alta")
    }

    fn leer(db: &Db, id: WhatsappAccountId) -> NumeroGuardado {
        db.numero(empresa(), id)
            .expect("lee")
            .unwrap_or_else(|| panic!("el número que se acaba de guardar"))
    }

    /// **Un número recién dado de alta no puede enviar.**
    ///
    /// Nace sin ninguna de las dos confirmaciones, y ésa es la puerta del canal.
    /// Darlas por supuestas al crear sería exactamente lo que estas marcas
    /// existen para impedir: que alguien pueda decir que nadie le avisó.
    ///
    /// Verificada rompiéndola el 29/09/2026: poniendo `?9` también en
    /// `coexistencia_confirmada_at` al insertar, esta prueba falla.
    #[test]
    fn un_numero_recien_dado_de_alta_no_puede_enviar() {
        let (_d, db) = base();
        let id = alta(&db, ModoDelCanal::Seguimiento);

        let n = leer(&db, id);
        assert_eq!(n.coexistencia_confirmada, None);
        assert_eq!(n.riesgo_aceptado, None);
        assert!(!n.puede_enviar(), "sin Coexistencia no se envía");
    }

    /// **Un número desactivado no envía, aunque tenga las dos confirmaciones.**
    ///
    /// Las confirmaciones son sobre lo que el usuario aceptó; el estado es
    /// sobre lo que la cuenta puede hacer hoy. Son cosas distintas y las dos
    /// tienen que dar permiso.
    ///
    /// Esta prueba faltaba, y lo destapó romper la comprobación a propósito: se
    /// podía cambiar `estado == "connected"` por cualquier cosa y ninguna
    /// prueba se enteraba, porque todas las demás miraban las confirmaciones
    /// sobre una cuenta que siempre estaba conectada.
    ///
    /// Los cuatro estados que no envían van juntos porque el esquema los
    /// admite y hay que verlos rechazados uno por uno — escrito por lo que se
    /// acepta, un estado nuevo tampoco enviaría.
    #[test]
    fn un_numero_que_no_esta_conectado_no_envia() {
        let (_d, db) = base();
        let id = alta(&db, ModoDelCanal::Seguimiento);
        db.confirmar_coexistencia(empresa(), id, "usuario_local")
            .expect("confirma");
        assert!(
            leer(&db, id).puede_enviar(),
            "conectado y confirmado sí envía"
        );

        for estado in ["needs_reauth", "circuit_open", "disabled"] {
            db.ejecutar_en_pruebas(|c| {
                c.execute(
                    "UPDATE whatsapp_account SET status = ?1 WHERE id = ?2",
                    params![estado, id.to_string()],
                )
            })
            .expect("cambia el estado");

            assert!(
                !leer(&db, id).puede_enviar(),
                "«{estado}» no puede enviar aunque tenga Coexistencia"
            );
        }
    }

    /// **Seguimiento necesita Coexistencia. Nada más.**
    ///
    /// Es el modo sin riesgo: se escribe a quien dio permiso o escribió
    /// primero. Pedirle además la aceptación del riesgo del modo en frío sería
    /// hacerle aceptar un riesgo que no corre — y eso enseña a marcar casillas
    /// sin leerlas, que es justo lo que arruina la que sí importa.
    #[test]
    fn seguimiento_solo_necesita_coexistencia() {
        let (_d, db) = base();
        let id = alta(&db, ModoDelCanal::Seguimiento);

        db.confirmar_coexistencia(empresa(), id, "usuario_local")
            .expect("confirma");

        let n = leer(&db, id);
        assert!(n.coexistencia_confirmada.is_some());
        assert_eq!(n.riesgo_aceptado, None, "no se le pide, y no se le pone");
        assert!(n.puede_enviar());
    }

    /// **Prospección directa necesita las dos, y sin la segunda no envía.**
    ///
    /// Es el caso que de verdad importa: el modo en frío es lo único de ARLES
    /// que puede acabar con un número inhabilitado, y eso no se recupera.
    ///
    /// Verificada rompiéndola el 29/09/2026: quitando la tercera condición de
    /// `puede_enviar`, esta prueba falla en la primera comprobación — el número
    /// enviaría en frío con sólo Coexistencia confirmada.
    #[test]
    fn la_prospeccion_directa_necesita_ademas_el_riesgo_aceptado() {
        let (_d, db) = base();
        let id = alta(&db, ModoDelCanal::ProspeccionDirecta);

        db.confirmar_coexistencia(empresa(), id, "usuario_local")
            .expect("confirma");
        assert!(
            !leer(&db, id).puede_enviar(),
            "escribir en frío exige haber aceptado el riesgo"
        );

        db.aceptar_el_riesgo(empresa(), id, "usuario_local")
            .expect("acepta");
        assert!(leer(&db, id).puede_enviar());
    }

    /// **Las dos confirmaciones dejan rastro con fecha en la bitácora.**
    ///
    /// Una casilla marcada no prueba nada dentro de seis meses. Lo que se va a
    /// preguntar el día que caiga un número es «¿le advirtieron?», y a eso
    /// contesta una línea con fecha en un registro que no se edita — no un sí
    /// en una columna que cualquiera puede cambiar.
    #[test]
    fn las_dos_confirmaciones_dejan_su_linea_en_la_bitacora() {
        let (_d, db) = base();
        let id = alta(&db, ModoDelCanal::ProspeccionDirecta);

        db.confirmar_coexistencia(empresa(), id, "usuario_local")
            .expect("confirma");
        db.aceptar_el_riesgo(empresa(), id, "usuario_local")
            .expect("acepta");

        let acciones: Vec<String> = db
            .ejecutar_en_pruebas(|c| {
                let mut q = c.prepare(
                    "SELECT action FROM audit_log
                      WHERE entity_id = ?1 ORDER BY created_at, action",
                )?;
                let filas = q.query_map(params![id.to_string()], |f| f.get::<_, String>(0))?;
                filas.collect()
            })
            .expect("bitácora");

        assert!(acciones.contains(&"crear".to_owned()));
        assert!(acciones.contains(&"confirmar_coexistencia".to_owned()));
        assert!(acciones.contains(&"aceptar_riesgo_de_whatsapp".to_owned()));
    }

    /// **Lo que sale hacia la pantalla no lleva el token.**
    ///
    /// Lo que se guarda es una referencia opaca al llavero, y es lo único que
    /// viaja. Se comprueba que esa referencia **no se parece a una credencial**
    /// y que el tipo no tiene por dónde colar una.
    #[test]
    fn lo_que_sale_no_lleva_el_token() {
        let (_d, db) = base();
        let id = alta(&db, ModoDelCanal::Seguimiento);
        let n = leer(&db, id);

        assert!(
            n.referencia_de_credencial.starts_with("llavero://"),
            "es una referencia, no un token"
        );
        assert!(
            !n.referencia_de_credencial.contains("EAA"),
            "los tokens de Meta empiezan por EAA, y aquí no puede haber ninguno"
        );
    }

    /// Un número de otra empresa no se lee ni se puede confirmar.
    #[test]
    fn un_numero_de_otra_empresa_no_se_toca() {
        let (_d, db) = base();
        let id = alta(&db, ModoDelCanal::Seguimiento);
        let otra: CompanyId = "22222222-2222-4222-8222-222222222222".parse().expect("id");

        assert!(db.numero(otra, id).expect("lee").is_none());
        assert!(db.confirmar_coexistencia(otra, id, "x").is_err());
    }

    /// **Confirmar otra vez actualiza la fecha.**
    ///
    /// Lo que el preflight pregunta es si Coexistencia sigue activa **hoy**, y
    /// a eso contesta la última vez que alguien lo miró. El historial completo
    /// queda en la bitácora, que no se edita.
    #[test]
    fn confirmar_otra_vez_actualiza_la_fecha() {
        let (_d, db) = base();
        let id = alta(&db, ModoDelCanal::Seguimiento);

        db.confirmar_coexistencia(empresa(), id, "usuario_local")
            .expect("confirma");
        let primera = leer(&db, id).coexistencia_confirmada;

        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(
                "UPDATE whatsapp_account SET coexistencia_confirmada_at = '2020-01-01T00:00:00Z'",
            )
        })
        .expect("retrasa la marca");

        db.confirmar_coexistencia(empresa(), id, "usuario_local")
            .expect("vuelve a confirmar");

        // Se compara contra la vieja y no por igualdad con la primera: los
        // sellos llevan nanosegundos, así que dos confirmaciones seguidas
        // nunca son idénticas. Lo que se exige es que la marca ya no sea la
        // de 2020 y que no haya retrocedido.
        let segunda = leer(&db, id)
            .coexistencia_confirmada
            .unwrap_or_else(|| panic!("tiene que haber marca"));
        assert!(
            !segunda.starts_with("2020"),
            "la marca se quedó en la vieja: {segunda}"
        );
        assert!(
            segunda >= primera.unwrap_or_default(),
            "la marca no puede retroceder"
        );

        let cuantas: i64 = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT count(*) FROM audit_log WHERE action = 'confirmar_coexistencia'",
                    [],
                    |f| f.get(0),
                )
            })
            .expect("cuenta");
        assert_eq!(cuantas, 2, "y las dos veces quedan en la bitácora");
    }
}
