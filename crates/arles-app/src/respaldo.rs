//! Escribir el respaldo: el archivo, el diálogo y la bitácora.
//!
//! Entrega 8.2c. Es el tramo que **cierra el hueco** que la 8.2b dejó abierto a
//! propósito: `Db::exportar` estaba construido, con trece pruebas, y nadie lo
//! llamaba. Lo vigila `el_exportador_tiene_quien_lo_llame` en `--fase 8`,
//! escrita antes de esto y vista en rojo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA FORMA DEL ARCHIVO: CABECERA EN CLARO, CUERPO CIFRADO
//!
//! ```text
//! ARLES-RESPALDO          ┐
//! formato→1               │
//! …                       │ en claro, siempre
//! sal→a3f1…               │
//! nonce→9c20…             │
//! columnas→contact=id,…   │
//!                         ┘ ← línea en blanco
//! «««  cuerpo + suma  »»»   ← cifrado si hay contraseña; texto si no
//! ```
//!
//! **La cabecera va en claro y no es un descuido.** Es lo que permite que la
//! restauración (8.3) diga «este respaldo va con contraseña, y es de hace cuatro
//! meses» **antes** de pedirla. Cifrarla también obligaría a pedir la
//! contraseña para averiguar si hace falta una contraseña.
//!
//! Y no filtra nada: dice de qué versión es el esquema, cuántas filas trae de
//! cada tabla y qué columnas tienen. Ni un dato de una persona, ni una ruta, ni
//! el nombre del equipo.
//!
//! **La suma de verificación va dentro del sobre**, al final del cuerpo. Cubre
//! la cabecera y el cuerpo **en claro**, que es lo que hay que poder comprobar
//! al restaurar: que lo que entra es lo que salió. La integridad de los bytes
//! del disco ya la garantiza el cifrado autenticado, que es más fuerte que un
//! sha256 — pero sólo existe si hay contraseña, y la suma tiene que existir
//! siempre.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA CONTRASEÑA ENTRA Y NO SALE (§29, ADR-0011, D-8A)
//!
//! Llega por la frontera, se usa para derivar la clave y se borra. **No va al
//! llavero, no va a la base, no se registra en la bitácora y no vuelve.** Si se
//! olvida, el respaldo no se abre: no hay puerta de atrás, y construirla sería
//! construir justo lo que el cifrado existe para impedir.
//!
//! Lo que sí queda en `audit_log` es que se hizo un respaldo, de qué alcance, si
//! iba cifrado y cuántas filas llevaba — **nunca la ruta**, que es la regla 4 de
//! la frontera aplicada al disco: un respaldo no debe decir de qué máquina salió.

use std::io::Write as _;

use arles_core::formato_de_respaldo::{self as formato, Cabecera, SumaEnCurso};
use arles_core::respaldo::MESES_EN_EL_ESENCIAL;
use arles_db::respaldo::{Destino, Inventario, QueSeLleva};

use crate::comandos::empresa_actual;
use crate::error::ErrorIpc;
use crate::estado::EstadoApp;

/// El nombre del evento con el que se avisa del avance.
///
/// Un evento y no un valor devuelto: con medio millón de filas, exportar tarda
/// y una barra parada veinte segundos es indistinguible de un programa colgado.
/// El §94 pide cifras, no una rueda.
pub const EVENTO_DE_AVANCE: &str = "respaldo:avance";

/// Cuántas filas van de cuántas.
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Avance {
    pub filas: u64,
    pub total: u64,
}

/// Qué llevaría un respaldo, **sin escribir nada**.
///
/// Es el patrón que la 7.4 dejó con el recorte del registro: primero se dice
/// con cifras qué va a pasar, y después se pulsa.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LoQueLlevaria {
    /// Hasta qué migración va esta base: `V11`.
    pub esquema: String,
    /// Una entrada por tabla, en el orden en que se escribirán.
    pub tablas: Vec<FilasDeTabla>,
    pub total: u64,
    /// Cuántos meses de historial se guarda el esencial. Va aquí para que la
    /// pantalla no tenga que repetir la cifra del núcleo (§94).
    pub meses_en_el_esencial: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FilasDeTabla {
    pub tabla: String,
    pub filas: u64,
}

/// Lo que se devuelve después de escribir.
///
/// **Sólo el nombre del archivo, nunca su ruta** (regla 4 de la frontera). Es
/// el mismo patrón que `entregar_informe_de_acceso` de la 3.4.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RespaldoEscrito {
    pub nombre_del_archivo: String,
    pub filas: u64,
    pub bytes: u64,
    pub cifrado: bool,
    pub alcance: String,
}

/// Los dos alcances, como los nombra la webview.
fn que_se_lleva<'c>(alcance: &str, corte: &'c str) -> Result<QueSeLleva<'c>, crate::AppError> {
    match alcance {
        "completo" => Ok(QueSeLleva::Todo),
        "esencial" => Ok(QueSeLleva::LoEsencialDesde(corte)),
        // **No se cae en «esencial»**: escribir un respaldo más pequeño que el
        // que alguien pidió, por un alcance mal escrito, es perder datos sin
        // decirlo.
        _ => Err(crate::AppError::AlcanceDeRespaldoDesconocido),
    }
}

/// Hace cuánto empieza el historial que se lleva el esencial.
///
/// Doce meses atrás desde hoy, en UTC. Se calcula **aquí y no en el núcleo**,
/// que no tiene reloj (cero I/O).
fn corte_del_esencial() -> String {
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;

    let meses = i64::from(MESES_EN_EL_ESENCIAL);
    OffsetDateTime::now_utc()
        .saturating_sub(time::Duration::days(meses * 365 / 12))
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}

pub(crate) fn ahora_utc() -> String {
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;

    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}

/// Qué llevaría el respaldo que se pida, con cifras y sin tocar nada.
///
/// # Errores
///
/// - `error.app.empresa_no_configurada` si todavía no hay empresa.
/// - `error.alcance_no_coincide` si el alcance no es uno de los dos.
/// - Lo que falle al leer la base.
#[tauri::command]
pub fn que_llevaria_el_respaldo(
    estado: tauri::State<'_, EstadoApp>,
    alcance: String,
) -> Result<LoQueLlevaria, ErrorIpc> {
    let _ = empresa_actual(&estado)?;
    let corte = corte_del_esencial();
    let que = que_se_lleva(&alcance, &corte)?;

    // Un destino que no escribe: sólo se queda con el inventario y corta.
    //
    // Y corta **de verdad**, con un error: sin eso, pedir el preflight de una
    // base de medio millón de filas las leería todas para no usarlas.
    #[derive(Default)]
    struct SoloElInventario(Option<Inventario>);
    impl Destino for SoloElInventario {
        fn inventario(&mut self, inv: &Inventario) -> Result<(), arles_db::DbError> {
            self.0 = Some(inv.clone());
            Err(arles_db::DbError::DatoInvalido {
                motivo: "sólo se pedía el inventario",
            })
        }
        fn fila(&mut self, _: &str, _: &[Option<String>]) -> Result<(), arles_db::DbError> {
            Ok(())
        }
    }

    let mut solo = SoloElInventario::default();
    // El error es el esperado: lo que importa es que el inventario llegó.
    let _ = estado.db().exportar(que, &mut solo);
    let inventario = solo.0.ok_or(crate::AppError::Db(
        arles_db::DbError::EsquemaNoSeDejaRespaldar {
            motivo: "no se pudo leer qué tablas lleva un respaldo de esta base",
        },
    ))?;

    Ok(LoQueLlevaria {
        esquema: inventario.esquema.clone(),
        total: inventario.total_de_filas(),
        tablas: inventario
            .tablas
            .iter()
            .map(|t| FilasDeTabla {
                tabla: t.tabla.clone(),
                filas: t.filas,
            })
            .collect(),
        meses_en_el_esencial: MESES_EN_EL_ESENCIAL,
    })
}

/// Escribe el respaldo donde diga el usuario.
///
/// Devuelve `None` si cerró el diálogo sin elegir sitio: **no es un error**, es
/// una decisión, y tratarla como fallo enseñaría un aviso rojo por cancelar.
///
/// `contrasena` vacía o ausente significa **sin cifrar**, que D-8A permite sólo
/// a propósito y con el aviso delante. Lo que no hace este comando es decidirlo
/// por nadie.
///
/// # Errores
///
/// - `error.app.empresa_no_configurada` si todavía no hay empresa.
/// - `error.alcance_no_coincide` si el alcance no es uno de los dos.
/// - `error.app.archivo_no_se_pudo_leer` si no se puede escribir donde se eligió.
/// - Lo que falle al leer la base o al cifrar.
#[tauri::command]
pub fn exportar_respaldo(
    estado: tauri::State<'_, EstadoApp>,
    app: tauri::AppHandle,
    alcance: String,
    contrasena: Option<String>,
) -> Result<Option<RespaldoEscrito>, ErrorIpc> {
    use tauri_plugin_dialog::DialogExt;

    let empresa = empresa_actual(&estado)?;
    let corte = corte_del_esencial();
    let que = que_se_lleva(&alcance, &corte)?;

    // Una contraseña vacía es «sin cifrar», no una contraseña de cero
    // caracteres: derivar una clave de la cadena vacía daría un respaldo que
    // cualquiera abre y que **parece** protegido, que es peor que uno que dice
    // no estarlo.
    let contrasena = contrasena.filter(|c| !c.is_empty());

    // El nombre propuesto lleva la fecha y el alcance: quien guarda uno cada
    // semana no acaba con cinco «respaldo.arles» en la misma carpeta y sin
    // forma de saber cuál es cuál.
    let sugerido = format!(
        "arles-{}-{}.arles",
        &ahora_utc().get(..10).unwrap_or("fecha").replace('-', ""),
        alcance
    );

    let Some(elegido) = app
        .dialog()
        .file()
        .set_file_name(&sugerido)
        .add_filter("Respaldo de ARLES", &["arles"])
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let ruta = elegido
        .into_path()
        .map_err(|_| ErrorIpc::from(crate::AppError::ArchivoNoSePudoLeer))?;

    // Aquí, y sólo aquí, el avance se convierte en un evento de Tauri. Si el
    // aviso no llega, el respaldo sigue: una barra que no se mueve es molesta;
    // abortar por no poder pintarla sería absurdo.
    let emisor = app.clone();
    let avisar = Box::new(move |a: Avance| {
        use tauri::Emitter as _;
        let _ = emisor.emit(EVENTO_DE_AVANCE, a);
    });

    let mut destino =
        ElArchivo::nuevo(&ruta, &alcance, contrasena.as_deref(), avisar, ahora_utc())?;
    let filas = estado
        .db()
        .exportar(que, &mut destino)
        .map_err(crate::AppError::Db)?;
    let bytes = destino.cerrar()?;

    // Se registra **después** de escribir. Anotar un respaldo que no llegó a
    // existir convertiría la bitácora en una lista de intenciones.
    estado
        .db()
        .registrar_respaldo(&empresa.to_string(), &alcance, contrasena.is_some(), filas)
        .map_err(crate::AppError::Db)?;

    Ok(Some(RespaldoEscrito {
        nombre_del_archivo: ruta
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_owned(),
        filas,
        bytes,
        cifrado: contrasena.is_some(),
        alcance,
    }))
}

/// El archivo mientras se escribe.
///
/// ── POR QUÉ EL CUERPO SE ACUMULA EN UN BÚFER Y NO SE ESCRIBE FILA A FILA ──
///
/// Porque cifrar sella trozos de 64 KiB: hasta que no hay uno lleno no hay nada
/// que escribir. El búfer **nunca pasa de dos trozos**, así que la memoria sigue
/// acotada — que es todo el trabajo de la 8.2b, y perderlo aquí habría dejado el
/// agujero justo en la junta de las dos piezas.
///
/// Sin cifrar se podría escribir directo, pero el camino es el mismo a propósito:
/// dos caminos distintos para el mismo archivo son dos sitios donde equivocarse.
pub(crate) struct ElArchivo {
    fichero: std::io::BufWriter<std::fs::File>,
    suma: SumaEnCurso,
    sobre: Option<arles_core::cifrado_de_respaldo::SobreEnCurso>,
    alcance: String,
    creado_en: String,
    contrasena: Option<String>,
    /// A quién se le avisa del avance.
    ///
    /// ── POR QUÉ UN CIERRE Y NO EL `AppHandle` DIRECTAMENTE ──
    ///
    /// Porque con el `AppHandle` dentro, **este escritor no se puede probar**:
    /// un `AppHandle` sólo existe dentro de una aplicación de Tauri corriendo.
    /// Y entonces la única prueba posible sería de las piezas por separado —el
    /// cursor de la 8.2b, el formato de la 8.1, el sobre de la 8.2a—, que es
    /// exactamente la forma del agujero que este proyecto ya ha visto trece
    /// veces: cada capa pasa lo suyo y nadie comprueba la junta.
    ///
    /// Con el cierre, `un_respaldo_se_escribe_y_se_vuelve_a_leer` recorre el
    /// camino entero: base poblada → archivo en disco → descifrar → comparar.
    avisar: Box<dyn FnMut(Avance)>,
    bytes: u64,
}

impl ElArchivo {
    pub(crate) fn nuevo(
        ruta: &std::path::Path,
        alcance: &str,
        contrasena: Option<&str>,
        avisar: Box<dyn FnMut(Avance)>,
        creado_en: String,
    ) -> Result<Self, ErrorIpc> {
        let fichero = std::fs::File::create(ruta)
            .map_err(|_| ErrorIpc::from(crate::AppError::ArchivoNoSePudoLeer))?;
        Ok(Self {
            fichero: std::io::BufWriter::new(fichero),
            suma: SumaEnCurso::new(),
            sobre: None,
            alcance: alcance.to_owned(),
            creado_en,
            contrasena: contrasena.map(str::to_owned),
            avisar,
            bytes: 0,
        })
    }

    /// Escribe bytes al archivo y lleva la cuenta.
    fn volcar(&mut self, bytes: &[u8]) -> Result<(), arles_db::DbError> {
        self.fichero
            .write_all(bytes)
            .map_err(|_| arles_db::DbError::DatoInvalido {
                motivo: "no se pudo escribir en el archivo del respaldo",
            })?;
        self.bytes += bytes.len() as u64;
        Ok(())
    }

    /// Mete texto del cuerpo: alimenta la suma y lo cifra si toca.
    fn cuerpo(&mut self, texto: &str) -> Result<(), arles_db::DbError> {
        self.suma.añadir(texto);
        self.sellar(texto.as_bytes())
    }

    /// Lo mismo, sin tocar la suma. Para la línea de cierre, que **es** la suma.
    fn sellar(&mut self, bytes: &[u8]) -> Result<(), arles_db::DbError> {
        match self.sobre.as_mut() {
            None => self.volcar(bytes),
            Some(sobre) => {
                let mut cifrado = Vec::new();
                sobre
                    .añadir(bytes, &mut cifrado)
                    .map_err(arles_db::DbError::Nucleo)?;
                self.volcar(&cifrado)
            }
        }
    }

    /// Cierra el cuerpo con su suma y el sobre con su último trozo.
    pub(crate) fn cerrar(mut self) -> Result<u64, ErrorIpc> {
        let cierre = formato::redactar_cierre(&self.suma.clone().terminar());
        self.sellar(cierre.as_bytes())
            .map_err(crate::AppError::Db)?;

        if let Some(sobre) = self.sobre.take() {
            let mut ultimo = Vec::new();
            sobre.terminar(&mut ultimo).map_err(crate::AppError::Core)?;
            self.volcar(&ultimo).map_err(crate::AppError::Db)?;
        }

        self.fichero
            .flush()
            .map_err(|_| ErrorIpc::from(crate::AppError::ArchivoNoSePudoLeer))?;
        Ok(self.bytes)
    }
}

impl Destino for ElArchivo {
    /// Escribe la cabecera **en claro** y, si hay contraseña, abre el sobre.
    ///
    /// La sal y el nonce se generan aquí, con la aleatoriedad del sistema, y
    /// **uno nuevo en cada respaldo**: repetir el par con la misma contraseña es
    /// lo único que rompe este cifrado.
    fn inventario(&mut self, inventario: &Inventario) -> Result<(), arles_db::DbError> {
        use arles_core::cifrado_de_respaldo::{BYTES_DE_NONCE, BYTES_DE_SAL, SobreEnCurso, a_hex};

        let (filas, columnas) = inventario.para_la_cabecera();

        let cifrado = match self.contrasena.as_deref() {
            None => formato::Cifrado::No,
            Some(_) => {
                let mut sal = [0_u8; BYTES_DE_SAL];
                let mut nonce = [0_u8; BYTES_DE_NONCE];
                for destino in [sal.as_mut_slice(), nonce.as_mut_slice()] {
                    getrandom::fill(destino).map_err(|_| {
                        // Sin aleatoriedad real no hay cifrado real: no se
                        // sigue con un valor fijo. Es la misma decisión que
                        // `ClaveMaestra::generar` (ADR-0011).
                        arles_db::DbError::DatoInvalido {
                            motivo: "el sistema no pudo dar aleatoriedad para cifrar el respaldo",
                        }
                    })?;
                }
                formato::Cifrado::Argon2idXChaCha20 {
                    sal: a_hex(&sal),
                    nonce: a_hex(&nonce),
                }
            }
        };

        let cabecera = Cabecera {
            formato: formato::VERSION_DEL_FORMATO,
            esquema: inventario.esquema.clone(),
            alcance: formato::Alcance::parse(&self.alcance).map_err(arles_db::DbError::Nucleo)?,
            creado_en: self.creado_en.clone(),
            cifrado: cifrado.clone(),
            filas,
            columnas,
        };

        // La cabecera va al archivo **sin pasar por el sobre** —se lee sin
        // contraseña— pero **sí entra en la suma**: lo que se comprueba al
        // restaurar es que la cabecera y el cuerpo son los que se escribieron
        // juntos, no sólo el cuerpo.
        let texto = cabecera.redactar();
        self.suma.añadir(&texto);
        self.volcar(texto.as_bytes())?;

        if let formato::Cifrado::Argon2idXChaCha20 { sal, nonce } = &cifrado {
            let contrasena = self
                .contrasena
                .clone()
                .ok_or(arles_db::DbError::DatoInvalido {
                    motivo: "se iba a cifrar sin contraseña",
                })?;
            let sal = de_hex_fijo::<{ BYTES_DE_SAL }>(sal)?;
            let nonce = de_hex_fijo::<{ BYTES_DE_NONCE }>(nonce)?;
            self.sobre = Some(
                SobreEnCurso::nuevo(&contrasena, &sal, &nonce)
                    .map_err(arles_db::DbError::Nucleo)?,
            );
        }
        Ok(())
    }

    fn fila(&mut self, tabla: &str, valores: &[Option<String>]) -> Result<(), arles_db::DbError> {
        let prestados: Vec<Option<&str>> = valores.iter().map(|v| v.as_deref()).collect();
        let linea = formato::redactar_fila(tabla, &prestados);
        self.cuerpo(&linea)
    }

    fn avance(&mut self, filas: u64, total: u64) {
        (self.avisar)(Avance { filas, total });
    }
}

/// Hexadecimal a un arreglo de tamaño fijo.
pub(crate) fn de_hex_fijo<const N: usize>(texto: &str) -> Result<[u8; N], arles_db::DbError> {
    let bytes =
        arles_core::cifrado_de_respaldo::de_hex(texto, N).map_err(arles_db::DbError::Nucleo)?;
    bytes
        .try_into()
        .map_err(|_| arles_db::DbError::DatoInvalido {
            motivo: "la sal o el nonce del respaldo no miden lo que deben",
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use arles_db::{ClaveMaestra, Db, QueSeLleva};

    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(
                "INSERT INTO company (id, commercial_name, country, timezone, corporate_email, created_at, updated_at)
                   VALUES ('E1', 'Telemetry', 'MX', 'America/Mexico_City', 'hola@telemetry.mx', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
                 INSERT INTO import_batch
                   (id, company_id, original_filename, stored_filename, file_hash, column_mapping,
                    total_rows, consent_affirmation, created_at)
                   VALUES ('L1', 'E1', 'lista.csv', 'L1.csv', 'abc', '{}', 2, 'sí', '2026-01-02T00:00:00Z');
                 INSERT INTO contact (id, company_id, import_batch_id, first_name, last_name, created_at, updated_at)
                   VALUES ('C1', 'E1', 'L1', 'Ana', 'Ruiz', '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z');
                 INSERT INTO contact (id, company_id, import_batch_id, first_name, last_name, created_at, updated_at)
                   VALUES ('C2', 'E1', 'L1', 'Beto con\tun tabulador', NULL, '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z');",
            )
        })
        .expect("siembra");
        (dir, db)
    }

    /// Escribe un respaldo con el escritor de verdad y devuelve los bytes.
    fn escribir(db: &Db, contrasena: Option<&str>) -> (Vec<u8>, u64, Vec<Avance>) {
        let dir = tempfile::tempdir().expect("temporal");
        let ruta = dir.path().join("prueba.arles");

        // El avance se recoge en memoria: es lo que en la aplicación se
        // convierte en un evento de Tauri.
        let avisos = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let recoge = std::sync::Arc::clone(&avisos);

        let mut destino = ElArchivo::nuevo(
            &ruta,
            "completo",
            contrasena,
            Box::new(move |a| recoge.lock().expect("mutex").push(a)),
            "2026-10-05T12:00:00Z".to_owned(),
        )
        .expect("abre el archivo");

        let filas = db
            .exportar(QueSeLleva::Todo, &mut destino)
            .expect("exporta");
        destino.cerrar().expect("cierra");

        let bytes = std::fs::read(&ruta).expect("lee el archivo");
        let avisos = avisos.lock().expect("mutex").clone();
        (bytes, filas, avisos)
    }

    /// **El camino entero: base poblada → archivo en disco → volver a leerlo.**
    ///
    /// Es la prueba que esta entrega existe para tener. Las piezas ya estaban
    /// probadas por separado —el cursor de la 8.2b, el formato de la 8.1, el
    /// sobre de la 8.2a— y eso es precisamente la forma del agujero que este
    /// proyecto ha visto trece veces: cada capa pasa lo suyo y nadie comprueba
    /// la junta.
    ///
    /// Comprueba las dos mitades de D-8A en la misma pasada: sin contraseña el
    /// archivo es texto y se lee entero; con contraseña **la cabecera sigue en
    /// claro** —es lo que permite saber que hace falta una contraseña sin
    /// tenerla— y el cuerpo sólo sale descifrándolo.
    ///
    /// Verificada el 05/10/2026 quitando la cabecera de la suma: el respaldo se
    /// escribe, se descifra, y la suma no cuadra — la prueba falla donde debe.
    #[test]
    fn un_respaldo_se_escribe_y_se_vuelve_a_leer() {
        use arles_core::formato_de_respaldo::{Cabecera, Cifrado, SumaEnCurso, leer_cierre};

        let (_d, db) = base();

        for contrasena in [None, Some("una contraseña larga de verdad")] {
            let (bytes, filas, _) = escribir(&db, contrasena);

            // ── La cabecera se lee SIEMPRE, con contraseña o sin ella ──
            //
            // Y hace falta que sea así: la restauración (8.3) tiene que poder
            // decir «este respaldo va con contraseña» antes de pedirla. Se lee
            // sobre los bytes crudos, que con cuerpo cifrado no son UTF-8
            // entero, así que se lee sólo hasta la línea en blanco.
            let hasta = bytes
                .windows(2)
                .position(|v| v == b"\n\n")
                .expect("la cabecera termina en una línea en blanco")
                + 2;
            let texto_cabecera = std::str::from_utf8(bytes.get(..hasta).expect("la cabecera"))
                .expect("la cabecera es texto");
            let (cabecera, _) = Cabecera::leer(texto_cabecera).expect("se lee la cabecera");

            // Se lee del propio motor de migraciones y no se escribe a mano:
            // un literal aquí convierte cada migración futura en una prueba
            // rota, y lo que esta comprueba es que la cabecera diga el esquema
            // de ESTA base, no que ese esquema sea uno concreto.
            assert_eq!(
                cabecera.esquema,
                format!("V{}", arles_db::ultima_version().expect("hay migraciones"))
            );
            assert_eq!(cabecera.total_de_filas(), filas, "{contrasena:?}");
            assert_eq!(
                cabecera.cifrado.lleva_contrasena(),
                contrasena.is_some(),
                "la cabecera no dice la verdad sobre si va cifrado"
            );
            assert!(
                cabecera
                    .columnas_de("contact")
                    .is_some_and(|c| c.contains(&"last_name".to_owned())),
                "la cabecera tiene que decir qué columna es cada campo"
            );

            // ── El cuerpo ──
            let cuerpo_crudo = bytes.get(hasta..).expect("hay cuerpo").to_vec();
            let cuerpo = match &cabecera.cifrado {
                Cifrado::No => cuerpo_crudo,
                Cifrado::Argon2idXChaCha20 { sal, nonce } => {
                    let sal = de_hex_fijo::<16>(sal).expect("sal");
                    let nonce = de_hex_fijo::<19>(nonce).expect("nonce");
                    // Y con otra contraseña **no** se abre.
                    assert!(
                        arles_core::cifrado_de_respaldo::descifrar(
                            "otra cualquiera",
                            &sal,
                            &nonce,
                            &cuerpo_crudo
                        )
                        .is_err(),
                        "cualquier contraseña abría el respaldo"
                    );
                    arles_core::cifrado_de_respaldo::descifrar(
                        contrasena.expect("hay contraseña"),
                        &sal,
                        &nonce,
                        &cuerpo_crudo,
                    )
                    .expect("se descifra con su contraseña")
                }
            };
            let cuerpo = String::from_utf8(cuerpo).expect("el cuerpo es texto");

            // Tantas filas como declaró la cabecera.
            let escritas = cuerpo
                .lines()
                .take_while(|l| !l.is_empty())
                .filter(|l| !l.is_empty())
                .count() as u64;
            assert_eq!(
                escritas, filas,
                "{contrasena:?}: el cuerpo no trae las filas declaradas"
            );

            // ── La suma cubre la cabecera Y el cuerpo ──
            let declarada = leer_cierre(&cuerpo).expect("hay cierre");
            let cierre = arles_core::formato_de_respaldo::redactar_cierre(&declarada);
            let sin_cierre = cuerpo
                .strip_suffix(&cierre)
                .expect("el cuerpo termina con su línea de cierre");
            let mut suma = SumaEnCurso::new();
            suma.añadir(texto_cabecera);
            suma.añadir(sin_cierre);
            arles_core::formato_de_respaldo::comprobar_suma(&suma.terminar(), &declarada)
                .expect("la suma cuadra");

            // ── Y el tabulador del nombre volvió entero ──
            let columnas = cabecera.columnas_de("contact").expect("contact");
            let i = columnas
                .iter()
                .position(|c| c == "first_name")
                .expect("contact tiene first_name");
            let nombres: Vec<Option<String>> = cuerpo
                .lines()
                .take_while(|l| !l.is_empty())
                .filter_map(|l| arles_core::formato_de_respaldo::leer_fila(l).ok())
                .filter(|(t, _)| t == "contact")
                .filter_map(|(_, v)| v.get(i).cloned())
                .collect();
            assert!(
                nombres.contains(&Some("Beto con\tun tabulador".to_owned())),
                "el tabulador del nombre no sobrevivió: {nombres:?}"
            );
        }
    }

    /// **Un respaldo cifrado no deja el texto claro en el archivo.**
    ///
    /// Suena obvio y es justo lo que hay que comprobar: un sobre mal enchufado
    /// —cifrar y escribir **también** el claro, o cifrar sólo parte— produce un
    /// archivo que se abre perfectamente con la contraseña y que además lleva
    /// los contactos a la vista. Ninguna otra prueba lo vería.
    ///
    /// Verificada el 05/10/2026 escribiendo el cuerpo sin pasarlo por el sobre:
    /// falla nombrando «Ana».
    #[test]
    fn un_respaldo_cifrado_no_deja_nada_legible() {
        let (_d, db) = base();
        let (cifrado, _, _) = escribir(&db, Some("una contraseña larga de verdad"));
        let (claro, _, _) = escribir(&db, None);

        // En el que no va cifrado, los nombres están. Si no estuvieran, la
        // comprobación de abajo pasaría por no haber nada que encontrar.
        assert!(
            claro.windows(3).any(|v| v == b"Ana"),
            "sin cifrar, el nombre tenía que estar en el archivo"
        );
        for secreto in [
            &b"Ana"[..],
            &b"Ruiz"[..],
            &b"hola@telemetry.mx"[..],
            &b"lista.csv"[..],
        ] {
            assert!(
                !cifrado.windows(secreto.len()).any(|v| v == secreto),
                "el respaldo cifrado deja a la vista {:?}",
                std::str::from_utf8(secreto).unwrap_or("?")
            );
        }
    }

    /// El avance avisa con cifras, y la última es el total.
    ///
    /// Una barra que se queda en el 98 % cuando ya terminó deja a alguien
    /// mirando un programa que cree colgado (§94).
    #[test]
    fn el_avance_llega_hasta_el_total() {
        let (_d, db) = base();
        let (_, filas, avisos) = escribir(&db, None);
        let ultimo = avisos.last().copied().expect("avisó al menos una vez");
        assert_eq!(
            (ultimo.filas, ultimo.total),
            (filas, filas),
            "el último aviso no es el total"
        );
    }

    /// Un alcance que no existe se rechaza, **no se aproxima a «esencial»**.
    ///
    /// Escribir un respaldo más pequeño del que alguien pidió es perder datos
    /// sin decirlo.
    #[test]
    fn un_alcance_inventado_no_se_aproxima() {
        assert!(que_se_lleva("todo-menos-las-bajas", "2026-01-01T00:00:00Z").is_err());
        assert!(que_se_lleva("", "2026-01-01T00:00:00Z").is_err());
        assert!(que_se_lleva("esencial", "2026-01-01T00:00:00Z").is_ok());
        assert!(que_se_lleva("completo", "2026-01-01T00:00:00Z").is_ok());
    }

    /// El corte del esencial cae **doce meses atrás**, no en una fecha fija.
    #[test]
    fn el_corte_del_esencial_es_de_hace_un_ano() {
        let corte = corte_del_esencial();
        let ahora = ahora_utc();
        assert!(corte < ahora, "el corte tiene que estar en el pasado");
        // Y a un año de distancia, no a un mes: el año que viene esta prueba
        // sigue valiendo porque compara con `ahora`, no con un literal.
        let anio_del_corte: i32 = corte.get(..4).unwrap_or("0").parse().unwrap_or(0);
        let anio_de_hoy: i32 = ahora.get(..4).unwrap_or("0").parse().unwrap_or(0);
        assert_eq!(
            anio_de_hoy - anio_del_corte,
            1,
            "el corte del esencial tiene que caer un año atrás: {corte}"
        );
    }
}
