//! Apertura de la base de datos cifrada.
//!
//! Ver `documentacion/04-seguridad/MODELO_DE_SECRETOS.md` y ADR-0011.

use std::path::Path;

use arles_core::Secret;
use rusqlite::Connection;

use crate::error::DbError;
use crate::migraciones;

/// Longitud de la clave maestra de SQLCipher.
///
/// 256 bits. Se pasan como blob hexadecimal para que SQLCipher los use
/// **directamente** como clave, sin derivación desde frase de paso: la clave ya
/// es aleatoria, así que derivarla no añadiría entropía y sí coste de apertura.
pub const LONGITUD_CLAVE: usize = 32;

/// La clave que cifra la base de datos.
///
/// Se genera en el primer arranque y vive en el llavero del sistema operativo.
/// **Si el llavero no está disponible, la aplicación no arranca**: no hay
/// degradación a texto plano, porque eso convertiría el cifrado en teatro
/// (ADR-0011).
pub struct ClaveMaestra(Secret<Vec<u8>>);

impl ClaveMaestra {
    /// Genera una clave nueva con el generador criptográfico del sistema.
    ///
    /// # Errores
    ///
    /// Si el sistema operativo no puede proporcionar aleatoriedad. En ese caso
    /// **no se debe continuar con un valor alternativo**: sin aleatoriedad real
    /// no hay cifrado real.
    pub fn generar() -> Result<Self, DbError> {
        let mut bytes = vec![0_u8; LONGITUD_CLAVE];
        getrandom::fill(&mut bytes).map_err(|_| {
            DbError::Migracion(
                "el sistema operativo no pudo proporcionar aleatoriedad criptográfica".into(),
            )
        })?;
        Ok(Self(Secret::new(bytes)))
    }

    /// Reconstruye la clave a partir de lo que devolvió el llavero.
    ///
    /// # Errores
    ///
    /// [`DbError::ClaveMalFormada`] si no mide exactamente [`LONGITUD_CLAVE`].
    pub fn desde_bytes(bytes: &[u8]) -> Result<Self, DbError> {
        if bytes.len() != LONGITUD_CLAVE {
            return Err(DbError::ClaveMalFormada {
                esperado: LONGITUD_CLAVE,
                recibido: bytes.len(),
            });
        }
        Ok(Self(Secret::new(bytes.to_vec())))
    }

    /// Los bytes, para guardarlos en el llavero. Único punto de salida.
    pub fn exponer(&self) -> &[u8] {
        self.0.expose_secret()
    }

    /// Literal hexadecimal para `PRAGMA key`, en la forma `x'...'`.
    ///
    /// Se escribe carácter a carácter en un búfer ya reservado. La versión con
    /// `format!("{b:02x}")` dejaba 32 `String` temporales con trozos de la clave
    /// en el montón, **sin limpiar**, justo dentro del código que se toma la
    /// molestia de limpiar todo lo demás.
    fn como_pragma(&self) -> Secret<String> {
        const NIBBLE: fn(u8) -> char = |n| match n {
            0..=9 => (b'0' + n) as char,
            _ => (b'a' + n - 10) as char,
        };

        let mut hex = String::with_capacity(LONGITUD_CLAVE * 2 + 3);
        hex.push_str("x'");
        for b in self.0.expose_secret() {
            hex.push(NIBBLE(b >> 4));
            hex.push(NIBBLE(b & 0x0F));
        }
        hex.push('\'');
        Secret::new(hex)
    }
}

impl std::fmt::Debug for ClaveMaestra {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ClaveMaestra([REDACTADO])")
    }
}

/// Abre la base de datos cifrada, aplica los PRAGMA y corre las migraciones.
///
/// # Errores
///
/// - [`DbError::ClaveIncorrecta`] si la clave no descifra el archivo. Puede
///   significar que la clave del llavero no corresponde a esta base, o que el
///   archivo está corrupto. **En ninguno de los dos casos se ofrece continuar
///   sin cifrado.**
/// - [`DbError::Migracion`] si el esquema no se puede poner al día.
pub fn abrir(ruta: &Path, clave: &ClaveMaestra) -> Result<Connection, DbError> {
    let mut conn = Connection::open(ruta)?;

    // `PRAGMA key` va ANTES que cualquier otra sentencia. Si se ejecuta algo
    // previo, SQLCipher falla con un error que no es evidente.
    //
    // El literal hexadecimal es tan sensible como la clave, así que vive en su
    // propio ámbito: `Secret` se limpia al soltarse, y así no sigue en memoria
    // durante las migraciones.
    {
        let pragma = clave.como_pragma();
        conn.pragma_update(None, "key", pragma.expose_secret().as_str())?;
    }

    verificar_clave(&conn)?;

    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    // NORMAL basta con WAL: no se pierde una transacción confirmada, solo las
    // más recientes ante un corte de corriente del sistema entero.
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;

    migraciones::aplicar(&mut conn)?;

    Ok(conn)
}

/// SQLCipher no falla al fijar la clave: falla al tocar la base.
///
/// Se distingue «la clave no descifra» de «el disco falló». Antes todo error se
/// reportaba como clave incorrecta, así que un disco lleno o un permiso denegado
/// mandaban al usuario a buscar un problema de credenciales que no existía — y
/// el §95 exige decirle qué pasó de verdad y cómo arreglarlo.
fn verificar_clave(conn: &Connection) -> Result<(), DbError> {
    match conn.query_row("SELECT count(*) FROM sqlite_schema", [], |f| {
        f.get::<_, i64>(0)
    }) {
        Ok(_) => Ok(()),

        // Lo que devuelve SQLCipher cuando los datos no se descifran: el
        // archivo deja de parecer una base de datos.
        Err(rusqlite::Error::SqliteFailure(e, _))
            if e.code == rusqlite::ErrorCode::NotADatabase =>
        {
            Err(DbError::ClaveIncorrecta)
        }

        Err(otro) => Err(DbError::Sqlite(otro)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ruta_temporal() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().expect("directorio temporal");
        let ruta = dir.path().join("arles.db");
        (dir, ruta)
    }

    #[test]
    fn la_clave_tiene_256_bits() {
        let c = ClaveMaestra::generar().expect("genera");
        assert_eq!(c.exponer().len(), 32);
    }

    #[test]
    fn dos_claves_generadas_son_distintas() {
        let a = ClaveMaestra::generar().expect("genera");
        let b = ClaveMaestra::generar().expect("genera");
        assert_ne!(a.exponer(), b.exponer());
    }

    #[test]
    fn rechaza_una_clave_de_longitud_equivocada() {
        assert!(matches!(
            ClaveMaestra::desde_bytes(&[0_u8; 16]),
            Err(DbError::ClaveMalFormada {
                esperado: 32,
                recibido: 16
            })
        ));
    }

    #[test]
    fn la_clave_no_aparece_al_formatearla() {
        let c = ClaveMaestra::generar().expect("genera");
        assert_eq!(format!("{c:?}"), "ClaveMaestra([REDACTADO])");
    }

    /// T-3: el requisito no es «usar SQLCipher», es que los datos personales no
    /// estén en claro en disco. Esto lo comprueba de verdad.
    #[test]
    fn el_archivo_en_disco_esta_cifrado() {
        let (_dir, ruta) = ruta_temporal();
        let clave = ClaveMaestra::generar().expect("genera");

        {
            let conn = abrir(&ruta, &clave).expect("abre");
            conn.execute(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    "01900000-0000-7000-8000-000000000001",
                    "CADENA_BUSCABLE_EN_DISCO",
                    "MX",
                    "America/Mexico_City",
                    "hola@empresa.com",
                    "2026-09-11T00:00:00Z",
                    "2026-09-11T00:00:00Z",
                ],
            )
            .expect("inserta");
        }

        let bytes = std::fs::read(&ruta).expect("lee el archivo");
        let aguja = b"CADENA_BUSCABLE_EN_DISCO";
        assert!(
            !bytes.windows(aguja.len()).any(|w| w == aguja),
            "los datos aparecen en claro: SQLCipher no está cifrando"
        );
        assert!(
            !bytes.starts_with(b"SQLite format 3"),
            "la cabecera está sin cifrar"
        );
    }

    #[test]
    fn no_abre_con_una_clave_equivocada() {
        let (_dir, ruta) = ruta_temporal();
        let buena = ClaveMaestra::generar().expect("genera");
        drop(abrir(&ruta, &buena).expect("abre"));

        let mala = ClaveMaestra::generar().expect("genera");
        assert!(matches!(abrir(&ruta, &mala), Err(DbError::ClaveIncorrecta)));
    }

    #[test]
    fn reabre_con_la_misma_clave() {
        let (_dir, ruta) = ruta_temporal();
        let clave = ClaveMaestra::generar().expect("genera");
        drop(abrir(&ruta, &clave).expect("primera apertura"));

        let misma = ClaveMaestra::desde_bytes(clave.exponer()).expect("reconstruye");
        assert!(abrir(&ruta, &misma).is_ok());
    }

    #[test]
    fn los_pragma_quedan_aplicados() {
        let (_dir, ruta) = ruta_temporal();
        let clave = ClaveMaestra::generar().expect("genera");
        let conn = abrir(&ruta, &clave).expect("abre");

        let fk: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |f| f.get(0))
            .expect("lee foreign_keys");
        assert_eq!(fk, 1, "las claves foráneas deben estar activas");

        let modo: String = conn
            .query_row("PRAGMA journal_mode", [], |f| f.get(0))
            .expect("lee journal_mode");
        assert_eq!(modo.to_lowercase(), "wal");
    }
}
