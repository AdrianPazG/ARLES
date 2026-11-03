//! El asa de la base de datos.
//!
//! Encapsula la conexión para que **`rusqlite` no salga de este crate**. Los
//! crates de dominio y el shell hablan con métodos tipados, no con SQL ni con
//! tipos del motor: así cambiar el motor —o simplemente auditar qué consultas
//! existen— es un trabajo acotado a `arles-db`.

use std::path::Path;
use std::sync::{Mutex, PoisonError};

use rusqlite::Connection;

use crate::conexion::{self, ClaveMaestra};
use crate::error::DbError;

/// Conexión a la base de datos cifrada, lista para usarse desde varios hilos.
///
/// **Un solo escritor.** SQLite con WAL admite lectores concurrentes pero un
/// único escritor; serializar aquí **elimina de raíz** la clase de errores
/// `SQLITE_BUSY` en vez de gestionarlos con reintentos que funcionan el 99 % de
/// las veces (ADR-0002).
#[derive(Debug)]
pub struct Db {
    conexion: Mutex<Connection>,

    /// La segunda conexión, **de sólo lectura** (ADR-0015).
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ TIENE SU PROPIO `Mutex` HABIENDO SÓLO UN LECTOR
    ///
    /// Porque `Connection` no es `Sync`: hace falta un cerrojo para
    /// compartirla entre hilos, aunque nadie compita. Y aunque compitiera, lo
    /// que esperaría es a **otra lectura**, que son milisegundos — no al
    /// escritor, que es lo que esta conexión viene a evitar: 91.4 s de una
    /// importación o 73.5 s de una activación.
    ///
    /// ── Y POR QUÉ ES `Option` ────────────────────────────────────────────
    ///
    /// Para que **no poder abrirla no impida arrancar**. Si la segunda
    /// conexión falla —un descriptor de archivo de menos, un permiso raro—, lo
    /// razonable es que ARLES funcione como funcionaba hasta hoy: con un solo
    /// cerrojo y las pantallas esperando. Negar el arranque por no poder
    /// *mejorar* el rendimiento sería cambiar una molestia por una avería.
    ///
    /// `None` significa exactamente eso: [`Db::leer`] cae al escritor.
    lectora: Option<Mutex<Connection>>,
}

/// Qué se sabe de la base al arrancar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumenArranque {
    /// Versión del esquema aplicada por las migraciones.
    pub version_esquema: Option<i32>,
    /// Cuántas empresas hay configuradas. Cero significa que toca el
    /// onboarding del §25.
    pub empresas: i64,
}

impl Db {
    /// Abre la base cifrada, aplica los PRAGMA y corre las migraciones.
    ///
    /// # Errores
    ///
    /// Ver [`crate::conexion::abrir`].
    pub fn abrir(ruta: &Path, clave: &ClaveMaestra) -> Result<Self, DbError> {
        let escritora = conexion::abrir(ruta, clave)?;

        // La lectora se abre **después** del escritor, y por eso no migra: el
        // esquema ya está al día. Y con la misma clave, que se suelta igual que
        // antes — ADR-0015 lo pone como condición: cero accesos nuevos al
        // llavero y cero aumento de cuánto vive la clave en memoria.
        //
        // Si no se puede abrir, se sigue sin ella (ver el campo `lectora`).
        let lectora = conexion::abrir_para_leer(ruta, clave).ok().map(Mutex::new);

        Ok(Self {
            conexion: Mutex::new(escritora),
            lectora,
        })
    }

    /// Toma la conexión.
    ///
    /// Es `pub(crate)` para que los repositorios de este crate la usen; **no
    /// sale de `arles-db`**, que es lo que mantiene `rusqlite` dentro (ADR-0002).
    ///
    /// Un mutex envenenado significa que otro hilo entró en pánico teniéndola
    /// tomada. Se recupera el guardia en vez de propagar el pánico: la conexión
    /// sigue siendo válida y tumbar la aplicación sería peor para el usuario.
    pub(crate) fn con<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    ) -> Result<T, DbError> {
        let guardia = self.conexion.lock().unwrap_or_else(PoisonError::into_inner);
        f(&guardia).map_err(DbError::Sqlite)
    }

    /// Lee por la **conexión de pantallas** (ADR-0015).
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// QUIÉN PUEDE USAR ESTO, Y QUIÉN NO
    ///
    /// **Las pantallas. El motor no.** Sus cinco lecturas se quedan en el
    /// escritor, y la primera —`esta_suprimida`— es la que decide si un mensaje
    /// sale: por aquí se perdería para siempre poder meterla con la escritura
    /// del intento en una misma transacción, y leer lo que acabas de escribir.
    ///
    /// Lo vigila `el_motor_no_lee_por_la_conexion_de_pantallas` en `--fase 1`,
    /// verificada en rojo con `esta_suprimida` pasada por aquí.
    ///
    /// ── UNA ESCRITURA POR AQUÍ NO FALLA A VECES: FALLA SIEMPRE ───────────
    ///
    /// La conexión lleva `PRAGMA query_only`, así que el motor de base la
    /// rechaza con `ReadOnly`. Medido contra SQLCipher el 28/10/2026. No hay
    /// que acordarse de nada.
    ///
    /// ── LO QUE SE VE POR AQUÍ ES UNA FOTO, Y ESO ES CORRECTO ─────────────
    ///
    /// Si el escritor tiene una transacción abierta, esta conexión ve **lo
    /// anterior a esa transacción** — medido: `Ok(0)` con la transacción
    /// abierta y `Ok(1)` tras el `commit`. Es WAL funcionando, y es lo que se
    /// quiere: una pantalla consistente en vez de una pantalla esperando. Pero
    /// significa que durante una importación o una activación largas, lo que se
    /// enseña está **vivo, no al día**.
    ///
    /// Cada llamada abre y cierra su propia transacción de lectura implícita,
    /// que es lo que mantiene el `-wal` acotado: un lector que retiene la suya
    /// lleva el `-wal` de 4.0 a 373.3 MB con 20 000 escrituras pequeñas
    /// (ADR-0015, incógnita 1).
    ///
    /// Si no hay segunda conexión, cae al escritor y todo sigue como antes.
    pub(crate) fn leer<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    ) -> Result<T, DbError> {
        match &self.lectora {
            Some(m) => {
                let guardia = m.lock().unwrap_or_else(PoisonError::into_inner);
                f(&guardia).map_err(DbError::Sqlite)
            }
            None => self.con(f),
        }
    }

    /// Igual que [`Db::leer`], pero el cierre puede fallar con un error **del
    /// dominio**.
    ///
    /// Existe por la misma razón que [`Db::con_dominio`] frente a [`Db::con`]:
    /// sólo cambia el tipo de error. **Las dos parejas tienen que existir**, o
    /// mover un método a la lectora dependería de con qué error falla, que no
    /// tiene nada que ver con si lee o escribe — y entonces `listar_contactos`,
    /// que es la pantalla que más pesa (357 ms, medido el 06/10/2026), no se
    /// podría mover.
    pub(crate) fn leer_dominio<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, DbError>,
    ) -> Result<T, DbError> {
        match &self.lectora {
            Some(m) => {
                let guardia = m.lock().unwrap_or_else(PoisonError::into_inner);
                f(&guardia)
            }
            None => self.con_dominio(f),
        }
    }

    /// Igual que [`Db::con`], pero el cierre puede fallar con un error **del
    /// dominio**, no sólo de SQLite.
    ///
    /// Hace falta cuando la decisión de abortar se toma **dentro** de la
    /// transacción: al escribir los canales de un contacto, la segunda
    /// dirección puede chocar con la de otro contacto, y entonces la primera no
    /// debe quedar escrita. Convertir el error fuera del cierre —como hace
    /// `guardar_empresa`— llegaría tarde: la transacción ya habría confirmado.
    pub(crate) fn con_dominio<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, DbError>,
    ) -> Result<T, DbError> {
        let guardia = self.conexion.lock().unwrap_or_else(PoisonError::into_inner);
        f(&guardia)
    }

    /// Estado de la base al arrancar.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn resumen_arranque(&self) -> Result<ResumenArranque, DbError> {
        self.leer(|conn| {
            let empresas: i64 =
                conn.query_row("SELECT count(*) FROM company", [], |f| f.get::<_, i64>(0))?;

            // `refinery` crea esta tabla; si no existe, la consulta falla y
            // devolvemos `None` en vez de tumbar el arranque.
            let version_esquema = conn
                .query_row(
                    "SELECT max(version) FROM refinery_schema_history",
                    [],
                    |f| f.get::<_, Option<i32>>(0),
                )
                .ok()
                .flatten();

            Ok(ResumenArranque {
                version_esquema,
                empresas,
            })
        })
    }

    /// Ejecuta SQL arbitrario. **Solo para tests**: el código de producción usa
    /// métodos tipados para que las consultas sean auditables (ADR-0002).
    #[cfg(any(test, feature = "test-util"))]
    pub fn ejecutar_en_pruebas<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    ) -> Result<T, DbError> {
        self.con(f)
    }

    /// Lo mismo, pero **por la conexión de lectura**.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ HACE FALTA, Y NO ES SIMETRÍA
    ///
    /// El 03/11/2026, midiendo el filtro de contactos, el camino entregado
    /// tardó **7 010 ms** y la suma de sus dos consultas, medidas con
    /// `ejecutar_en_pruebas`, **656**. Diez veces. Lo que las distingue no es
    /// el SQL —se comprobó quitando el `ESCAPE` y midiendo el recuento
    /// aparte— sino **por qué conexión pasan**: una va por el escritor y la
    /// otra por la lectora de ADR-0015.
    ///
    /// Sin este método, esa diferencia no se puede medir, y lo que se publica
    /// de cualquier pantalla sería la cifra de una conexión que la pantalla no
    /// usa. **Un instrumento que mide otro camino del que recorre el usuario
    /// no mide**: es la lección de la 8.5, cuando el banco medía un camino que
    /// nadie pisa.
    #[cfg(any(test, feature = "test-util"))]
    pub fn leer_en_pruebas<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    ) -> Result<T, DbError> {
        self.leer(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        (dir, db)
    }

    #[test]
    fn una_base_nueva_no_tiene_empresa_configurada() {
        let (_d, db) = base();
        let r = db.resumen_arranque().expect("resumen");
        assert_eq!(
            r.empresas, 0,
            "una instalación nueva debe llevar al onboarding"
        );
    }

    #[test]
    fn el_resumen_reporta_la_version_del_esquema() {
        let (_d, db) = base();
        let r = db.resumen_arranque().expect("resumen");
        assert_eq!(
            r.version_esquema,
            crate::migraciones::ultima_version(),
            "las migraciones deberían haber dejado el esquema en la última versión"
        );
    }

    #[test]
    fn el_resumen_ve_la_empresa_una_vez_creada() {
        let (_d, db) = base();
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('c1', 'TELEMETRY', 'MX', 'America/Mexico_City',
                         'hola@t.mx', '2026-09-11T00:00:00Z', '2026-09-11T00:00:00Z')",
                [],
            )
        })
        .expect("inserta");

        assert_eq!(db.resumen_arranque().expect("resumen").empresas, 1);
    }

    /// El asa debe poder compartirse entre hilos: el motor de ejecución correrá
    /// en su propio conjunto de tareas, independiente de la ventana (§51).
    #[test]
    fn el_asa_es_compartible_entre_hilos() {
        fn exige_sync_send<T: Sync + Send>() {}
        exige_sync_send::<Db>();
    }
    // ── LA SEGUNDA CONEXIÓN DE SÓLO LECTURA · ADR-0015 ──────────────────

    /// **Una escritura por la lectora no falla a veces: falla siempre.**
    ///
    /// Es la garantía que convierte «no escribas por aquí» en algo que el motor
    /// de base rechaza, en vez de algo que haya que recordar — el orden que
    /// pide el §1 bis: gana el tipo sobre el validador.
    ///
    /// Y se prueba contra **SQLCipher de verdad**, que es lo que había que
    /// comprobar antes de decidir: `query_only` es un pragma estándar, pero
    /// darlo por bueno sobre una base cifrada sin correrlo es la clase de
    /// supuesto que este proyecto exige romper.
    #[test]
    fn la_lectora_rechaza_una_escritura() {
        let (_d, db) = base();
        let r = db.leer(|conn| {
            conn.execute(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('x', 'X', 'MX', 'America/Mexico_City', 'x@x.mx',
                         '2026-10-28T00:00:00Z', '2026-10-28T00:00:00Z')",
                [],
            )
        });
        assert!(
            matches!(
                r,
                Err(DbError::Sqlite(rusqlite::Error::SqliteFailure(e, _)))
                    if e.code == rusqlite::ErrorCode::ReadOnly
            ),
            "tenía que dar ReadOnly y dio {r:?}"
        );
    }

    /// Y lee lo que hay, que es la otra mitad: una conexión que no deja
    /// escribir y tampoco lee no serviría de nada.
    #[test]
    fn la_lectora_lee() {
        let (_d, db) = base();
        let n: i64 = db
            .leer(|conn| conn.query_row("SELECT count(*) FROM company", [], |f| f.get(0)))
            .expect("lee");
        assert_eq!(n, 0);
    }

    /// **Y una lectura por la conexión de ESCRITURA espera lo que dure.**
    ///
    /// ───────────────────────────────────────────────────────────────────────
    /// LA MEDICIÓN QUE DICE CUÁNTO FALTA DE ADR-0015
    ///
    /// La prueba de abajo demuestra que la conexión de lectura no espera. Ésta
    /// mide la otra mitad, que es la que decide el trabajo que queda: **lo que
    /// pasa con las lecturas que todavía no se han movido.**
    ///
    /// Al 03/11/2026 son **4 por la conexión de lectura y 104 por la de
    /// escritura**, así que durante una activación de ~75 s la mayoría de las
    /// pantallas siguen esperando el cerrojo. Dos hilos, porque el `Mutex` no
    /// es reentrante y medirlo en uno solo sería colgarse.
    ///
    /// **Importa para no construir lo que no arregla nada.** La auditoría C
    /// recomendó «sacar la activación al hilo del motor», y lo que pedía con
    /// eso —que la ventana conteste, dé avance y deje cancelar— se cerró el
    /// 28/10. Mover el trabajo de hilo **no suelta este cerrojo**: lo tomaría
    /// el hilo del motor en vez del del comando, y estas 104 lecturas
    /// esperarían exactamente lo mismo.
    #[test]
    fn una_lectura_por_la_de_escritura_espera_a_que_acabe_la_escritura() {
        use std::sync::Arc;
        use std::time::{Duration, Instant};

        let (_d, db) = base();
        assert!(db.lectora.is_some(), "sin lectora esto no compara nada");
        let db = Arc::new(db);

        // Una escritura larga en otro hilo, como la activación de una campaña.
        let escritor = {
            let db = Arc::clone(&db);
            std::thread::spawn(move || {
                let guardia = db.conexion.lock().expect("cerrojo");
                guardia.execute_batch("BEGIN IMMEDIATE").expect("abre");
                std::thread::sleep(Duration::from_millis(800));
                guardia.execute_batch("COMMIT").expect("confirma");
            })
        };

        // Se le da tiempo a tomar el cerrojo. Sin esta espera la medición
        // podría hacerse antes de que empiece la escritura, y entonces las dos
        // cifras saldrían iguales **sobre código correcto**.
        std::thread::sleep(Duration::from_millis(150));

        let t = Instant::now();
        let por_la_de_lectura: i64 = db
            .leer(|c| c.query_row("SELECT count(*) FROM company", [], |f| f.get(0)))
            .expect("lee");
        let espera_de_la_lectora = t.elapsed();

        let t = Instant::now();
        let por_la_de_escritura: i64 = db
            .con(|c| c.query_row("SELECT count(*) FROM company", [], |f| f.get(0)))
            .expect("lee");
        let espera_del_escritor = t.elapsed();

        escritor.join().expect("el escritor termina");

        assert_eq!(
            por_la_de_lectura, por_la_de_escritura,
            "las dos leen lo mismo"
        );

        // Los umbrales son holgados a propósito: lo que se afirma es un orden
        // de magnitud, no un reloj. En una máquina cargada la lectora podría
        // tardar algunas decenas de ms; lo que no puede es tardar lo que dura
        // la escritura.
        assert!(
            espera_de_la_lectora < Duration::from_millis(200),
            "la conexión de lectura esperó {} ms: ADR-0015 no está sirviendo",
            espera_de_la_lectora.as_millis()
        );
        assert!(
            espera_del_escritor > Duration::from_millis(400),
            "la lectura por la conexión de escritura NO esperó ({} ms), así que              esta prueba no está midiendo lo que dice: o la escritura no tomó              el cerrojo, o `con` ha dejado de usarlo",
            espera_del_escritor.as_millis()
        );

        println!(
            "\nMIENTRAS UNA ESCRITURA DE 800 ms ESTA EN CURSO:\n  \
             por la conexion de LECTURA:   {} ms\n  \
             por la conexion de ESCRITURA: {} ms\n",
            espera_de_la_lectora.as_millis(),
            espera_del_escritor.as_millis(),
        );
    }

    /// **Lo que se lee con una transacción de escritura abierta es la foto
    /// anterior, y eso es lo correcto.**
    ///
    /// Es la consecuencia declarada de ADR-0015 y la razón de que valga la
    /// pena: durante una importación de 91 s o una activación de 73 s, la
    /// pantalla enseña datos **consistentes** en vez de quedarse esperando el
    /// cerrojo. Lo que no hace es enseñarlos al día, y eso hay que decirlo
    /// donde se note.
    ///
    /// Esta prueba es la que hace que esa frase no sea una suposición.
    #[test]
    fn la_lectora_no_espera_al_escritor_y_ve_la_foto_anterior() {
        let (_d, db) = base();

        // ── ESTO SE AFIRMA ANTES DE TOMAR EL CERROJO, Y NO ES CEREMONIA ──
        //
        // Sin esta línea, la prueba **se cuelga** en vez de fallar cuando la
        // lectora no existe: `leer` cae al escritor y se bloquea contra el
        // cerrojo que este mismo hilo tiene tomado — la no reentrancia que
        // `CLAUDE.md` documenta como trampa de la Fase 6.
        //
        // Lo encontró romperla a propósito: la ruptura de dejar `lectora: None`
        // no salió en rojo, se quedó colgada. **Una prueba que se cuelga no es
        // una prueba en rojo**: no dice qué pasó, y en CI parece una máquina
        // lenta. Así que la condición de la que depende se afirma primero.
        assert!(
            db.lectora.is_some(),
            "sin segunda conexión esta prueba no mide nada, y además se colgaría"
        );

        // Una transacción de escritura abierta y SIN confirmar.
        let guardia = db.conexion.lock().expect("cerrojo");
        guardia.execute_batch("BEGIN").expect("abre");
        guardia
            .execute(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('c1', 'TELEMETRY', 'MX', 'America/Mexico_City',
                         'x@x.mx', '2026-10-28T00:00:00Z', '2026-10-28T00:00:00Z')",
                [],
            )
            .expect("mete");

        // La lectora **no espera**: devuelve, y devuelve lo de antes.
        //
        // Si cayera al escritor —el caso de `lectora: None`— esto se
        // bloquearía contra el `Mutex` que este mismo hilo tiene tomado y la
        // prueba se colgaría. Que termine es parte de lo que comprueba.
        let durante: i64 = db
            .leer(|conn| conn.query_row("SELECT count(*) FROM company", [], |f| f.get(0)))
            .expect("lee durante");
        assert_eq!(
            durante, 0,
            "tiene que ver la foto anterior a la transacción"
        );

        guardia.execute_batch("COMMIT").expect("confirma");
        drop(guardia);

        let despues: i64 = db
            .leer(|conn| conn.query_row("SELECT count(*) FROM company", [], |f| f.get(0)))
            .expect("lee después");
        assert_eq!(despues, 1, "y lo nuevo en cuanto confirma");
    }

    /// El `-wal` se recorta, que es la condición que la medición añadió.
    ///
    /// Sin `journal_size_limit`, SQLite reutiliza el archivo y se queda en su
    /// pico para siempre: medido el 28/10/2026, 716.3 MB después de importar
    /// 500 000 contactos **y con la base ya volcada**.
    #[test]
    fn el_wal_tiene_tope() {
        let (_d, db) = base();
        let tope: i64 = db
            .con(|conn| conn.query_row("PRAGMA journal_size_limit", [], |f| f.get(0)))
            .expect("lee el tope");
        assert_eq!(
            tope,
            64 * 1024 * 1024,
            "64 MiB, ver `conexion::abrir_crudo`"
        );
    }
}
