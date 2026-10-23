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

    /// ¿Esta dirección está en la lista de bajas, ahora mismo?
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ SE PREGUNTA EN EL INSTANTE DE ENVIAR Y NO AL ENCOLAR
    ///
    /// La audiencia de una campaña se congela al activarla, y entre ese momento
    /// y el último envío pueden pasar **semanas**: con cincuenta correos al día,
    /// una campaña de cinco mil dura cuatro meses (§11 del motor).
    ///
    /// Si la comprobación viviera sólo en el encolado, alguien que pide la baja
    /// el martes seguiría recibiendo correos de esa campaña hasta enero. Habría
    /// pedido no recibir, ARLES lo habría registrado, y le seguiría llegando.
    ///
    /// Por eso se pregunta otra vez **justo antes de cada envío**. Cuesta una
    /// consulta por correo, sobre un índice único, a un ritmo de cincuenta al
    /// día: el coste es irrelevante y lo que compra no lo es.
    ///
    /// La comparación es por **dirección normalizada**, no por contacto: una
    /// baja sobrevive a que borren y reimporten a esa persona (§39).
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn esta_suprimida(
        &self,
        empresa: CompanyId,
        canal: Canal,
        direccion_normalizada: &str,
    ) -> Result<bool, DbError> {
        self.con(|conn| {
            let hay: Option<i64> = conn
                .query_row(
                    "SELECT 1 FROM suppression_entry
                      WHERE company_id = ?1 AND channel = ?2 AND address_normalized = ?3",
                    params![
                        empresa.to_string(),
                        canal.como_texto(),
                        direccion_normalizada
                    ],
                    |f| f.get(0),
                )
                .optional()?;
            Ok(hay.is_some())
        })
    }

    /// Mete una dirección en la lista de bajas **sin pasar por el contacto**.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ HACE FALTA, HABIENDO YA `suprimir`
    ///
    /// [`Db::suprimir`] parte de un contacto: recoge sus canales validados y
    /// los suprime. Es lo correcto cuando alguien pulsa «dar de baja» en una
    /// ficha.
    ///
    /// El motor no tiene ficha. Cuando un servidor de correo contesta «esa
    /// dirección no existe», lo único que hay es **la dirección**: el contacto
    /// puede haberse borrado, puede haber cambiado, o el rebote puede venir de
    /// una dirección que nunca estuvo en la agenda. Obligar a encontrar un
    /// contacto para poder registrar el rebote dejaría el caso más importante
    /// —una dirección muerta a la que seguimos escribiendo— sin registrar.
    ///
    /// Y desde el 23/10/2026 **también la usa una persona**, no sólo el motor.
    /// Es el mismo caso visto desde el otro lado: alguien contesta «ya no me
    /// escriban» desde una dirección que no está en la lista —le reenviaron el
    /// correo, o escribe desde otra suya— y hasta hoy esa baja **no se podía
    /// apuntar de ninguna forma**, porque `suprimir_contacto` exige un
    /// `ContactId`. Lo cuenta `una_baja_no_exige_que_la_direccion_sea_un_contacto`.
    ///
    /// `notas` es lo que esa persona escribió, y en el camino del motor va
    /// `None`: ahí no hay nadie escribiendo nada, y rellenarlo con la respuesta
    /// del servidor sería meter texto de fuera en una columna que el registro
    /// enseña tal cual.
    ///
    /// Devuelve `true` si la fila es nueva y `false` si ya estaba. Las dos son
    /// éxito: suprimir es idempotente (regla 1 de este archivo).
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn suprimir_direccion(
        &self,
        empresa: CompanyId,
        canal: Canal,
        direccion_normalizada: &str,
        motivo: MotivoDeSupresion,
        origen: OrigenDeSupresion,
        notas: Option<&str>,
    ) -> Result<bool, DbError> {
        let cuando = ahora();
        self.con(|conn| {
            let filas = conn.execute(
                "INSERT OR IGNORE INTO suppression_entry
                     (id, company_id, channel, address_normalized, scope,
                      request_id, reason, origin, notes, created_at, created_by)
                 VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, ?9, NULL)",
                params![
                    SuppressionEntryId::nuevo().to_string(),
                    empresa.to_string(),
                    canal.como_texto(),
                    direccion_normalizada,
                    AlcanceDeSupresion::Canal.como_texto(),
                    motivo.como_texto(),
                    origen.como_texto(),
                    notas,
                    cuando,
                ],
            )?;
            Ok(filas == 1)
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

    /// Cuáles de esas direcciones pidieron no recibir, y por qué.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// ES LO QUE HACE HONESTO AL INFORME DE IMPORTACIÓN
    ///
    /// Dirección decidió el 21/09/2026 que un archivo con gente que ya pidió la
    /// baja **se enseñe antes de importar**. Esto es la consulta que lo
    /// alimenta: se pregunta de una vez por todas las direcciones del archivo,
    /// y el núcleo las clasifica sin tocar la base.
    ///
    /// Devuelve la **clave de i18n** del motivo, no el motivo: lo que cruza
    /// hacia la pantalla es texto traducible (§139), y así esta consulta sirve
    /// igual para el informe que para cualquier otra cosa que lo enseñe.
    /// ─────────────────────────────────────────────────────────────────────
    ///
    /// Se consulta **en lotes** por la misma razón que `duenos_de_direcciones`:
    /// SQLite tiene un tope de variables por sentencia —medido en este build:
    /// 32 766— y un archivo de 500 000 filas lo pasa siempre.
    ///
    /// # Errores
    ///
    /// - [`DbError::DatoInvalido`] si una fila guardada trae un motivo que el
    ///   núcleo no reconoce. **No se salta**: una supresión que no se puede
    ///   leer es una persona de la que no sabemos si puede recibir, y omitirla
    ///   del informe la dejaría entrar como si nada.
    /// - [`DbError::Sqlite`] si la consulta falla.
    pub fn motivos_de_supresion(
        &self,
        empresa: CompanyId,
        direcciones: &[String],
    ) -> Result<std::collections::HashMap<String, String>, DbError> {
        let mut encontradas = std::collections::HashMap::new();
        if direcciones.is_empty() {
            return Ok(encontradas);
        }

        self.con_dominio(|conn| {
            // Holgado por debajo del tope de variables de cualquier build
            // razonable, incluido uno compilado con el límite antiguo de 999.
            const POR_LOTE: usize = 400;

            for lote in direcciones.chunks(POR_LOTE) {
                let huecos = std::iter::repeat_n("?", lote.len())
                    .collect::<Vec<_>>()
                    .join(",");
                let sql = format!(
                    "SELECT address_normalized, reason
                       FROM suppression_entry
                      WHERE company_id = ?1
                        AND address_normalized IN ({huecos})"
                );

                let mut consulta = conn.prepare(&sql)?;
                let mut parametros: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(lote.len() + 1);
                let empresa_txt = empresa.to_string();
                parametros.push(&empresa_txt);
                for d in lote {
                    parametros.push(d);
                }

                let filas = consulta.query_map(parametros.as_slice(), |f| {
                    Ok((f.get::<_, String>(0)?, f.get::<_, String>(1)?))
                })?;
                for fila in filas {
                    let (direccion, motivo) = fila?;
                    let motivo = MotivoDeSupresion::desde_texto(&motivo).map_err(|_| {
                        DbError::DatoInvalido {
                            motivo: "una fila de supresión tiene un motivo desconocido",
                        }
                    })?;
                    encontradas.insert(direccion, motivo.clave_i18n().to_owned());
                }
            }
            Ok(encontradas)
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
    /// **Se verificó con dos cebos distintos**, porque esta prueba hace dos
    /// afirmaciones y una sola rotura sólo habría probado una:
    ///
    ///   1. Haciendo que `borrar_contacto` borre también las supresiones de sus
    ///      direcciones → falla en el **paso 2**, diciendo que la baja se perdió
    ///      al borrar.
    ///   2. Haciendo que `importar_contactos` «limpie» las supresiones de lo
    ///      que acaba de importar → sobrevive al paso 2 y falla en el **paso 4**:
    ///      «Ana se reimportó y su baja ya no la protege».
    ///
    /// El segundo es el que importa: es un fallo que **no se ve al borrar**.
    /// Todo parecería correcto hasta la siguiente importación, seis meses
    /// después, cuando la baja ya se deshizo sola.
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
                lista: None,
                direcciones: Vec::new(),
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
    // ── La consulta que alimenta el informe de importación ─────────────────

    /// Devuelve la **clave de i18n** del motivo, no el texto guardado. Lo que
    /// cruza hacia la pantalla es traducible (§139).
    #[test]
    fn los_motivos_salen_como_clave_de_i18n_y_solo_de_lo_suprimido() {
        let (_d, db, empresa) = base();
        db.suprimir(
            empresa,
            &peticion(
                vec![cv(Canal::Correo, "baja@empresa.mx")],
                AlcanceDeSupresion::Canal,
                MotivoDeSupresion::Baja,
            ),
        )
        .expect("suprime");

        let motivos = db
            .motivos_de_supresion(
                empresa,
                &["baja@empresa.mx".to_owned(), "libre@empresa.mx".to_owned()],
            )
            .expect("consulta");

        assert_eq!(
            motivos.len(),
            1,
            "se coló una dirección que no está suprimida"
        );
        assert_eq!(
            motivos.get("baja@empresa.mx").map(String::as_str),
            Some(MotivoDeSupresion::Baja.clave_i18n())
        );
    }

    /// La consulta va **en lotes**: SQLite tiene un tope de variables por
    /// sentencia, y un archivo de 500 000 filas lo pasa siempre.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// EL NÚMERO NO ES DECORATIVO, Y LA PRIMERA VERSIÓN LO ERA
    ///
    /// Esta prueba pedía **3 000** direcciones, «muy por encima del tope de
    /// 999». Al romper la guarda —subiendo `POR_LOTE` a 100 000, es decir
    /// quitando el troceado— **seguía pasando**: el tope real de variables de
    /// SQLite en este build es **32 766**, así que 3 000 caben de sobra en una
    /// sola sentencia y el troceado nunca se ejercitaba.
    ///
    /// Es exactamente el mismo engaño que ya se coló una vez en este proyecto,
    /// con la misma cifra de por medio. Ahora se piden 40 000: por encima del
    /// tope medido, de forma que sin troceo la consulta **no puede** funcionar.
    ///
    /// Verificado: con `POR_LOTE` a 100 000 esta prueba falla con «too many SQL
    /// variables».
    /// ─────────────────────────────────────────────────────────────────────
    #[test]
    fn la_consulta_de_motivos_trocea_en_lotes() {
        let (_d, db, empresa) = base();
        // Por encima del tope de variables medido en este build (32 766).
        const CUANTAS: usize = 40_000;

        let direcciones: Vec<String> = (0..CUANTAS).map(|i| format!("s{i}@empresa.mx")).collect();
        db.suprimir(
            empresa,
            &peticion(
                vec![cv(Canal::Correo, "s7@empresa.mx")],
                AlcanceDeSupresion::Canal,
                MotivoDeSupresion::Queja,
            ),
        )
        .expect("suprime");

        let motivos = db
            .motivos_de_supresion(empresa, &direcciones)
            .expect("la consulta se troceó");
        assert_eq!(motivos.len(), 1);
        assert_eq!(
            motivos.get("s7@empresa.mx").map(String::as_str),
            Some(MotivoDeSupresion::Queja.clave_i18n())
        );
    }

    /// Una lista vacía no da un viaje a la base ni un SQL con `IN ()`, que es
    /// sintaxis inválida.
    #[test]
    fn sin_direcciones_no_se_consulta_nada() {
        let (_d, db, empresa) = base();
        assert!(
            db.motivos_de_supresion(empresa, &[])
                .expect("no falla")
                .is_empty()
        );
    }

    // ─── La consulta del motor (entrega 4.3b) ───────────────────────────────

    /// **Quien pide la baja deja de recibir esa misma campaña.**
    ///
    /// Es la prueba de por qué el motor vuelve a preguntar antes de cada envío
    /// en vez de fiarse de la audiencia congelada: con cincuenta correos
    /// diarios, una campaña de cinco mil tarda meses, y en ese plazo la gente
    /// se da de baja.
    #[test]
    fn una_baja_de_hoy_se_ve_en_el_envio_de_manana() {
        let (_d, db, empresa) = base();
        assert!(
            !db.esta_suprimida(empresa, Canal::Correo, "ana@empresa.mx")
                .expect("consulta"),
            "antes de la baja, se le puede escribir"
        );

        db.suprimir_direccion(
            empresa,
            Canal::Correo,
            "ana@empresa.mx",
            MotivoDeSupresion::Baja,
            OrigenDeSupresion::Usuario,
            None,
        )
        .expect("suprime");

        assert!(
            db.esta_suprimida(empresa, Canal::Correo, "ana@empresa.mx")
                .expect("consulta"),
            "después de la baja, no"
        );
    }

    /// Una baja de correo no cierra el WhatsApp de la misma persona.
    ///
    /// Suena obvio y no lo es: la tabla guarda una fila por canal justamente
    /// para que «no me mandes correos» no se interprete como «no me mandes
    /// nada». Cuando alguien pide las dos cosas, se escriben las dos filas.
    #[test]
    fn la_baja_es_por_canal_y_no_se_contagia() {
        let (_d, db, empresa) = base();
        db.suprimir_direccion(
            empresa,
            Canal::Correo,
            "ana@empresa.mx",
            MotivoDeSupresion::Baja,
            OrigenDeSupresion::Usuario,
            None,
        )
        .expect("suprime");

        assert!(
            !db.esta_suprimida(empresa, Canal::WhatsApp, "ana@empresa.mx")
                .expect("consulta"),
            "la baja de correo no puede cerrar otro canal por su cuenta"
        );
    }

    /// La lista es de cada empresa. La baja de una no protege en la otra, y
    /// —más importante— tampoco bloquea a nadie ajeno.
    #[test]
    fn la_baja_no_cruza_de_una_empresa_a_otra() {
        let (_d, db, empresa) = base();
        // Se inserta a mano: `guardar_empresa` mantiene **una sola** empresa
        // por instalación —actualiza la que haya en vez de añadir otra—, así
        // que por el camino normal no se puede llegar a dos. El esquema sí las
        // admite, y la consulta filtra por empresa; esto comprueba el filtro.
        let otra = CompanyId::nuevo();
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES (?1, 'OTRA SA', 'MX', 'America/Mexico_City',
                         'hola@otra.mx', '2026-09-23T12:00:00Z', '2026-09-23T12:00:00Z')",
                params![otra.to_string()],
            )
        })
        .expect("siembra la segunda empresa");

        db.suprimir_direccion(
            empresa,
            Canal::Correo,
            "ana@empresa.mx",
            MotivoDeSupresion::Baja,
            OrigenDeSupresion::Usuario,
            None,
        )
        .expect("suprime");

        assert!(
            !db.esta_suprimida(otra, Canal::Correo, "ana@empresa.mx")
                .expect("consulta"),
            "la lista de bajas es de cada empresa"
        );
    }

    /// Suprimir la misma dirección dos veces no es un error ni escribe dos
    /// filas: es la regla 1 de este archivo, aplicada al camino del motor.
    #[test]
    fn suprimir_una_direccion_dos_veces_no_duplica_la_fila() {
        let (_d, db, empresa) = base();
        let primera = db
            .suprimir_direccion(
                empresa,
                Canal::Correo,
                "rebote@empresa.mx",
                MotivoDeSupresion::RebotePermanente,
                OrigenDeSupresion::RespuestaSmtp,
                None,
            )
            .expect("suprime");
        let segunda = db
            .suprimir_direccion(
                empresa,
                Canal::Correo,
                "rebote@empresa.mx",
                MotivoDeSupresion::RebotePermanente,
                OrigenDeSupresion::RespuestaSmtp,
                None,
            )
            .expect("suprime otra vez");

        assert!(primera, "la primera es nueva");
        assert!(!segunda, "la segunda ya estaba");
        assert_eq!(db.contar_supresiones(empresa).expect("cuenta"), 1);
    }

    /// **Una baja de una dirección que no es un contacto se apunta, con sus
    /// notas, y la lista la enseña.**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ ESTA PRUEBA, HABIENDO YA LAS DE ARRIBA
    ///
    /// Porque las de arriba comprueban el camino del **motor**, que escribe
    /// `notes = NULL`. El camino de una persona —`suprimir_direccion` desde la
    /// pantalla de bajas, desde el 23/10/2026— escribe lo que esa persona
    /// anotó, y eso es **lo único** que explica dentro de un año dónde pidió
    /// la baja alguien que no estaba en la lista.
    ///
    /// Se comprueba leyendo la fila **por la lista**, no por un `SELECT` de la
    /// prueba: lo que hay que poder afirmar no es «la columna se escribió», es
    /// «la nota se ve». Verificada escribiendo `None` en vez de las notas: sale
    /// en rojo con `left: "", right: "Contestó al correo del 23"`.
    #[test]
    fn una_baja_a_mano_guarda_sus_notas_y_no_necesita_contacto() {
        let (_d, db, empresa) = base();

        let nueva = db
            .suprimir_direccion(
                empresa,
                Canal::Correo,
                "alguien@reenviado.mx",
                MotivoDeSupresion::Baja,
                OrigenDeSupresion::Usuario,
                Some("Contestó al correo del 23"),
            )
            .expect("suprime");
        assert!(nueva, "la fila es nueva");

        let filas = db.listar_supresiones(empresa, 0, 50).expect("lista");
        let fila = filas
            .iter()
            .find(|f| f.direccion == "alguien@reenviado.mx")
            .expect("la baja está en la lista");

        assert_eq!(fila.notas, "Contestó al correo del 23");
        // Y sin contacto detrás: es justo el caso que no se podía apuntar.
        assert_eq!(fila.de_quien, None, "no hay contacto con esa dirección");
        // La pidió la persona, así que un botón no la deshace. Lo decide el
        // núcleo; aquí sólo se comprueba que llega decidido.
        assert!(!fila.motivo.se_puede_levantar());
    }
}
