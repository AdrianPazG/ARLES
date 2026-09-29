//! Las ventanas de ejecución: cuándo puede enviar una campaña.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! UNA VENTANA SE VALIDA AL GUARDARLA, NO AL USARLA
//!
//! Sin días marcados, o con las horas al revés, una ventana **no se abre
//! nunca**. Y eso no da error en ninguna parte: la campaña se queda activada,
//! esperando para siempre, con la pantalla diciendo «en marcha».
//!
//! Por eso aquí se construye una [`Ventana`] del núcleo antes de escribir
//! nada. El tipo sólo existe válido, así que guardar algo que no se abriría
//! exige, literalmente, saltarse esta función.
//!
//! La zona horaria es la **de la ventana**, no la del sistema (D-10): una
//! campaña dirigida a Madrid se envía en horario de Madrid desde un equipo que
//! está en México. Se resuelve contra la base IANA al guardar, para que un
//! nombre inventado se rechace ahora y no en el primer envío.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! SE REUTILIZAN ENTRE CAMPAÑAS, Y POR ESO NO SE BORRAN AQUÍ
//!
//! «Laborables de 9 a 18» se escribe una vez y la usan todas las campañas. La
//! clave foránea de `campaign_stage` es `ON DELETE RESTRICT`: una ventana en
//! uso no se puede borrar, y es lo correcto — borrarla dejaría campañas sin
//! horario que el motor no sabría interpretar.

use arles_core::ids::{CompanyId, ExecutionWindowId};
use arles_core::ventana::{Dias, Ventana, zona};
use rusqlite::params;

use crate::db::Db;
use crate::empresa::ahora;
use crate::error::DbError;

/// Una ventana tal como la enseña el asistente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VentanaGuardada {
    pub id: String,
    pub nombre: String,
    /// Máscara de bits: bit 0 = lunes … bit 6 = domingo.
    pub dias: u8,
    pub desde: u8,
    pub hasta: u8,
    /// El nombre IANA, tal como se guardó. Se devuelve como texto y no como
    /// `TimeZone` porque quien lo recibe es una pantalla, que lo enseña.
    pub zona: String,
}

impl Db {
    /// Guarda una ventana de ejecución para esta empresa.
    ///
    /// # Errores
    ///
    /// [`DbError::VentanaInvalida`] si no se abriría nunca: sin días, con las
    /// horas al revés, o con una zona que no está en la base IANA. Se devuelve
    /// el motivo para que la pantalla marque el campo que toca (§95).
    ///
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn guardar_ventana(
        &self,
        empresa: CompanyId,
        nombre: &str,
        dias: u8,
        desde: u8,
        hasta: u8,
        zona_iana: &str,
    ) -> Result<String, DbError> {
        // Se construye la del núcleo **antes** de escribir: es toda la
        // validación, y está en un solo sitio.
        let tz = zona(zona_iana).map_err(DbError::VentanaInvalida)?;
        let _: Ventana =
            Ventana::nueva(Dias(dias), desde, hasta, tz).map_err(DbError::VentanaInvalida)?;

        let id = ExecutionWindowId::nuevo().to_string();

        self.con(|conn| {
            conn.execute(
                "INSERT INTO execution_window (id, company_id, name, days_of_week,
                                               start_hour, end_hour, timezone, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    id,
                    empresa.to_string(),
                    nombre.trim(),
                    i64::from(dias),
                    i64::from(desde),
                    i64::from(hasta),
                    zona_iana,
                    ahora()
                ],
            )
        })?;

        Ok(id)
    }

    /// Las ventanas de esta empresa, para elegir una.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn ventanas(&self, empresa: CompanyId) -> Result<Vec<VentanaGuardada>, DbError> {
        self.con(|conn| {
            let mut consulta = conn.prepare(
                "SELECT id, name, days_of_week, start_hour, end_hour, timezone
                   FROM execution_window
                  WHERE company_id = ?1
                  ORDER BY created_at",
            )?;

            let filas = consulta.query_map(params![empresa.to_string()], |f| {
                Ok(VentanaGuardada {
                    id: f.get(0)?,
                    nombre: f.get(1)?,
                    // Los `CHECK` de la tabla acotan las horas a 0..=24 y los
                    // días caben en un byte por construcción. Un valor fuera de
                    // rango sólo llega escribiendo por fuera de `guardar_ventana`,
                    // y entonces se enseña un 0 en vez de tumbar la lista: la
                    // ventana se ve mal y se corrige, que es mejor que una
                    // pantalla que no abre.
                    dias: u8::try_from(f.get::<_, i64>(2)?).unwrap_or(0),
                    desde: u8::try_from(f.get::<_, i64>(3)?).unwrap_or(0),
                    hasta: u8::try_from(f.get::<_, i64>(4)?).unwrap_or(0),
                    zona: f.get(5)?,
                })
            })?;

            let mut todas = Vec::new();
            for fila in filas {
                todas.push(fila?);
            }
            Ok(todas)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arles_core::ventana::VentanaInvalida;

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

    /// Una ventana normal se guarda y se lee tal cual.
    #[test]
    fn una_ventana_se_guarda_y_se_lee() {
        let (_d, db) = base();
        db.guardar_ventana(empresa(), "Laborables", 31, 9, 18, "America/Mexico_City")
            .expect("guarda");

        let todas = db.ventanas(empresa()).expect("lee");
        let unica = todas.first().expect("la que se acaba de guardar");
        assert_eq!(todas.len(), 1);
        assert_eq!(unica.dias, 31);
        assert_eq!(unica.desde, 9);
        assert_eq!(unica.hasta, 18);
        assert_eq!(unica.zona, "America/Mexico_City");
    }

    /// **Una ventana que no se abriría nunca no se guarda.**
    ///
    /// Es el fallo que no falla: la campaña se activa, el motor la mira, no
    /// toca enviar, y vuelve a mirar dentro de cinco segundos. Para siempre,
    /// con la pantalla diciendo «en marcha».
    ///
    /// Los tres motivos van juntos porque los tres acaban igual, y cada uno se
    /// arregla en un campo distinto del formulario (§95).
    ///
    /// Verificada rompiéndola el 29/09/2026: quitando la construcción de
    /// `Ventana::nueva` y escribiendo directo, los tres casos devuelven `Ok` y
    /// esta prueba falla en el primero.
    #[test]
    fn una_ventana_que_no_se_abriria_nunca_no_se_guarda() {
        let (_d, db) = base();

        let sin_dias = db.guardar_ventana(empresa(), "Nunca", 0, 9, 18, "America/Mexico_City");
        assert!(
            matches!(
                sin_dias,
                Err(DbError::VentanaInvalida(VentanaInvalida::SinDias))
            ),
            "sin ningún día marcado no se abre nunca"
        );

        let al_reves = db.guardar_ventana(empresa(), "Al revés", 31, 18, 9, "America/Mexico_City");
        assert!(
            matches!(
                al_reves,
                Err(DbError::VentanaInvalida(
                    VentanaInvalida::HorasAlReves { .. }
                ))
            ),
            "de 18 a 9 no es un horario"
        );

        let zona_rara = db.guardar_ventana(empresa(), "Atlántida", 31, 9, 18, "America/Atlantida");
        assert!(
            matches!(
                zona_rara,
                Err(DbError::VentanaInvalida(VentanaInvalida::ZonaDesconocida))
            ),
            "una zona que no existe se rechaza ahora, no en el primer envío"
        );

        assert!(
            db.ventanas(empresa()).expect("lee").is_empty(),
            "y ninguna de las tres llegó a escribirse"
        );
    }

    /// Las ventanas son de su empresa.
    #[test]
    fn las_ventanas_son_de_su_empresa() {
        let (_d, db) = base();
        db.guardar_ventana(empresa(), "Laborables", 31, 9, 18, "America/Mexico_City")
            .expect("guarda");

        let otra: CompanyId = "22222222-2222-4222-8222-222222222222".parse().expect("id");
        assert!(db.ventanas(otra).expect("lee").is_empty());
    }
}
