//! El freno: pausar, reanudar, detener y parar todo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTE ARCHIVO SE ESCRIBE EN LA FASE 7 Y NO EN LA 4
//!
//! Porque en la 4 se olvidó, y nadie se enteró durante seis entregas.
//!
//! La Fase 4 construyó las cuatro acciones enteras: `AccionDeCampana` en el
//! núcleo con sus reglas de transición, `aplicar_a_la_campana` en `arles-db`
//! con su transacción, la parada de emergencia persistida en `engine_control`,
//! y sus pruebas —incluida la de que la parada sobrevive a cerrar el programa—.
//! La Fase 6 escribió los textos en español: `campana.pausar`,
//! `campana.detener`, `campana.pausarExplicacion`.
//!
//! Y **ningún comando los llamaba**. Una campaña activada no se podía parar
//! desde la aplicación: sólo abriendo la base de datos a mano.
//!
//! Es el séptimo caso del patrón que `CLAUDE.md` §9 lista. Los seis anteriores
//! dejaban una función sin usar; éste dejaba al usuario **sin freno** sobre un
//! proceso que escribe a personas reales. Lo caza ahora
//! `toda_capacidad_del_nucleo_tiene_puerta()` en la fase 7 del validador,
//! escrita antes que este archivo para verla marcarlo en rojo (§2).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! DOS ACCIONES QUE SE PARECEN Y NO SON LA MISMA (D-7B)
//!
//! Con tres campañas a la vez —que es el escenario de Dirección— confundirlas
//! es lo que hace daño:
//!
//! | | Detener una campaña | Parar todo |
//! |---|---|---|
//! | Afecta a | Esa | Todas |
//! | Se deshace | **No** | **Sí** |
//! | Quién lo pulsa | Alguien que decidió | Alguien asustado |
//!
//! Por eso son comandos distintos con nombres que no se confunden, y por eso
//! `detener_campana` exige el nombre de la campaña: no como ceremonia, sino
//! porque con tres en marcha el riesgo real es **detener la equivocada**.
//!
//! La parada de emergencia, en cambio, **no pide confirmación**. Es
//! reversible, y una emergencia con un diálogo de por medio es una emergencia
//! peor. Lo que sí hace es dejar constancia de quién y cuándo.

use serde::Serialize;

use crate::error::ErrorIpc;
use crate::estado::EstadoApp;

/// Cómo queda una campaña tras pausarla, reanudarla o detenerla.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccionAplicadaIpc {
    /// La clave de texto del estado en que queda (§139). **No el estado
    /// crudo**: la pantalla no traduce valores de la base, traduce claves.
    pub estado: String,
    /// Cuántos envíos se cancelaron. Cero salvo al detener.
    pub cancelados: u32,
    /// Cuántas etapas acompañaron a la campaña.
    pub etapas: u32,
}

/// Si hay algo parado, y desde cuándo.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDelEnvioIpc {
    /// `true` si la parada de emergencia está puesta.
    pub parado: bool,
    /// Desde cuándo, si lo está.
    pub desde: Option<String>,
    /// Quién la puso, si consta.
    pub por: Option<String>,
    /// La clave de texto para la pantalla (§139).
    pub clave: String,
}

impl From<arles_core::campana::ParadaDeEmergencia> for EstadoDelEnvioIpc {
    fn from(p: arles_core::campana::ParadaDeEmergencia) -> Self {
        let clave = p.clave_i18n().to_string();
        match p {
            arles_core::campana::ParadaDeEmergencia::Levantada => Self {
                parado: false,
                desde: None,
                por: None,
                clave,
            },
            arles_core::campana::ParadaDeEmergencia::Puesta { desde, por } => Self {
                parado: true,
                desde: Some(desde),
                por,
                clave,
            },
        }
    }
}

/// Aplica una acción a una campaña y devuelve cómo queda.
///
/// Las tres acciones comparten este camino porque **las reglas de qué se puede
/// hacer desde qué estado viven en el núcleo**, no aquí: `AccionDeCampana`
/// decide, y `arles-db` lo aplica en una transacción con su comparación de
/// estado. Escribir esas reglas en este archivo las duplicaría, y el día que
/// las dos copias discrepasen ganaría la peor.
fn aplicar(
    estado: &tauri::State<'_, EstadoApp>,
    id: &str,
    accion: arles_core::campana::AccionDeCampana,
) -> Result<AccionAplicadaIpc, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(estado)?;

    let resumen = estado
        .db()
        .aplicar_a_la_campana(empresa, id, accion)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    Ok(AccionAplicadaIpc {
        estado: resumen.estado.clave_i18n().to_string(),
        cancelados: resumen.cancelados,
        etapas: resumen.etapas,
    })
}

/// Pausa una campaña. Se deshace con `reanudar_campana`.
///
/// La cola se queda como está: lo que no ha salido, no sale, y sigue ahí.
///
/// # Errores
///
/// [`ErrorIpc`] si la campaña no existe o si su estado no admite pausa — un
/// borrador no se pausa, y una detenida tampoco, porque daría a entender que
/// se puede reanudar.
#[tauri::command]
pub fn pausar_campana(
    estado: tauri::State<'_, EstadoApp>,
    id: String,
) -> Result<AccionAplicadaIpc, ErrorIpc> {
    aplicar(&estado, &id, arles_core::campana::AccionDeCampana::Pausar)
}

/// Reanuda una campaña pausada. Continúa donde iba, con sus límites.
///
/// # Errores
///
/// [`ErrorIpc`] si la campaña no existe o no está en pausa.
#[tauri::command]
pub fn reanudar_campana(
    estado: tauri::State<'_, EstadoApp>,
    id: String,
) -> Result<AccionAplicadaIpc, ErrorIpc> {
    aplicar(&estado, &id, arles_core::campana::AccionDeCampana::Reanudar)
}

/// Detiene una campaña. **No se deshace.**
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ PIDE EL NOMBRE, Y POR QUÉ SE COMPRUEBA AQUÍ
///
/// No es ceremonia: con tres campañas en marcha —el escenario de Dirección—
/// el riesgo real no es detener sin querer, es **detener la equivocada**. Un
/// «¿seguro?» no protege de eso; escribir el nombre sí, porque obliga a mirar
/// cuál se está deteniendo.
///
/// Se comprueba en este comando y no en la pantalla porque la pantalla es la
/// webview: una comprobación que sólo vive ahí la puede saltar cualquiera que
/// llame al comando por otro camino. `exige_confirmacion()` del núcleo dice
/// **que hace falta**; este comando **la exige**.
///
/// # Errores
///
/// [`ErrorIpc`] con `error.app.nombre_no_coincide` si el nombre escrito no es
/// el de la campaña, o si la campaña no existe o no se puede detener.
#[tauri::command]
pub fn detener_campana(
    estado: tauri::State<'_, EstadoApp>,
    id: String,
    nombre_escrito: String,
) -> Result<AccionAplicadaIpc, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;

    let campanas = estado
        .db()
        .campanas(empresa)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    let cual = campanas
        .iter()
        .find(|c| c.id.to_string() == id)
        .ok_or_else(|| ErrorIpc::from(crate::AppError::NombreNoCoincide))?;

    if !el_nombre_coincide(&cual.nombre, &nombre_escrito) {
        return Err(ErrorIpc::from(crate::AppError::NombreNoCoincide));
    }

    aplicar(&estado, &id, arles_core::campana::AccionDeCampana::Detener)
}

/// ¿El nombre escrito es el de la campaña?
///
/// Se comparan **sin espacios de sobra**: quien copia el nombre de la pantalla
/// se trae a veces un espacio al final, y rechazarlo por eso enseña a
/// desconfiar del control en vez de a leerlo.
///
/// Lo que **no** se hace es ignorar mayúsculas ni acentos. «Clientes de agosto»
/// y «clientes de agosto» son la misma campaña para una persona, pero el
/// control existe justo para obligar a mirar cuál se detiene, y una comparación
/// blanda deja pasar el error que este control busca impedir: escribir de
/// memoria en vez de leer la pantalla.
///
/// Está separado del comando para poder probarlo: dentro de `detener_campana`
/// viviría pegado a `tauri::State`, que no se construye en una prueba.
fn el_nombre_coincide(real: &str, escrito: &str) -> bool {
    real.trim() == escrito.trim()
}

/// Para **todos** los envíos de la empresa. Se deshace con `levantar_parada`.
///
/// No pide confirmación a propósito (D-7B): es reversible, y una emergencia
/// con un diálogo de por medio es una emergencia peor. Lo que sí queda es
/// constancia de quién y cuándo, que `arles-db` escribe en la misma operación.
///
/// # Errores
///
/// [`ErrorIpc`] si no se puede escribir en la base.
#[tauri::command]
pub fn parar_todo(
    estado: tauri::State<'_, EstadoApp>,
    quien: Option<String>,
) -> Result<EstadoDelEnvioIpc, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;

    let parada = estado
        .db()
        .poner_parada_de_emergencia(empresa, quien.as_deref())
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    Ok(parada.into())
}

/// Levanta la parada de emergencia. Los envíos siguen donde iban.
///
/// # Errores
///
/// [`ErrorIpc`] si no se puede escribir en la base.
#[tauri::command]
pub fn levantar_parada(estado: tauri::State<'_, EstadoApp>) -> Result<EstadoDelEnvioIpc, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;

    estado
        .db()
        .levantar_parada_de_emergencia(empresa)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    Ok(arles_core::campana::ParadaDeEmergencia::Levantada.into())
}

/// ¿Hay algo parado ahora mismo?
///
/// Lo pregunta la pantalla al abrirse, para saber si enseñar «parar todo» o
/// «volver a enviar».
///
/// # Errores
///
/// [`ErrorIpc`] si no se puede leer la base.
#[tauri::command]
pub fn estado_del_envio(
    estado: tauri::State<'_, EstadoApp>,
) -> Result<EstadoDelEnvioIpc, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;

    let parada = estado
        .db()
        .parada_de_emergencia(empresa)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    Ok(parada.into())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    /// Verificada rompiéndola (§2): con `real.trim().eq_ignore_ascii_case(...)`
    /// la última afirmación falla con
    /// «un nombre parecido no es el nombre: detener no se deshace».
    #[test]
    fn detener_exige_el_nombre_exacto() {
        assert!(el_nombre_coincide(
            "Clientes de agosto",
            "Clientes de agosto"
        ));

        // El espacio de sobra al copiar y pegar no cuenta como error.
        assert!(
            el_nombre_coincide("Clientes de agosto", "  Clientes de agosto "),
            "copiar el nombre con un espacio de más es normal y no debería \
             rechazarse"
        );

        assert!(
            !el_nombre_coincide("Clientes de agosto", "Clientes de Agosto"),
            "un nombre parecido no es el nombre: detener no se deshace"
        );
        assert!(
            !el_nombre_coincide("Clientes de agosto", "Reactivación"),
            "el riesgo real con varias campañas es detener la equivocada"
        );
        assert!(!el_nombre_coincide("Clientes de agosto", ""));
    }

    /// La parada levantada no inventa un instante ni un autor.
    ///
    /// Verificada rompiéndola: devolviendo `desde: Some(String::new())` en la
    /// rama `Levantada`, esta prueba falla — y la pantalla habría enseñado
    /// «detenido desde » con la fecha vacía.
    #[test]
    fn una_parada_levantada_no_trae_fecha_ni_autor() {
        let ipc: EstadoDelEnvioIpc = arles_core::campana::ParadaDeEmergencia::Levantada.into();

        assert!(!ipc.parado);
        assert_eq!(ipc.desde, None);
        assert_eq!(ipc.por, None);
        assert_eq!(ipc.clave, "campana.parada.levantada");
    }

    /// Y la puesta conserva las dos cosas que la describen.
    #[test]
    fn una_parada_puesta_dice_desde_cuando_y_quien() {
        let ipc: EstadoDelEnvioIpc = arles_core::campana::ParadaDeEmergencia::Puesta {
            desde: "2026-09-30T11:04:00Z".to_owned(),
            por: Some("adrian".to_owned()),
        }
        .into();

        assert!(ipc.parado);
        assert_eq!(ipc.desde.as_deref(), Some("2026-09-30T11:04:00Z"));
        assert_eq!(ipc.por.as_deref(), Some("adrian"));
        assert_eq!(ipc.clave, "campana.parada.puesta");
    }
}
