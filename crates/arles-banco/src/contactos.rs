//! Qué cuesta filtrar la lista de contactos, medido sobre medio millón.
//!
//! Entrega R-26, 03/11/2026.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTO EXISTE, Y POR QUÉ ANTES DE ESCRIBIR EL FILTRO
//!
//! C-2 —el filtro del registro— se construyó siguiendo una recomendación que
//! el propio proyecto llevaba un año repitiéndose en un comentario, **sin
//! medirla**, y resultó ser 18 veces peor que lo que venía a sustituir. La
//! conclusión quedó escrita el 02/11/2026: *una recomendación sin medir es una
//! afirmación como cualquier otra*.
//!
//! Así que el filtro de contactos se mide antes de diseñarse. Lo que decide
//! qué criterios se ofrecen **no es qué es cómodo de escribir en SQL**, es qué
//! contesta en el presupuesto de la pantalla (< 500 ms, §0 del presupuesto).
//!
//! Y la diferencia con C-2 importa: el registro buscaba texto libre en una
//! columna sin índice posible. Aquí hay **dos preguntas distintas** metidas en
//! la misma palabra «buscar»:
//!
//! - **Una dirección** es un valor exacto con índice único desde la V11
//!   (`idx_channel_direccion`). Se contesta en microsegundos.
//! - **Un nombre** es texto libre sobre `contact`, que no tiene ningún índice
//!   para eso.
//!
//! Medirlas juntas daría una cifra media que no describe ninguna de las dos.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE SE MIDE DE CADA CAMINO
//!
//! El tiempo de **tres corridas** y el **plan** (`EXPLAIN QUERY PLAN`). Las
//! dos cosas, porque una sola no basta: un tiempo sin plan no dice por qué, y
//! un plan sin tiempo fue exactamente el error de C-2 —el FTS5 usaba un índice
//! y era 18 veces más lento—.
//!
//! Y **la página y el recuento por separado.** En el registro, contar todas
//! las coincidencias costaba lo mismo que traer las 500 filas de la página, y
//! eso es lo que convirtió un problema técnico en la decisión D-19. Si aquí
//! pasa igual, hay que saberlo antes de prometer un total.

use std::time::{Duration, Instant};

/// Una corrida de un camino: tres tiempos, el plan y cuántas filas salieron.
#[derive(Debug)]
pub struct Camino {
    pub nombre: &'static str,
    /// Lo que el usuario escribiría. Vacío si el camino no lleva texto.
    pub busca: String,
    pub tiempos: Vec<Duration>,
    pub filas: u32,
    pub plan: Vec<String>,
}

impl Camino {
    fn ms(&self) -> String {
        self.tiempos
            .iter()
            .map(|t| format!("{:.1}", t.as_secs_f64() * 1000.0))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// La mediana de las tres corridas, en milisegundos.
    fn mediana(&self) -> f64 {
        let mut v: Vec<f64> = self
            .tiempos
            .iter()
            .map(|t| t.as_secs_f64() * 1000.0)
            .collect();
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    }
}

/// El resultado: los mismos caminos a dos tamaños.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ DOS TAMAÑOS Y NO UN PLAN
///
/// La pregunta que hay que contestar es «¿este criterio recorre la tabla
/// entera?», y con **una sola empresa** —que es todo ARLES, ADR-0008— el plan
/// no la contesta: `idx_contact_lista` empieza por `company_id`, así que
/// SQLite escribe `SEARCH contact USING INDEX idx_contact_lista (company_id=?)`
/// tanto cuando se para en la fila 100 como cuando visita las 455 864.
///
/// La primera versión de esta medición derivaba un «¿barre?» del plan y
/// **salió «no» nueve veces de nueve**, incluidos los dos caminos que tardaban
/// 1 500 ms. Al corregirla salió «sí» diez de diez, incluida la página sin
/// filtro, que tarda 0.1 ms. Una medida que contesta siempre lo mismo no mide.
///
/// Lo que no se puede falsear es **cómo crece**: un camino que se para pronto
/// tarda lo mismo con cuatro veces más filas; uno que las recorre, cuatro
/// veces más. Así que se corre a `n/4` y a `n`, y la columna que decide es la
/// proporción. No se ajusta el instrumento hasta que calle: se cambia por uno
/// que pueda decir las dos cosas.
#[derive(Debug)]
pub struct Corrida {
    pub pequena: u32,
    pub grande: u32,
    /// Los caminos de la base pequeña y los de la grande, emparejados.
    pub caminos: Vec<(Camino, Camino)>,
}

impl std::fmt::Display for Corrida {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "\nEl filtro de la lista de contactos · {} y {} contactos\n",
            self.pequena, self.grande
        )?;
        let veces = f64::from(self.grande) / f64::from(self.pequena);
        writeln!(
            f,
            "La base grande tiene {veces:.1}× las filas de la pequeña, así que un\n\
             camino que recorra la tabla entera tiene que tardar unas {veces:.1}×\n\
             más. Uno que se pare pronto, lo mismo.\n"
        )?;
        writeln!(
            f,
            "| Camino | Qué busca | Filas | {} filas (ms) | {} filas (ms) | Crece |",
            self.pequena, self.grande
        )?;
        writeln!(f, "|---|---|---|---|---|---|")?;
        for (p, g) in &self.caminos {
            let crece = g.mediana() / p.mediana().max(0.01);
            writeln!(
                f,
                "| {} | {} | {} | {} | {} | **{crece:.1}×** |",
                g.nombre,
                if g.busca.is_empty() { "—" } else { &g.busca },
                g.filas,
                p.ms(),
                g.ms(),
            )?;
        }
        writeln!(f, "\nLos planes de la base grande, uno a uno:\n")?;
        for (_, g) in &self.caminos {
            writeln!(f, "  {}", g.nombre)?;
            for p in &g.plan {
                writeln!(f, "      {p}")?;
            }
        }
        Ok(())
    }
}

/// Mide los caminos candidatos del filtro sobre una base recién importada.
///
/// # Errores
///
/// Lo que falle al generar, abrir la base o importar.
pub fn correr(cuantas: usize, semilla: u64) -> Result<Corrida, Box<dyn std::error::Error>> {
    // La pequeña va **con la misma semilla**: así sus filas son un prefijo de
    // las de la grande y el nombre y la dirección de muestra existen en las
    // dos. Con semillas distintas se estaría comparando la proporción de dos
    // conjuntos diferentes, que es otra cosa.
    let pequena = una_base(cuantas / 4, semilla)?;
    let grande = una_base(cuantas, semilla)?;
    if pequena.len() != grande.len() {
        return Err("las dos corridas no midieron los mismos caminos".into());
    }
    Ok(Corrida {
        pequena: pequena[0].contactos,
        grande: grande[0].contactos,
        caminos: pequena
            .into_iter()
            .map(|p| p.camino)
            .zip(grande.into_iter().map(|g| g.camino))
            .collect(),
    })
}

/// Un camino medido, con cuántos contactos tenía la base.
struct Medido {
    camino: Camino,
    contactos: u32,
}

fn una_base(cuantas: usize, semilla: u64) -> Result<Vec<Medido>, Box<dyn std::error::Error>> {
    use arles_core::ids::CompanyId;
    use arles_db::{ClaveMaestra, Db};

    let dir = tempfile::tempdir()?;
    let csv = dir.path().join("sintetico.csv");
    eprintln!("generando {cuantas} filas…");
    crate::escribir_csv(&csv, cuantas, semilla)?;

    let clave = ClaveMaestra::generar()?;
    let db = Db::abrir(&dir.path().join("arles.db"), &clave)?;
    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(
            "INSERT INTO company (id, commercial_name, country, timezone, corporate_email, created_at, updated_at)
               VALUES ('00000000-0000-4000-8000-000000000001', 'Banco', 'MX', 'America/Mexico_City',
                       'banco@ejemplo.mx', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');",
        )
    })?;
    let empresa: CompanyId = "00000000-0000-4000-8000-000000000001"
        .parse()
        .map_err(|e| format!("id de empresa: {e:?}"))?;

    eprintln!("importando…");
    let tabla = arles_import::leer(&csv)?;
    let t = Instant::now();
    crate::medir::importar_para_el_wal(&db, empresa, &tabla)?;
    eprintln!("importado en {:.1} s", t.elapsed().as_secs_f64());

    // Un nombre y una dirección que **existen**. Buscar algo que no está mide
    // el índice igual de bien y el camino del usuario nada: quien filtra está
    // buscando a alguien, no comprobando que no esté.
    let fila = crate::generar(cuantas.min(1000), semilla)
        .find(|f| !f.nombre.trim().is_empty() && f.correo.contains('@'))
        .ok_or("ninguna fila sirve de muestra")?;
    let nombre = fila.nombre.trim().to_lowercase();
    let correo = fila.correo.trim().to_lowercase();
    let dominio = correo
        .rsplit('@')
        .next()
        .ok_or("la muestra no tiene dominio")?
        .to_owned();

    let emp = empresa.to_string();
    // Con capacidad y no `Vec::new()`: son veinte caminos conocidos, y además
    // clippy pide un `vec![...]` cuando ve un `new()` seguido de `push`. Aquí
    // no cabe un literal — cada `medir` es fallible y lleva encima el
    // comentario de por qué ese camino está en la lista.
    let mut caminos: Vec<Camino> = Vec::with_capacity(20);

    // ── 1 · La página de hoy, sin filtro. La referencia. ──────────────────
    caminos.push(medir(
        &db,
        "La página de hoy, sin filtro",
        String::new(),
        "SELECT id FROM contact
          WHERE company_id = ?1 AND deleted_at IS NULL
          ORDER BY created_at DESC, id DESC LIMIT 100",
        &[&emp],
    )?);

    // ── 2 · El recuento de hoy, sin filtro ────────────────────────────────
    //
    // Va aparte porque en el registro **costaba lo mismo que la página**, y
    // eso es lo que hay que saber antes de prometer un total.
    caminos.push(medir(
        &db,
        "El recuento de hoy, sin filtro",
        String::new(),
        "SELECT count(*) FROM contact WHERE company_id = ?1 AND deleted_at IS NULL",
        &[&emp],
    )?);

    // ── 3 · Nombre, cualquier trozo ───────────────────────────────────────
    //
    // Es la forma que la gente espera de un buscador, y la que no tiene
    // índice posible: un `%algo%` no puede usar un árbol ordenado por el
    // principio de la cadena.
    caminos.push(medir(
        &db,
        "Nombre, cualquier trozo (`%x%`)",
        nombre.clone(),
        "SELECT id FROM contact
          WHERE company_id = ?1 AND deleted_at IS NULL
            AND (lower(COALESCE(first_name,'')) LIKE '%' || ?2 || '%'
              OR lower(COALESCE(last_name,'')) LIKE '%' || ?2 || '%')
          ORDER BY created_at DESC, id DESC LIMIT 100",
        &[&emp, &nombre],
    )?);

    // ── 3 bis · Un nombre que no existe ───────────────────────────────────
    //
    // ── EL CASO QUE DECIDE, Y NO ES EL QUE PARECE ──
    //
    // El camino 3 busca «joaquín», que en el conjunto sintético —y en una
    // lista de verdad— sale cientos de veces. Y por eso es **rápido**: el
    // `LIMIT 100` se cumple a las pocas filas y la consulta se para.
    //
    // Eso no mide el filtro: mide la frecuencia del nombre. Lo que de verdad
    // cuesta es **lo que no encuentra**, porque entonces el `LIMIT` no se
    // cumple nunca y hay que visitar las 455 864 filas para poder decir «no
    // hay nada». Es lo contrario de lo que dice la intuición, y es la misma
    // forma del hallazgo de C-2: cuanto más raro lo que se busca, peor.
    //
    // Quien teclea y se equivoca está en este caso, así que no es un caso
    // raro: es el que va a tocar varias veces por búsqueda, mientras escribe.
    caminos.push(medir(
        &db,
        "Nombre que NO existe (0 coincidencias)",
        "zzqx".to_owned(),
        "SELECT id FROM contact
          WHERE company_id = ?1 AND deleted_at IS NULL
            AND (lower(COALESCE(first_name,'')) LIKE '%' || ?2 || '%'
              OR lower(COALESCE(last_name,'')) LIKE '%' || ?2 || '%')
          ORDER BY created_at DESC, id DESC LIMIT 100",
        &[&emp, &"zzqx".to_owned()],
    )?);

    // ── 4 · Nombre, por el principio ──────────────────────────────────────
    //
    // La misma pregunta acotada: `ana%` en vez de `%ana%`. Sin índice barre
    // igual; con uno sería un rango. Se mide sin él para tener el antes.
    caminos.push(medir(
        &db,
        "Nombre, por el principio (`x%`)",
        nombre.clone(),
        "SELECT id FROM contact
          WHERE company_id = ?1 AND deleted_at IS NULL
            AND lower(COALESCE(first_name,'')) LIKE ?2 || '%'
          ORDER BY created_at DESC, id DESC LIMIT 100",
        &[&emp, &nombre],
    )?);

    // ── 5 · Dirección exacta, con el JOIN de la forma obvia ───────────────
    //
    // Tiene índice único desde la V11. Es la pregunta que de verdad se hace
    // quien busca a una persona concreta: pega la dirección del correo que
    // recibió. **Y aquí hay un hallazgo que ninguna prueba cazaría**, porque
    // el resultado es correcto: ver el camino 6.
    caminos.push(medir(
        &db,
        "Dirección exacta · con JOIN",
        correo.clone(),
        "SELECT c.id FROM contact c
           JOIN contact_channel k ON k.contact_id = c.id AND k.deleted_at IS NULL
          WHERE c.company_id = ?1 AND c.deleted_at IS NULL
            AND k.company_id = ?1 AND k.value_normalized = ?2
          ORDER BY c.created_at DESC, c.id DESC LIMIT 100",
        &[&emp, &correo],
    )?);

    // ── 6 · La misma pregunta, con un subconsulta ─────────────────────────
    //
    // ── EL ÍNDICE ESTABA Y LA CONSULTA NO LO USABA ──
    //
    // Con el `JOIN` del camino 5, SQLite **conduce desde `contact`**: elige
    // `idx_contact_lista` por su `ORDER BY`, recorre las filas de la empresa
    // una por una y para cada una pregunta por su dirección. El índice único
    // de la dirección se usa, pero del lado que no sirve: en vez de ir de la
    // dirección al contacto —uno—, va del contacto a la dirección —todos—.
    //
    // La forma que deja usarlo bien es decirle **primero qué contactos**, y
    // eso es una subconsulta y no una junta. Es lo que ya hace el filtro del
    // registro (7.4) por el mismo motivo.
    caminos.push(medir(
        &db,
        "Dirección exacta · con subconsulta",
        correo.clone(),
        "SELECT id FROM contact
          WHERE company_id = ?1 AND deleted_at IS NULL
            AND id IN (SELECT contact_id FROM contact_channel
                        WHERE company_id = ?1 AND value_normalized = ?2
                          AND deleted_at IS NULL)
          ORDER BY created_at DESC, id DESC LIMIT 100",
        &[&emp, &correo],
    )?);

    // ── 6 bis · La misma, conduciendo desde el canal ──────────────────────
    //
    // Los caminos 5 y 6 tardan lo mismo y crecen con la tabla aunque el
    // índice único les dé **una** fila. La razón no es el índice: es el
    // `ORDER BY`. SQLite recorre `idx_contact_lista` en orden y va probando
    // cada contacto contra la lista, y como sólo hay una coincidencia el
    // `LIMIT 100` **no se cumple nunca**: hay que llegar al final.
    //
    // Aquí se prueba al revés: conducir desde el canal —una fila— y buscar el
    // contacto por su clave primaria. Si funciona, el orden se resuelve sobre
    // un puñado de filas en vez de sobre media base.
    caminos.push(medir(
        &db,
        "Dirección exacta · desde el canal",
        correo.clone(),
        "SELECT c.id FROM contact_channel k
           JOIN contact c ON c.id = k.contact_id
          WHERE k.company_id = ?1 AND k.value_normalized = ?2 AND k.deleted_at IS NULL
            AND c.deleted_at IS NULL
          ORDER BY c.created_at DESC, c.id DESC LIMIT 100",
        &[&emp, &correo],
    )?);

    // ── 7 · Dirección, por el principio ───────────────────────────────────
    //
    // El mismo índice sirve para un prefijo: un árbol ordenado por la cadena
    // acota un `abc%` a un rango. Es lo que permitiría ofrecer «empieza por»
    // sin pagar un recorrido — si el plan lo confirma.
    let prefijo = correo
        .split('@')
        .next()
        .unwrap_or(&correo)
        .chars()
        .take(4)
        .collect::<String>();
    caminos.push(medir(
        &db,
        "Dirección, por el principio · subconsulta",
        format!("{prefijo}…"),
        "SELECT id FROM contact
          WHERE company_id = ?1 AND deleted_at IS NULL
            AND id IN (SELECT contact_id FROM contact_channel
                        WHERE company_id = ?1 AND deleted_at IS NULL
                          AND value_normalized >= ?2
                          AND value_normalized < ?2 || char(255))
          ORDER BY created_at DESC, id DESC LIMIT 100",
        &[&emp, &prefijo],
    )?);

    // ── 7 bis · La misma, desde el canal ──────────────────────────────────
    //
    // ── Y AQUÍ LA FORMA BUENA ES LA OTRA, QUE ES LO QUE HABÍA QUE MEDIR ──
    //
    // Conducir desde el canal fue 600× mejor para una dirección exacta
    // (camino 6 bis). Para un prefijo es **2.7× peor**: el rango casa muchas
    // filas de canal, y entonces el `DISTINCT` y el `ORDER BY` hay que
    // resolverlos en un árbol temporal sobre todas ellas en vez de pararse a
    // las cien.
    //
    // **No hay una forma buena de consulta: hay dos, y cuál gana depende de
    // cuántas filas case el filtro** — que es justo lo que no se sabe al
    // escribir el SQL. Por eso las dos se quedan medidas aquí.
    caminos.push(medir(
        &db,
        "Dirección, por el principio · desde el canal",
        format!("{prefijo}…"),
        "SELECT DISTINCT c.id FROM contact_channel k
           JOIN contact c ON c.id = k.contact_id
          WHERE k.company_id = ?1 AND k.deleted_at IS NULL
            AND k.value_normalized >= ?2
            AND k.value_normalized < ?2 || char(255)
            AND c.deleted_at IS NULL
          ORDER BY c.created_at DESC, c.id DESC LIMIT 100",
        &[&emp, &prefijo],
    )?);

    // ── 8 · Dirección, cualquier trozo ────────────────────────────────────
    //
    // Lo que de verdad pide alguien que trabaja por empresas: «todos los de
    // tal dominio». Y un `%@dominio` es un **sufijo**: ningún índice ordenado
    // por el principio de la cadena puede acotarlo. Es la misma pared que C-2,
    // y aquí se mide en vez de suponerse.
    caminos.push(medir(
        &db,
        "Dirección, cualquier trozo · subconsulta",
        format!("@{dominio}"),
        "SELECT id FROM contact
          WHERE company_id = ?1 AND deleted_at IS NULL
            AND id IN (SELECT contact_id FROM contact_channel
                        WHERE company_id = ?1 AND deleted_at IS NULL
                          AND value_normalized LIKE '%' || ?2 || '%')
          ORDER BY created_at DESC, id DESC LIMIT 100",
        &[&emp, &dominio],
    )?);

    // ── 8 · Qué canal tiene ───────────────────────────────────────────────
    //
    // «Sólo los que tienen WhatsApp» no es texto: es una pregunta sobre la
    // existencia de una fila, y se contesta con el índice del canal.
    caminos.push(medir(
        &db,
        "Sólo los que tienen WhatsApp",
        String::new(),
        "SELECT c.id FROM contact c
          WHERE c.company_id = ?1 AND c.deleted_at IS NULL
            AND EXISTS (SELECT 1 FROM contact_channel k
                         WHERE k.contact_id = c.id AND k.deleted_at IS NULL
                           AND k.channel = 'whatsapp')
          ORDER BY c.created_at DESC, c.id DESC LIMIT 100",
        &[&emp],
    )?);

    // ── 9 · La empresa del contacto ───────────────────────────────────────
    //
    // Vive en la columna `source` (trampa 1 de la Fase 6) y no tiene índice.
    caminos.push(medir(
        &db,
        "Empresa del contacto, exacta",
        fila.empresa.trim().to_lowercase(),
        "SELECT id FROM contact
          WHERE company_id = ?1 AND deleted_at IS NULL
            AND lower(COALESCE(source,'')) = ?2
          ORDER BY created_at DESC, id DESC LIMIT 100",
        &[&emp, &fila.empresa.trim().to_lowercase()],
    )?);

    // ── 9 bis · DÓNDE SE VA EL FACTOR 25, AISLADO ─────────────────────────
    //
    // El camino entregado con un texto que no existe tarda 22 segundos, y el
    // candidato equivalente 899 ms. Son la misma pregunta, así que la
    // diferencia está en una de tres cosas: el `ESCAPE` del patrón, el
    // recuento sin tope, o la conexión de lectura.
    //
    // **No se adivina: se parte en tres y se mide cada pedazo.** Es la lección
    // de la auditoría C —la causa obvia de los 73 s de la activación era la
    // generación de identificadores, y medida aparte resultó ser el 0.8 %—.
    let patron = format!("%{}%", "zzqx");
    caminos.push(medir(
        &db,
        "Sin resultados · la página, con ESCAPE",
        "zzqx".to_owned(),
        "SELECT c.id FROM contact c
          WHERE c.company_id = ?1 AND c.deleted_at IS NULL
            AND (lower(COALESCE(c.first_name,'')) LIKE ?2 ESCAPE '\\'
              OR lower(COALESCE(c.last_name,''))  LIKE ?2 ESCAPE '\\'
              OR lower(COALESCE(c.source,''))     LIKE ?2 ESCAPE '\\')
          ORDER BY c.created_at DESC, c.id DESC LIMIT 100",
        &[&emp, &patron],
    )?);
    caminos.push(medir(
        &db,
        "Sin resultados · la página, SIN ESCAPE",
        "zzqx".to_owned(),
        "SELECT c.id FROM contact c
          WHERE c.company_id = ?1 AND c.deleted_at IS NULL
            AND (lower(COALESCE(c.first_name,'')) LIKE ?2
              OR lower(COALESCE(c.last_name,''))  LIKE ?2
              OR lower(COALESCE(c.source,''))     LIKE ?2)
          ORDER BY c.created_at DESC, c.id DESC LIMIT 100",
        &[&emp, &patron],
    )?);
    // ── Y LA MISMA CONSULTA POR LA LECTORA, QUE ES LO QUE FALTABA ──
    //
    // Las tres de arriba van por el escritor, que es por donde van **todas**
    // las mediciones del banco desde la 8.5. Y la pantalla lee por la lectora
    // de ADR-0015. Si las dos cifras no coinciden, todo lo publicado de
    // cualquier pantalla está medido por un camino que el usuario no recorre.
    caminos.push(medir_por(
        &db,
        "Sin resultados · la página, POR LA LECTORA",
        "zzqx".to_owned(),
        "SELECT c.id FROM contact c
          WHERE c.company_id = ?1 AND c.deleted_at IS NULL
            AND (lower(COALESCE(c.first_name,'')) LIKE ?2 ESCAPE '\\'
              OR lower(COALESCE(c.last_name,''))  LIKE ?2 ESCAPE '\\'
              OR lower(COALESCE(c.source,''))     LIKE ?2 ESCAPE '\\')
          ORDER BY c.created_at DESC, c.id DESC LIMIT 100",
        &[&emp, &patron],
        Por::Lectora,
    )?);

    caminos.push(medir(
        &db,
        "Sin resultados · EL RECUENTO, con ESCAPE",
        "zzqx".to_owned(),
        "SELECT count(*) FROM contact c
          WHERE c.company_id = ?1 AND c.deleted_at IS NULL
            AND (lower(COALESCE(c.first_name,'')) LIKE ?2 ESCAPE '\\'
              OR lower(COALESCE(c.last_name,''))  LIKE ?2 ESCAPE '\\'
              OR lower(COALESCE(c.source,''))     LIKE ?2 ESCAPE '\\')",
        &[&emp, &patron],
    )?);

    // ── 10 a 13 · EL FILTRO QUE SE ENTREGÓ, NO LOS CANDIDATOS ─────────────
    //
    // Los nueve de arriba son SQL escrito aquí para decidir. Estos cuatro
    // llaman a `Db::listar_contactos` con su filtro, que es lo que la pantalla
    // llama de verdad — página **y** total, que es lo que cuesta.
    //
    // Hace falta medirlo aparte porque el camino real hace cosas que un SQL
    // suelto no: resuelve la dirección en su propia consulta, arma el patrón
    // con `ESCAPE`, y además lee los canales de la página. Dar por bueno el
    // candidato y no medir lo entregado es el patrón del §9 aplicado a una
    // medición: cada pieza correcta y la junta sin mirar.
    {
        use arles_db::FiltroDeContactos;
        let reales: [(&'static str, String, FiltroDeContactos<'_>); 7] = [
            (
                "ENTREGADO · sin filtro",
                String::new(),
                FiltroDeContactos::default(),
            ),
            (
                "ENTREGADO · texto que existe",
                nombre.clone(),
                FiltroDeContactos {
                    texto: Some(&nombre),
                    ..Default::default()
                },
            ),
            (
                "ENTREGADO · texto que NO existe",
                "zzqx".to_owned(),
                FiltroDeContactos {
                    texto: Some("zzqx"),
                    ..Default::default()
                },
            ),
            (
                "ENTREGADO · dirección exacta",
                correo.clone(),
                FiltroDeContactos {
                    direccion: Some(&correo),
                    ..Default::default()
                },
            ),
            // ── LOS DOS QUE FALTABAN, Y POR QUÉ ES UN HALLAZGO QUE FALTARAN ──
            //
            // La primera versión de esta tanda medía tres criterios de los
            // cuatro entregados. El canal se midió **como candidato** (0.1 ms)
            // y la baja **no se midió nunca**: es una subconsulta correlacionada
            // con una junta a `suppression_entry`, que es la clase de cosa que
            // en esta misma entrega resultó costar veinte segundos.
            //
            // Entregar un criterio sin medirlo es exactamente lo que este
            // tramo vino a corregir, así que no se da por bueno: se mide.
            (
                "ENTREGADO · sólo con WhatsApp",
                String::new(),
                FiltroDeContactos {
                    canal: Some(arles_core::canal::Canal::WhatsApp),
                    ..Default::default()
                },
            ),
            (
                "ENTREGADO · con alguna baja",
                String::new(),
                FiltroDeContactos {
                    con_alguna_baja: Some(true),
                    ..Default::default()
                },
            ),
            (
                "ENTREGADO · los cuatro a la vez",
                nombre.clone(),
                FiltroDeContactos {
                    texto: Some(&nombre),
                    direccion: Some(&correo),
                    canal: Some(arles_core::canal::Canal::Correo),
                    con_alguna_baja: Some(false),
                },
            ),
        ];
        for (nombre_del_camino, busca, filtro) in reales {
            let mut tiempos = Vec::new();
            let mut filas = 0u32;
            for _ in 0..3 {
                let t = Instant::now();
                let p = db.listar_contactos(empresa, &filtro, 0, 100)?;
                tiempos.push(t.elapsed());
                filas = p.total;
            }
            caminos.push(Camino {
                nombre: nombre_del_camino,
                busca,
                tiempos,
                filas,
                // `listar_contactos` corre tres consultas, así que un plan
                // suelto no la describe. Los planes de sus piezas son los de
                // los caminos de arriba.
                plan: vec!["(el camino real: página, total y canales)".to_owned()],
            });
        }
    }

    let contactos: u32 = db.ejecutar_en_pruebas(|c| {
        c.query_row(
            "SELECT count(*) FROM contact WHERE deleted_at IS NULL",
            [],
            |f| f.get(0),
        )
    })?;

    Ok(caminos
        .into_iter()
        .map(|camino| Medido { camino, contactos })
        .collect())
}

/// Corre una consulta tres veces y lee su plan.
///
/// **Tres y no una**, porque una sola corrida no se puede publicar: es la
/// lección de C-2, donde la cifra de la auditoría C —una corrida sin guardar—
/// no se pudo reproducir ni diagnosticar.
fn medir(
    db: &arles_db::Db,
    nombre: &'static str,
    busca: String,
    sql: &str,
    args: &[&String],
) -> Result<Camino, Box<dyn std::error::Error>> {
    medir_por(db, nombre, busca, sql, args, Por::Escritor)
}

/// Por qué conexión se mide. **No es un detalle de implementación**: el
/// 03/11/2026 resultó ser un factor 10, y lo que la pantalla usa es la lectora.
#[derive(Clone, Copy)]
enum Por {
    Escritor,
    Lectora,
}

fn medir_por(
    db: &arles_db::Db,
    nombre: &'static str,
    busca: String,
    sql: &str,
    args: &[&String],
    por: Por,
) -> Result<Camino, Box<dyn std::error::Error>> {
    let correr = |f: &dyn Fn(&rusqlite::Connection) -> Result<u32, rusqlite::Error>| match por {
        Por::Escritor => db.ejecutar_en_pruebas(f),
        Por::Lectora => db.leer_en_pruebas(f),
    };
    let mut tiempos = Vec::new();
    let mut filas = 0u32;
    for _ in 0..3 {
        let t = Instant::now();
        filas = correr(&|c| {
            let mut q = c.prepare(sql)?;
            let n = q
                .query_map(rusqlite::params_from_iter(args.iter()), |_| Ok(()))?
                .count();
            Ok(u32::try_from(n).unwrap_or(u32::MAX))
        })?;
        tiempos.push(t.elapsed());
    }

    let plan = db.ejecutar_en_pruebas(|c: &rusqlite::Connection| {
        let mut q = c.prepare(&format!("EXPLAIN QUERY PLAN {sql}"))?;
        q.query_map(rusqlite::params_from_iter(args.iter()), |f| {
            f.get::<_, String>(3)
        })?
        .collect::<Result<Vec<_>, _>>()
    })?;

    Ok(Camino {
        nombre,
        busca,
        tiempos,
        filas,
        plan,
    })
}
