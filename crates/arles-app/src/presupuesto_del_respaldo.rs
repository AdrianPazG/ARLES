//! **Cuánto tarda y cuánta memoria usa escribir y leer un respaldo entero.**
//!
//! Hallazgo F-4 de la auditoría de la Fase 8 (07/10/2026).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTA MEDICIÓN FALTABA, Y POR QUÉ ESTÁ AQUÍ Y NO EN EL BANCO
//!
//! El presupuesto de rendimiento tiene **trece cifras** y ninguna mide el
//! respaldo, la restauración ni el archivado — que es todo lo que construyó la
//! Fase 8, y lo único del producto que **recorre la base entera**. Es el mismo
//! hueco que la auditoría C encontró con la activación de campañas: una
//! operación que corre dentro de un comando de Tauri, con el cerrojo de la
//! conexión tomado y la interfaz sin contestar, que nadie había cronometrado.
//! Ahí resultó ser el peor número del producto.
//!
//! El banco de pruebas (8.5) es donde viven las otras medidas, y **aquí no
//! sirve**: el camino que el usuario pulsa lo orquestan `ElArchivo` y
//! `LeeElArchivo`, que viven en `arles-app` —la frontera— y el banco no puede
//! depender de ella sin arrastrar Tauri dentro de un binario de medición. La
//! alternativa era repetir la orquestación en el banco con las piezas públicas
//! del núcleo, y entonces se mediría **otra cosa parecida**: exactamente el
//! defecto que esta fase lleva ocho entregas cazando. Así que la medida vive
//! donde vive el código que mide.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! SE CORRE A MANO
//!
//! ```text
//! cargo test --release -p arles-app presupuesto -- --ignored --nocapture
//! ```
//!
//! Y vive dentro del crate, no en `tests/`, porque `ElArchivo::nuevo` y
//! `cerrar` son `pub(crate)`: una prueba de integración no las ve, y abrirlas
//! sólo para poder medirlas ampliaría la frontera —cada cosa pública de más es
//! superficie— por una cifra. El módulo entero sólo se compila en pruebas.
//!
//! Va marcada `#[ignore]` porque siembra decenas de miles de filas y tarda
//! minutos: en cada `cargo test` sería un peaje diario por una cifra que sólo
//! cambia cuando alguien toca el exportador. Lo que **sí** corre siempre es la
//! prueba de que el sembrado produce lo que dice producir, abajo.

use std::time::Instant;

/// Cuántos contactos siembra la medición.
///
/// Treinta mil y no medio millón a propósito: lo que hace falta es la **forma**
/// de la curva —si el tiempo crece con las filas o se dispara— y el techo
/// declarado del producto se extrapola de dos puntos. Medio millón dentro de
/// una prueba de `cargo test` tardaría lo que tarda el banco entero, y la cifra
/// del techo se toma en la máquina del usuario cuando se cierre R-07.
const CONTACTOS: usize = 30_000;

/// Memoria residente de este proceso, en bytes. `None` fuera de Linux.
fn residente() -> Option<u64> {
    let texto = std::fs::read_to_string("/proc/self/status").ok()?;
    for linea in texto.lines() {
        if let Some(resto) = linea.strip_prefix("VmRSS:") {
            let kb: u64 = resto.trim().trim_end_matches(" kB").trim().parse().ok()?;
            return Some(kb * 1024);
        }
    }
    None
}

/// Siembra una empresa, un lote y `cuantos` contactos con su canal de correo.
///
/// Va en SQL directo y en una sola transacción. El camino del repositorio ya
/// está medido en el banco (importar 500 000: 91.4 s) y meterlo aquí mediría la
/// importación otra vez en vez del respaldo, que es lo que esta prueba existe
/// para cronometrar.
fn sembrar(db: &arles_db::Db, cuantos: usize) {
    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(
            "INSERT INTO company (id, commercial_name, country, timezone, corporate_email, created_at, updated_at)
               VALUES ('E1', 'Telemetry', 'MX', 'America/Mexico_City', 'hola@telemetry.mx', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
             INSERT INTO import_batch
               (id, company_id, original_filename, stored_filename, file_hash, column_mapping,
                total_rows, consent_affirmation, created_at)
               VALUES ('L1', 'E1', 'lista.csv', 'L1.csv', 'abc', '{}', 1, 'sí', '2026-01-02T00:00:00Z');",
        )
    })
    .expect("siembra la empresa");

    db.ejecutar_en_pruebas(|c| {
        let tx = c.unchecked_transaction()?;
        {
            let mut mete_contacto = tx.prepare(
                "INSERT INTO contact (id, company_id, import_batch_id, first_name, last_name, source, created_at, updated_at)
                   VALUES (?1, 'E1', 'L1', ?2, ?3, ?4, '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z')",
            )?;
            let mut mete_canal = tx.prepare(
                "INSERT INTO contact_channel (id, company_id, contact_id, channel, value_raw, value_normalized, created_at, updated_at)
                   VALUES (?1, 'E1', ?2, 'email', ?3, ?3, '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z')",
            )?;
            for i in 0..cuantos {
                let id = format!("C{i:07}");
                // Con acentos y una coma: es lo que obliga a escapar de verdad
                // al redactar la fila, y escapar es parte de lo que se mide.
                mete_contacto.execute(rusqlite::params![
                    &id,
                    format!("Ana María {i}"),
                    "Álvarez Bañuelos",
                    "Café de Olla, S.A. de C.V."
                ])?;
                mete_canal.execute(rusqlite::params![
                    format!("K{i:07}"),
                    &id,
                    format!("ana.alvarez{i}@ejemplo.mx")
                ])?;
            }
        }
        tx.commit()
    })
    .expect("siembra los contactos");
}

/// **El sembrado produce lo que dice producir.**
///
/// Corre siempre, con cien filas. Sin esto, la medición de abajo podría estar
/// cronometrando una base casi vacía y devolver una cifra excelente y falsa —
/// que es el caso de la prueba de contactos con un solo contacto que el §2
/// documenta desde la Fase 3.
#[test]
fn el_sembrado_mete_las_filas_que_dice() {
    let dir = tempfile::tempdir().expect("temporal");
    let clave = arles_db::ClaveMaestra::generar().expect("genera");
    let db = arles_db::Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");

    sembrar(&db, 100);

    let (contactos, canales): (u64, u64) = db
        .ejecutar_en_pruebas(|c| {
            Ok((
                c.query_row("SELECT COUNT(*) FROM contact", [], |f| f.get(0))?,
                c.query_row("SELECT COUNT(*) FROM contact_channel", [], |f| f.get(0))?,
            ))
        })
        .expect("cuenta");

    assert_eq!(contactos, 100, "el sembrado no metió los contactos");
    assert_eq!(canales, 100, "el sembrado no metió los canales");
}

/// **Exportar y volver a restaurar, cronometrado y con la memoria medida.**
///
/// Las cuatro cifras que devuelve son las que el presupuesto no tenía:
///
///   1. **Exportar sin cifrar** — el recorrido de la base y la escritura.
///   2. **Exportar cifrado** — lo que cuesta Argon2id más XChaCha20 por trozos.
///      D-8A lo hace el valor por defecto, así que es el camino normal.
///   3. **Revisar** (el preflight) — tiene que ser inmediato: sólo lee la
///      cabecera, y si crece con las filas es que alguien le metió el cuerpo.
///   4. **Restaurar** sobre **otra base** — el viaje completo.
///
/// Y la memoria residente antes y después de cada tramo, porque el cursor de la
/// 8.2b y el sobre a trozos de la 8.2c prometen **memoria acotada**: eso es una
/// garantía de diseño que hasta hoy no tenía ni una cifra al lado.
#[test]
#[ignore = "siembra decenas de miles de filas: se corre a mano con --ignored"]
fn cuanto_cuesta_un_respaldo_entero() {
    use arles_db::{ClaveMaestra, Db};

    const CONTRASENA: &str = "una contraseña larga de verdad";
    let dir = tempfile::tempdir().expect("temporal");
    let base = dir.path().join("arles.db");

    let clave = ClaveMaestra::generar().expect("genera");
    let db = Db::abrir(&base, &clave).expect("abre");

    let sembrando = Instant::now();
    sembrar(&db, CONTACTOS);
    println!(
        "sembrar {CONTACTOS} contactos: {:.1} s",
        sembrando.elapsed().as_secs_f64()
    );
    let tras_sembrar = residente();

    // ── 1 · exportar sin cifrar ──
    let claro = dir.path().join("claro.arles");
    let reloj = Instant::now();
    escribir(&db, &claro, None);
    let exportar_claro = reloj.elapsed();

    // ── 2 · exportar cifrado, que es el camino por defecto (D-8A) ──
    let cifrado = dir.path().join("cifrado.arles");
    let reloj = Instant::now();
    escribir(&db, &cifrado, Some(CONTRASENA));
    let exportar_cifrado = reloj.elapsed();
    let tras_exportar = residente();

    // ── 3 · el preflight, que sólo lee la cabecera ──
    let reloj = Instant::now();
    let (cabecera, salta) = crate::restauracion::leer_la_cabecera(&cifrado).expect("cabecera");
    let revisar = reloj.elapsed();

    // ── 4 · restaurar sobre OTRA base ──
    let otra = dir.path().join("otra.db");
    let clave_nueva = ClaveMaestra::generar().expect("genera");
    let destino = Db::abrir(&otra, &clave_nueva).expect("abre la otra");
    let reloj = Instant::now();
    let mut origen =
        crate::restauracion::LeeElArchivo::nuevo(&cifrado, &cabecera, salta, Some(CONTRASENA))
            .expect("lector");
    destino
        .restaurar(
            &mut origen,
            &cabecera.esquema,
            &cabecera.columnas,
            cabecera.alcance.clone(),
            "el banco",
            "2026-10-07T12:00:00Z",
        )
        .expect("restaura");
    let restaurar = reloj.elapsed();
    let tras_restaurar = residente();

    // Que de verdad llegaron: una restauración que no trae nada sería la más
    // rápida de todas, y sin esta comprobación la cifra sería un elogio a un
    // fallo.
    let cuantos: u64 = destino
        .ejecutar_en_pruebas(|c| c.query_row("SELECT COUNT(*) FROM contact", [], |f| f.get(0)))
        .expect("cuenta");
    assert_eq!(
        cuantos as usize, CONTACTOS,
        "la restauración no trajo los contactos: la medida no mide nada"
    );

    let mb = |b: Option<u64>| {
        b.map_or_else(
            || "sin /proc".to_owned(),
            |v| format!("{:.0} MB", v as f64 / 1_048_576.0),
        )
    };
    let tamano = |p: &std::path::Path| {
        std::fs::metadata(p).map(|m| m.len()).unwrap_or(0) as f64 / 1_048_576.0
    };

    println!(
        "\n── RESPALDO DE {CONTACTOS} CONTACTOS ──\n\
         exportar sin cifrar : {:>8.0} ms   ({:.1} MB en disco)\n\
         exportar cifrado    : {:>8.0} ms   ({:.1} MB en disco)\n\
         revisar (cabecera)  : {:>8.1} ms\n\
         restaurar en otra   : {:>8.0} ms\n\
         memoria: tras sembrar {} · tras exportar {} · tras restaurar {}",
        exportar_claro.as_secs_f64() * 1000.0,
        tamano(&claro),
        exportar_cifrado.as_secs_f64() * 1000.0,
        tamano(&cifrado),
        revisar.as_secs_f64() * 1000.0,
        restaurar.as_secs_f64() * 1000.0,
        mb(tras_sembrar),
        mb(tras_exportar),
        mb(tras_restaurar),
    );

    // El preflight **no puede** crecer con las filas: lee la cabecera y para.
    // Si algún día alguien le mete el cuerpo, la pantalla de restaurar se queda
    // pensando delante de quien eligió el archivo, y nada fallaría.
    assert!(
        revisar.as_millis() < 500,
        "el preflight tardó {} ms: ya no está leyendo sólo la cabecera",
        revisar.as_millis()
    );
}

/// Escribe un respaldo completo con el orquestador de verdad.
fn escribir(db: &arles_db::Db, ruta: &std::path::Path, contrasena: Option<&str>) {
    let mut destino = crate::respaldo::ElArchivo::nuevo(
        ruta,
        "completo",
        contrasena,
        Box::new(|_| {}),
        "2026-10-07T12:00:00Z".to_owned(),
    )
    .expect("abre el archivo");
    db.exportar(arles_db::QueSeLleva::Todo, &mut destino)
        .expect("exporta");
    destino.cerrar().expect("cierra");
}
