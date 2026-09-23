//! La cola de envíos: tomar trabajo sin duplicar, y recuperar tras una caída.
//!
//! Ver `MOTOR_DE_EJECUCION.md` §2 y §3, y **ADR-0004**.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! ESTE MÓDULO EXISTE PARA QUE NADIE RECIBA UN CORREO DOS VECES
//!
//! Un duplicado es, de lejos, el peor fallo posible de ARLES:
//!
//!   · **Es invisible para nosotros.** Nada falla, ningún registro lo delata.
//!   · **Es irreversible para quien lo recibe.** Un correo que salió, salió.
//!   · **Es acumulativo para el dominio.** Los duplicados alimentan la tasa de
//!     queja, y la reputación de un dominio se pierde en un día y se recupera
//!     en meses.
//!
//! Un no-envío, en cambio, es visible y se arregla: el usuario lo ve y lo
//! reenvía. Esa asimetría gobierna todas las decisiones de este archivo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! TRES CAPAS, Y NINGUNA SOBRA
//!
//! **1 · El esquema.** `UNIQUE(campaign_id, channel, contact_address)` hace que
//! dos intentos para la misma persona en la misma campaña sean **imposibles**,
//! no improbables: la base los rechaza. Ningún fallo de lógica, ninguna carrera
//! y ningún doble clic pueden saltárselo.
//!
//! La clave es la **dirección**, no el `contact_id`. Un contacto borrado y
//! vuelto a importar tiene id nuevo, así que una clave por id permitiría
//! enviarle otra vez — el mismo motivo por el que la lista de bajas se indexa
//! por dirección (§39).
//!
//! **2 · El compare-and-swap de este archivo.** Impide que dos trabajadores
//! tomen el mismo intento. Está explicado donde ocurre.
//!
//! **3 · La clave de idempotencia**, que viaja como `Message-Id` del correo.
//! Se persiste **antes** de llamar al proveedor.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! Y UNA VENTANA QUE NINGUNA DE LAS TRES CIERRA
//!
//! ```text
//! 1. UPDATE state = 'sending'   <- queda escrito
//! 2. proveedor.enviar(...)      <- el proveedor ACEPTA
//! 3. X se va la luz aqui
//! 4. UPDATE state = 'sent'      <- nunca ocurre
//! ```
//!
//! Al reabrir, la fila dice `sending`. **El correo pudo salir o no, y no hay
//! forma de saberlo**: averiguarlo exigiría leer la bandeja de enviados, un
//! permiso que v1.2.0 no pide y no quiere pedir (§69).
//!
//! La decisión, de ADR-0004: esas filas pasan a `presumed_sent` y **nunca se
//! reenvían solas**. El usuario las ve con el nombre y la hora, y decide.

use arles_core::attempt::AttemptState;
use rusqlite::{Connection, OptionalExtension, params};

use crate::db::Db;
use crate::error::DbError;

/// Un intento tomado por este trabajador, listo para enviarse.
///
/// Que exista este valor significa que el compare-and-swap **ganó**: la fila
/// está en `claimed` y es de quien lo tiene. Ningún otro trabajador la va a
/// tocar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrabajoTomado {
    pub id: String,
    pub campana: String,
    pub canal: String,
    /// La dirección, ya normalizada cuando se encoló.
    pub direccion: String,
    /// El `Message-Id` que llevará el correo. Se generó al encolar, **antes**
    /// de que nadie llamara al proveedor.
    pub clave_de_idempotencia: String,
    /// Cuántas veces se reintentó ya. Alimenta `arles_core::reintento`.
    pub reintentos_ya_hechos: u8,
}

/// Lo que se encontró al arrancar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Recuperacion {
    /// Filas que quedaron a medio enviar y pasan a «envío no confirmado».
    pub envios_sin_confirmar: u32,
    /// Filas que alguien tomó y nunca llegó a enviar: vuelven a la cola.
    ///
    /// Éstas **sí** son seguras de reenviar: nadie llamó al proveedor, así que
    /// no hay ningún correo que pueda estar duplicado.
    pub devueltos_a_la_cola: u32,
}

impl Db {
    /// Toma el siguiente intento pendiente de una campaña.
    ///
    /// Devuelve `None` si no hay nada que hacer ahora mismo.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// EL COMPARE-AND-SWAP, Y POR QUÉ NO SE LEE Y LUEGO SE ESCRIBE
    ///
    /// Lo natural sería: leer el siguiente pendiente, y después marcarlo como
    /// tomado. Entre esas dos operaciones hay un hueco, y en ese hueco otro
    /// trabajador puede leer **el mismo**. Los dos lo marcan, los dos lo
    /// envían, y esa persona recibe el correo dos veces.
    ///
    /// Aquí la comprobación y la escritura son **la misma operación**:
    ///
    /// ```text
    /// UPDATE ... SET state='claimed' WHERE id=? AND state IN ('queued','failed')
    /// ```
    ///
    /// y se mira **cuántas filas cambiaron**. Si cambiaron cero, otro ganó la
    /// carrera y su `state` ya no es el que exigía el `WHERE`. Entonces este
    /// trabajador no insiste: pasa al siguiente candidato.
    ///
    /// La condición del `WHERE` no es decorativa: es el seguro. Quitarla
    /// dejaría un `UPDATE` que siempre gana, y con él el duplicado.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ HAY UN BUCLE, Y POR QUÉ ESTÁ ACOTADO
    ///
    /// Perder una carrera no significa que no haya trabajo: significa que ese
    /// intento ya no es nuestro. Así que se prueba con el siguiente.
    ///
    /// El tope existe porque un bucle sin tope aquí gira contra la base de
    /// datos mientras otro trabajador vaya ganando, y eso es peor que no tomar
    /// nada: consume la base para todos. Al agotarse se devuelve `None`, que
    /// significa «ahora mismo no hay nada para mí» — el trabajador espera y
    /// vuelve, que es exactamente lo correcto.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn tomar_siguiente(
        &self,
        campana: &str,
        ahora: &str,
    ) -> Result<Option<TrabajoTomado>, DbError> {
        /// Cuántas carreras perdidas se toleran antes de rendirse por esta vez.
        const INTENTOS_DE_TOMA: u8 = 8;

        self.con(|conn| {
            for _ in 0..INTENTOS_DE_TOMA {
                let Some(candidato) = siguiente_candidato(conn, campana, ahora)? else {
                    return Ok(None);
                };

                if intentar_tomar(conn, &candidato.id, ahora)? {
                    return Ok(Some(candidato));
                }
                // No se pudo: otro trabajador ganó. Al siguiente.
            }
            Ok(None)
        })
    }

    /// Pone el intento en `sending`, **antes** de llamar al proveedor.
    ///
    /// El orden es lo único que hace posible reconocer después un envío
    /// ambiguo. Si se marcara después de enviar, una caída entre el envío y la
    /// marca dejaría la fila en `claimed` — indistinguible de «nadie la tocó»—
    /// y al reabrir volvería a la cola y saldría **otra vez**.
    ///
    /// Escrito antes, una fila en `sending` dice exactamente lo que ocurrió:
    /// alguien estaba llamando al proveedor cuando esto se cayó.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    /// [`DbError::DatoInvalido`] si la fila ya no estaba en `claimed`.
    pub fn marcar_enviando(&self, id: &str, ahora: &str) -> Result<(), DbError> {
        self.con_dominio(|conn| {
            let filas = conn.execute(
                "UPDATE message_attempt
                    SET state = ?1, attempt_count = attempt_count + 1, updated_at = ?2
                  WHERE id = ?3 AND state = ?4",
                params![
                    AttemptState::Sending.as_str(),
                    ahora,
                    id,
                    AttemptState::Claimed.as_str(),
                ],
            )?;
            if filas == 1 {
                Ok(())
            } else {
                // Guardado por el mismo motivo que el CAS: si la fila ya no
                // estaba tomada, algo se saltó el orden y no se puede enviar
                // asumiendo lo contrario.
                Err(DbError::DatoInvalido {
                    motivo: "el intento no estaba tomado por nadie",
                })
            }
        })
    }

    /// El proveedor aceptó el mensaje. **Aceptado no es entregado** (§65).
    ///
    /// `id_del_proveedor` y `respuesta` vienen de fuera y llegan ya recortados
    /// por `arles_motor::recortar_respuesta`: son texto de un servidor ajeno.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    /// [`DbError::DatoInvalido`] si la fila no estaba en `sending`.
    pub fn marcar_enviado(
        &self,
        id: &str,
        id_del_proveedor: Option<&str>,
        respuesta: Option<&str>,
        ahora: &str,
    ) -> Result<(), DbError> {
        self.con_dominio(|conn| {
            let filas = conn.execute(
                "UPDATE message_attempt
                    SET state = ?1, sent_at = ?2, updated_at = ?2,
                        provider_message_id = ?3, provider_response = ?4,
                        error_kind = NULL, error_detail = NULL, next_retry_at = NULL
                  WHERE id = ?5 AND state = ?6",
                params![
                    AttemptState::Sent.as_str(),
                    ahora,
                    id_del_proveedor,
                    respuesta,
                    id,
                    AttemptState::Sending.as_str(),
                ],
            )?;
            exigir_una(filas, "el intento no estaba enviándose")
        })
    }

    /// Falló, pero se va a reintentar. `reintento_en` es **cuándo**.
    ///
    /// La hora del reintento se escribe **en la misma sentencia** que el
    /// estado. Separarlas dejaría, si el proceso muriera en medio, una fila en
    /// `failed` sin hora — y `siguiente_candidato` trata `next_retry_at IS NULL`
    /// como «ya toca», así que el reintento saldría de inmediato en vez de
    /// dentro de cinco minutos. El fallo se convertiría en una ráfaga contra un
    /// servidor que acaba de decir que no puede.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    /// [`DbError::DatoInvalido`] si la fila no estaba en `sending`.
    pub fn marcar_fallido(
        &self,
        id: &str,
        clave_del_error: &str,
        detalle: Option<&str>,
        reintento_en: &str,
        ahora: &str,
    ) -> Result<(), DbError> {
        self.con_dominio(|conn| {
            let filas = conn.execute(
                "UPDATE message_attempt
                    SET state = ?1, updated_at = ?2, next_retry_at = ?3,
                        error_kind = ?4, error_detail = ?5
                  WHERE id = ?6 AND state = ?7",
                params![
                    AttemptState::Failed.as_str(),
                    ahora,
                    reintento_en,
                    clave_del_error,
                    detalle,
                    id,
                    AttemptState::Sending.as_str(),
                ],
            )?;
            exigir_una(filas, "el intento no estaba enviándose")
        })
    }

    /// Se acabó: no se reintenta más.
    ///
    /// `next_retry_at` se borra a propósito. Si quedara escrito, una fila
    /// definitivamente fallida llevaría una hora futura que ningún código va a
    /// mirar —`siguiente_candidato` no selecciona `permanently_failed`— y que
    /// cualquiera que lea la tabla interpretaría como «va a volver a salir».
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    /// [`DbError::DatoInvalido`] si la fila no estaba en `sending`.
    pub fn marcar_rendido(
        &self,
        id: &str,
        clave_del_error: &str,
        detalle: Option<&str>,
        ahora: &str,
    ) -> Result<(), DbError> {
        self.con_dominio(|conn| {
            let filas = conn.execute(
                "UPDATE message_attempt
                    SET state = ?1, updated_at = ?2, next_retry_at = NULL,
                        error_kind = ?3, error_detail = ?4
                  WHERE id = ?5 AND state = ?6",
                params![
                    AttemptState::PermanentlyFailed.as_str(),
                    ahora,
                    clave_del_error,
                    detalle,
                    id,
                    AttemptState::Sending.as_str(),
                ],
            )?;
            exigir_una(filas, "el intento no estaba enviándose")
        })
    }

    /// Esa persona está en la lista de bajas: no se le escribe.
    ///
    /// **No es un fallo**, y por eso no lleva `error_kind`: es el sistema
    /// funcionando.
    ///
    /// Sólo se acepta desde `claimed`: **antes** de llamar a nadie. Un intento
    /// que ya llegó a `sending` pudo salir, y `suppressed` no cuenta como envío
    /// consumido — dejarlo ahí escondería un correo posible del presupuesto.
    /// Ese caso acaba en `permanently_failed`; está argumentado en
    /// `arles_core::attempt::puede_ir_a`.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    /// [`DbError::DatoInvalido`] si la fila no estaba tomada.
    pub fn marcar_suprimido(&self, id: &str, ahora: &str) -> Result<(), DbError> {
        self.cerrar_desde_tomado(id, AttemptState::Suppressed, ahora)
    }

    /// El problema no es del envío: es de la cuenta. El intento espera.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ ESTO NO ES UN `marcar_fallido` CON OTRO TEXTO
    ///
    /// Cuando una credencial caduca o se revoca, **ningún** envío de esa cuenta
    /// va a funcionar hasta que una persona la arregle. Tratarlo como un fallo
    /// corriente tendría dos consecuencias, las dos malas:
    ///
    /// **Gastaría reintentos que no se han usado.** `attempt_count` subió al
    /// empezar a enviar, pero ese intento no llegó a pasar por el servidor de
    /// correo. Si se dejara subido, tres campañas paradas por una credencial
    /// caducada dejarían a media audiencia sin reintentos, y al arreglar la
    /// cuenta esos envíos se rendirían enseguida sin haberlo intentado nunca de
    /// verdad. Por eso se **devuelve** el intento: `attempt_count - 1`.
    ///
    /// **Y le pondría una hora futura.** Esperar media hora no arregla una
    /// credencial: eso lo arregla una persona. La hora de reintento es
    /// **ahora**, para que el envío salga en cuanto la cuenta vuelva a servir.
    /// Lo que impide que gire mientras tanto no es la hora: es que el motor se
    /// detiene con `Decision::PararLaCuenta` y avisa.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    /// [`DbError::DatoInvalido`] si la fila no estaba en `sending`.
    pub fn marcar_en_espera_de_la_cuenta(
        &self,
        id: &str,
        clave_del_error: &str,
        detalle: Option<&str>,
        ahora: &str,
    ) -> Result<(), DbError> {
        self.con_dominio(|conn| {
            let filas = conn.execute(
                "UPDATE message_attempt
                    SET state = ?1, updated_at = ?2, next_retry_at = ?2,
                        attempt_count = MAX(attempt_count - 1, 0),
                        error_kind = ?3, error_detail = ?4
                  WHERE id = ?5 AND state = ?6",
                params![
                    AttemptState::Failed.as_str(),
                    ahora,
                    clave_del_error,
                    detalle,
                    id,
                    AttemptState::Sending.as_str(),
                ],
            )?;
            exigir_una(filas, "el intento no estaba enviándose")
        })
    }

    /// El usuario paró. Este intento no sale.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    /// [`DbError::DatoInvalido`] si la fila no estaba tomada.
    pub fn marcar_cancelado(&self, id: &str, ahora: &str) -> Result<(), DbError> {
        self.cerrar_desde_tomado(id, AttemptState::Cancelled, ahora)
    }

    /// Cierra un intento tomado sin haber llamado al proveedor.
    fn cerrar_desde_tomado(
        &self,
        id: &str,
        destino: AttemptState,
        ahora: &str,
    ) -> Result<(), DbError> {
        self.con_dominio(|conn| {
            let filas = conn.execute(
                "UPDATE message_attempt
                    SET state = ?1, updated_at = ?2, next_retry_at = NULL
                  WHERE id = ?3 AND state = ?4",
                params![destino.as_str(), ahora, id, AttemptState::Claimed.as_str(),],
            )?;
            exigir_una(filas, "el intento no estaba tomado por nadie")
        })
    }

    /// Ordena la cola al arrancar. **Se llama antes de tomar ningún trabajo.**
    ///
    /// Una caída deja dos clases de filas a medias, y **no se tratan igual**:
    ///
    /// | Quedó en | Qué se sabe | Qué se hace |
    /// |---|---|---|
    /// | `claimed` | Nadie llamó al proveedor | Vuelve a la cola. Reenviar es seguro |
    /// | `sending` | **Se llamó, y no se sabe si aceptó** | `presumed_sent`. No se reenvía |
    ///
    /// La segunda fila es ADR-0004, y conviene ver por qué la decisión no es
    /// simétrica:
    ///
    /// | Si eligiéramos… | Y el correo sí salió | Y no salió |
    /// |---|---|---|
    /// | **Reenviar** | **Duplicado.** Irreversible, invisible, quema dominio | Correcto |
    /// | **No reenviar** | Correcto | Alguien no recibió. **Visible y recuperable** |
    ///
    /// Se falla del lado recuperable.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn recuperar_al_arrancar(&self, ahora: &str) -> Result<Recuperacion, DbError> {
        self.con(|conn| {
            let tx = conn.unchecked_transaction()?;

            // Lo ambiguo primero. Si el proceso muriera entre las dos
            // sentencias, lo peor que puede pasar es que los `claimed` sigan
            // en `claimed` y se ordenen al siguiente arranque. Al revés, un
            // `sending` sin marcar podría llegar a un momento en que alguien
            // lo interpretara como tomado y lo reenviara.
            let envios_sin_confirmar = tx.execute(
                "UPDATE message_attempt
                    SET state = ?1, updated_at = ?2
                  WHERE state = ?3",
                params![
                    AttemptState::PresumedSent.as_str(),
                    ahora,
                    AttemptState::Sending.as_str(),
                ],
            )?;

            let devueltos_a_la_cola = tx.execute(
                "UPDATE message_attempt
                    SET state = ?1, claimed_at = NULL, updated_at = ?2
                  WHERE state = ?3",
                params![
                    AttemptState::Queued.as_str(),
                    ahora,
                    AttemptState::Claimed.as_str(),
                ],
            )?;

            tx.commit()?;
            Ok(Recuperacion {
                envios_sin_confirmar: u32::try_from(envios_sin_confirmar).unwrap_or(u32::MAX),
                devueltos_a_la_cola: u32::try_from(devueltos_a_la_cola).unwrap_or(u32::MAX),
            })
        })
    }
}

/// Los estados desde los que un intento se puede tomar.
///
/// Es una constante y no dos literales sueltos porque **hay tres sitios que
/// tienen que decir lo mismo**: el `SELECT` de candidatos, el `WHERE` del
/// compare-and-swap, y la máquina de estados de `arles-core`. Que se separen es
/// un fallo que no se ve: la cola seguiría funcionando y la máquina de estados
/// estaría describiendo otro programa.
const TOMABLES: [AttemptState; 2] = [AttemptState::Queued, AttemptState::Failed];

/// Un `UPDATE` con compare-and-swap cambió exactamente una fila, o no procedía.
///
/// Cero filas nunca es «no pasa nada»: significa que la fila no estaba en el
/// estado que esta operación exigía. Seguir adelante sería escribir el
/// desenlace de un envío sobre una fila que otro está manejando.
fn exigir_una(filas: usize, motivo: &'static str) -> Result<(), DbError> {
    if filas == 1 {
        Ok(())
    } else {
        Err(DbError::DatoInvalido { motivo })
    }
}

/// El compare-and-swap: marca el intento como tomado **sólo si sigue libre**.
///
/// Devuelve `true` si esta llamada se lo llevó, `false` si otro trabajador se
/// adelantó.
///
/// ─────────────────────────────────────────────────────────────────────────
/// VIVE EN SU PROPIA FUNCIÓN PARA PODER PROBARSE
///
/// Estaba escrito dentro de `tomar_siguiente`, y su prueba —«dos trabajadores
/// no pueden llevarse el mismo intento»— **pasaba con el seguro quitado**.
///
/// El motivo, descubierto poniéndole el cebo el 23/09/2026: aquella prueba
/// dejaba la fila en `claimed` y llamaba a `tomar_siguiente`, pero la consulta
/// de candidatos ya filtra por `state IN ('queued','failed')`, así que devolvía
/// `None` **sin llegar nunca al UPDATE**. La prueba comprobaba el filtro del
/// `SELECT` creyendo que comprobaba el `WHERE` del `UPDATE`.
///
/// El `WHERE` sólo entra en juego en el instante exacto de la carrera: entre
/// que se lee el candidato y se marca. Ese instante no se puede provocar desde
/// fuera sin lanzar hilos y confiar en el azar — y un test que «casi nunca»
/// falla es peor que ninguno, porque da confianza sin haber probado nada.
///
/// Sacándolo aquí, la carrera se prueba de frente: se le da una fila que ya
/// está tomada y tiene que decir que no.
fn intentar_tomar(conn: &Connection, id: &str, ahora: &str) -> Result<bool, rusqlite::Error> {
    // La comprobación y la escritura son UNA operación. Leer y luego escribir
    // deja un hueco, y en ese hueco otro trabajador puede llevarse la misma
    // fila: los dos enviarían, y esa persona recibiría el correo dos veces.
    //
    // El `AND state IN (...)` es el seguro entero. Sin él, este UPDATE gana
    // siempre y el duplicado está servido.
    let [desde_a, desde_b] = TOMABLES;
    let filas = conn.execute(
        "UPDATE message_attempt
            SET state = ?1, claimed_at = ?2, updated_at = ?2
          WHERE id = ?3
            AND state IN (?4, ?5)",
        params![
            AttemptState::Claimed.as_str(),
            ahora,
            id,
            desde_a.as_str(),
            desde_b.as_str(),
        ],
    )?;
    Ok(filas == 1)
}

/// El siguiente intento que **parece** disponible.
///
/// «Parece» es literal: entre esta lectura y el `UPDATE` que la sigue, otro
/// trabajador puede tomarlo. Por eso el `UPDATE` vuelve a comprobar el estado
/// en vez de fiarse de lo que se leyó aquí.
///
/// El orden es por antigüedad, que es lo que el usuario espera de una cola.
fn siguiente_candidato(
    conn: &Connection,
    campana: &str,
    ahora: &str,
) -> Result<Option<TrabajoTomado>, rusqlite::Error> {
    conn.query_row(
        "SELECT id, campaign_id, channel, contact_address, idempotency_key, attempt_count
           FROM message_attempt
          WHERE campaign_id = ?1
            AND state IN (?2, ?3)
            AND (scheduled_for IS NULL OR scheduled_for <= ?4)
            AND (next_retry_at  IS NULL OR next_retry_at  <= ?4)
          ORDER BY created_at, id
          LIMIT 1",
        params![
            campana,
            AttemptState::Queued.as_str(),
            AttemptState::Failed.as_str(),
            ahora,
        ],
        |f| {
            Ok(TrabajoTomado {
                id: f.get(0)?,
                campana: f.get(1)?,
                canal: f.get(2)?,
                direccion: f.get(3)?,
                clave_de_idempotencia: f.get(4)?,
                reintentos_ya_hechos: u8::try_from(f.get::<_, i64>(5)?).unwrap_or(u8::MAX),
            })
        },
    )
    .optional()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conexion::ClaveMaestra;

    const AHORA: &str = "2026-09-23T12:00:00Z";

    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('emp', 'TELEMETRY', 'MX', 'America/Mexico_City',
                         'hola@t.mx', ?1, ?1)",
                params![AHORA],
            )?;
            c.execute(
                "INSERT INTO campaign (id, company_id, name, created_at, updated_at)
                 VALUES ('camp', 'emp', 'Prueba', ?1, ?1)",
                params![AHORA],
            )
        })
        .expect("siembra");
        (dir, db)
    }

    /// Encola un intento con el estado y la antigüedad que se le pidan.
    fn encolar(db: &Db, id: &str, direccion: &str, estado: AttemptState, creado: &str) {
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO message_attempt
                     (id, campaign_id, channel, contact_address, idempotency_key,
                      state, attempt_count, created_at, updated_at)
                 VALUES (?1, 'camp', 'email', ?2, ?3, ?4, 0, ?5, ?5)",
                params![
                    id,
                    direccion,
                    format!("clave-{id}"),
                    estado.as_str(),
                    creado
                ],
            )
        })
        .expect("encola");
    }

    fn estado_de(db: &Db, id: &str) -> String {
        db.ejecutar_en_pruebas(|c| {
            c.query_row(
                "SELECT state FROM message_attempt WHERE id = ?1",
                params![id],
                |f| f.get(0),
            )
        })
        .expect("lee estado")
    }

    #[test]
    fn se_toma_el_mas_antiguo_primero() {
        let (_d, db) = base();
        encolar(
            &db,
            "b",
            "b@x.mx",
            AttemptState::Queued,
            "2026-09-23T11:00:00Z",
        );
        encolar(
            &db,
            "a",
            "a@x.mx",
            AttemptState::Queued,
            "2026-09-23T10:00:00Z",
        );

        let t = db
            .tomar_siguiente("camp", AHORA)
            .expect("toma")
            .expect("hay");
        assert_eq!(t.id, "a", "una cola sirve por antigüedad, no por id");
        assert_eq!(t.direccion, "a@x.mx");
        assert_eq!(t.clave_de_idempotencia, "clave-a");
        assert_eq!(estado_de(&db, "a"), "claimed");
        assert_eq!(estado_de(&db, "b"), "queued", "el otro no se toca");
    }

    /// Sin trabajo, se dice que no hay: no se inventa nada.
    #[test]
    fn sin_nada_pendiente_no_se_toma_nada() {
        let (_d, db) = base();
        assert_eq!(db.tomar_siguiente("camp", AHORA).expect("toma"), None);
    }

    /// Un intento ya tomado no se vuelve a tomar.
    #[test]
    fn lo_ya_tomado_no_se_toma_otra_vez() {
        let (_d, db) = base();
        encolar(&db, "a", "a@x.mx", AttemptState::Queued, AHORA);

        assert!(db.tomar_siguiente("camp", AHORA).expect("toma").is_some());
        assert_eq!(
            db.tomar_siguiente("camp", AHORA).expect("toma"),
            None,
            "el mismo intento no se puede tomar dos veces"
        );
    }

    // ── EL COMPARE-AND-SWAP · la prueba que sostiene el módulo ───────────

    /// **EL SEGURO CONTRA EL DUPLICADO, PROBADO DE FRENTE.**
    ///
    /// Se llama al compare-and-swap directamente, con la fila en cada estado
    /// posible. Es la única forma de probar la carrera sin lanzar hilos y
    /// confiar en el azar.
    ///
    /// SE HA VISTO FALLAR, Y ESTA PRUEBA NACIÓ DE AHÍ. La versión anterior
    /// dejaba la fila en `claimed` y llamaba a `tomar_siguiente`, y **pasaba
    /// con el seguro quitado**: la consulta de candidatos ya filtra por estado,
    /// así que devolvía `None` sin llegar nunca al `UPDATE`. Comprobaba el
    /// `SELECT` creyendo que comprobaba el `UPDATE`.
    ///
    /// Con el seguro quitado, esta prueba sí falla: el `UPDATE` se lleva una
    /// fila que ya era de otro trabajador, y los dos enviarían a la misma
    /// persona.
    #[test]
    fn el_seguro_solo_deja_tomar_lo_que_sigue_libre() {
        let (_d, db) = base();

        for (estado, deberia_poder) in [
            (AttemptState::Queued, true),
            (AttemptState::Failed, true),
            (AttemptState::Claimed, false),
            (AttemptState::Sending, false),
            (AttemptState::Sent, false),
            (AttemptState::PresumedSent, false),
            (AttemptState::Suppressed, false),
            (AttemptState::Cancelled, false),
            (AttemptState::PermanentlyFailed, false),
        ] {
            let id = format!("i-{}", estado.as_str());
            encolar(&db, &id, &format!("{id}@x.mx"), estado, AHORA);

            let pudo = db
                .ejecutar_en_pruebas(|c| intentar_tomar(c, &id, AHORA))
                .expect("cas");

            assert_eq!(
                pudo,
                deberia_poder,
                "con la fila en «{}» el seguro debería {}. Dejar tomar algo que \
                 ya está tomado o enviado es exactamente el duplicado que este \
                 módulo existe para impedir",
                estado.as_str(),
                if deberia_poder { "dejar" } else { "negarse" }
            );
        }
    }

    /// Y el mismo seguro, visto desde fuera: dos tomas seguidas no se pisan.
    ///
    /// Esta prueba **no** comprueba la carrera —para eso está la de arriba—:
    /// comprueba que el recorrido completo no entrega dos veces el mismo
    /// trabajo.
    #[test]
    fn dos_tomas_seguidas_no_entregan_el_mismo_intento() {
        let (_d, db) = base();
        encolar(&db, "a", "a@x.mx", AttemptState::Queued, AHORA);
        encolar(&db, "b", "b@x.mx", AttemptState::Queued, AHORA);

        let uno = db
            .tomar_siguiente("camp", AHORA)
            .expect("toma")
            .expect("hay");
        let dos = db
            .tomar_siguiente("camp", AHORA)
            .expect("toma")
            .expect("hay");
        assert_ne!(uno.id, dos.id, "el mismo intento se entregó dos veces");
        assert_eq!(db.tomar_siguiente("camp", AHORA).expect("toma"), None);
    }

    /// Cada campaña ve sólo lo suyo.
    #[test]
    fn no_se_toma_trabajo_de_otra_campana() {
        let (_d, db) = base();
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO campaign (id, company_id, name, created_at, updated_at)
                 VALUES ('otra', 'emp', 'Otra', ?1, ?1)",
                params![AHORA],
            )
        })
        .expect("otra campaña");
        encolar(&db, "a", "a@x.mx", AttemptState::Queued, AHORA);

        assert_eq!(db.tomar_siguiente("otra", AHORA).expect("toma"), None);
    }

    /// Un reintento programado para más tarde no se toma antes de tiempo.
    ///
    /// Es lo que hace que la política de esperas de D-12 signifique algo: sin
    /// esta condición, los 5, 15 y 30 minutos se ignorarían y el reintento
    /// saldría de inmediato contra el servidor que acaba de fallar.
    #[test]
    fn un_reintento_no_se_adelanta_a_su_hora() {
        let (_d, db) = base();
        encolar(&db, "a", "a@x.mx", AttemptState::Failed, AHORA);
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "UPDATE message_attempt SET next_retry_at = '2026-09-23T12:30:00Z'
                  WHERE id = 'a'",
                [],
            )
        })
        .expect("programa el reintento");

        assert_eq!(
            db.tomar_siguiente("camp", "2026-09-23T12:05:00Z")
                .expect("toma"),
            None,
            "todavía no toca: faltan veinticinco minutos"
        );
        assert!(
            db.tomar_siguiente("camp", "2026-09-23T12:31:00Z")
                .expect("toma")
                .is_some(),
            "pasada su hora, sí"
        );
    }

    // ── La recuperación al arrancar · ADR-0004 ───────────────────────────

    /// **La puerta de salida de la entrega 4.2.**
    ///
    /// Se simula exactamente la caída del §3: la aplicación muere con un envío
    /// en `sending`, es decir, después de llamar al proveedor y antes de poder
    /// anotar el resultado.
    ///
    /// Al arrancar, esa fila **no vuelve a la cola**. Pasa a «envío no
    /// confirmado» y se le enseña al usuario, que decide.
    ///
    /// ROTO A PROPÓSITO: se cambió el destino de `sending` a `queued` —que es
    /// lo que haría una recuperación ingenua, «lo que quedó a medias se
    /// reintenta»— y esta prueba falló nombrando el duplicado.
    #[test]
    fn un_envio_a_medias_no_se_reenvia_jamas() {
        let (_d, db) = base();
        encolar(&db, "medias", "ana@x.mx", AttemptState::Sending, AHORA);

        let r = db.recuperar_al_arrancar(AHORA).expect("recupera");
        assert_eq!(r.envios_sin_confirmar, 1);

        assert_eq!(
            estado_de(&db, "medias"),
            "presumed_sent",
            "un envío que pudo haber salido NO vuelve a la cola: si salió y se \
             reenvía, esa persona recibe dos veces y eso no se deshace"
        );

        // Y no lo toma nadie después.
        assert_eq!(
            db.tomar_siguiente("camp", AHORA).expect("toma"),
            None,
            "un envío no confirmado no es trabajo pendiente"
        );
    }

    /// Lo que quedó **tomado y sin enviar** sí vuelve a la cola.
    ///
    /// La distinción es todo el asunto: en `claimed` nadie llamó al proveedor,
    /// así que no existe ningún correo que pueda estar duplicado. Tratar los
    /// dos casos igual sería perder envíos sin motivo —si se dieran todos por
    /// ambiguos— o duplicarlos —si se reintentaran todos—.
    #[test]
    fn lo_tomado_y_no_enviado_vuelve_a_la_cola() {
        let (_d, db) = base();
        encolar(&db, "tomado", "beto@x.mx", AttemptState::Claimed, AHORA);

        let r = db.recuperar_al_arrancar(AHORA).expect("recupera");
        assert_eq!(r.devueltos_a_la_cola, 1);
        assert_eq!(r.envios_sin_confirmar, 0);
        assert_eq!(estado_de(&db, "tomado"), "queued");

        assert!(
            db.tomar_siguiente("camp", AHORA).expect("toma").is_some(),
            "nadie llamó al proveedor: reenviarlo es seguro"
        );
    }

    /// Los estados terminales no los toca la recuperación.
    ///
    /// Un `sent` que volviera a la cola sería un duplicado garantizado, y un
    /// `suppressed` que volviera escribiría a quien pidió no recibir nada.
    #[test]
    fn la_recuperacion_no_toca_lo_que_ya_terminó() {
        let (_d, db) = base();
        for (id, estado) in [
            ("enviado", AttemptState::Sent),
            ("suprimido", AttemptState::Suppressed),
            ("cancelado", AttemptState::Cancelled),
            ("fallido", AttemptState::PermanentlyFailed),
            ("ambiguo", AttemptState::PresumedSent),
        ] {
            encolar(&db, id, &format!("{id}@x.mx"), estado, AHORA);
        }

        let r = db.recuperar_al_arrancar(AHORA).expect("recupera");
        assert_eq!(r, Recuperacion::default(), "no había nada a medias");

        for (id, estado) in [
            ("enviado", "sent"),
            ("suprimido", "suppressed"),
            ("cancelado", "cancelled"),
            ("fallido", "permanently_failed"),
            ("ambiguo", "presumed_sent"),
        ] {
            assert_eq!(estado_de(&db, id), estado, "{id} no debía moverse");
        }
    }

    /// `marcar_enviando` escribe **antes** de llamar al proveedor, y sólo sobre
    /// un intento tomado.
    #[test]
    fn solo_se_puede_marcar_enviando_lo_que_esta_tomado() {
        let (_d, db) = base();
        encolar(&db, "a", "a@x.mx", AttemptState::Queued, AHORA);

        assert!(
            db.marcar_enviando("a", AHORA).is_err(),
            "no se puede empezar a enviar algo que nadie ha tomado"
        );

        db.tomar_siguiente("camp", AHORA).expect("toma");
        db.marcar_enviando("a", AHORA).expect("ahora sí");
        assert_eq!(estado_de(&db, "a"), "sending");
    }

    /// El contador de intentos sube al empezar a enviar, no al terminar.
    ///
    /// Si subiera al terminar, un envío que muere a mitad no contaría — y la
    /// política de reintentos de D-12 creería que va por el primero cuando ya
    /// es el tercero.
    #[test]
    fn el_contador_de_intentos_sube_al_empezar_no_al_acabar() {
        let (_d, db) = base();
        encolar(&db, "a", "a@x.mx", AttemptState::Queued, AHORA);
        db.tomar_siguiente("camp", AHORA).expect("toma");
        db.marcar_enviando("a", AHORA).expect("marca");

        let cuenta: i64 = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT attempt_count FROM message_attempt WHERE id = 'a'",
                    [],
                    |f| f.get(0),
                )
            })
            .expect("lee");
        assert_eq!(cuenta, 1);
    }

    /// **El esquema impide el duplicado aunque el código falle.**
    ///
    /// Es la primera de las tres capas, y la única que no depende de que
    /// nuestro código esté bien: la base rechaza dos intentos para la misma
    /// campaña, canal y dirección.
    #[test]
    fn la_base_rechaza_dos_intentos_para_la_misma_persona() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx", AttemptState::Queued, AHORA);

        let segundo = db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO message_attempt
                     (id, campaign_id, channel, contact_address, idempotency_key,
                      state, attempt_count, created_at, updated_at)
                 VALUES ('b', 'camp', 'email', 'ana@x.mx', 'otra', 'queued', 0, ?1, ?1)",
                params![AHORA],
            )
        });
        assert!(
            segundo.is_err(),
            "la base tiene que rechazar el segundo intento para ana@x.mx en la \
             misma campaña: es la capa que sigue funcionando cuando el código no"
        );
    }

    // ─── Los desenlaces (entrega 4.3b) ──────────────────────────────────────

    /// Deja un intento listo para recibir su desenlace: tomado y enviándose.
    fn enviandose(db: &Db, id: &str) {
        encolar(db, id, &format!("{id}@x.mx"), AttemptState::Queued, AHORA);
        db.tomar_siguiente("camp", AHORA).expect("toma");
        db.marcar_enviando(id, AHORA).expect("marca");
    }

    fn columna(db: &Db, id: &str, columna: &str) -> Option<String> {
        let sql = format!("SELECT {columna} FROM message_attempt WHERE id = ?1");
        db.ejecutar_en_pruebas(|c| c.query_row(&sql, params![id], |f| f.get(0)))
            .expect("lee columna")
    }

    #[test]
    fn aceptar_deja_la_respuesta_del_proveedor_y_la_hora() {
        let (_d, db) = base();
        enviandose(&db, "a");
        db.marcar_enviado("a", Some("prov-77"), Some("250 Ok"), AHORA)
            .expect("acepta");

        assert_eq!(estado_de(&db, "a"), "sent");
        assert_eq!(
            columna(&db, "a", "provider_message_id").as_deref(),
            Some("prov-77")
        );
        assert_eq!(
            columna(&db, "a", "provider_response").as_deref(),
            Some("250 Ok")
        );
        assert_eq!(columna(&db, "a", "sent_at").as_deref(), Some(AHORA));
    }

    /// **La hora del reintento se escribe con el estado, no después.**
    ///
    /// Sin ella la fila queda en `failed` con `next_retry_at` nulo, y
    /// `siguiente_candidato` lee ese nulo como «ya toca»: el reintento sale de
    /// inmediato en vez de dentro de cinco minutos.
    ///
    /// Verificada rompiéndola el 23/09/2026: quitando `next_retry_at = ?3` del
    /// `UPDATE`, esta prueba falla en la segunda comprobación, y la de abajo
    /// —`un_fallo_no_vuelve_a_salir_antes_de_su_hora`— devuelve el intento
    /// inmediatamente.
    #[test]
    fn fallar_programa_la_hora_del_reintento() {
        let (_d, db) = base();
        enviandose(&db, "a");
        db.marcar_fallido(
            "a",
            "motorRedCaida",
            Some("timeout"),
            "2026-09-23T12:05:00Z",
            AHORA,
        )
        .expect("falla");

        assert_eq!(estado_de(&db, "a"), "failed");
        assert_eq!(
            columna(&db, "a", "next_retry_at").as_deref(),
            Some("2026-09-23T12:05:00Z"),
            "un fallo sin hora de reintento vuelve a salir al instante"
        );
        assert_eq!(
            columna(&db, "a", "error_kind").as_deref(),
            Some("motorRedCaida")
        );
    }

    /// Un intento fallido no se vuelve a tomar hasta que llega su hora.
    #[test]
    fn un_fallo_no_vuelve_a_salir_antes_de_su_hora() {
        let (_d, db) = base();
        enviandose(&db, "a");
        db.marcar_fallido("a", "motorRedCaida", None, "2026-09-23T12:05:00Z", AHORA)
            .expect("falla");

        assert!(
            db.tomar_siguiente("camp", "2026-09-23T12:04:59Z")
                .expect("toma")
                .is_none(),
            "a un minuto de su hora todavía no le toca"
        );
        assert!(
            db.tomar_siguiente("camp", "2026-09-23T12:05:00Z")
                .expect("toma")
                .is_some(),
            "cuando llega su hora vuelve a la cola"
        );
    }

    /// Rendirse borra la hora del reintento: nadie va a volver a mirarla.
    #[test]
    fn rendirse_no_deja_una_hora_futura_escrita() {
        let (_d, db) = base();
        enviandose(&db, "a");
        db.marcar_fallido("a", "motorRedCaida", None, "2026-09-23T12:05:00Z", AHORA)
            .expect("falla");
        db.tomar_siguiente("camp", "2026-09-23T12:05:00Z")
            .expect("toma");
        db.marcar_enviando("a", "2026-09-23T12:05:00Z")
            .expect("marca");
        db.marcar_rendido("a", "motorRedCaida", None, "2026-09-23T12:05:01Z")
            .expect("se rinde");

        assert_eq!(estado_de(&db, "a"), "permanently_failed");
        assert_eq!(
            columna(&db, "a", "next_retry_at"),
            None,
            "una fila rendida con hora futura le dice a quien la lea que va a \
             volver a salir, y no va a volver a salir"
        );
    }

    /// Un intento suprimido se cierra sin motivo de error: no es un fallo.
    #[test]
    fn suprimir_cierra_el_intento_sin_marcarlo_como_error() {
        let (_d, db) = base();
        encolar(&db, "a", "a@x.mx", AttemptState::Queued, AHORA);
        db.tomar_siguiente("camp", AHORA).expect("toma");
        db.marcar_suprimido("a", AHORA).expect("suprime");

        assert_eq!(estado_de(&db, "a"), "suppressed");
        assert_eq!(
            columna(&db, "a", "error_kind"),
            None,
            "respetar una baja es el sistema funcionando, no fallando"
        );
    }

    /// **Un envío que ya empezó no se puede cerrar como «baja».**
    ///
    /// Se intentó permitirlo —el proveedor puede contestar «pidió la baja»
    /// mientras se le manda— y no procede: `suppressed` no cuenta como envío
    /// consumido, así que una fila que llegó a `sending`, donde el mensaje
    /// **pudo** salir, quedaría fuera del presupuesto. Ese caso se cierra como
    /// `permanently_failed`.
    #[test]
    fn un_envio_ya_empezado_no_se_cierra_como_baja() {
        let (_d, db) = base();
        enviandose(&db, "a");
        assert!(db.marcar_suprimido("a", AHORA).is_err());
        assert_eq!(estado_de(&db, "a"), "sending");
    }

    /// **Una credencial caducada no gasta reintentos.**
    ///
    /// `attempt_count` sube al empezar a enviar. Si una cuenta que no autentica
    /// lo dejara subido, al arreglarla los envíos se rendirían enseguida sin
    /// haber pasado nunca por un servidor de correo.
    ///
    /// Verificada rompiéndola el 23/09/2026: cambiando
    /// `MAX(attempt_count - 1, 0)` por `attempt_count`, esta prueba falla
    /// diciendo que el contador quedó en 1.
    #[test]
    fn una_cuenta_que_no_autentica_devuelve_el_intento_sin_gastarlo() {
        let (_d, db) = base();
        enviandose(&db, "a");
        db.marcar_en_espera_de_la_cuenta("a", "reintento.laCuentaNoAutentica", None, AHORA)
            .expect("espera");

        assert_eq!(estado_de(&db, "a"), "failed");
        let cuenta: i64 = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT attempt_count FROM message_attempt WHERE id = 'a'",
                    [],
                    |f| f.get(0),
                )
            })
            .expect("lee");
        assert_eq!(
            cuenta, 0,
            "el envío no llegó a intentarse: lo que falló fue la cuenta"
        );
        assert_eq!(
            columna(&db, "a", "next_retry_at").as_deref(),
            Some(AHORA),
            "esperar no arregla una credencial; tiene que salir en cuanto la \
             arreglen"
        );
    }

    #[test]
    fn cancelar_cierra_un_intento_tomado() {
        let (_d, db) = base();
        encolar(&db, "a", "a@x.mx", AttemptState::Queued, AHORA);
        db.tomar_siguiente("camp", AHORA).expect("toma");
        db.marcar_cancelado("a", AHORA).expect("cancela");
        assert_eq!(estado_de(&db, "a"), "cancelled");
    }

    /// **Un desenlace no se puede escribir sobre una fila que no lo espera.**
    ///
    /// Cada `UPDATE` de desenlace lleva su propio compare-and-swap. Sin él,
    /// dos trabajadores —o un reintento tardío— podrían escribir `sent` sobre
    /// una fila que otro dejó en `sending`, y el registro diría que salió una
    /// vez lo que salió dos.
    ///
    /// Verificada rompiéndola el 23/09/2026: quitando `AND state = ?` de
    /// `marcar_enviado`, los cuatro `assert` de abajo pasan a devolver `Ok`.
    #[test]
    fn ningun_desenlace_se_escribe_sobre_una_fila_en_otro_estado() {
        let (_d, db) = base();
        // Se queda en `queued`: nadie la tomó y nadie la está enviando.
        encolar(&db, "a", "a@x.mx", AttemptState::Queued, AHORA);

        assert!(db.marcar_enviado("a", None, None, AHORA).is_err());
        assert!(db.marcar_fallido("a", "x", None, AHORA, AHORA).is_err());
        assert!(db.marcar_rendido("a", "x", None, AHORA).is_err());
        assert!(db.marcar_suprimido("a", AHORA).is_err());
        assert_eq!(
            estado_de(&db, "a"),
            "queued",
            "y ninguno de los cinco llegó a tocar la fila"
        );
    }

    /// **La máquina de estados y la cola tienen que describir el mismo
    /// programa.**
    ///
    /// `arles_core::attempt` es la descripción que se lee; el `WHERE` del
    /// compare-and-swap es la que se ejecuta. Nada las obligaba a coincidir.
    ///
    /// Esta prueba las enfrenta, y **falló al escribirla** (23/09/2026): la
    /// cola toma directamente los intentos en `failed` cuyo `next_retry_at` ya
    /// pasó, pero la máquina sólo permitía `Failed → Queued`. La documentación
    /// describía un paso intermedio que ningún código ejecutaba. Se corrigió la
    /// máquina, que era la que mentía.
    #[test]
    fn la_maquina_de_estados_permite_todo_lo_que_la_cola_toma() {
        for estado in TOMABLES {
            assert!(
                estado.puede_ir_a(AttemptState::Claimed),
                "la cola toma intentos en «{estado}», y la máquina de estados \
                 dice que ese paso no existe. Una de las dos miente"
            );
        }
    }
}
