//! Un enum del dominio tiene **un solo nombre**, y es el de la base.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTE ARCHIVO EXISTE: EL H-05, QUE BLOQUEÓ EL PRODUCTO
//!
//! Dirección no podía dar de alta ni un contacto. El error decía:
//!
//! ```text
//! unknown variant `email`, expected `correo` or `whats_app`
//! ```
//!
//! `Canal` llevaba `#[serde(rename_all = "snake_case")]`, así que al cruzar la
//! frontera IPC se llamaba `correo` y `whats_app`. Pero en la base, en las
//! `CHECK` del esquema y en toda la interfaz se llama `email` y `whatsapp`.
//! Dos nombres para la misma cosa, y la junta entre ellos era la frontera.
//!
//! **Y rompía los dos sentidos.** De entrada, la webview mandaba `email` y
//! serde lo rechazaba. De salida, un canal llegaba a la pantalla como `correo`
//! y las comparaciones con `'email'` no encontraban nada: la lista de bajas
//! habría dicho «0 de correo» con la lista llena.
//!
//! Llevaba así desde la Fase 3, con todo en verde. Ninguna prueba podía verlo:
//! las de Rust construyen los valores directamente —sin pasar por JSON— y las
//! del frontend no cruzan la frontera. Es el patrón del §9 en la junta.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA REGLA QUE SE FIJA AQUÍ
//!
//! Para todo enum del dominio que se guarda en la base: **lo que serde escribe
//! es exactamente lo que dice `como_texto()`**.
//!
//! No es una preferencia de estilo. `como_texto()` es el contrato con las
//! `CHECK` de las migraciones, así que el nombre de la base no se puede
//! cambiar — hay datos escritos con él. Y la interfaz lee y escribe lo que ve
//! en la base. Que serde invente un tercer nombre no tiene ninguna ventaja y
//! cuesta un bloqueante.

// Un test de integración es **otro crate**, así que el permiso que
// `arles-core` se da a sí mismo en `#![cfg_attr(test, …)]` no llega aquí.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use arles_core::Canal;
use arles_core::campana::EstadoDeCampana;
use arles_core::etapa::CondicionDeEtapa;
use arles_core::supresion::{AlcanceDeSupresion, MotivoDeSupresion, OrigenDeSupresion};

/// Lo que serde escribe para un valor, sin las comillas del JSON.
fn como_lo_escribe_serde<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v)
        .expect("un enum del dominio siempre serializa")
        .trim_matches('"')
        .to_owned()
}

/// Comprueba las dos direcciones de un enum: cómo sale y cómo entra.
///
/// **Las dos hacen falta.** El H-05 rompía la entrada, pero la salida estaba
/// igual de rota y nadie lo había visto: una prueba que sólo mirara una de las
/// dos daría por bueno la mitad del problema.
macro_rules! un_solo_nombre {
    ($tipo:ty, $todos:expr) => {
        for v in $todos {
            let escrito = como_lo_escribe_serde(v);
            assert_eq!(
                escrito,
                v.como_texto(),
                "{} sale por la frontera como «{}» y en la base se llama «{}»",
                stringify!($tipo),
                escrito,
                v.como_texto()
            );

            let leido: $tipo = serde_json::from_str(&format!("\"{}\"", v.como_texto()))
                .unwrap_or_else(|e| {
                    panic!(
                        "{} no se puede leer por su nombre de la base «{}»: {e}",
                        stringify!($tipo),
                        v.como_texto()
                    )
                });
            assert_eq!(&leido, v, "{} no vuelve a ser el mismo", stringify!($tipo));
        }
    };
}

/// **La prueba que faltaba el día que el producto se bloqueó.**
///
/// Verificada rompiéndola el 20/10/2026: devolviendo `Canal` a
/// `rename_all = "snake_case"`, falla con el mensaje exacto del error que vio
/// Dirección — «sale por la frontera como «correo» y en la base se llama
/// «email»».
#[test]
fn el_nombre_que_cruza_la_frontera_es_el_de_la_base() {
    un_solo_nombre!(Canal, Canal::TODOS);
    un_solo_nombre!(MotivoDeSupresion, MotivoDeSupresion::TODOS);
    un_solo_nombre!(OrigenDeSupresion, OrigenDeSupresion::TODOS);
    un_solo_nombre!(AlcanceDeSupresion, AlcanceDeSupresion::TODOS);
    un_solo_nombre!(EstadoDeCampana, EstadoDeCampana::TODOS);
    un_solo_nombre!(CondicionDeEtapa, CondicionDeEtapa::TODAS);
}
