//! El programador: lo que ARLES hace solo cada cierto tiempo (R-30).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ CIERRA ESTE ARCHIVO
//!
//! R-25 dejó la revisión del buzón funcionando y con un límite escrito con
//! estas palabras en el manual: **«no se revisa solo cada cierto tiempo; hay
//! que pulsar Revisar el buzón ahora»**. Esto es lo que faltaba — y lo que
//! Dirección pidió el 10/11/2026 al decir «que sea posible darse de baja de
//! manera automática».
//!
//! Hoy tiene una sola tarea. Se llama «programador» y no «revisor de buzones»
//! porque el hilo y su ritmo son lo reutilizable, no la tarea; pero **no se ha
//! construido ningún registro de tareas genérico**, que sería inventar un
//! llamador para piezas que no existen.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ UN HILO PROPIO Y NO EL DEL MOTOR. ESTÁ MEDIDO
//!
//! El sitio cómodo era el bucle del motor: ya da vueltas cada cinco segundos y
//! ya tiene la base. Y **no cabe**, por una razón que no se ve leyendo:
//!
//! El latido del cerrojo de instancia única no mide «ARLES está vivo» sino
//! **«el bucle está dando vueltas»**, porque se refresca al principio de cada
//! una — lo dice la nota de `arles_motor::instancia::Cerrojo::latir`. Así que
//! el diseño del cerrojo da por supuesto que una vuelta es corta.
//!
//! El lector del buzón espera hasta `arles_email::buzon::ESPERA_MAXIMA` —**60
//! s**— por operación de socket. Dos cuentas cuyo servidor IMAP no contesta son
//! **120 s** en una sola vuelta, contra los **90** de
//! `arles_motor::instancia::LATIDO_MAXIMO`. Lo que pasa entonces son dos cosas,
//! y la segunda es la cara:
//!
//! 1. otro ARLES roba el cerrojo, creyendo que éste murió;
//! 2. y el `latir()` de éste falla, que es la señal con la que el bucle del
//!    motor sale — **ARLES deja de enviar porque un servidor de correo ajeno no
//!    contestó**.
//!
//! Lo mide `una_vuelta_lenta_le_cuesta_el_cerrojo_al_motor` en `arles-motor`,
//! verificada con tres rupturas, y lo vigila el validador: `motor.rs` no puede
//! nombrar la revisión del buzón.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! SÓLO CORRE EL ARLES QUE TIENE EL CERROJO
//!
//! El programador se arranca con el motor y no si el cerrojo no se pudo tomar.
//! Dos ARLES sobre la misma base revisando el mismo buzón no romperían nada
//! —suprimir es idempotente por (empresa, canal, dirección)— pero doblarían las
//! conexiones IMAP, que es exactamente lo que el intervalo existe para no
//! hacer. El cerrojo ya significa «este ARLES es el que actúa sobre esta base»,
//! así que no hace falta un segundo mecanismo que diga lo mismo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! Y ESTE HILO TAMPOCO MUERE POR UN ERROR
//!
//! Por el mismo motivo que el del motor: si termina, ARLES deja de revisar
//! buzones y la ventana sigue abierta como si nada. Un servidor que no
//! contesta, una base ocupada un instante o una cuenta mal configurada se
//! anotan y se vuelven a intentar en el intervalo siguiente.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use arles_core::baja_por_correo;
use arles_db::Db;
use arles_db::buzon::BuzonQueSeLee;

use crate::bajas_del_buzon::{ElBuzonDeVerdad, LeeUnBuzon, revisar_uno};
use crate::fabrica::{ElLlaveroDelSistema, FuenteDeContrasenas};

/// Cuánto duerme el hilo entre vueltas.
///
/// Un minuto, y no es el intervalo de la tarea: es cada cuánto se **pregunta**
/// si toca alguna. El intervalo de verdad lo decide
/// `baja_por_correo::REVISAR_CADA_MINUTOS`, y preguntarlo cada minuto lo cumple
/// con un margen de ±1 minuto sobre quince, que sobra.
///
/// Más corto sería una consulta a la base para nada; más largo haría que
/// encender la casilla tardara en notarse, que es lo que hace pensar que no
/// funcionó.
pub const DESCANSO: Duration = Duration::from_secs(60);

/// El programador corriendo. Mientras esto exista, ARLES revisa buzones solo.
///
/// Se guarda en el estado de la aplicación y **se para al soltarlo**, igual que
/// `MotorEnMarcha`: no hay un método «apagar» que alguien pueda olvidarse de
/// llamar.
#[derive(Debug)]
pub struct ProgramadorEnMarcha {
    parar: Arc<AtomicBool>,
    hilo: Option<JoinHandle<()>>,
}

impl ProgramadorEnMarcha {
    /// Arranca el hilo.
    ///
    /// No toma el cerrojo —lo tiene el motor— porque quien llama sólo lo
    /// arranca si el motor arrancó. Ver la nota de la cabecera.
    #[must_use]
    pub fn arrancar(db: Arc<Db>) -> Self {
        let parar = Arc::new(AtomicBool::new(false));
        let bandera = Arc::clone(&parar);

        let hilo = thread::Builder::new()
            .name("arles-programador".to_owned())
            .spawn(move || correr(&db, &ElBuzonDeVerdad, &ElLlaveroDelSistema, &bandera))
            .ok();

        Self { parar, hilo }
    }

    /// Pide al hilo que termine y lo espera.
    pub fn parar(&mut self) {
        self.parar.store(true, Ordering::Relaxed);
        if let Some(h) = self.hilo.take() {
            let _ = h.join();
        }
    }
}

impl Drop for ProgramadorEnMarcha {
    fn drop(&mut self) {
        self.parar();
    }
}

/// El bucle del hilo. **No devuelve error**: ver la nota de la cabecera.
///
/// El lector y el llavero entran como parámetros por la misma razón que en el
/// motor: sin eso, **este bucle no se puede probar**. El llavero del sistema no
/// existe en la integración continua y un servidor IMAP de verdad tampoco, así
/// que el pegamento que este archivo añade se quedaría sin una sola
/// comprobación hasta que alguien abriera ARLES en una máquina con buzón.
fn correr(
    db: &Db,
    lector: &impl LeeUnBuzon,
    secretos: &impl FuenteDeContrasenas,
    parar: &AtomicBool,
) {
    while !parar.load(Ordering::Relaxed) {
        una_vuelta(db, lector, secretos, parar);
        dormir_a_trozos(parar);
    }
}

/// Una pasada por los buzones encendidos. Devuelve cuántos se revisaron.
///
/// Siempre descansa después, al contrario que el motor: aquí no hay una cola
/// que vaciar cuanto antes, hay un ritmo que respetar.
fn una_vuelta(
    db: &Db,
    lector: &impl LeeUnBuzon,
    secretos: &impl FuenteDeContrasenas,
    parar: &AtomicBool,
) -> u32 {
    // Sin empresa configurada no hay buzones: es el primer arranque, antes del
    // alta. No es un error y no se anota como tal.
    let Ok(Some(empresa)) = db.empresa() else {
        return 0;
    };

    let Ok(buzones) = db.buzones_que_se_leen(empresa.id) else {
        // La base puede estar ocupada un instante —el motor escribe en su
        // hilo—. Se vuelve a mirar en la vuelta siguiente; salir del bucle
        // dejaría ARLES sin revisar buzones para siempre.
        return 0;
    };

    let ahora = time::OffsetDateTime::now_utc();
    let mut revisados = 0;
    for buzon in buzones {
        if parar.load(Ordering::Relaxed) {
            break;
        }
        if revisar_si_toca(db, lector, secretos, empresa.id, &buzon, ahora) {
            revisados += 1;
        }
    }
    revisados
}

/// Revisa este buzón si le toca. Devuelve si se llegó a intentar.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LA MARCA DEL INTENTO VA ANTES DEL INTENTO
///
/// Y no es una preferencia de orden: **un intento que no se apunta es un
/// intento que se repite.** Si la marca se escribiera al terminar, cualquier
/// camino que no llegue al final —un cierre de ARLES a mitad, un `return`
/// temprano que alguien añada mañana— dejaría este buzón sin marca nueva, y la
/// vuelta siguiente lo intentaría otra vez. A un minuto por vuelta, eso son 60
/// intentos de autenticación por hora contra el servidor de correo del propio
/// usuario, y lo que viene después es una cuenta bloqueada que ARLES no puede
/// desbloquear.
///
/// Apuntarla antes falla al revés: si ARLES muere entre la marca y la lectura,
/// este buzón espera un intervalo de más. Se pierde un cuarto de hora, no una
/// baja — y `inbox_last_uid` no avanzó, así que esos correos siguen ahí.
///
/// Lo vigila el validador midiendo **la posición** dentro de este cuerpo, no la
/// aparición del nombre: la versión que lo anota en la rama del éxito es
/// precisamente la que martillea.
fn revisar_si_toca(
    db: &Db,
    lector: &impl LeeUnBuzon,
    secretos: &impl FuenteDeContrasenas,
    empresa: arles_core::ids::CompanyId,
    buzon: &BuzonQueSeLee,
    ahora: time::OffsetDateTime,
) -> bool {
    if !baja_por_correo::toca_revisar(buzon.ultimo_intento, ahora) {
        return false;
    }

    // Si esto falla no se revisa, y es lo correcto: sin marca, el intervalo no
    // limitaría nada y este buzón se intentaría en cada vuelta.
    if db
        .anotar_el_intento_del_buzon(empresa, buzon.cuenta, None)
        .is_err()
    {
        return false;
    }

    // `revisar_uno` es el mismo camino que recorre el botón: lee, pregunta al
    // núcleo y escribe, en ese orden. Aquí no se repite nada de esa lógica —si
    // se repitiera, el botón y el automatismo podrían dar resultados distintos
    // y el que se ve es el del botón.
    let fallo = match revisar_uno(db, lector, secretos, empresa, buzon) {
        // El buzón contestó. `no_se_pudo` lleva la clave si algo salió mal
        // dentro: una contraseña que no está, un servidor que rechaza.
        Ok(resultado) => resultado.no_se_pudo,
        // Y esto es un fallo de antes de hablar con nadie —el llavero—, que
        // llega como error en vez de dentro del resultado.
        //
        // ── SE GUARDA SU CLAVE, NO UNA ESCRITA AQUÍ ──
        //
        // La primera versión de esta línea inventaba
        // `error.app.buzon.no_se_pudo_entrar`, y mirar la pantalla la pilló:
        // la clave no existe, así que la pantalla enseñaba «Ocurrió un error
        // inesperado». Y lo que habría pasado con una clave de buzón que sí
        // existiera es peor: decirle a quien tiene el llavero bloqueado que
        // revise el servidor de su buzón — el diagnóstico equivocado de
        // DOC-14, que no hace dudar sino que manda a hacer algo que no toca.
        Err(e) => Some(e.clave),
    };

    // La segunda anotación: la misma marca de tiempo se reescribe, y lo que
    // cambia es el motivo. Sin esto la pantalla diría «revisado hace 3 min» de
    // un buzón que falla siempre.
    let _ = db.anotar_el_intento_del_buzon(empresa, buzon.cuenta, fallo.as_deref());
    true
}

/// Duerme el descanso en trozos, mirando si hay que parar.
///
/// Con un `sleep` entero, cerrar ARLES se quedaría pensando hasta un minuto —y
/// el usuario ve una ventana que no se cierra—. Es el mismo troceado que el
/// hilo del motor, y por el mismo motivo.
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
mod pruebas {
    use super::*;
    use arles_email::buzon::{CorreoDelBuzon, CuentaDeLectura};
    use arles_email::error::ErrorDeCorreo;
    use std::sync::Mutex;

    /// Un buzón que apunta cuántas veces se le pidió leer.
    ///
    /// Es lo único que esta prueba necesita medir: **cuántas conexiones se
    /// hacen**, que es el recurso que el intervalo protege.
    #[derive(Debug, Default)]
    struct BuzonQueCuenta {
        veces: Mutex<u32>,
        /// Si contesta con un error, para el caso del servidor caído.
        revienta: bool,
    }

    impl LeeUnBuzon for BuzonQueCuenta {
        fn correos_de(
            &self,
            _cuenta: &CuentaDeLectura,
            _contrasena: &str,
            _dias: u16,
        ) -> Result<Vec<CorreoDelBuzon>, ErrorDeCorreo> {
            *self.veces.lock().expect("sin envenenar") += 1;
            if self.revienta {
                return Err(ErrorDeCorreo::NoSePudoEntrarAlBuzon);
            }
            Ok(Vec::new())
        }
    }

    /// Un llavero que siempre da la misma contraseña.
    #[derive(Debug)]
    struct UnLlavero;

    impl FuenteDeContrasenas for UnLlavero {
        fn contrasena_de(
            &self,
            _referencia: &str,
        ) -> Result<Option<String>, crate::error::AppError> {
            Ok(Some("la-de-aplicacion".to_owned()))
        }

        /// El programador no toca WhatsApp: R-25 es del canal de correo. Aquí
        /// `None` es la respuesta correcta, no un hueco por rellenar.
        fn token_del_numero(
            &self,
            _id_numero: &str,
        ) -> Result<Option<String>, crate::error::AppError> {
            Ok(None)
        }
    }

    fn base_con_buzon() -> (tempfile::TempDir, Db, arles_core::ids::CompanyId) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = arles_db::ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        let empresa = db
            .guardar_empresa(
                &arles_core::empresa::DatosDeEmpresa::validar(
                    &arles_core::empresa::BorradorDeEmpresa {
                        nombre_comercial: "TELEMETRY".into(),
                        pais: "MX".into(),
                        zona_horaria: "America/Mexico_City".into(),
                        correo_corporativo: "hola@telemetry.mx".into(),
                        sitio_web: String::new(),
                    },
                )
                .expect("válidos"),
            )
            .expect("guarda");

        let cuenta = arles_core::ids::EmailAccountId::nuevo();
        let datos =
            arles_core::correo::DatosDeLaCuenta::validar(&arles_core::correo::BorradorDeCuenta {
                servidor: "smtp.telemetrymx.com".into(),
                puerto: "587".into(),
                cifrado: "starttls".into(),
                usuario: "enlace@telemetrymx.com".into(),
                remitente: "enlace@telemetrymx.com".into(),
                nombre_visible: "Enlace".into(),
            })
            .expect("válidos");
        db.alta_de_cuenta_de_correo(
            empresa,
            cuenta,
            &datos,
            arles_db::correo::LimitesDeLaCuenta::default(),
            "llavero://cuenta-1",
        )
        .expect("alta");
        db.configurar_la_lectura_del_buzon(empresa, cuenta, Some("imap.telemetrymx.com"), true)
            .expect("enciende");

        (dir, db, empresa)
    }

    /// **La primera vuelta revisa, la segunda no.**
    ///
    /// Es la prueba de que el intervalo se respeta de verdad, y lo que mide es
    /// el recurso que protege: cuántas conexiones se abren. Sin esto, un hilo
    /// que da una vuelta por minuto haría 60 conexiones por hora contra el
    /// servidor de correo del usuario en vez de 4.
    ///
    /// Verificada rompiéndola: quitando la pregunta a `toca_revisar`, la
    /// segunda vuelta también lee y la cuenta sale a 2.
    #[test]
    fn el_intervalo_se_respeta_entre_dos_vueltas() {
        let (_d, db, _empresa) = base_con_buzon();
        let buzon = BuzonQueCuenta::default();
        let parar = AtomicBool::new(false);

        assert_eq!(
            una_vuelta(&db, &buzon, &UnLlavero, &parar),
            1,
            "un buzón que nunca se revisó toca en la primera vuelta"
        );
        assert_eq!(*buzon.veces.lock().expect("sin envenenar"), 1);

        assert_eq!(
            una_vuelta(&db, &buzon, &UnLlavero, &parar),
            0,
            "un minuto después NO toca: el intervalo son quince"
        );
        assert_eq!(
            *buzon.veces.lock().expect("sin envenenar"),
            1,
            "la segunda vuelta no puede abrir otra conexión IMAP"
        );
    }

    /// **Un buzón cuyo servidor no contesta tampoco se reintenta.**
    ///
    /// Es el caso que la V19 existe para cubrir, y el que más daño hace: sin la
    /// marca escrita al fallar, este buzón se intentaría en cada vuelta hasta
    /// que el proveedor bloqueara la cuenta por demasiados intentos.
    ///
    /// Verificada rompiéndola, y lo que midió hay que decirlo entero: la
    /// ruptura que la caza es **anotar sólo en el éxito**, y entonces sale en
    /// rojo nombrando las tres vueltas.
    ///
    /// Lo que **no** caza —medido, no supuesto— es quitar la anotación previa
    /// dejando la de después: las cinco pruebas pasan en verde, porque la de
    /// después escribe la marca igual con fallo o sin él. La anotación previa
    /// protege otra cosa: que ARLES muera, o que alguien añada mañana un
    /// `return` temprano, **entre** la marca y la lectura. Eso no se puede
    /// ejercitar en una prueba sin matar el proceso.
    ///
    /// Así que su única red es el validador, que mide **la posición** de las
    /// dos llamadas dentro de este cuerpo. Dicho en vez de darlo por cubierto:
    /// lo que aquí vigila una prueba y lo que vigila el validador no es lo
    /// mismo, y suponer que la prueba cubre las dos cosas es cómo se llega a
    /// una garantía que nadie mide.
    #[test]
    fn un_servidor_caido_no_se_martillea() {
        let (_d, db, empresa) = base_con_buzon();
        let buzon = BuzonQueCuenta {
            revienta: true,
            ..BuzonQueCuenta::default()
        };
        let parar = AtomicBool::new(false);

        una_vuelta(&db, &buzon, &UnLlavero, &parar);
        una_vuelta(&db, &buzon, &UnLlavero, &parar);
        una_vuelta(&db, &buzon, &UnLlavero, &parar);

        assert_eq!(
            *buzon.veces.lock().expect("sin envenenar"),
            1,
            "tres vueltas y una sola conexión: un servidor que no contesta no \
             se reintenta hasta el intervalo siguiente"
        );

        // Y el motivo queda guardado, o la pantalla diría que todo va bien.
        let estado = db
            .como_estan_los_buzones(empresa)
            .expect("lee")
            .first()
            .cloned()
            .expect("hay buzón");
        assert!(
            estado.ultimo_fallo.is_some(),
            "un intento que falló tiene que dejar su motivo: si no, la \
             pantalla enseña «revisado hace un momento» de un buzón roto"
        );
        assert!(estado.ultimo_intento.is_some());
    }

    /// Sin empresa configurada no se revisa nada, y no es un error.
    ///
    /// Es el primer arranque, antes del alta. Un `unwrap` aquí tiraría el hilo
    /// del programador en la primera vuelta de una instalación nueva.
    #[test]
    fn el_primer_arranque_no_revienta() {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = arles_db::ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        let buzon = BuzonQueCuenta::default();

        assert_eq!(
            una_vuelta(&db, &buzon, &UnLlavero, &AtomicBool::new(false)),
            0
        );
        assert_eq!(*buzon.veces.lock().expect("sin envenenar"), 0);
    }

    /// **Cerrar ARLES no espera un minuto.**
    ///
    /// La misma garantía que el hilo del motor tiene desde la 5.4b, y por el
    /// mismo motivo: con un `sleep` entero, cerrar el programa se queda
    /// pensando hasta que el hilo termine su siesta.
    ///
    /// Verificada rompiéndola: con `thread::sleep(DESCANSO)` en vez del
    /// troceado, esta prueba falla.
    #[test]
    fn pedir_parar_durante_el_descanso_no_espera_al_descanso_entero() {
        let parar = Arc::new(AtomicBool::new(false));
        let bandera = Arc::clone(&parar);

        let hilo = thread::spawn(move || {
            let empezo = std::time::Instant::now();
            dormir_a_trozos(&bandera);
            empezo.elapsed()
        });

        thread::sleep(Duration::from_millis(150));
        parar.store(true, Ordering::Relaxed);
        let tardo = hilo.join().expect("el hilo del descanso termina");

        assert!(
            tardo < DESCANSO / 2,
            "pedir parar tardó {tardo:?} de un descanso de {DESCANSO:?}: \
             cerrar ARLES no puede esperar a que termine la siesta"
        );
    }

    /// **Soltar el programador lo para y espera al hilo.**
    ///
    /// Sin esto, cerrar ARLES dejaría un hilo revisando buzones con la base ya
    /// soltada.
    #[test]
    fn soltar_el_programador_termina_el_hilo() {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = arles_db::ClaveMaestra::generar().expect("genera");
        let db = Arc::new(Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre"));

        let programador = ProgramadorEnMarcha::arrancar(Arc::clone(&db));
        let empezo = std::time::Instant::now();
        drop(programador);

        assert!(
            empezo.elapsed() < DESCANSO,
            "soltar el programador tiene que parar el hilo, no esperarlo"
        );
    }
}
