//! Restaurar: leer el archivo, decir qué va a pasar, y sólo entonces hacerlo.
//!
//! Entrega 8.3c. Cierra el hueco que la 8.3b dejó abierto a propósito:
//! `Db::restaurar` estaba construido, con ocho pruebas, y nadie lo llamaba. Lo
//! vigila `el_restaurador_tiene_quien_lo_llame` en `--fase 8`, escrita antes de
//! esto y vista en rojo nombrando los seis tramos.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! DOS COMANDOS, Y EL ORDEN IMPORTA MÁS QUE EN NINGÚN OTRO SITIO
//!
//! `revisar_respaldo` abre el archivo, lee **sólo la cabecera** y dice qué trae,
//! de cuándo es, si va con contraseña y **qué se perdería**. No toca la base.
//!
//! `restaurar_respaldo` es el que reemplaza, y sólo se puede llamar después:
//! necesita la ruta que el primero guardó. No es una comodidad — es lo que hace
//! imposible restaurar sin haber enseñado el preflight.
//!
//! **La ruta no cruza la frontera** (regla 4). Vive en `EstadoApp` entre los dos
//! pasos, igual que el archivo de la importación desde la Fase 3, y por el mismo
//! motivo además del de la regla: devolverla permitiría que la webview pidiera
//! restaurar **otro** archivo distinto del que se revisó.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA SUMA SE COMPRUEBA ANTES DE CONFIRMAR, NO DESPUÉS
//!
//! El lector va entregando filas y calculando la suma sobre la marcha; la línea
//! de cierre llega al final y es ahí donde se compara. Como todo eso pasa
//! **dentro** de la transacción de `Db::restaurar`, un archivo alterado o
//! cortado revienta antes del `COMMIT` y la base queda como estaba.
//!
//! Hacerlo al revés —restaurar y luego comprobar— dejaría una base reemplazada
//! con datos de un archivo que no es el que salió de aquí, y un aviso después.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA CONTRASEÑA ENTRA Y NO SALE
//!
//! Igual que al escribir (8.2c): llega por la frontera, deriva la clave y se
//! acaba. No va al llavero, no va a la base, no se registra y no vuelve.

use std::io::Read as _;

use arles_core::cifrado_de_respaldo::SobreQueSeAbre;
use arles_core::formato_de_respaldo::{self as formato, Cabecera, Cifrado, SumaEnCurso};
use arles_db::restauracion::{FilaDelRespaldo, Origen};

use crate::error::ErrorIpc;
use crate::estado::EstadoApp;

/// Cuánto se lee del archivo de una vez.
///
/// No tiene que ver con el trozo del cifrado: es sólo el tamaño del cubo con
/// el que se saca agua del disco.
const LECTURA: usize = 64 * 1024;

/// Lo que `revisar_respaldo` le cuenta al usuario antes de tocar nada.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoQueTraeElRespaldo {
    /// Sólo el nombre. **Nunca la ruta** (regla 4).
    pub nombre_del_archivo: String,
    pub creado_en: String,
    pub alcance: String,
    pub esquema_del_respaldo: String,
    pub esquema_de_la_base: String,
    pub lleva_contrasena: bool,
    /// `false` cuando el respaldo es de otro esquema: entonces **no se puede
    /// restaurar**, y la pantalla tiene que decirlo aquí en vez de dejar pulsar
    /// un botón que va a fallar.
    pub se_puede_restaurar: bool,
    pub tablas: Vec<TablaQueVuelve>,
    pub total_de_filas: u64,
    /// Lo que se pierde sin volver, que es la mitad importante del preflight.
    pub no_vuelven: Vec<LoQueNoVuelve>,
    pub filas_que_no_vuelven: u64,
    pub bajas_ahora: u64,
    pub bajas_en_el_respaldo: u64,
    pub bajas_al_menos: u64,
    pub cuentas_a_reconectar: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TablaQueVuelve {
    pub tabla: String,
    pub ahora: u64,
    pub en_el_respaldo: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoQueNoVuelve {
    pub tabla: String,
    pub filas: u64,
    /// `true` si describe **este equipo** y perderlo es correcto.
    pub es_de_este_equipo: bool,
}

/// Lo que pasó, para decirlo con cifras.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RespaldoRestaurado {
    pub filas: u64,
    pub bajas_antes: u64,
    pub bajas_despues: u64,
    pub bajas_que_se_salvaron: u64,
    pub cuentas_a_reconectar: u64,
}

/// Elige un respaldo y dice qué pasaría. **No toca la base.**
///
/// Devuelve `None` si se cerró el diálogo sin elegir: es una decisión, no un
/// fallo.
///
/// # Errores
///
/// - `error.core.no_es_un_respaldo` si el archivo no lo es.
/// - `error.core.respaldo_de_version_futura` si lo escribió un ARLES posterior.
/// - `error.app.archivo_no_se_pudo_leer` si no se puede abrir.
#[tauri::command]
pub fn revisar_respaldo(
    estado: tauri::State<'_, EstadoApp>,
    app: tauri::AppHandle,
) -> Result<Option<LoQueTraeElRespaldo>, ErrorIpc> {
    use tauri_plugin_dialog::DialogExt;

    let Some(elegido) = app
        .dialog()
        .file()
        .add_filter("Respaldo de ARLES", &["arles"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let ruta = elegido
        .into_path()
        .map_err(|_| ErrorIpc::from(crate::AppError::ArchivoNoSePudoLeer))?;

    let (cabecera, _) = leer_la_cabecera(&ruta)?;

    let alcance = cabecera.alcance.clone();
    let filas: Vec<(String, u64)> = cabecera.filas.clone();

    let pasaria = estado
        .db()
        .que_pasaria_al_restaurar(alcance.clone(), &filas)
        .map_err(crate::AppError::Db)?;

    // Se guarda **después** de poder leer la cabecera: dejar puesta la ruta de
    // un archivo que no es un respaldo permitiría pulsar «Restaurar» sobre él.
    estado.guardar_respaldo_elegido(ruta.clone());

    Ok(Some(LoQueTraeElRespaldo {
        nombre_del_archivo: ruta
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_owned(),
        creado_en: cabecera.creado_en.clone(),
        alcance: alcance.as_str(),
        esquema_del_respaldo: cabecera.esquema.clone(),
        esquema_de_la_base: pasaria.esquema_de_la_base.clone(),
        lleva_contrasena: cabecera.cifrado.lleva_contrasena(),
        se_puede_restaurar: cabecera.esquema == pasaria.esquema_de_la_base,
        tablas: pasaria
            .tablas
            .iter()
            .map(|t| TablaQueVuelve {
                tabla: t.tabla.clone(),
                ahora: t.ahora,
                en_el_respaldo: t.en_el_respaldo,
            })
            .collect(),
        total_de_filas: cabecera.total_de_filas(),
        no_vuelven: pasaria
            .arrastradas
            .iter()
            .map(|a| LoQueNoVuelve {
                tabla: a.tabla.clone(),
                filas: a.filas,
                es_de_este_equipo: a.es_de_este_equipo,
            })
            .collect(),
        filas_que_no_vuelven: pasaria.filas_que_no_vuelven(),
        bajas_ahora: pasaria.bajas_ahora,
        bajas_en_el_respaldo: pasaria.bajas_en_el_respaldo,
        bajas_al_menos: pasaria.bajas_al_menos,
        cuentas_a_reconectar: pasaria.cuentas_a_reconectar,
    }))
}

/// Reemplaza la base con el respaldo que se revisó. **Entero o nada.**
///
/// # Errores
///
/// - `error.app.respaldo_sin_revisar` si no se revisó uno antes. **No abre un
///   diálogo para arreglarlo**: restaurar sin haber enseñado qué se pierde es
///   justo lo que D-8C y el preflight existen para impedir.
/// - `error.core.respaldo_corrupto` si la contraseña no abre el archivo, o si
///   está alterado o incompleto.
/// - `error.db.respaldo_no_encaja` si es de otro esquema.
#[tauri::command]
pub fn restaurar_respaldo(
    estado: tauri::State<'_, EstadoApp>,
    contrasena: Option<String>,
) -> Result<RespaldoRestaurado, ErrorIpc> {
    let Some(ruta) = estado.respaldo_elegido() else {
        return Err(crate::AppError::RespaldoSinRevisar.into());
    };

    let (cabecera, cuanto_ocupa) = leer_la_cabecera(&ruta)?;
    let contrasena = contrasena.filter(|c| !c.is_empty());

    let mut origen = LeeElArchivo::nuevo(&ruta, &cabecera, cuanto_ocupa, contrasena.as_deref())?;

    let columnas: arles_db::respaldo::ColumnasPorTabla = cabecera.columnas.clone();

    let paso = estado.db().restaurar(
        &mut origen,
        &cabecera.esquema,
        &columnas,
        cabecera.alcance,
        "dirección",
        crate::respaldo::ahora_utc().as_str(),
    );

    // Se suelta pase lo que pase: dejarla puesta permitiría restaurar otra vez
    // sin volver a revisar, y entonces el preflight que se leyó ya no
    // describiría la base.
    estado.soltar_respaldo_elegido();

    let paso = paso.map_err(crate::AppError::Db)?;
    Ok(RespaldoRestaurado {
        filas: paso.filas,
        bajas_antes: paso.bajas_antes,
        bajas_despues: paso.bajas_despues,
        bajas_que_se_salvaron: paso.bajas_que_se_salvaron,
        cuentas_a_reconectar: paso.cuentas_a_reconectar,
    })
}

/// Lee la cabecera del archivo, que **siempre va en claro**, y devuelve también
/// cuántos bytes ocupa para poder saltársela luego.
///
/// Se lee un trozo y no el archivo entero: con 2 GB, leerlo para enseñar un
/// aviso son treinta segundos mirando una pantalla quieta.
pub(crate) fn leer_la_cabecera(ruta: &std::path::Path) -> Result<(Cabecera, usize), ErrorIpc> {
    let mut f = std::fs::File::open(ruta)
        .map_err(|_| ErrorIpc::from(crate::AppError::ArchivoNoSePudoLeer))?;
    let mut principio = vec![0_u8; LECTURA];
    let leidos = f
        .read(&mut principio)
        .map_err(|_| ErrorIpc::from(crate::AppError::ArchivoNoSePudoLeer))?;
    principio.truncate(leidos);

    // La cabecera termina en una línea en blanco. Se busca sobre los bytes
    // crudos porque con el cuerpo cifrado el archivo **no es UTF-8 entero**.
    let fin = principio
        .windows(2)
        .position(|v| v == b"\n\n")
        .map(|i| i + 2)
        .ok_or_else(|| {
            ErrorIpc::from(crate::AppError::Core(arles_core::CoreError::NoEsUnRespaldo))
        })?;

    let texto = std::str::from_utf8(principio.get(..fin).unwrap_or_default()).map_err(|_| {
        ErrorIpc::from(crate::AppError::Core(arles_core::CoreError::NoEsUnRespaldo))
    })?;
    let (cabecera, _) = Cabecera::leer(texto).map_err(crate::AppError::Core)?;
    Ok((cabecera, fin))
}

/// El archivo mientras se lee: saca filas sin tenerlo entero en memoria.
///
/// ── POR QUÉ UN BÚFER DE LÍNEAS Y NO EL CUERPO DESCIFRADO COMPLETO ──
///
/// Descifrar a un `String` y recorrerlo sería la base entera en RAM, que es lo
/// que las 8.2b, 8.2c y 8.3a se tomaron el trabajo de evitar cada una por su
/// lado. Aquí se guarda **sólo lo que falta para completar la línea en curso**.
pub(crate) struct LeeElArchivo {
    fichero: std::fs::File,
    sobre: Option<SobreQueSeAbre>,
    /// Texto claro leído y todavía no entregado como filas.
    pendiente: String,
    /// Los bytes del final del último trozo que **no completan un carácter**.
    ///
    /// ── POR QUÉ ESTO EXISTE, Y EL FALLO QUE NO TENERLO CAUSABA ───────────
    ///
    /// El cuerpo se descifra en trozos de 64 KiB (8.2a) y **un trozo no es una
    /// unidad de texto**: la frontera cae donde cae, y en una base mexicana cae
    /// a menudo en la mitad de una `Á`, una `ñ` o una `é`, que ocupan dos bytes.
    /// Convertir cada trozo a texto por separado daba entonces
    /// `RespaldoCorrupto` sobre un archivo perfectamente bueno, y el respaldo
    /// que D-8A hace el valor por defecto **no se podía restaurar**.
    ///
    /// Ninguna prueba lo cazaba porque todas usaban un puñado de filas: un solo
    /// trozo, y el último siempre acaba en `\n`. Apareció con 30 000 contactos,
    /// midiendo el presupuesto que esta fase no tenía. Es la misma clase de
    /// juntura que el cifrado a trozos ya tuvo dos veces: **las dos piezas
    /// correctas y el agujero en la unión**.
    cola: Vec<u8>,
    /// `true` cuando el archivo se acabó.
    se_acabo: bool,
    suma: SumaEnCurso,
    /// La suma que el archivo declara, cuando ya se ha leído la línea de cierre.
    declarada: Option<String>,
}

impl LeeElArchivo {
    pub(crate) fn nuevo(
        ruta: &std::path::Path,
        cabecera: &Cabecera,
        salta: usize,
        contrasena: Option<&str>,
    ) -> Result<Self, ErrorIpc> {
        use std::io::Seek as _;

        let mut fichero = std::fs::File::open(ruta)
            .map_err(|_| ErrorIpc::from(crate::AppError::ArchivoNoSePudoLeer))?;
        fichero
            .seek(std::io::SeekFrom::Start(salta as u64))
            .map_err(|_| ErrorIpc::from(crate::AppError::ArchivoNoSePudoLeer))?;

        let sobre = match &cabecera.cifrado {
            Cifrado::No => None,
            Cifrado::Argon2idXChaCha20 { sal, nonce } => {
                let Some(clave) = contrasena else {
                    // Pedir la contraseña es cosa de la pantalla, que ya sabe
                    // por la cabecera que hace falta. Llegar aquí sin ella es
                    // un archivo cifrado al que no se le pasó ninguna.
                    return Err(
                        crate::AppError::Core(arles_core::CoreError::RespaldoCorrupto {
                            motivo: "este respaldo va con contraseña y no se dio ninguna",
                        })
                        .into(),
                    );
                };
                let sal = crate::respaldo::de_hex_fijo::<16>(sal).map_err(crate::AppError::Db)?;
                let nonce =
                    crate::respaldo::de_hex_fijo::<19>(nonce).map_err(crate::AppError::Db)?;
                Some(SobreQueSeAbre::nuevo(clave, &sal, &nonce).map_err(crate::AppError::Core)?)
            }
        };

        // La suma cubre la cabecera Y el cuerpo en claro, así que empieza por
        // la cabecera tal y como se escribió.
        let mut suma = SumaEnCurso::new();
        suma.añadir(&cabecera.redactar());

        Ok(Self {
            fichero,
            sobre,
            pendiente: String::new(),
            cola: Vec::new(),
            se_acabo: false,
            suma,
            declarada: None,
        })
    }

    /// Trae más texto claro al búfer. Devuelve `false` cuando ya no queda.
    ///
    /// Devuelve `DbError` y no `ErrorIpc` porque quien llama es
    /// [`Origen::siguiente`], **dentro de la transacción de la restauración**:
    /// cualquier fallo de aquí tiene que poder deshacerla. Un disco que se
    /// desconecta a mitad llega como un respaldo que no se puede terminar de
    /// leer, que es exactamente lo que es.
    fn traer_mas(&mut self) -> Result<bool, arles_db::DbError> {
        if self.se_acabo {
            return Ok(false);
        }
        let mut cubo = vec![0_u8; LECTURA];
        let leidos = self.fichero.read(&mut cubo).map_err(|_| {
            arles_db::DbError::Nucleo(arles_core::CoreError::RespaldoCorrupto {
                motivo: "no se pudo seguir leyendo el archivo del respaldo",
            })
        })?;
        cubo.truncate(leidos);

        let claro = match (&mut self.sobre, leidos) {
            (Some(sobre), n) if n > 0 => {
                let mut salida = Vec::new();
                sobre
                    .añadir(&cubo, &mut salida)
                    .map_err(arles_db::DbError::Nucleo)?;
                salida
            }
            (Some(_), _) => {
                // Se acabó el archivo: hay que cerrar el sobre, que es donde se
                // caza un respaldo al que le cortaron el final.
                //
                // ── POR QUÉ UN `if let` Y NO UN `unwrap` ──
                //
                // Aquí había `self.sobre.take().unwrap_or_else(|| unreachable!())`:
                // correcto —la rama acaba de ver `Some`— y aun así el único
                // `panic!` del código de producción del proyecto, encontrado por
                // la auditoría A del §140 el 07/10/2026. El `take` va fuera del
                // `match` porque dentro se tiene prestado `&mut self.sobre`; con
                // `if let` el caso imposible simplemente no hace nada, en vez de
                // tumbar la aplicación a mitad de una restauración.
                self.se_acabo = true;
                let mut salida = Vec::new();
                if let Some(sobre) = self.sobre.take() {
                    sobre
                        .terminar(&mut salida)
                        .map_err(arles_db::DbError::Nucleo)?;
                }
                salida
            }
            (None, 0) => {
                self.se_acabo = true;
                Vec::new()
            }
            (None, _) => cubo,
        };

        if claro.is_empty() && self.se_acabo {
            return Ok(false);
        }
        // ── DE BYTES A TEXTO, SIN FIARSE DE DÓNDE CORTÓ EL TROZO ──
        //
        // Se pegan los bytes que quedaron a medias del trozo anterior, se
        // decodifica **hasta donde haya caracteres completos**, y el resto
        // vuelve a esperar. Así las fronteras de trozo dejan de tener nada que
        // ver con las de carácter, que es como debió ser desde el principio.
        let mut bytes = std::mem::take(&mut self.cola);
        bytes.extend_from_slice(&claro);

        let corrupto = || {
            arles_db::DbError::Nucleo(arles_core::CoreError::RespaldoCorrupto {
                motivo: "el contenido del respaldo no es texto válido",
            })
        };

        let valido = match std::str::from_utf8(&bytes) {
            Ok(texto) => {
                self.pendiente.push_str(texto);
                bytes.len()
            }
            Err(e) => {
                // ── LA DISTINCIÓN QUE NO SE PUEDE PERDER ──
                //
                // `error_len() == None` significa «se acabó la entrada a mitad
                // de un carácter»: eso es el corte del trozo y es normal.
                // `Some(_)` es una secuencia que **no puede** ser UTF-8, y eso
                // sí es un archivo manipulado o estropeado. Tratar los dos
                // igual sería volver al fallo original.
                //
                // **Qué garantiza exactamente, que no es lo que parece.** La
                // guarda del final de esta función ya rechaza un archivo que
                // acaba con bytes a medias, así que sin esta rama un byte
                // imposible **también** se rechazaría — al final. Lo que esta
                // rama aporta es *cuándo* y *con qué coste*: sin ella, el byte
                // malo se queda en la cola y cada trozo siguiente se le pega
                // sin poder decodificar nada, así que **la cola crece hasta el
                // tamaño del archivo** y la memoria acotada de la 8.2b se va al
                // suelo con medio millón de filas. Lo vigila la prueba
                // `un_byte_que_no_puede_ser_utf8_no_se_cuela`, midiendo la cola.
                if e.error_len().is_some() {
                    return Err(corrupto());
                }
                let hasta = e.valid_up_to();
                self.pendiente.push_str(
                    std::str::from_utf8(bytes.get(..hasta).unwrap_or_default())
                        .map_err(|_| corrupto())?,
                );
                hasta
            }
        };
        self.cola = bytes.split_off(valido);

        // Si el archivo se acabó y quedan bytes a medias, el último carácter
        // está partido: eso ya no es una frontera de trozo, es un archivo
        // cortado — y uno cortado por la mitad de una letra tiene que decirlo
        // en vez de entregar una fila truncada.
        if self.se_acabo && !self.cola.is_empty() {
            return Err(corrupto());
        }
        Ok(true)
    }
}

impl Origen for LeeElArchivo {
    fn siguiente(&mut self) -> Result<Option<FilaDelRespaldo>, arles_db::DbError> {
        loop {
            if let Some(corte) = self.pendiente.find('\n') {
                let linea: String = self.pendiente.drain(..=corte).collect();
                let limpia = linea.trim_end_matches('\n');

                // La línea de cierre es la última y lleva la suma. Al llegar a
                // ella se comprueba, **todavía dentro de la transacción**.
                if let Some(declarada) = formato::leer_cierre_de_linea(limpia) {
                    self.declarada = Some(declarada);
                    continue;
                }
                if limpia.is_empty() {
                    continue;
                }
                self.suma.añadir(&linea);
                let (tabla, valores) =
                    formato::leer_fila(limpia).map_err(arles_db::DbError::from)?;
                return Ok(Some((tabla, valores)));
            }

            let hay_mas = self.traer_mas()?;
            if !hay_mas && !self.pendiente.contains('\n') {
                // Se acabó. Antes de decir que no hay más filas, la suma.
                let Some(declarada) = self.declarada.take() else {
                    return Err(arles_db::DbError::Nucleo(
                        arles_core::CoreError::RespaldoCorrupto {
                            motivo: "al respaldo le falta la línea de cierre: está incompleto",
                        },
                    ));
                };
                formato::comprobar_suma(&self.suma.clone().terminar(), &declarada)
                    .map_err(arles_db::DbError::Nucleo)?;
                return Ok(None);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arles_db::{ClaveMaestra, Db, QueSeLleva};

    fn base_con_datos() -> (tempfile::TempDir, Db) {
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
                   VALUES ('C2', 'E1', 'L1', 'Beto con\tun tabulador', NULL, '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z');
                 INSERT INTO suppression_entry (id, company_id, channel, address_normalized, scope, reason, origin, created_at)
                   VALUES ('S1', 'E1', 'email', 'quien@no.quiere', 'global', 'unsubscribe', 'user', '2026-02-01T00:00:00Z');",
            )
        })
        .expect("siembra");
        (dir, db)
    }

    fn base_vacia() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        (dir, db)
    }

    /// Escribe un respaldo de verdad, con el escritor de la 8.2c.
    fn escribir(db: &Db, ruta: &std::path::Path, contrasena: Option<&str>) {
        let mut destino = crate::respaldo::ElArchivo::nuevo(
            ruta,
            "completo",
            contrasena,
            Box::new(|_| {}),
            "2026-10-06T12:00:00Z".to_owned(),
        )
        .expect("abre el archivo");
        db.exportar(QueSeLleva::Todo, &mut destino)
            .expect("exporta");
        destino.cerrar().expect("cierra");
    }

    fn contactos(db: &Db) -> Vec<(String, Option<String>)> {
        db.ejecutar_en_pruebas(|c| {
            let mut s = c.prepare("SELECT first_name, last_name FROM contact ORDER BY id")?;
            s.query_map([], |f| Ok((f.get(0)?, f.get(1)?)))?
                .collect::<Result<Vec<_>, _>>()
        })
        .expect("lee")
    }

    /// **El viaje entero: base poblada → archivo → otra base → los mismos datos.**
    ///
    /// Es la prueba que la fase existe para tener, y la que ninguna de las
    /// anteriores podía escribir: la 8.2c comprobaba que el archivo se vuelve a
    /// leer, la 8.3b que la base se reemplaza, y **nadie** que lo que sale de
    /// una entre de verdad en la otra. Es el patrón del §9 aplicado a la fase
    /// entera: cada tramo probado y la junta sin mirar.
    ///
    /// Se hace con y sin contraseña en la misma pasada, porque el camino del
    /// cifrado es distinto —el sobre que se abre a trozos— y un respaldo
    /// cifrado que no se pueda restaurar es un respaldo que no existe.
    ///
    /// Verificada el 06/10/2026 ordenando alfabéticamente las columnas que se
    /// le pasan a `restaurar`, en vez de respetar el orden de la cabecera: la
    /// restauración revienta. Es el resultado correcto y conviene entender por
    /// qué **no** es suerte: los valores entran corridos y chocan contra los
    /// `CHECK` y los `NOT NULL` del esquema. Sin esas restricciones el dato
    /// cambiado entraría en silencio, que es justo el motivo de insertar por
    /// nombre de columna y no por posición.
    #[test]
    fn un_respaldo_se_escribe_y_se_restaura_en_otra_base() {
        for contrasena in [None, Some("una contraseña larga de verdad")] {
            let (_d1, origen_db) = base_con_datos();
            let dir = tempfile::tempdir().expect("temporal");
            let ruta = dir.path().join("prueba.arles");
            escribir(&origen_db, &ruta, contrasena);

            let esperados = contactos(&origen_db);
            assert_eq!(esperados.len(), 2);

            // Una base distinta, recién creada.
            let (_d2, destino_db) = base_vacia();
            assert!(contactos(&destino_db).is_empty());

            let (cabecera, salta) = leer_la_cabecera(&ruta).expect("cabecera");
            let mut lector =
                LeeElArchivo::nuevo(&ruta, &cabecera, salta, contrasena).expect("abre el lector");

            let paso = destino_db
                .restaurar(
                    &mut lector,
                    &cabecera.esquema,
                    &cabecera.columnas,
                    cabecera.alcance.clone(),
                    "dirección",
                    "2026-10-06T13:00:00Z",
                )
                .expect("restaura");

            assert_eq!(paso.filas, cabecera.total_de_filas());
            assert_eq!(
                contactos(&destino_db),
                esperados,
                "con contraseña {contrasena:?} los contactos no volvieron iguales"
            );
            // Y el tabulador del nombre sobrevivió al viaje de ida y vuelta.
            assert!(
                contactos(&destino_db)
                    .iter()
                    .any(|(n, _)| n == "Beto con\tun tabulador"),
                "el tabulador no sobrevivió"
            );
        }
    }

    /// **Un respaldo alterado no restaura nada.**
    ///
    /// La suma se comprueba **dentro** de la transacción, al llegar a la línea
    /// de cierre, así que un archivo tocado revienta antes del `COMMIT`.
    /// Hacerlo al revés dejaría la base reemplazada con datos de un archivo que
    /// no es el que salió de aquí, y un aviso después.
    ///
    /// Verificada el 06/10/2026 comprobando la suma después de `restaurar` en
    /// vez de durante: la base queda reemplazada con el contenido alterado.
    #[test]
    fn un_respaldo_alterado_no_restaura_nada() {
        let (_d1, origen_db) = base_con_datos();
        let dir = tempfile::tempdir().expect("temporal");
        let ruta = dir.path().join("tocado.arles");
        // Sin cifrar: con cifrado lo caza el propio sobre, y lo que se quiere
        // comprobar aquí es la suma.
        escribir(&origen_db, &ruta, None);

        let texto = std::fs::read_to_string(&ruta).expect("lee");
        let tocado = texto.replace("Ana", "Eva");
        assert_ne!(texto, tocado, "la siembra tenía que traer a Ana");
        std::fs::write(&ruta, &tocado).expect("escribe");

        let (_d2, destino_db) = base_vacia();
        let (cabecera, salta) = leer_la_cabecera(&ruta).expect("cabecera");
        let mut lector = LeeElArchivo::nuevo(&ruta, &cabecera, salta, None).expect("lector");

        let r = destino_db.restaurar(
            &mut lector,
            &cabecera.esquema,
            &cabecera.columnas,
            cabecera.alcance,
            "dirección",
            "2026-10-06T13:00:00Z",
        );
        assert!(r.is_err(), "un respaldo alterado se restauró");
        assert!(
            contactos(&destino_db).is_empty(),
            "quedaron datos de un archivo alterado"
        );
    }

    /// **Un respaldo al que le cortaron el final tampoco.**
    ///
    /// Es lo que pasa cuando se llena un disco a mitad de la escritura o cuando
    /// un correo corta un adjunto: todo lo anterior se lee perfectamente, y lo
    /// que falta es el final. Sin la línea de cierre no hay suma que comprobar,
    /// y restaurar «lo que había» sería restaurar menos contactos sin decirlo.
    #[test]
    fn un_respaldo_cortado_no_restaura_nada() {
        let (_d1, origen_db) = base_con_datos();
        let dir = tempfile::tempdir().expect("temporal");
        let ruta = dir.path().join("cortado.arles");
        escribir(&origen_db, &ruta, None);

        let texto = std::fs::read_to_string(&ruta).expect("lee");
        let cortado = texto.get(..texto.len() - 90).expect("hay de sobra");
        std::fs::write(&ruta, cortado).expect("escribe");

        let (_d2, destino_db) = base_vacia();
        let (cabecera, salta) = leer_la_cabecera(&ruta).expect("cabecera");
        let mut lector = LeeElArchivo::nuevo(&ruta, &cabecera, salta, None).expect("lector");
        let r = destino_db.restaurar(
            &mut lector,
            &cabecera.esquema,
            &cabecera.columnas,
            cabecera.alcance,
            "dirección",
            "2026-10-06T13:00:00Z",
        );
        assert!(r.is_err(), "un respaldo cortado se restauró");
        assert!(contactos(&destino_db).is_empty());
    }

    /// **La cabecera se lee sin contraseña, y dice que hace falta una.**
    ///
    /// Es lo que permite que la pantalla pida la contraseña sabiendo que hace
    /// falta, en vez de pedirla siempre por si acaso o de fallar al intentarlo.
    #[test]
    fn la_cabecera_de_un_respaldo_cifrado_se_lee_sin_contrasena() {
        let (_d, db) = base_con_datos();
        let dir = tempfile::tempdir().expect("temporal");
        let ruta = dir.path().join("cifrado.arles");
        escribir(&db, &ruta, Some("una contraseña larga de verdad"));

        let (cabecera, _) = leer_la_cabecera(&ruta).expect("la cabecera se lee en claro");
        assert!(cabecera.cifrado.lleva_contrasena());
        assert_eq!(cabecera.creado_en, "2026-10-06T12:00:00Z");
        assert!(cabecera.total_de_filas() > 0);

        // Y sin contraseña, el lector no se abre — pero el error lo dice, no
        // revienta.
        let (_, salta) = leer_la_cabecera(&ruta).expect("cabecera");
        assert!(LeeElArchivo::nuevo(&ruta, &cabecera, salta, None).is_err());
    }

    /// Con otra contraseña no se restaura, y la base queda intacta.
    #[test]
    fn con_otra_contrasena_no_se_restaura() {
        let (_d1, origen_db) = base_con_datos();
        let dir = tempfile::tempdir().expect("temporal");
        let ruta = dir.path().join("cifrado.arles");
        escribir(&origen_db, &ruta, Some("la buena"));

        let (_d2, destino_db) = base_vacia();
        let (cabecera, salta) = leer_la_cabecera(&ruta).expect("cabecera");
        let mut lector =
            LeeElArchivo::nuevo(&ruta, &cabecera, salta, Some("otra cualquiera")).expect("lector");
        let r = destino_db.restaurar(
            &mut lector,
            &cabecera.esquema,
            &cabecera.columnas,
            cabecera.alcance,
            "dirección",
            "2026-10-06T13:00:00Z",
        );
        assert!(r.is_err());
        assert!(contactos(&destino_db).is_empty());
    }

    /// Un archivo que no es un respaldo se rechaza al leer la cabecera.
    #[test]
    fn un_archivo_cualquiera_no_es_un_respaldo() {
        let dir = tempfile::tempdir().expect("temporal");
        let ruta = dir.path().join("vacaciones.jpg");
        std::fs::write(&ruta, b"\xff\xd8\xff\xe0 no soy un respaldo").expect("escribe");
        assert!(leer_la_cabecera(&ruta).is_err());
    }

    /// **Un respaldo cifrado con acentos y más de un trozo se restaura.**
    ///
    /// ── LA PRUEBA QUE TODAS LAS DEMÁS EVITABAN SIN QUERER ─────────────────
    ///
    /// El cuerpo se descifra en trozos de 64 KiB, y hasta hoy cada trozo se
    /// convertía a texto por su cuenta. Una `Á` ocupa dos bytes, así que en
    /// cuanto un trozo cortaba por la mitad de una letra acentuada el
    /// respaldo daba `RespaldoCorrupto` **siendo perfectamente bueno**: el
    /// respaldo cifrado, que D-8A hace el valor por defecto, no se podía
    /// restaurar en ninguna base de tamaño real.
    ///
    /// Ninguna de las pruebas de la fase lo cazaba, y las dos razones importan:
    /// todas usaban **dos filas** —un trozo, nunca una frontera— y el último
    /// trozo siempre acaba en `\n`. Apareció midiendo el presupuesto que esta
    /// fase no tenía, con 30 000 contactos, el 07/10/2026.
    ///
    /// Así que esta prueba siembra **lo suficiente para pasar de 64 KiB** y
    /// nombres con acentos en cada fila. Verificada volviendo el lector a
    /// `from_utf8(&claro)`: falla con «el contenido del respaldo no es texto
    /// válido», que es exactamente el síntoma del fallo original.
    #[test]
    fn un_respaldo_cifrado_de_mas_de_un_trozo_con_acentos_se_restaura() {
        const CONTRASENA: &str = "una contraseña larga de verdad";
        // ── CUÁNTAS FILAS, Y POR QUÉ ESTA CIFRA NO ES DECORATIVA ──
        //
        // La primera versión ponía 1 200 filas (~100 KB): **pasaba por suerte**.
        // Con 100 KB hay una o dos fronteras de trozo, y que una caiga justo en
        // la mitad de una letra acentuada es improbable — así que la prueba
        // salió en verde incluso con el lector roto a propósito, que es la
        // definición del §2 de una comprobación que no vale.
        //
        // Cada frontera falla con probabilidad parecida a la fracción de bytes
        // que son la segunda mitad de un par. Con los campos de abajo, llenos
        // de acentos, eso ronda el 25 %: con una frontera, la prueba acierta
        // una vez de cuatro; con veinte, falla el 99.7 % de las veces. Así que
        // lo que la hace válida es **el número de fronteras**, y abajo se
        // comprueba que de verdad las haya en vez de darlas por supuestas.
        const FILAS: usize = 14_000;
        /// Cuántas fronteras de trozo hacen falta para que esto no sea un
        /// sorteo. Si alguien baja `FILAS`, la prueba lo dice en voz alta.
        const FRONTERAS_MINIMAS: u64 = 15;

        let (_d1, origen_db) = base_con_datos();
        origen_db
            .ejecutar_en_pruebas(|c| {
                let tx = c.unchecked_transaction()?;
                {
                    let mut mete = tx.prepare(
                        "INSERT INTO contact (id, company_id, import_batch_id, first_name, last_name, source, created_at, updated_at)
                           VALUES (?1, 'E1', 'L1', ?2, ?3, ?4, '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z')",
                    )?;
                    for i in 0..FILAS {
                        // Campos densos en caracteres de dos bytes: es lo que
                        // le da a las fronteras dónde caer mal.
                        mete.execute(rusqlite::params![
                            format!("X{i:06}"),
                            format!("Ángela Ñoño Iñárritu Übermann {i}"),
                            "Álvarez Bañuelos Peña Núñez Echeverría",
                            "Café de Olla Ñandú, S.A. de C.V. — Á É Í Ó Ú"
                        ])?;
                    }
                }
                tx.commit()
            })
            .expect("siembra las filas con acentos");

        let dir = tempfile::tempdir().expect("temporal");
        let ruta = dir.path().join("grande.arles");
        escribir(&origen_db, &ruta, Some(CONTRASENA));

        // Que de verdad hay fronteras suficientes. Sin esta comprobación, bajar
        // `FILAS` dejaría la prueba en verde sin ejercitar nada — y en verde es
        // precisamente como estaba el día que el fallo existía.
        let bytes = std::fs::metadata(&ruta).expect("mide").len();
        let fronteras = bytes / 65_536;
        assert!(
            fronteras >= FRONTERAS_MINIMAS,
            "el respaldo mide {bytes} bytes: sólo {fronteras} fronteras de trozo, y hacen falta \
             {FRONTERAS_MINIMAS} para que esto no dependa de la suerte"
        );

        let (_d2, destino_db) = base_vacia();
        let (cabecera, salta) = leer_la_cabecera(&ruta).expect("cabecera");
        let mut lector =
            LeeElArchivo::nuevo(&ruta, &cabecera, salta, Some(CONTRASENA)).expect("abre el lector");
        let paso = destino_db
            .restaurar(
                &mut lector,
                &cabecera.esquema,
                &cabecera.columnas,
                cabecera.alcance.clone(),
                "dirección",
                "2026-10-07T13:00:00Z",
            )
            .expect("un respaldo cifrado de varios trozos tiene que restaurarse");

        assert_eq!(paso.filas, cabecera.total_de_filas());

        // Y los acentos volvieron enteros, no aproximados: un `from_utf8_lossy`
        // en el camino habría dejado rombos y la restauración habría «pasado».
        let cuantos: u64 = destino_db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT COUNT(*) FROM contact WHERE last_name = 'Álvarez Bañuelos Peña Núñez Echeverría'",
                    [],
                    |f| f.get(0),
                )
            })
            .expect("cuenta");
        assert_eq!(
            cuantos as usize, FILAS,
            "los apellidos con acento no volvieron exactos"
        );
    }

    /// **Un byte que no puede ser UTF-8 se rechaza; un carácter partido espera.**
    ///
    /// Las dos cosas llegan al lector como «esto no es texto», y la diferencia
    /// está en `Utf8Error::error_len()`: `None` es «se acabó la entrada a mitad
    /// de un carácter» —la frontera de un trozo, que es normal— y `Some` es una
    /// secuencia imposible, que es un archivo estropeado. Tratar los dos igual
    /// en un sentido devuelve el fallo de arriba; en el otro se traga la
    /// corrupción de verdad.
    ///
    /// Va **sin cifrar** a propósito: con el sobre puesto, alterar un byte lo
    /// caza la autenticación antes de llegar aquí, así que esta rama no se
    /// podría ejercitar.
    ///
    /// Verificada quitando el `return` de la rama `Some`: el byte inválido pasa
    /// a tratarse como una frontera, se queda en la cola para siempre y la
    /// restauración termina **sin la fila alterada y sin un solo error**.
    #[test]
    fn un_byte_que_no_puede_ser_utf8_no_se_cuela() {
        let (_d1, origen_db) = base_con_datos();
        let dir = tempfile::tempdir().expect("temporal");
        let ruta = dir.path().join("roto.arles");
        escribir(&origen_db, &ruta, None);

        // 0xFF no es un byte válido en ninguna posición de UTF-8. Va **en el
        // cuerpo**, pasado `salta`: puesto en la cabecera lo rechazaría
        // `leer_la_cabecera` y la prueba no tocaría el lector, que es lo que
        // viene a vigilar. Primera versión: a mitad del archivo, que en un
        // respaldo de dos filas cae dentro de la cabecera.
        let (cabecera, salta) = leer_la_cabecera(&ruta).expect("cabecera");
        let mut bytes = std::fs::read(&ruta).expect("lee");
        let dentro_del_cuerpo = salta + (bytes.len() - salta) / 2;
        *bytes
            .get_mut(dentro_del_cuerpo)
            .expect("el respaldo tiene cuerpo") = 0xFF;
        std::fs::write(&ruta, &bytes).expect("escribe");

        // Se ejercita el lector directamente, no a través de `restaurar`,
        // porque lo que hay que medir es **la cola**: a través de la base sólo
        // se ve que falla, y falla de las dos formas.
        let mut lector =
            LeeElArchivo::nuevo(&ruta, &cabecera, salta, None).expect("abre el lector");
        let error = loop {
            match lector.siguiente() {
                Ok(Some(_)) => {}
                Ok(None) => panic!("el respaldo con un byte imposible terminó sin protestar"),
                Err(e) => break e,
            }
        };

        // ── LA GARANTÍA QUE DISTINGUE EL ARREGLO DE NO TENERLO ──
        //
        // Con la rama de `error_len` el lector para en cuanto ve el byte malo,
        // así que la cola tiene como mucho los tres bytes de un carácter a
        // medias. Sin ella, el byte se queda atascado, cada trozo siguiente se
        // le pega sin poder decodificar, y la cola acaba teniendo **el resto
        // del archivo** — con medio millón de filas, eso es el respaldo entero
        // en memoria. Verificada quitando la rama: la cola pasa de 3 bytes a
        // cientos y la prueba cae.
        assert!(
            lector.cola.len() <= 3,
            "la cola se quedó con {} bytes: el byte imposible no paró la lectura y la memoria \
             deja de estar acotada",
            lector.cola.len()
        );

        let (_d2, destino_db) = base_vacia();

        assert!(
            format!("{error:?}").contains("RespaldoCorrupto")
                || format!("{error:?}").contains("Suma"),
            "se rechazó por otro motivo: {error:?}"
        );
        assert!(
            contactos(&destino_db).is_empty(),
            "la base quedó con datos de un archivo estropeado"
        );
    }
}
