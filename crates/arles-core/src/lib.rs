//! Tipos de dominio de ARLES RELAY.
//!
//! Este crate **no hace I/O**: ni base de datos, ni red, ni sistema de archivos.
//! Es la regla de frontera 3.2 de `documentacion/03-arquitectura/ARQUITECTURA.md`,
//! y lo que permite testear la lógica de dominio sin levantar nada.

// El §138 prohíbe `unwrap()` y `panic!()` indiscriminados, y el workspace los
// deniega. En los tests son legítimos: un test que no puede fallar ruidosamente
// no sirve de nada. La denegación sigue activa en el código de producción.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used, clippy::panic))]
// `forbid`, no `deny`: aquí no hay excepción posible ni la habrá. Este crate no
// toca el sistema operativo, así que un `unsafe` sería un error, nunca una
// necesidad. El workspace deniega; estas dos raíces prohíben.
#![forbid(unsafe_code)]

pub mod analisis;
pub mod arco;
pub mod attempt;
pub mod campana;
pub mod canal;
pub mod cancelacion;
pub mod contacto;
pub mod correo;
pub mod disyuntor;
pub mod duracion;
pub mod email;
pub mod empresa;
pub mod error;
pub mod etapa;
pub mod ids;
pub mod importacion;
pub mod onboarding;
pub mod plantilla;
pub mod preflight;
pub mod reintento;
pub mod respaldo;
pub mod resumen;
pub mod ritmo;
pub mod sanear;
pub mod secret;
pub mod supresion;
pub mod texto;
pub mod ventana;
pub mod whatsapp;

pub use analisis::{Analisis, Choque, FilaRechazada, FilaSuprimida, analizar};
pub use arco::{Expediente, redactar};
pub use attempt::{AttemptState, TransitionError};
pub use campana::{AccionDeCampana, AccionImposible, EstadoDeCampana, ParadaDeEmergencia};
pub use canal::{Canal, PhoneNumber, ValorDeCanal};
pub use contacto::{
    BorradorDeCanal, BorradorDeContacto, CampoDeContacto, CanalValidado, DatosDeContacto,
    ErrorDeContacto, MAX_CANALES,
};
pub use disyuntor::{Circuito, Efecto, tras_un_acierto, tras_un_fallo};
pub use duracion::{Desenlace, DiaDeEnvio, Estimacion, QuienManda, estimar, techo_diario};
pub use email::EmailAddress;
pub use empresa::{
    BorradorDeEmpresa, CampoDeEmpresa, DatosDeEmpresa, ErrorDeCampo, PAISES_SOPORTADOS,
    ZONAS_SOPORTADAS,
};
pub use error::CoreError;
pub use etapa::{
    CondicionDeEtapa, ErrorDeSecuencia, EtapaPlanificada, MAX_ESPERA_HORAS, MAX_ETAPAS,
    validar_secuencia,
};
pub use importacion::{
    CampoImportable, HuellaEnCurso, MAX_COLUMNAS, MAX_FILAS, MotivoDeRechazo, OrigenDeLaLista,
    adivinar_campo, fila_a_borrador, huella, huella_de_bytes, proponer_mapeo,
};
pub use onboarding::{ListaDeOnboarding, PasoDeOnboarding, RecuentoDeAlta};
pub use preflight::{
    Observaciones, Preflight, Punto, PuntoRevisado, Resultado, Veredicto, revisar,
};
pub use secret::Secret;
pub use supresion::{
    AlcanceDeSupresion, MotivoDeSupresion, OrigenDeSupresion, PeticionDeSupresion,
};
