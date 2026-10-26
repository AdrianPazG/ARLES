//! Estado compartido de la aplicación.

use std::sync::{Arc, Mutex};

use arles_db::Db;

use crate::error::AppError;
use crate::llavero::Llavero;
use arles_motor::instancia::Cerrojo;

use crate::motor::MotorEnMarcha;
use crate::rutas::Rutas;

/// Lo que los comandos comparten.
///
/// Nótese que **no aparece `rusqlite` por ninguna parte**: el shell habla con
/// `arles-db` mediante métodos tipados, no con el motor. Es lo que mantiene
/// acotadas las consultas y hace que la capa de datos sea auditable (ADR-0002).
#[derive(Debug)]
pub struct EstadoApp {
    /// La base, compartida con el hilo del motor.
    ///
    /// Es un `Arc` desde la 5.4b y no antes: el motor corre en su propio hilo
    /// —cerrar la ventana no detiene una campaña (§51)— y necesita la misma
    /// conexión, no otra. Abrir una segunda conexión sería peor de lo que
    /// parece: `Db` serializa las escrituras con un mutex justamente para que
    /// `SQLITE_BUSY` no exista (ADR-0002), y dos conexiones se saltan ese
    /// mutex.
    db: Arc<Db>,

    /// El motor, si está corriendo.
    ///
    /// `None` cuando no se pudo tomar el cerrojo de instancia única: hay otro
    /// ARLES abierto sobre esta misma base. **La aplicación sí arranca** en ese
    /// caso —se pueden mirar contactos, campañas y la configuración— y lo que
    /// no hace es enviar. Negar el arranque entero por eso sería peor: el
    /// segundo ARLES suele abrirse sin querer, con doble clic.
    ///
    /// Se para al soltarlo, así que cerrar la aplicación espera a que el hilo
    /// termine su envío en curso.
    motor: Option<MotorEnMarcha>,
    /// El archivo que se está importando, entre que se lee y se confirma.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ SE QUEDA AQUÍ Y NO SE VUELVE A LEER
    ///
    /// Importar son tres pasos: leer, analizar y confirmar. Entre el segundo y
    /// el tercero el usuario está mirando el informe de choques, y eso puede
    /// durar minutos.
    ///
    /// Volver a abrir el archivo en cada paso dejaría una ventana en la que
    /// alguien —o el propio Excel, que reescribe al guardar— puede cambiarlo
    /// debajo. Entonces el usuario aprobaría un informe de un archivo y ARLES
    /// importaría otro, sin que nada fallara.
    ///
    /// Se guarda **uno solo**: empezar una importación descarta la anterior.
    /// Dos a la vez no tienen sentido —hay una sola pantalla— y guardarlas
    /// todas sería una fuga de memoria con nombre propio.
    /// ─────────────────────────────────────────────────────────────────────
    importacion: Mutex<Option<arles_import::TablaLeida>>,

    /// El respaldo que se está revisando, entre que se elige y se confirma.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ LA RUTA VIVE AQUÍ Y NO VA Y VUELVE POR LA FRONTERA
    ///
    /// Restaurar son dos pasos: revisar —que dice qué trae el respaldo y qué se
    /// perdería— y confirmar. Entre los dos el usuario está leyendo cifras, y
    /// eso puede durar un minuto.
    ///
    /// Devolverle la ruta a la webview para que la mande de vuelta rompería la
    /// **regla 4 de la frontera**: ninguna ruta del sistema de archivos la
    /// cruza. Y además dejaría que la webview pidiera restaurar **otro**
    /// archivo distinto del que se revisó, que es la misma ventana que la
    /// importación cerró en la Fase 3.
    ///
    /// Se guarda una sola: revisar un respaldo descarta el anterior.
    /// ─────────────────────────────────────────────────────────────────────
    respaldo_elegido: Mutex<Option<std::path::PathBuf>>,

    /// Lo que ya se sabe del DNS de cada dominio (7.5).
    ///
    /// Vive en el estado y no en cada llamada porque el preflight y la
    /// pantalla de Remitentes hacen **la misma pregunta desde dos sitios**: con
    /// una caché por llamada, abrir el asistente y mirar el remitente darían
    /// dos respuestas distintas si el DNS cambia en medio, y además serían seis
    /// consultas de red en vez de tres.
    dominios: crate::entregabilidad::Dominios,
}

impl EstadoApp {
    /// Arranca: resuelve rutas, obtiene la clave del llavero, abre y migra.
    ///
    /// # Errores
    ///
    /// [`AppError::LlaveroNoDisponible`] si el almacén del sistema no responde.
    /// **La aplicación no arranca en ese caso**: seguir sin poder descifrar la
    /// base significaría funcionar sin los datos del usuario (ADR-0011).
    pub fn arrancar() -> Result<Self, AppError> {
        Self::arrancar_con(&Rutas::resolver()?, &Llavero::del_sistema())
    }

    /// Igual que [`Self::arrancar`], con rutas y llavero explícitos.
    ///
    /// Los tests inyectan un llavero con cuenta propia: comparten el almacén
    /// real de la máquina, y con una cuenta fija un test que borra la entrada
    /// rompe a otro que corre en paralelo.
    ///
    /// # Errores
    ///
    /// Ver [`Self::arrancar`].
    pub fn arrancar_con(rutas: &Rutas, llavero: &Llavero) -> Result<Self, AppError> {
        let base = rutas.base_de_datos();

        let clave = match llavero.leer()? {
            Some(c) => c,

            // No hay clave en el llavero pero **sí hay una base en disco**.
            //
            // Generar una clave nueva aquí sería lo cómodo y lo peor posible: la
            // base existente quedaría irrecuperable y el usuario solo vería «no
            // se pudo descifrar», sin enterarse de que lo que necesita es
            // restaurar un respaldo. Se nombra la situación en vez de taparla
            // (riesgo R-10).
            None if base.exists() => return Err(AppError::ClaveMaestraPerdida),

            // Primer arranque de verdad: no hay clave y tampoco hay base.
            None => llavero.crear()?,
        };

        let db = Arc::new(Db::abrir(&base, &clave)?);

        // El cerrojo de instancia única, y con él el motor. Si no se puede
        // tomar —hay otro ARLES sobre esta base— la aplicación arranca **sin
        // enviar**, que es lo que dice el campo `motor`.
        //
        // El identificador es el del proceso: lo bastante para que un ARLES
        // reconozca su propio cerrojo, y no dice nada de la máquina ni del
        // usuario en un archivo que se queda en disco.
        let motor = Cerrojo::tomar(rutas.datos(), &std::process::id().to_string())
            .ok()
            .map(|cerrojo| MotorEnMarcha::arrancar(Arc::clone(&db), cerrojo));

        Ok(Self {
            db,
            motor,
            importacion: Mutex::new(None),
            respaldo_elegido: Mutex::new(None),
            dominios: crate::entregabilidad::Dominios::default(),
        })
    }

    #[must_use]
    pub fn db(&self) -> &Db {
        &self.db
    }

    /// Lo que se sabe del DNS de cada dominio, compartido (7.5).
    #[must_use]
    pub const fn dominios(&self) -> &crate::entregabilidad::Dominios {
        &self.dominios
    }

    /// ¿Está el motor enviando?
    ///
    /// `false` significa que hay otro ARLES abierto sobre esta misma base. Es
    /// un hecho que la pantalla tiene que poder decir: una campaña «en marcha»
    /// que no avanza, sin explicación, se lee como una avería (§95).
    #[must_use]
    pub fn el_motor_esta_en_marcha(&self) -> bool {
        self.motor.is_some()
    }

    /// Guarda el archivo leído, descartando el anterior si lo hubiera.
    pub(crate) fn guardar_importacion(&self, tabla: arles_import::TablaLeida) {
        let mut hueco = self
            .importacion
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *hueco = Some(tabla);
    }

    /// Hace algo con el archivo en curso, sin sacarlo del estado.
    ///
    /// Se pasa un cierre en vez de devolver una copia: el archivo puede tener
    /// medio millón de filas, y clonarlo en cada paso multiplicaría por tres la
    /// memoria de la importación.
    pub(crate) fn con_importacion<T>(
        &self,
        f: impl FnOnce(&arles_import::TablaLeida) -> T,
    ) -> Option<T> {
        let hueco = self
            .importacion
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        hueco.as_ref().map(f)
    }

    /// Suelta el archivo en curso.
    ///
    /// Se llama al confirmar y al cancelar. Sin esto, medio millón de filas se
    /// quedarían en memoria hasta cerrar ARLES.
    /// Guarda el respaldo que se acaba de elegir, descartando el anterior.
    pub(crate) fn guardar_respaldo_elegido(&self, ruta: std::path::PathBuf) {
        let mut hueco = self
            .respaldo_elegido
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *hueco = Some(ruta);
    }

    /// La ruta del respaldo en revisión, si hay una.
    pub(crate) fn respaldo_elegido(&self) -> Option<std::path::PathBuf> {
        self.respaldo_elegido
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Olvida el respaldo elegido.
    ///
    /// Se llama al terminar de restaurar, bien o mal: dejar la ruta puesta
    /// permitiría restaurar dos veces seguidas sin volver a revisar, y la
    /// segunda vez el preflight que el usuario leyó ya no describiría la base.
    pub(crate) fn soltar_respaldo_elegido(&self) {
        let mut hueco = self
            .respaldo_elegido
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *hueco = None;
    }

    pub(crate) fn soltar_importacion(&self) {
        let mut hueco = self
            .importacion
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *hueco = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un llavero con nombre propio: las pruebas comparten el almacén real de
    /// la máquina y una cuenta fija haría que una borrara la entrada de otra.
    fn llavero(nombre: &str) -> Llavero {
        Llavero::con_cuenta(format!("prueba-estado-{nombre}-{}", std::process::id()))
    }

    /// Al arrancar, el motor queda en marcha: ARLES envía.
    ///
    /// Es la garantía que esta entrega añade. Hasta la 5.4b el motor estaba
    /// construido y probado entero y **nadie lo arrancaba**.
    #[test]
    fn al_arrancar_el_motor_queda_en_marcha() {
        let dir = tempfile::tempdir().expect("temporal");
        let l = llavero("solo");
        let Ok(estado) = EstadoApp::arrancar_con(&Rutas::en(dir.path().to_path_buf()), &l) else {
            println!("RAMA: sin llavero, no se puede arrancar la aplicación entera");
            return;
        };
        println!("RAMA: llavero disponible, arranque completo ejercitado");

        assert!(
            estado.el_motor_esta_en_marcha(),
            "sin motor, ARLES abre y no envía nunca"
        );
        drop(estado);
        let _ = l.borrar();
    }

    /// **El segundo ARLES sobre la misma base abre, pero no envía.**
    ///
    /// Las dos mitades importan y son dos decisiones distintas:
    ///
    /// **No envía**, porque dos motores sobre la misma base se pisan al
    /// arrancar: el segundo ordena la cola y marca como «envío no confirmado»
    /// lo que el primero está mandando en ese momento. El resultado no es un
    /// duplicado, es una lista de envíos que el usuario tiene que revisar uno a
    /// uno cuando todos salieron bien.
    ///
    /// **Y sí abre**, porque el segundo ARLES suele abrirse sin querer, con un
    /// doble clic. Negarle el arranque entero dejaría a alguien sin poder mirar
    /// sus contactos por haber pulsado dos veces.
    ///
    /// Verificada rompiéndola el 28/09/2026: arrancando el motor sin pedir el
    /// cerrojo, esta prueba falla.
    #[test]
    fn el_segundo_arles_sobre_la_misma_base_abre_pero_no_envia() {
        let dir = tempfile::tempdir().expect("temporal");
        let rutas = Rutas::en(dir.path().to_path_buf());
        let l = llavero("dos");

        let Ok(primero) = EstadoApp::arrancar_con(&rutas, &l) else {
            println!("RAMA: sin llavero, no se puede arrancar la aplicación entera");
            return;
        };
        println!("RAMA: llavero disponible, dos arranques ejercitados");
        assert!(primero.el_motor_esta_en_marcha());

        let segundo = EstadoApp::arrancar_con(&rutas, &l).expect("el segundo también abre");
        assert!(
            !segundo.el_motor_esta_en_marcha(),
            "dos motores sobre la misma base marcan como no confirmados los \
             envíos que el otro está haciendo ahora mismo"
        );

        drop(segundo);
        drop(primero);
        let _ = l.borrar();
    }

    /// Y cuando el primero cierra, el siguiente sí envía.
    ///
    /// Sin esto, la prueba de arriba pasaría igual con un motor que no arranca
    /// nunca: hay que ver el cerrojo soltarse y a alguien tomarlo.
    #[test]
    fn cuando_el_primero_cierra_el_siguiente_si_envia() {
        let dir = tempfile::tempdir().expect("temporal");
        let rutas = Rutas::en(dir.path().to_path_buf());
        let l = llavero("relevo");

        let Ok(primero) = EstadoApp::arrancar_con(&rutas, &l) else {
            println!("RAMA: sin llavero, no se puede arrancar la aplicación entera");
            return;
        };
        println!("RAMA: llavero disponible, relevo ejercitado");
        assert!(primero.el_motor_esta_en_marcha());
        drop(primero);

        let siguiente = EstadoApp::arrancar_con(&rutas, &l).expect("arranca");
        assert!(
            siguiente.el_motor_esta_en_marcha(),
            "al cerrar bien, el cerrojo queda libre para el siguiente"
        );
        drop(siguiente);
        let _ = l.borrar();
    }
}
