//! El bucle del trabajador: la pieza donde todo lo demás funciona junto.
//!
//! Sigue al pie de la letra `MOTOR_DE_EJECUCION.md` §13.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE ESTE ARCHIVO NO HACE, Y ES DELIBERADO
//!
//! No sabe de SMTP, no sabe de WhatsApp, no sabe de SQL y no duerme. Pregunta,
//! decide y anota; lo demás se lo pasan hecho:
//!
//! · las **reglas** —ritmo, ventana, reintentos— vienen de `arles-core`;
//! · la **cola y el presupuesto**, de `arles-db`;
//! · la **entrega**, de un [`Proveedor`], que en toda la Fase 4 es el simulador;
//! · el **reloj y el azar**, de un [`Entorno`], que en las pruebas se controla.
//!
//! Sin esa última pieza, este archivo no se podría probar: cada ejecución
//! caería en una hora distinta y sortearía números distintos, y habría que
//! aceptar el resultado sin comprobarlo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! UN PASO, NO UN BUCLE INFINITO
//!
//! La unidad pública es [`Trabajador::un_paso`]: mira una vez, hace como mucho
//! un envío, y dice lo que pasó. El bucle de verdad —esperar y volver— es
//! [`Trabajador::correr`], cuatro líneas construidas encima.
//!
//! Se separan porque un bucle infinito no se puede probar: habría que pararlo
//! desde fuera y confiar en el azar del planificador. Un paso se prueba de
//! frente, y cada rama del §13 tiene el suyo.

use arles_core::canal::Canal;
use arles_core::disyuntor::{Circuito, Efecto, tras_un_acierto, tras_un_fallo};
use arles_core::ids::CompanyId;
use arles_core::reintento::{Decision, Fallo, que_hacer};
use arles_core::ritmo::{Limites, NoSePuede, cabe_uno_mas};
use arles_core::supresion::{MotivoDeSupresion, OrigenDeSupresion};
// `TimeZone` es de `jiff` y llega reexportada por el núcleo. El motor **no**
// depende de `jiff`: la recibe ya resuelta por `arles_core::ventana::zona`, que
// es el único sitio que la construye y el único que sabe de esa biblioteca.
use arles_core::ventana::{TimeZone, Ventana, clave_de_dia, clave_de_hora};
use arles_db::{Db, DbError, TrabajoTomado};
use time::OffsetDateTime;

use crate::proveedor::{Mensaje, Proveedor, recortar_respuesta};
use crate::tiempo::sello;

/// Lo que el trabajador necesita saber de la campaña que está sirviendo.
///
/// Se le pasa entero al construirlo y no cambia mientras corre. Una campaña
/// cuya ventana o cuyos límites cambiaran a mitad de un paso podría enviar
/// fuera de horario sin que nada fallara; cambiarlos exige parar y volver a
/// arrancar el trabajador, que es un acto visible.
#[derive(Debug, Clone)]
pub struct Ajustes {
    /// La empresa. Es lo que acota la lista de bajas: la de una empresa no
    /// bloquea ni protege en otra.
    pub empresa: CompanyId,
    /// La campaña de la que se toma trabajo.
    pub campana: String,
    /// La cuenta remitente. **El presupuesto cuelga de ella**, no de la
    /// campaña: dos campañas de la misma cuenta comparten los cincuenta
    /// diarios, porque quien los cuenta es el proveedor de correo.
    pub cuenta: String,
    /// Por qué medio se manda.
    pub canal: Canal,
    /// Los días y horas en que esta campaña puede enviar, con su zona (D-10).
    pub ventana: Ventana,
    /// Cuántos por hora y cuántos por día.
    pub limites: Limites,
    /// La zona de **la empresa**, que es donde se cuentan las horas y los días
    /// del presupuesto. No es necesariamente la de la campaña: una campaña
    /// dirigida a Madrid se envía en horario de Madrid, pero gasta del cupo
    /// diario de una cuenta que vive en México.
    pub zona_de_la_empresa: TimeZone,
}

/// De dónde salen el reloj y el azar.
///
/// Existe por una razón y sólo una: **para poder probar el bucle**. Un motor
/// que llama a `now()` por dentro sólo se puede probar a la hora que sea
/// cuando alguien ejecute las pruebas, y uno que se sortea sus propios números
/// da un resultado distinto cada vez.
pub trait Entorno: Send + Sync {
    /// Qué hora es, en UTC.
    fn ahora(&self) -> OffsetDateTime;

    /// Un número de 0.0 a 1.0 para repartir los reintentos.
    ///
    /// Sin él, cien envíos que fallan a la vez —porque se cayó la red— vuelven
    /// todos a la vez cinco minutos después, y el servidor recibe la misma
    /// ráfaga que acaba de rechazar.
    fn azar(&self) -> f64;
}

/// El reloj y el azar de verdad.
#[derive(Debug, Clone, Copy, Default)]
pub struct EntornoDelSistema;

impl Entorno for EntornoDelSistema {
    fn ahora(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }

    /// Ocho bytes del sistema operativo, llevados a 0.0..1.0.
    ///
    /// Si el sistema no pudiera darlos —no pasa, salvo en un sistema roto—, se
    /// devuelve el punto medio: el reintento sale en su hora exacta, sin
    /// reparto. Es peor que con azar y mucho mejor que no reintentar.
    fn azar(&self) -> f64 {
        let mut bytes = [0u8; 8];
        if getrandom::fill(&mut bytes).is_err() {
            return 0.5;
        }
        // Entre 0.0 y 1.0: se divide por el mayor valor posible.
        #[allow(clippy::cast_precision_loss)]
        {
            u64::from_le_bytes(bytes) as f64 / u64::MAX as f64
        }
    }
}

/// El freno de verdad: el que pregunta a la base antes de cada envío.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ ESTO NO ES UNA BANDERA EN MEMORIA
///
/// El bucle pregunta «¿hay que parar?» dos veces por envío, y hasta la entrega
/// 4.4 detrás de esa pregunta no había nada. Ahora hay dos cosas, y al
/// trabajador le da igual cuál de las dos conteste que sí:
///
/// **La parada de emergencia**, que vale para toda la empresa y **sobrevive a
/// cerrar el programa**. Una bandera en memoria se levantaría sola al
/// arrancar, y quien la pulsó no estaría delante para verlo.
///
/// **El estado de la campaña.** Si alguien la pausó o la detuvo, este
/// trabajador para. Sin esto, pausar cambiaría el rótulo de la pantalla y los
/// correos seguirían saliendo.
///
/// ─────────────────────────────────────────────────────────────────────────
/// ANTE LA DUDA, SE PARA
///
/// Si la base no puede contestar, el freno dice **que sí hay que parar**. Los
/// dos errores posibles no se parecen en nada: parar de más se ve —la campaña
/// no avanza y alguien pregunta— y enviar de más no se ve hasta que los
/// correos ya salieron.
///
/// Cuesta una consulta por envío, a un ritmo de cincuenta al día. No es un
/// coste, es el precio de que el botón de parar signifique lo que dice.
pub fn freno_de_la_base<'a>(
    db: &'a Db,
    empresa: CompanyId,
    campana: &'a str,
) -> impl Fn() -> bool + 'a {
    move || db.hay_que_parar(empresa, campana).unwrap_or(true)
}

/// Lo que pasó en un paso. **Cifras y hechos, no adjetivos** (§94).
///
/// Es lo que la pantalla de Actividad va a enseñar, así que cada variante dice
/// algo que una persona puede entender sin saber qué es una cola.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Paso {
    /// Alguien pulsó parar. No sale nada más.
    Parado,
    /// Estamos fuera del horario de la campaña.
    FueraDeVentana,
    /// Se acabó el cupo de esta hora o de este día.
    SinPresupuesto(NoSePuede),
    /// No hay nada pendiente ahora mismo.
    NoHayTrabajo,
    /// Esa persona está en la lista de bajas. **No es un fallo.**
    Baja { intento: String },
    /// Se paró entre tomar el trabajo y enviarlo.
    Cancelado { intento: String },
    /// El proveedor se hizo cargo. **No significa «entregado»** (§65).
    Aceptado { intento: String },
    /// Falló y volverá a intentarse. «Reintento 2 de 3» es una cifra.
    Reintentara { intento: String, cual: u8, de: u8 },
    /// Se agotaron los reintentos.
    Rendido { intento: String },
    /// Esa dirección no existe: se rinde y **entra en la lista de bajas**.
    DadoDeBaja { intento: String },
    /// La cuenta no autentica. **Se detiene la cuenta entera**, no este envío.
    CuentaParada { intento: String },
    /// El disyuntor de la cuenta está abierto: no se toma trabajo (§7).
    ///
    /// Lleva los segundos que faltan para poder decirlo con una cifra. «La
    /// cuenta está parada» a secas se lee como una avería; «vuelve a intentarlo
    /// en 23 minutos» es un hecho (§94).
    CuentaEnPausa { segundos_restantes: u64 },
}

impl Paso {
    /// ¿Tiene sentido volver a mirar inmediatamente?
    ///
    /// `false` significa «espera»: no hay trabajo, no es la hora, no hay cupo,
    /// o hay que parar. `true` significa que acaba de pasar algo y puede haber
    /// más detrás.
    #[must_use]
    pub const fn hay_mas_que_hacer(&self) -> bool {
        !matches!(
            self,
            Self::Parado
                | Self::FueraDeVentana
                | Self::SinPresupuesto(_)
                | Self::NoHayTrabajo
                | Self::CuentaParada { .. }
                | Self::CuentaEnPausa { .. }
        )
    }

    /// La clave de texto para la pantalla (§139).
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// UN SUCESO, UN TEXTO
    ///
    /// La primera versión tenía una clave `motor.*` propia para cada paso, y
    /// eso creaba **dos textos para el mismo suceso**: la pantalla de Actividad
    /// habría dicho «Se volverá a intentar» y el detalle del envío, a un clic
    /// de distancia, «Reintento 2 de 3». El mismo hecho contado de dos maneras
    /// en dos sitios, y una de las dos sin la cifra que exige el §94.
    ///
    /// Se corrigió al auditar la fase (23/09/2026): cuando el paso **es** el
    /// desenlace de un intento, la clave es la del desenlace; cuando el paso
    /// **es** una razón para no enviar, la clave es la de esa razón. Las
    /// `motor.*` que quedan son las que no existen en ningún otro sitio.
    #[must_use]
    pub fn clave_i18n(&self) -> &'static str {
        match self {
            // Estados del bucle. No los describe nadie más.
            Self::Parado => "motor.parado",
            Self::FueraDeVentana => "motor.fueraDeVentana",
            Self::NoHayTrabajo => "motor.noHayTrabajo",
            Self::Cancelado { .. } => "motor.cancelado",
            Self::Aceptado { .. } => "motor.aceptadoPorElProveedor",
            Self::CuentaEnPausa { .. } => "motor.laCuentaEstaEnPausa",

            // El motivo por el que no cabe un envío ya tiene su texto, y lleva
            // la cifra: «Se alcanzó el tope de 50 envíos al día».
            Self::SinPresupuesto(motivo) => motivo.clave_i18n(),

            // Los desenlaces de un intento los nombra `arles_core::reintento`,
            // que es quien los decide. Repetirlos aquí era duplicar el texto.
            Self::Baja { .. } => "reintento.pidioLaBaja",
            Self::Reintentara { .. } => "reintento.seVolveraAIntentar",
            Self::Rendido { .. } => "reintento.seAgotaronLosIntentos",
            Self::DadoDeBaja { .. } => "reintento.direccionInexistente",
            Self::CuentaParada { .. } => "reintento.laCuentaNoAutentica",
        }
    }
}

/// Quien manda: comprueba, toma, envía y anota.
pub struct Trabajador<'a> {
    db: &'a Db,
    proveedor: &'a dyn Proveedor,
    entorno: &'a dyn Entorno,
    /// La parada de emergencia. Se le **pregunta dos veces** por paso.
    ///
    /// En la Fase 4.4 detrás de esto habrá una bandera en la base de datos,
    /// para que parar sobreviva a cerrar el programa. El bucle no necesita
    /// saber cuál de las dos es: pregunta y obedece.
    freno: &'a dyn Fn() -> bool,
    ajustes: Ajustes,
}

impl std::fmt::Debug for Trabajador<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // El proveedor, el entorno y el freno son objetos de trait: no se
        // pueden imprimir. Los ajustes sí, y son lo que interesa en un
        // diagnóstico.
        f.debug_struct("Trabajador")
            .field("ajustes", &self.ajustes)
            .finish_non_exhaustive()
    }
}

impl<'a> Trabajador<'a> {
    /// Monta un trabajador con todo lo que necesita.
    pub fn nuevo(
        db: &'a Db,
        proveedor: &'a dyn Proveedor,
        entorno: &'a dyn Entorno,
        freno: &'a dyn Fn() -> bool,
        ajustes: Ajustes,
    ) -> Self {
        Self {
            db,
            proveedor,
            entorno,
            freno,
            ajustes,
        }
    }

    /// Un paso del §13: mira, toma como mucho uno, envía y anota.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// EL ORDEN NO ES ARBITRARIO
    ///
    /// **1 · La parada, primero de todo.** Lo más barato y lo más importante.
    ///
    /// **2 · La ventana y el presupuesto, antes de tomar nada.** Tomar un
    /// trabajo que no se va a poder enviar lo deja marcado como `claimed` para
    /// nadie, y hasta el siguiente arranque no vuelve a la cola.
    ///
    /// **3 · La lista de bajas, después de tomar y antes de enviar.** Tiene que
    /// ser aquí: entre que la campaña se activó y este envío pueden haber
    /// pasado meses, y alguien puede haber pedido la baja ayer.
    ///
    /// **4 · La parada, OTRA VEZ.** Entre la primera comprobación y este punto
    /// ha habido dos consultas a la base de datos. Es tiempo de sobra para que
    /// alguien pulse el botón, y el botón significa «ningún correo más», no
    /// «ningún correo más dentro de un rato».
    ///
    /// **5 · `sending` en la base ANTES de llamar al proveedor.** Es lo único
    /// que permite reconocer después un envío ambiguo (ADR-0004).
    ///
    /// # Errores
    ///
    /// [`DbError`] si la base falla. Un fallo del **proveedor** no es un error
    /// de esta función: es un [`Paso`] con su desenlace ya anotado.
    pub fn un_paso(&self) -> Result<Paso, DbError> {
        if (self.freno)() {
            return Ok(Paso::Parado);
        }

        let ahora = self.entorno.ahora();
        // La ventana se mira **antes** de consultar el presupuesto, y por eso
        // hay una vuelta aquí en vez de dejárselo entero a `cabe_uno_mas`:
        // fuera de horario no hace falta tocar la base. Una campaña de fin de
        // semana estaría consultándola sin parar para nada.
        let dentro_de_ventana = self.ajustes.ventana.contiene(ahora);
        if !dentro_de_ventana {
            return Ok(Paso::FueraDeVentana);
        }

        let marca = sello(ahora);

        // El disyuntor, antes del presupuesto y antes de tomar nada (§7, §13).
        //
        // Va aquí y no más abajo porque una cuenta con el circuito abierto no
        // debe ni tocar la cola: tomar un trabajo que no se va a poder enviar
        // lo deja marcado para nadie hasta el siguiente arranque.
        if let Circuito::Abierto { segundos_restantes } = self
            .db
            .salud_de_la_cuenta(&self.ajustes.cuenta, &marca)?
            .circuito
        {
            return Ok(Paso::CuentaEnPausa { segundos_restantes });
        }

        let hora = clave_de_hora(&self.ajustes.zona_de_la_empresa, ahora);
        let dia = clave_de_dia(&self.ajustes.zona_de_la_empresa, ahora);
        let gastado = self.db.gastado_de(&self.ajustes.cuenta, &hora, &dia)?;
        if let Err(motivo) = cabe_uno_mas(self.ajustes.limites, gastado, dentro_de_ventana) {
            // Se **pausa y se avisa**. No se busca otra cuenta: la rotación
            // automática de remitentes está prohibida (T-1, ADR-0008).
            return Ok(Paso::SinPresupuesto(motivo));
        }

        let Some(trabajo) = self.db.tomar_siguiente(&self.ajustes.campana, &marca)? else {
            return Ok(Paso::NoHayTrabajo);
        };

        if self
            .db
            .esta_suprimida(self.ajustes.empresa, self.ajustes.canal, &trabajo.direccion)?
        {
            self.db.marcar_suprimido(&trabajo.id, &marca)?;
            return Ok(Paso::Baja {
                intento: trabajo.id,
            });
        }

        if (self.freno)() {
            self.db.marcar_cancelado(&trabajo.id, &marca)?;
            return Ok(Paso::Cancelado {
                intento: trabajo.id,
            });
        }

        self.db.marcar_enviando(&trabajo.id, &marca)?;

        let mensaje = Mensaje {
            para: &trabajo.direccion,
            clave_de_idempotencia: &trabajo.clave_de_idempotencia,
        };
        match self.proveedor.enviar(&mensaje) {
            Ok(aceptado) => {
                // ─────────────────────────────────────────────────────────
                // EL PRESUPUESTO SE ANOTA ANTES QUE EL ESTADO, Y ESO IMPORTA
                //
                // Si el programa muriera entre las dos escrituras:
                //
                //   · Presupuesto primero → queda un permiso gastado por un
                //     envío que la fila no confirma. La campaña rinde uno
                //     menos. **Se envía de menos.**
                //   · Estado primero → queda un envío registrado que no gastó
                //     permiso. La cuenta puede pasarse del límite del
                //     proveedor. **Se envía de más.**
                //
                // Enviar de menos se ve y se arregla; pasarse del límite de
                // Gmail cuesta la cuenta. Es la misma asimetría de ADR-0004,
                // aplicada aquí.
                self.db
                    .anotar_un_envio(&self.ajustes.cuenta, &hora, &dia, &marca)?;

                let id_del_proveedor = aceptado.id_del_proveedor.as_deref().map(recortar_respuesta);
                self.db
                    .marcar_enviado(&trabajo.id, id_del_proveedor.as_deref(), None, &marca)?;
                // Un envío que sale borra la racha de fallos de la cuenta.
                self.db.aplicar_al_disyuntor(
                    &self.ajustes.cuenta,
                    tras_un_acierto(),
                    &marca,
                    None,
                )?;
                Ok(Paso::Aceptado {
                    intento: trabajo.id,
                })
            }
            Err(fallo) => self.anotar_el_fallo(&trabajo, fallo, ahora, &marca),
        }
    }

    /// Traduce un fallo del proveedor a una decisión, y la escribe.
    ///
    /// La decisión la toma `arles_core::reintento::que_hacer`, no este archivo:
    /// la política de reintentos es una regla de negocio (D-12) y vive con las
    /// demás. Aquí sólo se obedece.
    fn anotar_el_fallo(
        &self,
        trabajo: &TrabajoTomado,
        fallo: Fallo,
        ahora: OffsetDateTime,
        marca: &str,
    ) -> Result<Paso, DbError> {
        // El disyuntor primero: habla de la CUENTA, y la decisión de abajo
        // habla de ESTE ENVÍO. Son dos cosas distintas y las dos ocurren.
        let salud = self.db.salud_de_la_cuenta(&self.ajustes.cuenta, marca)?;
        let efecto = tras_un_fallo(fallo, salud.fallos_seguidos);
        let hasta = match efecto {
            Efecto::Abrir { durante, .. } => Some(sello(
                ahora.saturating_add(durante.try_into().unwrap_or(time::Duration::ZERO)),
            )),
            _ => None,
        };
        self.db
            .aplicar_al_disyuntor(&self.ajustes.cuenta, efecto, marca, hasta.as_deref())?;

        let decision = que_hacer(fallo, trabajo.reintentos_ya_hechos, self.entorno.azar());
        let clave = decision.clave_i18n();
        let id = trabajo.id.clone();

        match decision {
            Decision::Reintentar { espera, cual, de } => {
                let cuando = sello(ahora.saturating_add(espera.try_into().unwrap_or_default()));
                self.db.marcar_fallido(&id, clave, None, &cuando, marca)?;
                Ok(Paso::Reintentara {
                    intento: id,
                    cual,
                    de,
                })
            }
            Decision::Rendirse => {
                self.db.marcar_rendido(&id, clave, None, marca)?;
                Ok(Paso::Rendido { intento: id })
            }
            Decision::RendirseYDarDeBaja => {
                self.db.marcar_rendido(&id, clave, None, marca)?;
                // Por dirección y no por contacto: el contacto puede haberse
                // borrado, y el rebote puede venir de una dirección que nunca
                // estuvo en la agenda. Lo que no se puede volver a escribir es
                // **la dirección**.
                self.db.suprimir_direccion(
                    self.ajustes.empresa,
                    self.ajustes.canal,
                    &trabajo.direccion,
                    MotivoDeSupresion::RebotePermanente,
                    OrigenDeSupresion::RespuestaSmtp,
                )?;
                Ok(Paso::DadoDeBaja { intento: id })
            }
            Decision::PararLaCuenta => {
                self.db
                    .marcar_en_espera_de_la_cuenta(&id, clave, None, marca)?;
                Ok(Paso::CuentaParada { intento: id })
            }
            // El proveedor dice que esa persona pidió la baja. No es un fallo
            // nuestro, pero el envío ya empezó: la fila se cierra como
            // definitiva —con la clave que dice por qué— y la dirección entra
            // en la lista de bajas, que es lo que de verdad la protege.
            //
            // No se cierra como `suppressed` aunque lo parezca: ese estado no
            // cuenta como envío consumido, y una fila que llegó a `sending`
            // pudo salir. Está argumentado en `arles_core::attempt::puede_ir_a`.
            Decision::NoEsUnFallo => {
                self.db.marcar_rendido(&id, clave, None, marca)?;
                self.db.suprimir_direccion(
                    self.ajustes.empresa,
                    self.ajustes.canal,
                    &trabajo.direccion,
                    MotivoDeSupresion::Baja,
                    OrigenDeSupresion::RespuestaSmtp,
                )?;
                Ok(Paso::Baja { intento: id })
            }
        }
    }

    /// Da pasos hasta que no haya más que hacer ahora mismo.
    ///
    /// Devuelve el último paso, que dice **por qué** se detuvo: no hay trabajo,
    /// no es la hora, no hay cupo, alguien paró, o la cuenta no autentica. Ese
    /// «por qué» es lo que la pantalla enseña; un motor que se para sin decirlo
    /// se lee como una avería (§95).
    ///
    /// Quien llama decide cuánto esperar antes de volver. El bucle **no
    /// duerme**: dormir aquí haría que cada prueba tardara lo que tarda el
    /// reloj de verdad.
    ///
    /// `tope` acota cuántos pasos se dan de una tacada, para que una cola de
    /// quinientos mil no ocupe el hilo sin soltarlo nunca.
    ///
    /// # Errores
    ///
    /// [`DbError`] si la base falla.
    pub fn correr(&self, tope: u32) -> Result<Paso, DbError> {
        let mut ultimo = Paso::NoHayTrabajo;
        for _ in 0..tope {
            ultimo = self.un_paso()?;
            if !ultimo.hay_mas_que_hacer() {
                break;
            }
        }
        Ok(ultimo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    use arles_core::campana::AccionDeCampana::{Pausar, Reanudar};
    use arles_core::disyuntor::FALLOS_PARA_ABRIR;
    use arles_core::ritmo::Limites;
    use arles_core::ventana::{Dias, zona};
    use arles_db::ClaveMaestra;
    use rusqlite::params;
    use time::macros::datetime;

    use crate::simulador::{ProveedorSimulado, Respuesta};

    /// Un miércoles a las 10 de la mañana en Ciudad de México: dentro de una
    /// ventana de lunes a viernes de 9 a 18.
    const EN_HORARIO: OffsetDateTime = datetime!(2026-09-23 16:00:00 UTC);
    /// El mismo día a las 23:00 de México. Fuera.
    const DE_NOCHE: OffsetDateTime = datetime!(2026-09-24 05:00:00 UTC);

    /// Reloj y azar bajo control. Ver [`Entorno`].
    struct EntornoFijo {
        ahora: OffsetDateTime,
        azar: f64,
    }

    impl Entorno for EntornoFijo {
        fn ahora(&self) -> OffsetDateTime {
            self.ahora
        }
        fn azar(&self) -> f64 {
            self.azar
        }
    }

    /// El azar en el punto medio: el reintento sale en su hora exacta, sin
    /// desplazamiento. Es lo que permite afirmar «a los cinco minutos» y que
    /// sea verdad.
    fn entorno(ahora: OffsetDateTime) -> EntornoFijo {
        EntornoFijo { ahora, azar: 0.5 }
    }

    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
    const CUENTA: &str = "cuenta-1";
    const CAMPANA: &str = "camp";

    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES (?1, 'TELEMETRY', 'MX', 'America/Mexico_City',
                         'hola@t.mx', '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z')",
                params![EMPRESA],
            )?;
            c.execute(
                "INSERT INTO email_account (id, company_id, display_name, email_address,
                                            provider_kind, credential_ref, status,
                                            daily_limit, hourly_limit, created_at, updated_at)
                 VALUES (?1, ?2, 'Ventas', 'ventas@t.mx', 'smtp', 'llavero://ventas',
                         'connected', 50, 10, '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z')",
                params![CUENTA, EMPRESA],
            )?;
            c.execute(
                // `running`: una campaña activada. El estado importa desde la
                // entrega 4.4 — el freno de la base lo consulta antes de cada
                // envío, y una campaña en borrador no envía nada.
                "INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES (?1, ?2, 'Prueba', 'running',
                         '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z')",
                params![CAMPANA, EMPRESA],
            )
        })
        .expect("siembra");
        (dir, db)
    }

    fn encolar(db: &Db, id: &str, direccion: &str) {
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO message_attempt
                     (id, campaign_id, channel, contact_address, idempotency_key,
                      state, attempt_count, created_at, updated_at)
                 VALUES (?1, ?2, 'email', ?3, ?4, 'queued', 0,
                         '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z')",
                params![id, CAMPANA, direccion, format!("msgid-{id}")],
            )
        })
        .expect("encola");
    }

    fn fila(db: &Db, id: &str, columna: &str) -> Option<String> {
        let sql = format!("SELECT {columna} FROM message_attempt WHERE id = ?1");
        db.ejecutar_en_pruebas(|c| c.query_row(&sql, params![id], |f| f.get(0)))
            .expect("lee")
    }

    fn ajustes() -> Ajustes {
        let mx = zona("America/Mexico_City").expect("zona conocida");
        Ajustes {
            empresa: EMPRESA.parse().expect("id válido"),
            campana: CAMPANA.to_owned(),
            cuenta: CUENTA.to_owned(),
            canal: Canal::Correo,
            ventana: Ventana::nueva(Dias::LABORABLES, 9, 18, mx.clone()).expect("ventana"),
            limites: Limites::nuevos(10, 50),
            zona_de_la_empresa: mx,
        }
    }

    /// Nadie ha pulsado parar.
    fn sin_freno() -> impl Fn() -> bool {
        || false
    }

    // ─── El camino que tiene que funcionar ──────────────────────────────────

    #[test]
    fn un_envio_normal_sale_y_gasta_presupuesto() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        let p = ProveedorSimulado::que_siempre(Respuesta::Acepta("prov-1"));
        let e = entorno(EN_HORARIO);
        let f = sin_freno();
        let t = Trabajador::nuevo(&db, &p, &e, &f, ajustes());

        assert_eq!(
            t.un_paso().expect("paso"),
            Paso::Aceptado {
                intento: "a".to_owned()
            }
        );
        assert_eq!(fila(&db, "a", "state").as_deref(), Some("sent"));
        assert_eq!(
            fila(&db, "a", "provider_message_id").as_deref(),
            Some("prov-1")
        );
        assert!(p.le_pidieron_enviar_a("ana@x.mx"));

        let gastado = db
            .gastado_de(
                CUENTA,
                &clave_de_hora(&ajustes().zona_de_la_empresa, EN_HORARIO),
                &clave_de_dia(&ajustes().zona_de_la_empresa, EN_HORARIO),
            )
            .expect("presupuesto");
        assert_eq!(gastado.en_este_dia, 1);
        assert_eq!(gastado.en_esta_hora, 1);
    }

    /// El `Message-Id` que se generó al encolar es el que viaja al proveedor.
    ///
    /// Es la tercera capa contra el duplicado (`arles_db::cola`): si el motor
    /// se inventara uno nuevo en cada intento, un reintento de algo que sí
    /// salió llegaría como un mensaje distinto y el proveedor no podría
    /// reconocerlo.
    #[test]
    fn el_mensaje_lleva_la_clave_que_se_guardo_al_encolar() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        let p = ProveedorSimulado::que_siempre_acepta();
        let e = entorno(EN_HORARIO);
        let f = sin_freno();
        Trabajador::nuevo(&db, &p, &e, &f, ajustes())
            .un_paso()
            .expect("paso");

        let pedido = p
            .recibidos()
            .into_iter()
            .next()
            .expect("el proveedor recibió el pedido");
        assert_eq!(pedido.clave_de_idempotencia, "msgid-a");
    }

    // ─── Las cuatro puertas de antes de enviar ──────────────────────────────

    /// **Fuera de horario no sale nada, y ni siquiera se toma el trabajo.**
    ///
    /// Verificada rompiéndola el 23/09/2026: quitando el `return` de la
    /// ventana, esta prueba falla en las tres comprobaciones — el paso pasa a
    /// `Aceptado`, la fila a `sent` y el simulador recibe el pedido.
    #[test]
    fn de_noche_no_sale_nada_ni_se_toca_la_cola() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        let p = ProveedorSimulado::que_siempre_acepta();
        let e = entorno(DE_NOCHE);
        let f = sin_freno();

        assert_eq!(
            Trabajador::nuevo(&db, &p, &e, &f, ajustes())
                .un_paso()
                .expect("paso"),
            Paso::FueraDeVentana
        );
        assert_eq!(
            fila(&db, "a", "state").as_deref(),
            Some("queued"),
            "tomar un trabajo que no se va a enviar lo deja marcado para nadie"
        );
        assert_eq!(p.cuantos_pedidos(), 0);
    }

    /// **Agotado el cupo del día, se pausa y se avisa. No se busca otra
    /// cuenta.**
    ///
    /// La rotación automática de remitentes está prohibida (T-1, ADR-0008): es
    /// el comportamiento que distingue a una herramienta de envío legítima de
    /// una de spam, y lo que hace que un proveedor cierre todas las cuentas de
    /// un dominio a la vez en lugar de una.
    #[test]
    fn sin_cupo_del_dia_se_pausa_y_no_se_busca_otra_cuenta() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        let limites = Limites::nuevos(10, 2);
        let hora = clave_de_hora(&ajustes().zona_de_la_empresa, EN_HORARIO);
        let dia = clave_de_dia(&ajustes().zona_de_la_empresa, EN_HORARIO);
        for _ in 0..2 {
            db.anotar_un_envio(CUENTA, &hora, &dia, "2026-09-23T16:00:00Z")
                .expect("gasta");
        }

        let p = ProveedorSimulado::que_siempre_acepta();
        let e = entorno(EN_HORARIO);
        let f = sin_freno();
        let paso = Trabajador::nuevo(
            &db,
            &p,
            &e,
            &f,
            Ajustes {
                limites,
                ..ajustes()
            },
        )
        .un_paso()
        .expect("paso");

        assert_eq!(
            paso,
            Paso::SinPresupuesto(NoSePuede::TopeDelDia { cabian: 2 })
        );
        assert_eq!(p.cuantos_pedidos(), 0);
        assert_eq!(fila(&db, "a", "state").as_deref(), Some("queued"));
    }

    /// **Quien pidió la baja no recibe, aunque estuviera en la audiencia
    /// congelada.**
    ///
    /// Es el motivo por el que la lista de bajas se consulta en el instante de
    /// enviar y no al encolar: entre las dos cosas pasan meses.
    ///
    /// Verificada rompiéndola el 23/09/2026: quitando la consulta,
    /// `le_pidieron_enviar_a` pasa a devolver `true` y esta prueba falla
    /// diciendo que se le escribió a quien había pedido no recibir.
    #[test]
    fn quien_pidio_la_baja_no_recibe_nada() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        db.suprimir_direccion(
            EMPRESA.parse().expect("id"),
            Canal::Correo,
            "ana@x.mx",
            MotivoDeSupresion::Baja,
            OrigenDeSupresion::Usuario,
        )
        .expect("suprime");

        let p = ProveedorSimulado::que_siempre_acepta();
        let e = entorno(EN_HORARIO);
        let f = sin_freno();
        let paso = Trabajador::nuevo(&db, &p, &e, &f, ajustes())
            .un_paso()
            .expect("paso");

        assert_eq!(
            paso,
            Paso::Baja {
                intento: "a".to_owned()
            }
        );
        assert_eq!(fila(&db, "a", "state").as_deref(), Some("suppressed"));
        assert!(
            !p.le_pidieron_enviar_a("ana@x.mx"),
            "al proveedor no se le llegó a pedir nada: esa persona pidió no \
             recibir"
        );
    }

    /// **La parada se comprueba OTRA VEZ justo antes de enviar.**
    ///
    /// Entre la primera comprobación y el envío hay dos consultas a la base de
    /// datos: tiempo de sobra para que alguien pulse el botón. Sin la segunda,
    /// «parar» significaría «parar dentro de un correo más», y ese correo sale
    /// de verdad.
    ///
    /// El freno de esta prueba dice «no» la primera vez y «sí» la segunda, que
    /// es exactamente lo que pasa cuando alguien pulsa en medio.
    ///
    /// Verificada rompiéndola el 23/09/2026: quitando la segunda comprobación,
    /// el paso pasa a `Aceptado` y el simulador recibe el pedido.
    #[test]
    fn la_parada_se_comprueba_otra_vez_justo_antes_de_enviar() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        let veces = Cell::new(0u32);
        let freno = || {
            let n = veces.get();
            veces.set(n + 1);
            // La primera vez todavía no ha pulsado. La segunda, sí.
            n > 0
        };

        let p = ProveedorSimulado::que_siempre_acepta();
        let e = entorno(EN_HORARIO);
        let paso = Trabajador::nuevo(&db, &p, &e, &freno, ajustes())
            .un_paso()
            .expect("paso");

        assert_eq!(
            paso,
            Paso::Cancelado {
                intento: "a".to_owned()
            }
        );
        assert_eq!(fila(&db, "a", "state").as_deref(), Some("cancelled"));
        assert_eq!(
            p.cuantos_pedidos(),
            0,
            "«parar» significa ningún correo más, no un correo más"
        );
    }

    #[test]
    fn con_la_parada_puesta_no_se_mira_ni_la_cola() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        let p = ProveedorSimulado::que_siempre_acepta();
        let e = entorno(EN_HORARIO);
        let f = || true;
        assert_eq!(
            Trabajador::nuevo(&db, &p, &e, &f, ajustes())
                .un_paso()
                .expect("paso"),
            Paso::Parado
        );
        assert_eq!(fila(&db, "a", "state").as_deref(), Some("queued"));
    }

    // ─── Los desenlaces de un fallo ─────────────────────────────────────────

    /// Un fallo pasajero programa el primer reintento a los cinco minutos
    /// (D-12), con el azar en el punto medio.
    #[test]
    fn un_fallo_pasajero_reintenta_a_los_cinco_minutos() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        let p = ProveedorSimulado::que_siempre(Respuesta::SeCayoLaRed);
        let e = entorno(EN_HORARIO);
        let f = sin_freno();

        let paso = Trabajador::nuevo(&db, &p, &e, &f, ajustes())
            .un_paso()
            .expect("paso");

        assert_eq!(
            paso,
            Paso::Reintentara {
                intento: "a".to_owned(),
                cual: 1,
                de: 3
            }
        );
        assert_eq!(fila(&db, "a", "state").as_deref(), Some("failed"));
        assert_eq!(
            fila(&db, "a", "next_retry_at").as_deref(),
            Some("2026-09-23T16:05:00Z")
        );
    }

    /// **Un envío que falló no gasta presupuesto.**
    ///
    /// Un permiso gastado por un correo que no salió hace que la campaña rinda
    /// menos de lo que el usuario configuró, y esa diferencia no se puede
    /// explicar mirando la pantalla.
    #[test]
    fn un_envio_que_fallo_no_gasta_presupuesto() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        let p = ProveedorSimulado::que_siempre(Respuesta::SeCayoLaRed);
        let e = entorno(EN_HORARIO);
        let f = sin_freno();
        Trabajador::nuevo(&db, &p, &e, &f, ajustes())
            .un_paso()
            .expect("paso");

        let gastado = db
            .gastado_de(
                CUENTA,
                &clave_de_hora(&ajustes().zona_de_la_empresa, EN_HORARIO),
                &clave_de_dia(&ajustes().zona_de_la_empresa, EN_HORARIO),
            )
            .expect("presupuesto");
        assert_eq!(gastado.en_este_dia, 0);
    }

    /// Agotados los tres reintentos, se rinde.
    #[test]
    fn al_cuarto_fallo_se_rinde() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        // Ya se reintentó las tres veces que permite D-12.
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "UPDATE message_attempt SET attempt_count = 3 WHERE id = 'a'",
                [],
            )
        })
        .expect("avanza el contador");

        let p = ProveedorSimulado::que_siempre(Respuesta::TardaDemasiado);
        let e = entorno(EN_HORARIO);
        let f = sin_freno();
        assert_eq!(
            Trabajador::nuevo(&db, &p, &e, &f, ajustes())
                .un_paso()
                .expect("paso"),
            Paso::Rendido {
                intento: "a".to_owned()
            }
        );
        assert_eq!(
            fila(&db, "a", "state").as_deref(),
            Some("permanently_failed")
        );
    }

    /// **Una dirección que no existe entra en la lista de bajas a la primera.**
    ///
    /// No se reintenta: insistir sobre una dirección muerta sube el contador
    /// que los proveedores usan para marcar a un remitente como spam.
    #[test]
    fn una_direccion_que_no_existe_se_da_de_baja_sin_reintentar() {
        let (_d, db) = base();
        encolar(&db, "a", "nadie@x.mx");
        let p = ProveedorSimulado::que_siempre(Respuesta::NoExisteEsaDireccion);
        let e = entorno(EN_HORARIO);
        let f = sin_freno();

        assert_eq!(
            Trabajador::nuevo(&db, &p, &e, &f, ajustes())
                .un_paso()
                .expect("paso"),
            Paso::DadoDeBaja {
                intento: "a".to_owned()
            }
        );
        assert_eq!(
            fila(&db, "a", "state").as_deref(),
            Some("permanently_failed")
        );
        assert!(
            db.esta_suprimida(EMPRESA.parse().expect("id"), Canal::Correo, "nadie@x.mx")
                .expect("consulta"),
            "una dirección que no existe tiene que dejar de intentarse en ésta \
             y en todas las campañas siguientes"
        );
    }

    /// Y si el proveedor dice que esa persona pidió la baja, tampoco se
    /// reintenta: se cierra y se registra la baja.
    #[test]
    fn si_el_proveedor_dice_que_pidio_la_baja_se_registra() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        let p = ProveedorSimulado::que_siempre(Respuesta::PidioLaBaja);
        let e = entorno(EN_HORARIO);
        let f = sin_freno();

        assert_eq!(
            Trabajador::nuevo(&db, &p, &e, &f, ajustes())
                .un_paso()
                .expect("paso"),
            Paso::Baja {
                intento: "a".to_owned()
            }
        );
        assert!(
            db.esta_suprimida(EMPRESA.parse().expect("id"), Canal::Correo, "ana@x.mx")
                .expect("consulta")
        );
    }

    /// **Una credencial caducada para la cuenta entera y no gasta reintentos.**
    ///
    /// Ningún envío de esa cuenta va a funcionar hasta que una persona la
    /// arregle. Seguir intentándolo gastaría los tres reintentos de media
    /// audiencia contra una puerta cerrada.
    #[test]
    fn una_credencial_caducada_para_la_cuenta_y_no_gasta_reintentos() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        let p = ProveedorSimulado::que_siempre(Respuesta::LaCredencialNoSirve);
        let e = entorno(EN_HORARIO);
        let f = sin_freno();

        let paso = Trabajador::nuevo(&db, &p, &e, &f, ajustes())
            .un_paso()
            .expect("paso");
        assert_eq!(
            paso,
            Paso::CuentaParada {
                intento: "a".to_owned()
            }
        );
        assert!(
            !paso.hay_mas_que_hacer(),
            "seguir dando pasos contra una cuenta que no autentica sólo gasta \
             los reintentos de todo el que venga detrás"
        );
        let reintentos: i64 = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT attempt_count FROM message_attempt WHERE id = 'a'",
                    [],
                    |f| f.get(0),
                )
            })
            .expect("lee");
        assert_eq!(reintentos, 0, "lo que falló fue la cuenta, no el envío");
    }

    /// El proveedor dijo cuánto esperar, y se respeta **literalmente**.
    #[test]
    fn cuando_el_proveedor_dice_cuanto_esperar_se_le_hace_caso() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        let p = ProveedorSimulado::que_siempre(Respuesta::EsperaDurante(
            std::time::Duration::from_secs(20 * 60),
        ));
        let e = entorno(EN_HORARIO);
        let f = sin_freno();
        Trabajador::nuevo(&db, &p, &e, &f, ajustes())
            .un_paso()
            .expect("paso");

        assert_eq!(
            fila(&db, "a", "next_retry_at").as_deref(),
            Some("2026-09-23T16:20:00Z"),
            "acortar lo que pide un servidor es lo que hace el software de \
             spam, y es como se consigue que te bloqueen"
        );
    }

    // ─── El bucle ───────────────────────────────────────────────────────────

    /// Tres en cola salen los tres, y el bucle para diciendo por qué.
    #[test]
    fn el_bucle_vacia_la_cola_y_dice_por_que_se_para() {
        let (_d, db) = base();
        for (id, quien) in [("a", "ana@x.mx"), ("b", "beto@x.mx"), ("c", "cris@x.mx")] {
            encolar(&db, id, quien);
        }
        let p = ProveedorSimulado::que_siempre_acepta();
        let e = entorno(EN_HORARIO);
        let f = sin_freno();

        let ultimo = Trabajador::nuevo(&db, &p, &e, &f, ajustes())
            .correr(100)
            .expect("corre");

        assert_eq!(ultimo, Paso::NoHayTrabajo);
        assert_eq!(p.cuantos_pedidos(), 3);
    }

    /// **El tope del bucle existe y se respeta.**
    ///
    /// Sin él, una campaña de quinientos mil ocuparía el hilo sin soltarlo: ni
    /// se podría parar, ni se podría cerrar el programa.
    #[test]
    fn el_bucle_suelta_el_hilo_al_llegar_al_tope() {
        let (_d, db) = base();
        for (id, quien) in [("a", "ana@x.mx"), ("b", "beto@x.mx"), ("c", "cris@x.mx")] {
            encolar(&db, id, quien);
        }
        let p = ProveedorSimulado::que_siempre_acepta();
        let e = entorno(EN_HORARIO);
        let f = sin_freno();

        Trabajador::nuevo(&db, &p, &e, &f, ajustes())
            .correr(2)
            .expect("corre");

        assert_eq!(
            p.cuantos_pedidos(),
            2,
            "el tope tiene que cortar, o el bucle no suelta nunca"
        );
    }

    // ─── El freno conectado a la base (entrega 4.4) ─────────────────────────

    /// **Pausar para el motor de verdad, no sólo en la pantalla.**
    ///
    /// Es la prueba de punta a punta de la entrega 4.4: se manda uno, alguien
    /// pausa, y no sale ni uno más. Sin el freno conectado a la base, la
    /// campaña diría «en pausa» y el proveedor seguiría recibiendo pedidos.
    ///
    /// Verificada rompiéndola el 23/09/2026: sustituyendo `freno_de_la_base`
    /// por un freno que siempre dice que no, el simulador recibe los tres.
    #[test]
    fn pausar_la_campana_detiene_el_bucle_de_verdad() {
        let (_d, db) = base();
        for (id, quien) in [("a", "ana@x.mx"), ("b", "beto@x.mx"), ("c", "cris@x.mx")] {
            encolar(&db, id, quien);
        }
        let p = ProveedorSimulado::que_siempre_acepta();
        let e = entorno(EN_HORARIO);
        let f = freno_de_la_base(&db, EMPRESA.parse().expect("id"), CAMPANA);
        let t = Trabajador::nuevo(&db, &p, &e, &f, ajustes());

        assert!(matches!(t.un_paso().expect("paso"), Paso::Aceptado { .. }));

        db.aplicar_a_la_campana(EMPRESA.parse().expect("id"), CAMPANA, Pausar)
            .expect("pausa");

        assert_eq!(t.un_paso().expect("paso"), Paso::Parado);
        assert_eq!(t.correr(100).expect("corre"), Paso::Parado);
        assert_eq!(
            p.cuantos_pedidos(),
            1,
            "salió el de antes de pausar, y ninguno después"
        );
    }

    /// Y reanudar la vuelve a poner en marcha con la cola intacta.
    #[test]
    fn reanudar_devuelve_el_motor_al_trabajo() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        let p = ProveedorSimulado::que_siempre_acepta();
        let e = entorno(EN_HORARIO);
        let empresa: CompanyId = EMPRESA.parse().expect("id");
        let f = freno_de_la_base(&db, empresa, CAMPANA);
        let t = Trabajador::nuevo(&db, &p, &e, &f, ajustes());

        db.aplicar_a_la_campana(empresa, CAMPANA, Pausar)
            .expect("pausa");
        assert_eq!(t.un_paso().expect("paso"), Paso::Parado);

        db.aplicar_a_la_campana(empresa, CAMPANA, Reanudar)
            .expect("reanuda");
        assert!(matches!(t.un_paso().expect("paso"), Paso::Aceptado { .. }));
    }

    /// **La parada de emergencia para cualquier campaña, esté como esté.**
    #[test]
    fn la_parada_de_emergencia_para_el_bucle() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        let p = ProveedorSimulado::que_siempre_acepta();
        let e = entorno(EN_HORARIO);
        let empresa: CompanyId = EMPRESA.parse().expect("id");
        let f = freno_de_la_base(&db, empresa, CAMPANA);
        let t = Trabajador::nuevo(&db, &p, &e, &f, ajustes());

        db.poner_parada_de_emergencia(empresa, Some("adrian"))
            .expect("para");
        assert_eq!(t.un_paso().expect("paso"), Paso::Parado);
        assert_eq!(p.cuantos_pedidos(), 0);

        db.levantar_parada_de_emergencia(empresa).expect("levanta");
        assert!(matches!(t.un_paso().expect("paso"), Paso::Aceptado { .. }));
    }

    /// **Si la base no sabe contestar, el motor para.**
    ///
    /// Se corrompe el estado de la campaña —lo que pasaría con una base editada
    /// por fuera o una migración a medias— y se comprueba que el freno decide
    /// parar en vez de seguir. Parar de más se ve; enviar de más no se ve hasta
    /// que los correos ya salieron.
    ///
    /// Verificada rompiéndola el 23/09/2026: cambiando el `unwrap_or(true)` de
    /// `freno_de_la_base` por `unwrap_or(false)`, el correo sale sobre una
    /// campaña cuyo estado el programa no entiende.
    #[test]
    fn si_la_base_no_sabe_contestar_el_motor_para() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch("PRAGMA ignore_check_constraints = ON")?;
            let r = c.execute(
                "UPDATE campaign SET status = 'halted' WHERE id = ?1",
                params![CAMPANA],
            );
            c.execute_batch("PRAGMA ignore_check_constraints = OFF")?;
            r
        })
        .expect("corrompe");

        let p = ProveedorSimulado::que_siempre_acepta();
        let e = entorno(EN_HORARIO);
        let f = freno_de_la_base(&db, EMPRESA.parse().expect("id"), CAMPANA);

        assert_eq!(
            Trabajador::nuevo(&db, &p, &e, &f, ajustes())
                .un_paso()
                .expect("paso"),
            Paso::Parado
        );
        assert_eq!(p.cuantos_pedidos(), 0, "ante la duda, no se envía");
    }

    // ─── Un suceso, un texto (auditoría de la fase) ─────────────────────────

    /// **Cada paso apunta al texto de quien decide ese suceso, no a una copia.**
    ///
    /// Antes de la auditoría había una clave `motor.*` por cada paso, y eso
    /// creaba dos textos para el mismo hecho: la pantalla de Actividad habría
    /// dicho «Se volverá a intentar» y el detalle del envío, a un clic,
    /// «Reintento 2 de 3». El mismo suceso contado de dos maneras, y una de las
    /// dos sin la cifra que exige el §94.
    ///
    /// Esta prueba fija a quién pertenece cada texto, para que la duplicación
    /// no vuelva la próxima vez que alguien añada un paso.
    #[test]
    fn cada_paso_apunta_al_texto_de_quien_decide_ese_suceso() {
        let casos: Vec<(Paso, &str)> = vec![
            // Estados del bucle: sólo existen aquí.
            (Paso::Parado, "motor."),
            (Paso::FueraDeVentana, "motor."),
            (Paso::NoHayTrabajo, "motor."),
            (
                Paso::CuentaEnPausa {
                    segundos_restantes: 60,
                },
                "motor.",
            ),
            (
                Paso::Cancelado {
                    intento: "a".into(),
                },
                "motor.",
            ),
            (
                Paso::Aceptado {
                    intento: "a".into(),
                },
                "motor.",
            ),
            // El motivo por el que no cabe un envío lo nombra `ritmo`, con su
            // cifra: «Se alcanzó el tope de 50 envíos al día».
            (
                Paso::SinPresupuesto(NoSePuede::TopeDelDia { cabian: 50 }),
                "ritmo.",
            ),
            (
                Paso::SinPresupuesto(NoSePuede::TopeDeLaHora { cabian: 10 }),
                "ritmo.",
            ),
            // Los desenlaces de un intento los nombra `reintento`, que es quien
            // los decide.
            (
                Paso::Baja {
                    intento: "a".into(),
                },
                "reintento.",
            ),
            (
                Paso::Reintentara {
                    intento: "a".into(),
                    cual: 2,
                    de: 3,
                },
                "reintento.",
            ),
            (
                Paso::Rendido {
                    intento: "a".into(),
                },
                "reintento.",
            ),
            (
                Paso::DadoDeBaja {
                    intento: "a".into(),
                },
                "reintento.",
            ),
            (
                Paso::CuentaParada {
                    intento: "a".into(),
                },
                "reintento.",
            ),
        ];

        for (paso, familia) in casos {
            assert!(
                paso.clave_i18n().starts_with(familia),
                "{paso:?} debería usar el texto de «{familia}» y usa «{}»",
                paso.clave_i18n()
            );
        }
    }

    /// Y ningún paso se queda sin texto.
    #[test]
    fn ningun_paso_se_queda_sin_texto() {
        for paso in [
            Paso::Parado,
            Paso::FueraDeVentana,
            Paso::NoHayTrabajo,
            Paso::SinPresupuesto(NoSePuede::FueraDeVentana),
            Paso::CuentaEnPausa {
                segundos_restantes: 0,
            },
            Paso::Baja {
                intento: "a".into(),
            },
            Paso::Cancelado {
                intento: "a".into(),
            },
            Paso::Aceptado {
                intento: "a".into(),
            },
            Paso::Reintentara {
                intento: "a".into(),
                cual: 1,
                de: 3,
            },
            Paso::Rendido {
                intento: "a".into(),
            },
            Paso::DadoDeBaja {
                intento: "a".into(),
            },
            Paso::CuentaParada {
                intento: "a".into(),
            },
        ] {
            let clave = paso.clave_i18n();
            assert!(clave.contains('.'), "{paso:?} sin clave");
            assert!(!clave.is_empty());
        }
    }

    // ─── El disyuntor por cuenta (§7, auditoría de la fase) ─────────────────

    /// **Cinco fallos de red seguidos paran la cuenta, y el bucle lo respeta.**
    ///
    /// Sin esto, cada envío de la cola gastaría sus tres reintentos contra un
    /// servidor que no está: quince intentos para descubrir lo que ya se sabía
    /// al quinto, y un proveedor tomando nota de que ese remitente insiste.
    ///
    /// Verificada rompiéndola el 23/09/2026: quitando la comprobación del
    /// circuito en `un_paso`, el bucle sigue pidiendo envíos al proveedor con
    /// el disyuntor abierto y esta prueba falla.
    #[test]
    fn el_disyuntor_abierto_detiene_el_bucle() {
        let (_d, db) = base();
        // Seis en cola: más que los cinco fallos que abren el disyuntor. Con
        // menos, la cola se vaciaría antes —cada fallo programa su reintento a
        // cinco minutos— y la prueba mediría eso en vez del circuito.
        for (id, quien) in [
            ("a", "a@x.mx"),
            ("b", "b@x.mx"),
            ("c", "c@x.mx"),
            ("d", "d@x.mx"),
            ("e", "e@x.mx"),
            ("f", "f@x.mx"),
        ] {
            encolar(&db, id, quien);
        }
        let p = ProveedorSimulado::que_siempre(Respuesta::SeCayoLaRed);
        let e = entorno(EN_HORARIO);
        let f = sin_freno();
        let t = Trabajador::nuevo(&db, &p, &e, &f, ajustes());

        // Se dan pasos hasta que el disyuntor corta. Con tres en cola y
        // reintentos programados a cinco minutos, lo que corta es el circuito.
        let mut pasos = 0;
        let mut ultimo = Paso::NoHayTrabajo;
        while pasos < 20 {
            ultimo = t.un_paso().expect("paso");
            pasos += 1;
            if matches!(ultimo, Paso::CuentaEnPausa { .. }) {
                break;
            }
        }

        assert!(
            matches!(ultimo, Paso::CuentaEnPausa { .. }),
            "tras una racha de fallos de red la cuenta tiene que pararse, no              seguir insistiendo: fue {ultimo:?}"
        );
        assert!(
            p.cuantos_pedidos() <= FALLOS_PARA_ABRIR as usize,
            "no se puede llamar al proveedor más veces de las que el disyuntor              tolera: fueron {}",
            p.cuantos_pedidos()
        );
        assert!(!ultimo.hay_mas_que_hacer(), "y el bucle deja de girar");
    }

    /// **Una racha de direcciones muertas NO para la cuenta.**
    ///
    /// Es la mitad que rompería el producto: una lista importada con
    /// direcciones viejas tiene rachas de rebotes, y si contaran, la campaña se
    /// pararía sola cada pocos minutos sin que el usuario entendiera por qué.
    #[test]
    fn una_racha_de_rebotes_no_para_la_cuenta() {
        let (_d, db) = base();
        for (id, quien) in [
            ("a", "a@x.mx"),
            ("b", "b@x.mx"),
            ("c", "c@x.mx"),
            ("d", "d@x.mx"),
            ("e", "e@x.mx"),
            ("f", "f@x.mx"),
        ] {
            encolar(&db, id, quien);
        }
        let p = ProveedorSimulado::que_siempre(Respuesta::NoExisteEsaDireccion);
        let e = entorno(EN_HORARIO);
        let f = sin_freno();
        let t = Trabajador::nuevo(&db, &p, &e, &f, ajustes());

        let ultimo = t.correr(100).expect("corre");

        assert_eq!(
            p.cuantos_pedidos(),
            6,
            "las seis direcciones se intentan: que no existan no dice nada              malo del remitente"
        );
        assert_eq!(ultimo, Paso::NoHayTrabajo, "la cola se vacía entera");
        assert!(
            db.salud_de_la_cuenta(CUENTA, "2026-09-23T16:00:00Z")
                .expect("lee")
                .circuito
                .puede_enviar()
        );
    }

    /// **Una credencial caducada para la cuenta a la primera**, y la marca para
    /// que el usuario sepa que hay que renovarla, no esperar.
    #[test]
    fn una_credencial_caducada_para_la_cuenta_a_la_primera() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        encolar(&db, "b", "beto@x.mx");
        let p = ProveedorSimulado::que_siempre(Respuesta::LaCredencialNoSirve);
        let e = entorno(EN_HORARIO);
        let f = sin_freno();
        let t = Trabajador::nuevo(&db, &p, &e, &f, ajustes());

        t.un_paso().expect("paso");

        let salud = db
            .salud_de_la_cuenta(CUENTA, "2026-09-23T16:00:00Z")
            .expect("lee");
        assert!(salud.hay_que_renovar_la_credencial());
        assert!(matches!(
            t.un_paso().expect("paso"),
            Paso::CuentaEnPausa { .. }
        ));
        assert_eq!(
            p.cuantos_pedidos(),
            1,
            "esperar a cinco cuando ya sabemos que la contraseña no sirve sólo              gasta los reintentos de otros cuatro envíos"
        );
    }

    /// **Un envío que sale borra la racha.**
    ///
    /// Sin esto, una campaña de cuatro meses acabaría abriendo el disyuntor por
    /// cinco tropiezos de red repartidos a lo largo de semanas.
    #[test]
    fn un_envio_que_sale_borra_la_racha_de_fallos() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx");
        encolar(&db, "b", "beto@x.mx");
        // Falla el primero, acepta el segundo.
        let p =
            ProveedorSimulado::con_guion(vec![Respuesta::SeCayoLaRed, Respuesta::Acepta("prov-1")]);
        let e = entorno(EN_HORARIO);
        let f = sin_freno();
        let t = Trabajador::nuevo(&db, &p, &e, &f, ajustes());

        t.un_paso().expect("falla");
        assert_eq!(
            db.salud_de_la_cuenta(CUENTA, "2026-09-23T16:00:00Z")
                .expect("lee")
                .fallos_seguidos,
            1
        );

        t.un_paso().expect("acierta");
        assert_eq!(
            db.salud_de_la_cuenta(CUENTA, "2026-09-23T16:00:00Z")
                .expect("lee")
                .fallos_seguidos,
            0,
            "un envío que sale prueba que la cuenta funciona"
        );
    }

    /// **Nadie recibe dos veces aunque el bucle pase mil veces.**
    ///
    /// Es la garantía que gobierna el motor entero, y aquí se mide de punta a
    /// punta: tres personas, cien vueltas, tres correos.
    #[test]
    fn por_mucho_que_gire_el_bucle_nadie_recibe_dos_veces() {
        let (_d, db) = base();
        for (id, quien) in [("a", "ana@x.mx"), ("b", "beto@x.mx"), ("c", "cris@x.mx")] {
            encolar(&db, id, quien);
        }
        let p = ProveedorSimulado::que_siempre_acepta();
        let e = entorno(EN_HORARIO);
        let f = sin_freno();
        let t = Trabajador::nuevo(&db, &p, &e, &f, ajustes());

        for _ in 0..100 {
            t.un_paso().expect("paso");
        }

        let destinatarios: Vec<String> = p.recibidos().into_iter().map(|r| r.para).collect();
        assert_eq!(destinatarios.len(), 3, "tres personas, tres correos");
        for quien in ["ana@x.mx", "beto@x.mx", "cris@x.mx"] {
            assert_eq!(
                destinatarios.iter().filter(|d| *d == quien).count(),
                1,
                "{quien} recibió más de un correo"
            );
        }
    }
}
