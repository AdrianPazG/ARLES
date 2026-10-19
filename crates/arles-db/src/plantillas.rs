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
}
