//! Cuánta gente contestó a una etapa de WhatsApp.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! AQUÍ NO SE GUARDA LO QUE DIJO NADIE
//!
//! La decisión de Dirección del 29/09/2026 es contar las respuestas, no
//! guardarlas. El texto se descarta en `arles_whatsapp::entrante`, que es donde
//! se lee; esto sólo recibe quién y cuándo.
//!
//! La tabla no tiene columna donde quepa un mensaje, y hay una prueba de
//! invariantes que vigila su forma: añadir una columna para el contenido es una
//! decisión de privacidad, no un detalle de implementación.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTA MÉTRICA IMPORTA MÁS QUE LAS OTRAS
//!
//! Meta premia que te contesten. La proporción de respuestas es lo que más
//! protege a un número —más que el volumen, más que el ritmo—, y su caída
//! precede a que el número se venga abajo.
//!
//! Por eso se cuenta **gente** y no mensajes: quien conteste cinco veces es una
//! persona a la que le interesó, no cinco. El `UNIQUE` del esquema lo impone.

use arles_core::ids::{CompanyId, WhatsappReplyId};
use rusqlite::params;

use crate::db::Db;
use crate::error::DbError;

impl Db {
    /// Apunta que alguien contestó a una etapa.
    ///
    /// **Es idempotente**: la misma persona en la misma etapa no se cuenta dos
    /// veces, y volver a recibir el mismo aviso —que Meta hace— no cambia nada.
    /// Lo impone el `UNIQUE` del esquema y no el código: los avisos llegan
    /// repetidos, y confiar en que el código los deduplique es confiar en que
    /// nadie añada otro camino de escritura.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla por algo que no sea el choque
    /// de unicidad, que es el caso normal y no es un error.
    pub fn apuntar_respuesta(
        &self,
        empresa: CompanyId,
        etapa: &str,
        de_normalizado: &str,
        cuando: &str,
    ) -> Result<(), DbError> {
        self.con(|conn| {
            conn.execute(
                "INSERT OR IGNORE INTO whatsapp_reply
                     (id, company_id, campaign_stage_id, from_normalized, first_reply_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    WhatsappReplyId::nuevo().to_string(),
                    empresa.to_string(),
                    etapa,
                    de_normalizado,
                    cuando
                ],
            )
        })?;
        Ok(())
    }

    /// A cuánta **gente** le interesó.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn cuantos_contestaron(&self, etapa: &str) -> Result<u32, DbError> {
        let n: i64 = self.con(|conn| {
            conn.query_row(
                "SELECT count(*) FROM whatsapp_reply WHERE campaign_stage_id = ?1",
                params![etapa],
                |f| f.get(0),
            )
        })?;
        Ok(u32::try_from(n).unwrap_or(u32::MAX))
    }

    /// Borra las respuestas de un número, para el derecho de cancelación.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// ESTO EXISTE POR LA LFPDPPP, Y HAY QUE PODER EJERCERLO
    ///
    /// El número de quien contestó es el único dato personal de esta tabla.
    /// Quien ejerza su derecho de cancelación tiene que poder desaparecer de
    /// aquí, y una métrica agregada no es excusa para conservarlo: lo que
    /// protege al número es la **cifra**, y la cifra sobrevive perfectamente a
    /// que se borre una fila.
    ///
    /// Es el mismo problema que la V3 documentó con `consent_entry`: registrar
    /// algo sobre alguien no puede impedirle ejercer sus derechos.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si el borrado falla.
    pub fn olvidar_respuestas_de(
        &self,
        empresa: CompanyId,
        de_normalizado: &str,
    ) -> Result<u32, DbError> {
        let borradas = self.con(|conn| {
            conn.execute(
                "DELETE FROM whatsapp_reply
                  WHERE company_id = ?1 AND from_normalized = ?2",
                params![empresa.to_string(), de_normalizado],
            )
        })?;
        Ok(u32::try_from(borradas).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conexion::ClaveMaestra;

    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
    const AHORA: &str = "2026-09-29T00:00:00Z";
    const ETAPA: &str = "et-wa-1";

    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");

        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('{EMPRESA}', 'TELEMETRY', 'MX', 'America/Mexico_City',
                         'hola@t.mx', '{AHORA}', '{AHORA}');

                 INSERT INTO whatsapp_account (id, company_id, display_name, phone_e164,
                                               waba_id, phone_number_id, credential_ref,
                                               modo, status, created_at, updated_at)
                 VALUES ('wa-1', '{EMPRESA}', 'Ventas', '+528112345678', 'w', 'p',
                         'llavero://w', 'seguimiento', 'connected', '{AHORA}', '{AHORA}');

                 INSERT INTO execution_window (id, company_id, name, days_of_week,
                                               start_hour, end_hour, timezone, created_at)
                 VALUES ('v-1', '{EMPRESA}', 'Laborables', 31, 9, 18,
                         'America/Mexico_City', '{AHORA}');

                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES ('k-1', '{EMPRESA}', 'Con WhatsApp', 'draft', '{AHORA}', '{AHORA}');

                 INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             whatsapp_account_id, execution_window_id,
                                             status, created_at, updated_at)
                 VALUES ('{ETAPA}', 'k-1', 1, 'whatsapp', 'wa-1', 'v-1', 'draft',
                         '{AHORA}', '{AHORA}');"
            ))
        })
        .expect("siembra");
        (dir, db)
    }

    fn empresa() -> CompanyId {
        EMPRESA.parse().expect("id")
    }

    /// **Quien contesta cinco veces cuenta una, y no da error.**
    ///
    /// Meta reenvía avisos, y una conversación son varios mensajes. Si esto
    /// devolviera error al repetir, cada aviso repetido se vería como un fallo
    /// en la pantalla — y no lo es: es el protocolo funcionando.
    ///
    /// Verificada rompiéndola el 29/09/2026: cambiando `INSERT OR IGNORE` por
    /// `INSERT`, la segunda llamada devuelve error y esta prueba falla.
    #[test]
    fn contestar_varias_veces_cuenta_una_y_no_falla() {
        let (_d, db) = base();

        for _ in 0..5 {
            db.apuntar_respuesta(empresa(), ETAPA, "5218112345678", AHORA)
                .expect("un aviso repetido no es un error");
        }
        db.apuntar_respuesta(empresa(), ETAPA, "5218199999999", AHORA)
            .expect("otra persona");

        assert_eq!(db.cuantos_contestaron(ETAPA).expect("cuenta"), 2);
    }

    /// **Quien pide que le olviden desaparece, y la cifra sigue teniendo
    /// sentido.**
    ///
    /// El número es el único dato personal de esta tabla, y una métrica
    /// agregada no es excusa para conservarlo (LFPDPPP). Es el mismo problema
    /// que la V3 documentó con `consent_entry`: registrar algo sobre alguien no
    /// puede impedirle ejercer sus derechos.
    #[test]
    fn se_puede_olvidar_a_quien_lo_pide() {
        let (_d, db) = base();
        db.apuntar_respuesta(empresa(), ETAPA, "5218112345678", AHORA)
            .expect("apunta");
        db.apuntar_respuesta(empresa(), ETAPA, "5218199999999", AHORA)
            .expect("apunta");

        let borradas = db
            .olvidar_respuestas_de(empresa(), "5218112345678")
            .expect("olvida");

        assert_eq!(borradas, 1);
        assert_eq!(
            db.cuantos_contestaron(ETAPA).expect("cuenta"),
            1,
            "la cifra baja, y sigue siendo cierta"
        );
    }

    /// Las respuestas de otra empresa no se borran ni se cuentan por error.
    #[test]
    fn olvidar_no_toca_a_otra_empresa() {
        let (_d, db) = base();
        db.apuntar_respuesta(empresa(), ETAPA, "5218112345678", AHORA)
            .expect("apunta");

        let otra: CompanyId = "22222222-2222-4222-8222-222222222222".parse().expect("id");
        assert_eq!(
            db.olvidar_respuestas_de(otra, "5218112345678")
                .expect("olvida"),
            0
        );
        assert_eq!(db.cuantos_contestaron(ETAPA).expect("cuenta"), 1);
    }
}
