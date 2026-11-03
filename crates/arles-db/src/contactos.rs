//! Guardar, leer y editar contactos con sus canales.
//!
//! Entrega 3.2, tramo A. Es el **primer código que usa de verdad el esquema de
//! canales** de la migración V3: hasta aquí las tablas existían y nadie las
//! tocaba.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! TRES DECISIONES QUE SE VEN EN TODO EL ARCHIVO
//!
//! **1 · Un contacto y sus canales se escriben en una sola transacción.** Un
//! contacto a medias —guardado sin sus direcciones— es una fila que engorda la
//! lista, cuenta en el total de la campaña y no recibe nada. Peor que no
//! guardarlo.
//!
//! **2 · Leer es siempre por páginas.** El objetivo son 500 000 contactos
//! (T-7). Una función que devuelva «todos» es una función que alguien llamará
//! con medio millón de filas detrás, y el problema aparecerá en el equipo de un
//! cliente y no aquí.
//!
//! **3 · Un choque de direcciones no se devuelve como error de SQLite.** El
//! índice único de `contact_channel` lo rechaza, pero «UNIQUE constraint
//! failed» no dice **qué dirección** ni **de quién** es. Se traduce a un error
//! con la dirección dentro, porque es lo único que permite a la pantalla decir
//! algo útil (§95).
//! ─────────────────────────────────────────────────────────────────────────

use arles_core::canal::Canal;
use arles_core::contacto::{CanalValidado, DatosDeContacto};
use arles_core::ids::{CompanyId, ContactId, ImportBatchId};
use rusqlite::{OptionalExtension, Transaction, params};

use crate::db::Db;
use crate::empresa::ahora;
use crate::error::DbError;

/// Tope de una página. Por encima, la consulta deja de caber cómodamente en
/// memoria y la tabla de la pantalla no puede dibujar tantas filas de golpe.
pub const MAX_POR_PAGINA: u32 = 1_000;

/// Un contacto tal y como está guardado.
///
/// **No lleva `serde`.** Este crate no conoce el formato en que viajan las
/// cosas por la IPC, y no debe: quien decide cómo se ve un contacto al otro
/// lado es `arles-app`, que lo convierte en su propio tipo. Si esta estructura
/// se serializara directamente, cambiar un nombre de columna aquí cambiaría el
/// JSON que recibe la pantalla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactoGuardado {
    pub id: ContactId,
    pub datos: DatosDeContacto,
    pub creado_en: String,
    pub actualizado_en: String,
}

/// Con qué se recorta la lista de contactos. Todo opcional y todo combinable.
///
/// ─────────────────────────────────────────────────────────────────────────
/// R-26 · POR QUÉ ESTOS CUATRO CRITERIOS Y NO OTROS
///
/// Los decidió **medir**, y no la comodidad de escribirlos en SQL. Medido el
/// 03/11/2026 con `cargo run --release -p arles-banco -- contactos 500000`,
/// que corre los mismos caminos a dos tamaños para distinguir lo que se para
/// pronto de lo que recorre la tabla entera. Los nueve caminos candidatos y
/// sus cifras están en la cabecera de `arles-banco/src/contactos.rs`.
///
/// Lo que la medición dijo, y que no se podía suponer:
///
/// - **Una dirección exacta es gratis** —0.0 ms, y no crece con la tabla—
///   **pero sólo si la consulta conduce desde el canal.** El índice único
///   existe desde la V11 y la forma obvia —un `JOIN` con la empresa en el
///   `WHERE`— lo usaba del lado inútil: 60 ms y creciendo 4.3×. **600× de
///   diferencia entre dos consultas que devuelven lo mismo.**
/// - **Un texto sobre el nombre cuesta lo que NO encuentra.** Buscar un nombre
///   frecuente tarda 1.4 ms porque el tope de cien se cumple en las primeras
///   filas; buscar uno que no existe tarda **4.5× más por cada 4× de filas**,
///   porque para decir «no hay nada» hay que visitarlas todas. Es lo contrario
///   de lo que dice la intuición, y es el caso que se toca **mientras se
///   escribe**.
/// - **Un trozo de dirección es la pared de C-2 otra vez**: un `%@dominio` es
///   un sufijo, y ningún índice ordenado por el principio de la cadena lo
///   acota. Así que **no se ofrece**: lo que se ofrece es la dirección
///   completa, que es gratis, y el nombre, que al menos es la pregunta que la
///   gente hace de memoria.
///
/// ── Y UNA QUE SE DESCARTÓ CON SU CIFRA ──
///
/// «Empieza por» sobre el nombre parecía la salida —un prefijo sí se puede
/// indexar—, y **medido no lo es**: sin índice tarda igual que `%x%`, y con
/// uno habría que pagar una migración que deja sin restaurar todo respaldo
/// anterior (8.3b) más el tiempo de importación que la 8.6 midió en +78 % por
/// seis índices. Se descarta hasta que alguien tenga una lista en la que
/// duela, y entonces se mide con el banco en vez de suponerlo.
#[derive(Debug, Clone, Default)]
pub struct FiltroDeContactos<'a> {
    /// Nombre, apellido o empresa — cualquier trozo, sin distinguir mayúsculas.
    ///
    /// Los tres campos juntos y no tres criterios: quien busca «Müller» no
    /// sabe si lo tiene escrito como apellido o como empresa, y obligarle a
    /// elegir es pedirle el dato que viene a buscar.
    pub texto: Option<&'a str>,
    /// Una dirección **completa**, de correo o de WhatsApp.
    ///
    /// Completa y no un trozo, y eso es una decisión medida: con el índice de
    /// la V11 es gratis; en trozos cuesta un recorrido de la tabla de canales.
    /// La pantalla lo dice con esas palabras en vez de dejarlo adivinar.
    pub direccion: Option<&'a str>,
    /// Sólo los que tienen al menos un canal de éste.
    pub canal: Option<Canal>,
    /// Sólo los que tienen alguno de sus canales dado de baja, o sólo los que
    /// no tienen ninguno.
    ///
    /// Es la pregunta de «¿a quién no le va a llegar nada?», que hoy la
    /// pantalla sólo contesta mirando la insignia fila por fila.
    pub con_alguna_baja: Option<bool>,
}

/// Una página de la lista.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginaDeContactos {
    pub contactos: Vec<ContactoGuardado>,
    /// Cuántos hay **en total**, no en esta página.
    ///
    /// Va aquí y no en otra llamada porque la pantalla necesita las dos cosas a
    /// la vez —«mostrando 50 de 12 400»— y pedirlas por separado deja un
    /// instante en que el total y la página no corresponden.
    pub total: u32,
}

/// Todo lo que hay que declarar para dejar constancia de una importación.
///
/// Va junto y no como ocho parámetros sueltos porque **ninguno es opcional**:
/// una importación sin origen declarado, o sin la huella del archivo, no deja
/// la prueba que la ley pide (ADR-0013 §1). Con una estructura, olvidarse de
/// uno no compila.
#[derive(Debug, Clone)]
pub struct DatosDelLote {
    /// Tal y como llegó. **Sólo metadato**: nunca se usa para escribir en disco.
    pub nombre_del_archivo: String,
    /// Huella de los bytes del archivo, para reconocerlo si vuelve.
    pub huella_del_archivo: String,
    /// El mapeo de columnas que se usó, en JSON. Sin él no se puede explicar
    /// por qué un contacto quedó con el nombre en el campo de la empresa.
    pub mapeo_en_json: String,
    /// El texto íntegro que el usuario aceptó. Su huella se calcula al guardar.
    pub texto_del_consentimiento: String,
    pub origen: arles_core::OrigenDeLaLista,
    /// Cifras del análisis, para que el registro cuadre con lo que se enseñó.
    pub total_de_filas: usize,
    pub choques: usize,
    pub invalidas: usize,
    /// A qué lista van estos contactos, si el usuario eligió una (D-15).
    ///
    /// `None` es lo que hacían todas las importaciones hasta el 23/10/2026:
    /// los contactos entran a la empresa y ya. Sigue siendo lo correcto cuando
    /// se carga el padrón general.
    pub lista: Option<arles_core::ids::ContactListId>,
    /// **Las direcciones normalizadas que traía el archivo**, todas: las que
    /// se importan y las que chocaron con alguien que ya estaba.
    ///
    /// ── POR QUÉ VIAJAN AQUÍ Y NO SE DEDUCEN DE LOS CONTACTOS ────────────
    ///
    /// Porque los repetidos **no se importan**. Una tabla de trescientas filas
    /// con cincuenta que ya estaban crea doscientos cincuenta contactos, y si
    /// la lista se armara con «los que acabo de crear», esos cincuenta
    /// quedarían fuera: la campaña saldría a 250 de las 300 que se cargaron,
    /// sin un solo error. Y son los que más importan, porque con ellos ya hay
    /// relación.
    ///
    /// Se mueven, no se copian: con medio millón de filas, duplicarlas sería
    /// la mitad del presupuesto de memoria.
    pub direcciones: Vec<String>,
}

/// Lo que pasó al importar. Cifras, no adjetivos (§94).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumenDeImportacion {
    pub lote: ImportBatchId,
    pub importados: usize,
    pub choques: usize,
    pub invalidas: usize,
    pub total_de_filas: usize,
}

/// Recorta un nombre de archivo para guardarlo.
///
/// Un nombre de 4 000 caracteres no aporta nada y sí puede reventar la
/// maquetación de cualquier pantalla que lo enseñe.
fn recortar(nombre: &str) -> String {
    nombre.chars().take(255).collect()
}

impl Db {
    /// Da de alta un contacto con sus canales.
    ///
    /// # Errores
    ///
    /// [`DbError::DireccionEnUso`] si alguna dirección ya pertenece a otro
    /// contacto de la misma empresa, con la dirección dentro para que la
    /// pantalla pueda señalarla. [`DbError::Sqlite`] si la escritura falla.
    pub fn crear_contacto(
        &self,
        empresa: CompanyId,
        datos: &DatosDeContacto,
    ) -> Result<ContactId, DbError> {
        let id = ContactId::nuevo();
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();

            tx.execute(
                "INSERT INTO contact (id, company_id, first_name, last_name, source,
                                      created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                params![
                    id.to_string(),
                    empresa.to_string(),
                    vacio_a_nulo(&datos.nombre),
                    vacio_a_nulo(&datos.apellido),
                    vacio_a_nulo(&datos.empresa),
                    cuando,
                ],
            )?;

            escribir_canales(&tx, empresa, id, &datos.canales, &cuando)?;
            tx.commit()?;
            Ok(())
        })?;
        Ok(id)
    }

    /// Cambia los datos y los canales de un contacto que ya existe.
    ///
    /// Los canales se reemplazan por completo por los que lleguen. Los que
    /// desaparecen **no se borran**: se marcan con `deleted_at`, y el índice
    /// único es parcial (`WHERE deleted_at IS NULL`), así que la dirección
    /// queda libre para volver a usarse. Borrarlos de verdad perdería la
    /// respuesta a «¿a qué dirección se le escribió en marzo?».
    ///
    /// # Errores
    ///
    /// [`DbError::ContactoNoExiste`] si el id no corresponde a ningún contacto
    /// vivo de esa empresa; [`DbError::DireccionEnUso`] si una dirección nueva
    /// ya es de otro contacto.
    pub fn editar_contacto(
        &self,
        empresa: CompanyId,
        id: ContactId,
        datos: &DatosDeContacto,
    ) -> Result<(), DbError> {
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();

            let filas = tx.execute(
                "UPDATE contact
                    SET first_name = ?3, last_name = ?4, source = ?5, updated_at = ?6
                  WHERE id = ?1 AND company_id = ?2 AND deleted_at IS NULL",
                params![
                    id.to_string(),
                    empresa.to_string(),
                    vacio_a_nulo(&datos.nombre),
                    vacio_a_nulo(&datos.apellido),
                    vacio_a_nulo(&datos.empresa),
                    cuando,
                ],
            )?;
            if filas == 0 {
                return Err(DbError::ContactoNoExiste);
            }

            // Se retiran todos y se vuelven a escribir. Comparar cuáles
            // siguen, cuáles cambiaron y cuáles se fueron daría el mismo
            // resultado con tres veces el código y un caso frontera por rama.
            tx.execute(
                "UPDATE contact_channel SET deleted_at = ?2, updated_at = ?2
                  WHERE contact_id = ?1 AND deleted_at IS NULL",
                params![id.to_string(), cuando],
            )?;

            escribir_canales(&tx, empresa, id, &datos.canales, &cuando)?;
            tx.commit()?;
            Ok(())
        })
    }

    /// La ficha de un contacto, con **todos** sus canales ordenados (L-14).
    ///
    /// # Errores
    ///
    /// [`DbError::DatoInvalido`] si una fila guardada trae un canal que no
    /// reconocemos; [`DbError::Sqlite`] si la consulta falla.
    pub fn contacto(
        &self,
        empresa: CompanyId,
        id: ContactId,
    ) -> Result<Option<ContactoGuardado>, DbError> {
        self.con_dominio(|conn| {
            let cabecera: Option<(String, String, String, String, String)> = conn
                .query_row(
                    "SELECT COALESCE(first_name, ''), COALESCE(last_name, ''),
                            COALESCE(source, ''), created_at, updated_at
                       FROM contact
                      WHERE id = ?1 AND company_id = ?2 AND deleted_at IS NULL",
                    params![id.to_string(), empresa.to_string()],
                    |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?, f.get(3)?, f.get(4)?)),
                )
                .optional()?;

            let Some((nombre, apellido, empresa_del_contacto, creado, actualizado)) = cabecera
            else {
                return Ok(None);
            };

            let canales = leer_canales(conn, &[id.to_string()])?
                .into_iter()
                .next()
                .map(|(_, c)| c)
                .unwrap_or_default();

            Ok(Some(ContactoGuardado {
                id,
                datos: DatosDeContacto {
                    nombre,
                    apellido,
                    empresa: empresa_del_contacto,
                    canales,
                },
                creado_en: creado,
                actualizado_en: actualizado,
            }))
        })
    }

    /// Una página de la lista, ordenada por fecha de alta descendente.
    ///
    /// `tamano` se recorta a [`MAX_POR_PAGINA`] en vez de rechazarse: una
    /// pantalla que pide de más no es un error del usuario, y devolverle un
    /// fallo la dejaría vacía en lugar de con las primeras mil filas.
    ///
    /// Qué se puede filtrar y por qué esos criterios y no otros está en
    /// [`FiltroDeContactos`], con las cifras que lo decidieron.
    ///
    /// # Errores
    ///
    /// [`DbError::DatoInvalido`] si una fila guardada trae un canal
    /// desconocido; [`DbError::Sqlite`] si la consulta falla.
    ///
    /// **POR LA DE ESCRITURA A PROPÓSITO:** la conexión de lectura es **12×
    /// más lenta** recorriendo esta tabla, medido el 03/11/2026 con la misma
    /// consulta sobre la misma base de 182 261 contactos: **306 ms por el
    /// escritor y 3 742 por la lectora**, tres corridas cada uno. Con el filtro
    /// de texto puesto —que recorre la tabla para poder decir que no hay
    /// nada— eso convierte 2.6 segundos en 22, y es lo que hace la pantalla
    /// inusable.
    ///
    /// Esta lectura **se movió a la lectora el 03/11/2026 por la mañana** con
    /// las otras doce, y vuelve aquí la misma tarde porque el filtro la obligó
    /// a medirse con un recorrido de verdad: hasta entonces sólo se había
    /// medido la página sin filtro, que se para en la fila 100 y no distingue
    /// las dos conexiones. **Mover una lectura por la regla general y medirla
    /// después es el orden equivocado**, y es el error de ayer.
    ///
    /// Lo que la lectora sí arregla —no esperar al cerrojo del motor— sigue
    /// siendo cierto y sigue siendo lo que mide la prueba
    /// `una_lectura_por_la_de_escritura_espera_a_que_acabe_la_escritura`. Son
    /// dos costes distintos, y esta pantalla paga el barato: esperar a una
    /// escritura del motor cuesta lo que dure esa escritura; pasar por la
    /// lectora cuesta 12× **siempre**.
    ///
    /// **La causa concreta no se sabe y no se inventa** (la lección de C-2).
    /// Lo medido es que las dos conexiones difieren; por qué, es el tramo
    /// siguiente, y la primera hipótesis a medir es el tamaño de la caché de
    /// páginas, que ninguna de las dos declara.
    pub fn listar_contactos(
        &self,
        empresa: CompanyId,
        filtro: &FiltroDeContactos<'_>,
        desde: u32,
        tamano: u32,
    ) -> Result<PaginaDeContactos, DbError> {
        let tamano = tamano.clamp(1, MAX_POR_PAGINA);
        self.con_dominio(|conn| {
            let empresa = empresa.to_string();

            // ── LA DIRECCIÓN SE RESUELVE ANTES, Y ES LO QUE LA HACE GRATIS ──
            //
            // Medido el 03/11/2026: metida dentro del `WHERE` como
            // `id IN (SELECT …)`, SQLite conduce desde `idx_contact_lista`
            // por el `ORDER BY` y va probando contacto por contacto. Como una
            // dirección casa **una** fila, el tope de cien no se cumple nunca
            // y hay que llegar al final de la tabla: 60 ms con 73 000
            // contactos, creciendo 4.3× por cada 4× de filas.
            //
            // Resuelta aquí, en su propia consulta, usa el índice único de la
            // V11 por el lado que sirve —de la dirección al contacto— y tarda
            // 0.0 ms sin crecer. **600× entre dos formas que devuelven lo
            // mismo**, y ninguna prueba de resultado habría notado la
            // diferencia.
            let ids_de_la_direccion: Option<Vec<String>> = match filtro.direccion {
                None => None,
                Some(d) => {
                    let formas = formas_normalizadas(d);
                    let mut ids: Vec<String> = Vec::new();
                    if !formas.is_empty() {
                        let huecos = std::iter::repeat_n("?", formas.len())
                            .collect::<Vec<_>>()
                            .join(",");
                        let mut q = conn.prepare(&format!(
                            "SELECT DISTINCT contact_id FROM contact_channel
                              WHERE company_id = ?1 AND deleted_at IS NULL
                                AND value_normalized IN ({huecos})"
                        ))?;
                        let mut atar: Vec<&dyn rusqlite::ToSql> = vec![&empresa];
                        for f in &formas {
                            atar.push(f);
                        }
                        ids = q
                            .query_map(atar.as_slice(), |f| f.get(0))?
                            .collect::<Result<_, _>>()?;
                    }
                    Some(ids)
                }
            };

            // Una dirección que no es de nadie da una página vacía **sin
            // tocar `contact`**. Dejar que siga con una lista vacía obligaría
            // a escribir un `IN ()`, que no es SQL válido.
            if ids_de_la_direccion.as_ref().is_some_and(Vec::is_empty) {
                return Ok(PaginaDeContactos {
                    contactos: Vec::new(),
                    total: 0,
                });
            }

            // ── EL MISMO `WHERE` PARA CONTAR Y PARA LEER ──────────────────
            //
            // Construido una sola vez, por el motivo que el registro ya tiene
            // escrito (7.4): dos cadenas parecidas es como se acaba enseñando
            // «50 de 12 400» con un total que no corresponde a lo que se ve.
            //
            // El patrón del texto se arma en Rust con `ESCAPE` y no se pega en
            // SQL. No es cosmética: es H-13, el defecto que el filtro del
            // registro tuvo desde la 7.4 hasta el 02/11/2026 — escribir un `%`
            // devolvía **todas** las filas, que es un filtro contestando como
            // si no hubiera filtro.
            // ── EL TEXTO SE BAJA EN RUST, NO EN SQL, Y ESO ES UN DEFECTO
            //    QUE ENCONTRÓ UNA PRUEBA CON UN ACENTO ──
            //
            // `lower()` de SQLite **sólo baja ASCII**: `lower('TÉLLEZ')` da
            // `tÉllez`, con la `É` intacta. Y `LIKE` tampoco distingue más
            // allá del ASCII. Así que, con el texto pasado tal cual, buscar
            // «TÉLLEZ» no encontraba a Téllez — **sin ningún error**, diciendo
            // que esa persona no está. En un producto mexicano eso no es un
            // caso raro: es el apellido de alguien escrito en mayúsculas.
            //
            // `to_lowercase()` de Rust sí conoce Unicode, así que el patrón
            // entra ya bajado y el `lower()` de SQL sólo tiene que ocuparse de
            // las mayúsculas ASCII de la columna, que es lo que sabe hacer.
            //
            // **Lo que esto NO cubre, dicho en vez de darlo por cubierto:** un
            // dato **guardado** con mayúsculas acentuadas —un «TÉLLEZ» en la
            // columna, no en la búsqueda— seguiría sin encontrarse al buscar
            // «téllez». Cerrarlo del todo exige una función propia en la
            // conexión o una columna normalizada en una migración, y ninguna
            // de las dos cosas se hace sin medir antes si pasa de verdad.
            let texto = filtro
                .texto
                .map(str::to_lowercase)
                .map(|t| crate::actividad::patron_de_busqueda(&t));
            let canal = filtro.canal.map(Canal::como_texto);
            let baja = filtro.con_alguna_baja.map(i64::from);

            let en_la_direccion = match &ids_de_la_direccion {
                None => String::new(),
                Some(ids) => format!(
                    // Los identificadores se interpolan porque son los que
                    // acaba de devolver la base, no texto de nadie: es el
                    // único caso que SQLite no parametriza (lista variable),
                    // y el mismo que los siete `format!` que la auditoría B
                    // dejó medidos como limpios.
                    " AND c.id IN ({})",
                    ids.iter()
                        .map(|i| format!("'{i}'"))
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            };

            // ── UN CRITERIO QUE NO SE PIDIÓ NO ENTRA EN EL SQL ────────────
            //
            // ── Y ES EL HALLAZGO DE ESTA ENTREGA, ENCONTRADO MIDIENDO LO
            //    QUE SE ENTREGÓ Y NO LOS CANDIDATOS ──
            //
            // La primera versión usaba el idioma del registro (7.4) para los
            // cuatro criterios: `(?3 IS NULL OR …)`, con el `WHERE` entero
            // siempre en la consulta y los huecos atados a `NULL` cuando no se
            // piden. Es correcto, es legible, y **con una subconsulta
            // correlacionada dentro cuesta veinte segundos.**
            //
            // Medido el 03/11/2026 sobre 455 470 contactos, tres corridas:
            //
            // | Camino                       | Con `IS NULL OR` | Separado |
            // |---|---|---|
            // | Texto que existe             | 11 390 ms | ver cabecera |
            // | Texto que NO existe          | 21 898 ms | ver cabecera |
            //
            // Contra los 1.4 y 895 ms que medía la consulta suelta sin esos dos
            // criterios. **La diferencia es que SQLite no se salta el `EXISTS`
            // de un criterio vacío**: lo evalúa fila por fila, dos veces, para
            // 455 470 filas, y el `?4 IS NULL` de delante no le ahorra el
            // trabajo porque la subconsulta ya está en el plan.
            //
            // Con una comparación de columna —lo que hace el registro— el
            // idioma es gratis. Con una subconsulta, no. **Copiar una forma que
            // funciona en otro sitio no es medirla**, y ésta es la tercera vez
            // en tres días que algo dado por bueno se cae al medirlo: el FTS5
            // de C-2, el hilo del motor, y esto.
            //
            // Así que el `WHERE` se arma con **sólo los criterios pedidos**. Y
            // sigue siendo **uno** —el mismo para contar y para leer—, que es
            // la garantía que importa: dos cadenas parecidas es como se acaba
            // enseñando «50 de 12 400» con un total que no corresponde.
            let mut trozos = vec![String::from("c.company_id = ?1 AND c.deleted_at IS NULL")];
            let mut atados: Vec<&dyn rusqlite::ToSql> = vec![&empresa];

            if let Some(t) = &texto {
                trozos.push(format!(
                    "(lower(COALESCE(c.first_name,'')) LIKE ?{n} ESCAPE '\\'
                   OR lower(COALESCE(c.last_name,''))  LIKE ?{n} ESCAPE '\\'
                   OR lower(COALESCE(c.source,''))     LIKE ?{n} ESCAPE '\\')",
                    n = atados.len() + 1
                ));
                atados.push(t);
            }
            if let Some(c) = &canal {
                trozos.push(format!(
                    "EXISTS (SELECT 1 FROM contact_channel k
                              WHERE k.contact_id = c.id AND k.deleted_at IS NULL
                                AND k.channel = ?{n})",
                    n = atados.len() + 1
                ));
                atados.push(c);
            }
            if let Some(b) = &baja {
                trozos.push(format!(
                    "?{n} = EXISTS (SELECT 1 FROM contact_channel k
                                      JOIN suppression_entry s
                                        ON s.company_id = k.company_id
                                       AND s.channel = k.channel
                                       AND s.address_normalized = k.value_normalized
                                     WHERE k.contact_id = c.id AND k.deleted_at IS NULL)",
                    n = atados.len() + 1
                ));
                atados.push(b);
            }
            let donde = format!("{}{en_la_direccion}", trozos.join("\n             AND "));
            let atados = atados.as_slice();

            let total: u32 = conn.query_row(
                &format!("SELECT count(*) FROM contact c WHERE {donde}"),
                atados,
                |f| f.get(0),
            )?;

            let mut consulta = conn.prepare(&format!(
                "SELECT c.id, COALESCE(c.first_name, ''), COALESCE(c.last_name, ''),
                        COALESCE(c.source, ''), c.created_at, c.updated_at
                   FROM contact c
                  WHERE {donde}
                  ORDER BY c.created_at DESC, c.id DESC
                  LIMIT {tamano} OFFSET {desde}"
            ))?;
            let filas: Vec<(String, String, String, String, String, String)> = consulta
                .query_map(atados, |f| {
                    Ok((
                        f.get(0)?,
                        f.get(1)?,
                        f.get(2)?,
                        f.get(3)?,
                        f.get(4)?,
                        f.get(5)?,
                    ))
                })?
                .collect::<Result<_, _>>()?;

            // Los canales de toda la página en **una** consulta. Uno por
            // contacto serían mil viajes a la base por página, que es la forma
            // habitual de que una lista vaya bien con diez filas y mal con mil.
            let ids: Vec<String> = filas.iter().map(|f| f.0.clone()).collect();
            let mut por_contacto = leer_canales(conn, &ids)?;

            let mut contactos = Vec::with_capacity(filas.len());
            for (id, nombre, apellido, empresa_del_contacto, creado, actualizado) in filas {
                let canales = por_contacto.remove(&id).unwrap_or_default();
                contactos.push(ContactoGuardado {
                    id: id.parse().map_err(|_| DbError::DatoInvalido {
                        motivo: "el identificador de un contacto no es un UUID",
                    })?,
                    datos: DatosDeContacto {
                        nombre,
                        apellido,
                        empresa: empresa_del_contacto,
                        canales,
                    },
                    creado_en: creado,
                    actualizado_en: actualizado,
                });
            }

            Ok(PaginaDeContactos { contactos, total })
        })
    }

    /// Da de baja un contacto.
    ///
    /// **Borrado lógico.** El borrado físico existe y es otra cosa: lo exige el
    /// derecho de cancelación, tiene su propio flujo en la entrega 3.4 y no se
    /// dispara desde el botón de una lista.
    ///
    /// # Errores
    ///
    /// [`DbError::ContactoNoExiste`] si no había nada que dar de baja.
    pub fn borrar_contacto(&self, empresa: CompanyId, id: ContactId) -> Result<(), DbError> {
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();
            let filas = tx.execute(
                "UPDATE contact SET deleted_at = ?3, updated_at = ?3
                  WHERE id = ?1 AND company_id = ?2 AND deleted_at IS NULL",
                params![id.to_string(), empresa.to_string(), cuando],
            )?;
            if filas == 0 {
                return Err(DbError::ContactoNoExiste);
            }
            // Los canales se van con él: si sobrevivieran, sus direcciones
            // seguirían ocupadas y no se podría volver a dar de alta a la misma
            // persona.
            tx.execute(
                "UPDATE contact_channel SET deleted_at = ?2, updated_at = ?2
                  WHERE contact_id = ?1 AND deleted_at IS NULL",
                params![id.to_string(), cuando],
            )?;
            tx.commit()?;
            Ok(())
        })
    }

    /// Cuáles de esas direcciones están en la lista de supresión.
    ///
    /// Es el aviso de **L-13**: al cambiar el correo o el móvil de alguien, la
    /// dirección vieja puede estar dada de baja y **la nueva no hereda esa
    /// baja** — la supresión es de la dirección, no de la persona (§39).
    /// Callarlo convertiría corregir un dato en una forma silenciosa de
    /// saltarse una baja.
    ///
    /// Devuelve lo que hay, **no bloquea**: quien edita decide.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    /// Importa una tanda de contactos con su registro de origen.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// O ENTRA TODO, O NO ENTRA NADA
    ///
    /// Una transacción para el lote entero. No es la opción cómoda —confirmar
    /// cada mil sería más rápido en un archivo grande— y se elige igual:
    ///
    /// Una importación a medias deja una lista de la que **nadie sabe dónde se
    /// quedó**. Reintentarla duplica lo que ya entró; no reintentarla deja
    /// fuera a gente que el usuario cree tener. Y como los choques ya se
    /// descartaron en el análisis, que esto falle a mitad sólo puede ser por
    /// algo raro —disco lleno, base corrupta—. Ante algo raro, prefiero no
    /// dejar nada a dejar la mitad.
    ///
    /// El precio está medido y aceptado: con 500 000 contactos la transacción
    /// es larga y ARLES no responde mientras dura. Es una operación que el
    /// usuario lanza a propósito y espera a que termine, no algo que ocurra de
    /// fondo.
    /// ─────────────────────────────────────────────────────────────────────
    ///
    /// `contactos` son los que el análisis marcó como **listos**. Aquí no se
    /// vuelve a analizar: si llega uno que choca, la restricción de la base lo
    /// rechaza y **se cae el lote entero**, que es lo correcto —significa que
    /// la base cambió entre el análisis y esto, y entonces el informe que el
    /// usuario aprobó ya no describe la realidad—.
    ///
    /// # Errores
    ///
    /// [`DbError::DireccionEnUso`] si alguna dirección se ocupó entre el
    /// análisis y ahora; [`DbError::Sqlite`] si la escritura falla. En los dos
    /// casos **no se escribe nada**: ni los contactos ni el registro del lote.
    pub fn importar_contactos(
        &self,
        empresa: CompanyId,
        lote: &DatosDelLote,
        contactos: &[DatosDeContacto],
    ) -> Result<ResumenDeImportacion, DbError> {
        let id_lote = ImportBatchId::nuevo();

        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();

            tx.execute(
                "INSERT INTO import_batch
                     (id, company_id, original_filename, stored_filename, file_hash,
                      column_mapping, total_rows, imported, duplicates, invalid,
                      suppressed, consent_affirmation, consent_hash, origin,
                      status, created_at, completed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, ?11, ?12, ?13,
                         'completed', ?14, ?14)",
                params![
                    id_lote.to_string(),
                    empresa.to_string(),
                    recortar(&lote.nombre_del_archivo),
                    // El nombre con el que se guarda es NUESTRO, no el
                    // suministrado. El de fuera nunca toca el disco
                    // (THREAT_MODEL.md §4.1).
                    id_lote.to_string(),
                    lote.huella_del_archivo,
                    lote.mapeo_en_json,
                    lote.total_de_filas,
                    contactos.len(),
                    lote.choques,
                    lote.invalidas,
                    lote.texto_del_consentimiento,
                    arles_core::huella(&lote.texto_del_consentimiento),
                    lote.origen.como_texto(),
                    cuando,
                ],
            )?;

            for datos in contactos {
                let id = ContactId::nuevo();
                tx.execute(
                    "INSERT INTO contact (id, company_id, first_name, last_name,
                                          source, import_batch_id, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
                    params![
                        id.to_string(),
                        empresa.to_string(),
                        vacio_a_nulo(&datos.nombre),
                        vacio_a_nulo(&datos.apellido),
                        vacio_a_nulo(&datos.empresa),
                        id_lote.to_string(),
                        cuando,
                    ],
                )?;
                escribir_canales(&tx, empresa, id, &datos.canales, &cuando)?;
            }

            // ── La lista, DENTRO de la misma transacción (D-15) ───────────
            //
            // Y no en una llamada aparte después, que es lo cómodo: si la
            // importación saliera bien y la asignación fallara, los contactos
            // estarían dentro y **fuera de su lista**, y la campaña saldría
            // corta sin que nada lo dijera. Entero o nada.
            if let Some(lista) = lote.lista {
                crate::listas::meter_en_la_lista(&tx, empresa, lista, &lote.direcciones, &cuando)?;
            }

            tx.commit()?;
            Ok(())
        })?;

        Ok(ResumenDeImportacion {
            lote: id_lote,
            importados: contactos.len(),
            choques: lote.choques,
            invalidas: lote.invalidas,
            total_de_filas: lote.total_de_filas,
        })
    }

    /// Quién es el dueño de cada dirección que ya está registrada.
    ///
    /// Es lo que alimenta la detección de choques de una importación: el
    /// análisis necesita saber **con quién** choca cada dirección, no sólo que
    /// choca. Decir «esta dirección ya existe» sin decir de quién obliga a
    /// buscarla a mano entre miles de contactos.
    ///
    /// Sólo mira las direcciones que se le pasan, en lotes: traerse las 500 000
    /// de la base para comparar contra un archivo de cien filas sería gastar
    /// memoria por nada, y con un archivo de 500 000 sería gastarla dos veces.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn duenos_de_direcciones(
        &self,
        empresa: CompanyId,
        direcciones: &[String],
    ) -> Result<std::collections::HashMap<String, String>, DbError> {
        let mut duenos = std::collections::HashMap::new();
        if direcciones.is_empty() {
            return Ok(duenos);
        }

        self.con_dominio(|conn| {
            // En lotes, con tantos interrogantes como direcciones lleve el
            // lote.
            //
            // SQLite tiene un tope de variables por sentencia. **Medido en este
            // build: 32 766** —ver `el_tope_de_variables_esta_por_encima_del_lote`—,
            // no las 999 que dice la documentación vieja y que este comentario
            // repetía antes de comprobarlo. Con 500 000 contactos (T-7) se pasa
            // igual, y no «a veces»: siempre, y justo en el archivo grande.
            //
            // 400 es holgado por debajo del tope de cualquier build razonable,
            // incluido uno compilado con el límite antiguo.
            const POR_LOTE: usize = 400;

            for lote in direcciones.chunks(POR_LOTE) {
                let huecos = std::iter::repeat_n("?", lote.len())
                    .collect::<Vec<_>>()
                    .join(",");
                let sql = format!(
                    "SELECT ch.value_normalized,
                            COALESCE(
                                TRIM(COALESCE(c.first_name, '') || ' ' || COALESCE(c.last_name, '')),
                                ''
                            )
                       FROM contact_channel ch
                       JOIN contact c ON c.id = ch.contact_id
                      WHERE ch.company_id = ?1
                        AND ch.deleted_at IS NULL
                        AND c.deleted_at IS NULL
                        AND ch.value_normalized IN ({huecos})"
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
                    let (direccion, nombre) = fila?;
                    duenos.insert(direccion, nombre);
                }
            }
            Ok(duenos)
        })
    }

    /// Cuántos contactos se llaman ya como cada uno de estos nombres.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// LA PREGUNTA QUE ESTO CONTESTA, Y QUIÉN LA HIZO
    ///
    /// Dirección, el 23/10/2026: la misma empresa en dos campañas, con correos
    /// y teléfonos distintos en cada una. **ARLES hace lo correcto** —lo que
    /// hace único a un contacto es su dirección, nunca su nombre, así que
    /// entran como dos contactos separados y cada uno conserva lo suyo— pero
    /// en la lista quedan dos filas con el mismo nombre, y el `{{nombre}}` del
    /// correo será el mismo para los dos.
    ///
    /// Avisarlo exige mirar **lo que ya está en la base**, no sólo el archivo:
    /// el caso son dos archivos distintos, el de enero y el de marzo, y mirar
    /// sólo el que se carga no vería nada.
    ///
    /// La clave del mapa va **en minúsculas y recortada**, que es como el
    /// núcleo compara: dos tablas de clientes distintos casi nunca escriben un
    /// nombre con las mismas mayúsculas, y comparar literalmente callaría justo
    /// en el caso más frecuente. En SQL lo hace `lower(trim(…))` y no
    /// `COLLATE NOCASE`, porque `NOCASE` de SQLite **sólo cubre el ASCII**: con
    /// él, «Telemetría» y «TELEMETRÍA» serían dos nombres distintos.
    ///
    /// En lotes de 400 por el tope de variables por sentencia, igual que
    /// [`Db::duenos_de_direcciones`].
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn cuantos_se_llaman_asi(
        &self,
        empresa: CompanyId,
        nombres: &[String],
    ) -> Result<std::collections::HashMap<String, usize>, DbError> {
        let mut cuenta = std::collections::HashMap::new();
        if nombres.is_empty() {
            return Ok(cuenta);
        }

        self.con_dominio(|conn| {
            const POR_LOTE: usize = 400;
            for lote in nombres.chunks(POR_LOTE) {
                let huecos = std::iter::repeat_n("?", lote.len())
                    .collect::<Vec<_>>()
                    .join(",");
                // El `format!` interpola **interrogantes**, nunca un dato.
                // Nombre Y apellido, que es lo que se ve en la lista y lo
                // que compara `arles_core::nombre_entero`. Comparando sólo el
                // nombre de pila, «Ana Ruiz» y «Ana Pérez» saldrían como
                // repetidas — y en una tabla de trescientas filas hay varias
                // Anas. Las dos capas tienen que comparar igual o la cifra del
                // aviso no cuadra.
                let sql = format!(
                    "SELECT lower(trim(trim(COALESCE(first_name, '')) || ' '
                                       || trim(COALESCE(last_name, '')))) AS n,
                            count(*)
                       FROM contact
                      WHERE company_id = ?1 AND deleted_at IS NULL
                        AND lower(trim(trim(COALESCE(first_name, '')) || ' '
                                       || trim(COALESCE(last_name, '')))) IN ({huecos})
                      GROUP BY n"
                );

                let mut consulta = conn.prepare(&sql)?;
                let mut parametros: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(lote.len() + 1);
                let empresa_txt = empresa.to_string();
                parametros.push(&empresa_txt);
                for n in lote {
                    parametros.push(n);
                }

                let filas = consulta.query_map(parametros.as_slice(), |f| {
                    Ok((f.get::<_, String>(0)?, f.get::<_, i64>(1)?))
                })?;
                for fila in filas {
                    let (nombre, cuantos) = fila?;
                    cuenta.insert(nombre, usize::try_from(cuantos).unwrap_or(0));
                }
            }
            Ok(cuenta)
        })
    }

    pub fn direcciones_suprimidas(
        &self,
        empresa: CompanyId,
        canales: &[CanalValidado],
    ) -> Result<Vec<String>, DbError> {
        if canales.is_empty() {
            return Ok(Vec::new());
        }
        self.con_dominio(|conn| {
            let mut suprimidas = Vec::new();
            let mut consulta = conn.prepare(
                "SELECT 1 FROM suppression_entry
                  WHERE company_id = ?1 AND channel = ?2 AND address_normalized = ?3",
            )?;
            for c in canales {
                let hay: Option<i64> = consulta
                    .query_row(
                        params![
                            empresa.to_string(),
                            c.canal.como_texto(),
                            c.valor_normalizado,
                        ],
                        |f| f.get(0),
                    )
                    .optional()?;
                if hay.is_some() {
                    suprimidas.push(c.valor_normalizado.clone());
                }
            }
            Ok(suprimidas)
        })
    }
}

/// `""` se guarda como `NULL`, no como cadena vacía.
///
/// Son dos valores distintos para la base y significan lo mismo para el
/// usuario; dejar que convivan obliga a que cada consulta contemple los dos.
pub(crate) fn vacio_a_nulo(texto: &str) -> Option<&str> {
    let t = texto.trim();
    if t.is_empty() { None } else { Some(t) }
}

/// Inserta los canales de un contacto, traduciendo el choque de direcciones.
fn escribir_canales(
    tx: &Transaction<'_>,
    empresa: CompanyId,
    contacto: ContactId,
    canales: &[CanalValidado],
    cuando: &str,
) -> Result<(), DbError> {
    for c in canales {
        let r = tx.execute(
            "INSERT INTO contact_channel
                 (id, company_id, contact_id, channel, value_raw, value_normalized,
                  created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![
                arles_core::ids::ContactChannelId::nuevo().to_string(),
                empresa.to_string(),
                contacto.to_string(),
                c.canal.como_texto(),
                c.valor_raw,
                c.valor_normalizado,
                cuando,
            ],
        );
        match r {
            Ok(_) => {}
            // El índice único de `contact_channel` rechaza la fila, pero
            // «UNIQUE constraint failed» no dice qué dirección ni de quién. La
            // pantalla no puede señalar nada con eso.
            Err(rusqlite::Error::SqliteFailure(e, _))
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                return Err(DbError::DireccionEnUso {
                    canal: c.canal.como_texto(),
                    direccion: c.valor_normalizado.clone(),
                });
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

/// Las formas normalizadas que puede tener lo que alguien teclea como
/// dirección, para buscarla con el índice de la V11.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ LAS DOS Y NO «LA SUYA»
///
/// La columna indexada guarda la forma **canónica**: un correo en minúsculas,
/// un teléfono en E.164 (`+525512345678`). Y quien filtra no escribe eso:
/// escribe `Ana@Empresa.MX`, o `55 1234 5678`, o pega lo que le llegó por
/// WhatsApp. Comparar lo tecleado tal cual contra la columna no encontraría
/// nada **y no daría ningún error** — el filtro parecería funcionar y diría
/// que esa persona no está.
///
/// Así que se normaliza con **el validador del núcleo**, que es el único sitio
/// donde vive esa regla (§4: un tipo validado sólo sale del validador). Y con
/// los dos canales, porque el filtro es un campo y no dos: nadie que pega una
/// dirección tiene que declarar antes de qué clase es.
///
/// Lo que no se valida como ninguno de los dos devuelve una lista **vacía**, y
/// quien llama lo traduce en una página vacía. No es un error: «pepe» no es
/// una dirección, y decirle «dato inválido» a quien está escribiendo todavía
/// sería un error en medio de la frase.
fn formas_normalizadas(escrito: &str) -> Vec<String> {
    use arles_core::canal::ValorDeCanal;
    let mut formas = Vec::new();
    for canal in Canal::TODOS {
        if let Ok(v) = ValorDeCanal::parse(*canal, escrito, "MX") {
            let n = v.normalized().to_owned();
            if !formas.contains(&n) {
                formas.push(n);
            }
        }
    }
    formas
}

/// Los canales de varios contactos, en una sola consulta y ya ordenados.
///
/// El orden sale de **la base**, no de reordenar en Rust: es el mismo criterio
/// de L-14 —correos primero, móviles después, el principal arriba— y hacerlo
/// aquí evita que la lista y la ficha puedan acabar ordenando distinto.
fn leer_canales(
    conn: &rusqlite::Connection,
    ids: &[String],
) -> Result<std::collections::HashMap<String, Vec<CanalValidado>>, DbError> {
    let mut por_contacto: std::collections::HashMap<String, Vec<CanalValidado>> =
        std::collections::HashMap::new();
    if ids.is_empty() {
        return Ok(por_contacto);
    }

    // `IN` con tantos interrogantes como ids. Interpolar los ids en el SQL
    // sería inyección aunque hoy vengan de nuestra propia consulta.
    let huecos = std::iter::repeat_n("?", ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT contact_id, channel, value_raw, value_normalized,
                CASE WHEN id = (
                    SELECT p.id FROM contact_channel p
                     WHERE p.contact_id = contact_channel.contact_id
                       AND p.channel = contact_channel.channel
                       AND p.deleted_at IS NULL
                     ORDER BY p.created_at, p.id LIMIT 1
                ) THEN 1 ELSE 0 END AS es_primero
           FROM contact_channel
          WHERE contact_id IN ({huecos}) AND deleted_at IS NULL
          ORDER BY contact_id,
                   CASE channel WHEN 'email' THEN 0 ELSE 1 END,
                   created_at, id"
    );

    let mut consulta = conn.prepare(&sql)?;
    let filas = consulta.query_map(rusqlite::params_from_iter(ids), |f| {
        Ok((
            f.get::<_, String>(0)?,
            f.get::<_, String>(1)?,
            f.get::<_, String>(2)?,
            f.get::<_, String>(3)?,
            f.get::<_, i64>(4)?,
        ))
    })?;

    for fila in filas {
        let (contacto, canal, raw, normalizado, primero) = fila?;
        let canal = Canal::desde_texto(&canal).map_err(|_| DbError::DatoInvalido {
            motivo: "un canal guardado no es ni correo ni WhatsApp",
        })?;
        por_contacto
            .entry(contacto)
            .or_default()
            .push(CanalValidado {
                canal,
                valor_raw: raw,
                valor_normalizado: normalizado,
                principal: primero == 1,
            });
    }
    Ok(por_contacto)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arles_core::contacto::{BorradorDeCanal, BorradorDeContacto};

    use crate::conexion::ClaveMaestra;

    pub(super) fn base() -> (tempfile::TempDir, Db, CompanyId) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        let empresa = db
            .guardar_empresa(&empresa_de_prueba())
            .expect("guarda empresa");
        (dir, db, empresa)
    }

    fn empresa_de_prueba() -> arles_core::DatosDeEmpresa {
        arles_core::DatosDeEmpresa::validar(&arles_core::BorradorDeEmpresa {
            nombre_comercial: "TELEMETRY INSIGHT".into(),
            pais: "MX".into(),
            zona_horaria: "America/Mexico_City".into(),
            correo_corporativo: "hola@telemetrymx.com".into(),
            sitio_web: String::new(),
        })
        .expect("empresa válida")
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

    /// Mete `cuantos` contactos en **una** transacción y por SQL directo.
    ///
    /// Mil llamadas a `crear_contacto` serían mil transacciones y la prueba
    /// tardaría más de lo que nadie espera de un `cargo test`. Aquí lo que se
    /// mide es el recorte de la página, no el alta.
    fn sembrar_contactos(db: &Db, empresa: CompanyId, cuantos: u32) {
        db.ejecutar_en_pruebas(|conn| {
            let tx = conn.unchecked_transaction()?;
            for i in 0..cuantos {
                tx.execute(
                    "INSERT INTO contact (id, company_id, first_name, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?4)",
                    params![
                        ContactId::nuevo().to_string(),
                        empresa.to_string(),
                        format!("Contacto {i}"),
                        ahora(),
                    ],
                )?;
            }
            tx.commit()
        })
        .expect("siembra");
    }

    #[test]
    fn lo_guardado_se_lee_igual() {
        let (_d, db, empresa) = base();
        let datos = contacto(&[
            (Canal::Correo, "Ana@Empresa.MX"),
            (Canal::WhatsApp, "81 1234 5678"),
        ]);
        let id = db.crear_contacto(empresa, &datos).expect("crea");

        let leido = db.contacto(empresa, id).expect("lee").expect("existe");
        assert_eq!(leido.datos.nombre, "Ana");
        assert_eq!(leido.datos.canales.len(), 2);
        assert_eq!(
            leido
                .datos
                .canales
                .first()
                .map(|c| c.valor_normalizado.as_str()),
            Some("ana@empresa.mx")
        );
        // El original se conserva, no sólo la forma canónica.
        assert_eq!(
            leido.datos.canales.first().map(|c| c.valor_raw.as_str()),
            Some("Ana@Empresa.MX")
        );
    }

    /// **L-14 leído de la base.** El orden no se reconstruye en Rust: sale de
    /// la consulta, para que la lista y la ficha no puedan ordenar distinto.
    ///
    /// Se entra con los cuatro canales **desordenados a propósito**: mezclados
    /// los dos tipos y con el que acabará siendo principal en tercer lugar. Lo
    /// que se exige es doble:
    ///
    /// 1. que salgan agrupados —correos, luego móviles— con el principal
    ///    arriba de su grupo, y
    /// 2. que ese orden sea **exactamente** el que el núcleo dejó al validar.
    ///
    /// El punto 2 es el que importa: el núcleo ordena en memoria y la base
    /// vuelve a ordenar al leer. Si los dos criterios se separaran algún día,
    /// la ficha y la lista enseñarían los mismos canales en distinto orden sin
    /// que nada fallara. Aquí falla.
    #[test]
    fn la_ficha_devuelve_los_canales_agrupados_y_con_el_principal_arriba() {
        let (_d, db, empresa) = base();
        let datos = contacto(&[
            (Canal::WhatsApp, "+528100000002"),
            (Canal::Correo, "alta@empresa.mx"),
            (Canal::WhatsApp, "+528100000001"),
            (Canal::Correo, "beta@empresa.mx"),
        ]);
        let id = db.crear_contacto(empresa, &datos).expect("crea");

        let leido = db.contacto(empresa, id).expect("lee").expect("existe");
        let orden: Vec<_> = leido
            .datos
            .canales
            .iter()
            .map(|c| (c.canal, c.valor_normalizado.as_str(), c.principal))
            .collect();

        // El principal de cada tipo es el **primero que el usuario escribió de
        // ese tipo**, no el alfabético ni el primero del formulario: aquí el
        // móvil …002 venía antes que el …001.
        assert_eq!(
            orden,
            vec![
                (Canal::Correo, "alta@empresa.mx", true),
                (Canal::Correo, "beta@empresa.mx", false),
                (Canal::WhatsApp, "+528100000002", true),
                (Canal::WhatsApp, "+528100000001", false),
            ]
        );

        // Y lo mismo que dejó el núcleo, canal por canal.
        let del_nucleo: Vec<_> = datos
            .canales
            .iter()
            .map(|c| (c.canal, c.valor_normalizado.as_str(), c.principal))
            .collect();
        assert_eq!(
            orden, del_nucleo,
            "la base y el núcleo ordenan los canales con criterios distintos"
        );
    }

    /// El agrupado por tipo lo hace **la consulta**, no el orden en que se
    /// escribieron las filas.
    ///
    /// La prueba de arriba no lo demuestra, y se vio: quitando el
    /// `CASE channel WHEN 'email' …` del `ORDER BY`, seguía en verde. La razón
    /// es que el núcleo ya ordena antes de insertar, así que `created_at` sola
    /// reproduce el mismo resultado y la cláusula sobraba sin que se notara.
    ///
    /// Aquí los canales se escriben **intercalados a mano**, que es lo que
    /// pasará en cuanto exista un camino que añada un canal suelto a un
    /// contacto que ya existe, sin reescribir los demás. Si el agrupado
    /// dependiera del orden de inserción, esto saldría mezclado.
    #[test]
    fn el_agrupado_por_tipo_no_depende_del_orden_en_que_se_escribieron() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "uno@empresa.mx")]))
            .expect("crea");

        // …y ahora, uno a uno y alternando tipo, como si se hubieran ido
        // añadiendo con el tiempo.
        let sueltos = [
            (Canal::WhatsApp, "+528100000001"),
            (Canal::Correo, "dos@empresa.mx"),
            (Canal::WhatsApp, "+528100000002"),
            (Canal::Correo, "tres@empresa.mx"),
        ];
        db.ejecutar_en_pruebas(|conn| {
            // El canal que ya existe se ancla a una fecha fija, porque el alta
            // lo escribió con la hora real: si se dejara, «antes» y «después»
            // dependerían de a qué hora se corre la prueba.
            conn.execute(
                "UPDATE contact_channel SET created_at = '2026-09-17T10:00:00Z'
                  WHERE contact_id = ?1",
                params![id.to_string()],
            )?;

            for (n, (canal, valor)) in sueltos.iter().enumerate() {
                conn.execute(
                    "INSERT INTO contact_channel
                         (id, company_id, contact_id, channel, value_raw,
                          value_normalized, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?6)",
                    params![
                        arles_core::ids::ContactChannelId::nuevo().to_string(),
                        empresa.to_string(),
                        id.to_string(),
                        canal.como_texto(),
                        valor,
                        // Fechas crecientes y distintas: el orden de escritura
                        // queda fijado sin ambigüedad.
                        format!("2026-09-17T10:0{n}:00Z"),
                    ],
                )?;
            }
            Ok(())
        })
        .expect("escribe los canales sueltos");

        let leido = db.contacto(empresa, id).expect("lee").expect("existe");
        let tipos: Vec<_> = leido.datos.canales.iter().map(|c| c.canal).collect();

        assert_eq!(
            tipos,
            vec![
                Canal::Correo,
                Canal::Correo,
                Canal::Correo,
                Canal::WhatsApp,
                Canal::WhatsApp,
            ],
            "los canales salen intercalados: el agrupado depende del orden de inserción"
        );

        // Y dentro del grupo, el más antiguo arriba y marcado como principal.
        assert_eq!(
            leido
                .datos
                .canales
                .first()
                .map(|c| (c.valor_normalizado.as_str(), c.principal)),
            Some(("uno@empresa.mx", true))
        );
        assert_eq!(
            leido
                .datos
                .canales
                .get(3)
                .map(|c| (c.valor_normalizado.as_str(), c.principal)),
            Some(("+528100000001", true))
        );
    }

    /// Dos contactos no pueden compartir dirección, y el error dice **cuál**.
    /// Con «UNIQUE constraint failed» la pantalla no puede señalar nada.
    #[test]
    fn una_direccion_repetida_da_un_error_que_dice_cual_es() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("el primero entra");

        let e = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "ANA@empresa.mx")]))
            .expect_err("debe fallar");
        match e {
            DbError::DireccionEnUso { canal, direccion } => {
                assert_eq!(canal, "email");
                assert_eq!(direccion, "ana@empresa.mx");
            }
            otro => panic!("se esperaba DireccionEnUso, llegó {otro:?}"),
        }
    }

    /// **Lo que protege la transacción.** Si el segundo canal choca, el
    /// contacto **no puede quedarse guardado sin sus direcciones**: sería una
    /// fila que engorda la lista y no recibe nada.
    #[test]
    fn un_choque_en_el_segundo_canal_no_deja_el_contacto_a_medias() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ocupado@empresa.mx")]))
            .expect("el primero entra");

        let antes = db
            .listar_contactos(empresa, &FiltroDeContactos::default(), 0, 100)
            .expect("lista")
            .total;
        let e = db.crear_contacto(
            empresa,
            &contacto(&[
                (Canal::WhatsApp, "+528112345678"),
                (Canal::Correo, "ocupado@empresa.mx"),
            ]),
        );
        assert!(e.is_err());

        let despues = db
            .listar_contactos(empresa, &FiltroDeContactos::default(), 0, 100)
            .expect("lista")
            .total;
        assert_eq!(
            antes, despues,
            "quedó un contacto a medias: guardado y sin poder recibir nada"
        );
    }

    #[test]
    fn editar_cambia_los_datos_y_los_canales() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "viejo@empresa.mx")]))
            .expect("crea");

        let nuevos = contacto(&[
            (Canal::Correo, "nuevo@empresa.mx"),
            (Canal::WhatsApp, "+528112345678"),
        ]);
        db.editar_contacto(empresa, id, &nuevos).expect("edita");

        let leido = db.contacto(empresa, id).expect("lee").expect("existe");
        assert_eq!(leido.datos.canales.len(), 2);
        assert!(
            !leido
                .datos
                .canales
                .iter()
                .any(|c| c.valor_normalizado == "viejo@empresa.mx"),
            "el canal retirado sigue apareciendo"
        );
    }

    /// El canal retirado deja su dirección **libre**: el índice único es
    /// parcial. Sin esto, corregir una errata dejaría la dirección buena
    /// bloqueada por la mala para siempre.
    #[test]
    fn la_direccion_de_un_canal_retirado_se_puede_volver_a_usar() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "aana@empresa.mx")]))
            .expect("crea");

        db.editar_contacto(empresa, id, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("corrige la errata");

        // Y otra persona puede quedarse con la dirección vieja.
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "aana@empresa.mx")]))
            .expect("la dirección retirada quedó libre");
    }

    #[test]
    fn editar_un_contacto_que_no_existe_lo_dice() {
        let (_d, db, empresa) = base();
        let e = db.editar_contacto(
            empresa,
            ContactId::nuevo(),
            &contacto(&[(Canal::Correo, "ana@empresa.mx")]),
        );
        assert!(matches!(e, Err(DbError::ContactoNoExiste)));
    }

    /// Borrar es lógico y se lleva los canales: si sobrevivieran, la dirección
    /// seguiría ocupada y no se podría volver a dar de alta a esa persona.
    #[test]
    fn borrar_libera_las_direcciones_y_saca_de_la_lista() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");

        db.borrar_contacto(empresa, id).expect("borra");
        assert_eq!(
            db.listar_contactos(empresa, &FiltroDeContactos::default(), 0, 10)
                .expect("lista")
                .total,
            0
        );
        assert!(db.contacto(empresa, id).expect("lee").is_none());

        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("la dirección quedó libre");
    }

    /// El objetivo son 500 000 contactos. Una lectura que devuelva «todos» es
    /// una lectura que alguien llamará con medio millón de filas detrás.
    #[test]
    fn la_lista_va_por_paginas_y_dice_el_total() {
        let (_d, db, empresa) = base();
        for i in 0..25 {
            db.crear_contacto(
                empresa,
                &contacto(&[(Canal::Correo, &format!("c{i}@empresa.mx"))]),
            )
            .expect("crea");
        }

        let primera = db
            .listar_contactos(empresa, &FiltroDeContactos::default(), 0, 10)
            .expect("lista");
        assert_eq!(primera.contactos.len(), 10);
        assert_eq!(
            primera.total, 25,
            "el total es el de la empresa, no el de la página"
        );

        let ultima = db
            .listar_contactos(empresa, &FiltroDeContactos::default(), 20, 10)
            .expect("lista");
        assert_eq!(ultima.contactos.len(), 5);

        // Sin solaparse: la primera y la última no comparten ningún contacto.
        let ids_primera: Vec<_> = primera.contactos.iter().map(|c| c.id).collect();
        assert!(!ultima.contactos.iter().any(|c| ids_primera.contains(&c.id)));
    }

    fn lote_de_prueba() -> DatosDelLote {
        DatosDelLote {
            nombre_del_archivo: "contactos-marzo.xlsx".into(),
            huella_del_archivo: "sha256:abc".into(),
            mapeo_en_json: r#"["nombre","correo"]"#.into(),
            texto_del_consentimiento:
                "Declaro que esta lista tiene origen lícito. [TEXTO PROVISIONAL]".into(),
            origen: arles_core::OrigenDeLaLista::FormularioPropio,
            total_de_filas: 3,
            choques: 0,
            invalidas: 0,
            lista: None,
            direcciones: Vec::new(),
        }
    }

    /// **Importar a una lista mete también a los que ya estaban (D-15).**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// EL CASO QUE DECIDE SI ESTO SIRVE O NO
    ///
    /// Dirección carga la tabla de «Videovigilancia»: trescientas filas, de
    /// las que cincuenta ya están en la base de otra campaña. Los repetidos
    /// **no se importan** —es la decisión del 18/09/2026, los choques se
    /// enseñan y no se fusionan solos—, así que la forma obvia de armar la
    /// lista, «los contactos que acabo de crear», la dejaría en doscientos
    /// cincuenta.
    ///
    /// La campaña saldría a 250 de las 300 que se cargaron, **sin un solo
    /// error y sin ninguna prueba en rojo**. Y las cincuenta que faltan son
    /// las que más importan: con ellas ya hay relación.
    ///
    /// Aquí se siembra a propósito el caso extremo: el archivo trae dos
    /// direcciones y **las dos ya existen**, así que no se importa a nadie y
    /// la lista tiene que salir con dos. Con una sola dirección repetida no se
    /// probaría nada — la lista podría venir de cualquiera de los dos sitios.
    ///
    /// Verificada rompiéndola: quitando la llamada a `meter_en_la_lista` de
    /// `importar_contactos`, `cuantos` sale 0 en vez de 2.
    #[test]
    fn importar_a_una_lista_mete_tambien_a_los_que_ya_estaban() {
        let (_d, db, empresa) = base();

        // Primera importación, sin lista: la gente entra a la empresa.
        let gente = [
            contacto(&[(Canal::Correo, "ana@empresa.mx")]),
            contacto(&[(Canal::Correo, "beto@empresa.mx")]),
        ];
        db.importar_contactos(empresa, &lote_de_prueba(), &gente)
            .expect("primera importación");

        // Segunda: el archivo de «Videovigilancia» trae a los DOS mismos, así
        // que no se importa nadie nuevo.
        let lista = db.crear_lista(empresa, "Videovigilancia").expect("crea");
        let lote = DatosDelLote {
            lista: Some(lista),
            direcciones: vec!["ana@empresa.mx".into(), "beto@empresa.mx".into()],
            choques: 2,
            ..lote_de_prueba()
        };
        let resumen = db
            .importar_contactos(empresa, &lote, &[])
            .expect("segunda importación");

        assert_eq!(resumen.importados, 0, "los dos ya estaban");

        let listas = db.listar_listas(empresa).expect("listas");
        assert_eq!(listas.len(), 1);
        let primera = listas.first().expect("hay una lista");
        assert_eq!(
            primera.cuantos, 2,
            "la lista tiene a los dos repetidos, que es lo que el archivo pedía"
        );
    }

    /// **Y sin lista, importar no crea ninguna.**
    ///
    /// La pareja de la de arriba. Sin ella, el filtro podría estar escrito al
    /// revés —crear siempre una lista con el nombre del archivo— y la de
    /// arriba seguiría en verde.
    #[test]
    fn importar_sin_lista_no_crea_ninguna() {
        let (_d, db, empresa) = base();
        let gente = [contacto(&[(Canal::Correo, "ana@empresa.mx")])];
        db.importar_contactos(empresa, &lote_de_prueba(), &gente)
            .expect("importa");

        assert!(
            db.listar_listas(empresa).expect("listas").is_empty(),
            "sin lista elegida no se inventa ninguna"
        );
    }

    #[test]
    fn una_importacion_deja_los_contactos_y_su_registro() {
        let (_d, db, empresa) = base();
        let gente = [
            contacto(&[(Canal::Correo, "uno@empresa.mx")]),
            contacto(&[(Canal::Correo, "dos@empresa.mx")]),
        ];

        let resumen = db
            .importar_contactos(empresa, &lote_de_prueba(), &gente)
            .expect("importa");

        assert_eq!(resumen.importados, 2);
        assert_eq!(
            db.listar_contactos(empresa, &FiltroDeContactos::default(), 0, 100)
                .expect("lista")
                .total,
            2
        );

        // Y cada contacto sabe de qué importación viene.
        let con_lote: i64 = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row(
                    "SELECT count(*) FROM contact WHERE import_batch_id = ?1",
                    params![resumen.lote.to_string()],
                    |f| f.get(0),
                )
            })
            .expect("cuenta");
        assert_eq!(con_lote, 2);
    }

    /// **El disco se llena a mitad de una importación.**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ ESTA PRUEBA LA ESCRIBIÓ UNA AUDITORÍA Y NO LA 3.3
    ///
    /// `ESTRATEGIA_QA.md` §3 lo lleva como caso frontera desde la Fase 0
    /// —«disco lleno a mitad de importación → transacción revertida, sin
    /// estado a medias»— y afirma de toda esa lista que «cada uno tiene un
    /// test con nombre». La auditoría C del §140 los buscó uno a uno el
    /// 07/10/2026 y **éste no existía**, seguramente porque llenar un disco
    /// de verdad no se puede hacer en una prueba.
    ///
    /// Pero sí se puede **provocar el error que daría**. SQLite tiene
    /// `PRAGMA max_page_count`: con el tope bajado, la escritura que lo pasa
    /// falla con `SQLITE_FULL`, que es exactamente el código de un disco
    /// lleno. No es una simulación aproximada: es el mismo error por el mismo
    /// camino.
    ///
    /// Lo que se comprueba es lo que importa: que **no queda nada a medias**.
    /// Una importación de diez mil filas a medias es la peor de las tres
    /// salidas posibles — reintentarla duplica, no reintentarla pierde gente,
    /// y nadie sabe por dónde se quedó.
    ///
    /// **Verificada rompiéndola**, y no por el camino fácil. Quitar el tope de
    /// páginas sólo demostraría que el disparador funciona; lo que hay que ver
    /// caer es la **garantía**. Así que se partió la atomicidad: un
    /// `tx.commit()` colado entre el registro del lote y el bucle de los
    /// contactos. Sale en rojo por la segunda aserción, la del registro —
    /// *«quedó el registro de una importación que no ocurrió, y la pantalla lo
    /// enseñaría como hecha»*—, que es exactamente el estado a medias que el
    /// caso frontera describe.
    #[test]
    fn un_disco_lleno_a_mitad_de_importacion_no_deja_nada_a_medias() {
        let (_d, db, empresa) = base();

        // Primero una importación normal, para tener un estado conocido que
        // comparar. Sin ella la prueba sólo diría «de cero no salió nada».
        db.importar_contactos(
            empresa,
            &lote_de_prueba(),
            &[contacto(&[(Canal::Correo, "antes@empresa.mx")])],
        )
        .expect("la primera importación cabe");

        let antes = db
            .listar_contactos(empresa, &FiltroDeContactos::default(), 0, 100)
            .expect("lista")
            .total;
        let lotes_antes: i64 = db
            .ejecutar_en_pruebas(|c| {
                c.query_row("SELECT count(*) FROM import_batch", [], |f| f.get(0))
            })
            .expect("cuenta lotes");

        // Y ahora el disco se acaba. El tope se pone al número de páginas que
        // ya hay: la siguiente escritura que necesite una más falla.
        let paginas: i64 = db
            .ejecutar_en_pruebas(|c| c.query_row("PRAGMA page_count", [], |f| f.get(0)))
            .expect("páginas");
        db.ejecutar_en_pruebas(|c| {
            c.pragma_update(None, "max_page_count", paginas)?;
            Ok(())
        })
        .expect("baja el tope");

        // Bastante gente para que haga falta una página nueva. Con dos o tres
        // podría caber en el hueco libre de las que ya hay, y la prueba
        // pasaría sin haber provocado nada — el caso clásico de la prueba de
        // la página de contactos con un solo contacto (§2).
        let muchos: Vec<_> = (0..400)
            .map(|i| contacto(&[(Canal::Correo, &format!("lleno{i}@empresa.mx"))]))
            .collect();
        let resultado = db.importar_contactos(empresa, &lote_de_prueba(), &muchos);

        assert!(
            resultado.is_err(),
            "con el disco lleno la importación tiene que fallar, no escribir \
             lo que quepa"
        );

        // Se devuelve el tope para poder leer la base y comprobarlo.
        db.ejecutar_en_pruebas(|c| {
            c.pragma_update(None, "max_page_count", 1_073_741_823_i64)?;
            Ok(())
        })
        .expect("devuelve el tope");

        assert_eq!(
            db.listar_contactos(empresa, &FiltroDeContactos::default(), 0, 100)
                .expect("lista")
                .total,
            antes,
            "quedaron contactos de la importación que falló"
        );
        let lotes_despues: i64 = db
            .ejecutar_en_pruebas(|c| {
                c.query_row("SELECT count(*) FROM import_batch", [], |f| f.get(0))
            })
            .expect("cuenta lotes");
        assert_eq!(
            lotes_despues, lotes_antes,
            "quedó el registro de una importación que no ocurrió, y la \
             pantalla lo enseñaría como hecha"
        );
    }

    /// **La garantía de esta entrega: o entra todo, o no entra nada.**
    ///
    /// Una importación a medias deja una lista de la que nadie sabe dónde se
    /// quedó: reintentarla duplica lo ya entrado, y no reintentarla deja fuera a
    /// gente que el usuario cree tener.
    ///
    /// Se provoca con una dirección que se ocupó **entre el análisis y la
    /// escritura**, que es el caso real: el análisis dijo que estaba libre y
    /// alguien la dio de alta mientras el usuario revisaba el informe.
    #[test]
    fn si_una_fila_falla_no_entra_ninguna() {
        let (_d, db, empresa) = base();
        // Alguien dio de alta esta dirección después del análisis.
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ocupada@empresa.mx")]))
            .expect("alta previa");

        let gente = [
            contacto(&[(Canal::Correo, "primera@empresa.mx")]),
            contacto(&[(Canal::Correo, "ocupada@empresa.mx")]), // choca
            contacto(&[(Canal::Correo, "tercera@empresa.mx")]),
        ];

        let fallo = db.importar_contactos(empresa, &lote_de_prueba(), &gente);
        assert!(matches!(fallo, Err(DbError::DireccionEnUso { .. })));

        // Sólo queda el contacto previo: ni la primera fila, que iba bien y se
        // escribió antes del choque, ni la tercera.
        let pagina = db
            .listar_contactos(empresa, &FiltroDeContactos::default(), 0, 100)
            .expect("lista");
        assert_eq!(
            pagina.total, 1,
            "quedaron {} contactos: la importación entró a medias",
            pagina.total
        );

        // Y tampoco queda el registro del lote: un lote «completado» sin sus
        // contactos sería una prueba falsa de una importación que no ocurrió.
        let lotes: i64 = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row("SELECT count(*) FROM import_batch", [], |f| f.get(0))
            })
            .expect("cuenta");
        assert_eq!(lotes, 0, "quedó el registro de un lote que no se escribió");
    }

    /// El registro guarda **qué** se declaró, no sólo que se declaró algo: el
    /// origen concreto, el texto íntegro y su huella (ADR-0013 §1).
    #[test]
    fn el_registro_guarda_el_origen_el_texto_y_su_huella() {
        let (_d, db, empresa) = base();
        let lote = lote_de_prueba();
        let resumen = db
            .importar_contactos(
                empresa,
                &lote,
                &[contacto(&[(Canal::Correo, "uno@empresa.mx")])],
            )
            .expect("importa");

        let (origen, texto, hash): (String, String, String) = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row(
                    "SELECT origin, consent_affirmation, consent_hash
                       FROM import_batch WHERE id = ?1",
                    params![resumen.lote.to_string()],
                    |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?)),
                )
            })
            .expect("lee el lote");

        assert_eq!(origen, "formulario_propio");
        assert_eq!(texto, lote.texto_del_consentimiento);
        assert_eq!(
            hash,
            arles_core::huella(&lote.texto_del_consentimiento),
            "la huella no corresponde al texto guardado"
        );
    }

    /// El nombre del archivo se guarda **recortado y como metadato**. Nunca
    /// toca el disco: el archivo se guarda con un identificador nuestro
    /// (THREAT_MODEL.md §4.1).
    #[test]
    fn el_nombre_del_archivo_no_se_usa_para_escribir() {
        let (_d, db, empresa) = base();
        let mut lote = lote_de_prueba();
        lote.nombre_del_archivo = format!("../../{}.xlsx", "a".repeat(400));

        let resumen = db
            .importar_contactos(
                empresa,
                &lote,
                &[contacto(&[(Canal::Correo, "uno@empresa.mx")])],
            )
            .expect("importa");

        let (original, guardado): (String, String) = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row(
                    "SELECT original_filename, stored_filename
                       FROM import_batch WHERE id = ?1",
                    params![resumen.lote.to_string()],
                    |f| Ok((f.get(0)?, f.get(1)?)),
                )
            })
            .expect("lee");

        assert!(original.chars().count() <= 255, "no se recortó");
        assert_eq!(
            guardado,
            resumen.lote.to_string(),
            "el nombre con el que se guarda tiene que ser nuestro, no el de fuera"
        );
        assert!(
            !guardado.contains(".."),
            "el nombre de fuera llegó al disco"
        );
    }

    /// Las cifras del resumen son las del análisis: si no cuadraran, el
    /// registro contaría una historia distinta de la que el usuario aprobó.
    #[test]
    fn las_cifras_del_resumen_cuadran_con_las_del_analisis() {
        let (_d, db, empresa) = base();
        let mut lote = lote_de_prueba();
        lote.total_de_filas = 10;
        lote.choques = 3;
        lote.invalidas = 2;

        let gente: Vec<_> = (0..5)
            .map(|i| contacto(&[(Canal::Correo, &format!("c{i}@empresa.mx"))]))
            .collect();
        let resumen = db
            .importar_contactos(empresa, &lote, &gente)
            .expect("importa");

        assert_eq!(resumen.importados, 5);
        assert_eq!(resumen.choques, 3);
        assert_eq!(resumen.invalidas, 2);
        assert_eq!(resumen.total_de_filas, 10);
        assert_eq!(
            resumen.importados + resumen.choques + resumen.invalidas,
            resumen.total_de_filas,
            "las cifras no suman: el informe y el registro dirían cosas distintas"
        );
    }

    /// Importar cero contactos deja el registro igual: es una importación que
    /// ocurrió y en la que no entró nadie, y eso también hay que poder contarlo.
    #[test]
    fn una_importacion_sin_nada_que_importar_deja_su_registro() {
        let (_d, db, empresa) = base();
        let mut lote = lote_de_prueba();
        lote.total_de_filas = 2;
        lote.invalidas = 2;

        let resumen = db.importar_contactos(empresa, &lote, &[]).expect("importa");
        assert_eq!(resumen.importados, 0);

        let lotes: i64 = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row("SELECT count(*) FROM import_batch", [], |f| f.get(0))
            })
            .expect("cuenta");
        assert_eq!(lotes, 1);
    }

    /// La consulta de dueños dice **con quién** choca cada dirección, no sólo
    /// que choca. Es lo que permite al informe de una importación decir «ya la
    /// tiene Ana Ruiz» en vez de «ya existe».
    #[test]
    fn los_duenos_dicen_de_quien_es_cada_direccion() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");

        let duenos = db
            .duenos_de_direcciones(empresa, &["ana@empresa.mx".to_owned()])
            .expect("consulta");

        assert_eq!(duenos.len(), 1);
        assert_eq!(
            duenos.get("ana@empresa.mx").map(String::as_str),
            Some("Ana Ruiz")
        );
    }

    /// Una dirección que no está no aparece en el mapa. Si apareciera con el
    /// nombre vacío, el análisis la contaría como choque y la importación
    /// dejaría fuera filas perfectamente buenas.
    #[test]
    fn una_direccion_que_no_existe_no_sale_en_el_mapa() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");

        let duenos = db
            .duenos_de_direcciones(
                empresa,
                &["ana@empresa.mx".to_owned(), "nadie@empresa.mx".to_owned()],
            )
            .expect("consulta");

        assert_eq!(duenos.len(), 1);
        assert!(!duenos.contains_key("nadie@empresa.mx"));
    }

    /// Un contacto dado de baja **libera** su dirección, así que no puede salir
    /// como dueño: si saliera, reimportar a alguien que se dio de baja sería
    /// imposible y el informe culparía a un contacto que ya no está.
    #[test]
    fn un_contacto_dado_de_baja_no_es_dueno_de_nada() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");
        db.borrar_contacto(empresa, id).expect("baja");

        let duenos = db
            .duenos_de_direcciones(empresa, &["ana@empresa.mx".to_owned()])
            .expect("consulta");
        assert!(
            duenos.is_empty(),
            "una dirección liberada salió como ocupada"
        );
    }

    /// El filtro por `contact.deleted_at` es **cinturón y tirantes**, y aquí se
    /// prueba contra el caso para el que existe.
    ///
    /// Hoy `borrar_contacto` marca el contacto **y** sus canales, así que
    /// filtrar por el canal ya bastaría — el cebo que quitaba este filtro no
    /// rompía nada—. Pero una base editada por fuera, o una migración a medias,
    /// puede dejar un contacto de baja con sus canales vivos. Sin el filtro, ese
    /// contacto fantasma saldría como dueño y bloquearía una importación
    /// perfectamente legítima.
    ///
    /// Se construye ese estado a mano, que es la única forma de llegar a él.
    #[test]
    fn un_contacto_de_baja_con_canales_vivos_tampoco_es_dueno() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(
                empresa,
                &contacto(&[(Canal::Correo, "fantasma@empresa.mx")]),
            )
            .expect("crea");

        db.ejecutar_en_pruebas(|conn| {
            // Sólo el contacto, dejando sus canales vivos: el estado
            // inconsistente que el filtro existe para tolerar.
            conn.execute(
                "UPDATE contact SET deleted_at = ?2 WHERE id = ?1",
                params![id.to_string(), ahora()],
            )?;
            Ok(())
        })
        .expect("deja el contacto a medio borrar");

        let duenos = db
            .duenos_de_direcciones(empresa, &["fantasma@empresa.mx".to_owned()])
            .expect("consulta");

        assert!(
            duenos.is_empty(),
            "un contacto de baja con canales vivos salió como dueño y bloquearía la importación"
        );
    }

    /// Las direcciones de otra empresa no cuentan. Sin esto, una importación
    /// rechazaría filas por chocar con datos que no son suyos — y de paso
    /// filtraría que existen.
    #[test]
    fn los_duenos_no_cruzan_empresas() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");

        let otra = db
            .ejecutar_en_pruebas(|conn| {
                let id = CompanyId::nuevo();
                conn.execute(
                    "INSERT INTO company (id, commercial_name, country, timezone,
                                          corporate_email, created_at, updated_at)
                     VALUES (?1, 'Otra', 'MX', 'UTC', 'otra@otra.mx', ?2, ?2)",
                    params![id.to_string(), ahora()],
                )?;
                Ok(id)
            })
            .expect("otra empresa");

        let duenos = db
            .duenos_de_direcciones(otra, &["ana@empresa.mx".to_owned()])
            .expect("consulta");
        assert!(duenos.is_empty());
    }

    /// **El lote.** SQLite tiene un tope de variables por sentencia —999 por
    /// defecto—, así que una consulta con una dirección por interrogante falla
    /// en cuanto el archivo pasa de ahí. Y falla en el archivo grande, que es
    /// justo donde importa.
    ///
    /// Se piden más del doble del tamaño de lote para que se recorra más de una
    /// vez, y una de ellas es real: si el troceado perdiera lotes, no saldría.
    #[test]
    fn se_consultan_en_lotes_para_no_pasar_del_tope_de_sqlite() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "aguja@empresa.mx")]))
            .expect("crea");

        // Por encima del tope medido (32 766). Con 1 500 —lo que pedía la
        // primera versión de esta prueba— el cebo que quitaba el troceado NO
        // rompía nada, porque 1 500 variables caben de sobra. No medía nada.
        let mut direcciones: Vec<String> = (0..40_000)
            .map(|i| format!("pajar{i}@empresa.mx"))
            .collect();
        // La real, al final del todo: si sólo se consultara el primer lote,
        // esto no la encontraría.
        direcciones.push("aguja@empresa.mx".to_owned());

        let duenos = db
            .duenos_de_direcciones(empresa, &direcciones)
            .expect("la consulta no debería pasarse del tope de variables");

        assert_eq!(duenos.len(), 1);
        assert!(duenos.contains_key("aguja@empresa.mx"));
    }

    /// Un contacto sin nombre sale con cadena vacía, no con un espacio suelto
    /// del `nombre || ' ' || apellido`. La pantalla decide qué poner en su
    /// lugar; lo que no puede es recibir « » y creer que hay nombre.
    #[test]
    fn un_contacto_sin_nombre_sale_con_cadena_vacia() {
        let (_d, db, empresa) = base();
        let datos = DatosDeContacto::validar(
            &BorradorDeContacto {
                nombre: String::new(),
                apellido: String::new(),
                empresa: String::new(),
                canales: vec![BorradorDeCanal {
                    canal: Canal::Correo,
                    valor: "anonimo@empresa.mx".into(),
                    principal: true,
                }],
            },
            "MX",
        )
        .expect("válido");
        db.crear_contacto(empresa, &datos).expect("crea");

        let duenos = db
            .duenos_de_direcciones(empresa, &["anonimo@empresa.mx".to_owned()])
            .expect("consulta");
        assert_eq!(
            duenos.get("anonimo@empresa.mx").map(String::as_str),
            Some("")
        );
    }

    /// Dónde está de verdad el tope de variables de este SQLite.
    ///
    /// Se mide preparando sentencias cada vez más anchas hasta que una falla.
    /// No es ceremonia: el comentario del troceado afirmaba «999 por defecto»,
    /// y el cebo que quitaba el troceado **no rompió nada**, así que o la
    /// afirmación era vieja o la prueba no llegaba al tope. Había que medirlo
    /// en vez de repetirlo.
    #[test]
    fn el_tope_de_variables_esta_por_encima_del_lote() {
        let (_d, db, _e) = base();
        let mut tope = 0;
        for n in [
            900_usize, 999, 1_000, 2_000, 10_000, 32_766, 32_767, 100_000,
        ] {
            let sql = format!(
                "SELECT 1 WHERE 1 IN ({})",
                std::iter::repeat_n("?", n).collect::<Vec<_>>().join(",")
            );
            let cabe = db
                .ejecutar_en_pruebas(|conn| Ok(conn.prepare(&sql).is_ok()))
                .unwrap_or(false);
            if cabe {
                tope = n;
            } else {
                break;
            }
        }
        println!("el tope real de variables está entre {tope} y el siguiente escalón");

        assert!(
            tope >= 999,
            "este SQLite no admite ni 999 variables: el troceado tendría que ser más pequeño"
        );
    }

    /// Pedir nada no consulta nada.
    #[test]
    fn pedir_cero_direcciones_no_consulta() {
        let (_d, db, empresa) = base();
        assert!(
            db.duenos_de_direcciones(empresa, &[])
                .expect("consulta")
                .is_empty()
        );
    }

    /// Pedir de más no es un error del usuario: se recorta. Devolver un fallo
    /// dejaría la pantalla vacía en vez de con las primeras mil filas.
    ///
    /// Hacen falta **más contactos que el tope** para que esto mida algo. La
    /// primera versión de esta prueba tenía uno solo: con un contacto, pedir
    /// diez mil también devuelve uno, y el recorte se podía quitar entero sin
    /// que nada se pusiera rojo. Se vio quitándolo.
    #[test]
    fn pedir_una_pagina_desmesurada_se_recorta_en_vez_de_fallar() {
        let (_d, db, empresa) = base();
        sembrar_contactos(&db, empresa, MAX_POR_PAGINA + 5);

        let p = db
            .listar_contactos(
                empresa,
                &FiltroDeContactos::default(),
                0,
                MAX_POR_PAGINA * 10,
            )
            .expect("no falla");

        assert_eq!(
            p.contactos.len() as u32,
            MAX_POR_PAGINA,
            "se pidieron {} filas y volvieron {}: el tope no está recortando",
            MAX_POR_PAGINA * 10,
            p.contactos.len()
        );
        // El total sí es el de verdad: el recorte limita lo que se trae, no lo
        // que se dice que hay. Si no, la paginación de la pantalla mentiría.
        assert_eq!(p.total, MAX_POR_PAGINA + 5);
    }

    /// La lista trae los canales de todos sus contactos, no sólo los del
    /// primero: es lo que la tabla necesita para enseñar el principal de cada
    /// uno sin una consulta por fila.
    #[test]
    fn la_lista_trae_los_canales_de_cada_contacto() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "uno@empresa.mx")]))
            .expect("crea");
        db.crear_contacto(
            empresa,
            &contacto(&[
                (Canal::Correo, "dos@empresa.mx"),
                (Canal::WhatsApp, "+528112345678"),
            ]),
        )
        .expect("crea");

        let p = db
            .listar_contactos(empresa, &FiltroDeContactos::default(), 0, 10)
            .expect("lista");
        assert!(
            p.contactos.iter().all(|c| !c.datos.canales.is_empty()),
            "algún contacto llegó sin canales"
        );
        assert_eq!(
            p.contactos
                .iter()
                .map(|c| c.datos.canales.len())
                .sum::<usize>(),
            3
        );
    }

    // ── L-13 · el aviso de la dirección suprimida ──

    /// La supresión es **de la dirección, no de la persona**. Al cambiar el
    /// correo de alguien, la dirección nueva no hereda la baja de la vieja, y
    /// callarlo convierte corregir un dato en saltarse una baja.
    #[test]
    fn se_avisa_si_una_direccion_esta_suprimida() {
        let (_d, db, empresa) = base();
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO suppression_entry (id, company_id, channel, address_normalized,
                                                reason, origin, created_at)
                 VALUES ('s1', ?1, 'email', 'baja@empresa.mx', 'unsubscribe', 'user', ?2)",
                params![empresa.to_string(), ahora()],
            )
        })
        .expect("suprime");

        let datos = contacto(&[
            (Canal::Correo, "baja@empresa.mx"),
            (Canal::WhatsApp, "+528112345678"),
        ]);
        let avisos = db
            .direcciones_suprimidas(empresa, &datos.canales)
            .expect("consulta");
        assert_eq!(avisos, vec!["baja@empresa.mx".to_owned()]);
    }

    /// El aviso **no bloquea**. Quien edita decide, y la supresión sigue
    /// haciendo su trabajo en el momento del envío (§39).
    #[test]
    fn el_aviso_no_impide_guardar() {
        let (_d, db, empresa) = base();
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO suppression_entry (id, company_id, channel, address_normalized,
                                                reason, origin, created_at)
                 VALUES ('s1', ?1, 'email', 'baja@empresa.mx', 'unsubscribe', 'user', ?2)",
                params![empresa.to_string(), ahora()],
            )
        })
        .expect("suprime");

        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "baja@empresa.mx")]))
            .expect("se puede guardar: el aviso informa, no prohíbe");
    }

    #[test]
    fn sin_canales_que_mirar_no_se_consulta_nada() {
        let (_d, db, empresa) = base();
        assert!(
            db.direcciones_suprimidas(empresa, &[])
                .expect("no falla")
                .is_empty()
        );
    }

    /// Los contactos de una empresa no se ven desde otra. Hoy hay una sola,
    /// pero el §8 deja la puerta abierta a varias y el aislamiento no se añade
    /// después: se comprueba desde el principio.
    #[test]
    fn un_contacto_de_otra_empresa_no_se_lee() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");

        let otra = CompanyId::nuevo();
        assert!(db.contacto(otra, id).expect("lee").is_none());
        assert_eq!(
            db.listar_contactos(otra, &FiltroDeContactos::default(), 0, 10)
                .expect("lista")
                .total,
            0
        );
        assert!(matches!(
            db.editar_contacto(otra, id, &contacto(&[(Canal::Correo, "x@empresa.mx")])),
            Err(DbError::ContactoNoExiste)
        ));
        assert!(matches!(
            db.borrar_contacto(otra, id),
            Err(DbError::ContactoNoExiste)
        ));
    }

    // ── R-26 · EL FILTRO DE LA LISTA ──────────────────────────────────────
    //
    // Las cinco pruebas son de **comportamiento**, y hay una medición aparte
    // para lo que no se puede afirmar con tres filas: que una dirección exacta
    // no cueste un recorrido de la tabla. Esa vive en el banco
    // (`arles-banco/src/contactos.rs`), porque con tres contactos **toda**
    // consulta es instantánea y una prueba verde no diría nada.

    /// Tres contactos distintos, para poder comprobar que el filtro deja fuera
    /// a los que no piden.
    fn tres_distintos(db: &Db, empresa: CompanyId) {
        for (nombre, apellido, emp, correo) in [
            ("Ana", "Ruiz", "Müller Logística", "ana.ruiz@muller.mx"),
            ("Beto", "Salas", "Café del Norte", "beto@cafenorte.mx"),
            (
                "Caro",
                "Téllez",
                "Müller Logística",
                "caro.tellez@muller.mx",
            ),
        ] {
            let d = DatosDeContacto::validar(
                &BorradorDeContacto {
                    nombre: nombre.into(),
                    apellido: apellido.into(),
                    empresa: emp.into(),
                    canales: vec![BorradorDeCanal {
                        canal: Canal::Correo,
                        valor: correo.into(),
                        principal: true,
                    }],
                },
                "MX",
            )
            .expect("contacto válido");
            db.crear_contacto(empresa, &d).expect("crea");
        }
    }

    #[test]
    fn el_texto_busca_en_el_nombre_el_apellido_y_la_empresa() {
        let (_d, db, empresa) = base();
        tres_distintos(&db, empresa);

        // Los tres campos, uno por uno, y el total **con el filtro puesto**:
        // es lo que la pantalla enseña al lado de la página, y si contara sin
        // filtrar diría «1 de 3» con un 3 que no corresponde a nada.
        for (busca, cuantos) in [
            ("ana", 1),    // el nombre
            ("salas", 1),  // el apellido
            ("müller", 2), // la empresa, y casa a dos
            ("TÉLLEZ", 1), // sin distinguir mayúsculas
            ("z", 2),      // un trozo: Ruiz y Téllez
            ("nadie", 0),
        ] {
            let f = FiltroDeContactos {
                texto: Some(busca),
                ..Default::default()
            };
            let p = db.listar_contactos(empresa, &f, 0, 100).expect("lista");
            assert_eq!(
                (p.contactos.len(), p.total),
                (cuantos, u32::try_from(cuantos).expect("cabe")),
                "buscando «{busca}»"
            );
        }
    }

    /// Un comodín del `LIKE` se busca como el carácter que es.
    ///
    /// ── LA PRIMERA VERSIÓN DE ESTA PRUEBA PASABA ROTA, Y VALE CONTARLO ──
    ///
    /// Comprobaba que buscar `%` diera **cero** contactos. Y quitando el
    /// `ESCAPE` de la consulta seguía dando cero, así que **pasaba en verde
    /// con el defecto dentro**: sin `ESCAPE`, la barra que pone
    /// `patron_de_busqueda` deja de ser un escape y el patrón `%\%%` busca una
    /// barra literal, que no está en ningún nombre. Cero por accidente, no por
    /// la garantía.
    ///
    /// La garantía son **dos mitades** —escapar en Rust y declarar el `ESCAPE`
    /// en SQL— y hace falta un caso que distinga las tres situaciones. Lo da un
    /// contacto que de verdad lleve un `%` en su nombre:
    ///
    /// - Correcto → encuentra **ése y sólo ése**.
    /// - Sin el `ESCAPE` en SQL → busca `\%` literal y encuentra **cero**.
    /// - Sin escapar en Rust → el `%` es comodín y encuentra **todos**.
    ///
    /// **Verificada por los dos caminos, los dos en rojo.** Es H-13 otra vez,
    /// el defecto que el filtro del registro tuvo desde la 7.4 hasta el
    /// 02/11/2026.
    #[test]
    fn un_comodin_de_like_se_busca_como_el_caracter_que_es() {
        let (_d, db, empresa) = base();
        tres_distintos(&db, empresa);

        // Un nombre con los tres caracteres que el `LIKE` trata aparte. No es
        // un nombre verosímil, y da igual: lo que se comprueba es que un
        // carácter del texto no cambie el significado de la consulta.
        let raro = DatosDeContacto::validar(
            &BorradorDeContacto {
                nombre: "Fito".into(),
                apellido: "100%_\\Puro".into(),
                empresa: "Rara SA".into(),
                canales: vec![BorradorDeCanal {
                    canal: Canal::Correo,
                    valor: "fito@rara.mx".into(),
                    principal: true,
                }],
            },
            "MX",
        )
        .expect("contacto válido");
        db.crear_contacto(empresa, &raro).expect("crea");

        for comodin in ["%", "_", "\\"] {
            let f = FiltroDeContactos {
                texto: Some(comodin),
                ..Default::default()
            };
            let p = db.listar_contactos(empresa, &f, 0, 100).expect("lista");
            assert_eq!(
                p.total, 1,
                "«{comodin}» tiene que encontrar al contacto que lo lleva en el \
                 apellido, y sólo a ése: cero significa que el escapado no se \
                 interpreta, y cuatro que sigue siendo un comodín"
            );
            assert_eq!(
                p.contactos.first().map(|c| c.datos.nombre.as_str()),
                Some("Fito")
            );
        }
    }

    /// La dirección se normaliza con el validador del núcleo, así que entra
    /// como la escriba quien la escriba.
    ///
    /// **Verificada rompiéndola**: comparando lo tecleado tal cual en vez de
    /// normalizarlo, los casos con mayúsculas y el teléfono con espacios
    /// devuelven cero — y sin ningún error, que es lo que lo hacía peligroso.
    #[test]
    fn la_direccion_entra_como_la_escriba_quien_la_escriba() {
        let (_d, db, empresa) = base();
        // Con una dirección propia, y no la de Ana de `tres_distintos`: el
        // índice único de `contact_channel` rechaza la segunda —correctamente,
        // y así se vio: la primera versión de esta prueba cayó con
        // `DireccionEnUso` antes de llegar a comprobar nada—.
        let d = DatosDeContacto::validar(
            &BorradorDeContacto {
                nombre: "Elena".into(),
                apellido: "Nava".into(),
                empresa: "Müller".into(),
                canales: vec![
                    BorradorDeCanal {
                        canal: Canal::Correo,
                        valor: "elena.nava@muller.mx".into(),
                        principal: true,
                    },
                    BorradorDeCanal {
                        canal: Canal::WhatsApp,
                        valor: "+52 55 1234 5678".into(),
                        principal: false,
                    },
                ],
            },
            "MX",
        )
        .expect("contacto válido");
        db.crear_contacto(empresa, &d).expect("crea");
        tres_distintos(&db, empresa);

        for escrito in [
            "elena.nava@muller.mx",
            "  Elena.Nava@Muller.MX  ",
            "+525512345678",
            "55 1234 5678",
        ] {
            let f = FiltroDeContactos {
                direccion: Some(escrito),
                ..Default::default()
            };
            let p = db.listar_contactos(empresa, &f, 0, 100).expect("lista");
            assert_eq!(p.total, 1, "escribiendo «{escrito}»");
        }

        // Y lo que no es una dirección da una página vacía, no un error: quien
        // está escribiendo todavía no ha terminado de escribir.
        let f = FiltroDeContactos {
            direccion: Some("pepe"),
            ..Default::default()
        };
        let p = db.listar_contactos(empresa, &f, 0, 100).expect("lista");
        assert_eq!((p.total, p.contactos.len()), (0, 0));
    }

    #[test]
    fn el_canal_y_la_baja_recortan_la_lista() {
        let (_d, db, empresa) = base();
        tres_distintos(&db, empresa);
        // Uno más, con WhatsApp.
        let d = DatosDeContacto::validar(
            &BorradorDeContacto {
                nombre: "Dani".into(),
                apellido: "Vega".into(),
                empresa: "Müller".into(),
                canales: vec![BorradorDeCanal {
                    canal: Canal::WhatsApp,
                    valor: "+52 55 9999 0000".into(),
                    principal: true,
                }],
            },
            "MX",
        )
        .expect("contacto válido");
        db.crear_contacto(empresa, &d).expect("crea");

        let solo_wa = FiltroDeContactos {
            canal: Some(Canal::WhatsApp),
            ..Default::default()
        };
        assert_eq!(
            db.listar_contactos(empresa, &solo_wa, 0, 100)
                .expect("lista")
                .total,
            1
        );
        let solo_correo = FiltroDeContactos {
            canal: Some(Canal::Correo),
            ..Default::default()
        };
        assert_eq!(
            db.listar_contactos(empresa, &solo_correo, 0, 100)
                .expect("lista")
                .total,
            3
        );

        // Una baja de la dirección de Beto.
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO suppression_entry
                    (id, company_id, channel, address_normalized, scope, reason, origin, created_at)
                 VALUES ('s-1', ?1, 'email', 'beto@cafenorte.mx', 'channel',
                         'unsubscribe', 'user', ?2)",
                params![empresa.to_string(), ahora()],
            )
        })
        .expect("siembra la baja");

        let con_baja = FiltroDeContactos {
            con_alguna_baja: Some(true),
            ..Default::default()
        };
        let p = db
            .listar_contactos(empresa, &con_baja, 0, 100)
            .expect("lista");
        assert_eq!(p.total, 1, "sólo Beto tiene una dirección dada de baja");
        assert_eq!(
            p.contactos.first().map(|c| c.datos.apellido.as_str()),
            Some("Salas")
        );

        let sin_baja = FiltroDeContactos {
            con_alguna_baja: Some(false),
            ..Default::default()
        };
        assert_eq!(
            db.listar_contactos(empresa, &sin_baja, 0, 100)
                .expect("lista")
                .total,
            3,
            "los otros tres, Dani incluido"
        );
    }

    /// Los cuatro criterios a la vez, que es lo que ninguna prueba de uno solo
    /// cubre: cada uno por separado puede estar bien y el `AND` mal armado.
    #[test]
    fn los_criterios_se_combinan_con_y_y_no_con_o() {
        let (_d, db, empresa) = base();
        tres_distintos(&db, empresa);

        // «müller» casa a dos, y la dirección de Caro a uno. Juntos, uno.
        let f = FiltroDeContactos {
            texto: Some("müller"),
            direccion: Some("caro.tellez@muller.mx"),
            canal: Some(Canal::Correo),
            con_alguna_baja: Some(false),
        };
        let p = db.listar_contactos(empresa, &f, 0, 100).expect("lista");
        assert_eq!(p.total, 1);
        assert_eq!(
            p.contactos.first().map(|c| c.datos.nombre.as_str()),
            Some("Caro")
        );

        // Y dos criterios que no se cumplen a la vez dan cero, no la unión.
        let f = FiltroDeContactos {
            texto: Some("café"),
            direccion: Some("caro.tellez@muller.mx"),
            ..Default::default()
        };
        assert_eq!(
            db.listar_contactos(empresa, &f, 0, 100)
                .expect("lista")
                .total,
            0,
            "con un O aquí saldrían dos"
        );
    }
}

#[cfg(test)]
mod reproduccion_h05 {
    //! Reproducción del H-05, reportado por Dirección el 16/10/2026: dar de
    //! alta un contacto a mano devolvió «Ocurrió un error al acceder a los
    //! datos».
    //!
    //! Hace EXACTAMENTE lo de la captura: empresa configurada, lista vacía, y
    //! un contacto con nombre, apellido, empresa y un solo correo.
    //!
    //! ─────────────────────────────────────────────────────────────────────
    //! ESTAS PRUEBAS PASARON DESDE EL PRIMER DÍA, Y ÉSA ERA LA PISTA
    //!
    //! El 17/10/2026 se escribieron para reproducir el fallo y **pasaron
    //! enteras**, así que el defecto se declaró «sin causa conocida» y se
    //! arregló el mensaje para que trajera el detalle técnico.
    //!
    //! Ese detalle, que Dirección mandó el 20/10/2026, dijo dónde estaba:
    //!
    //! ```text
    //! unknown variant `email`, expected `correo` or `whats_app`
    //! ```
    //!
    //! **El alta nunca llegaba hasta aquí.** Fallaba al cruzar la frontera
    //! IPC, deserializando el borrador: `Canal` se llamaba `email` en la
    //! interfaz y `correo` en serde. Estas pruebas construyen el borrador en
    //! Rust, así que se saltan exactamente el tramo que estaba roto — por eso
    //! pasaban, y por eso tenían que pasar.
    //!
    //! La lección: **reproducir un fallo en la capa de abajo no lo reproduce**
    //! si lo que falla es la junta de arriba. Lo que faltaba era una prueba
    //! que serializara y deserializara, y es
    //! `arles-core/tests/nombres_de_la_frontera.rs`.
    use super::FiltroDeContactos;
    use super::tests::base;
    use arles_core::Canal;
    use arles_core::contacto::{BorradorDeCanal, BorradorDeContacto, DatosDeContacto};

    #[test]
    fn el_alta_a_mano_de_la_captura_de_direccion() {
        let (_d, db, empresa) = base();

        let antes = db
            .listar_contactos(empresa, &FiltroDeContactos::default(), 100, 0)
            .expect("leer la lista vacía");
        assert_eq!(
            antes.contactos.len(),
            0,
            "la base empieza vacía, como en la captura"
        );

        let borrador = BorradorDeContacto {
            nombre: "Adrian".into(),
            apellido: "Paz".into(),
            empresa: "Insight".into(),
            canales: vec![BorradorDeCanal {
                canal: Canal::Correo,
                valor: "apazg@outlook.com".into(),
                principal: true,
            }],
        };
        let datos =
            DatosDeContacto::validar(&borrador, "MX").expect("el borrador de la captura es válido");

        let r = db.crear_contacto(empresa, &datos);
        assert!(r.is_ok(), "el alta de la captura falló con: {:?}", r.err());

        // Y lo que la pantalla hace JUSTO DESPUÉS: recargar. Aquí la lista ya
        // trae un canal, así que `direcciones_suprimidas` recibe algo — con
        // cero contactos devolvía pronto y ese tramo no se ejercitaba.
        let despues = db
            .listar_contactos(empresa, &FiltroDeContactos::default(), 0, 100)
            .expect("recargar tras el alta");
        assert_eq!(despues.contactos.len(), 1, "el contacto tiene que estar");

        let canales: Vec<_> = despues
            .contactos
            .iter()
            .flat_map(|c| c.datos.canales.iter().cloned())
            .collect();
        let supr = db.direcciones_suprimidas(empresa, &canales);
        assert!(
            supr.is_ok(),
            "las suprimidas fallaron con: {:?}",
            supr.err()
        );
    }

    /// El segundo contacto, que es donde se cruzan índices únicos.
    #[test]
    fn dos_altas_seguidas_a_mano() {
        let (_d, db, empresa) = base();
        for (n, correo) in [("Adrian", "apazg@outlook.com"), ("Ana", "ana@empresa.mx")] {
            let b = BorradorDeContacto {
                nombre: n.into(),
                apellido: "Paz".into(),
                empresa: "Insight".into(),
                canales: vec![BorradorDeCanal {
                    canal: Canal::Correo,
                    valor: correo.into(),
                    principal: true,
                }],
            };
            let d = DatosDeContacto::validar(&b, "MX").expect("válido");
            let r = db.crear_contacto(empresa, &d);
            assert!(r.is_ok(), "el alta de {correo} falló con: {:?}", r.err());
        }
    }
}
