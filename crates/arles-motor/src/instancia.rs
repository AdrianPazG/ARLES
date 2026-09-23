//! El cerrojo de instancia única: un solo ARLES sobre una sola base.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ PASA SI SE ABREN DOS, Y POR QUÉ NO BASTA EL COMPARE-AND-SWAP
//!
//! La cola ya impide que dos trabajadores se lleven el mismo envío: ésa es la
//! parte fácil, y está resuelta en `arles_db::cola`.
//!
//! El daño de verdad lo hace el **arranque**. Al abrir, ARLES ordena la cola:
//! todo lo que quedó en `sending` pasa a «envío no confirmado», porque sólo
//! una caída deja filas así (ADR-0004). Si un segundo ARLES arranca mientras
//! el primero está enviando, encuentra filas en `sending` que **no son restos
//! de nada** —son los envíos que el otro está haciendo ahora mismo— y las
//! marca como no confirmadas.
//!
//! El resultado no es un duplicado. Es peor de leer: una lista de envíos que
//! el usuario tiene que revisar uno a uno para decidir si reenviarlos, cuando
//! todos salieron bien. La confianza en esa lista se pierde entera.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! CÓMO SE HACE, Y QUÉ NO SE PROMETE
//!
//! Un archivo al lado de la base, con un identificador y un **latido**: la
//! hora, reescrita cada pocos segundos por el bucle del trabajador. Por eso
//! este cerrojo llega con el bucle y no antes: sin algo que lata, un ARLES que
//! muere de golpe deja el archivo puesto y **nadie puede volver a abrir el
//! programa**. Ése es un fallo mucho peor que el que se quiere evitar.
//!
//! Con latido, el arranque distingue:
//!
//! ```text
//! el latido es reciente  ->  hay otro ARLES vivo. No se abre.
//! el latido esta viejo   ->  el otro murio. Se toma el cerrojo.
//! no hay archivo         ->  no hay nadie. Se toma el cerrojo.
//! ```
//!
//! **Lo que no se promete.** Esto no es un cerrojo del sistema operativo. Dos
//! ARLES que arrancaran en el mismo milisegundo sobre un cerrojo caducado
//! podrían creerse los dos dueños. Por eso al robar un cerrojo viejo se escribe
//! y se **vuelve a leer** tras una pausa: quien lea un identificador que no es
//! el suyo, se retira. La ventana no queda cerrada por demostración, queda
//! cerrada en la práctica — y el caso exige dos arranques simultáneos justo
//! después de una caída.
//!
//! Lo que sí es una garantía dura sigue siendo la de siempre: el
//! `UNIQUE(campaign_id, channel, contact_address)` del esquema. Nadie recibe
//! dos correos aunque este cerrojo falle.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Cuánto puede llevar un latido sin refrescarse antes de darlo por muerto.
///
/// Noventa segundos es holgado a propósito. El bucle late cada pocos segundos,
/// así que un ARLES vivo nunca se acerca; el margen es para el equipo que se
/// queda pensando —un disco lento, una copia de seguridad, el propio sistema
/// suspendido un momento— y para que nunca se le robe el cerrojo a alguien que
/// simplemente iba despacio.
pub const LATIDO_MAXIMO: Duration = Duration::from_secs(90);

/// Cuánto se espera antes de confirmar que un cerrojo robado sigue siendo
/// nuestro. Ver la nota de arriba sobre lo que no se promete.
pub const ESPERA_DE_CONFIRMACION: Duration = Duration::from_millis(50);

/// Cómo se llama el archivo, al lado de la base de datos.
pub const NOMBRE: &str = "arles.lock";

/// Por qué no se pudo tomar el cerrojo.
///
/// Cada variante lleva su clave de texto (§139), y las dos dicen las tres
/// cosas del §95 en la pantalla que las enseña: qué pasó, cómo arreglarlo y
/// qué está a salvo.
#[derive(Debug)]
pub enum NoSePudoTomar {
    /// Hay otro ARLES abierto sobre esta misma base, y está vivo.
    YaHayOtroAbierto {
        /// Hace cuántos segundos latió. Es una cifra, no un adjetivo (§94).
        latio_hace_s: u64,
    },
    /// Se tomó y otro lo tomó a la vez. Se retira éste.
    LoTomoOtroALaVez,
    /// No se pudo leer ni escribir el archivo.
    NoSePudoEscribir(io::Error),
}

impl NoSePudoTomar {
    /// La clave de texto (§139).
    #[must_use]
    pub const fn clave_i18n(&self) -> &'static str {
        match self {
            Self::YaHayOtroAbierto { .. } => "instancia.yaHayOtroAbierto",
            Self::LoTomoOtroALaVez => "instancia.loTomoOtroALaVez",
            Self::NoSePudoEscribir(_) => "instancia.noSePudoEscribir",
        }
    }
}

impl std::fmt::Display for NoSePudoTomar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.clave_i18n())
    }
}

impl std::error::Error for NoSePudoTomar {}

/// El cerrojo tomado. Mientras exista, este proceso es el dueño.
///
/// Al soltarlo —o al terminar el programa— se borra el archivo. Si el programa
/// muere sin soltarlo, el archivo se queda, y es el latido el que permite
/// distinguir «se quedó puesto» de «hay alguien dentro».
#[derive(Debug)]
pub struct Cerrojo {
    ruta: PathBuf,
    /// Quién es el dueño. Se comprueba antes de borrar, para no borrar el
    /// cerrojo de otro que nos lo hubiera robado.
    quien: String,
}

impl Cerrojo {
    /// Toma el cerrojo del directorio donde vive la base.
    ///
    /// # Errores
    ///
    /// [`NoSePudoTomar`] con el motivo, que la pantalla enseña tal cual.
    pub fn tomar(directorio: &Path, quien: &str) -> Result<Self, NoSePudoTomar> {
        let ruta = directorio.join(NOMBRE);

        if let Some(edad) = edad_del_latido(&ruta)? {
            if edad <= LATIDO_MAXIMO {
                return Err(NoSePudoTomar::YaHayOtroAbierto {
                    latio_hace_s: edad.as_secs(),
                });
            }
            // El cerrojo está caducado: su dueño murió sin soltarlo. Se roba,
            // y se confirma.
            escribir(&ruta, quien)?;
            std::thread::sleep(ESPERA_DE_CONFIRMACION);
            if leer_duenio(&ruta)?.as_deref() != Some(quien) {
                return Err(NoSePudoTomar::LoTomoOtroALaVez);
            }
        } else {
            escribir(&ruta, quien)?;
        }

        Ok(Self {
            ruta,
            quien: quien.to_owned(),
        })
    }

    /// Refresca el latido. **Lo llama el bucle del trabajador.**
    ///
    /// Si esto dejara de llamarse, al cabo de [`LATIDO_MAXIMO`] otro ARLES
    /// podría abrir la misma base creyendo que éste murió. Por eso el latido no
    /// va en un temporizador aparte sino en el bucle: si el bucle no corre, el
    /// programa no está enviando, y que otro tome el relevo es correcto.
    ///
    /// # Errores
    ///
    /// [`NoSePudoTomar::LoTomoOtroALaVez`] si el cerrojo ya no es nuestro, que
    /// es la señal de que este proceso tiene que dejar de enviar.
    /// [`NoSePudoTomar::NoSePudoEscribir`] si el archivo no se puede escribir.
    pub fn latir(&self) -> Result<(), NoSePudoTomar> {
        if leer_duenio(&self.ruta)?.as_deref() != Some(self.quien.as_str()) {
            return Err(NoSePudoTomar::LoTomoOtroALaVez);
        }
        escribir(&self.ruta, &self.quien)
    }

    /// Quién dice el archivo que es el dueño.
    #[must_use]
    pub fn quien(&self) -> &str {
        &self.quien
    }
}

impl Drop for Cerrojo {
    /// Suelta el cerrojo, **si sigue siendo nuestro**.
    ///
    /// La comprobación no sobra: si otro ARLES nos lo robó por un latido que no
    /// llegó a tiempo, borrar aquí dejaría la base sin cerrojo teniendo dentro
    /// a alguien enviando. Un fallo silencioso al borrar es preferible a eso, y
    /// el archivo que se quede caduca solo.
    fn drop(&mut self) {
        if leer_duenio(&self.ruta).ok().flatten().as_deref() == Some(self.quien.as_str()) {
            let _ = fs::remove_file(&self.ruta);
        }
    }
}

/// Escribe el archivo entero: quién es el dueño y cuándo latió.
///
/// La hora del latido **no se guarda dentro**: se lee de la fecha de
/// modificación del archivo. Guardarla dentro obligaría a confiar en que los
/// dos procesos leen el mismo reloj de la misma manera, y a analizar un texto
/// que puede venir corrupto de una escritura a medias.
fn escribir(ruta: &Path, quien: &str) -> Result<(), NoSePudoTomar> {
    fs::write(ruta, quien).map_err(NoSePudoTomar::NoSePudoEscribir)
}

/// Quién figura como dueño, o `None` si no hay archivo.
fn leer_duenio(ruta: &Path) -> Result<Option<String>, NoSePudoTomar> {
    match fs::read_to_string(ruta) {
        Ok(texto) => Ok(Some(texto.trim().to_owned())),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(NoSePudoTomar::NoSePudoEscribir(e)),
    }
}

/// Cuánto hace que se escribió el archivo, o `None` si no existe.
///
/// Un archivo con fecha **en el futuro** —el reloj del equipo se movió— cuenta
/// como recién latido: asumir que está vivo hace esperar de más, y asumir lo
/// contrario abre una segunda instancia. Se falla del lado que no rompe nada.
fn edad_del_latido(ruta: &Path) -> Result<Option<Duration>, NoSePudoTomar> {
    let datos = match fs::metadata(ruta) {
        Ok(d) => d,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(NoSePudoTomar::NoSePudoEscribir(e)),
    };
    let modificado = datos.modified().map_err(NoSePudoTomar::NoSePudoEscribir)?;
    Ok(Some(
        SystemTime::now()
            .duration_since(modificado)
            .unwrap_or(Duration::ZERO),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Envejece el latido a mano, tocando la fecha de modificación.
    ///
    /// Esperar noventa segundos de verdad haría la prueba inútil: una suite que
    /// tarda minuto y medio por caso se acaba borrando, y entonces esto deja de
    /// estar probado.
    fn envejecer(ruta: &Path, segundos: u64) {
        let archivo = fs::File::options()
            .write(true)
            .open(ruta)
            .expect("abre para tocar la fecha");
        archivo
            .set_modified(SystemTime::now() - Duration::from_secs(segundos))
            .expect("atrasa la fecha");
    }

    /// Adelanta el latido al futuro: el caso del reloj del equipo movido.
    fn adelantar(ruta: &Path, segundos: u64) {
        let archivo = fs::File::options()
            .write(true)
            .open(ruta)
            .expect("abre para tocar la fecha");
        archivo
            .set_modified(SystemTime::now() + Duration::from_secs(segundos))
            .expect("adelanta la fecha");
    }

    /// **Dos ARLES no abren la misma base.**
    ///
    /// Es la prueba de por qué existe este archivo: el segundo arranque, con el
    /// primero vivo, se tiene que negar.
    #[test]
    fn el_segundo_arles_no_abre_la_misma_base() {
        let dir = tempfile::tempdir().expect("temporal");
        let primero = Cerrojo::tomar(dir.path(), "uno").expect("el primero entra");

        let segundo = Cerrojo::tomar(dir.path(), "dos");
        assert!(
            matches!(segundo, Err(NoSePudoTomar::YaHayOtroAbierto { .. })),
            "con un ARLES vivo, el segundo no puede abrir: al arrancar ordena \
             la cola y marcaría como «no confirmados» los envíos que el \
             primero está haciendo ahora mismo"
        );
        drop(primero);
    }

    /// Al cerrar bien, el siguiente entra sin esperar nada.
    #[test]
    fn al_cerrar_el_cerrojo_queda_libre() {
        let dir = tempfile::tempdir().expect("temporal");
        drop(Cerrojo::tomar(dir.path(), "uno").expect("entra"));
        assert!(
            !dir.path().join(NOMBRE).exists(),
            "cerrar bien tiene que dejar la puerta abierta"
        );
        Cerrojo::tomar(dir.path(), "dos").expect("el siguiente entra");
    }

    /// **Un ARLES que murió de golpe no deja el programa inservible.**
    ///
    /// Es el fallo que un cerrojo **sin** latido produce, y es peor que el que
    /// evita: el archivo se queda puesto y nadie puede volver a abrir ARLES
    /// nunca más sin saber que hay que borrar un archivo a mano.
    ///
    /// Aquí se simula la muerte súbita —el archivo queda, su dueño no— y se
    /// comprueba que el siguiente arranque entra.
    ///
    /// Verificada rompiéndola el 23/09/2026: haciendo que `tomar` devuelva
    /// `YaHayOtroAbierto` con cualquier archivo presente, esta prueba falla y
    /// `el_segundo_arles_no_abre_la_misma_base` sigue pasando — que es
    /// exactamente el par de casos que hay que distinguir.
    #[test]
    fn un_cerrojo_de_un_arles_muerto_no_bloquea_para_siempre() {
        let dir = tempfile::tempdir().expect("temporal");
        let ruta = dir.path().join(NOMBRE);

        // Muerte súbita: el archivo se queda, nadie lo suelta.
        escribir(&ruta, "el-que-murio").expect("escribe");
        envejecer(&ruta, LATIDO_MAXIMO.as_secs() + 60);

        let nuevo = Cerrojo::tomar(dir.path(), "el-que-llega")
            .expect("un cerrojo caducado no puede dejar ARLES inservible");
        assert_eq!(nuevo.quien(), "el-que-llega");
    }

    /// Y uno que lleva poco sin latir **no** se roba: su dueño está vivo.
    #[test]
    fn un_latido_reciente_no_se_roba() {
        let dir = tempfile::tempdir().expect("temporal");
        let ruta = dir.path().join(NOMBRE);
        escribir(&ruta, "el-que-va-despacio").expect("escribe");
        // Por debajo del margen: un equipo lento, no un equipo muerto.
        envejecer(&ruta, LATIDO_MAXIMO.as_secs() - 10);

        assert!(
            matches!(
                Cerrojo::tomar(dir.path(), "impaciente"),
                Err(NoSePudoTomar::YaHayOtroAbierto { .. })
            ),
            "ir despacio no es haber muerto: robarle el cerrojo a un ARLES \
             vivo pone dos a enviar sobre la misma base"
        );
    }

    /// Un latido refresca la edad y el cerrojo sigue siendo del mismo.
    #[test]
    fn latir_mantiene_el_cerrojo() {
        let dir = tempfile::tempdir().expect("temporal");
        let c = Cerrojo::tomar(dir.path(), "uno").expect("entra");
        c.latir().expect("late");
        assert_eq!(c.quien(), "uno");
        assert_eq!(
            leer_duenio(&dir.path().join(NOMBRE))
                .expect("lee")
                .as_deref(),
            Some("uno")
        );
    }

    /// **Si alguien nos robó el cerrojo, dejamos de enviar.**
    ///
    /// No es un caso de laboratorio: pasa si este proceso se queda congelado
    /// más de [`LATIDO_MAXIMO`] —un disco que no responde, el equipo
    /// suspendido— y otro ARLES arranca creyéndolo muerto. Al volver en sí,
    /// este proceso tiene que **enterarse y callarse**, no seguir enviando en
    /// paralelo con el otro.
    #[test]
    fn si_nos_robaron_el_cerrojo_el_latido_lo_dice() {
        let dir = tempfile::tempdir().expect("temporal");
        let c = Cerrojo::tomar(dir.path(), "uno").expect("entra");

        // Otro proceso se lo lleva.
        escribir(&dir.path().join(NOMBRE), "dos").expect("escribe");

        assert!(
            matches!(c.latir(), Err(NoSePudoTomar::LoTomoOtroALaVez)),
            "un proceso que perdió el cerrojo tiene que enterarse al latir; si \
             no, dos ARLES envían a la vez sobre la misma base"
        );
    }

    /// Y al soltarlo no borra el cerrojo del otro.
    #[test]
    fn soltar_un_cerrojo_robado_no_borra_el_del_otro() {
        let dir = tempfile::tempdir().expect("temporal");
        let c = Cerrojo::tomar(dir.path(), "uno").expect("entra");
        escribir(&dir.path().join(NOMBRE), "dos").expect("escribe");
        drop(c);

        assert_eq!(
            leer_duenio(&dir.path().join(NOMBRE))
                .expect("lee")
                .as_deref(),
            Some("dos"),
            "borrar aquí dejaría la base sin cerrojo teniendo a otro dentro \
             enviando"
        );
    }

    /// Un archivo con fecha en el futuro no revienta ni abre una segunda
    /// instancia: cuenta como recién latido.
    #[test]
    fn un_reloj_que_se_movio_no_abre_una_segunda_instancia() {
        let dir = tempfile::tempdir().expect("temporal");
        let _c = Cerrojo::tomar(dir.path(), "uno").expect("entra");
        adelantar(&dir.path().join(NOMBRE), 3_600);
        assert!(
            matches!(
                Cerrojo::tomar(dir.path(), "dos"),
                Err(NoSePudoTomar::YaHayOtroAbierto { .. })
            ),
            "ante una fecha rara, no abrir es el lado que no rompe nada"
        );
    }
}
