//! Plantillas con versiones, y firmas.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! UNA VERSIÓN NO SE EDITA NUNCA. SE CREA OTRA.
//!
//! En este módulo **no hay ni un `UPDATE` sobre `template_version`**, y esa
//! ausencia es la regla entera.
//!
//! Una campaña no apunta a una plantilla: apunta a **una versión concreta**
//! (§44). Editar la plantilla después de activar no altera lo que ya salió ni
//! lo que está en cola, y eso es lo que permite contestar dentro de seis meses
//! a «¿qué decía exactamente el correo que le llegó a este cliente?».
//!
//! Sin esto, la pregunta no tiene respuesta: el texto se habría reescrito
//! encima, y el registro de envíos apuntaría a algo que ya no existe. No
//! fallaría nada — simplemente la campaña dejaría de ser auditable, que es la
//! mitad de lo que ARLES promete.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE SE GUARDA NO ES LO QUE LLEGÓ
//!
//! Dos cosas se recalculan aquí y **no se aceptan de quien llama**:
//!
//! **El HTML se sanea** antes de escribirlo, con la lista blanca del núcleo.
//! Quien llama es la frontera con la webview, y lo que la webview manda es un
//! dato de entrada — incluido el resultado de su propio saneado.
//!
//! **Las variables se leen del texto**, no se copian de un campo. Si se
//! aceptara la lista que manda la interfaz, una plantilla podría declarar que
//! usa `{{nombre}}` sin usarlo —o al revés— y el preflight contaría mal a
//! cuántos contactos les falta un dato.

use arles_core::ids::{CompanyId, SignatureId, TemplateId, TemplateVersionId};
use arles_core::plantilla::{Plantilla, SiFalta, Variable};
use arles_core::sanear;
use rusqlite::{OptionalExtension, params};

use crate::db::Db;
use crate::empresa::{ahora, auditar};
use crate::error::DbError;

/// Lo que alguien escribe al redactar un mensaje.
///
/// Llega **sin sanear y sin lista de variables**: las dos cosas las produce
/// este módulo. Ver la nota de la cabecera.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redaccion {
    pub asunto: String,
    pub cuerpo_html: String,
}

/// Una versión de plantilla tal como quedó guardada. **Inmutable.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionGuardada {
    pub id: TemplateVersionId,
    /// 1 la primera, 2 la siguiente. Se enseña: «versión 3» es una cifra (§94).
    pub numero: u32,
    pub asunto: String,
    /// Ya saneado. Lo que salga de aquí se puede mandar.
    pub cuerpo_html: String,
    /// La alternativa en texto plano, derivada del HTML.
    pub cuerpo_texto: String,
    /// Las variables que este texto usa de verdad, leídas de él.
    pub variables: Vec<Variable>,
}

/// Una plantilla con su versión vigente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantillaGuardada {
    pub id: TemplateId,
    pub nombre: String,
    /// La versión que se usaría hoy. `None` sólo en una fila corrupta: una
    /// plantilla se crea siempre con su primera versión, en la misma
    /// transacción.
    pub version_actual: Option<VersionGuardada>,
}

/// Una firma guardada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirmaGuardada {
    pub id: SignatureId,
    pub nombre: String,
    pub cuerpo_html: String,
    pub cuerpo_texto: String,
    pub por_defecto: bool,
}

/// Prepara una redacción para guardarla: saneada, con su texto plano y sus
/// variables leídas del contenido.
fn preparar(r: &Redaccion) -> (String, String, String, Vec<Variable>) {
    let html = sanear::html(&r.cuerpo_html);
    let texto = sanear::a_texto_plano(&html);

    // El asunto **no lleva HTML**: es una línea de texto, y sanearlo como HTML
    // dejaría entrar entidades que el cliente de correo enseñaría en crudo. Se
    // limpia de etiquetas pasándolo por el texto plano.
    let asunto = sanear::a_texto_plano(&r.asunto);

    // Las variables salen del asunto Y del cuerpo: una plantilla que sólo usa
    // `{{nombre}}` en el asunto sigue necesitando ese dato.
    let mut variables = Plantilla::nueva(&asunto, SiFalta::Nada).variables();
    for v in Plantilla::nueva(&texto, SiFalta::Nada).variables() {
        if !variables.contains(&v) {
            variables.push(v);
        }
    }

    (asunto, html, texto, variables)
}

/// Cómo se guarda la lista de variables en su columna.
///
/// Separadas por comas y por su clave, no por su nombre en Rust: la clave es lo
/// que aparece en el texto y lo que sobrevive a renombrar una variante.
fn variables_a_texto(vs: &[Variable]) -> String {
    vs.iter().map(|v| v.clave()).collect::<Vec<_>>().join(",")
}

/// Y cómo se vuelven a leer.
///
/// Una clave que ya no existe **se ignora en silencio**, y es deliberado: pasa
/// si una versión futura retira una variable, y una plantilla vieja que la usara
/// no puede impedir que se lea el resto. El texto seguiría enseñando el
/// `{{loquesea}}` en crudo, que es lo que el motor hace con lo desconocido.
fn variables_de_texto(s: &str) -> Vec<Variable> {
    s.split(',')
        .filter(|t| !t.is_empty())
        .filter_map(Variable::de_clave)
        .collect()
}

impl Db {
    /// Crea una plantilla con su **versión 1**, en una sola transacción.
    ///
    /// Las dos cosas juntas porque una plantilla sin versión no se puede usar
    /// para nada: sería una fila que la interfaz enseñaría como elegible y que
    /// reventaría al elegirla.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn crear_plantilla(
        &self,
        empresa: CompanyId,
        nombre: &str,
        redaccion: &Redaccion,
    ) -> Result<(TemplateId, TemplateVersionId), DbError> {
        let (asunto, html, texto, variables) = preparar(redaccion);
        let id = TemplateId::nuevo();
        let version = TemplateVersionId::nuevo();

        self.con(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();

            tx.execute(
                "INSERT INTO template (id, company_id, name, current_version_id,
                                       created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                params![
                    id.to_string(),
                    empresa.to_string(),
                    nombre.trim(),
                    version.to_string(),
                    cuando
                ],
            )?;

            tx.execute(
                "INSERT INTO template_version (id, template_id, version_number, subject,
                                               body_html, body_text, variables_used,
                                               created_at)
                 VALUES (?1, ?2, 1, ?3, ?4, ?5, ?6, ?7)",
                params![
                    version.to_string(),
                    id.to_string(),
                    asunto,
                    html,
                    texto,
                    variables_a_texto(&variables),
                    cuando
                ],
            )?;

            auditar(
                &tx,
                &empresa.to_string(),
                "crear",
                "template",
                &id.to_string(),
            )?;
            tx.commit()?;
            Ok((id, version))
        })
    }

    /// Guarda una redacción nueva como **la siguiente versión**.
    ///
    /// No toca la anterior. El número sale de contar las que hay, no de un
    /// contador aparte que pudiera desincronizarse.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla, o [`DbError::DatoInvalido`]
    /// si la plantilla no existe — un `INSERT` con una clave foránea rota
    /// fallaría, pero decirlo con el nombre del campo evita mandar a buscar el
    /// problema al disco.
    pub fn nueva_version_de_plantilla(
        &self,
        plantilla: TemplateId,
        redaccion: &Redaccion,
    ) -> Result<TemplateVersionId, DbError> {
        let (asunto, html, texto, variables) = preparar(redaccion);
        let version = TemplateVersionId::nuevo();

        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();

            let empresa: Option<String> = tx
                .query_row(
                    "SELECT company_id FROM template WHERE id = ?1",
                    params![plantilla.to_string()],
                    |f| f.get(0),
                )
                .optional()?;
            let Some(empresa) = empresa else {
                return Err(DbError::DatoInvalido {
                    motivo: "template.id",
                });
            };

            // El siguiente número sale de lo que hay. Un contador en `template`
            // podría quedarse atrás si alguien insertara una versión sin
            // tocarlo, y entonces habría dos «versión 3».
            let siguiente: i64 = tx.query_row(
                "SELECT COALESCE(MAX(version_number), 0) + 1
                   FROM template_version WHERE template_id = ?1",
                params![plantilla.to_string()],
                |f| f.get(0),
            )?;

            tx.execute(
                "INSERT INTO template_version (id, template_id, version_number, subject,
                                               body_html, body_text, variables_used,
                                               created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    version.to_string(),
                    plantilla.to_string(),
                    siguiente,
                    asunto,
                    html,
                    texto,
                    variables_a_texto(&variables),
                    cuando
                ],
            )?;

            // La plantilla pasa a apuntar a la nueva. Las campañas que ya
            // apuntaban a la anterior **no se tocan**: ésa es la regla.
            tx.execute(
                "UPDATE template SET current_version_id = ?2, updated_at = ?3
                  WHERE id = ?1",
                params![plantilla.to_string(), version.to_string(), cuando],
            )?;

            auditar(
                &tx,
                &empresa,
                "nueva_version",
                "template",
                &plantilla.to_string(),
            )?;
            tx.commit()?;
            Ok(version)
        })
    }

    /// Una versión concreta, tal como se guardó.
    ///
    /// Es lo que el motor pide al enviar: la campaña guarda el identificador de
    /// la versión, no el de la plantilla.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn version_de_plantilla(
        &self,
        version: TemplateVersionId,
    ) -> Result<Option<VersionGuardada>, DbError> {
        self.con(|conn| {
            conn.query_row(
                "SELECT id, version_number, subject, body_html, body_text, variables_used
                   FROM template_version WHERE id = ?1",
                params![version.to_string()],
                |f| {
                    Ok(VersionGuardada {
                        id: version,
                        numero: u32::try_from(f.get::<_, i64>(1)?).unwrap_or(0),
                        asunto: f.get(2)?,
                        cuerpo_html: f.get(3)?,
                        cuerpo_texto: f.get(4)?,
                        variables: variables_de_texto(&f.get::<_, String>(5)?),
                    })
                },
            )
            .optional()
        })
    }

    /// Las plantillas de una empresa, con su versión vigente.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn plantillas(&self, empresa: CompanyId) -> Result<Vec<PlantillaGuardada>, DbError> {
        self.con_dominio(|conn| {
            let mut c = conn.prepare(
                "SELECT t.id, t.name,
                        v.id, v.version_number, v.subject, v.body_html, v.body_text,
                        v.variables_used
                   FROM template t
                   LEFT JOIN template_version v ON v.id = t.current_version_id
                  WHERE t.company_id = ?1
                  ORDER BY t.updated_at DESC",
            )?;

            let filas = c.query_map(params![empresa.to_string()], |f| {
                Ok((
                    f.get::<_, String>(0)?,
                    f.get::<_, String>(1)?,
                    f.get::<_, Option<String>>(2)?,
                    f.get::<_, Option<i64>>(3)?,
                    f.get::<_, Option<String>>(4)?,
                    f.get::<_, Option<String>>(5)?,
                    f.get::<_, Option<String>>(6)?,
                    f.get::<_, Option<String>>(7)?,
                ))
            })?;

            let mut salida = Vec::new();
            for fila in filas {
                let (id, nombre, vid, num, asunto, html, texto, vars) = fila?;
                let id = id
                    .parse::<TemplateId>()
                    .map_err(|_| DbError::DatoInvalido {
                        motivo: "template.id",
                    })?;

                let version_actual = match (vid, num, asunto, html, texto, vars) {
                    (Some(vid), Some(num), Some(asunto), Some(html), Some(texto), Some(vars)) => {
                        Some(VersionGuardada {
                            id: vid.parse().map_err(|_| DbError::DatoInvalido {
                                motivo: "template_version.id",
                            })?,
                            numero: u32::try_from(num).unwrap_or(0),
                            asunto,
                            cuerpo_html: html,
                            cuerpo_texto: texto,
                            variables: variables_de_texto(&vars),
                        })
                    }
                    _ => None,
                };

                salida.push(PlantillaGuardada {
                    id,
                    nombre,
                    version_actual,
                });
            }
            Ok(salida)
        })
    }

    /// Guarda una firma.
    ///
    /// **Sólo puede haber una por defecto**, y se impone aquí: marcar una
    /// desmarca la anterior, en la misma transacción. Dos por defecto no
    /// fallarían — la interfaz elegiría una de las dos según el orden de la
    /// consulta, que puede cambiar entre versiones de SQLite.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn crear_firma(
        &self,
        empresa: CompanyId,
        nombre: &str,
        cuerpo_html: &str,
        por_defecto: bool,
    ) -> Result<SignatureId, DbError> {
        // ── LA GUARDA DE R-04, ANTES DE TOCAR LA BASE ─────────────────────
        //
        // Una firma con `{{nombre}}` pone el nombre de QUIEN RECIBE debajo del
        // mensaje, en cada correo de la campaña. Se para aquí y no sólo en la
        // pantalla porque una firma puede llegar por otro camino: escrita a
        // mano, desde un respaldo, o desde un formulario futuro que olvide
        // filtrar los botones.
        //
        // Va antes del saneado y de la transacción: no hay nada que deshacer.
        arles_core::plantilla::revisar_firma(cuerpo_html).map_err(DbError::Nucleo)?;

        let html = sanear::html(cuerpo_html);
        let texto = sanear::a_texto_plano(&html);
        let id = SignatureId::nuevo();

        self.con(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();

            if por_defecto {
                tx.execute(
                    "UPDATE signature SET is_default = 0, updated_at = ?2
                      WHERE company_id = ?1",
                    params![empresa.to_string(), cuando],
                )?;
            }

            tx.execute(
                "INSERT INTO signature (id, company_id, name, body_html, body_text,
                                        is_default, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
                params![
                    id.to_string(),
                    empresa.to_string(),
                    nombre.trim(),
                    html,
                    texto,
                    i32::from(por_defecto),
                    cuando
                ],
            )?;

            auditar(
                &tx,
                &empresa.to_string(),
                "crear",
                "signature",
                &id.to_string(),
            )?;
            tx.commit()?;
            Ok(id)
        })
    }

    /// Reescribe una firma **en su sitio**, y es la única cosa de este módulo
    /// que se edita encima.
    ///
    /// ── POR QUÉ AQUÍ SÍ Y EN UNA PLANTILLA NO ──────────────────────────────
    ///
    /// Una versión de plantilla no se toca porque es lo que contesta «¿qué
    /// decía el correo que recibió este cliente?». Una firma **no tiene
    /// versiones en el esquema**, así que esa pregunta no se le puede hacer, y
    /// un `UPDATE` no destruye ninguna respuesta que hoy exista.
    ///
    /// Y hoy, además, **ninguna campaña apunta a una firma**: la columna
    /// `campaign_stage.signature_id` existe desde la V4 y **nada la escribe**
    /// —medido en el 17/10/2026 sobre los ocho crates—, igual que el cuerpo de
    /// la plantilla no llega todavía al remitente (auditoría B). Así que
    /// editar una firma no puede cambiar lo que va a salir de una campaña en
    /// marcha, porque no sale ninguna firma.
    ///
    /// **El día que el contenido se conecte al envío, esto hay que decidirlo
    /// otra vez**, y la decisión correcta será la de la plantilla: congelar en
    /// la etapa lo que se va a mandar. Si en su lugar la etapa apunta a la
    /// firma viva, editarla a media corrida cambiará la firma de los que
    /// falten, y nada protestará. Está anotado como H-12.
    ///
    /// # Errores
    ///
    /// [`DbError::Nucleo`] si el cuerpo lleva variables de quien recibe (R-04),
    /// [`DbError::DatoInvalido`] si la firma no es de esta empresa — o no
    /// existe, que desde fuera es lo mismo— y [`DbError::Sqlite`] si la
    /// escritura falla.
    pub fn actualizar_firma(
        &self,
        empresa: CompanyId,
        firma: SignatureId,
        nombre: &str,
        cuerpo_html: &str,
        por_defecto: bool,
    ) -> Result<(), DbError> {
        // La misma guarda que al crear, y por el mismo motivo: editar es otro
        // camino hasta la base, y una guarda que sólo cubre uno de los dos no
        // cubre ninguno.
        arles_core::plantilla::revisar_firma(cuerpo_html).map_err(DbError::Nucleo)?;

        let html = sanear::html(cuerpo_html);
        let texto = sanear::a_texto_plano(&html);

        // `con_dominio` y no `con`: la decisión de abortar se toma **dentro**
        // de la transacción, cuando ya se sabe de quién es la firma.
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();

            // Se comprueba la pertenencia **dentro** de la transacción. Un
            // `UPDATE … WHERE id = ?1 AND company_id = ?2` que no encaja
            // devuelve 0 filas y ningún error: la pantalla diría «guardado»
            // sobre una firma de otra empresa que no se tocó.
            let suya: Option<i64> = tx
                .query_row(
                    "SELECT 1 FROM signature WHERE id = ?1 AND company_id = ?2",
                    params![firma.to_string(), empresa.to_string()],
                    |f| f.get(0),
                )
                .optional()?;
            if suya.is_none() {
                return Err(DbError::DatoInvalido {
                    motivo: "signature.id",
                });
            }

            if por_defecto {
                tx.execute(
                    "UPDATE signature SET is_default = 0, updated_at = ?2
                      WHERE company_id = ?1",
                    params![empresa.to_string(), cuando],
                )?;
            }

            tx.execute(
                "UPDATE signature
                    SET name = ?2, body_html = ?3, body_text = ?4,
                        is_default = ?5, updated_at = ?6
                  WHERE id = ?1",
                params![
                    firma.to_string(),
                    nombre.trim(),
                    html,
                    texto,
                    i32::from(por_defecto),
                    cuando
                ],
            )?;

            auditar(
                &tx,
                &empresa.to_string(),
                "editar",
                "signature",
                &firma.to_string(),
            )?;
            tx.commit()?;
            Ok(())
        })
    }

    /// Borra una plantilla con todas sus versiones.
    ///
    /// ── SE NIEGA SI ALGUNA CAMPAÑA LA USÓ, Y ESO ES LO IMPORTANTE ─────────
    ///
    /// Una campaña apunta a la **versión exacta** que envió (§44). Borrarla
    /// dejaría el registro de envíos señalando un texto que ya no existe, y la
    /// pregunta «¿qué decía el correo que recibió este cliente?» se quedaría
    /// sin respuesta.
    ///
    /// El esquema ya lo impide —`ON DELETE RESTRICT`— pero fallar por una
    /// clave foránea es, para quien pulsó, «algo falló». Aquí se cuenta antes
    /// y se dice con su cifra (§94).
    ///
    /// **Cuenta campañas y no etapas**: dos etapas de la misma campaña son una
    /// sola cosa para quien lee el aviso.
    ///
    /// # Errores
    ///
    /// [`DbError::LaPlantillaEstaEnUso`] con cuántas campañas la usan,
    /// [`DbError::DatoInvalido`] si no es de esta empresa, y
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn borrar_plantilla(
        &self,
        empresa: CompanyId,
        plantilla: TemplateId,
    ) -> Result<(), DbError> {
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;

            let suya: Option<i64> = tx
                .query_row(
                    "SELECT 1 FROM template WHERE id = ?1 AND company_id = ?2",
                    params![plantilla.to_string(), empresa.to_string()],
                    |f| f.get(0),
                )
                .optional()?;
            if suya.is_none() {
                return Err(DbError::DatoInvalido {
                    motivo: "template.id",
                });
            }

            let campanas: i64 = tx.query_row(
                "SELECT COUNT(DISTINCT cs.campaign_id)
                   FROM campaign_stage cs
                   JOIN template_version tv ON tv.id = cs.template_version_id
                  WHERE tv.template_id = ?1",
                params![plantilla.to_string()],
                |f| f.get(0),
            )?;
            if campanas > 0 {
                return Err(DbError::LaPlantillaEstaEnUso {
                    campanas: u32::try_from(campanas).unwrap_or(u32::MAX),
                });
            }

            // Las versiones caen con ella: `template_version.template_id` es
            // `ON DELETE CASCADE`. No hace falta borrarlas a mano, y hacerlo
            // sería una segunda copia de esa regla.
            tx.execute(
                "DELETE FROM template WHERE id = ?1",
                params![plantilla.to_string()],
            )?;

            auditar(
                &tx,
                &empresa.to_string(),
                "borrar",
                "template",
                &plantilla.to_string(),
            )?;
            tx.commit()?;
            Ok(())
        })
    }

    /// Borra una firma.
    ///
    /// ── AQUÍ NO HAY NADA QUE PROTEGER, Y POR ESO NO SE NIEGA ──────────────
    ///
    /// A diferencia de una plantilla, una firma **no es lo que contesta «qué
    /// decía el correo»**: no tiene versiones y hoy ninguna campaña apunta a
    /// una (H-12). `campaign_stage.signature_id` es además `ON DELETE SET
    /// NULL`, así que borrarla no rompe ninguna campaña.
    ///
    /// Lo que sí puede pasar es que fuera **la de por defecto**, y entonces
    /// las campañas siguientes saldrían sin firma. Eso lo avisa la pantalla
    /// antes de confirmar; aquí no se impide, porque es una decisión legítima.
    ///
    /// # Errores
    ///
    /// [`DbError::DatoInvalido`] si no es de esta empresa, y
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn borrar_firma(&self, empresa: CompanyId, firma: SignatureId) -> Result<(), DbError> {
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;

            let suya: Option<i64> = tx
                .query_row(
                    "SELECT 1 FROM signature WHERE id = ?1 AND company_id = ?2",
                    params![firma.to_string(), empresa.to_string()],
                    |f| f.get(0),
                )
                .optional()?;
            if suya.is_none() {
                return Err(DbError::DatoInvalido {
                    motivo: "signature.id",
                });
            }

            tx.execute(
                "DELETE FROM signature WHERE id = ?1",
                params![firma.to_string()],
            )?;

            auditar(
                &tx,
                &empresa.to_string(),
                "borrar",
                "signature",
                &firma.to_string(),
            )?;
            tx.commit()?;
            Ok(())
        })
    }

    /// Las firmas de una empresa. La de por defecto, primero.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn firmas(&self, empresa: CompanyId) -> Result<Vec<FirmaGuardada>, DbError> {
        self.con_dominio(|conn| {
            let mut c = conn.prepare(
                "SELECT id, name, body_html, body_text, is_default
                   FROM signature WHERE company_id = ?1
                  ORDER BY is_default DESC, name",
            )?;
            let filas = c.query_map(params![empresa.to_string()], |f| {
                Ok((
                    f.get::<_, String>(0)?,
                    f.get::<_, String>(1)?,
                    f.get::<_, String>(2)?,
                    f.get::<_, String>(3)?,
                    f.get::<_, i64>(4)?,
                ))
            })?;

            let mut salida = Vec::new();
            for fila in filas {
                let (id, nombre, html, texto, def) = fila?;
                salida.push(FirmaGuardada {
                    id: id.parse().map_err(|_| DbError::DatoInvalido {
                        motivo: "signature.id",
                    })?,
                    nombre,
                    cuerpo_html: html,
                    cuerpo_texto: texto,
                    por_defecto: def != 0,
                });
            }
            Ok(salida)
        })
    }
}

/// Lo que una campaña manda, antes de ponerle los datos de cada persona.
///
/// Es **constante durante toda la campaña**: se lee una vez al arrancar el
/// motor y no en cada envío. Lo que cambia por persona son los valores de las
/// variables, y ésos vienen con el trabajo de la cola.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedaccionDeLaCampana {
    /// El asunto de la versión de plantilla **que la campaña congeló**.
    pub asunto: String,
    pub cuerpo_html: String,
    /// La firma que se pega al final, si hay alguna.
    pub firma_html: Option<String>,
    /// El nombre comercial de la empresa: es `{{mi_empresa}}`.
    pub mi_empresa: String,
}

impl Db {
    /// Qué manda esta campaña: su plantilla congelada, su firma y su empresa.
    ///
    /// ── LA VERSIÓN ES LA QUE LA CAMPAÑA ELIGIÓ, NO LA VIGENTE ──────────────
    ///
    /// Se lee de `campaign_stage.template_version_id`, que apunta a **una
    /// versión concreta** (§44). Leer la vigente de la plantilla haría que
    /// corregir el texto cambiara lo que está saliendo de una campaña ya
    /// activa — que es exactamente lo que el versionado existe para evitar, y
    /// lo que la pantalla promete que no pasa.
    ///
    /// ── LA FIRMA ES LA DE POR DEFECTO, Y ESO ES UNA LIMITACIÓN ─────────────
    ///
    /// `campaign_stage.signature_id` existe desde la V4 y **nada lo escribe**
    /// (H-12): el asistente no pregunta por una firma. Así que se usa la que
    /// la empresa marcó por defecto, que es literalmente lo que su casilla
    /// promete: «usar esta firma cuando no se elija otra». Hoy nunca se elige
    /// otra.
    ///
    /// El día que el asistente la pregunte, esta consulta mira primero
    /// `signature_id` y cae en la de por defecto — y entonces habrá que
    /// decidir si la etapa congela una copia, como hace con la plantilla.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla. Devuelve `None` si la campaña
    /// no tiene etapa de correo o su etapa no tiene plantilla, que es un
    /// estado que `crear_campana` no produce.
    pub fn redaccion_de_la_campana(
        &self,
        campana: &str,
    ) -> Result<Option<RedaccionDeLaCampana>, DbError> {
        self.con(|conn| {
            conn.query_row(
                "SELECT tv.subject, tv.body_html, co.commercial_name,
                        COALESCE(
                            (SELECT s.body_html FROM signature s
                              WHERE s.id = cs.signature_id),
                            (SELECT s.body_html FROM signature s
                              WHERE s.company_id = co.id AND s.is_default = 1
                              LIMIT 1)
                        )
                   FROM campaign_stage cs
                   JOIN campaign ca ON ca.id = cs.campaign_id
                   JOIN company co ON co.id = ca.company_id
                   JOIN template_version tv ON tv.id = cs.template_version_id
                  WHERE cs.campaign_id = ?1 AND cs.channel = 'email'
                  ORDER BY cs.position
                  LIMIT 1",
                params![campana],
                |f| {
                    Ok(RedaccionDeLaCampana {
                        asunto: f.get(0)?,
                        cuerpo_html: f.get(1)?,
                        mi_empresa: f.get(2)?,
                        // Una firma vacía y «no hay firma» son lo mismo para
                        // quien la pega, así que se unifican aquí en vez de
                        // dejar que cada lado lo decida.
                        firma_html: f
                            .get::<_, Option<String>>(3)?
                            .filter(|h| !h.trim().is_empty()),
                    })
                },
            )
            .optional()
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

    fn redaccion(asunto: &str, html: &str) -> Redaccion {
        Redaccion {
            asunto: asunto.to_owned(),
            cuerpo_html: html.to_owned(),
        }
    }

    /// Una plantilla nace con su versión 1, y se lee entera.
    #[test]
    fn una_plantilla_nace_con_su_primera_version() {
        let (_d, db) = base();
        let (_, v) = db
            .crear_plantilla(
                empresa(),
                "Bienvenida",
                &redaccion("Hola {{nombre}}", "<p>Gracias, {{nombre}}</p>"),
            )
            .expect("crea");

        let version = db
            .version_de_plantilla(v)
            .expect("consulta")
            .expect("existe");
        assert_eq!(version.numero, 1);
        assert_eq!(version.asunto, "Hola {{nombre}}");
        assert!(version.cuerpo_html.contains("<p>"));
        assert_eq!(version.variables, vec![Variable::Nombre]);
    }

    /// **Guardar otra versión no toca la anterior.**
    ///
    /// Es la regla del módulo. Una campaña apunta a una versión concreta, y
    /// editar la plantilla después no puede cambiar lo que ya salió — si
    /// pudiera, «¿qué decía el correo que le llegó a este cliente?» dejaría de
    /// tener respuesta, sin que nada fallara.
    ///
    /// Verificada rompiéndola el 29/09/2026: cambiando el `INSERT` de
    /// `nueva_version_de_plantilla` por un `UPDATE` sobre la versión vigente,
    /// esta prueba falla.
    #[test]
    fn una_version_nueva_no_altera_la_anterior() {
        let (_d, db) = base();
        let (p, v1) = db
            .crear_plantilla(empresa(), "Bienvenida", &redaccion("Uno", "<p>Uno</p>"))
            .expect("crea");

        let v2 = db
            .nueva_version_de_plantilla(p, &redaccion("Dos", "<p>Dos</p>"))
            .expect("segunda versión");

        let vieja = db.version_de_plantilla(v1).expect("c").expect("sigue ahí");
        assert_eq!(vieja.asunto, "Uno", "la versión 1 no se toca");
        assert_eq!(vieja.numero, 1);

        let nueva = db.version_de_plantilla(v2).expect("c").expect("existe");
        assert_eq!(nueva.asunto, "Dos");
        assert_eq!(nueva.numero, 2);
    }

    /// **El HTML se sanea al guardar.**
    ///
    /// Quien llama es la frontera con la webview, y lo que la webview manda es
    /// un dato de entrada — incluido el resultado de su propio saneado.
    #[test]
    fn el_html_se_sanea_antes_de_guardarlo() {
        let (_d, db) = base();
        let (_, v) = db
            .crear_plantilla(
                empresa(),
                "Con script",
                &redaccion(
                    "Asunto",
                    r#"<p onclick="alert(1)">Hola</p><script>x</script>"#,
                ),
            )
            .expect("crea");

        let guardada = db.version_de_plantilla(v).expect("c").expect("existe");
        assert!(
            !guardada.cuerpo_html.contains("script"),
            "{}",
            guardada.cuerpo_html
        );
        assert!(
            !guardada.cuerpo_html.contains("onclick"),
            "{}",
            guardada.cuerpo_html
        );
        assert!(guardada.cuerpo_html.contains("Hola"));
    }

    /// **Y el asunto no lleva HTML.**
    ///
    /// Es una línea de texto. Un asunto con etiquetas se enseña en crudo en el
    /// cliente de correo, con las etiquetas a la vista.
    #[test]
    fn el_asunto_sale_sin_etiquetas() {
        let (_d, db) = base();
        let (_, v) = db
            .crear_plantilla(
                empresa(),
                "X",
                &redaccion("<b>Oferta</b> <script>x</script>", "<p>Hola</p>"),
            )
            .expect("crea");

        let g = db.version_de_plantilla(v).expect("c").expect("existe");
        assert!(!g.asunto.contains('<'), "quedó una etiqueta: {}", g.asunto);
        assert!(g.asunto.contains("Oferta"));
    }

    /// **Las variables se leen del texto, no se copian de un campo.**
    ///
    /// Si se aceptara la lista que manda la interfaz, una plantilla podría
    /// declarar que usa `{{nombre}}` sin usarlo —o al revés— y el preflight
    /// contaría mal a cuántos contactos les falta un dato.
    ///
    /// Se comprueba además que una variable que sólo está en el **asunto**
    /// cuenta: una plantilla que personaliza sólo el asunto sigue necesitando
    /// ese dato.
    #[test]
    fn las_variables_se_leen_del_contenido_incluido_el_asunto() {
        let (_d, db) = base();
        let (_, v) = db
            .crear_plantilla(
                empresa(),
                "X",
                &redaccion("Para {{empresa}}", "<p>Hola {{nombre}}</p>"),
            )
            .expect("crea");

        let g = db.version_de_plantilla(v).expect("c").expect("existe");
        assert!(g.variables.contains(&Variable::Empresa), "la del asunto");
        assert!(g.variables.contains(&Variable::Nombre), "la del cuerpo");
        assert_eq!(g.variables.len(), 2);
    }

    /// Se guarda también la versión en texto plano, derivada del HTML.
    #[test]
    fn se_guarda_la_version_en_texto_plano() {
        let (_d, db) = base();
        let (_, v) = db
            .crear_plantilla(empresa(), "X", &redaccion("A", "<p>Hola</p><p>Adiós</p>"))
            .expect("crea");

        let g = db.version_de_plantilla(v).expect("c").expect("existe");
        assert_eq!(g.cuerpo_texto, "Hola\n\nAdiós");
        assert!(!g.cuerpo_texto.contains('<'));
    }

    /// La plantilla apunta a su versión vigente, y se lista con ella.
    #[test]
    fn la_plantilla_se_lista_con_su_version_vigente() {
        let (_d, db) = base();
        let (p, _) = db
            .crear_plantilla(empresa(), "Bienvenida", &redaccion("Uno", "<p>Uno</p>"))
            .expect("crea");
        db.nueva_version_de_plantilla(p, &redaccion("Dos", "<p>Dos</p>"))
            .expect("segunda");

        let lista = db.plantillas(empresa()).expect("lista");
        assert_eq!(lista.len(), 1);
        let v = lista
            .first()
            .and_then(|p| p.version_actual.as_ref())
            .expect("tiene versión");
        assert_eq!(v.numero, 2, "la vigente es la última");
        assert_eq!(v.asunto, "Dos");
    }

    /// Versionar una plantilla que no existe se dice, no se traga.
    #[test]
    fn versionar_una_plantilla_que_no_existe_da_error() {
        let (_d, db) = base();
        let error = db
            .nueva_version_de_plantilla(TemplateId::nuevo(), &redaccion("X", "<p>x</p>"))
            .expect_err("no existe");
        assert_eq!(error.clave_i18n(), "error.db.dato_invalido");
    }

    /// **Sólo puede haber una firma por defecto.**
    ///
    /// Dos no fallarían: la interfaz elegiría una de las dos según el orden de
    /// la consulta, que puede cambiar entre versiones de SQLite. Un correo
    /// firmado con la firma de otro departamento no da ningún error.
    ///
    /// Verificada rompiéndola el 29/09/2026: quitando el `UPDATE` que desmarca
    /// la anterior, esta prueba falla.
    #[test]
    fn marcar_una_firma_por_defecto_desmarca_la_anterior() {
        let (_d, db) = base();
        db.crear_firma(empresa(), "Ventas", "<p>Ventas</p>", true)
            .expect("primera");
        db.crear_firma(empresa(), "Soporte", "<p>Soporte</p>", true)
            .expect("segunda");

        let firmas = db.firmas(empresa()).expect("lista");
        let por_defecto: Vec<_> = firmas.iter().filter(|f| f.por_defecto).collect();
        assert_eq!(por_defecto.len(), 1, "sólo una: {firmas:?}");
        assert_eq!(
            por_defecto.first().expect("hay una").nombre,
            "Soporte",
            "la última marcada"
        );
    }

    /// Y la firma también se sanea.
    #[test]
    fn la_firma_se_sanea_igual_que_la_plantilla() {
        let (_d, db) = base();
        db.crear_firma(empresa(), "Mala", r#"<p>Ana</p><script>x</script>"#, false)
            .expect("crea");

        let firmas = db.firmas(empresa()).expect("lista");
        let f = firmas.first().expect("hay una firma");
        assert!(!f.cuerpo_html.contains("script"), "{}", f.cuerpo_html);
        assert!(f.cuerpo_html.contains("Ana"));
        assert_eq!(f.cuerpo_texto, "Ana");
    }

    /// Las plantillas de otra empresa no se mezclan.
    #[test]
    fn las_plantillas_de_otra_empresa_no_aparecen() {
        let (_d, db) = base();
        let otra = "22222222-2222-4222-8222-222222222222";
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('{otra}', 'OTRA', 'MX', 'America/Monterrey',
                         'h@o.mx', '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');"
            ))
        })
        .expect("segunda empresa");

        db.crear_plantilla(empresa(), "Mía", &redaccion("A", "<p>a</p>"))
            .expect("crea");

        let suyas = db
            .plantillas(otra.parse().expect("id"))
            .expect("lista de la otra");
        assert!(
            suyas.is_empty(),
            "no puede ver la plantilla de otra empresa"
        );
        assert_eq!(db.plantillas(empresa()).expect("lista").len(), 1);
    }

    // ── R-06 · EDITAR UNA FIRMA ───────────────────────────────────────────
    //
    // Dirección no podía arreglar una firma mal escrita: se creaban y no se
    // tocaban. Las cuatro pruebas siguientes son las cuatro cosas que editar
    // puede hacer mal, y ninguna de ellas falla por sí sola en la base.
    //
    // **Las cuatro se vieron en rojo el 17/10/2026**, con una ruptura cada una
    // y cada una cazada sólo por la suya: que el `UPDATE` no escribiera el
    // cuerpo, quitar el desmarcado del por defecto, quitar la guarda de R-04,
    // y quitar la comprobación de a qué empresa pertenece la firma.

    /// Lo que se edita se guarda, y **no nace una firma nueva**.
    ///
    /// La segunda mitad es la que importa: un `INSERT` en vez de un `UPDATE`
    /// dejaría la firma arreglada en la lista junto a la mala, y quien la use
    /// después elegiría entre dos cosas con el mismo nombre.
    #[test]
    fn editar_una_firma_la_reescribe_y_no_crea_otra() {
        let (_d, db) = base();
        let id = db
            .crear_firma(empresa(), "Ventas", "<p>Con una errata</p>", false)
            .expect("crea");

        db.actualizar_firma(empresa(), id, "Ventas", "<p>Ya sin errata</p>", true)
            .expect("edita");

        let firmas = db.firmas(empresa()).expect("lista");
        assert_eq!(firmas.len(), 1, "sigue habiendo una sola: {firmas:?}");
        let f = firmas.first().expect("hay una");
        assert_eq!(f.id, id, "la misma firma, no otra");
        assert!(f.cuerpo_html.contains("Ya sin errata"), "{}", f.cuerpo_html);
        assert!(f.por_defecto, "y el por defecto se aplicó");
    }

    /// Marcar por defecto **al editar** desmarca la anterior, igual que al
    /// crear.
    ///
    /// Es la mitad que se olvida: la regla estaba escrita en `crear_firma`, y
    /// un camino nuevo hasta la misma columna es un camino nuevo para
    /// romperla. Dos firmas por defecto no dan ningún error.
    #[test]
    fn editar_marcando_por_defecto_desmarca_la_anterior() {
        let (_d, db) = base();
        db.crear_firma(empresa(), "Ventas", "<p>Ventas</p>", true)
            .expect("primera");
        let soporte = db
            .crear_firma(empresa(), "Soporte", "<p>Soporte</p>", false)
            .expect("segunda");

        db.actualizar_firma(empresa(), soporte, "Soporte", "<p>Soporte</p>", true)
            .expect("edita");

        let firmas = db.firmas(empresa()).expect("lista");
        let por_defecto: Vec<_> = firmas.iter().filter(|f| f.por_defecto).collect();
        assert_eq!(por_defecto.len(), 1, "sólo una: {firmas:?}");
        assert_eq!(por_defecto.first().expect("hay una").nombre, "Soporte");
    }

    /// La guarda de R-04 también cubre la edición.
    ///
    /// Sin esto, el camino para firmar con el nombre de quien recibe seguiría
    /// abierto: se crea una firma limpia y se edita después.
    #[test]
    fn editar_no_deja_meter_variables_del_contacto() {
        let (_d, db) = base();
        let id = db
            .crear_firma(empresa(), "Ventas", "<p>Ventas</p>", false)
            .expect("crea");

        let error = db
            .actualizar_firma(empresa(), id, "Ventas", "<p>Atte. {{nombre}}</p>", false)
            .expect_err("tiene que negarse");
        assert_eq!(error.clave_i18n(), "error.firma_con_variables_del_contacto");

        let f = db.firmas(empresa()).expect("lista");
        let f = f.first().expect("sigue ahí");
        assert!(
            !f.cuerpo_html.contains("nombre"),
            "y la firma buena no se tocó: {}",
            f.cuerpo_html
        );
    }

    /// Editar la firma de otra empresa **da error**, no cero filas.
    ///
    /// Un `UPDATE … WHERE id = ?1 AND company_id = ?2` que no encaja no falla:
    /// afecta a cero filas y devuelve `Ok`. La pantalla diría «guardado» sobre
    /// algo que no se guardó, que es la peor forma de fallar.
    #[test]
    fn editar_la_firma_de_otra_empresa_no_se_traga() {
        let (_d, db) = base();
        let otra = "22222222-2222-4222-8222-222222222222";
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('{otra}', 'OTRA', 'MX', 'America/Monterrey',
                         'h@o.mx', '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');"
            ))
        })
        .expect("segunda empresa");

        let mia = db
            .crear_firma(empresa(), "Mía", "<p>Mía</p>", false)
            .expect("crea");

        let error = db
            .actualizar_firma(
                otra.parse().expect("id"),
                mia,
                "Robada",
                "<p>Robada</p>",
                true,
            )
            .expect_err("no es suya");
        assert_eq!(error.clave_i18n(), "error.db.dato_invalido");

        let f = db.firmas(empresa()).expect("lista");
        let f = f.first().expect("sigue ahí");
        assert_eq!(f.nombre, "Mía", "intacta");
        assert!(!f.por_defecto, "y sin el por defecto que pedía la otra");
    }

    // ── R-08 · BORRAR UNA PLANTILLA O UNA FIRMA (21/10/2026) ──────────────
    //
    // Dirección: «tampoco me permite eliminar una plantilla ni una firma,
    // donde también debería tener esa opción».

    /// Una plantilla que nadie usó se borra, **con sus versiones**.
    #[test]
    fn una_plantilla_sin_campanas_se_borra_entera() {
        let (_d, db) = base();
        let (p, v) = db
            .crear_plantilla(empresa(), "Borrable", &redaccion("A", "<p>a</p>"))
            .expect("crea");
        db.nueva_version_de_plantilla(p, &redaccion("B", "<p>b</p>"))
            .expect("segunda");

        db.borrar_plantilla(empresa(), p).expect("se borra");

        assert!(db.plantillas(empresa()).expect("lista").is_empty());
        // Y la versión se fue con ella: si quedara, el registro de envíos
        // podría apuntar a un texto huérfano.
        assert_eq!(
            db.version_de_plantilla(v).expect("consulta"),
            None,
            "la versión sobrevivió a su plantilla"
        );
    }

    /// **Una plantilla que una campaña usó NO se borra, y se dice con su
    /// cifra.**
    ///
    /// El esquema lo impide igualmente (`ON DELETE RESTRICT`), pero fallar por
    /// una clave foránea es, para quien pulsó, «algo falló». Lo que esta
    /// prueba fija es que se diga **antes y con el número**.
    #[test]
    fn una_plantilla_que_una_campana_uso_no_se_borra() {
        let (_d, db) = base();
        let (p, v) = db
            .crear_plantilla(empresa(), "En uso", &redaccion("A", "<p>a</p>"))
            .expect("crea");

        // Una campaña con su etapa apuntando a esa versión.
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO email_account (id, company_id, display_name, email_address,
                                            provider_kind, credential_ref, status,
                                            daily_limit, hourly_limit,
                                            created_at, updated_at)
                 VALUES ('c-1', '{EMPRESA}', 'Ventas', 'v@t.mx', 'smtp',
                         'llavero://v', 'connected', 50, 10,
                         '2026-10-21T00:00:00Z', '2026-10-21T00:00:00Z');
                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES ('k-1', '{EMPRESA}', 'Septiembre', 'draft',
                         '2026-10-21T00:00:00Z', '2026-10-21T00:00:00Z');
                 -- La `CHECK` de la V4 exige que una etapa de correo tenga su
                 -- cuenta: sin ella la fila no entra, y la prueba mediría otra
                 -- cosa.
                 INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             email_account_id, template_version_id,
                                             status, created_at, updated_at)
                 VALUES ('st-1', 'k-1', 1, 'email', 'c-1', '{v}', 'draft',
                         '2026-10-21T00:00:00Z', '2026-10-21T00:00:00Z');"
            ))
        })
        .expect("campaña");

        let error = db
            .borrar_plantilla(empresa(), p)
            .expect_err("no se puede borrar");
        assert_eq!(error.clave_i18n(), "error.db.la_plantilla_esta_en_uso");
        assert!(
            matches!(error, DbError::LaPlantillaEstaEnUso { campanas: 1 }),
            "tiene que decir cuántas: {error:?}"
        );
        assert_eq!(
            db.plantillas(empresa()).expect("lista").len(),
            1,
            "sigue ahí"
        );
    }

    /// Una firma sí se borra: no es lo que contesta «qué decía el correo».
    #[test]
    fn una_firma_se_borra() {
        let (_d, db) = base();
        let f = db
            .crear_firma(empresa(), "Ventas", "<p>Ventas</p>", true)
            .expect("crea");

        db.borrar_firma(empresa(), f).expect("se borra");
        assert!(db.firmas(empresa()).expect("lista").is_empty());
    }

    /// Y **no se puede borrar lo de otra empresa**, ni plantilla ni firma.
    ///
    /// Un `DELETE … WHERE id = ?1` sin comprobar de quién es borra lo ajeno
    /// sin un solo error. Es la misma guarda que `actualizar_firma`.
    #[test]
    fn no_se_borra_lo_de_otra_empresa() {
        let (_d, db) = base();
        let otra = "22222222-2222-4222-8222-222222222222";
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('{otra}', 'OTRA', 'MX', 'America/Monterrey',
                         'h@o.mx', '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');"
            ))
        })
        .expect("segunda empresa");
        let ajena: CompanyId = otra.parse().expect("id");

        let (p, _) = db
            .crear_plantilla(empresa(), "Mía", &redaccion("A", "<p>a</p>"))
            .expect("crea");
        let f = db
            .crear_firma(empresa(), "Mía", "<p>Mía</p>", false)
            .expect("crea");

        assert_eq!(
            db.borrar_plantilla(ajena, p)
                .expect_err("no es suya")
                .clave_i18n(),
            "error.db.dato_invalido"
        );
        assert_eq!(
            db.borrar_firma(ajena, f)
                .expect_err("no es suya")
                .clave_i18n(),
            "error.db.dato_invalido"
        );

        assert_eq!(db.plantillas(empresa()).expect("lista").len(), 1, "intacta");
        assert_eq!(db.firmas(empresa()).expect("lista").len(), 1, "intacta");
    }
}
