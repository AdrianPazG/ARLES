//! Un proveedor de mentira al que se le puede ordenar cómo fallar.
//!
//! Ver `MOTOR_DE_EJECUCION.md` §11 y los casos de red de `ESTRATEGIA_QA.md` §3.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTO ES IMPRESCINDIBLE Y NO UN LUJO
//!
//! Las decisiones más delicadas del motor son todas respuestas a un fallo: qué
//! hacer si se cae la red, si la contraseña caducó, si el servidor pide
//! esperar, si la dirección no existe.
//!
//! Y **no se le puede pedir a Gmail que se caiga a voluntad**. Sin un proveedor
//! al que ordenarle que falle, esas ramas del código sólo se ejercitarían el
//! día que fallaran de verdad — en mitad de una campaña de un cliente, que es
//! el peor momento posible para descubrir que la rama estaba mal escrita.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! Y ADEMÁS DEJA CONSTANCIA DE LO QUE LE PIDIERON
//!
//! El simulador **guarda cada mensaje que se le mandó**. Eso permite afirmar lo
//! que de otra forma habría que suponer: que a una persona dada de baja no se
//! la llamó ni una vez, que una parada de emergencia no dejó salir ninguno
//! más, que un duplicado no se pidió dos veces.
//!
//! Comprobar el estado final de la base diría «no consta un segundo envío».
//! Comprobar lo que recibió el proveedor dice **«no se pidió»**, que es la
//! afirmación que de verdad importa: lo que llega al proveedor ya no se puede
//! retirar.

use std::sync::Mutex;
use std::time::Duration;

use arles_core::reintento::Fallo;

use crate::proveedor::{Aceptado, Mensaje, Proveedor, recortar_respuesta};

/// Qué contestará el simulador la próxima vez.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Respuesta {
    /// Acepta, con el identificador que devolvería un servidor real.
    Acepta(&'static str),
    /// Acepta, pero contestando basura. Sirve para comprobar que el texto que
    /// viene de fuera se recorta y se limpia antes de guardarse.
    AceptaDiciendo(String),
    /// Se cayó la red, saltó el tiempo de espera, el servidor está saturado.
    SeCayoLaRed,
    /// El servidor tardó más de la cuenta. Para el motor es lo mismo que la
    /// red caída, y **eso es lo que hay que comprobar**: que no se trate como
    /// un fallo definitivo por venir de otro sitio.
    TardaDemasiado,
    /// «No me mandes nada durante N.»
    EsperaDurante(Duration),
    /// Esa dirección no existe.
    NoExisteEsaDireccion,
    /// La contraseña de aplicación caducó, o el permiso fue revocado.
    LaCredencialNoSirve,
    /// El proveedor contesta que **esa persona pidió la baja**.
    ///
    /// Pasa de verdad, y sobre todo en WhatsApp: quien bloquea o responde
    /// «STOP» se lo dice a Meta, no a nosotros, y nos enteramos al intentarlo.
    /// Para el motor **no es un fallo** —no se reintenta y no cuenta como
    /// error—, pero sí obliga a dos cosas: cerrar el intento y meter esa
    /// dirección en la lista de bajas.
    PidioLaBaja,
}

impl Respuesta {
    /// Traduce la respuesta a lo que el motor entiende.
    fn resolver(&self) -> Result<Aceptado, Fallo> {
        match self {
            Self::Acepta(id) => Ok(Aceptado {
                id_del_proveedor: Some(recortar_respuesta(id)),
            }),
            Self::AceptaDiciendo(bruto) => Ok(Aceptado {
                id_del_proveedor: Some(recortar_respuesta(bruto)),
            }),
            // Un tiempo de espera agotado y una red caída llegan por caminos
            // distintos y significan lo mismo: no se pudo, pero puede que sí
            // más tarde. Tratar el primero como definitivo dejaría sin enviar a
            // todo el que estuviera detrás de un servidor lento.
            Self::SeCayoLaRed | Self::TardaDemasiado => Err(Fallo::Pasajero),
            Self::EsperaDurante(d) => Err(Fallo::NoMeMandesNadaDurante(*d)),
            Self::NoExisteEsaDireccion => Err(Fallo::LaDireccionNoExiste),
            Self::LaCredencialNoSirve => Err(Fallo::LaCuentaNoAutentica),
            Self::PidioLaBaja => Err(Fallo::EsaPersonaPidioLaBaja),
        }
    }
}

/// Lo que el simulador recibió, para poder afirmar qué se pidió y qué no.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pedido {
    pub para: String,
    pub clave_de_idempotencia: String,
    /// Lo que se pidió mandar, **tal como llegó**.
    ///
    /// Está aquí para poder afirmar que el cuerpo de la plantilla llega al
    /// proveedor con los datos de esa persona puestos. Antes del paso 2b no
    /// había nada que afirmar: el contenido lo guardaba el adaptador y el
    /// mensaje no lo llevaba, así que **ninguna prueba podía ver** que una
    /// campaña mandaba un cuerpo vacío (H-15).
    pub asunto: String,
    pub html: String,
}

/// El proveedor de mentira.
#[derive(Debug)]
pub struct ProveedorSimulado {
    /// Lo que contestará, en orden. Se consume de la cabeza.
    guion: Mutex<Vec<Respuesta>>,
    /// Cuando el guion se acaba.
    ///
    /// Por defecto **acepta**: así una prueba que sólo quiere ver el camino
    /// feliz no tiene que escribir un guion de mil entradas.
    al_acabarse: Respuesta,
    /// Todo lo que se le pidió entregar, en orden.
    recibidos: Mutex<Vec<Pedido>>,
}

impl Default for ProveedorSimulado {
    fn default() -> Self {
        Self::que_siempre_acepta()
    }
}

impl ProveedorSimulado {
    /// Acepta todo lo que le llegue. El camino feliz.
    #[must_use]
    pub fn que_siempre_acepta() -> Self {
        Self {
            guion: Mutex::new(Vec::new()),
            al_acabarse: Respuesta::Acepta("simulado"),
            recibidos: Mutex::new(Vec::new()),
        }
    }

    /// Contesta lo que diga el guion, en orden, y después acepta.
    ///
    /// Un guion y no una respuesta fija porque los casos que importan son
    /// **secuencias**: falla, falla, y a la tercera acepta. Con una respuesta
    /// fija no se puede probar que un reintento llegue a buen puerto.
    #[must_use]
    pub fn con_guion(guion: Vec<Respuesta>) -> Self {
        Self {
            guion: Mutex::new(guion),
            al_acabarse: Respuesta::Acepta("simulado"),
            recibidos: Mutex::new(Vec::new()),
        }
    }

    /// Contesta siempre lo mismo, para siempre.
    #[must_use]
    pub fn que_siempre(respuesta: Respuesta) -> Self {
        Self {
            guion: Mutex::new(Vec::new()),
            al_acabarse: respuesta,
            recibidos: Mutex::new(Vec::new()),
        }
    }

    /// Todo lo que se le pidió entregar, en orden.
    #[must_use]
    pub fn recibidos(&self) -> Vec<Pedido> {
        self.recibidos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Cuántas veces se le pidió entregar algo.
    #[must_use]
    pub fn cuantos_pedidos(&self) -> usize {
        self.recibidos().len()
    }

    /// ¿Se le pidió entregar algo a esta dirección?
    ///
    /// Es la pregunta que hacen las pruebas de supresión y de parada: no «¿se
    /// envió?», sino **«¿se pidió?»**. Lo que llega al proveedor ya no se puede
    /// retirar.
    #[must_use]
    pub fn le_pidieron_enviar_a(&self, direccion: &str) -> bool {
        self.recibidos().iter().any(|p| p.para == direccion)
    }
}

impl Proveedor for ProveedorSimulado {
    fn enviar(&self, mensaje: &Mensaje<'_>) -> Result<Aceptado, Fallo> {
        // Se anota ANTES de decidir la respuesta. Si se anotara después, un
        // fallo dejaría el pedido sin registrar, y entonces «no se le pidió a
        // nadie» y «se le pidió y falló» serían indistinguibles — que es justo
        // la diferencia que este registro existe para poder afirmar.
        self.recibidos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(Pedido {
                para: mensaje.para.to_owned(),
                clave_de_idempotencia: mensaje.clave_de_idempotencia.to_owned(),
                asunto: mensaje.asunto.to_owned(),
                html: mensaje.html.to_owned(),
            });

        let siguiente = {
            let mut guion = self
                .guion
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if guion.is_empty() {
                self.al_acabarse.clone()
            } else {
                guion.remove(0)
            }
        };
        siguiente.resolver()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mensaje<'a>(para: &'a str) -> Mensaje<'a> {
        Mensaje {
            para,
            asunto: "Aviso",
            html: "<p>Hola</p>",
            de_una_campana: true,
            clave_de_idempotencia: "clave-1",
        }
    }

    /// El camino feliz: acepta y devuelve su identificador.
    #[test]
    fn por_defecto_acepta() {
        let p = ProveedorSimulado::que_siempre_acepta();
        let r = p.enviar(&mensaje("ana@x.mx")).expect("acepta");
        assert_eq!(r.id_del_proveedor.as_deref(), Some("simulado"));
    }

    /// **Las seis formas de fallar, cada una traducida a lo que el motor
    /// entiende.**
    ///
    /// Es la razón de existir del simulador: estas seis ramas del motor sólo se
    /// ejercitarían, sin él, el día que fallaran de verdad — en mitad de la
    /// campaña de un cliente.
    #[test]
    fn las_seis_formas_de_fallar_llegan_como_deben() {
        let diez_minutos = Duration::from_secs(600);
        let casos = [
            (Respuesta::SeCayoLaRed, Err(Fallo::Pasajero)),
            (Respuesta::TardaDemasiado, Err(Fallo::Pasajero)),
            (
                Respuesta::EsperaDurante(diez_minutos),
                Err(Fallo::NoMeMandesNadaDurante(diez_minutos)),
            ),
            (
                Respuesta::NoExisteEsaDireccion,
                Err(Fallo::LaDireccionNoExiste),
            ),
            (
                Respuesta::LaCredencialNoSirve,
                Err(Fallo::LaCuentaNoAutentica),
            ),
        ];

        for (respuesta, esperado) in casos {
            let p = ProveedorSimulado::que_siempre(respuesta.clone());
            assert_eq!(
                p.enviar(&mensaje("ana@x.mx")),
                esperado,
                "«{respuesta:?}» no llega al motor como debería"
            );
        }
    }

    /// **Tardar demasiado es pasajero, no definitivo.**
    ///
    /// Llega por un camino distinto que la red caída y significa lo mismo: no
    /// se pudo, pero puede que sí más tarde.
    ///
    /// Tratarlo como definitivo dejaría sin enviar a todo el que estuviera
    /// detrás de un servidor lento, y los daría de baja para siempre — porque
    /// un fallo definitivo de dirección mete a esa persona en la lista de
    /// bajas.
    #[test]
    fn tardar_demasiado_no_es_un_fallo_definitivo() {
        let p = ProveedorSimulado::que_siempre(Respuesta::TardaDemasiado);
        assert_eq!(
            p.enviar(&mensaje("ana@x.mx")),
            Err(Fallo::Pasajero),
            "un servidor lento no es una dirección inexistente: si se \
             confundieran, a esa persona se la daría de baja para siempre"
        );
    }

    /// El guion se consume en orden, y después se acepta.
    ///
    /// Es lo que permite probar una secuencia: falla, falla, y a la tercera
    /// entra. Con una respuesta fija no se puede comprobar que un reintento
    /// llegue a buen puerto.
    #[test]
    fn el_guion_se_consume_en_orden_y_despues_acepta() {
        let p = ProveedorSimulado::con_guion(vec![Respuesta::SeCayoLaRed, Respuesta::SeCayoLaRed]);

        assert_eq!(p.enviar(&mensaje("a@x.mx")), Err(Fallo::Pasajero));
        assert_eq!(p.enviar(&mensaje("a@x.mx")), Err(Fallo::Pasajero));
        assert!(
            p.enviar(&mensaje("a@x.mx")).is_ok(),
            "agotado el guion, el simulador acepta: el reintento llega a buen puerto"
        );
    }

    /// **Se anota lo que se pidió, aunque falle.**
    ///
    /// Es la propiedad que hace útil al simulador. «No consta un segundo
    /// envío» y «no se pidió un segundo envío» son afirmaciones distintas, y la
    /// que importa es la segunda: lo que llega al proveedor ya no se retira.
    ///
    /// ROTO A PROPÓSITO: se movió el registro a después de resolver la
    /// respuesta, y esta prueba falló — con el fallo, el pedido desaparecía del
    /// registro y «no se le pidió a nadie» se volvía indistinguible de «se le
    /// pidió y falló».
    #[test]
    fn se_anota_el_pedido_aunque_el_envio_falle() {
        let p = ProveedorSimulado::que_siempre(Respuesta::NoExisteEsaDireccion);
        let _ = p.enviar(&mensaje("fantasma@x.mx"));

        assert_eq!(p.cuantos_pedidos(), 1);
        assert!(
            p.le_pidieron_enviar_a("fantasma@x.mx"),
            "un envío que falla SÍ llegó al proveedor. Si no constara, no se \
             podría distinguir de uno que nunca se intentó"
        );
    }

    /// A quien no se le pidió, no se le pidió.
    #[test]
    fn se_puede_afirmar_que_a_alguien_no_se_le_escribio() {
        let p = ProveedorSimulado::que_siempre_acepta();
        p.enviar(&mensaje("ana@x.mx")).expect("acepta");

        assert!(p.le_pidieron_enviar_a("ana@x.mx"));
        assert!(
            !p.le_pidieron_enviar_a("beto@x.mx"),
            "es la afirmación que sostiene las pruebas de baja y de parada"
        );
    }

    /// La clave de idempotencia llega al proveedor.
    ///
    /// Es la tercera capa contra el duplicado: viaja como `Message-Id` del
    /// correo y da trazabilidad.
    #[test]
    fn la_clave_de_idempotencia_viaja_con_el_mensaje() {
        let p = ProveedorSimulado::que_siempre_acepta();
        p.enviar(&Mensaje {
            para: "ana@x.mx",
            asunto: "Aviso",
            html: "<p>Hola</p>",
            de_una_campana: true,
            clave_de_idempotencia: "01J-abc",
        })
        .expect("acepta");

        let pedido = p.recibidos().into_iter().next().expect("consta el pedido");
        assert_eq!(pedido.clave_de_idempotencia, "01J-abc");
    }

    /// **La basura del proveedor se limpia antes de llegar al motor.**
    ///
    /// Caso «el servidor devuelve 5 000 caracteres de basura» de
    /// `ESTRATEGIA_QA.md` §3, esta vez por el camino completo: el motor recibe
    /// el texto ya recortado, no la respuesta cruda.
    #[test]
    fn la_basura_del_proveedor_llega_ya_limpia() {
        let hostil = format!("250 ok\u{202E}{}", "x".repeat(5_000));
        let p = ProveedorSimulado::que_siempre(Respuesta::AceptaDiciendo(hostil));

        let r = p.enviar(&mensaje("ana@x.mx")).expect("acepta");
        let id = r.id_del_proveedor.expect("trae identificador");

        assert!(id.chars().count() <= crate::proveedor::RESPUESTA_MAXIMA);
        assert!(!id.contains('\u{202E}'));
    }

    /// Se puede compartir entre hilos: el motor corre aparte de la ventana.
    #[test]
    fn el_simulador_se_puede_compartir_entre_hilos() {
        fn exige_send_sync<T: Send + Sync>() {}
        exige_send_sync::<ProveedorSimulado>();
    }
}
