//! Crear una campaña, listarla, y **activarla**.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! ACTIVAR ES EL ACTO IRREVERSIBLE DE ARLES
//!
//! Todo lo demás se deshace: una plantilla se versiona, un contacto se edita,
//! una cuenta se reconfigura. Activar una campaña **no**. A partir de ese
//! instante hay correos en una cola que van a salir, y un correo que salió no
//! vuelve.
//!
//! Por eso activar hace tres cosas **en una sola transacción**, y o pasan las
//! tres o no pasa ninguna:
//!
//! 1. **Se congela la audiencia.** Quién entra en esta campaña se decide ahora
//!    y no se vuelve a calcular.
//! 2. **Se llena la cola**, una fila por destinatario, con su clave de
//!    idempotencia ya escrita.
//! 3. **La campaña pasa a «enviando».**
//!
//! Si se hicieran por separado, una caída entre la primera y la segunda dejaría
//! una campaña con audiencia y sin cola —enviando a nadie, para siempre— y una
//! entre la segunda y la tercera dejaría correos en cola de una campaña que
//! sigue diciendo «borrador». Las dos cosas son silenciosas.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ SE CONGELA LA AUDIENCIA
//!
//! Sin instantánea, «¿a quién le llegó esta campaña?» se contestaría
//! recalculando la lista hoy — y hoy hay contactos que se han dado de baja,
//! otros que se importaron después y otros que se borraron.
//!
//! La respuesta cambiaría cada vez que se hace la pregunta, y eso no es una
//! respuesta. La instantánea guarda **la dirección**, no sólo el contacto: si
//! el contacto se borra, la fila se queda con el `contact_id` a nulo y la
//! dirección intacta. Una instantánea que encoge sola no es una instantánea.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LAS BAJAS SE MIRAN DOS VECES, Y NO SOBRA NINGUNA
//!
//! Aquí, al congelar, para no meter en la cola a quien ya había pedido no
//! recibir. Y otra vez en el motor, antes de cada envío (`bucle.rs`): entre que
//! una campaña se activa y un correo concreto sale pueden pasar semanas, y
//! alguien puede haber pedido la baja ayer.
//!
//! Quitar la de aquí llenaría la cola de gente a la que no hay que escribir —el
//! motor los descartaría uno a uno, pero las cifras de la campaña mentirían
//! desde el primer minuto. Quitar la del motor mandaría correos a quien pidió
//! no recibirlos.

use arles_core::campana::EstadoDeCampana;
use arles_core::ids::{
    CampaignId, CompanyId, ContactListId, EmailAccountId, IdempotencyKey, MessageAttemptId,
    TemplateVersionId, WhatsappAccountId, WhatsappTemplateId,
};
use arles_core::plantilla::tiene_salida_de_baja;
use rusqlite::{OptionalExtension, params};

use arles_core::ritmo::Limites;

use time::OffsetDateTime;

use crate::db::Db;
use crate::empresa::{ahora, auditar};
use crate::error::DbError;

/// Por dónde sale una campaña, con lo que ese canal necesita y nada más.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ UNA VARIANTE Y NO CINCO CAMPOS OPCIONALES (D-WA, 27/10/2026)
///
/// La forma cómoda sería añadir `canal`, `numero` y `plantilla_de_meta` al
/// lado de `cuenta`, `version_de_plantilla` y `firma`, los seis opcionales. Y
/// entonces caben los estados imposibles: canal de WhatsApp con una plantilla
/// de correo, canal de correo con un número de Meta, o un canal sin ninguna de
/// las dos cosas. **Nada fallaría al construirlo**: el `CHECK` del esquema
/// rechazaría la fila, y el error llegaría como un fallo de SQLite en vez de
/// decir qué falta.
///
/// Es la misma decisión que [`CuentaDeLaEtapa`] en `trabajo.rs`,
/// `Cifrado::Argon2idXChaCha20 { sal, nonce }` y `RecortadaEnElEsencial
/// { fecha }`: **el dato que sólo tiene sentido con otro va dentro de la
/// variante**.
///
/// Y tiene el segundo efecto, que es el que de verdad protege: el compilador
/// **exige una decisión** en cada sitio que antes daba el correo por supuesto.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LA FIRMA SÓLO EXISTE EN CORREO, Y ESO ES UNA REGLA DEL ESQUEMA
///
/// `campaign_stage` lleva desde la V4 un `CHECK (channel = 'email' OR
/// signature_id IS NULL)`, con su motivo escrito: «una firma colgada de una
/// etapa de WhatsApp es un campo que alguien rellenaría esperando que saliera
/// en el mensaje». Aquí ni se puede escribir.
#[derive(Debug, Clone)]
pub enum PorDondeSale<'a> {
    Correo {
        cuenta: EmailAccountId,
        version_de_plantilla: TemplateVersionId,
        /// La firma que se pega al final de cada correo, o `None` para la de
        /// por defecto de la empresa.
        ///
        /// ── LA COLUMNA EXISTÍA DESDE LA V4 Y NADIE LA ESCRIBÍA (H-12) ────
        ///
        /// `campaign_stage.signature_id` lleva ahí desde la V4, y
        /// `redaccion_de_la_campana` ya la consultaba —con caída a la de por
        /// defecto— **meses antes de que nada la rellenara**. El patrón del §9
        /// en su forma más silenciosa: la consulta correcta, la columna
        /// correcta, y el cable sin conectar; nada fallaba, simplemente todas
        /// las campañas salían con la misma firma.
        ///
        /// `None` sigue significando «la de por defecto de la empresa», que es
        /// literalmente lo que promete la casilla de Plantillas: «usar esta
        /// firma cuando no se elija otra».
        firma: Option<&'a str>,
    },
    /// Una etapa de WhatsApp: el número y la plantilla que Meta aprobó.
    ///
    /// **La plantilla es opcional aquí y obligatoria al activar.** La etapa
    /// nace al elegir canal y número, y la plantilla se elige dos pasos
    /// después del asistente; exigirla aquí obligaría a preguntar en el orden
    /// contrario al que la pantalla pregunta, y el primero que tropieza con eso
    /// es quien vuelve atrás a cambiar algo. El motivo entero está en la V16.
    Whatsapp {
        numero: WhatsappAccountId,
        plantilla_de_meta: Option<WhatsappTemplateId>,
    },
}

impl PorDondeSale<'_> {
    /// Cómo se escribe este canal en la columna `channel`.
    const fn como_se_guarda(&self) -> &'static str {
        match self {
            Self::Correo { .. } => "email",
            Self::Whatsapp { .. } => "whatsapp",
        }
    }
}

/// Lo que hace falta para crear una campaña.
///
/// Una campaña **nace con su etapa**: sin remitente y sin horario no se puede
/// activar, y una campaña que no se puede activar es una fila que la interfaz
/// enseña y que no lleva a ninguna parte.
#[derive(Debug, Clone)]
pub struct CampanaNueva<'a> {
    pub nombre: &'a str,
    /// Por dónde sale, con lo que ese canal necesita. Ver [`PorDondeSale`].
    pub sale_por: PorDondeSale<'a>,
    /// **A quién va dirigida** (D-15, 23/10/2026).
    ///
    /// `None` significa «toda la empresa», que es lo que hacían todas las
    /// campañas hasta hoy. Sigue siendo una opción legítima —un aviso que va a
    /// todo el padrón existe— pero **la pantalla la ofrece con esas palabras**:
    /// elegida a propósito y nunca por omisión, que es justo lo que llevaba a
    /// mandar una campaña de videovigilancia a quien pidió una aplicación.
    pub lista: Option<ContactListId>,
    /// La ventana de ejecución ya guardada. Es de la empresa, y se reutiliza
    /// entre campañas.
    pub ventana: &'a str,
    /// Los topes de esta etapa. `None` deja mandar el de la cuenta.
    ///
    /// **Y mandan ellos desde el 22/10/2026**, también cuando piden más que la
    /// cuenta: ver la cabecera de `trabajo.rs`.
    pub limite_por_hora: Option<u32>,
    pub limite_diario: Option<u32>,
    /// ¿Se aceptó el riesgo de pasar de 50 diarios (§48)?
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// LA REGLA VIVE AQUÍ Y NO EN LA PANTALLA
    ///
    /// La casilla la enseña el asistente, pero quien **exige** la aceptación es
    /// esta función. Si la regla viviera arriba, cualquier otro camino hacia
    /// `crear_campana` —otra pantalla, una prueba, el día que haya importación
    /// de campañas— crearía la campaña sin ella y sin que nada fallara.
    ///
    /// Y no bloquea el volumen: §154 pide informar, no dictar. Lo que se exige
    /// es **constancia de que se informó**, que es distinto.
    pub acepto_el_riesgo_del_volumen: bool,
}

/// Lo que queda al crear una campaña.
///
/// Lleva **el id de la etapa** además del de la campaña, y no por comodidad: el
/// envío de prueba del §45 cuelga de la etapa, no de la campaña. Sin esto,
/// quien acabe de crearla tendría que reconstruir ese identificador por su
/// cuenta —hoy es `st-1-{campaña}`—, y ese formato es un detalle de esta
/// función que dejaría de serlo en cuanto alguien lo copiara.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampanaCreada {
    pub id: CampaignId,
    pub etapa: String,
}

/// Una campaña tal como la enseña la lista.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampanaGuardada {
    pub id: CampaignId,
    pub nombre: String,
    pub estado: EstadoDeCampana,
    /// Cuántos destinatarios tiene congelados. Cero mientras es borrador.
    pub audiencia: u32,
    /// Cuántos han salido ya. **No dice «entregados»** (§65): dice que el
    /// proveedor se hizo cargo.
    pub aceptados: u32,
}

/// Lo que pasó al activar. Cifras, no adjetivos (§94).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Activacion {
    /// Cuántos entraron en la audiencia congelada.
    pub en_la_audiencia: u32,
    /// Cuántos se dejaron fuera por estar en la lista de bajas.
    ///
    /// Se devuelve para poder decirlo: «se activó para 1 240 de 1 300 — 60
    /// habían pedido no recibir» es una cifra con la que se puede vivir. «Se
    /// activó» a secas deja a alguien preguntándose por qué el contador no
    /// coincide con su lista.
    pub fuera_por_baja: u32,
}

impl Db {
    /// Crea una campaña en borrador, con su etapa de correo.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla — entre otras cosas, si el
    /// remitente, la plantilla o la ventana no existen: son claves foráneas.
    pub fn crear_campana(
        &self,
        empresa: CompanyId,
        nueva: &CampanaNueva<'_>,
    ) -> Result<CampanaCreada, DbError> {
        // El aviso del §48 se exige **antes** de escribir nada: una campaña
        // creada a medias y luego rechazada deja una fila que la lista enseña.
        let de_alto_volumen = nueva
            .limite_diario
            .is_some_and(|d| d > Limites::AVISO_DIARIO);
        if de_alto_volumen && !nueva.acepto_el_riesgo_del_volumen {
            return Err(DbError::FaltaAceptarElVolumen);
        }

        // Un cero crea una campaña que se activa y no manda nunca nada. Se
        // comprueba aquí además de en la pantalla porque la frontera no se fía
        // de la webview, y porque los dos campos valen: con cero por hora
        // tampoco sale nada, aunque el diario sea generoso.
        if nueva.limite_diario == Some(0) || nueva.limite_por_hora == Some(0) {
            return Err(DbError::LimiteDeEnviosEnCero);
        }

        let id = CampaignId::nuevo();
        let etapa = format!("st-1-{id}");

        self.con(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();

            tx.execute(
                "INSERT INTO campaign (id, company_id, name, status, contact_list_id,
                                       created_at, updated_at)
                 VALUES (?1, ?2, ?3, 'draft', ?5, ?4, ?4)",
                params![
                    id.to_string(),
                    empresa.to_string(),
                    nueva.nombre.trim(),
                    cuando,
                    nueva.lista.map(|l| l.to_string()),
                ],
            )?;

            // ── LA ETAPA 1, DEL CANAL QUE SEA (D-WA, 27/10/2026) ──────
            //
            // Las dos escrituras llevan las MISMAS columnas y en el mismo
            // orden, con los huecos del otro canal a `NULL`. Es a propósito:
            // dos `INSERT` con listas de columnas distintas son dos sitios
            // donde añadir un campo —la V16 añadió uno— y ponerlo en uno solo.
            // El `CHECK` del esquema exige exactamente una cuenta por canal, y
            // otro que una firma sólo cuelgue del correo.
            //
            // El `match` no tiene `_ =>`, y es lo que protege: añadir un canal
            // mañana no compila hasta decidir qué escribe, en vez de crear una
            // campaña sin remitente que falla al activar.
            let (correo, version, firma, numero, plantilla_de_meta) = match &nueva.sale_por {
                PorDondeSale::Correo {
                    cuenta,
                    version_de_plantilla,
                    firma,
                } => (
                    Some(cuenta.to_string()),
                    Some(version_de_plantilla.to_string()),
                    *firma,
                    None,
                    None,
                ),
                PorDondeSale::Whatsapp {
                    numero,
                    plantilla_de_meta,
                } => (
                    None,
                    None,
                    None,
                    Some(numero.to_string()),
                    plantilla_de_meta.map(|p| p.to_string()),
                ),
            };

            tx.execute(
                "INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             email_account_id, whatsapp_account_id,
                                             template_version_id, whatsapp_template_id,
                                             signature_id, execution_window_id,
                                             hourly_limit, daily_limit,
                                             status, created_at, updated_at)
                 VALUES (?1, ?2, 1, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'draft', ?12, ?12)",
                params![
                    etapa,
                    id.to_string(),
                    nueva.sale_por.como_se_guarda(),
                    correo,
                    numero,
                    version,
                    plantilla_de_meta,
                    firma,
                    nueva.ventana,
                    nueva.limite_por_hora,
                    nueva.limite_diario,
                    cuando
                ],
            )?;

            auditar(
                &tx,
                &empresa.to_string(),
                "crear",
                "campaign",
                &id.to_string(),
            )?;

            // La aceptación del §48 se registra aparte, y en la misma
            // transacción: es el rastro de que se avisó y de que alguien dijo
            // que sí, con su fecha. Sin línea propia, el día que una cuenta se
            // marque como spam no habría forma de saber si se informó.
            if de_alto_volumen {
                auditar(
                    &tx,
                    &empresa.to_string(),
                    "aceptar_riesgo_de_volumen",
                    "campaign",
                    &id.to_string(),
                )?;
            }
            tx.commit()?;
            Ok(CampanaCreada { id, etapa })
        })
    }

    /// Las campañas de una empresa, con sus cifras.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla, o [`DbError::DatoInvalido`] si
    /// una fila guarda un estado que el núcleo no conoce. **No se salta**: una
    /// campaña que desaparece de la lista sin decir por qué es peor que un
    /// error, porque nadie la va a echar de menos hasta que pregunte por ella.
    pub fn campanas(&self, empresa: CompanyId) -> Result<Vec<CampanaGuardada>, DbError> {
        self.con_dominio(|conn| {
            let mut c = conn.prepare(
                "SELECT c.id, c.name, c.status,
                        (SELECT count(*) FROM campaign_audience a
                          WHERE a.campaign_id = c.id),
                        (SELECT count(*) FROM message_attempt m
                          WHERE m.campaign_id = c.id AND m.state = 'sent')
                   FROM campaign c
                  WHERE c.company_id = ?1
                  ORDER BY c.updated_at DESC",
            )?;

            let filas = c.query_map(params![empresa.to_string()], |f| {
                Ok((
                    f.get::<_, String>(0)?,
                    f.get::<_, String>(1)?,
                    f.get::<_, String>(2)?,
                    f.get::<_, i64>(3)?,
                    f.get::<_, i64>(4)?,
                ))
            })?;

            let mut salida = Vec::new();
            for fila in filas {
                let (id, nombre, estado, audiencia, aceptados) = fila?;
                salida.push(CampanaGuardada {
                    id: id.parse().map_err(|_| DbError::DatoInvalido {
                        motivo: "campaign.id",
                    })?,
                    nombre,
                    estado: EstadoDeCampana::desde_texto(&estado).map_err(|_| {
                        DbError::DatoInvalido {
                            motivo: "campaign.status",
                        }
                    })?,
                    audiencia: u32::try_from(audiencia).unwrap_or(u32::MAX),
                    aceptados: u32::try_from(aceptados).unwrap_or(u32::MAX),
                });
            }
            Ok(salida)
        })
    }

    /// Activa una campaña: congela la audiencia, llena la cola y la pone a
    /// enviar. **En una sola transacción.**
    ///
    /// La audiencia son los contactos activos de la empresa con una dirección
    /// de correo, **menos los que están en la lista de bajas**.
    ///
    /// # Errores
    ///
    /// - [`DbError::DatoInvalido`] si la campaña no existe, si no está en
    ///   borrador o lista, o si no tiene etapa de correo.
    /// - [`DbError::AudienciaVacia`] si no queda nadie a quien escribir.
    ///   **Es un error y no un éxito con cero**: una campaña activada sin
    ///   destinatarios se queda diciendo «enviando» para siempre sin que nada
    ///   explique por qué el contador no sube.
    /// - [`DbError::Sqlite`] si la escritura falla.
    pub fn activar_campana(
        &self,
        empresa: CompanyId,
        campana: CampaignId,
    ) -> Result<Activacion, DbError> {
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();
            let id = campana.to_string();

            // 1 · La campaña existe, es de esta empresa y se puede activar.
            let estado: Option<String> = tx
                .query_row(
                    "SELECT status FROM campaign WHERE id = ?1 AND company_id = ?2",
                    params![id, empresa.to_string()],
                    |f| f.get(0),
                )
                .optional()?;
            let Some(estado) = estado else {
                return Err(DbError::DatoInvalido {
                    motivo: "campaign.id",
                });
            };
            if !matches!(estado.as_str(), "draft" | "ready") {
                // Activar dos veces duplicaría la cola entera. El
                // `UNIQUE(campaign_id, channel, contact_address)` lo impediría,
                // pero fallaría con un error de SQLite en vez de decir lo que
                // pasa.
                return Err(DbError::DatoInvalido {
                    motivo: "campaign.status",
                });
            }

            // ── 2 · LA ETAPA, DEL CANAL QUE SEA (D-WA, 27/10/2026) ────
            //
            // Hasta hoy esta consulta pedía `channel = 'email'` y, como la
            // siguiente —y la de la cola, y la de la supresión— escribían
            // `'email'` de literal, activar una campaña de WhatsApp **congelaba
            // una audiencia de correos** y llenaba la cola de direcciones de
            // correo con el canal equivocado. No fallaba nada: el motor
            // recorrería esa etapa (tramo 1), pediría el contenido de WhatsApp
            // y se lo mandaría a un correo.
            //
            // Ahora el canal **se lee de la etapa** y viaja como parámetro a
            // las cuatro consultas. Leerlo en vez de recibirlo es lo que
            // impide que quien activa diga uno distinto del que se guardó.
            let de_la_etapa = crate::trabajo::leer_la_etapa_para_activar(&tx, &id, empresa)?;
            let etapa = de_la_etapa.etapa.etapa.clone();
            let canal = de_la_etapa.etapa.cuenta.canal();
            let canal_txt = canal.como_texto();

            // ── 2 bis · UNA ETAPA DE WHATSAPP SIN PLANTILLA NO SE ACTIVA ─
            //
            // Lo prometía el comentario de la V16 —«quien SÍ lo exige es
            // `activar_campana`»— y **no lo exigía nadie**: la columna no
            // lleva `CHECK` a propósito, porque la etapa nace antes de la
            // elección. Sin esto, una campaña de WhatsApp se activaba, se
            // congelaba su audiencia, el motor la recorría, pedía el contenido
            // y no había ninguno: quieta para siempre y diciendo «enviando».
            //
            // Es el patrón del §9 con la cara más cara: la comprobación
            // descrita, el sitio nombrado, y el cable sin conectar.
            //
            // Y **aprobada**, que es la segunda mitad: una plantilla en
            // revisión o rechazada existe en la base, así que exigir sólo que
            // esté elegida dejaría activar una campaña entera que Meta rechaza
            // mensaje por mensaje. La regla de qué estado puede enviar vive en
            // `EstadoEnMeta::se_puede_usar`, escrita por lo que acepta y no por
            // lo que rechaza: un estado nuevo de Meta no empieza a enviar solo.
            if matches!(canal, arles_core::canal::Canal::WhatsApp) {
                let Some(plantilla) = de_la_etapa.plantilla_whatsapp.as_deref() else {
                    return Err(DbError::SinPlantillaDeMeta);
                };

                // Se pregunta aparte y no en `SELECT_DE_LA_ETAPA` a propósito:
                // esa consulta la corre el motor en cada vuelta, y una unión
                // más que sólo le hace falta al activar la pagarían todas.
                let estado_en_meta: Option<String> = tx
                    .query_row(
                        "SELECT status FROM whatsapp_template
                          WHERE id = ?1 AND company_id = ?2",
                        params![plantilla, empresa.to_string()],
                        |f| f.get(0),
                    )
                    .optional()?;

                let puede = estado_en_meta
                    .as_deref()
                    .and_then(|e| arles_core::whatsapp::EstadoEnMeta::de_guardado(e).ok())
                    .is_some_and(arles_core::whatsapp::EstadoEnMeta::se_puede_usar);
                if !puede {
                    return Err(DbError::LaPlantillaNoEstaAprobada);
                }
            }

            // 2 bis · A quién va dirigida (D-15).
            //
            // ── POR QUÉ SE LEE AQUÍ Y NO SE PASA COMO PARÁMETRO ──────────
            //
            // Porque la lista es un dato **de la campaña**, elegido al
            // crearla, y leerlo de la base es lo que impide que la webview
            // mande otro distinto del que se guardó. Es la misma razón por la
            // que `suprimir_contacto` lee las direcciones aquí en vez de
            // recibirlas.
            //
            // `None` significa «toda la empresa», que es lo que hicieron todas
            // las campañas hasta el 23/10/2026 y lo que siguen haciendo las ya
            // creadas. Es una opción legítima; lo que no puede ser es la que
            // ocurre sin que nadie la elija, y de eso se encarga la pantalla.
            let lista: Option<String> = tx.query_row(
                "SELECT contact_list_id FROM campaign WHERE id = ?1",
                params![id],
                |f| f.get(0),
            )?;

            // 3 · La audiencia. Se congela con la dirección NORMALIZADA, que es
            // la forma con la que se compara todo lo demás —la deduplicación y
            // la supresión—, y la que el motor usa para escribir.
            //
            // Tres filtros, y los tres hacen falta: el contacto activo, **el
            // canal activo** y la dirección no suprimida. El del canal se
            // olvidó al escribir esto y lo cazó la prueba
            // `un_canal_dado_de_baja_no_entra`: un contacto puede seguir activo
            // y haber borrado uno de sus correos, y escribir a una dirección
            // que su dueño quitó es exactamente lo que ARLES promete no hacer.
            // Y desde el 23/10/2026 un cuarto: **estar en la lista de la
            // campaña**, si tiene (D-15).
            //
            // ── POR QUÉ CAMBIA EL ORIGEN Y NO SÓLO EL `WHERE` ────────────
            //
            // La forma obvia es dejar el `FROM contact c` de siempre y añadir
            // un `EXISTS` sobre `contact_list_member`. Funciona, y se escribió
            // así primero. **Medido con `EXPLAIN QUERY PLAN` el 23/10/2026**,
            // su plan empieza con `SCAN ch`: recorre `contact_channel` entera
            // aunque la lista tenga trescientas personas. Una campaña pequeña
            // costaría lo mismo que una a todo el padrón —73.5 s con 451 196
            // contactos, medido en la auditoría C— y el aviso de espera de la
            // 8.6 E, que estima por número de destinatarios, prometería un
            // segundo.
            //
            // Entrando por `contact_list_member` el plan **no tiene ningún
            // `SCAN`**: `SEARCH m (list_id=?)` → `SEARCH c (id=?)` →
            // `SEARCH ch USING idx_channel_contact`. Proporcional a la lista y
            // no al padrón. Lo vigila `planes_de_consulta.rs`.
            //
            // Así que cambia **el origen**, y los filtros siguen escritos una
            // sola vez: son los que no se pueden olvidar, y dos copias de
            // ellos serían dos sitios donde arreglar uno y dejar el otro. El
            // `format!` interpola **constantes de este archivo**, nunca un
            // dato — que es lo único que SQLite no sabe parametrizar.
            //
            // Sin lista, `?4` entra igual y vale `NULL`: así las dos formas
            // llevan los mismos cuatro parámetros y nadie tiene que acordarse
            // de cambiarlos al tocar una.
            let (origen, filtro_lista) = if lista.is_some() {
                (
                    "contact_list_member m
                       JOIN contact c ON c.id = m.contact_id
                       JOIN contact_channel ch ON ch.contact_id = c.id",
                    "m.list_id = ?4",
                )
            } else {
                (
                    "contact c
                       JOIN contact_channel ch ON ch.contact_id = c.id",
                    "?4 IS NULL",
                )
            };

            // El recuento de los que quedan fuera por baja usa **el mismo
            // origen**. Sin eso diría «450 fuera por baja» contando a gente
            // que ni siquiera estaba invitada a esta campaña: una cifra
            // correcta sobre otra pregunta, que es la peor clase de cifra
            // (§94).
            let fuera: i64 = tx.query_row(
                &format!(
                    "SELECT count(*)
                       FROM {origen}
                      WHERE {filtro_lista}
                        AND c.company_id = ?2 AND c.status = 'active'
                        AND ch.channel = ?5 AND ch.status = 'active'
                        AND EXISTS (SELECT 1 FROM suppression_entry s
                                     WHERE s.company_id = ?2 AND s.channel = ?5
                                       AND s.address_normalized = ch.value_normalized)"
                ),
                params![id, empresa.to_string(), cuando, lista, canal_txt],
                |f| f.get(0),
            )?;

            let metidos = tx.execute(
                &format!(
                    "INSERT INTO campaign_audience (campaign_id, channel, contact_address,
                                                    contact_id, added_at)
                     SELECT ?1, ?5, ch.value_normalized, c.id, ?3
                       FROM {origen}
                      WHERE {filtro_lista}
                        AND c.company_id = ?2 AND c.status = 'active'
                        AND ch.channel = ?5 AND ch.status = 'active'
                        AND NOT EXISTS (SELECT 1 FROM suppression_entry s
                                         WHERE s.company_id = ?2 AND s.channel = ?5
                                           AND s.address_normalized = ch.value_normalized)
                      GROUP BY ch.value_normalized"
                ),
                params![id, empresa.to_string(), cuando, lista, canal_txt],
            )?;

            if metidos == 0 {
                return Err(DbError::AudienciaVacia);
            }

            // ── 3 bis · ¿Esta campaña puede terminar? ─────────────────────
            //
            // ── EL NÚCLEO YA LO SABÍA Y NADIE SE LO PREGUNTABA ───────────
            //
            // `estimar` devuelve tres desenlaces desde la Fase 4, y los tres
            // textos están escritos desde entonces — incluido el que dice «la
            // campaña no termina en cinco años: {faltan} mensajes se quedarían
            // sin enviar», cuyo comentario llama a esto «el caso que hay que
            // cazar antes de activar».
            //
            // No se cazaba. Hasta la 8.6 E se podía activar una campaña de
            // 451 196 personas con 50 correos al día: el motor enviaría
            // correctamente durante años, el contador subiría, y **386 046
            // personas no recibirían nada nunca** sin que nada fallara en
            // ningún momento. Es el patrón del §9 número diecisiete.
            //
            // ── POR QUÉ AQUÍ Y NO ANTES DE ESCRIBIR NADA ─────────────────
            //
            // Lo limpio sería rechazar antes de tocar la base, pero la
            // audiencia **no se sabe** hasta contarla, y contarla aparte es
            // otro recorrido completo del `JOIN` — segundos sobre medio
            // millón, pagados también por las campañas correctas.
            //
            // Así que se pregunta con la audiencia ya congelada y **antes del
            // bucle de la cola**, que es la parte cara: devolver un
            // `Err` aquí deshace la transacción entera —audiencia incluida— y
            // ahorra las 451 196 inserciones de `message_attempt`. Entero o
            // nada se mantiene porque lo mantiene la transacción, no el orden.
            //
            // Los límites son **el más bajo de la etapa y la cuenta**, igual
            // que en el motor y en el preflight: estimar con el de la etapa
            // cuando la cuenta tiene uno menor prometería un ritmo que el
            // proveedor no va a aceptar.
            {
                use arles_core::duracion::{Desenlace, estimar};

                // Los límites y la ventana salen de **la misma consulta que
                // el motor va a usar** (`SELECT_DE_LA_ETAPA`), y no de la del
                // preflight, que es sólo de correo. Que sean la misma es lo
                // que hace que esta estimación prometa el ritmo que de verdad
                // se va a aplicar — incluido el techo de Meta, que en WhatsApp
                // manda aunque la campaña pida más.
                let limites = de_la_etapa.etapa.limites;
                let audiencia = u32::try_from(metidos).unwrap_or(u32::MAX);
                // ── EL INSTANTE SE LEE AQUÍ, Y EN `lo_que_se_sabe_de` ENTRA
                //    POR PARÁMETRO. LA DIFERENCIA ES A PROPÓSITO ───────────
                //
                // Allí hace falta poder situarse un martes a las nueve, porque
                // lo que se enseña es **una fecha** y una fecha depende del día
                // en que se empiece.
                //
                // Aquí lo que se decide es un **veredicto**, y los dos que
                // rechazan no dependen del instante: con el tope a cero no sale
                // ni uno cualquier día que se empiece, y una audiencia que no
                // cabe en cinco años no cabe empezando un lunes ni un sábado —
                // empezar en fin de semana mueve el final dos días, no cinco
                // años. Pedir el instante por parámetro aquí sería un argumento
                // que ningún llamador tiene motivo para variar.
                let estimacion = estimar(
                    audiencia,
                    limites,
                    &de_la_etapa.etapa.ventana,
                    OffsetDateTime::now_utc(),
                );

                // ── Y LA SALIDA DE BAJA, ANTES DEL BUCLE DE LA COLA ────
                //
                // Se pregunta aquí por la misma razón que el desenlace: es el
                // único sitio donde se conocen **la plantilla y la firma** que
                // esta campaña va a usar, y la salida puede venir de
                // cualquiera de las dos.
                //
                // Devolver el error en este punto deshace la transacción
                // entera —audiencia incluida— y ahorra las inserciones de la
                // cola. **Entero o nada lo mantiene la transacción, no el
                // orden**, igual que con el desenlace.
                // ── Y EN WHATSAPP NO SE EXIGE, QUE ES DISTINTO DE QUE
                //    NO HAGA FALTA (D-WA, 27/10/2026) ──────────────────────
                //
                // `redaccion_en` devuelve `None` en una etapa de WhatsApp —no
                // hay plantilla de correo que leer—, así que hasta hoy la
                // exigencia **pasaba en verde sin mirar nada**. El `match` la
                // hace explícita: un `if let` que no entra se lee como «no
                // aplica», y aquí lo que ocurre es que no se puede comprobar.
                //
                // Por qué no se puede: la salida de correo se exige por la
                // variable `{{baja}}`, y lo que ARLES puede afirmar es que
                // **la sustituye por una salida que funciona**. En una
                // plantilla de Meta no hay nada que sustituir: el texto lo
                // aprueba Meta palabra por palabra y ARLES sólo manda su
                // nombre y los valores de sus huecos. Y buscar la palabra
                // «baja» en el cuerpo es justo lo que el comentario de
                // `tiene_salida_de_baja` descarta — pasaría en verde con «este
                // mes damos de baja los precios».
                //
                // ── Y POR QUÉ NO SE EXIGE QUE LA PLANTILLA LA LLEVE
                //    ESCRITA · DECISIÓN DE DIRECCIÓN DEL 27/10/2026 ────────
                //
                // Se planteó exigir que la plantilla registrada dijera su
                // propia salida —«responde BAJA»— como constancia, igual que
                // el aviso del §48. **Se descartó, y no por comodidad.**
                //
                // ARLES no podría atenderla. Dos veces:
                //
                //   1 · **No hay por dónde recibirla.** Los avisos de Meta son
                //       webhooks y necesitan una URL pública; ARLES es una
                //       aplicación de escritorio sin componente de red propio.
                //       Está escrito en la cabecera de
                //       `arles_whatsapp::entrante`.
                //   2 · **Y el texto se tira a propósito** (D-7A, 29/09/2026):
                //       se cuentan las respuestas, nunca qué dijeron.
                //       `Respuesta` no tiene dónde meter un texto y la V9 no
                //       tiene columna donde quepa. Así que la palabra «BAJA»
                //       no se leería ni con transporte.
                //
                // Imprimir en cada mensaje una salida que el producto no puede
                // atender es el §9 con su cara más cara: una promesa
                // que no se cumple. Lo que sí se cumple es la salida **a
                // mano**: con Coexistencia la respuesta se lee en el teléfono,
                // y Supresión acepta un número de WhatsApp y le da de baja.
                // Eso es lo que dicen el manual y la guía del canal.
                match canal {
                    arles_core::canal::Canal::Correo => {
                        if let Some(r) = crate::plantillas::redaccion_en(&tx, &id.to_string())? {
                            let en_el_cuerpo = tiene_salida_de_baja(&r.cuerpo_html);
                            let en_la_firma =
                                r.firma_html.as_deref().is_some_and(tiene_salida_de_baja);
                            if !en_el_cuerpo && !en_la_firma {
                                return Err(DbError::SinSalidaDeBaja);
                            }
                        }
                    }
                    arles_core::canal::Canal::WhatsApp => {}
                }

                match estimacion.desenlace {
                    Desenlace::NoAvanza => return Err(DbError::LaCampanaNoAvanza),
                    Desenlace::NoCabeEnElHorizonte { faltan } => {
                        return Err(DbError::LaCampanaNoTermina { faltan });
                    }
                    Desenlace::Termina { .. } => {}
                }
            }

            // 4 · La cola. Una fila por destinatario, con su clave de
            // idempotencia escrita ANTES de que nadie llame a un proveedor
            // (ADR-0004). Es lo único que permite reconocer después un envío
            // ambiguo.
            {
                // La cuenta va en **su** columna: la de correo o la del
                // número. Las dos existen en `message_attempt` desde la V4, y
                // escribir la de correo con un número dentro dejaría una clave
                // ajena apuntando a una tabla que no es — un `INSERT` que
                // falla, en el mejor caso, y una fila que el motor no sabe de
                // quién es en el peor.
                let (de_correo, de_whatsapp) = match &de_la_etapa.etapa.cuenta {
                    crate::trabajo::CuentaDeLaEtapa::Correo(c) => (Some(c.to_string()), None),
                    crate::trabajo::CuentaDeLaEtapa::Whatsapp(n) => (None, Some(n.to_string())),
                };

                let mut insertar = tx.prepare(
                    "INSERT INTO message_attempt (id, campaign_id, stage_id, channel,
                                                  contact_id, contact_address,
                                                  email_account_id, whatsapp_account_id,
                                                  idempotency_key,
                                                  state, attempt_count, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'queued', 0, ?10, ?10)",
                )?;

                let mut destinatarios = tx.prepare(
                    "SELECT contact_address, contact_id FROM campaign_audience
                      WHERE campaign_id = ?1 AND channel = ?2",
                )?;
                let filas = destinatarios.query_map(params![id, canal_txt], |f| {
                    Ok((f.get::<_, String>(0)?, f.get::<_, Option<String>>(1)?))
                })?;

                for fila in filas {
                    let (direccion, contacto) = fila?;
                    insertar.execute(params![
                        MessageAttemptId::nuevo().to_string(),
                        id,
                        etapa,
                        canal_txt,
                        contacto,
                        direccion,
                        de_correo,
                        de_whatsapp,
                        IdempotencyKey::nueva().to_string(),
                        cuando
                    ])?;
                }
            }

            // 5 · Y ahora sí, la campaña envía. La etapa también: son dos
            // estados desde la L-6, y dejar la etapa en borrador haría que el
            // motor no la viera nunca.
            tx.execute(
                "UPDATE campaign SET status = 'running', activated_at = ?2, updated_at = ?2
                  WHERE id = ?1",
                params![id, cuando],
            )?;
            tx.execute(
                "UPDATE campaign_stage SET status = 'running', activated_at = ?2,
                        updated_at = ?2
                  WHERE id = ?1",
                params![etapa, cuando],
            )?;

            auditar(&tx, &empresa.to_string(), "activar", "campaign", &id)?;
            tx.commit()?;

            Ok(Activacion {
                en_la_audiencia: u32::try_from(metidos).unwrap_or(u32::MAX),
                fuera_por_baja: u32::try_from(fuera).unwrap_or(u32::MAX),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conexion::ClaveMaestra;
    use crate::plantillas::Redaccion;

    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
    const CUENTA: &str = "01900000-0000-7000-8000-00000000000a";

    fn empresa() -> CompanyId {
        EMPRESA.parse().expect("id")
    }

    /// Base con empresa, cuenta, ventana y una plantilla lista.
    fn base() -> (tempfile::TempDir, Db, TemplateVersionId) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");

        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('{EMPRESA}', 'TELEMETRY', 'MX', 'America/Mexico_City',
                         'hola@t.mx', '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');

                 INSERT INTO email_account (id, company_id, display_name, email_address,
                                            provider_kind, credential_ref, status,
                                            daily_limit, hourly_limit, created_at, updated_at)
                 VALUES ('{CUENTA}', '{EMPRESA}', 'Ventas', 'ventas@t.mx', 'smtp',
                         'llavero://v', 'connected', 50, 10,
                         '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');

                 INSERT INTO execution_window (id, company_id, name, days_of_week,
                                               start_hour, end_hour, timezone, created_at)
                 VALUES ('vent-1', '{EMPRESA}', 'Laborables', 31, 9, 18,
                         'America/Mexico_City', '2026-09-29T00:00:00Z');"
            ))
        })
        .expect("siembra");

        let (_, version) = db
            .crear_plantilla(
                empresa(),
                "Bienvenida",
                &Redaccion {
                    asunto: "Hola {{nombre}}".to_owned(),
                    // Con `{{baja}}`: desde el 22/10/2026 `activar_campana`
                    // rechaza una campaña cuyo correo no le dé a quien recibe
                    // una forma de darse de baja. Sin esto, SEIS pruebas de
                    // este módulo caen — y verlas caer fue la comprobación de
                    // que la exigencia llega hasta donde importa.
                    cuerpo_html: "<p>Gracias</p><p>{{baja}}</p>".to_owned(),
                },
            )
            .expect("plantilla");

        (dir, db, version)
    }

    /// Añade un contacto activo con su correo.
    fn contacto(db: &Db, id: &str, correo: &str) {
        canal(db, id, correo, "active");
    }

    /// Un contacto con su canal en el estado que se le diga.
    fn canal(db: &Db, id: &str, correo: &str, estado: &str) {
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT OR IGNORE INTO contact (id, company_id, status, created_at, updated_at)
                 VALUES ('{id}', '{EMPRESA}', 'active',
                         '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');
                 INSERT INTO contact_channel (id, company_id, contact_id, channel,
                                              value_raw, value_normalized, status,
                                              created_at, updated_at)
                 VALUES ('ch-{id}', '{EMPRESA}', '{id}', 'email', '{correo}', '{correo}',
                         '{estado}', '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');"
            ))
        })
        .expect("contacto");
    }

    /// Y da de baja una dirección.
    fn suprimir(db: &Db, correo: &str) {
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO suppression_entry (id, company_id, channel,
                                                address_normalized, scope, reason,
                                                origin, created_at)
                 VALUES ('sup-{correo}', '{EMPRESA}', 'email', '{correo}', 'global',
                         'unsubscribe', 'user', '2026-09-29T00:00:00Z');"
            ))
        })
        .expect("supresión");
    }

    fn una_campana(db: &Db, version: TemplateVersionId) -> CampaignId {
        db.crear_campana(
            empresa(),
            &CampanaNueva {
                lista: None,
                nombre: "Septiembre",
                sale_por: PorDondeSale::Correo {
                    cuenta: CUENTA.parse().expect("id"),
                    version_de_plantilla: version,
                    firma: None,
                },
                ventana: "vent-1",
                limite_por_hora: None,
                limite_diario: None,
                // Sin tope propio no se pasa de 50: el de la cuenta manda.
                acepto_el_riesgo_del_volumen: false,
            },
        )
        .expect("crea")
        .id
    }

    // ─── Qué manda la campaña (R-07 paso 2b, 20/10/2026) ───────────────────

    /// **La campaña manda la versión que congeló, no la vigente.**
    ///
    /// Es la garantía entera del versionado vista desde el envío. La pantalla
    /// promete, al corregir una plantilla, que «las campañas que ya estén
    /// activas siguen enviando con la anterior» — y esto es lo único que puede
    /// comprobarlo: leyendo la vigente, corregir una errata cambiaría el texto
    /// de lo que está saliendo.
    ///
    /// Verificada rompiéndola el 20/10/2026: leyendo
    /// `template.current_version_id` en vez de
    /// `campaign_stage.template_version_id`, esta prueba falla con el asunto
    /// corregido.
    #[test]
    fn la_campana_manda_la_version_que_congelo_y_no_la_de_despues() {
        let (_d, db, version) = base();
        let plantilla = db
            .plantillas(empresa())
            .expect("lista")
            .first()
            .expect("hay una")
            .id;
        let campana = una_campana(&db, version);

        // Y después se corrige la plantilla, que es lo que R-06 permite.
        db.nueva_version_de_plantilla(
            plantilla,
            &Redaccion {
                asunto: "Hola de nuevo {{nombre}}".to_owned(),
                cuerpo_html: "<p>Texto corregido</p>".to_owned(),
            },
        )
        .expect("segunda versión");

        let r = db
            .redaccion_de_la_campana(&campana.to_string())
            .expect("consulta")
            .expect("la campaña tiene qué mandar");
        assert_eq!(r.asunto, "Hola {{nombre}}", "mandó la versión corregida");
        assert!(r.cuerpo_html.contains("Gracias"), "{}", r.cuerpo_html);
        assert_eq!(r.mi_empresa, "TELEMETRY", "es {{mi_empresa}}");
    }

    /// La firma que se pega es **la que la empresa marcó por defecto**.
    ///
    /// `campaign_stage.signature_id` existe desde la V4 y nada lo escribe
    /// (H-12), así que se usa la de por defecto — que es literalmente lo que
    /// promete su casilla: «usar esta firma cuando no se elija otra».
    #[test]
    fn la_firma_por_defecto_es_la_que_se_pega() {
        let (_d, db, version) = base();
        let campana = una_campana(&db, version);

        // Sin ninguna firma marcada, no se pega nada.
        let sin = db
            .redaccion_de_la_campana(&campana.to_string())
            .expect("consulta")
            .expect("existe");
        assert_eq!(sin.firma_html, None, "sin firma no se inventa una");

        db.crear_firma(empresa(), "Otra", "<p>No es la de por defecto</p>", false)
            .expect("una sin marcar");
        db.crear_firma(empresa(), "Ventas", "<p>Equipo de Ventas</p>", true)
            .expect("la de por defecto");

        let con = db
            .redaccion_de_la_campana(&campana.to_string())
            .expect("consulta")
            .expect("existe");
        let firma = con.firma_html.expect("hay firma");
        assert!(firma.contains("Equipo de Ventas"), "{firma}");
        assert!(
            !firma.contains("No es la de por defecto"),
            "se pegó la que no estaba marcada: {firma}"
        );
    }

    // ─── El desenlace, que hasta la 8.6 E nadie preguntaba ─────────────────

    /// **Una campaña que no termina nunca no se activa.**
    ///
    /// ── POR QUÉ ESTA PRUEBA NO EXISTÍA ANTES ─────────────────────────────
    ///
    /// Porque la garantía no existía. `estimar` devolvía
    /// `NoCabeEnElHorizonte { faltan }` desde la Fase 4 y su texto estaba
    /// escrito desde entonces, pero `activar_campana` no lo consultaba: se
    /// podía activar una campaña de medio millón de personas con 50 correos al
    /// día, y el motor enviaría correctamente durante años mientras cientos de
    /// miles de personas **no recibían nada nunca**, sin que nada fallara.
    ///
    /// La cifra que se comprueba es la del error, no sólo que falle: «no
    /// termina» sin decir cuánta gente se queda fuera no permite decidir si
    /// hay que subir el tope o partir la campaña (§94).
    ///
    /// **Verificada quitando el rechazo** —las tres ramas del `match` por un
    /// `let _ = estimacion;`—: la activación devuelve `Ok` y la prueba falla
    /// diciendo que esperaba `LaCampanaNoTermina`.
    #[test]
    fn una_campana_que_no_termina_en_cinco_anos_no_se_activa() {
        let (_d, db, version) = base();
        let campana = una_campana(&db, version);

        // ── POR QUÉ EL TOPE A UNO Y NO MEDIO MILLÓN DE CONTACTOS ──
        //
        // Lo que decide el veredicto es la aritmética: audiencia ÷ tope diario
        // contra los ~1 304 días laborables que caben en cinco años. Da igual
        // por qué lado se desequilibre.
        //
        // El primer intento fue al revés —60 000 contactos con el tope de la
        // cuenta en 50— y **la prueba falló sobre código correcto**: 60 000 ÷ 50
        // son 1 200 días, que **sí** caben. Estaba al filo y cayó del lado malo.
        // Con el tope a uno hacen falta 1 400 contactos en vez de 65 000, la
        // prueba corre en un segundo en vez de en cincuenta, y el margen es de
        // casi cien días en vez de cien días en contra.
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "UPDATE email_account SET daily_limit = 1 WHERE id = ?1",
                params![CUENTA],
            )
        })
        .expect("tope diario a uno");

        for i in 0..1_400 {
            contacto(&db, &format!("c-{i}"), &format!("p{i}@empresa.mx"));
        }

        match db.activar_campana(empresa(), campana) {
            Err(DbError::LaCampanaNoTermina { faltan }) => {
                assert!(
                    faltan > 0,
                    "el error tiene que decir CUÁNTA gente se queda fuera, no \
                     sólo que no termina"
                );
            }
            otro => panic!("se esperaba LaCampanaNoTermina y salió {otro:?}"),
        }
    }

    /// **Y no deja nada detrás: ni audiencia congelada, ni cola.**
    ///
    /// El rechazo ocurre con la audiencia ya escrita —contarla aparte sería
    /// otro recorrido completo del `JOIN`— así que lo que lo mantiene limpio es
    /// la transacción, no el orden. Esto lo comprueba: si alguien mueve el
    /// `return` fuera de la transacción, o confirma antes, quedaría una campaña
    /// en borrador con 1 400 filas de audiencia que nadie va a usar.
    #[test]
    fn la_campana_rechazada_no_deja_audiencia_ni_cola() {
        let (_d, db, version) = base();
        let campana = una_campana(&db, version);
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "UPDATE email_account SET daily_limit = 1 WHERE id = ?1",
                params![CUENTA],
            )
        })
        .expect("tope diario a uno");
        for i in 0..1_400 {
            contacto(&db, &format!("c-{i}"), &format!("p{i}@empresa.mx"));
        }

        let _ = db.activar_campana(empresa(), campana);

        assert_eq!(cuantos(&db, "campaign_audience", campana), 0);
        assert_eq!(cuantos(&db, "message_attempt", campana), 0);
        // Y sigue en borrador, no «enviando».
        let estado: String = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT status FROM campaign WHERE id = ?1",
                    params![campana.to_string()],
                    |f| f.get(0),
                )
            })
            .expect("estado");
        assert_eq!(estado, "draft");
    }

    /// **Con el tope a cero no sale ni un mensaje, y tampoco se activa.**
    ///
    /// Es el otro desenlace que el núcleo caza y nadie preguntaba. Una campaña
    /// «enviando» que nunca manda nada es la razón número uno por la que
    /// alguien cree que ARLES se rompió (7.3), y aquí se evita antes de llegar
    /// a esa pantalla.
    ///
    /// **Verificada quitando la rama `NoAvanza`**: devuelve `Ok` y la prueba
    /// falla.
    #[test]
    fn con_el_tope_a_cero_la_campana_no_se_activa() {
        let (_d, db, version) = base();
        contacto(&db, "c-1", "uno@empresa.mx");

        // El tope de la cuenta a cero. No se toca el de la etapa: lo que manda
        // es el más bajo de los dos, y así se comprueba además que se mira el
        // de la cuenta y no sólo el propio.
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "UPDATE email_account SET daily_limit = 0, hourly_limit = 0 WHERE id = ?1",
                params![CUENTA],
            )
        })
        .expect("tope a cero");

        let campana = una_campana(&db, version);
        assert!(matches!(
            db.activar_campana(empresa(), campana),
            Err(DbError::LaCampanaNoAvanza)
        ));
    }

    /// **Y la campaña que sí termina se activa, que es lo que no puede
    /// romperse al añadir un rechazo.**
    ///
    /// Sin esta prueba, un rechazo demasiado estricto —estimar con el tope de
    /// la etapa en vez del más bajo, o equivocar el horizonte— dejaría ARLES
    /// sin poder activar nada y las otras tres seguirían en verde.
    #[test]
    fn una_campana_que_termina_se_activa_igual_que_antes() {
        let (_d, db, version) = base();
        contacto(&db, "c-1", "uno@empresa.mx");
        contacto(&db, "c-2", "dos@empresa.mx");
        let campana = una_campana(&db, version);

        let activada = db.activar_campana(empresa(), campana).expect("activa");
        assert_eq!(activada.en_la_audiencia, 2);
    }

    fn cuantos(db: &Db, tabla: &str, campana: CampaignId) -> i64 {
        db.ejecutar_en_pruebas(|c| {
            c.query_row(
                &format!("SELECT count(*) FROM {tabla} WHERE campaign_id = ?1"),
                params![campana.to_string()],
                |f| f.get(0),
            )
        })
        .expect("cuenta")
    }

    /// **Más de 50 diarios exige aceptar el aviso, y deja rastro.**
    ///
    /// El §48 no pone un límite: pone una advertencia. ARLES no bloquea el
    /// volumen —§154 pide informar, no dictar—, pero **no deja crear la campaña
    /// sin que conste que se informó**, y la constancia va a `audit_log`, no a
    /// un campo que alguien pueda cambiar después.
    ///
    /// Es la diferencia entre «alguien decidió esto sabiendo el riesgo» y «no
    /// sabemos qué pasó», el día que el proveedor marque la cuenta como spam.
    ///
    /// Verificada rompiéndola el 29/09/2026: quitando la comprobación del
    /// `de_alto_volumen`, la campaña de 120 diarios se crea sin aceptación y
    /// esta prueba falla.
    /// **La firma que elige la campaña llega al correo** (H-12).
    ///
    /// `campaign_stage.signature_id` existía desde la V4 y **nada la
    /// escribía**: `redaccion_de_la_campana` ya la consultaba —con caída a la
    /// de por defecto— meses antes de que hubiera nada que consultar, así que
    /// todas las campañas salían con la misma firma y el asistente ni
    /// preguntaba. El patrón del §9 en su forma más silenciosa: nada falla.
    ///
    /// Va por el camino real, `crear_campana`, que es la mitad que la prueba
    /// de `plantillas.rs` no puede cubrir — allí la etapa se siembra con SQL a
    /// mano y romper esto la deja en verde (se probó).
    ///
    /// Verificada rompiéndola: quitando `nueva.firma` del `INSERT`, cae con
    /// «Atentamente, Ventas» en vez de «Un saludo, Soporte».
    #[test]
    fn la_firma_elegida_llega_hasta_el_correo() {
        let (_d, db, v) = base();
        db.crear_firma(
            empresa(),
            "La de siempre",
            "<p>Atentamente, Ventas</p>",
            true,
        )
        .expect("firma por defecto");
        let otra = db
            .crear_firma(
                empresa(),
                "La de soporte",
                "<p>Un saludo, Soporte</p>",
                false,
            )
            .expect("otra firma");

        let con_firma = db
            .crear_campana(
                empresa(),
                &CampanaNueva {
                    lista: None,
                    nombre: "Con firma propia",
                    sale_por: PorDondeSale::Correo {
                        cuenta: CUENTA.parse().expect("id"),
                        firma: Some(&otra.to_string()),
                        version_de_plantilla: v,
                    },
                    ventana: "vent-1",
                    limite_por_hora: None,
                    limite_diario: Some(40),
                    acepto_el_riesgo_del_volumen: false,
                },
            )
            .expect("crea")
            .id;

        assert_eq!(
            db.redaccion_de_la_campana(&con_firma.to_string())
                .expect("consulta")
                .expect("hay redacción")
                .firma_html
                .as_deref(),
            Some("<p>Un saludo, Soporte</p>"),
            "la firma elegida en el asistente no llegó al correo"
        );

        // Y sin elegir ninguna sigue ganando la de por defecto, que es lo que
        // promete su casilla en Plantillas.
        let sin_firma = db
            .crear_campana(
                empresa(),
                &CampanaNueva {
                    lista: None,
                    nombre: "Sin elegir",
                    sale_por: PorDondeSale::Correo {
                        cuenta: CUENTA.parse().expect("id"),
                        firma: None,
                        version_de_plantilla: v,
                    },
                    ventana: "vent-1",
                    limite_por_hora: None,
                    limite_diario: Some(40),
                    acepto_el_riesgo_del_volumen: false,
                },
            )
            .expect("crea")
            .id;
        assert_eq!(
            db.redaccion_de_la_campana(&sin_firma.to_string())
                .expect("consulta")
                .expect("hay redacción")
                .firma_html
                .as_deref(),
            Some("<p>Atentamente, Ventas</p>")
        );
    }

    #[test]
    fn mas_de_cincuenta_diarios_exige_aceptar_el_aviso() {
        let (_d, db, v) = base();

        let sin_aceptar = db.crear_campana(
            empresa(),
            &CampanaNueva {
                lista: None,
                nombre: "Deprisa",
                sale_por: PorDondeSale::Correo {
                    cuenta: CUENTA.parse().expect("id"),
                    version_de_plantilla: v,
                    firma: None,
                },
                ventana: "vent-1",
                limite_por_hora: None,
                limite_diario: Some(120),
                acepto_el_riesgo_del_volumen: false,
            },
        );
        assert!(
            matches!(sin_aceptar, Err(DbError::FaltaAceptarElVolumen)),
            "120 diarios pasan del aviso del §48"
        );
        assert_eq!(
            db.campanas(empresa()).expect("lista").len(),
            0,
            "y no quedó una campaña a medias"
        );

        // ── UN CERO NO CREA UNA CAMPAÑA MUDA (22/10/2026) ──────────────────
        //
        // Se activaría, pondría «en marcha» en la pantalla, y no mandaría nunca
        // nada: sin error que mirar y sin cola que explique la espera. Los dos
        // campos valen — con cero por hora tampoco sale nada aunque el diario
        // sea generoso.
        //
        // Se comprueba aquí y no sólo en el asistente porque la frontera no se
        // fía de la webview. Verificada rompiéndola: quitando la guarda de
        // `crear_campana`, las dos ramas devuelven `Ok` y esto cae.
        for (por_hora, diario) in [(None, Some(0)), (Some(0), Some(40))] {
            let en_cero = db.crear_campana(
                empresa(),
                &CampanaNueva {
                    lista: None,
                    nombre: "Muda",
                    sale_por: PorDondeSale::Correo {
                        cuenta: CUENTA.parse().expect("id"),
                        version_de_plantilla: v,
                        firma: None,
                    },
                    ventana: "vent-1",
                    limite_por_hora: por_hora,
                    limite_diario: diario,
                    acepto_el_riesgo_del_volumen: true,
                },
            );
            assert!(
                matches!(en_cero, Err(DbError::LimiteDeEnviosEnCero)),
                "con {por_hora:?} por hora y {diario:?} al día se creó una campaña que no envía"
            );
        }
        assert_eq!(
            db.campanas(empresa()).expect("lista").len(),
            0,
            "y tampoco quedó una campaña a medias"
        );

        let aceptando = db.crear_campana(
            empresa(),
            &CampanaNueva {
                lista: None,
                nombre: "Deprisa",
                sale_por: PorDondeSale::Correo {
                    cuenta: CUENTA.parse().expect("id"),
                    version_de_plantilla: v,
                    firma: None,
                },
                ventana: "vent-1",
                limite_por_hora: None,
                limite_diario: Some(120),
                acepto_el_riesgo_del_volumen: true,
            },
        );
        let id = aceptando.expect("con el aviso aceptado sí se crea").id;

        // Y queda escrito quién lo aceptó y cuándo.
        let lineas: i64 = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT count(*) FROM audit_log
                      WHERE entity_id = ?1 AND action = 'aceptar_riesgo_de_volumen'",
                    params![id.to_string()],
                    |f| f.get(0),
                )
            })
            .expect("bitácora");
        assert_eq!(lineas, 1, "la aceptación deja rastro");
    }

    /// **Justo 50 no avisa.** El umbral es «más de 50», y está en un solo sitio.
    ///
    /// Se comprueba porque un `>=` en vez de un `>` obligaría a aceptar un
    /// riesgo en el caso que el §48 pone como seguro, y nadie lo notaría salvo
    /// por la casilla de más.
    #[test]
    fn justo_cincuenta_no_pide_aceptar_nada() {
        let (_d, db, v) = base();
        db.crear_campana(
            empresa(),
            &CampanaNueva {
                lista: None,
                nombre: "Al filo",
                sale_por: PorDondeSale::Correo {
                    cuenta: CUENTA.parse().expect("id"),
                    version_de_plantilla: v,
                    firma: None,
                },
                ventana: "vent-1",
                limite_por_hora: None,
                limite_diario: Some(arles_core::ritmo::Limites::AVISO_DIARIO),
                acepto_el_riesgo_del_volumen: false,
            },
        )
        .expect("50 no pasa del aviso");
    }

    /// Una campaña nace en borrador, con su etapa y sin audiencia.
    #[test]
    fn una_campana_nace_en_borrador_con_su_etapa() {
        let (_d, db, v) = base();
        let c = una_campana(&db, v);

        let lista = db.campanas(empresa()).expect("lista");
        let guardada = lista.first().expect("hay una");
        assert_eq!(guardada.id, c);
        assert_eq!(guardada.estado, EstadoDeCampana::Borrador);
        assert_eq!(guardada.audiencia, 0, "en borrador no hay audiencia");
        assert_eq!(cuantos(&db, "campaign_stage", c), 1, "nace con su etapa");
    }

    /// **Activar congela la audiencia y llena la cola, las dos cosas.**
    ///
    /// Es el corazón de la entrega. Una sin la otra deja una campaña que dice
    /// «enviando» y no envía, o correos en cola de una campaña que dice
    /// «borrador» — las dos cosas, en silencio.
    #[test]
    fn activar_congela_la_audiencia_y_llena_la_cola() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");
        let c = una_campana(&db, v);

        let r = db.activar_campana(empresa(), c).expect("activa");
        assert_eq!(r.en_la_audiencia, 2);
        assert_eq!(r.fuera_por_baja, 0);

        assert_eq!(
            cuantos(&db, "campaign_audience", c),
            2,
            "audiencia congelada"
        );
        assert_eq!(cuantos(&db, "message_attempt", c), 2, "cola llena");

        let guardada = db.campanas(empresa()).expect("lista");
        assert_eq!(
            guardada.first().expect("hay una").estado,
            EstadoDeCampana::Enviando
        );
    }

    /// **Quien pidió la baja no entra en la cola, y se dice cuántos fueron.**
    ///
    /// La cifra importa: «se activó para 1 de 2 — uno había pedido no recibir»
    /// es algo con lo que se puede vivir. «Se activó» a secas deja a alguien
    /// preguntándose por qué el contador no coincide con su lista (§94).
    ///
    /// Verificada rompiéndola el 29/09/2026: quitando el `NOT EXISTS` del
    /// `INSERT`, esta prueba falla.
    #[test]
    fn quien_pidio_la_baja_no_entra_en_la_campana() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");
        suprimir(&db, "beto@x.mx");
        let c = una_campana(&db, v);

        let r = db.activar_campana(empresa(), c).expect("activa");
        assert_eq!(r.en_la_audiencia, 1);
        assert_eq!(r.fuera_por_baja, 1, "y se dice cuántos quedaron fuera");

        let en_cola: String = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row(
                    "SELECT contact_address FROM message_attempt WHERE campaign_id = ?1",
                    params![c.to_string()],
                    |f| f.get(0),
                )
            })
            .expect("lee");
        assert_eq!(en_cola, "ana@x.mx", "el suprimido no está en la cola");
    }

    /// **Sin nadie a quien escribir no se activa.**
    ///
    /// Es un error y no un éxito con cero: una campaña activada sin
    /// destinatarios se queda diciendo «enviando» para siempre y nada explica
    /// por qué el contador no sube.
    ///
    /// Y no deja rastro: la transacción revierte la audiencia que se hubiera
    /// escrito.
    ///
    /// Verificada rompiéndola el 29/09/2026: quitando el `if metidos == 0`,
    /// esta prueba falla.
    #[test]
    fn sin_nadie_a_quien_escribir_no_se_activa() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        suprimir(&db, "ana@x.mx");
        let c = una_campana(&db, v);

        let error = db.activar_campana(empresa(), c).expect_err("no hay nadie");
        assert_eq!(error.clave_i18n(), "error.db.audiencia_vacia");

        assert_eq!(cuantos(&db, "campaign_audience", c), 0, "no queda rastro");
        assert_eq!(cuantos(&db, "message_attempt", c), 0);
        assert_eq!(
            db.campanas(empresa())
                .expect("lista")
                .first()
                .expect("una")
                .estado,
            EstadoDeCampana::Borrador,
            "y sigue en borrador"
        );
    }

    /// **Activar dos veces no duplica la cola.**
    ///
    /// Es el peor fallo posible: cada fila de más es un correo de más a una
    /// persona real. El `UNIQUE` del esquema lo impediría, pero fallaría con un
    /// error de SQLite; aquí se dice lo que pasa.
    #[test]
    fn activar_dos_veces_no_duplica_nada() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        let c = una_campana(&db, v);

        db.activar_campana(empresa(), c).expect("primera");
        let error = db.activar_campana(empresa(), c).expect_err("la segunda no");
        assert_eq!(error.clave_i18n(), "error.db.dato_invalido");

        assert_eq!(
            cuantos(&db, "message_attempt", c),
            1,
            "una sola vez en la cola"
        );
    }

    /// **El esquema impide que dos contactos compartan una dirección.**
    ///
    /// ── LO QUE ESTA PRUEBA IBA A SER, Y LO QUE ES ──
    ///
    /// Se escribió para comprobar que una dirección repetida entra una sola vez
    /// en la audiencia — el caso de una importación con duplicados. **No se
    /// puede llegar a ese estado**: la V3 puso un
    /// `UNIQUE(company_id, channel, value_normalized)`, así que la segunda
    /// inserción falla antes.
    ///
    /// El `GROUP BY` de la consulta de activación se queda igualmente, como
    /// defensa en profundidad, y se dice aquí que **hoy no puede dispararse**:
    /// una defensa que nadie sabe que es redundante se convierte en la razón
    /// por la que alguien no busca el problema donde está.
    ///
    /// Lo que la prueba comprueba ahora es la garantía de verdad, que es más
    /// fuerte: la base no admite el duplicado.
    #[test]
    fn el_esquema_impide_dos_contactos_con_la_misma_direccion() {
        let (_d, db, _v) = base();
        contacto(&db, "k1", "ana@x.mx");

        let choque = db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO contact (id, company_id, status, created_at, updated_at)
                 VALUES ('k2', '{EMPRESA}', 'active',
                         '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');
                 INSERT INTO contact_channel (id, company_id, contact_id, channel,
                                              value_raw, value_normalized, status,
                                              created_at, updated_at)
                 VALUES ('ch-k2b', '{EMPRESA}', 'k2', 'email', 'Ana@X.MX', 'ana@x.mx',
                         'active', '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');"
            ))
        });

        assert!(
            choque.is_err(),
            "la base tiene que impedir que dos contactos compartan una dirección: \
             si no, la misma persona recibiría dos correos iguales"
        );
    }

    /// Un contacto archivado no recibe.
    #[test]
    fn un_contacto_archivado_no_entra() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch("UPDATE contact SET status = 'archived' WHERE id = 'k2'")
        })
        .expect("archiva");

        let c = una_campana(&db, v);
        assert_eq!(
            db.activar_campana(empresa(), c)
                .expect("activa")
                .en_la_audiencia,
            1
        );
    }

    /// **Un canal dado de baja no entra, aunque el contacto siga activo.**
    ///
    /// ── ESTA PRUEBA ENCONTRÓ UN FALLO DE VERDAD ──
    ///
    /// La primera versión de la consulta filtraba por contacto activo y por
    /// dirección no suprimida, y **se olvidaba del estado del canal**. Un
    /// contacto puede seguir siendo cliente y haber borrado uno de sus correos;
    /// escribir a una dirección que su dueño quitó es exactamente lo que ARLES
    /// promete no hacer, y no habría fallado nada.
    ///
    /// Verificada rompiéndola el 29/09/2026: quitando `ch.status = 'active'` de
    /// la consulta —que es como estaba escrita— esta prueba falla y las demás
    /// siguen pasando.
    #[test]
    fn un_canal_dado_de_baja_no_entra() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch("UPDATE contact_channel SET status = 'archived' WHERE id = 'ch-k2'")
        })
        .expect("da de baja el canal");

        let c = una_campana(&db, v);
        let r = db.activar_campana(empresa(), c).expect("activa");
        assert_eq!(r.en_la_audiencia, 1, "sólo el que sigue teniendo correo");

        let quien: String = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row(
                    "SELECT contact_address FROM message_attempt WHERE campaign_id = ?1",
                    params![c.to_string()],
                    |f| f.get(0),
                )
            })
            .expect("lee");
        assert_eq!(quien, "ana@x.mx");
    }

    /// **Una campaña con lista sólo alcanza a los de su lista (D-15).**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// LO QUE ESTO IMPIDE, Y NO ES UN DETALLE
    ///
    /// Hasta el 23/10/2026 **toda campaña iba a todo el padrón**: la consulta
    /// de la audiencia recorría `contact` por `company_id` y nada más. Mandar
    /// una campaña de videovigilancia a quien pidió una aplicación no es sólo
    /// ruido — es la vía más rápida a que marquen el correo como no deseado, y
    /// la tasa de quejas es la cifra que más pesa en la reputación del dominio.
    ///
    /// Se siembran **dos** contactos y sólo uno entra en la lista. Con uno no
    /// se prueba nada: la audiencia saldría de 1 con el filtro y sin él, que es
    /// el defecto de la prueba de contactos con un solo contacto que este
    /// proyecto ya documenta.
    ///
    /// Verificada rompiéndola: quitando el `EXISTS` de `contact_list_member`
    /// —que es como estaba escrita la consulta— sale
    /// `left: 2, right: 1` y nombra a `beto@x.mx`, que no estaba invitado.
    #[test]
    fn una_campana_con_lista_solo_alcanza_a_su_lista() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");

        let c = una_campana(&db, v);
        db.ejecutar_en_pruebas(|conn| {
            conn.execute_batch(&format!(
                "INSERT INTO contact_list (id, company_id, name, created_at, updated_at)
                 VALUES ('lst-1', '{e}', 'Videovigilancia', '{t}', '{t}');
                 INSERT INTO contact_list_member (list_id, contact_id, added_at)
                 VALUES ('lst-1', 'k1', '{t}');
                 UPDATE campaign SET contact_list_id = 'lst-1' WHERE id = '{c}';",
                e = empresa(),
                t = "2026-10-23T10:00:00Z",
                c = c,
            ))
        })
        .expect("siembra la lista");

        let r = db.activar_campana(empresa(), c).expect("activa");
        assert_eq!(r.en_la_audiencia, 1, "sólo quien está en la lista");

        let quien: String = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row(
                    "SELECT contact_address FROM message_attempt WHERE campaign_id = ?1",
                    params![c.to_string()],
                    |f| f.get(0),
                )
            })
            .expect("lee");
        assert_eq!(quien, "ana@x.mx", "beto@x.mx no estaba invitado");
    }

    /// **Con lista, una baja sigue mandando sobre la invitación.**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ ESTA PRUEBA EXISTE, HABIENDO YA `quien_pidio_la_baja_no_entra`
    ///
    /// Porque ésa corre **sin lista**, y desde el 23/10/2026 la consulta de la
    /// audiencia tiene dos orígenes: uno entra por `contact` y el otro por
    /// `contact_list_member`. Los filtros se escriben una sola vez —van en el
    /// trozo compartido— pero eso es una propiedad del código de hoy, no una
    /// garantía: el día que alguien separe las dos consultas, **la rama con
    /// lista podría perder el filtro de bajas y las demás pruebas seguirían en
    /// verde**. Es el hueco que vive entre dos caminos que cada uno pasa lo
    /// suyo.
    ///
    /// Y lo que estaría en juego no es una función: es escribirle a quien
    /// pidió no recibir, que es lo peor que ARLES puede hacer.
    ///
    /// Verificada rompiéndola: quitando el `NOT EXISTS` de supresión sale
    /// `left: 2, right: 1`.
    #[test]
    fn una_baja_manda_mas_que_estar_en_la_lista() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");

        let c = una_campana(&db, v);
        db.ejecutar_en_pruebas(|conn| {
            conn.execute_batch(&format!(
                "INSERT INTO contact_list (id, company_id, name, created_at, updated_at)
                 VALUES ('lst-1', '{e}', 'Videovigilancia', '{t}', '{t}');
                 INSERT INTO contact_list_member (list_id, contact_id, added_at)
                 VALUES ('lst-1', 'k1', '{t}'), ('lst-1', 'k2', '{t}');
                 UPDATE campaign SET contact_list_id = 'lst-1' WHERE id = '{c}';",
                e = empresa(),
                t = "2026-10-23T10:00:00Z",
                c = c,
            ))
        })
        .expect("siembra la lista con los dos");

        // Beto está invitado **y** pidió no recibir. Gana la baja.
        db.suprimir_direccion(
            empresa(),
            arles_core::Canal::Correo,
            "beto@x.mx",
            arles_core::MotivoDeSupresion::Baja,
            arles_core::OrigenDeSupresion::Usuario,
            None,
        )
        .expect("suprime");

        let r = db.activar_campana(empresa(), c).expect("activa");
        assert_eq!(
            r.en_la_audiencia, 1,
            "la baja saca a Beto de su propia lista"
        );
        assert_eq!(r.fuera_por_baja, 1, "y se cuenta, con la lista por delante");
    }

    /// **Y sin lista va a toda la empresa, que es lo que hacía siempre.**
    ///
    /// No es compatibilidad por inercia: un aviso que va a todo el padrón es
    /// una campaña legítima, y las ya creadas no tienen lista. Lo que no puede
    /// pasar es que sea **lo que ocurre por omisión sin que nadie lo elija**,
    /// y de eso se encarga la pantalla.
    ///
    /// Esta prueba es la pareja de la de arriba, y sin ella el filtro podría
    /// estar escrito al revés —sólo con lista se envía— sin que nada fallara.
    #[test]
    fn sin_lista_la_campana_va_a_toda_la_empresa() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");

        let c = una_campana(&db, v);
        let r = db.activar_campana(empresa(), c).expect("activa");
        assert_eq!(r.en_la_audiencia, 2, "sin lista entran los dos");
    }

    /// **Una lista vacía no es una campaña a todo el padrón.**
    ///
    /// Es el caso que convierte este filtro en un peligro si se escribe mal:
    /// alguien crea «Videovigilancia», no le mete a nadie todavía, y activa.
    /// Con un `EXISTS` mal puesto —o con el `SET NULL` que esta migración
    /// descartó— la campaña saldría **a los 450 000**.
    ///
    /// Lo correcto es `AudienciaVacia`, el mismo error que ya existe: dice que
    /// no hay a quién escribir y no deja nada detrás.
    #[test]
    fn una_lista_vacia_no_se_convierte_en_toda_la_empresa() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");

        let c = una_campana(&db, v);
        db.ejecutar_en_pruebas(|conn| {
            conn.execute_batch(&format!(
                "INSERT INTO contact_list (id, company_id, name, created_at, updated_at)
                 VALUES ('lst-1', '{e}', 'Vacía', '{t}', '{t}');
                 UPDATE campaign SET contact_list_id = 'lst-1' WHERE id = '{c}';",
                e = empresa(),
                t = "2026-10-23T10:00:00Z",
                c = c,
            ))
        })
        .expect("siembra la lista vacía");

        let r = db.activar_campana(empresa(), c);
        assert!(
            matches!(r, Err(DbError::AudienciaVacia)),
            "una lista vacía tiene que parar, no salir a todo el padrón: {r:?}"
        );
    }

    /// **Cada envío de la cola nace con su clave de idempotencia, y todas son
    /// distintas.**
    ///
    /// Se escribe **antes** de que nadie llame a un proveedor (ADR-0004): es lo
    /// único que permite reconocer después un envío ambiguo. Dos iguales harían
    /// que un reintento se confundiera con otro envío.
    #[test]
    fn cada_envio_nace_con_su_clave_de_idempotencia() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");
        let c = una_campana(&db, v);
        db.activar_campana(empresa(), c).expect("activa");

        let distintas: i64 = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row(
                    "SELECT count(DISTINCT idempotency_key) FROM message_attempt
                      WHERE campaign_id = ?1 AND idempotency_key <> ''",
                    params![c.to_string()],
                    |f| f.get(0),
                )
            })
            .expect("cuenta");
        assert_eq!(distintas, 2, "dos claves, y las dos distintas y no vacías");
    }

    /// La campaña de otra empresa no se puede activar desde aquí.
    #[test]
    fn no_se_puede_activar_la_campana_de_otra_empresa() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        let c = una_campana(&db, v);

        let otra: CompanyId = "22222222-2222-4222-8222-222222222222".parse().expect("id");
        let error = db.activar_campana(otra, c).expect_err("no es suya");
        assert_eq!(error.clave_i18n(), "error.db.dato_invalido");
        assert_eq!(cuantos(&db, "message_attempt", c), 0);
    }
    /// **MEDICIÓN PARA DIRECCIÓN (22/10/2026): el mismo contacto en dos
    /// campañas recibe DOS correos.**
    ///
    /// La pregunta era literal: «al colocar en dos campañas diferentes al mismo
    /// contacto y el mismo remitente en ambas, ¿envía el correo 2 veces?».
    ///
    /// La respuesta es sí, y esta prueba la fija para que nadie la tenga que
    /// volver a suponer. La audiencia se congela como «todos los contactos
    /// activos de la empresa menos las bajas», sin mirar qué otras campañas
    /// existen, y la clave de idempotencia es `IdempotencyKey::nueva()` — nace
    /// distinta en cada fila, así que protege contra reenviar DOS VECES EL MISMO
    /// intento (ADR-0004) y no contra dos intentos distintos a la misma persona.
    ///
    /// No es un defecto escondido: es lo que el producto hace hoy, y hace falta
    /// que esté medido antes de decidir si debe cambiar.
    #[test]
    fn el_mismo_contacto_en_dos_campanas_recibe_dos_correos() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");

        let mut campanas = Vec::new();
        for nombre in ["Primera", "Segunda"] {
            let c = db
                .crear_campana(
                    empresa(),
                    &CampanaNueva {
                        lista: None,
                        nombre,
                        sale_por: PorDondeSale::Correo {
                            cuenta: CUENTA.parse().expect("id"),
                            version_de_plantilla: v,
                            firma: None,
                        },
                        ventana: "vent-1",
                        limite_por_hora: None,
                        limite_diario: Some(50),
                        acepto_el_riesgo_del_volumen: false,
                    },
                )
                .expect("se crea")
                .id;
            db.activar_campana(empresa(), c).expect("activa");
            campanas.push(c);
        }

        // Lo que de verdad importa: cuántos envíos hay EN COLA para esa persona.
        let para_ana: i64 = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT count(*) FROM message_attempt WHERE contact_address = ?1",
                    params!["ana@x.mx"],
                    |f| f.get(0),
                )
            })
            .expect("cuenta");
        assert_eq!(
            para_ana, 2,
            "ana está en las dos campañas, así que tiene dos correos en cola"
        );

        // Y con claves de idempotencia distintas: el motor no los reconoce como
        // el mismo envío, y no hay nada que los junte.
        let claves: i64 = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT count(DISTINCT idempotency_key) FROM message_attempt
                      WHERE contact_address = ?1",
                    params!["ana@x.mx"],
                    |f| f.get(0),
                )
            })
            .expect("cuenta");
        assert_eq!(
            claves, 2,
            "dos claves distintas: la idempotencia no los une, y no está para eso"
        );

        // Las dos campañas comparten remitente, que es la otra mitad de la
        // pregunta: el mismo buzón manda los dos.
        assert_eq!(campanas.len(), 2);
    }
    /// **Una campaña cuyo correo no dice cómo darse de baja no se activa.**
    ///
    /// ── EL RIESGO QUE ESTO CIERRA (R-7, 22/10/2026) ──────────────────────
    ///
    /// La cabecera `List-Unsubscribe` sí se mandaba, y está bien hecha. Lo que
    /// no existía era una salida **en el cuerpo**, y nada la exigía: se podía
    /// crear, activar y mandar una campaña entera sin una línea que dijera cómo
    /// dejar de recibirla.
    ///
    /// La cabecera la leen los filtros; **la persona busca en el cuerpo**, y
    /// quien no la encuentra usa el botón de spam — la señal que más daño hace
    /// a la reputación del dominio.
    ///
    /// Esto **impide** en vez de avisar, al contrario que casi todo en ARLES, y
    /// la razón es de quién es el riesgo: un volumen alto lo paga quien envía y
    /// es su decisión; un correo sin salida lo paga quien lo recibe, que no
    /// está en la conversación.
    #[test]
    fn sin_salida_de_baja_la_campana_no_se_activa() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        let c = una_campana(&db, v);

        // Se le quita la salida a la versión que la campaña congeló.
        db.ejecutar_en_pruebas(|conn| {
            conn.execute(
                "UPDATE template_version SET body_html = '<p>Gracias</p>'",
                [],
            )
        })
        .expect("quita la salida");

        let r = db.activar_campana(empresa(), c);
        assert!(
            matches!(r, Err(DbError::SinSalidaDeBaja)),
            "se activó una campaña sin salida de baja: {r:?}"
        );

        // Y **no queda nada a medias**: ni audiencia congelada ni cola. Entero
        // o nada lo mantiene la transacción, no el orden de las comprobaciones.
        assert_eq!(cuantos(&db, "campaign_audience", c), 0, "sin audiencia");
        assert_eq!(cuantos(&db, "message_attempt", c), 0, "sin cola");
    }

    /// **Y basta con que la lleve la FIRMA.**
    ///
    /// Es la razón de que esto se compruebe al activar y no al guardar la
    /// plantilla: al guardarla no se sabe qué firma se va a usar. Exigirla en
    /// la plantilla obligaría a repetirla aunque la firma ya la traiga, y
    /// entonces cada correo diría dos veces lo mismo.
    #[test]
    fn la_salida_de_baja_puede_venir_de_la_firma() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        let c = una_campana(&db, v);

        // Sin salida en el cuerpo, y con salida en la firma de por defecto.
        db.ejecutar_en_pruebas(|conn| {
            conn.execute(
                "UPDATE template_version SET body_html = '<p>Gracias</p>'",
                [],
            )?;
            // Se INSERTA: la base de estas pruebas no siembra ninguna firma,
            // así que un `UPDATE` no habría tocado nada y la prueba habría
            // salido en rojo diciendo que la firma no vale — cuando lo que no
            // existía era la firma.
            conn.execute(
                "INSERT INTO signature
                     (id, company_id, name, body_html, body_text, is_default,
                      created_at, updated_at)
                 VALUES ('firma-1', ?1, 'Por defecto',
                         '<p>TELEMETRY · {{baja}}</p>', 'TELEMETRY', 1,
                         '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z')",
                params![empresa().to_string()],
            )
        })
        .expect("mueve la salida a la firma");

        db.activar_campana(empresa(), c)
            .expect("con la salida en la firma sí se activa");
    }
    // ══ D-WA · UNA CAMPAÑA DE WHATSAPP, DE PRINCIPIO A FIN (27/10/2026) ══
    //
    // Lo que estas pruebas existen para impedir está escrito en una sola
    // frase: hasta hoy `activar_campana` escribía `'email'` de literal en
    // CUATRO consultas —la etapa, la supresión, la audiencia y la cola—, así
    // que activar una campaña de WhatsApp **congelaba una audiencia de
    // correos**. No fallaba nada: el motor recorría la etapa (tramo 1), pedía
    // el contenido de WhatsApp, y se lo mandaba a direcciones de correo.

    const NUMERO: &str = "01900000-0000-7000-8000-0000000000b1";
    const PLANTILLA: &str = "01900000-0000-7000-8000-0000000000d1";

    /// Añade a la base un número de WhatsApp y una plantilla de Meta.
    ///
    /// `estado` es el de Meta: `approved`, `pending`, `rejected`… Entra como
    /// parámetro porque la diferencia entre aprobada y no aprobada es justo lo
    /// que una de estas pruebas mide.
    fn con_whatsapp(db: &Db, estado: &str) {
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO whatsapp_account (id, company_id, display_name, phone_e164,
                                               waba_id, phone_number_id, credential_ref,
                                               modo, calidad, limite_diario_meta,
                                               status, created_at, updated_at)
                 VALUES ('{NUMERO}', '{EMPRESA}', 'Ventas', '+528100000000',
                         'waba-1', 'pn-1', 'llavero://w', 'prospeccion_directa',
                         'green', 1000, 'connected',
                         '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');

                 INSERT INTO whatsapp_template (id, company_id, whatsapp_account_id,
                                                meta_name, language, status, body, slots,
                                                created_at, updated_at)
                 VALUES ('{PLANTILLA}', '{EMPRESA}', '{NUMERO}', 'bienvenida', 'es_MX',
                         '{estado}', 'Hola {{{{1}}}}, somos {{{{2}}}}.', '[]',
                         '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');"
            ))
        })
        .expect("siembra de whatsapp");
    }

    /// Y un contacto con número de móvil, que es el canal de WhatsApp.
    fn contacto_con_movil(db: &Db, id: &str, movil: &str) {
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT OR IGNORE INTO contact (id, company_id, status, created_at, updated_at)
                 VALUES ('{id}', '{EMPRESA}', 'active',
                         '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');
                 INSERT INTO contact_channel (id, company_id, contact_id, channel,
                                              value_raw, value_normalized, status,
                                              created_at, updated_at)
                 VALUES ('chw-{id}', '{EMPRESA}', '{id}', 'whatsapp', '{movil}', '{movil}',
                         'active', '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');"
            ))
        })
        .expect("contacto con móvil");
    }

    fn una_campana_de_whatsapp(db: &Db, con_plantilla: bool) -> CampaignId {
        db.crear_campana(
            empresa(),
            &CampanaNueva {
                lista: None,
                nombre: "WhatsApp de septiembre",
                sale_por: PorDondeSale::Whatsapp {
                    numero: NUMERO.parse().expect("id"),
                    plantilla_de_meta: if con_plantilla {
                        Some(PLANTILLA.parse().expect("id"))
                    } else {
                        None
                    },
                },
                ventana: "vent-1",
                limite_por_hora: None,
                limite_diario: None,
                acepto_el_riesgo_del_volumen: false,
            },
        )
        .expect("crea")
        .id
    }

    /// **La prueba que este tramo existía para tener.**
    ///
    /// Un contacto con correo y otro con móvil. La campaña es de WhatsApp, así
    /// que la audiencia congelada y la cola tienen que ser **del móvil**, con
    /// `channel = 'whatsapp'` y colgando del NÚMERO, no de una cuenta de
    /// correo.
    ///
    /// Verificada el 27/10/2026 con **siete** rupturas, las siete en rojo: los
    /// cuatro sitios donde estaba escrito `'email'` (la audiencia, el filtro
    /// del canal del contacto, la supresión y la cola), el número guardado en
    /// la columna de la cuenta de correo, y los dos topes de WhatsApp cayendo a
    /// cero. Es el fallo que no se ve mirando, porque las dos capas pasan lo
    /// suyo.
    ///
    /// Y una de las rupturas estaba mal elegida: poner el número en la columna
    /// del correo dejaba un `None` sin tipo y **no compilaba**. Una ruptura que
    /// no compila no prueba nada —lo que hay que ver caer es la prueba, no el
    /// compilador—, así que se rehízo con el tipo anotado.
    #[test]
    fn una_campana_de_whatsapp_congela_los_moviles_y_no_los_correos() {
        let (_d, db, _v) = base();
        con_whatsapp(&db, "approved");
        contacto(&db, "k-correo", "ana@x.mx");
        contacto_con_movil(&db, "k-movil", "+528111111111");

        let c = una_campana_de_whatsapp(&db, true);
        let hecho = db.activar_campana(empresa(), c).expect("se activa");

        assert_eq!(hecho.en_la_audiencia, 1, "sólo el del móvil");

        let (canal, direccion, de_correo, de_whatsapp): (
            String,
            String,
            Option<String>,
            Option<String>,
        ) = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row(
                    "SELECT channel, contact_address, email_account_id, whatsapp_account_id
                       FROM message_attempt WHERE campaign_id = ?1",
                    params![c.to_string()],
                    |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?, f.get(3)?)),
                )
            })
            .expect("una fila en la cola");

        assert_eq!(canal, "whatsapp", "el canal de la cola es el de la etapa");
        assert_eq!(direccion, "+528111111111", "y la dirección, el móvil");
        assert_eq!(
            de_correo, None,
            "un intento de WhatsApp no cuelga de una cuenta de correo"
        );
        assert_eq!(
            de_whatsapp.as_deref(),
            Some(NUMERO),
            "cuelga del número, que es quien lo manda"
        );
    }

    /// **Una baja de WhatsApp saca del WhatsApp, no del correo.**
    ///
    /// La supresión se indexa por (empresa, canal, dirección): la misma persona
    /// puede haber pedido no recibir correos y seguir queriendo el WhatsApp, o
    /// al revés. Con el canal cableado a `'email'`, una baja de WhatsApp no
    /// habría sacado a nadie de una campaña de WhatsApp — y es la promesa que
    /// más importa de las que hace ARLES.
    #[test]
    fn una_baja_de_whatsapp_no_entra_en_la_audiencia() {
        let (_d, db, _v) = base();
        con_whatsapp(&db, "approved");
        contacto_con_movil(&db, "k-1", "+528111111111");
        contacto_con_movil(&db, "k-2", "+528122222222");

        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO suppression_entry (id, company_id, channel,
                                                address_normalized, scope, reason,
                                                origin, created_at)
                 VALUES ('sup-wa', '{EMPRESA}', 'whatsapp', '+528111111111', 'global',
                         'unsubscribe', 'user', '2026-09-29T00:00:00Z');"
            ))
        })
        .expect("baja");

        let c = una_campana_de_whatsapp(&db, true);
        let hecho = db.activar_campana(empresa(), c).expect("se activa");

        assert_eq!(
            hecho.en_la_audiencia, 1,
            "entra sólo el que no pidió la baja"
        );
        assert_eq!(
            hecho.fuera_por_baja, 1,
            "y se dice cuántos se quedaron fuera"
        );
    }

    /// **Una etapa de WhatsApp sin plantilla no se activa.**
    ///
    /// Lo prometía el comentario de la V16 —«quien SÍ lo exige es
    /// `activar_campana`»— y no lo exigía nadie. Sin esto, la campaña se
    /// activaba, el motor la recorría, pedía el contenido, no había ninguno, y
    /// se quedaba **quieta para siempre diciendo «enviando»**.
    #[test]
    fn una_etapa_de_whatsapp_sin_plantilla_no_se_activa() {
        let (_d, db, _v) = base();
        con_whatsapp(&db, "approved");
        contacto_con_movil(&db, "k-1", "+528111111111");

        let c = una_campana_de_whatsapp(&db, false);
        assert!(
            matches!(
                db.activar_campana(empresa(), c),
                Err(DbError::SinPlantillaDeMeta)
            ),
            "sin plantilla elegida no se puede activar"
        );
    }

    /// **Ni una plantilla que Meta todavía no ha aprobado.**
    ///
    /// Es un error distinto del anterior porque manda a hacer otra cosa:
    /// esperar a Meta o elegir otra, no elegir ésta. Sin esta puerta se
    /// activaba la campaña entera y Meta la rechazaba **mensaje por mensaje**:
    /// el registro se llenaría de fallos idénticos y el número gastaría
    /// reputación por algo que se sabía antes de empezar.
    #[test]
    fn una_plantilla_que_meta_no_ha_aprobado_no_se_activa() {
        let (_d, db, _v) = base();
        con_whatsapp(&db, "pending");
        contacto_con_movil(&db, "k-1", "+528111111111");

        let c = una_campana_de_whatsapp(&db, true);
        assert!(
            matches!(
                db.activar_campana(empresa(), c),
                Err(DbError::LaPlantillaNoEstaAprobada)
            ),
            "en revisión no es aprobada"
        );
    }

    /// **Y la etapa guarda el número en SU columna, no en la del correo.**
    ///
    /// Lo exige un `CHECK` del esquema desde la V4, así que un `INSERT` mal
    /// hecho fallaría — pero fallaría con un error de SQLite en vez de decir
    /// qué pasa. Esta prueba comprueba la fila, que es lo que el motor lee.
    #[test]
    fn la_etapa_de_whatsapp_cuelga_del_numero_y_lleva_su_plantilla() {
        let (_d, db, _v) = base();
        con_whatsapp(&db, "approved");
        let c = una_campana_de_whatsapp(&db, true);

        // La fila, en una estructura y no en una tupla de seis: con la tupla
        // basta cambiar dos columnas de sitio en el `SELECT` para que la prueba
        // compruebe otra cosa sin fallar — y es justo el defecto del cuerpo del
        // respaldo que la 8.2b arregló declarando las columnas.
        struct Fila {
            canal: String,
            correo: Option<String>,
            numero: Option<String>,
            version: Option<String>,
            plantilla: Option<String>,
            firma: Option<String>,
        }

        let fila = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row(
                    "SELECT channel, email_account_id, whatsapp_account_id,
                            template_version_id, whatsapp_template_id, signature_id
                       FROM campaign_stage WHERE campaign_id = ?1",
                    params![c.to_string()],
                    |f| {
                        Ok(Fila {
                            canal: f.get(0)?,
                            correo: f.get(1)?,
                            numero: f.get(2)?,
                            version: f.get(3)?,
                            plantilla: f.get(4)?,
                            firma: f.get(5)?,
                        })
                    },
                )
            })
            .expect("la etapa");

        assert_eq!(fila.canal, "whatsapp");
        assert_eq!(fila.correo, None, "ninguna cuenta de correo");
        assert_eq!(fila.numero.as_deref(), Some(NUMERO));
        assert_eq!(fila.version, None, "ninguna plantilla de correo");
        assert_eq!(fila.plantilla.as_deref(), Some(PLANTILLA));
        assert_eq!(
            fila.firma, None,
            "una firma es cosa del correo, y la variante no deja escribirla"
        );
    }

    /// **La exigencia de la salida de baja NO se aplica en WhatsApp, y esta
    /// prueba lo deja escrito para que nadie lea el verde como que sí.**
    ///
    /// La plantilla de esta base lleva `{{baja}}` a propósito —seis pruebas de
    /// este módulo dependen de ello—, pero una campaña de WhatsApp **no usa
    /// ninguna plantilla de correo**: `redaccion_en` devuelve `None`, y hasta
    /// el 27/10/2026 eso hacía que la comprobación pasara en verde sin mirar
    /// nada. Ahora es un `match` explícito, y esto fija la rama.
    ///
    /// Por qué no se puede comprobar hoy: la salida de correo se exige por la
    /// variable `{{baja}}`, que ARLES **sustituye por una salida que
    /// funciona**. En una plantilla de Meta no hay nada que sustituir: el texto
    /// lo aprueba Meta palabra por palabra. Buscar la palabra «baja» en el
    /// cuerpo es lo que `tiene_salida_de_baja` descarta con su propio ejemplo
    /// —«este mes damos de baja los precios»—.
    ///
    /// **Decisión de Dirección del 27/10/2026: no se exige.** Se planteó
    /// pedir que la plantilla registrada llevara su propia salida —«responde
    /// BAJA»— como constancia, y se descartó porque ARLES **no podría
    /// atenderla**: no hay por dónde recibir un aviso de Meta (ver la cabecera
    /// de `arles_whatsapp::entrante`) y el texto de una respuesta se tira a
    /// propósito desde la D-7A, así que la palabra «BAJA» no se leería ni con
    /// transporte. Imprimir una salida que el producto no atiende es una
    /// promesa que no se cumple.
    ///
    /// La salida que sí se cumple es **a mano**: con Coexistencia la respuesta
    /// se lee en el teléfono, y Supresión acepta un número de WhatsApp. Está
    /// dicho con esas palabras en el manual y en la guía del canal.
    ///
    /// Esta prueba cambia de signo el día que exista el «relay ciego» **y** se
    /// revise la D-7A, que son dos decisiones y no una.
    #[test]
    fn hoy_una_campana_de_whatsapp_se_activa_sin_salida_de_baja_comprobada() {
        let (_d, db, _v) = base();
        con_whatsapp(&db, "approved");
        contacto_con_movil(&db, "k-1", "+528111111111");

        // La plantilla de Meta no dice nada de darse de baja, y se activa.
        let c = una_campana_de_whatsapp(&db, true);
        db.activar_campana(empresa(), c)
            .expect("hoy se activa: la exigencia del correo no se puede medir aquí");
    }
}
