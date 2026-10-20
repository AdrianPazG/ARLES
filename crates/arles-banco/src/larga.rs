//! La corrida larga del motor: ¿la memoria se estabiliza o crece sin parar?
//!
//! Entrega 8.6 C, auditoría de QA y rendimiento del §140. Es el tercer punto
//! de esa pasada —«sin fugas de memoria en ejecuciones largas del motor»— y
//! **nunca se había intentado**: las trece cifras de la 8.5 incluían «memoria
//! en reposo con motor activo» como `SinMedir`, y el motivo que daba era que
//! sin llavero no arranca la aplicación entera. Eso es verdad de la
//! aplicación y falso del motor, que corre contra un proveedor simulado desde
//! la 4.3a.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO DIFÍCIL DE ESTO NO ES MEDIR LA MEMORIA: ES SABER QUÉ CRECIMIENTO ES UNA
//! FUGA
//!
//! Un proceso que envía cien mil correos **tiene** que crecer al principio:
//! SQLite llena su caché de páginas, el asignador pide arenas al sistema y no
//! se las devuelve en cuanto puede. Ese crecimiento es correcto y se detiene.
//!
//! Lo que `PRESUPUESTO_RENDIMIENTO.md` §7 llama fuga es otra cosa: «memoria
//! que **crece sin estabilizarse**». Así que no se mide un número, se mide una
//! **forma**: se parte la corrida en cuatro tramos y se compara el último con
//! el segundo. Si el segundo tramo ya está en su techo, los dos son iguales.
//! Si algo se acumula por cada envío, el último es mayor y la diferencia crece
//! con la corrida.
//!
//! Comparar el último con el **primero** sería el error obvio: ahí está todo
//! el arranque, y una corrida sana saldría en rojo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! Y SE VIO FALLAR, PORQUE SI NO NO ESTÁ VERIFICADA (§2)
//!
//! Medido el 07/10/2026, 20 000 envíos consecutivos:
//!
//! ```text
//! sin fuga:  13.9 · 13.9 · 13.9 · 13.9 MB   crecimiento  0.00 MB
//! con fuga:  13.9 · 14.2 · 14.6 · 15.0 MB   crecimiento +0.79 MB
//! ```
//!
//! La fuga se mete con `banco larga 20000 fuga`, que le dice al proveedor que
//! **guarde cada pedido**. Y no es un ejemplo inventado para la ocasión: es
//! exactamente lo que hace `ProveedorSimulado`, el del taller, y es la razón
//! por la que esta corrida **no lo usa**. Un instrumento que acumula por cada
//! envío acusaría al motor de una fuga que es suya.
//!
//! De paso sale el rendimiento: 13 ms por envío, unas 77 por segundo, contra
//! los 50 **diarios** que el producto permite (§48).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ NO SE USA `ProveedorSimulado`
//!
//! Porque **guarda cada pedido que recibe** para que las pruebas puedan
//! preguntarle a quién le pidieron enviar. En una corrida de cien mil pasos
//! eso es un vector de cien mil registros creciendo por cada envío: no es una
//! fuga del producto, pero la comprobación no sabe distinguirlas y saldría en
//! rojo acusando al motor de algo que hace el instrumento.
//!
//! El de aquí no recuerda nada, y es la razón por la que está escrito aparte.

use std::time::Instant;

use arles_motor::{Ajustes, Entorno, Proveedor, Trabajador};

/// Un proveedor que acepta todo y **no recuerda nada** — o que recuerda, si se
/// le pide, para poder ver fallar la comprobación.
///
/// Ver la nota de la cabecera: el del motor guarda los pedidos, y eso
/// falsearía justo lo que esto mide. Aquí se puede elegir, y el modo que
/// recuerda **existe para romper la comprobación a propósito** (§2): sin él,
/// «la memoria no crece» sería una afirmación que nadie ha visto caer.
#[derive(Debug, Default)]
struct SiempreAcepta {
    /// Lo que recibió, si se le dijo que lo guarde. Es la fuga.
    recordado: std::sync::Mutex<Vec<String>>,
    recuerda: bool,
}

impl Proveedor for SiempreAcepta {
    fn enviar(
        &self,
        mensaje: &arles_motor::Mensaje<'_>,
    ) -> Result<arles_motor::Aceptado, arles_core::reintento::Fallo> {
        if self.recuerda
            && let Ok(mut v) = self.recordado.lock()
        {
            v.push(format!(
                "{}|{}",
                mensaje.para, mensaje.clave_de_idempotencia
            ));
        }
        Ok(arles_motor::Aceptado {
            id_del_proveedor: None,
        })
    }
}

/// Reloj y azar quietos: la corrida no depende de cuándo se lance.
#[derive(Debug)]
struct Quieto(time::OffsetDateTime);

impl Entorno for Quieto {
    fn ahora(&self) -> time::OffsetDateTime {
        self.0
    }
    fn azar(&self) -> f64 {
        0.5
    }
}

/// Lo que salió de la corrida.
#[derive(Debug, Clone)]
pub struct Corrida {
    /// Cuántos pasos dio el motor de verdad.
    pub pasos: u32,
    /// Cuántos acabaron en un envío aceptado.
    pub enviados: u32,
    /// La memoria residente en cada muestra, en bytes.
    pub muestras: Vec<u64>,
    pub tardo: std::time::Duration,
}

impl Corrida {
    /// La memoria media del tramo `[desde, hasta)` de las muestras.
    #[must_use]
    pub fn tramo(&self, desde: usize, hasta: usize) -> u64 {
        let trozo = self.muestras.get(desde..hasta).unwrap_or(&[]);
        if trozo.is_empty() {
            return 0;
        }
        let suma: u128 = trozo.iter().map(|m| u128::from(*m)).sum();
        u64::try_from(suma / trozo.len() as u128).unwrap_or(u64::MAX)
    }

    /// Cuánto creció el último cuarto respecto al segundo, en bytes.
    ///
    /// Puede ser negativo —el asignador devuelve páginas—, así que es `i64`.
    /// Un cero o un negativo es la forma sana.
    #[must_use]
    pub fn crecimiento(&self) -> i64 {
        let n = self.muestras.len();
        if n < 4 {
            return 0;
        }
        let cuarto = n / 4;
        let segundo = self.tramo(cuarto, cuarto * 2);
        let ultimo = self.tramo(cuarto * 3, n);
        i64::try_from(ultimo).unwrap_or(i64::MAX) - i64::try_from(segundo).unwrap_or(i64::MAX)
    }
}

/// La memoria residente **de ahora**, no la marca de agua.
///
/// `VmHWM` —lo que mide el pico de la importación— no baja nunca, así que no
/// puede distinguir «creció y se estabilizó» de «sigue creciendo». Aquí hace
/// falta `VmRSS`.
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

/// Corre el motor `pasos` veces contra una cola llena y muestrea la memoria.
///
/// # Errores
///
/// Lo que falle al abrir la base, sembrarla o encolar. Un fallo del proveedor
/// no es un error: no hay ninguno que pueda fallar.
///
/// # Panics
///
/// Si los identificadores constantes de la siembra no se pueden interpretar,
/// que sería un error de este archivo y no del producto.
pub fn correr(pasos: u32, con_fuga: bool) -> Result<Corrida, Box<dyn std::error::Error>> {
    use arles_core::ritmo::Limites;
    use arles_core::ventana::{Dias, Ventana, zona};
    use arles_db::{ClaveMaestra, Db};

    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
    const CUENTA: &str = "cuenta-larga";
    const CAMPANA: &str = "camp-larga";
    /// Un miércoles a las 10 de la mañana en México: dentro de la ventana.
    const EN_HORARIO: time::OffsetDateTime = time::macros::datetime!(2026-09-23 16:00:00 UTC);

    let dir = tempfile::tempdir()?;
    let clave = ClaveMaestra::generar()?;
    let db = Db::abrir(&dir.path().join("arles.db"), &clave)?;

    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(&format!(
            "INSERT INTO company (id, commercial_name, country, timezone,
                                  corporate_email, created_at, updated_at)
             VALUES ('{EMPRESA}', 'Banco', 'MX', 'America/Mexico_City',
                     'banco@ejemplo.mx', '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');

             INSERT INTO email_account (id, company_id, display_name, email_address,
                                        provider_kind, credential_ref, status,
                                        daily_limit, hourly_limit, created_at, updated_at)
             VALUES ('{CUENTA}', '{EMPRESA}', 'Banco', 'banco@ejemplo.mx', 'smtp',
                     'llavero://banco', 'connected', 50, 10,
                     '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');

             INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
             VALUES ('{CAMPANA}', '{EMPRESA}', 'Corrida larga', 'running',
                     '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');"
        ))
    })?;

    // La cola. Una dirección distinta por fila: el `UNIQUE(campaign_id,
    // channel, contact_address)` no deja repetir, y además repetir mediría una
    // cola que el producto no puede tener.
    db.ejecutar_en_pruebas(|c| {
        let tx = c.unchecked_transaction()?;
        {
            let mut ins = tx.prepare(
                "INSERT INTO message_attempt
                     (id, campaign_id, channel, contact_address, idempotency_key,
                      state, attempt_count, created_at, updated_at)
                 VALUES (?1, ?2, 'email', ?3, ?4, 'queued', 0,
                         '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z')",
            )?;
            for i in 0..pasos {
                ins.execute(rusqlite::params![
                    format!("att-{i}"),
                    CAMPANA,
                    format!("persona{i}@ejemplo.mx"),
                    format!("msgid-{i}")
                ])?;
            }
        }
        tx.commit()
    })?;

    let mx = zona("America/Mexico_City").map_err(|e| format!("zona: {e:?}"))?;
    let ajustes = Ajustes {
        empresa: EMPRESA.parse().map_err(|e| format!("empresa: {e:?}"))?,
        campana: CAMPANA.to_owned(),
        cuenta: CUENTA.to_owned(),
        canal: arles_core::Canal::Correo,
        ventana: Ventana::nueva(Dias::LABORABLES, 9, 18, mx.clone())
            .map_err(|e| format!("ventana: {e:?}"))?,
        // Topes muy por encima de la corrida **a propósito**: con los 50
        // diarios del §48 el motor se pararía al quinto paso y esto mediría el
        // arranque, no una corrida larga. El tope es del producto, no del
        // motor, y aquí estorba.
        limites: Limites::nuevos(1_000_000, 1_000_000),
        zona_de_la_empresa: mx,
        // Sin redacción: lo que esta corrida mide es la memoria del motor a lo
        // largo de 20 000 envíos, y redactar añadiría una cadena por envío que
        // **no es del motor**. Medirla aquí haría parecer del motor un consumo
        // que es del texto, que es justo el error que esta corrida existe para
        // no cometer (ver la nota del proveedor que acumula).
        redaccion: None,
    };

    let proveedor = SiempreAcepta {
        recuerda: con_fuga,
        ..SiempreAcepta::default()
    };
    let entorno = Quieto(EN_HORARIO);
    let sin_freno = || false;
    let trabajador = Trabajador::nuevo(&db, &proveedor, &entorno, &sin_freno, ajustes);

    // Cien muestras, repartidas. Más no dicen nada nuevo y leer /proc en cada
    // paso mediría el coste de leer /proc.
    let cada = (pasos / 100).max(1);
    let mut muestras = Vec::with_capacity(128);
    let mut dados = 0;
    let mut enviados = 0;
    let t = Instant::now();
    for i in 0..pasos {
        match trabajador.un_paso()? {
            arles_motor::Paso::Aceptado { .. } => enviados += 1,
            // Cualquier otra cosa en esta corrida significa que la cola se
            // acabó o que algo frenó. Se para y se dice cuántos pasos hubo:
            // seguir giraría en vacío y las muestras del final medirían un
            // motor que no hace nada.
            otro => {
                eprintln!("la corrida se detuvo en el paso {i}: {otro:?}");
                dados = i;
                break;
            }
        }
        dados = i + 1;
        if i % cada == 0
            && let Some(r) = residente()
        {
            muestras.push(r);
        }
    }

    Ok(Corrida {
        pasos: dados,
        enviados,
        muestras,
        tardo: t.elapsed(),
    })
}
