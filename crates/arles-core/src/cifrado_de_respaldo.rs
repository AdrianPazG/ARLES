//! El sobre cifrado del respaldo (D-8A).
//!
//! **Cero I/O y nada de azar.** La sal y el nonce entran como parámetros en vez
//! de generarse aquí: así el núcleo sigue siendo puro —no pide entropía al
//! sistema operativo— y, de paso, **el cifrado se puede probar con vectores
//! fijos**. Una función que genera su propia aleatoriedad sólo se puede probar
//! cifrando y descifrando, que no distingue «cifra bien» de «cifra y descifra
//! mal de la misma forma».
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ POR TROZOS, Y NO EL ARCHIVO ENTERO DE UNA VEZ
//!
//! Lo obvio —cifrar el cuerpo como un único bloque— obliga a tenerlo entero en
//! memoria **dos veces**: el texto y su versión cifrada. Un respaldo completo de
//! una instalación de dos años son cientos de megas, y el presupuesto de
//! rendimiento fija 500 MB como techo durante una importación de medio millón
//! de filas. Se pasaría.
//!
//! Así que se cifra en **trozos de 64 KiB**, cada uno con su propia etiqueta de
//! autenticidad, usando la construcción STREAM de `aead`. Cada trozo lleva el
//! número de trozo dentro del nonce, lo que cierra dos ataques que el cifrado
//! bloque a bloque dejaría abiertos:
//!
//! - **Reordenar trozos.** Sin el contador, un atacante podría intercambiar dos
//!   trozos y el resultado seguiría descifrando: saldría una base con los
//!   contactos barajados.
//! - **Cortar el final.** El último trozo va marcado como último. Un archivo al
//!   que le quiten trozos del final falla al descifrar en vez de producir un
//!   respaldo más corto y perfectamente válido.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA CONTRASEÑA ENTRA Y NO SALE
//!
//! Es la misma regla que las credenciales de correo (§29, ADR-0011), aplicada
//! aquí: la contraseña del respaldo **no se guarda en ningún sitio**. No va al
//! llavero, no va a la base y no vuelve por la frontera. Vive en el campo
//! mientras se escribe y se borra de la memoria al terminar.
//!
//! Por eso el aviso de la pantalla no es decorativo: **si se olvida, el respaldo
//! no se puede abrir**. No hay puerta de atrás, y construirla sería construir
//! justo lo que el cifrado existe para impedir.

use crate::error::CoreError;

/// Cuánto se cifra de una vez.
///
/// 64 KiB es el tamaño que usan casi todas las herramientas de este tipo, y la
/// razón es la de siempre: cada trozo paga 16 bytes de etiqueta, así que
/// trozos pequeños engordan el archivo, y trozos grandes se comen la memoria
/// que esto venía a ahorrar. A 64 KiB el sobrecoste es del 0,02 %.
pub const TROZO: usize = 64 * 1024;

/// Bytes de la sal de Argon2id.
pub const BYTES_DE_SAL: usize = 16;

/// Bytes del nonce.
///
/// La construcción STREAM se queda con los últimos 5 para su contador y su
/// marca de último trozo, así que de los 24 de XChaCha20 quedan 19 de nonce
/// propio. Sigue siendo mucho más de lo que hace falta para que dos respaldos
/// no repitan nonce ni por accidente.
pub const BYTES_DE_NONCE: usize = 19;

/// Lo que cuesta derivar la clave de la contraseña.
///
/// ── POR QUÉ ESTOS NÚMEROS, Y NO LOS DE POR DEFECTO ──
///
/// Argon2id con 64 MiB, 3 pasadas y 1 hilo son los parámetros que el propio RFC
/// 9106 da como «segunda recomendación», pensada justo para esto: una máquina de
/// escritorio modesta —la de referencia tiene 8 GB (`PRESUPUESTO_RENDIMIENTO`
/// §2)— donde reservar 64 MiB un instante no molesta a nadie.
///
/// Lo que compran esos 64 MiB es que probar contraseñas a lo bruto salga caro:
/// una tarjeta gráfica puede hacer millones de SHA-256 por segundo, pero no
/// puede tener millones de bloques de 64 MiB a la vez. El coste para el usuario
/// es **una vez por respaldo**, y se nota menos que escribir la contraseña.
const MEMORIA_KIB: u32 = 64 * 1024;
const PASADAS: u32 = 3;
const HILOS: u32 = 1;

/// Deriva la clave de cifrado a partir de la contraseña.
///
/// Se devuelve envuelta en algo que se borra de la memoria al soltarlo: una
/// clave que se queda en un `Vec` cualquiera sobrevive en el montón hasta que
/// otra cosa pise esos bytes, y un volcado de memoria la encontraría.
fn derivar(contrasena: &str, sal: &[u8; BYTES_DE_SAL]) -> Result<[u8; 32], CoreError> {
    use argon2::{Algorithm, Argon2, Params, Version};

    let params = Params::new(MEMORIA_KIB, PASADAS, HILOS, Some(32)).map_err(|_| {
        CoreError::RespaldoCorrupto {
            motivo: "los parámetros de derivación de clave no son válidos",
        }
    })?;
    let a2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut clave = [0u8; 32];
    a2.hash_password_into(contrasena.as_bytes(), sal, &mut clave)
        .map_err(|_| CoreError::RespaldoCorrupto {
            motivo: "no se pudo derivar la clave a partir de la contraseña",
        })?;
    Ok(clave)
}

/// Cifra el cuerpo de un respaldo.
///
/// `sal` y `nonce` los genera quien llama, con la aleatoriedad del sistema.
/// **Nunca se reutilizan entre dos respaldos**: repetir el par con la misma
/// contraseña es lo único que rompe este cifrado, y por eso no se generan aquí
/// —donde sería fácil cachearlos sin querer— sino en cada llamada.
///
/// # Errores
///
/// [`CoreError::RespaldoCorrupto`] si la derivación o el cifrado fallan, que en
/// la práctica sólo ocurre si no hay memoria para Argon2id.
pub fn cifrar(
    contrasena: &str,
    sal: &[u8; BYTES_DE_SAL],
    nonce: &[u8; BYTES_DE_NONCE],
    claro: &[u8],
) -> Result<Vec<u8>, CoreError> {
    use chacha20poly1305::aead::stream::EncryptorBE32;
    use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
    use zeroize::Zeroize;

    let mut clave = derivar(contrasena, sal)?;
    let cifrador = XChaCha20Poly1305::new(clave.as_slice().into());
    clave.zeroize();

    let mut sobre = EncryptorBE32::from_aead(cifrador, nonce.as_slice().into());
    let mut salida = Vec::with_capacity(claro.len() + 64);

    // ── EL ÚLTIMO TROZO SE MARCA COMO ÚLTIMO ──
    //
    // Es lo que hace que cortar el final del archivo falle al descifrar en vez
    // de producir un respaldo más corto y perfectamente válido. Un cuerpo vacío
    // también tiene su trozo final: sin él, un archivo de cero trozos y uno
    // truncado a cero serían indistinguibles.
    let mut trozos = claro.chunks(TROZO).peekable();
    if trozos.peek().is_none() {
        let ultimo = sobre
            .encrypt_last(&[][..])
            .map_err(|_| CoreError::RespaldoCorrupto {
                motivo: "no se pudo cifrar el respaldo",
            })?;
        salida.extend_from_slice(&ultimo);
        return Ok(salida);
    }

    while let Some(trozo) = trozos.next() {
        if trozos.peek().is_none() {
            let ultimo = sobre
                .encrypt_last(trozo)
                .map_err(|_| CoreError::RespaldoCorrupto {
                    motivo: "no se pudo cifrar el último trozo del respaldo",
                })?;
            salida.extend_from_slice(&ultimo);
            break;
        }
        let cifrado = sobre
            .encrypt_next(trozo)
            .map_err(|_| CoreError::RespaldoCorrupto {
                motivo: "no se pudo cifrar un trozo del respaldo",
            })?;
        salida.extend_from_slice(&cifrado);
    }

    Ok(salida)
}

/// Descifra el cuerpo de un respaldo.
///
/// # Errores
///
/// [`CoreError::RespaldoCorrupto`] si la contraseña no es la que lo cifró, si el
/// archivo está alterado o si le falta el final.
///
/// **Los tres casos dan el mismo error, y es correcto.** El cifrado autenticado
/// no puede distinguir «contraseña equivocada» de «archivo manipulado»: las dos
/// cosas son «esta clave no abre esto». Fingir que lo distingue sería inventarse
/// un diagnóstico. Lo que sí hace la pantalla es sugerir primero lo probable —la
/// contraseña— sin afirmarlo.
pub fn descifrar(
    contrasena: &str,
    sal: &[u8; BYTES_DE_SAL],
    nonce: &[u8; BYTES_DE_NONCE],
    cifrado: &[u8],
) -> Result<Vec<u8>, CoreError> {
    use chacha20poly1305::aead::stream::DecryptorBE32;
    use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
    use zeroize::Zeroize;

    /// Lo que ocupa un trozo ya cifrado: el contenido más su etiqueta.
    const TROZO_CIFRADO: usize = TROZO + 16;

    let mut clave = derivar(contrasena, sal)?;
    let descifrador = XChaCha20Poly1305::new(clave.as_slice().into());
    clave.zeroize();

    let mut sobre = DecryptorBE32::from_aead(descifrador, nonce.as_slice().into());
    let mut salida = Vec::with_capacity(cifrado.len());

    let no_abre = || CoreError::RespaldoCorrupto {
        motivo: "la contraseña no abre este respaldo, o el archivo está alterado o incompleto",
    };

    let mut trozos = cifrado.chunks(TROZO_CIFRADO).peekable();
    if trozos.peek().is_none() {
        // Ni siquiera hay un trozo final: el archivo está vacío.
        return Err(no_abre());
    }

    while let Some(trozo) = trozos.next() {
        if trozos.peek().is_none() {
            salida.extend_from_slice(&sobre.decrypt_last(trozo).map_err(|_| no_abre())?);
            break;
        }
        salida.extend_from_slice(&sobre.decrypt_next(trozo).map_err(|_| no_abre())?);
    }

    Ok(salida)
}

/// Bytes a hexadecimal, para meterlos en la cabecera de texto.
///
/// A mano y no con una biblioteca: son seis líneas, y una dependencia más es
/// una dependencia más que auditar en `cargo deny` cada vez.
#[must_use]
pub fn a_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// Lo contrario de [`a_hex`].
///
/// # Errores
///
/// [`CoreError::RespaldoCorrupto`] si no es hexadecimal o no mide lo esperado.
pub fn de_hex(texto: &str, bytes_esperados: usize) -> Result<Vec<u8>, CoreError> {
    let mal = CoreError::RespaldoCorrupto {
        motivo: "la cabecera trae una sal o un nonce que no son hexadecimal del tamaño esperado",
    };
    if texto.len() != bytes_esperados * 2 {
        return Err(mal);
    }
    (0..bytes_esperados)
        .map(|i| {
            texto
                .get(i * 2..i * 2 + 2)
                .and_then(|par| u8::from_str_radix(par, 16).ok())
                .ok_or(CoreError::RespaldoCorrupto {
                    motivo:
                        "la cabecera trae una sal o un nonce que no son hexadecimal del tamaño esperado",
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAL: [u8; BYTES_DE_SAL] = [7u8; BYTES_DE_SAL];
    const NONCE: [u8; BYTES_DE_NONCE] = [9u8; BYTES_DE_NONCE];

    /// Lo que se cifra se descifra, y sale igual.
    ///
    /// La garantía mínima. Se prueba con un cuerpo que cruza varios trozos para
    /// que el camino de `encrypt_next` se recorra de verdad: con 100 bytes sólo
    /// se ejercita `encrypt_last`, y el troceado —que es lo que esta entrega
    /// construye— no se tocaría.
    #[test]
    fn lo_que_se_cifra_se_descifra() {
        for largo in [0, 1, 100, TROZO - 1, TROZO, TROZO + 1, TROZO * 3 + 77] {
            let claro: Vec<u8> = (0..largo).map(|i| (i % 251) as u8).collect();
            let sobre = cifrar("una contraseña larga", &SAL, &NONCE, &claro).expect("cifra");
            let vuelta = descifrar("una contraseña larga", &SAL, &NONCE, &sobre).expect("descifra");
            assert_eq!(vuelta, claro, "no sobrevivió un cuerpo de {largo} bytes");
        }
    }

    /// **Una contraseña distinta no abre el respaldo.**
    ///
    /// Es lo que separa «cifrado» de «ofuscado». Verificada el 05/10/2026
    /// cambiando una sola letra de la contraseña: falla.
    #[test]
    fn otra_contrasena_no_abre() {
        let sobre = cifrar("la buena", &SAL, &NONCE, b"datos de contactos").expect("cifra");
        assert!(descifrar("la buena ", &SAL, &NONCE, &sobre).is_err());
        assert!(descifrar("La buena", &SAL, &NONCE, &sobre).is_err());
        assert!(descifrar("", &SAL, &NONCE, &sobre).is_err());
    }

    /// **Cambiar un solo byte del archivo lo invalida.**
    ///
    /// Es la diferencia entre cifrar y cifrar *autenticado*. Sin la etiqueta,
    /// alguien podría voltear bits del cuerpo y el resultado descifraría sin
    /// protestar — con los datos cambiados.
    #[test]
    fn tocar_un_byte_invalida_el_respaldo() {
        let claro = vec![b'x'; TROZO * 2 + 10];
        let sobre = cifrar("contraseña", &SAL, &NONCE, &claro).expect("cifra");

        for posicion in [0, sobre.len() / 2, sobre.len() - 1] {
            let mut tocado = sobre.clone();
            // Se le da la vuelta a un bit: el cambio más pequeño posible.
            if let Some(b) = tocado.get_mut(posicion) {
                *b ^= 0b0000_0001;
            }
            assert!(
                descifrar("contraseña", &SAL, &NONCE, &tocado).is_err(),
                "un byte cambiado en la posición {posicion} pasó sin detectarse"
            );
        }
    }

    /// **Cortar el final no produce un respaldo más corto y válido.**
    ///
    /// Es el ataque que el cifrado por bloques sin marca de final deja abierto:
    /// quitar trozos del final daría una base con menos contactos, perfectamente
    /// descifrable. Con la construcción STREAM, el último trozo va marcado.
    ///
    /// Verificada el 05/10/2026 quitando el último trozo: falla.
    #[test]
    fn cortar_el_final_no_da_un_respaldo_valido() {
        let claro = vec![b'y'; TROZO * 3];
        let sobre = cifrar("contraseña", &SAL, &NONCE, &claro).expect("cifra");

        // `get` y no una rebanada: si el cifrado dejara de trocear, la prueba
        // diría qué esperaba en vez de reventar con un pánico de índice.
        let cortado = sobre
            .get(..sobre.len() - (TROZO + 16))
            .expect("el sobre trae al menos un trozo entero");
        assert!(
            descifrar("contraseña", &SAL, &NONCE, cortado).is_err(),
            "un archivo al que le falta el último trozo se leyó como si estuviera entero"
        );
    }

    /// **Barajar dos trozos se detecta.**
    ///
    /// Sin el contador dentro del nonce, intercambiar dos trozos descifraría sin
    /// error y saldría una base con los contactos mezclados entre sí.
    #[test]
    fn intercambiar_dos_trozos_se_detecta() {
        let claro: Vec<u8> = (0..TROZO * 3).map(|i| (i % 251) as u8).collect();
        let sobre = cifrar("contraseña", &SAL, &NONCE, &claro).expect("cifra");

        let n = TROZO + 16;
        let trozo = |desde: usize, hasta: Option<usize>| {
            hasta.map_or_else(
                || sobre.get(desde..).expect("hay trozo"),
                |h| sobre.get(desde..h).expect("hay trozo"),
            )
        };
        let mut barajado = Vec::with_capacity(sobre.len());
        barajado.extend_from_slice(trozo(n, Some(n * 2)));
        barajado.extend_from_slice(trozo(0, Some(n)));
        barajado.extend_from_slice(trozo(n * 2, None));

        assert!(
            descifrar("contraseña", &SAL, &NONCE, &barajado).is_err(),
            "dos trozos intercambiados pasaron sin detectarse"
        );
    }

    /// Dos sales distintas dan dos archivos distintos con la misma contraseña.
    ///
    /// Sin sal, dos respaldos de la misma base con la misma contraseña saldrían
    /// byte a byte iguales — y eso le dice a quien los vea que no cambió nada
    /// entre los dos, que ya es información.
    #[test]
    fn la_sal_hace_que_dos_respaldos_iguales_no_se_parezcan() {
        let otra_sal = [8u8; BYTES_DE_SAL];
        let a = cifrar("misma", &SAL, &NONCE, b"mismo contenido").expect("cifra");
        let b = cifrar("misma", &otra_sal, &NONCE, b"mismo contenido").expect("cifra");
        assert_ne!(a, b);
        // Y cada uno sólo se abre con la suya.
        assert!(descifrar("misma", &otra_sal, &NONCE, &a).is_err());
    }

    /// El hexadecimal va y vuelve, y rechaza lo que no lo es.
    #[test]
    fn el_hexadecimal_va_y_vuelve() {
        assert_eq!(a_hex(&[0, 15, 16, 255]), "000f10ff");
        assert_eq!(de_hex("000f10ff", 4).expect("lee"), vec![0, 15, 16, 255]);

        // Lo que no vale: tamaño distinto, caracteres que no son hexadecimal.
        assert!(de_hex("000f10", 4).is_err());
        assert!(de_hex("000f10ff00", 4).is_err());
        assert!(de_hex("000f10zz", 4).is_err());
        assert!(de_hex("", 4).is_err());

        // La sal entera, que es el uso real.
        let hex = a_hex(&SAL);
        assert_eq!(hex.len(), BYTES_DE_SAL * 2);
        assert_eq!(de_hex(&hex, BYTES_DE_SAL).expect("lee"), SAL.to_vec());
    }

    /// **La misma contraseña y la misma sal dan siempre la misma clave.**
    ///
    /// Si no, un respaldo no se podría abrir dos veces. Parece obvio y es la
    /// clase de cosa que se rompe al tocar los parámetros de Argon2 sin darse
    /// cuenta de que cambian la clave derivada — y entonces los respaldos
    /// antiguos dejan de abrirse, en silencio, hasta que alguien lo necesita.
    #[test]
    fn la_derivacion_es_estable() {
        let a = derivar("contraseña", &SAL).expect("deriva");
        let b = derivar("contraseña", &SAL).expect("deriva");
        assert_eq!(a, b);

        let c = derivar("contraseña", &[1u8; BYTES_DE_SAL]).expect("deriva");
        assert_ne!(a, c, "la sal tiene que cambiar la clave");
    }
}
