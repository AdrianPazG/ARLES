//! Escribir y leer la lista de supresión (entrega 3.4).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA TABLA YA EXISTÍA. LO QUE NO HABÍA ERA QUIEN ESCRIBIERA EN ELLA
//!
//! `suppression_entry` se creó en la V1 y se rehízo por canal en la V3. Hasta
//! esta entrega sólo se **leía**, en `contactos::direcciones_suprimidas`, para
//! avisar al editar un contacto. Es decir: ARLES sabía enseñar una lista de
//! supresión que nunca podía llenarse.
//!
//! Aquí está lo que la llena, lo que la vacía cuando se puede, y lo que la
//! consulta antes de importar.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LAS TRES REGLAS DE ESTE ARCHIVO
//!
//! **1 · Suprimir es idempotente.** Pedir dos veces lo mismo no es un error ni
//! escribe dos filas: la segunda no añade nada y la persona ya estaba
//! protegida. Un error aquí llevaría a la pantalla a decir «ya estaba» cuando
//! lo que el usuario quiere oír es «no recibe».
//!
//! **2 · Levantar una supresión no la borra en silencio.** Borra la fila —la
//! tabla es el estado actual, no un historial— pero sólo si el motivo lo
//! permite, y eso lo decide el núcleo, no esta capa.
//!
//! **3 · Una supresión sobrevive al contacto.** Borrar a alguien no le levanta
//! la baja. Está argumentado en `arles_core::supresion` y medido por la puerta
//! de salida de esta entrega.
//! ─────────────────────────────────────────────────────────────────────────

use arles_core::canal::Canal;
use arles_core::contacto::CanalValidado;
use arles_core::ids::{CompanyId, ContactId, SuppressionEntryId};
use arles_core::supresion::{
    AlcanceDeSupresion, MotivoDeSupresion, OrigenDeSupresion, PeticionDeSupresion,
};
use rusqlite::{OptionalExtension, params};

use crate::db::Db;
use crate::empresa::ahora;
use crate::error::DbError;

/// Cuántas filas de supresión devuelve una página.
///
/// La lista de supresión de una empresa que lleva años enviando puede tener
/// cientos de miles de filas: es la suma de todos los rebotes y todas las bajas
/// de su historia. Una función que devolviera «todas» acabaría llamándose con
/// medio millón detrás, y el problema saldría en el equipo del cliente.
pub const MAX_SUPRESIONES_POR_PAGINA: usize = 200;

/// Una fila de la lista, tal y como se enseña.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupresionGuardada {
    pub id: SuppressionEntryId,
    pub canal: Canal,
    /// La forma normalizada. Es lo que se compara, y lo que se enseña: la
    /// dirección en bruto no se guarda aquí porque una supresión no tiene
    /// «cómo la escribió el usuario» — puede venir de un servidor SMTP.
    pub direccion: String,
    pub alcance: AlcanceDeSupresion,
    pub motivo: MotivoDeSupresion,
    pub origen: OrigenDeSupresion,
    pub notas: String,
    pub creada: String,
    /// Une las filas que salieron de la misma petición global.
    pub peticion: Option<String>,
    /// Si hay un contacto vivo con esa dirección, su nombre. Para que la
    /// pantalla no enseñe una lista de cadenas sin cara.
    pub de_quien: Option<String>,
}

/// Qué pasó al pedir una supresión. Cifras, no adjetivos (§94).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResumenDeSupresion {
    /// Direcciones que quedaron bloqueadas por esta petición.
    pub nuevas: usize,
    /// Direcciones que **ya estaban** en la lista. No es un error.
    pub ya_estaban: usize,
}

impl Db {
    /// Mete unas direcciones en la lista de supresión.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// IDEMPOTENTE, Y ESO ES UNA DECISIÓN
    ///
    /// Suprimir una dirección que ya estaba suprimida **no es un error**.
    /// Devuelve cuántas eran nuevas y cuántas ya estaban, y la pantalla decide
    /// qué decir. Tratarlo como error obligaría a la interfaz a consultar antes
    /// de escribir —dos viajes por la frontera y una carrera entre ellos— para
    /// acabar en el mismo sitio.
    ///
    /// **No se actualiza la fila que ya existía.** Si alguien estaba suprimido
    /// por una baja que pidió, y ahora alguien lo suprime «a mano», la fila
    /// sigue diciendo que fue una baja. Sobrescribirla convertiría una
    /// supresión que no se puede levantar en una que sí.
    /// ─────────────────────────────────────────────────────────────────────
    ///
    /// Toda la petición va en **una transacción**: una supresión global a
    /// medias dejaría a alguien bloqueado por correo y abierto por WhatsApp,
    /// habiendo pedido las dos cosas.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn suprimir(
        &self,
        empresa: CompanyId,
        peticion: &PeticionDeSupresion,
    ) -> Result<ResumenDeSupresion, DbError> {
        // Un identificador común para todas las filas de esta petición. Es lo
        // único que después permite deshacerla entera sin adivinar cuáles de
        // las filas de esa persona eran suyas.
        let id_peticion = SuppressionEntryId::nuevo().to_string();
        let cuando = ahora();

        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            let mut nuevas = 0;
            let mut ya_estaban = 0;

            for canal in &peticion.canales {
                // `INSERT OR IGNORE` contra el índice único. `execute`
                // devuelve cuántas filas entraron: 0 significa que ya estaba.
                let filas = tx.execute(
                    "INSERT OR IGNORE INTO suppression_entry
                         (id, company_id, channel, address_normalized, scope,
                          request_id, reason, origin, notes, created_at, created_by)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, NULL)",
                    params![
                        SuppressionEntryId::nuevo().to_string(),
                        empresa.to_string(),
                        canal.canal.como_texto(),
                        canal.valor_normalizado,
                        peticion.alcance.como_texto(),
                        // Sólo las peticiones globales necesitan agruparse: una
                        // de un solo canal ya se identifica por su fila.
                        if peticion.alcance == AlcanceDeSupresion::Global {
                            Some(id_peticion.as_str())
                        } else {
                            None
                        },
                        peticion.motivo.como_texto(),
                        peticion.origen.como_texto(),
                        crate::contactos::vacio_a_nulo(&peticion.notas),
                        cuando,
                    ],
                )?;
                if filas == 0 {
                    ya_estaban += 1;
                } else {
                    nuevas += 1;
                }
            }

            tx.commit()?;
            Ok(ResumenDeSupresion { nuevas, ya_estaban })
        })
    }

    /// Saca una dirección de la lista, **si su motivo lo permite**.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// QUIÉN DECIDE SI SE PUEDE
    ///
    /// Lo decide el núcleo, con [`MotivoDeSupresion::se_puede_levantar`], y no
    /// esta capa ni la pantalla. La razón es la de siempre: si la regla viviera
    /// en la interfaz, cualquier otro camino hasta la base —un comando nuevo,
    /// una herramienta, la propia importación— la esquivaría sin enterarse.
    ///
    /// Aquí se **lee el motivo guardado** y se pregunta. No se confía en lo que
    /// venga de fuera: quien llama podría decir que era un rebote.
    /// ─────────────────────────────────────────────────────────────────────
    ///
    /// # Errores
    ///
    /// - [`DbError::SupresionNoExiste`] si no hay tal fila en esta empresa.
    /// - [`DbError::SupresionNoSeLevanta`] si la persona pidió la baja o marcó
    ///   el correo como no deseado.
    /// - [`DbError::DatoInvalido`] si la fila guardada trae un motivo que el
    ///   núcleo no reconoce.
    pub fn levantar_supresion(
        &self,
        empresa: CompanyId,
        id: SuppressionEntryId,
    ) -> Result<(), DbError> {
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;

            let motivo_guardado: Option<String> = tx
                .query_row(
                    "SELECT reason FROM suppression_entry
                      WHERE id = ?1 AND company_id = ?2",
                    params![id.to_string(), empresa.to_string()],
                    |f| f.get(0),
                )
                .optional()?;

            let Some(texto) = motivo_guardado else {
                return Err(DbError::SupresionNoExiste);
            };
            let motivo =
                MotivoDeSupresion::desde_texto(&texto).map_err(|_| DbError::DatoInvalido {
                    motivo: "el motivo de esa supresión no es uno de los admitidos",
                })?;

            if !motivo.se_puede_levantar() {
                return Err(DbError::SupresionNoSeLevanta {
                    motivo: motivo.clave_i18n(),
                });
            }

            tx.execute(
                "DELETE FROM suppression_entry WHERE id = ?1 AND company_id = ?2",
                params![id.to_string(), empresa.to_string()],
            )?;
            tx.commit()?;
            Ok(())
        })
    }

    /// Una página de la lista de supresión, de la más reciente a la más vieja.
    ///
    /// `desplazamiento` cuenta filas, no páginas. `cuantas` se recorta a
    /// [`MAX_SUPRESIONES_POR_PAGINA`] **en silencio y a propósito**: pedir más
    /// no es un error del usuario, es una llamada que hay que acotar.
    ///
    /// # Errores
    ///
    /// [`DbError::DatoInvalido`] si una fila trae un canal, motivo, origen o
    /// alcance que el núcleo no reconoce. **No se salta la fila**: una fila de
    /// supresión que no se puede leer es una persona de la que no sabemos si
    /// puede recibir, y esconderla sería peor que parar.
    pub fn listar_supresiones(
        &self,
        empresa: CompanyId,
        desplazamiento: usize,
        cuantas: usize,
    ) -> Result<Vec<SupresionGuardada>, DbError> {
        let limite = cuantas.min(MAX_SUPRESIONES_POR_PAGINA);
        self.con_dominio(|conn| {
            let mut consulta = conn.prepare(
                "SELECT s.id, s.channel, s.address_normalized, s.scope, s.reason,
                        s.origin, COALESCE(s.notes, ''), s.created_at, s.request_id,
                        (SELECT TRIM(COALESCE(c.first_name, '') || ' ' ||
                                     COALESCE(c.last_name, ''))
                           FROM contact_channel ch
                           JOIN contact c ON c.id = ch.contact_id
                          WHERE ch.company_id = s.company_id
                            AND ch.channel = s.channel
                            AND ch.value_normalized = s.address_normalized
                            AND ch.deleted_at IS NULL
                            AND c.deleted_at IS NULL
                          LIMIT 1)
                   FROM suppression_entry s
                  WHERE s.company_id = ?1
                  ORDER BY s.created_at DESC, s.id DESC
                  LIMIT ?2 OFFSET ?3",
            )?;

            let filas = consulta.query_map(
                params![empresa.to_string(), limite as i64, desplazamiento as i64],
                |f| {
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
                    ))
                },
            )?;

            let mut salida = Vec::new();
            for fila in filas {
                let (id, canal, direccion, alcance, motivo, origen, notas, creada, pet, quien) =
                    fila?;
                salida.push(SupresionGuardada {
                    id: id.parse().map_err(|_| DbError::DatoInvalido {
                        motivo: "una fila de supresión tiene un identificador inválido",
                    })?,
                    canal: Canal::desde_texto(&canal).map_err(|_| DbError::DatoInvalido {
                        motivo: "una fila de supresión tiene un canal desconocido",
                    })?,
                    direccion,
                    alcance: AlcanceDeSupresion::desde_texto(&alcance).map_err(|_| {
                        DbError::DatoInvalido {
                            motivo: "una fila de supresión tiene un alcance desconocido",
                        }
                    })?,
                    motivo: MotivoDeSupresion::desde_texto(&motivo).map_err(|_| {
                        DbError::DatoInvalido {
                            motivo: "una fila de supresión tiene un motivo desconocido",
                        }
                    })?,
                    origen: OrigenDeSupresion::desde_texto(&origen).map_err(|_| {
                        DbError::DatoInvalido {
                            motivo: "una fila de supresión tiene un origen desconocido",
                        }
                    })?,
                    notas,
                    creada,
                    peticion: pet,
                    de_quien: quien.filter(|n| !n.trim().is_empty()),
                });
            }
            Ok(salida)
        })
    }

    /// Cuántas filas tiene la lista. Para la paginación y para la cifra.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn contar_supresiones(&self, empresa: CompanyId) -> Result<usize, DbError> {
        self.con_dominio(|conn| {
            let n: i64 = conn.query_row(
                "SELECT COUNT(*) FROM suppression_entry WHERE company_id = ?1",
                params![empresa.to_string()],
                |f| f.get(0),
            )?;
            Ok(usize::try_from(n).unwrap_or(0))
        })
    }

    /// Todas las direcciones de un contacto vivo, para suprimirlo entero.
    ///
    /// Se leen **de la base y no de la pantalla**: la pantalla enseña lo que
    /// cargó cuando abrió la ficha, y entre ese momento y el clic puede haberse
    /// añadido un canal. Suprimir «todo» con una lista vieja dejaría el canal
    /// nuevo abierto, que es exactamente lo que la persona pidió cerrar.
    ///
    /// # Errores
    ///
    /// - [`DbError::ContactoNoExiste`] si no hay tal contacto vivo aquí.
    /// - [`DbError::DatoInvalido`] si un canal guardado no se reconoce.
    pub fn canales_para_suprimir(
        &self,
        empresa: CompanyId,
        contacto: ContactId,
    ) -> Result<Vec<CanalValidado>, DbError> {
        self.con_dominio(|conn| {
            let vivo: Option<i64> = conn
                .query_row(
                    "SELECT 1 FROM contact
                      WHERE id = ?1 AND company_id = ?2 AND deleted_at IS NULL",
                    params![contacto.to_string(), empresa.to_string()],
                    |f| f.get(0),
                )
                .optional()?;
            if vivo.is_none() {
                return Err(DbError::ContactoNoExiste);
            }

            // `principal` no es una columna: se deriva, igual que en
            // `contactos.rs` — el más antiguo de cada canal. Se repite la misma
            // derivación en vez de inventar un valor: para suprimir da igual
            // cuál sea el principal, pero un `CanalValidado` que mienta sobre
            // eso acabaría enseñándose en algún sitio.
            let mut consulta = conn.prepare(
                "SELECT channel, value_raw, value_normalized,
                        CASE WHEN id = (
                            SELECT p.id FROM contact_channel p
                             WHERE p.contact_id = contact_channel.contact_id
                               AND p.channel = contact_channel.channel
                               AND p.deleted_at IS NULL
                             ORDER BY p.created_at, p.id LIMIT 1
                        ) THEN 1 ELSE 0 END
                   FROM contact_channel
                  WHERE contact_id = ?1 AND company_id = ?2 AND deleted_at IS NULL
                  ORDER BY CASE channel WHEN 'email' THEN 0 ELSE 1 END,
                           created_at, id",
            )?;
            let filas =
                consulta.query_map(params![contacto.to_string(), empresa.to_string()], |f| {
                    Ok((
                        f.get::<_, String>(0)?,
                        f.get::<_, String>(1)?,
                        f.get::<_, String>(2)?,
                        f.get::<_, i64>(3)?,
                    ))
                })?;

            let mut salida = Vec::new();
            for fila in filas {
                let (canal, raw, normalizado, principal) = fila?;
                salida.push(CanalValidado {
                    canal: Canal::desde_texto(&canal).map_err(|_| DbError::DatoInvalido {
                        motivo: "un canal guardado no es uno de los admitidos",
                    })?,
                    valor_raw: raw,
                    valor_normalizado: normalizado,
                    principal: principal != 0,
                });
            }
            Ok(salida)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arles_core::canal::ValorDeCanal;
    use arles_core::contacto::{BorradorDeCanal, BorradorDeContacto, DatosDeContacto};

    use crate::conexion::ClaveMaestra;
    use crate::contactos::DatosDelLote;

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

    /// Un canal validado por el camino de verdad. Construirlo a mano dejaría
    /// pasar una forma normalizada que el validador nunca produce — y la
    /// supresión entera compara justo esa forma.
    fn cv(canal: Canal, valor: &str) -> CanalValidado {
        let v = ValorDeCanal::parse(canal, valor, "MX").expect("valor válido");
        CanalValidado {
            canal: v.canal(),
            valor_raw: v.raw().to_owned(),
            valor_normalizado: v.normalized().to_owned(),
            principal: true,
        }
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

    fn peticion(
        canales: Vec<CanalValidado>,
        alcance: AlcanceDeSupresion,
        motivo: MotivoDeSupresion,
    ) -> PeticionDeSupresion {
        PeticionDeSupresion::nueva(canales, alcance, motivo, OrigenDeSupresion::Usuario, "")
            .expect("petición válida")
    }

    #[test]
    fn lo_suprimido_se_lee_igual() {
        let (_d, db, empresa) = base();
        let p = peticion(
            vec![cv(Canal::Correo, "Ana@Empresa.MX")],
            AlcanceDeSupresion::Canal,
            MotivoDeSupresion::Baja,
        );
        let r = db.suprimir(empresa, &p).expect("suprime");
        assert_eq!(r.nuevas, 1);
        assert_eq!(r.ya_estaban, 0);

        let lista = db.listar_supresiones(empresa, 0, 50).expect("lista");
        assert_eq!(lista.len(), 1);
        let fila = lista.first().expect("hay una");
        assert_eq!(fila.canal, Canal::Correo);
        assert_eq!(fila.motivo, MotivoDeSupresion::Baja);
        assert_eq!(fila.alcance, AlcanceDeSupresion::Canal);
        // Se guarda normalizada, no como se escribió: es lo que se compara.
        assert_eq!(fila.direccion, "ana@empresa.mx");
    }

    /// Pedir dos veces lo mismo no es un error ni escribe dos filas. La
    /// pantalla necesita saber **cuántas** eran nuevas para poder decirlo.
    #[test]
    fn suprimir_dos_veces_no_duplica_ni_falla() {
        let (_d, db, empresa) = base();
        let p = peticion(
            vec![cv(Canal::Correo, "ana@empresa.mx")],
            AlcanceDeSupresion::Canal,
            MotivoDeSupresion::Manual,
        );

        assert_eq!(db.suprimir(empresa, &p).expect("1ª").nuevas, 1);
        let segunda = db.suprimir(empresa, &p).expect("2ª no falla");
        assert_eq!(segunda.nuevas, 0);
        assert_eq!(segunda.ya_estaban, 1);
        assert_eq!(db.contar_supresiones(empresa).expect("cuenta"), 1);
    }

    /// **La fila que ya existía no se pisa.** Si una baja que pidió la persona
    /// se sobrescribiera con una supresión «manual», pasaría a poder levantarse
    /// con un botón — que es justo lo que no debe pasar.
    #[test]
    fn suprimir_otra_vez_no_reescribe_el_motivo_original() {
        let (_d, db, empresa) = base();
        let direccion = cv(Canal::Correo, "ana@empresa.mx");

        db.suprimir(
            empresa,
            &peticion(
                vec![direccion.clone()],
                AlcanceDeSupresion::Canal,
                MotivoDeSupresion::Baja,
            ),
        )
        .expect("baja");
        db.suprimir(
            empresa,
            &peticion(
                vec![direccion],
                AlcanceDeSupresion::Canal,
                MotivoDeSupresion::Manual,
            ),
        )
        .expect("manual");

        let lista = db.listar_supresiones(empresa, 0, 50).expect("lista");
        assert_eq!(lista.len(), 1);
        assert_eq!(
            lista.first().expect("hay una").motivo,
            MotivoDeSupresion::Baja,
            "el motivo original se pisó: una baja pedida pasó a poder levantarse"
        );
    }

    /// Una petición global escribe una fila por canal, todas con el mismo
    /// identificador de petición: es lo único que permite deshacerla entera.
    #[test]
    fn una_peticion_global_escribe_una_fila_por_canal_y_las_agrupa() {
        let (_d, db, empresa) = base();
        let p = peticion(
            vec![
                cv(Canal::Correo, "ana@empresa.mx"),
                cv(Canal::WhatsApp, "8112345678"),
            ],
            AlcanceDeSupresion::Global,
            MotivoDeSupresion::Baja,
        );
        assert_eq!(db.suprimir(empresa, &p).expect("suprime").nuevas, 2);

        let lista = db.listar_supresiones(empresa, 0, 50).expect("lista");
        assert_eq!(lista.len(), 2);
        let peticiones: Vec<_> = lista.iter().map(|f| f.peticion.clone()).collect();
        assert!(
            peticiones.iter().all(Option::is_some),
            "una petición global sin identificador no se puede deshacer entera"
        );
        assert_eq!(
            peticiones.first(),
            peticiones.last(),
            "las dos filas de la misma petición no comparten identificador"
        );
    }

    /// Lo que dedujo una máquina se levanta; lo que pidió una persona, no.
    #[test]
    fn solo_se_levanta_lo_que_dedujo_una_maquina() {
        let (_d, db, empresa) = base();

        db.suprimir(
            empresa,
            &peticion(
                vec![cv(Canal::Correo, "rebote@empresa.mx")],
                AlcanceDeSupresion::Canal,
                MotivoDeSupresion::RebotePermanente,
            ),
        )
        .expect("rebote");
        db.suprimir(
            empresa,
            &peticion(
                vec![cv(Canal::Correo, "baja@empresa.mx")],
                AlcanceDeSupresion::Canal,
                MotivoDeSupresion::Baja,
            ),
        )
        .expect("baja");

        let lista = db.listar_supresiones(empresa, 0, 50).expect("lista");
        for fila in &lista {
            let r = db.levantar_supresion(empresa, fila.id);
            if fila.motivo == MotivoDeSupresion::RebotePermanente {
                assert!(r.is_ok(), "un rebote tendría que poder levantarse: {r:?}");
            } else {
                assert!(
                    matches!(r, Err(DbError::SupresionNoSeLevanta { .. })),
                    "una baja que pidió la persona se levantó con un botón: {r:?}"
                );
            }
        }
        assert_eq!(
            db.contar_supresiones(empresa).expect("cuenta"),
            1,
            "quedó otra cosa que la baja"
        );
    }

    /// El motivo se lee **de la base**, no de quien llama. Si se confiara en el
    /// que venga de fuera, bastaría con decir «era un rebote» para levantar una
    /// baja.
    #[test]
    fn levantar_una_supresion_que_no_existe_da_error_y_no_borra_nada() {
        let (_d, db, empresa) = base();
        db.suprimir(
            empresa,
            &peticion(
                vec![cv(Canal::Correo, "ana@empresa.mx")],
                AlcanceDeSupresion::Canal,
                MotivoDeSupresion::Manual,
            ),
        )
        .expect("suprime");

        let inventado = SuppressionEntryId::nuevo();
        assert!(matches!(
            db.levantar_supresion(empresa, inventado),
            Err(DbError::SupresionNoExiste)
        ));
        assert_eq!(db.contar_supresiones(empresa).expect("cuenta"), 1);
    }

    /// Los canales se leen de la base y no de la pantalla: entre que se abrió
    /// la ficha y se pulsó el botón puede haberse añadido un canal, y suprimir
    /// «todo» con una lista vieja dejaría ese canal abierto.
    #[test]
    fn los_canales_para_suprimir_salen_de_la_base_y_estan_todos() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(
                empresa,
                &contacto(&[
                    (Canal::Correo, "ana@empresa.mx"),
                    (Canal::WhatsApp, "81 1234 5678"),
                ]),
            )
            .expect("crea");

        let canales = db.canales_para_suprimir(empresa, id).expect("lee");
        assert_eq!(canales.len(), 2);
        let p = PeticionDeSupresion::nueva(
            canales,
            AlcanceDeSupresion::Global,
            MotivoDeSupresion::Baja,
            OrigenDeSupresion::Usuario,
            "lo pidió por teléfono",
        )
        .expect("petición");
        assert_eq!(db.suprimir(empresa, &p).expect("suprime").nuevas, 2);
    }

    #[test]
    fn un_contacto_borrado_no_da_canales_para_suprimir() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");
        db.borrar_contacto(empresa, id).expect("borra");

        assert!(matches!(
            db.canales_para_suprimir(empresa, id),
            Err(DbError::ContactoNoExiste)
        ));
    }

    /// Pedir más de lo admitido se recorta, no revienta.
    #[test]
    fn la_pagina_de_la_lista_se_recorta() {
        let (_d, db, empresa) = base();
        for i in 0..(MAX_SUPRESIONES_POR_PAGINA + 20) {
            db.suprimir(
                empresa,
                &peticion(
                    vec![cv(Canal::Correo, &format!("s{i}@empresa.mx"))],
                    AlcanceDeSupresion::Canal,
                    MotivoDeSupresion::Manual,
                ),
            )
            .expect("suprime");
        }
        let lista = db.listar_supresiones(empresa, 0, 100_000).expect("lista");
        assert_eq!(lista.len(), MAX_SUPRESIONES_POR_PAGINA);
        assert_eq!(
            db.contar_supresiones(empresa).expect("cuenta"),
            MAX_SUPRESIONES_POR_PAGINA + 20
        );
    }

    /// La lista enseña de quién es cada dirección, si hay alguien vivo con
    /// ella. Una lista de cadenas sin cara no se puede revisar.
    #[test]
    fn la_lista_dice_de_quien_es_la_direccion_cuando_se_sabe() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");
        db.suprimir(
            empresa,
            &peticion(
                vec![
                    cv(Canal::Correo, "ana@empresa.mx"),
                    cv(Canal::Correo, "nadie@empresa.mx"),
                ],
                AlcanceDeSupresion::Global,
                MotivoDeSupresion::Manual,
            ),
        )
        .expect("suprime");

        let lista = db.listar_supresiones(empresa, 0, 50).expect("lista");
        let de_ana = lista
            .iter()
            .find(|f| f.direccion == "ana@empresa.mx")
            .expect("está");
        let de_nadie = lista
            .iter()
            .find(|f| f.direccion == "nadie@empresa.mx")
            .expect("está");
        assert_eq!(de_ana.de_quien.as_deref(), Some("Ana Ruiz"));
        assert_eq!(de_nadie.de_quien, None);
    }

    // ── LA PUERTA DE SALIDA DE LA ENTREGA 3.4 ──────────────────────────────

    /// **Borrar un contacto, reimportarlo, y comprobar que sigue sin recibir.**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// ES LA PRUEBA QUE DEFINE LA ENTREGA
    ///
    /// El ROADMAP la escribe con esas palabras desde la Fase 0, y es la única
    /// forma de demostrar que la supresión hace lo que promete: sobrevivir.
    ///
    /// El recorrido completo, tal y como pasaría de verdad:
    ///
    ///   1. Ana existe y pide la baja.
    ///   2. Alguien borra su contacto —«ya no está en la lista».
    ///   3. Seis meses después se importa **la misma lista comprada**, donde
    ///      Ana sigue estando.
    ///   4. Ana vuelve a existir como contacto… y **sigue sin poder recibir**.
    ///
    /// El paso 4 es todo el asunto. Si la supresión se hubiera ido con el
    /// contacto, la baja se habría respetado durante seis meses y se habría
    /// deshecho sola, sin que nada fallara ni nadie se enterara.
    ///
    /// Se verificó rompiendo la garantía: haciendo que `borrar_contacto` borre
    /// también las supresiones de sus direcciones, esta prueba falla en el
    /// paso 4 y dice exactamente qué dirección quedó desprotegida.
    /// ─────────────────────────────────────────────────────────────────────
    #[test]
    fn una_baja_sobrevive_a_borrar_el_contacto_y_a_reimportarlo() {
        let (_d, db, empresa) = base();

        // 1 · Ana existe y pide la baja.
        let ana = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");
        let canales = db.canales_para_suprimir(empresa, ana).expect("canales");
        db.suprimir(
            empresa,
            &PeticionDeSupresion::nueva(
                canales,
                AlcanceDeSupresion::Global,
                MotivoDeSupresion::Baja,
                OrigenDeSupresion::Usuario,
                "pidió la baja por correo",
            )
            .expect("petición"),
        )
        .expect("suprime");

        // 2 · Alguien la borra.
        db.borrar_contacto(empresa, ana).expect("borra");
        assert_eq!(
            db.contar_supresiones(empresa).expect("cuenta"),
            1,
            "borrar el contacto se llevó su baja por delante"
        );

        // 3 · Seis meses después, la misma lista vuelve a importarse.
        let de_nuevo = contacto(&[(Canal::Correo, "ana@empresa.mx")]);
        db.importar_contactos(
            empresa,
            &DatosDelLote {
                nombre_del_archivo: "lista-de-la-feria.csv".into(),
                huella_del_archivo: "sha256:loquesea".into(),
                mapeo_en_json: "{}".into(),
                texto_del_consentimiento: "texto".into(),
                origen: arles_core::OrigenDeLaLista::EventoOFeria,
                total_de_filas: 1,
                choques: 0,
                invalidas: 0,
            },
            std::slice::from_ref(&de_nuevo),
        )
        .expect("importa");

        // 4 · Ana vuelve a existir… y sigue sin poder recibir.
        let suprimidas = db
            .direcciones_suprimidas(empresa, &de_nuevo.canales)
            .expect("consulta");
        assert_eq!(
            suprimidas,
            vec!["ana@empresa.mx".to_owned()],
            "Ana se reimportó y su baja ya no la protege: volvería a recibir correos"
        );
    }
}
