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
use arles_db::actividad::SucesoNuevo;
use arles_db::trabajo::EtapaEnMarcha;
use arles_email::Contenido;
use arles_motor::bucle::{Ajustes, EntornoDelSistema, Paso, Trabajador, freno_de_la_base};
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

    // ── AQUÍ SE TIRABA LO QUE EL MOTOR SABE (7.1) ──────────────────────────
    //
    // Esto era:
    //
    //     .correr(PASOS_POR_VUELTA)
    //     .map(|ultimo| ultimo.hay_mas_que_hacer())
    //
    // Once desenlaces distintos, cada uno con su texto en español escrito desde
    // la Fase 4 y pensado para la pantalla de Actividad, convertidos en un
    // sí/no. De que a las 10:04 la cuenta se paró no quedaba nada.
    //
    // Ahora cada paso se anota. Y se anota **fuera de la transacción del
    // envío**, a propósito: meter una segunda escritura dentro de la
    // transacción que decide si un correo sale es arriesgar lo que importa
    // —el intento— por lo que acompaña —la narración—. El precio es que una
    // línea se puede perder si el programa muere justo ahí, y está aceptado y
    // escrito en la cabecera de la migración V10.
    let mut anotar = |paso: &Paso| anotar_el_paso(db, empresa, etapa, paso);

    // Un fallo de la base en un paso no saca al hilo del bucle: se vuelve a
    // intentar en la siguiente vuelta.
    trabajador
        .correr_contando(PASOS_POR_VUELTA, &mut anotar)
        .map(|ultimo| ultimo.hay_mas_que_hacer())
        .unwrap_or(false)
}

/// Escribe un paso del motor en el registro de actividad.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LAS DOS PUERTAS, Y POR QUÉ NO ES UNA SOLA
///
/// Un desenlace de envío se anota **siempre**: cien aceptados son cien correos
/// a cien personas, y contarlos como uno perdería justo lo que hay que
/// guardar.
///
/// Un suceso de bucle se anota **sólo si cambió**. El motor da una vuelta cada
/// cinco segundos: fuera de horario, una etapa diría «fuera de ventana» 17 280
/// veces al día. Con el recorte, un fin de semana entero son dos líneas — la
/// del viernes y la del lunes—, que es lo que alguien querría leer.
///
/// **Un fallo al anotar no para el motor.** Es la regla que ordena las dos
/// cosas: enviar es el trabajo, contarlo es el acompañamiento, y perder la
/// bitácora de un paso nunca puede costar un envío. Se traga en silencio
/// porque el sitio donde se diría —la pantalla de Actividad— es justo el que
/// no se pudo escribir.
fn anotar_el_paso(
    db: &Db,
    empresa: arles_core::ids::CompanyId,
    etapa: &EtapaEnMarcha,
    paso: &Paso,
) {
    // `como_texto()` y no un `match` aquí: esa función **es** el contrato con
    // el esquema —las `CHECK` de las migraciones contienen literalmente esas
    // cadenas— y repetirlo sería una segunda copia que puede divergir.
    let canal = etapa.canal.como_texto();

    let (intento, a, b) = cifras_del_paso(paso);

    let suceso = SucesoNuevo {
        clave: paso.clave_i18n(),
        es_desenlace: intento.is_some(),
        campana: Some(&etapa.campana),
        etapa: Some(&etapa.etapa),
        canal: Some(canal),
        intento,
        numero_a: a,
        numero_b: b,
    };

    let _ = if suceso.es_desenlace {
        db.anotar_suceso(empresa, &suceso)
    } else {
        db.anotar_si_cambia(empresa, &suceso).map(|_| ())
    };
}

/// El intento y las dos cifras que acompañan a un paso.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ ESTÁ FUERA DE `anotar_el_paso`
///
/// Para poder probarla sin base de datos y sin motor, que es lo que hace falta
/// para comprobar la garantía de verdad: **que cada clave que lleva un
/// marcador en su texto traiga la cifra que ese marcador necesita**. Dentro de
/// `anotar_el_paso` haría falta montar un `Db` y una etapa para responder a una
/// pregunta que no depende de ninguno de los dos.
///
/// El orden de las dos cifras es el orden en que aparecen los marcadores en el
/// texto del catálogo. Lo comprueba `toda_razon_trae_las_cifras_de_su_texto`.
fn cifras_del_paso(paso: &Paso) -> (Option<&str>, Option<i64>, Option<i64>) {
    match paso {
        Paso::Reintentara { intento, cual, de } => (
            Some(intento.as_str()),
            Some(i64::from(*cual)),
            Some(i64::from(*de)),
        ),
        // `de` y no nada: el texto dice «Se agotaron los **{de}** intentos», y
        // sin la cifra la pantalla enseñaba el marcador. El número sale de la
        // misma constante del núcleo que usa la decisión de reintentar, no de
        // una copia: dos formas de decir cuántos intentos hay es como se acaba
        // enseñando «se agotaron los 3» cuando la tabla tiene cuatro.
        Paso::Rendido { intento } => (
            Some(intento.as_str()),
            Some(i64::from(arles_core::reintento::total_de_reintentos())),
            None,
        ),
        Paso::Baja { intento }
        | Paso::Cancelado { intento }
        | Paso::Aceptado { intento }
        | Paso::DadoDeBaja { intento }
        | Paso::CuentaParada { intento } => (Some(intento.as_str()), None, None),
        Paso::CuentaEnPausa { segundos_restantes } => {
            (None, i64::try_from(*segundos_restantes).ok(), None)
        }
        // ── EL TOPE, CON SU CIFRA ──────────────────────────────────────────
        //
        // Sin esto, `SinPresupuesto` caía en el `_` de abajo y se anotaba sin
        // números. Y su texto —`ritmo.topeDeLaHora`— dice «Se alcanzó el tope
        // de **{cabian}** envíos por hora»: la pantalla habría enseñado el
        // marcador `{cabian}` tal cual, en la razón más frecuente de las seis.
        //
        // No lo cazaba ninguna prueba porque las dos capas pasan lo suyo: el
        // motor anota una clave correcta, y el catálogo tiene su texto. El
        // hueco está en que la clave pide una cifra que nadie guardó. Lo caza
        // ahora `toda_razon_trae_las_cifras_de_su_texto()`.
        Paso::SinPresupuesto(
            arles_core::ritmo::NoSePuede::TopeDeLaHora { cabian }
            | arles_core::ritmo::NoSePuede::TopeDelDia { cabian },
        ) => (None, Some(i64::from(*cabian)), None),
        Paso::Parado
        | Paso::FueraDeVentana
        | Paso::NoHayTrabajo
        | Paso::SinPresupuesto(arles_core::ritmo::NoSePuede::FueraDeVentana) => (None, None, None),
    }
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

    /// **El motor deja constancia de lo que hizo** (7.1).
    ///
    /// Es la prueba de que el `Paso` ya no se tira. Antes de esta entrega,
    /// `una_vuelta` lo convertía en un booleano y de un envío entero no
    /// quedaba ni cuándo ni por qué — sólo el estado final del intento.
    ///
    /// Se mide sobre la base de verdad y no sobre el valor devuelto: lo que
    /// importa no es que `correr_contando` llame al aviso, es que **la línea
    /// acabe escrita**, que es lo que la pantalla va a leer.
    ///
    /// Verificada rompiéndola: volviendo a `.correr(PASOS_POR_VUELTA)` en
    /// `servir`, esta prueba falla con «el motor dio una vuelta entera y no
    /// dejó constancia de nada».
    #[test]
    fn una_vuelta_deja_constancia_en_el_registro() {
        let (_dir, db) = base_con_una_campana_enviando();
        let empresa: arles_core::ids::CompanyId = EMPRESA.parse().expect("id");

        assert_eq!(
            db.sucesos(empresa, &arles_db::actividad::Filtro::default())
                .expect("lee")
                .total,
            0,
            "antes de la vuelta el registro está vacío"
        );

        una_vuelta(&db, &LlaveroDeMentira, &AtomicBool::new(false));

        let p = db
            .sucesos(empresa, &arles_db::actividad::Filtro::default())
            .expect("lee");
        assert!(
            p.total > 0,
            "el motor dio una vuelta entera y no dejó constancia de nada"
        );

        // Y lo que quedó es una clave de i18n, no un texto suelto (§139): es
        // lo que la pantalla pasa a `t()`, y un texto en español aquí haría
        // imposible traducir o corregir el mensaje sin migrar la tabla.
        let s = p
            .sucesos
            .first()
            .expect("acaba de comprobarse que hay al menos uno");
        assert!(
            s.clave.contains('.') && !s.clave.contains(' '),
            "la clave del suceso tiene que ser de i18n, y salió «{}»",
            s.clave
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

    // ═══════════════════════════════════════════════════════════════════════
    // LA CLAVE Y SU CIFRA · entrega 7.3
    //
    // El hueco que esto caza es del octavo tipo del §9 y el más difícil de
    // ver: las dos capas pasan lo suyo. El motor anota una clave correcta
    // —`ritmo.topeDeLaHora`—, y el catálogo tiene su texto —«Se alcanzó el tope
    // de {cabian} envíos por hora»—. Nadie compara las dos cosas, y lo que
    // llega a la pantalla es el marcador `{cabian}` tal cual, en la razón más
    // frecuente por la que una campaña está quieta.
    //
    // Lo encontró esta prueba antes de existir la pantalla, en dos claves:
    // `ritmo.topeDeLaHora`/`topeDelDia`, que caían en el `_` del `match`, y
    // `reintento.seAgotaronLosIntentos`, que pedía `{de}` y no traía nada.
    // ═══════════════════════════════════════════════════════════════════════

    /// Cuántos marcadores `{…}` tiene el texto de una clave del catálogo.
    ///
    /// Lee `es.ts` de verdad y no una copia: el catálogo es la fuente, y una
    /// lista de marcadores escrita a mano aquí se quedaría vieja el día que
    /// alguien reescriba un texto — que es exactamente el día en que hace falta
    /// que esta prueba salte.
    fn marcadores_de(clave: &str) -> usize {
        let catalogo = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../app/src/app/locales/es.ts"),
        )
        .expect("el catálogo de textos está en el repositorio");

        // La clave viaja como `grupo.nombre`, y en `es.ts` vive anidada. Se
        // busca el grupo y dentro de él la propiedad, que es lo bastante
        // preciso para las claves de un nivel que emite el motor. Si algún día
        // hubiera dos niveles, esta prueba fallaría al no encontrar el texto —
        // que es el fallo correcto: obliga a mirarla.
        let (grupo, nombre) = clave.split_once('.').expect("la clave es grupo.nombre");
        let desde = catalogo
            .find(&format!("\n  {grupo}: {{"))
            .unwrap_or_else(|| panic!("el catálogo no tiene el grupo «{grupo}» de «{clave}»"));
        // El grupo acaba en el primer `\n  },` a su nivel de sangría.
        let hasta = catalogo[desde..]
            .find("\n  },")
            .map_or(catalogo.len(), |n| desde + n);
        let bloque = &catalogo[desde..hasta];

        // ── SE RECORRE POR LÍNEAS, Y LA PRIMERA VERSIÓN NO ──────────────────
        //
        // Buscaba el fin del texto con `find("\n    ")`, cuatro espacios. Y
        // `\n      ` —seis, la continuación de un texto partido con `+`— empieza
        // por esos cuatro, así que cortaba en la primera continuación y se
        // perdía los marcadores de la segunda línea. Se vio al estrenarla:
        // `motor.laCuentaEstaEnPausa` salía con cero marcadores y su `{minutos}`
        // está justo ahí. Otra comprobación que miraba la línea equivocada (§2).
        //
        // Ahora una línea pertenece al texto mientras esté MÁS sangrada que la
        // propiedad, que es lo que significa ser su continuación.
        let mut dentro = false;
        let mut texto = String::new();
        for linea in bloque.lines() {
            let sangria = linea.len() - linea.trim_start().len();
            if dentro {
                // Una línea al nivel de la propiedad o menos cierra el texto.
                if !linea.trim().is_empty() && sangria <= 4 {
                    break;
                }
                texto.push_str(linea);
            } else if linea.trim_start().starts_with(&format!("{nombre}:")) && sangria == 4 {
                dentro = true;
                texto.push_str(linea);
            }
        }
        assert!(dentro, "el grupo «{grupo}» no tiene «{nombre}»");

        // Un comentario de línea detrás del texto no forma parte de él, y podría
        // llevar llaves: se corta cada línea en su `//`.
        texto
            .lines()
            .filter_map(|l| l.split("//").next())
            .map(|l| l.matches('{').count())
            .sum()
    }

    /// **Cada clave que el motor anota trae las cifras que su texto pide.**
    ///
    /// Si el texto dice «{cabian} envíos por hora», el suceso tiene que traer
    /// una cifra; si dice «{cual} de {de}», dos. Un marcador sin cifra sale a la
    /// pantalla tal cual, y una cifra sin marcador es un dato que nadie enseña.
    ///
    /// Verificada rompiéndola el 30/09/2026 de dos formas: devolviendo
    /// `(None, None, None)` para `SinPresupuesto` falla con
    /// «ritmo.topeDeLaHora: el texto pide 1 cifra(s) y el paso trae 0», y
    /// quitando el `de` de `Rendido` falla igual con
    /// `reintento.seAgotaronLosIntentos`.
    #[test]
    fn toda_razon_trae_las_cifras_de_su_texto() {
        use arles_core::ritmo::NoSePuede;

        // Las trece variantes, con un valor cualquiera: lo que se comprueba es
        // la forma, no los números. La lista es exhaustiva a mano **y el
        // `match` de `cifras_del_paso` no tiene comodín**, así que un `Paso`
        // nuevo no compila hasta que alguien decida sus cifras — y entonces
        // esta prueba comprueba que coinciden con su texto.
        let un_id = || "intento-1".to_owned();
        let pasos = vec![
            Paso::Parado,
            Paso::FueraDeVentana,
            Paso::NoHayTrabajo,
            Paso::SinPresupuesto(NoSePuede::FueraDeVentana),
            Paso::SinPresupuesto(NoSePuede::TopeDeLaHora { cabian: 100 }),
            Paso::SinPresupuesto(NoSePuede::TopeDelDia { cabian: 500 }),
            Paso::Baja { intento: un_id() },
            Paso::Cancelado { intento: un_id() },
            Paso::Aceptado { intento: un_id() },
            Paso::Reintentara {
                intento: un_id(),
                cual: 2,
                de: 3,
            },
            Paso::Rendido { intento: un_id() },
            Paso::DadoDeBaja { intento: un_id() },
            Paso::CuentaParada { intento: un_id() },
            Paso::CuentaEnPausa {
                segundos_restantes: 1_380,
            },
        ];

        for paso in &pasos {
            let clave = paso.clave_i18n();
            let (_, a, b) = cifras_del_paso(paso);
            let cifras = usize::from(a.is_some()) + usize::from(b.is_some());
            let pide = marcadores_de(clave);
            assert_eq!(
                pide, cifras,
                "{clave}: el texto pide {pide} cifra(s) y el paso trae {cifras}"
            );
        }
    }

    /// **Y el catálogo tiene texto para las catorce**, no sólo para las que
    /// alguien se acordó de escribir.
    ///
    /// Sin esta segunda prueba, un `Paso` cuya clave no existe en `es.ts` pasa
    /// la de arriba —cero marcadores, cero cifras— y la pantalla enseña la
    /// clave cruda, `motor.parado`, delante del usuario. Es lo que hace el §139
    /// con los errores; los pasos del motor no lo tenían.
    #[test]
    fn el_catalogo_tiene_texto_para_cada_paso_del_motor() {
        use arles_core::ritmo::NoSePuede;

        for clave in [
            Paso::Parado.clave_i18n(),
            Paso::FueraDeVentana.clave_i18n(),
            Paso::NoHayTrabajo.clave_i18n(),
            Paso::SinPresupuesto(NoSePuede::FueraDeVentana).clave_i18n(),
            Paso::SinPresupuesto(NoSePuede::TopeDeLaHora { cabian: 1 }).clave_i18n(),
            Paso::SinPresupuesto(NoSePuede::TopeDelDia { cabian: 1 }).clave_i18n(),
            Paso::Baja {
                intento: String::new(),
            }
            .clave_i18n(),
            Paso::Cancelado {
                intento: String::new(),
            }
            .clave_i18n(),
            Paso::Aceptado {
                intento: String::new(),
            }
            .clave_i18n(),
            Paso::Reintentara {
                intento: String::new(),
                cual: 1,
                de: 1,
            }
            .clave_i18n(),
            Paso::Rendido {
                intento: String::new(),
            }
            .clave_i18n(),
            Paso::DadoDeBaja {
                intento: String::new(),
            }
            .clave_i18n(),
            Paso::CuentaParada {
                intento: String::new(),
            }
            .clave_i18n(),
            Paso::CuentaEnPausa {
                segundos_restantes: 0,
            }
            .clave_i18n(),
        ] {
            // `marcadores_de` entra en pánico con un mensaje que nombra la
            // clave si no la encuentra, que es la comprobación.
            let _ = marcadores_de(clave);
        }
    }
}
