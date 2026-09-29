//! El motor corriendo de verdad: un hilo, aparte de la ventana.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ CAMBIA AQUÍ
//!
//! Hasta esta entrega el motor de la Fase 4 existía, estaba probado entero y
//! **nadie lo arrancaba**. `arles-app` ni siquiera dependía de `arles-motor`.
//! Estaban construidas las dos mitades —la que sabe gestionar una cola y la que
//! sabe hablar SMTP— y no estaban presentadas.
//!
//! Esto las presenta: un hilo que cada pocos segundos pregunta qué campañas
//! están enviando, arma el adaptador de cada una con la fábrica de la 5.4a, y
//! le da pasos al trabajador.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ UN HILO Y NO EL DE LA VENTANA
//!
//! El §51 lo pide con todas las letras: **cerrar la ventana no detiene una
//! campaña**. En Tauri, el hilo de la interfaz se bloquea mientras la webview
//! hace lo suyo, y un envío que espera sesenta segundos a un servidor que no
//! contesta congelaría el programa entero.
//!
//! Y al revés: lo que pasa en la ventana no puede parar el motor. Parar es un
//! acto explícito —el botón, que escribe en la base— y no un efecto secundario
//! de cerrar una pantalla.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! EL HILO NO MUERE POR UN ERROR. NUNCA.
//!
//! Es la decisión más importante de este archivo, y va al revés de lo que hace
//! el resto del programa.
//!
//! En cualquier otro sitio, un error se propaga hasta quien pueda hacer algo.
//! Aquí no hay nadie: si este hilo termina, **ARLES deja de enviar y la ventana
//! sigue abierta como si nada**. Las campañas se quedan en «en marcha», la
//! pantalla no cambia, y el usuario se entera horas después de que no salió
//! ningún correo.
//!
//! Así que un fallo al consultar la base, al armar un adaptador o al dar un
//! paso **no saca al hilo del bucle**: se anota y se vuelve a intentar en la
//! siguiente vuelta. La base puede estar bloqueada un instante; el llavero
//! puede tardar; una cuenta puede estar mal configurada mientras las otras
//! cinco funcionan.
//!
//! Lo único que sí detiene el hilo es perder el cerrojo: eso significa que otro
//! ARLES está enviando sobre esta misma base, y entonces callarse es lo
//! correcto.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use arles_db::Db;
use arles_db::trabajo::EtapaEnMarcha;
use arles_email::Contenido;
use arles_motor::bucle::{Ajustes, EntornoDelSistema, Trabajador, freno_de_la_base};
use arles_motor::instancia::Cerrojo;

use crate::fabrica::{ElLlaveroDelSistema, FuenteDeContrasenas, proveedor_de_la_cuenta};

/// Cuánto duerme el hilo entre vueltas cuando no hay nada que hacer.
///
/// Cinco segundos. A cincuenta correos diarios, mirar más a menudo no adelanta
/// ningún envío y sólo consulta la base para nada. Y mirar mucho menos haría
/// que pulsar «Reanudar» tardara en notarse, que es lo que hace pensar que el
/// botón no funcionó.
pub const DESCANSO: Duration = Duration::from_secs(5);

/// Cuántos pasos se dan de una tacada por etapa antes de soltar el hilo.
///
/// Acota el trabajo de una vuelta para que una cola de medio millón no ocupe el
/// hilo sin soltarlo nunca — y para que parar se note dentro de la misma vuelta
/// y no al final de la cola.
pub const PASOS_POR_VUELTA: u32 = 25;

/// El motor corriendo. Mientras esto exista, ARLES envía.
///
/// Se guarda en el estado de la aplicación y **se para al soltarlo**: no hay un
/// método «apagar» que alguien pueda olvidarse de llamar.
#[derive(Debug)]
pub struct MotorEnMarcha {
    parar: Arc<AtomicBool>,
    hilo: Option<JoinHandle<()>>,
}

impl MotorEnMarcha {
    /// Arranca el hilo. El cerrojo se le entrega: el motor es su dueño.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// EL CERROJO VIVE DENTRO DEL HILO, A PROPÓSITO
    ///
    /// El latido lo da el bucle (ver `instancia.rs`), así que el cerrojo tiene
    /// que estar donde late. Dejarlo fuera —en el estado de la aplicación—
    /// significaría que un hilo muerto deja un cerrojo que ya no late pero que
    /// sigue existiendo, y ARLES parecería vivo sin estar enviando nada.
    ///
    /// Al terminar el hilo, el cerrojo se suelta solo y el archivo se borra.
    #[must_use]
    pub fn arrancar(db: Arc<Db>, cerrojo: Cerrojo) -> Self {
        let parar = Arc::new(AtomicBool::new(false));
        let bandera = Arc::clone(&parar);

        let hilo = thread::Builder::new()
            .name("arles-motor".to_owned())
            .spawn(move || correr(&db, &ElLlaveroDelSistema, &cerrojo, &bandera))
            .ok();

        Self { parar, hilo }
    }

    /// Pide al hilo que termine y lo espera.
    ///
    /// Se llama solo al soltar el motor. Esperar importa: sin esto, cerrar
    /// ARLES podría dejar un envío a medias entre el `sending` de la base y la
    /// respuesta del servidor, que es exactamente el estado ambiguo que el
    /// ADR-0004 obliga a revisar a mano después.
    pub fn parar(&mut self) {
        self.parar.store(true, Ordering::Relaxed);
        if let Some(h) = self.hilo.take() {
            let _ = h.join();
        }
    }
}

impl Drop for MotorEnMarcha {
    fn drop(&mut self) {
        self.parar();
    }
}

/// El bucle del hilo. **No devuelve error**: ver la nota de la cabecera.
///
/// La fuente de contraseñas entra como parámetro por la misma razón que en la
/// fábrica: sin eso, **este bucle no se puede probar**. El llavero del sistema
/// no existe en la integración continua, y el pegamento que esta entrega añade
/// —leer las etapas, armar el adaptador, dar pasos— se quedaría sin una sola
/// comprobación hasta que alguien abriera ARLES en una máquina de verdad.
fn correr(db: &Db, secretos: &dyn FuenteDeContrasenas, cerrojo: &Cerrojo, parar: &AtomicBool) {
    while !parar.load(Ordering::Relaxed) {
        // El latido, primero. Si el cerrojo ya no es nuestro, otro ARLES está
        // enviando sobre esta base y aquí no se manda nada más.
        if cerrojo.latir().is_err() {
            return;
        }

        let hubo_trabajo = una_vuelta(db, secretos, parar);

        // Sólo se descansa si no pasó nada. Con trabajo pendiente se vuelve
        // enseguida: el ritmo lo pone el presupuesto del motor, no este sueño.
        if !hubo_trabajo {
            dormir_a_trozos(parar);
        }
    }
}

/// Una pasada por todas las etapas que están enviando.
///
/// Devuelve si alguna hizo algo, para decidir si toca descansar.
fn una_vuelta(db: &Db, secretos: &dyn FuenteDeContrasenas, parar: &AtomicBool) -> bool {
    // Sin empresa configurada no hay nada que enviar: es el primer arranque,
    // antes del alta. No es un error y no se anota como tal.
    let Ok(Some(empresa)) = db.empresa() else {
        return false;
    };

    let etapas = match db.etapas_en_marcha(empresa.id) {
        Ok(e) => e,
        // La base puede estar ocupada un instante. Se vuelve a mirar en la
        // siguiente vuelta; salir del bucle dejaría ARLES mudo para siempre.
        Err(_) => return false,
    };

    let mut hubo_trabajo = false;
    for etapa in etapas {
        if parar.load(Ordering::Relaxed) {
            break;
        }
        hubo_trabajo |= servir(db, secretos, empresa.id, &etapa);
    }
    hubo_trabajo
}

/// Da pasos por una etapa. Devuelve si quedó trabajo por hacer.
///
/// Una cuenta que no puede enviar —sin contraseña, pendiente de configurar— no
/// detiene a las demás: se la salta esta vuelta. La razón por la que no envía
/// ya está en la base, y la pantalla de Remitentes la enseña; insistir aquí
/// sólo gastaría intentos de autenticación contra el proveedor.
fn servir(
    db: &Db,
    secretos: &dyn FuenteDeContrasenas,
    empresa: arles_core::ids::CompanyId,
    etapa: &EtapaEnMarcha,
) -> bool {
    let Ok(proveedor) = proveedor_de_la_cuenta(
        db,
        secretos,
        etapa.cuenta,
        // Provisional hasta la Fase 6, donde el asunto y el cuerpo los produce
        // la plantilla de la campaña por contacto. Está aquí y no inventado
        // dentro del adaptador para que ese cambio sea sustituir esto y nada
        // más.
        Contenido {
            asunto: String::new(),
            cuerpo: String::new(),
        },
    ) else {
        return false;
    };

    let freno = freno_de_la_base(db, empresa, &etapa.campana);
    let trabajador = Trabajador::nuevo(
        db,
        &proveedor,
        &EntornoDelSistema,
        &freno,
        Ajustes {
            empresa,
            campana: etapa.campana.clone(),
            cuenta: etapa.cuenta.to_string(),
            canal: etapa.canal,
            ventana: etapa.ventana.clone(),
            limites: etapa.limites,
            zona_de_la_empresa: etapa.zona_de_la_empresa.clone(),
        },
    );

    // Un fallo de la base en un paso no saca al hilo del bucle: se vuelve a
    // intentar en la siguiente vuelta.
    trabajador
        .correr(PASOS_POR_VUELTA)
        .map(|ultimo| ultimo.hay_mas_que_hacer())
        .unwrap_or(false)
}

/// Duerme, pero mirando la bandera de parada por el camino.
///
/// Un `sleep` de cinco segundos haría que cerrar ARLES tardara hasta cinco
/// segundos en cada campaña. Se trocea para que cerrar sea inmediato.
fn dormir_a_trozos(parar: &AtomicBool) {
    const TROZO: Duration = Duration::from_millis(100);
    let mut restante = DESCANSO;
    while restante > Duration::ZERO && !parar.load(Ordering::Relaxed) {
        let este = restante.min(TROZO);
        thread::sleep(este);
        restante -= este;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Cerrar ARLES no espera cinco segundos por campaña.**
    ///
    /// Es la razón de que el descanso se trocee. Con un `sleep` entero, cerrar
    /// el programa se quedaría pensando hasta que cada hilo terminara su siesta
    /// — y el usuario ve una ventana que no se cierra.
    ///
    /// Se mide: pedir parar durante el descanso tiene que volver en mucho menos
    /// de lo que dura el descanso.
    ///
    /// Verificada rompiéndola el 28/09/2026: cambiando `dormir_a_trozos` por un
    /// `thread::sleep(DESCANSO)`, esta prueba falla.
    #[test]
    fn pedir_parar_durante_el_descanso_no_espera_al_descanso_entero() {
        let parar = Arc::new(AtomicBool::new(false));
        let bandera = Arc::clone(&parar);

        let hilo = thread::spawn(move || {
            let t0 = std::time::Instant::now();
            dormir_a_trozos(&bandera);
            t0.elapsed()
        });

        thread::sleep(Duration::from_millis(150));
        parar.store(true, Ordering::Relaxed);

        let tardo = hilo.join().expect("el hilo del descanso termina");
        assert!(
            tardo < DESCANSO / 2,
            "el descanso tiene que mirar la bandera: tardó {tardo:?} de {DESCANSO:?}"
        );
    }

    /// Y sin nadie que le diga que pare, duerme lo que tiene que dormir.
    ///
    /// Sin esta segunda prueba, `dormir_a_trozos` podría no dormir nada y la de
    /// arriba seguiría pasando — el motor consultaría la base sin parar.
    #[test]
    fn sin_que_nadie_pare_el_descanso_dura_lo_que_dice() {
        let parar = AtomicBool::new(false);
        let t0 = std::time::Instant::now();
        dormir_a_trozos(&parar);
        assert!(
            t0.elapsed() >= DESCANSO,
            "un descanso que no descansa consulta la base en un bucle cerrado"
        );
    }

    /// **Soltar el motor lo para y espera al hilo.**
    ///
    /// Sin el `join`, cerrar ARLES podría dejar un envío a medias entre el
    /// `sending` de la base y la respuesta del servidor — el estado ambiguo que
    /// el ADR-0004 obliga a revisar a mano después.
    #[test]
    fn soltar_el_motor_termina_el_hilo() {
        use arles_db::ClaveMaestra;

        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Arc::new(Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre"));
        let cerrojo = Cerrojo::tomar(dir.path(), "prueba").expect("toma el cerrojo");

        let motor = MotorEnMarcha::arrancar(Arc::clone(&db), cerrojo);
        // Sin empresa configurada el bucle no hace nada, que es justo lo que
        // hace falta para medir el apagado y no el envío.
        let t0 = std::time::Instant::now();
        drop(motor);

        assert!(
            t0.elapsed() < DESCANSO,
            "soltar el motor no puede tardar más que un descanso entero"
        );
        // Y el cerrojo queda libre para el siguiente ARLES.
        Cerrojo::tomar(dir.path(), "el-siguiente").expect("el cerrojo se soltó al parar");
    }
}

/// El pegamento de esta entrega, ejercitado de punta a punta.
///
/// ─────────────────────────────────────────────────────────────────────────
/// QUÉ SE COMPRUEBA AQUÍ Y QUÉ NO
///
/// Lo que esta entrega añade es el **pegamento**: leer de la base qué etapas
/// están enviando, armar el adaptador de cada una con su contraseña, y darle
/// pasos al trabajador. Eso es lo que se ejercita aquí, con una base sembrada
/// y un llavero de mentira.
///
/// **Lo que no se comprueba aquí es que salga un correo.** El servidor de
/// mentira que habla cifrado vive dentro de `arles-email` y no se exporta a
/// propósito: sacarlo metería un motor de TLS y un generador de certificados en
/// el programa que se entrega. Así que el envío en sí sigue probado donde
/// siempre —en `arles-email`, contra ese servidor— y el bucle del trabajador
/// donde siempre —en `arles-motor`, contra el proveedor simulado—.
///
/// Lo que estas pruebas usan es un servidor que **no existe**: un puerto
/// cerrado. Basta para lo que hace falta ver, que es que la cadena entera
/// llega hasta intentar la conexión — y no se queda en el camino por una
/// consulta que no devuelve nada o una cuenta que no se arma.
#[cfg(test)]
mod el_pegamento {
    use super::*;
    use arles_db::ClaveMaestra;
    use rusqlite::params;

    use crate::error::AppError;

    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
    const CUENTA: &str = "01900000-0000-7000-8000-00000000000a";

    struct LlaveroDeMentira;
    impl FuenteDeContrasenas for LlaveroDeMentira {
        fn contrasena_de(&self, _id: &str) -> Result<Option<String>, AppError> {
            Ok(Some("dieciseis-letras".to_owned()))
        }

        fn token_del_numero(&self, _id: &str) -> Result<Option<String>, AppError> {
            Ok(Some("EAAG-token-de-mentira".to_owned()))
        }
    }

    /// Un llavero que no tiene nada: la cuenta no se podrá armar.
    struct LlaveroVacio;
    impl FuenteDeContrasenas for LlaveroVacio {
        fn contrasena_de(&self, _id: &str) -> Result<Option<String>, AppError> {
            Ok(None)
        }

        fn token_del_numero(&self, _id: &str) -> Result<Option<String>, AppError> {
            Ok(None)
        }
    }

    /// Una base con una empresa, una cuenta apuntando a **un puerto cerrado**,
    /// una campaña en marcha con su etapa, y un envío pendiente en la cola.
    fn base_con_una_campana_enviando() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");

        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('{EMPRESA}', 'TELEMETRY', 'MX', 'America/Mexico_City',
                         'hola@t.mx', '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');

                 -- `a.localhost` es la máquina misma (RFC 6761) y el puerto 1 no
                 -- escucha: la conexión se rechaza enseguida, sin salir de aquí
                 -- y sin esperar a ningún tiempo de espera.
                 --
                 -- El nombre lleva punto a propósito. `localhost` a secas no
                 -- pasa la validación del núcleo —un servidor de correo sin
                 -- punto no es un nombre de dominio—, y la primera versión de
                 -- esta prueba se cortaba ahí, en la fábrica, sin llegar nunca
                 -- al trabajador. Lo cazó esta misma prueba.
                 INSERT INTO email_account (id, company_id, display_name, email_address,
                                            provider_kind, credential_ref, status,
                                            daily_limit, hourly_limit, created_at, updated_at,
                                            smtp_host, smtp_port, smtp_security, smtp_username)
                 VALUES ('{CUENTA}', '{EMPRESA}', 'Ventas', 'ventas@t.mx', 'smtp',
                         'llavero://v', 'connected', 50, 10,
                         '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z',
                         'a.localhost', 1, 'starttls', 'ventas@t.mx');

                 INSERT INTO execution_window (id, company_id, name, days_of_week,
                                               start_hour, end_hour, timezone, created_at)
                 VALUES ('vent-1', '{EMPRESA}', 'Siempre', 127, 0, 24,
                         'America/Mexico_City', '2026-09-01T00:00:00Z');

                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES ('camp-1', '{EMPRESA}', 'Prueba', 'running',
                         '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');

                 INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             email_account_id, execution_window_id,
                                             status, created_at, updated_at)
                 VALUES ('etapa-1', 'camp-1', 1, 'email', '{CUENTA}', 'vent-1',
                         'running', '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');

                 INSERT INTO message_attempt (id, campaign_id, channel, contact_id,
                                              contact_address, email_account_id,
                                              idempotency_key, state, attempt_count,
                                              created_at, updated_at)
                 VALUES ('int-1', 'camp-1', 'email', NULL, 'quien@ejemplo.mx',
                         '{CUENTA}', 'idem-1', 'queued', 0,
                         '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');"
            ))
        })
        .expect("siembra");

        (dir, db)
    }

    fn estado_del_intento(db: &Db) -> String {
        db.ejecutar_en_pruebas(|c| {
            c.query_row(
                "SELECT state FROM message_attempt WHERE id = 'int-1'",
                params![],
                |f| f.get::<_, String>(0),
            )
        })
        .expect("lee el estado")
    }

    /// **Una vuelta del motor toma el trabajo pendiente y lo intenta.**
    ///
    /// Es la prueba de que el pegamento está puesto: la consulta encuentra la
    /// etapa, la fábrica arma el adaptador con la contraseña del llavero, y el
    /// trabajador da un paso de verdad contra un servidor que no existe.
    ///
    /// El intento no puede quedarse en `queued`: eso significaría que la cadena
    /// se cortó en algún punto y el motor no llegó a tocarlo.
    ///
    /// Verificada rompiéndola el 28/09/2026 de dos formas, y las dos la tumban:
    /// quitando la llamada a `servir` de `una_vuelta`, y haciendo que
    /// `etapas_en_marcha` no devuelva nada.
    #[test]
    fn una_vuelta_toma_el_trabajo_pendiente_y_lo_intenta() {
        let (_dir, db) = base_con_una_campana_enviando();
        assert_eq!(estado_del_intento(&db), "queued", "antes, sin tocar");

        una_vuelta(&db, &LlaveroDeMentira, &AtomicBool::new(false));

        assert_ne!(
            estado_del_intento(&db),
            "queued",
            "el motor no llegó a tocar el envío: la cadena se cortó antes"
        );
    }

    /// **Sin contraseña en el llavero, el trabajo se queda donde estaba.**
    ///
    /// Es la otra mitad de la prueba de arriba, y hace falta: sin ella, aquélla
    /// pasaría igual con un motor que marca los envíos por su cuenta sin
    /// haberlos intentado.
    ///
    /// Y es la garantía de la 5.4a vista desde el motor: una cuenta sin
    /// contraseña **no** se intenta con la contraseña vacía, que gastaría un
    /// intento de autenticación fallido contra el proveedor.
    #[test]
    fn sin_contrasena_el_trabajo_no_se_toca() {
        let (_dir, db) = base_con_una_campana_enviando();

        una_vuelta(&db, &LlaveroVacio, &AtomicBool::new(false));

        assert_eq!(
            estado_del_intento(&db),
            "queued",
            "sin contraseña no se puede enviar, y el trabajo tiene que seguir \
             esperando: marcarlo sería perderlo"
        );
    }

    /// **Una campaña pausada no se toca, aunque haya trabajo en la cola.**
    ///
    /// Es la comprobación de que el motor obedece el botón. Sin esto, pausar
    /// cambiaría el rótulo de la pantalla y los correos seguirían saliendo.
    #[test]
    fn una_campana_pausada_no_se_toca() {
        let (_dir, db) = base_con_una_campana_enviando();
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch("UPDATE campaign SET status='paused' WHERE id='camp-1'")
        })
        .expect("pausa");

        una_vuelta(&db, &LlaveroDeMentira, &AtomicBool::new(false));

        assert_eq!(estado_del_intento(&db), "queued", "pausada no envía");
    }

    /// **Y con la bandera de parar puesta, el motor no toca nada.**
    ///
    /// Es lo que hace que cerrar ARLES no deje un envío a medias entre el
    /// `sending` de la base y la respuesta del servidor.
    #[test]
    fn con_la_bandera_de_parar_puesta_no_se_toca_nada() {
        let (_dir, db) = base_con_una_campana_enviando();

        una_vuelta(&db, &LlaveroDeMentira, &AtomicBool::new(true));

        assert_eq!(estado_del_intento(&db), "queued");
    }
}
