//! El formato del archivo `.arles`: cómo se escribe y cómo se vuelve a leer.
//!
//! **Cero I/O.** Aquí entra una cabecera y sale un texto; entra un texto y sale
//! una cabecera o un error tipado. Quien abre archivos es `arles-app` (8.2).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA FORMA DEL ARCHIVO, Y POR QUÉ ES ÉSTA
//!
//! En el diagrama, `→` marca dónde va un tabulador de verdad: clippy no admite
//! tabuladores dentro de un comentario, y dibujarlos con espacios haría creer
//! que el separador son espacios — que es justo el error que rompería el
//! archivo al leerlo.
//!
//! ```text
//! ARLES-RESPALDO                      ← la primera línea, siempre igual
//! formato→1                           ← versión DEL FORMATO, no de ARLES
//! esquema→V10                         ← hasta qué migración llegó la base
//! alcance→esencial                    ← esencial | completo (D-8B)
//! creado→2026-10-05T14:03:00Z
//! cifrado→no                          ← lo rellena la 8.2 (D-8A)
//! filas→contact=120 suppression_entry=8
//!                                     ← una línea en blanco cierra la cabecera
//! contact→01900…→Ana→Ruiz→\N
//! contact→01900…→Beto→→Pérez
//! …
//!                                     ← otra línea en blanco cierra el cuerpo
//! suma→a3f1…                          ← sha256 de TODO lo anterior
//! ```
//!
//! **Por qué líneas y no un único JSON.** Un respaldo de 500 000 contactos no
//! cabe en memoria dos veces —una como estructura y otra como texto—, y el
//! presupuesto dice que importar medio millón se hace con memoria acotada. Una
//! línea por fila se escribe y se lee de una en una. Un JSON de un solo objeto
//! obliga a tenerlo entero antes de empezar.
//!
//! **Por qué la suma va al final y no en la cabecera.** Para calcularla hay que
//! haber escrito el cuerpo. Ponerla arriba obligaría a dos pasadas sobre medio
//! millón de filas, o a construirlo todo en memoria — justo lo que se evita.
//!
//! **Por qué la cabecera declara las filas de cada tabla.** Para que el
//! preflight de la restauración (8.3) pueda decir qué trae el respaldo **sin
//! leerlo entero**: con un archivo de 2 GB, leerlo dos veces para enseñar un
//! aviso es medio minuto de espera antes de una pantalla.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE EL ARCHIVO NO LLEVA, Y ES DELIBERADO
//!
//! - **Ninguna ruta del sistema de archivos**, ni nombre de equipo, ni usuario.
//!   Es la regla 4 de la frontera aplicada al disco: un respaldo no debe decir
//!   de qué máquina salió, porque se manda por correo.
//! - **Ninguna credencial** (§75). Ni la contraseña de una cuenta de correo ni
//!   el token de WhatsApp. Viven en el llavero y no salen de él.
//! - **Nada de `rate_budget`, `engine_control` ni `ui_preference`**: lo decide
//!   el manifiesto de la 8.0, no este módulo.

use crate::error::CoreError;

/// La versión **del formato**, no la de ARLES.
///
/// Son cosas distintas a propósito: ARLES puede ir por la v1.4 y seguir
/// escribiendo respaldos del formato 1 porque nada cambió en el archivo. Atar
/// las dos obligaría a subir el formato en cada versión y a que cada una
/// tuviera que saber leer todas las anteriores sin que hubiera diferencias.
pub const VERSION_DEL_FORMATO: u32 = 1;

/// La primera línea de todo respaldo.
///
/// Sirve para contestar «esto no es un respaldo» **antes** de leer nada más, y
/// para que un archivo truncado al principio no se confunda con uno corrupto.
pub const MARCA: &str = "ARLES-RESPALDO";

/// El separador. Un tabulador, y por eso hay que escaparlo en los valores.
const SEP: char = '\t';

/// Lo que se escribe donde no hay valor.
///
/// Un campo vacío es la cadena vacía; `\N` es NULL. **No son lo mismo**: un
/// contacto con apellido `""` es uno al que alguien le borró el apellido, y uno
/// con NULL es uno del que no se sabe. Confundirlos al restaurar cambiaría los
/// datos sin que nada fallara.
const NULO: &str = "\\N";

/// Cuánto de la base se lleva este respaldo (D-8B).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alcance {
    /// Lo que no se puede volver a conseguir, con el historial recortado.
    /// **Es el que viene elegido.**
    Esencial,
    /// Todo. Para cambiar de equipo.
    Completo,
}

impl Alcance {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Esencial => "esencial",
            Self::Completo => "completo",
        }
    }

    /// # Errores
    ///
    /// [`CoreError::RespaldoCorrupto`] si la cabecera trae un alcance que no
    /// existe. **No se cae en «esencial»**: restaurar un esencial creyendo que
    /// era completo dejaría fuera el historial sin decirlo.
    pub fn parse(s: &str) -> Result<Self, CoreError> {
        match s {
            "esencial" => Ok(Self::Esencial),
            "completo" => Ok(Self::Completo),
            _ => Err(CoreError::RespaldoCorrupto {
                motivo: "el alcance declarado no es «esencial» ni «completo»",
            }),
        }
    }
}

/// Cómo está protegido el cuerpo del respaldo (D-8A).
///
/// La variante existe desde el formato 1 aunque el cifrado lo construya la 8.2:
/// un campo añadido después obligaría a subir la versión del formato, y los
/// respaldos ya escritos no tendrían dónde decir que no van cifrados.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cifrado {
    /// Sin contraseña. Dirección lo permite **sólo a propósito**, con el aviso
    /// delante (D-8A).
    No,
    /// Contraseña del usuario, derivada con Argon2id. Lo rellena la 8.2.
    Argon2idXChaCha20,
}

impl Cifrado {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::No => "no",
            Self::Argon2idXChaCha20 => "argon2id-xchacha20poly1305",
        }
    }

    /// # Errores
    ///
    /// [`CoreError::RespaldoCorrupto`] si no se reconoce. **No se asume «no»**:
    /// tratar un cuerpo cifrado como texto plano produciría basura que luego
    /// habría que distinguir de un archivo roto.
    pub fn parse(s: &str) -> Result<Self, CoreError> {
        match s {
            "no" => Ok(Self::No),
            "argon2id-xchacha20poly1305" => Ok(Self::Argon2idXChaCha20),
            _ => Err(CoreError::RespaldoCorrupto {
                motivo: "el respaldo dice ir protegido de una forma que esta versión no conoce",
            }),
        }
    }
}

/// Lo que va antes del cuerpo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cabecera {
    pub formato: u32,
    /// Hasta qué migración llegó la base que se exportó: `V10`.
    pub esquema: String,
    pub alcance: Alcance,
    /// Cuándo se hizo, en UTC. Se escribe para que el preflight pueda decir
    /// «este respaldo es de hace cuatro meses», que es la frase que evita
    /// restaurar el equivocado.
    pub creado_en: String,
    pub cifrado: Cifrado,
    /// Cuántas filas trae de cada tabla, en el orden en que se escriben.
    pub filas: Vec<(String, u64)>,
}

impl Cabecera {
    /// Cuántas filas trae en total.
    #[must_use]
    pub fn total_de_filas(&self) -> u64 {
        self.filas.iter().map(|(_, n)| n).sum()
    }

    /// Cuántas filas declara de una tabla.
    #[must_use]
    pub fn filas_de(&self, tabla: &str) -> u64 {
        self.filas
            .iter()
            .find(|(t, _)| t == tabla)
            .map_or(0, |(_, n)| *n)
    }

    /// La cabecera como texto, **incluida la línea en blanco** que la cierra.
    #[must_use]
    pub fn redactar(&self) -> String {
        let filas = self
            .filas
            .iter()
            .map(|(t, n)| format!("{t}={n}"))
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            "{MARCA}\n\
             formato{SEP}{}\n\
             esquema{SEP}{}\n\
             alcance{SEP}{}\n\
             creado{SEP}{}\n\
             cifrado{SEP}{}\n\
             filas{SEP}{filas}\n\
             \n",
            self.formato,
            self.esquema,
            self.alcance.as_str(),
            self.creado_en,
            self.cifrado.as_str(),
        )
    }

    /// Lee la cabecera del principio del archivo.
    ///
    /// Devuelve la cabecera y **por dónde sigue el cuerpo**, para que quien lea
    /// no tenga que volver a contar líneas.
    ///
    /// # Errores
    ///
    /// - [`CoreError::NoEsUnRespaldo`] si no empieza por la marca. Es el caso de
    ///   «elegí el archivo equivocado», y tiene su propio mensaje.
    /// - [`CoreError::RespaldoDeVersionFutura`] si el formato es posterior al
    ///   que esta versión sabe leer. **No se intenta igualmente.**
    /// - [`CoreError::RespaldoCorrupto`] si falta un campo o no se entiende.
    pub fn leer(texto: &str) -> Result<(Self, usize), CoreError> {
        let mut lineas = texto.lines();

        if lineas.next() != Some(MARCA) {
            return Err(CoreError::NoEsUnRespaldo);
        }

        let mut campos: Vec<(String, String)> = Vec::new();
        let mut consumidas = 1usize;
        for linea in lineas {
            consumidas += 1;
            if linea.is_empty() {
                break;
            }
            let (clave, valor) = linea.split_once(SEP).ok_or(CoreError::RespaldoCorrupto {
                motivo: "una línea de la cabecera no tiene la forma «clave<tab>valor»",
            })?;
            campos.push((clave.to_owned(), valor.to_owned()));
        }

        let buscar = |c: &str| -> Result<String, CoreError> {
            campos
                .iter()
                .find(|(k, _)| k == c)
                .map(|(_, v)| v.clone())
                .ok_or(CoreError::RespaldoCorrupto {
                    motivo: "a la cabecera del respaldo le falta un campo obligatorio",
                })
        };

        let formato: u32 = buscar("formato")?
            .parse()
            .map_err(|_| CoreError::RespaldoCorrupto {
                motivo: "la versión del formato no es un número",
            })?;

        // ── SE COMPRUEBA ANTES QUE NADA MÁS, Y NO ES CASUAL ──
        //
        // Un formato posterior puede traer campos que aquí no se entienden. Si
        // se siguiera leyendo, el primero que no casara saldría como «respaldo
        // corrupto» — y mandaría a buscar otra copia a quien lo único que
        // necesita es actualizar ARLES.
        if formato > VERSION_DEL_FORMATO {
            return Err(CoreError::RespaldoDeVersionFutura {
                formato,
                admitido: VERSION_DEL_FORMATO,
            });
        }

        let filas = buscar("filas")?
            .split_whitespace()
            .map(|par| {
                let (t, n) = par.split_once('=').ok_or(CoreError::RespaldoCorrupto {
                    motivo: "el recuento de filas no tiene la forma «tabla=n»",
                })?;
                let n: u64 = n.parse().map_err(|_| CoreError::RespaldoCorrupto {
                    motivo: "el recuento de filas de una tabla no es un número",
                })?;
                Ok((t.to_owned(), n))
            })
            .collect::<Result<Vec<_>, CoreError>>()?;

        Ok((
            Self {
                formato,
                esquema: buscar("esquema")?,
                alcance: Alcance::parse(&buscar("alcance")?)?,
                creado_en: buscar("creado")?,
                cifrado: Cifrado::parse(&buscar("cifrado")?)?,
                filas,
            },
            consumidas,
        ))
    }
}

/// Un valor listo para escribir en una línea.
///
/// ── POR QUÉ ESCAPAR Y NO ELEGIR OTRO SEPARADOR ──
///
/// Porque no hay ningún carácter que un contacto no pueda tener. El nombre de
/// una empresa puede llevar tabuladores si alguien lo pegó de un Excel, y una
/// plantilla lleva saltos de línea **siempre**. Un separador «que nadie usa» es
/// un separador que alguien usará, y el fallo sería silencioso: la fila se
/// partiría en dos al restaurar y el contacto saldría con los datos corridos.
#[must_use]
pub fn escapar(valor: Option<&str>) -> String {
    let Some(v) = valor else {
        return NULO.to_owned();
    };
    let mut salida = String::with_capacity(v.len());
    for c in v.chars() {
        match c {
            // La barra va PRIMERO: escaparla después convertiría las barras que
            // acaba de poner el escape de `\t` en barras dobles.
            '\\' => salida.push_str("\\\\"),
            '\t' => salida.push_str("\\t"),
            '\n' => salida.push_str("\\n"),
            '\r' => salida.push_str("\\r"),
            _ => salida.push(c),
        }
    }
    salida
}

/// Lo contrario de [`escapar`].
///
/// # Errores
///
/// [`CoreError::RespaldoCorrupto`] si hay una barra al final sin nada detrás o
/// una secuencia que no existe. **No se deja pasar en crudo**: una barra suelta
/// significa que el archivo se cortó o que alguien lo editó, y en los dos casos
/// restaurar sería restaurar algo que no se sabe qué es.
pub fn desescapar(campo: &str) -> Result<Option<String>, CoreError> {
    if campo == NULO {
        return Ok(None);
    }
    let mut salida = String::with_capacity(campo.len());
    let mut cs = campo.chars();
    while let Some(c) = cs.next() {
        if c != '\\' {
            salida.push(c);
            continue;
        }
        match cs.next() {
            Some('\\') => salida.push('\\'),
            Some('t') => salida.push('\t'),
            Some('n') => salida.push('\n'),
            Some('r') => salida.push('\r'),
            _ => {
                return Err(CoreError::RespaldoCorrupto {
                    motivo: "un valor trae una barra invertida que no escapa nada",
                });
            }
        }
    }
    Ok(Some(salida))
}

/// Una fila del cuerpo: el nombre de la tabla y sus valores, ya escapados.
#[must_use]
pub fn redactar_fila(tabla: &str, valores: &[Option<&str>]) -> String {
    let mut linea = String::from(tabla);
    for v in valores {
        linea.push(SEP);
        linea.push_str(&escapar(*v));
    }
    linea.push('\n');
    linea
}

/// Lee una fila del cuerpo.
///
/// # Errores
///
/// [`CoreError::RespaldoCorrupto`] si la línea está vacía o algún valor trae un
/// escape que no existe.
pub fn leer_fila(linea: &str) -> Result<(String, Vec<Option<String>>), CoreError> {
    let mut partes = linea.split(SEP);
    let tabla = partes
        .next()
        .filter(|t| !t.is_empty())
        .ok_or(CoreError::RespaldoCorrupto {
            motivo: "una línea del cuerpo no dice de qué tabla es",
        })?;
    let valores = partes.map(desescapar).collect::<Result<Vec<_>, _>>()?;
    Ok((tabla.to_owned(), valores))
}

/// La suma de verificación, calculada a trozos.
///
/// Se alimenta a medida que se escribe o se lee, para no tener que guardar el
/// archivo entero en memoria. Es el mismo patrón que `HuellaEnCurso` usa al
/// importar, y por la misma razón.
#[derive(Clone)]
pub struct SumaEnCurso(sha2::Sha256);

impl std::fmt::Debug for SumaEnCurso {
    /// **No enseña el estado interno, y es a propósito.**
    ///
    /// El estado de un hash a medias es el contenido que se le ha metido, y por
    /// aquí pasan contactos y plantillas. Una traza de depuración que lo
    /// imprimiera sacaría datos personales a una bitácora.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SumaEnCurso(…)")
    }
}

impl Default for SumaEnCurso {
    fn default() -> Self {
        Self::new()
    }
}

impl SumaEnCurso {
    #[must_use]
    pub fn new() -> Self {
        use sha2::Digest;
        Self(sha2::Sha256::new())
    }

    pub fn añadir(&mut self, trozo: &str) {
        use sha2::Digest;
        self.0.update(trozo.as_bytes());
    }

    /// La suma en hexadecimal, sin prefijo.
    #[must_use]
    pub fn terminar(self) -> String {
        use sha2::Digest;
        format!("{:x}", self.0.finalize())
    }
}

/// La última línea del archivo, **incluida la línea en blanco** que la precede.
#[must_use]
pub fn redactar_cierre(suma: &str) -> String {
    format!("\nsuma{SEP}{suma}\n")
}

/// Saca la suma declarada de la última línea del archivo.
///
/// # Errores
///
/// [`CoreError::RespaldoCorrupto`] si no hay línea de cierre. **Es el caso de
/// un archivo truncado**, que es lo que pasa cuando se llena un disco a mitad
/// de la escritura o cuando un correo corta un adjunto: todo lo anterior se lee
/// perfectamente y lo que falta es el final.
pub fn leer_cierre(texto: &str) -> Result<String, CoreError> {
    texto
        .lines()
        .next_back()
        .and_then(|l| l.strip_prefix("suma\t"))
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .ok_or(CoreError::RespaldoCorrupto {
            motivo: "el respaldo no termina con su suma de verificación: está cortado",
        })
}

/// Comprueba que lo leído cuadra con lo que el archivo declara.
///
/// # Errores
///
/// [`CoreError::RespaldoCorrupto`] si no coincide.
pub fn comprobar_suma(calculada: &str, declarada: &str) -> Result<(), CoreError> {
    if calculada == declarada {
        Ok(())
    } else {
        Err(CoreError::RespaldoCorrupto {
            motivo: "la suma de verificación no coincide: el archivo está alterado o incompleto",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cabecera() -> Cabecera {
        Cabecera {
            formato: VERSION_DEL_FORMATO,
            esquema: "V10".to_owned(),
            alcance: Alcance::Esencial,
            creado_en: "2026-10-05T14:03:00Z".to_owned(),
            cifrado: Cifrado::No,
            filas: vec![
                ("contact".to_owned(), 2),
                ("suppression_entry".to_owned(), 1),
            ],
        }
    }

    /// **Ida y vuelta: lo que se escribe es lo que se lee.**
    ///
    /// Es la garantía que sostiene toda la fase. Si falla, un respaldo
    /// restaurado no es el que se hizo — y nada lo diría.
    #[test]
    fn una_cabecera_sobrevive_a_la_ida_y_la_vuelta() {
        let c = cabecera();
        let (leida, _) = Cabecera::leer(&c.redactar()).expect("se lee");
        assert_eq!(leida, c);
    }

    /// **Los cuatro caracteres que parten una fila en dos.**
    ///
    /// Un tabulador pegado de un Excel, un salto de línea de una plantilla
    /// —que los lleva SIEMPRE—, un retorno de Windows y la propia barra del
    /// escape. Sin escaparlos, la fila se parte al restaurar y el contacto sale
    /// con los datos corridos, sin un solo error.
    ///
    /// Verificada el 05/10/2026 quitando el escape del tabulador: la fila se
    /// lee con un valor de más y la prueba falla.
    #[test]
    fn un_valor_con_tabuladores_y_saltos_vuelve_igual() {
        let raros = [
            "con\tun tabulador",
            "con\nun salto",
            "con\r\nun retorno de Windows",
            "con \\ una barra",
            "con \\N que NO es nulo",
            "con\\tuna t escapada a mano",
            "",
            "normal y corriente",
            "con acentos: ñáéíóú ¿¡",
        ];
        for v in raros {
            let linea = redactar_fila("contact", &[Some(v)]);
            let (tabla, vals) = leer_fila(linea.trim_end_matches('\n')).expect("se lee");
            assert_eq!(tabla, "contact");
            assert_eq!(vals, vec![Some(v.to_owned())], "no sobrevivió: {v:?}");
        }
    }

    /// **Vacío y nulo no son lo mismo, y no pueden confundirse.**
    ///
    /// Un apellido `""` es uno que alguien borró; uno nulo es uno que no se
    /// sabe. Al restaurar, confundirlos cambia el dato sin que nada falle.
    #[test]
    fn el_vacio_y_el_nulo_no_se_confunden() {
        let linea = redactar_fila("contact", &[Some(""), None, Some("\\N")]);
        let (_, vals) = leer_fila(linea.trim_end_matches('\n')).expect("se lee");
        assert_eq!(
            vals,
            vec![Some(String::new()), None, Some("\\N".to_owned())],
            "el vacío, el nulo y el texto «\\N» tienen que seguir siendo tres cosas"
        );
    }

    /// Un archivo que no es un respaldo se dice con su propio error.
    ///
    /// Y **no** como «corrupto»: lo que hay que hacer es distinto. Aquí se elige
    /// otro archivo; allí se busca otra copia.
    #[test]
    fn otro_archivo_cualquiera_no_se_confunde_con_uno_roto() {
        for basura in ["", "hola", "{\"json\": true}", "ARLES-RESPALDOX"] {
            assert!(
                matches!(Cabecera::leer(basura), Err(CoreError::NoEsUnRespaldo)),
                "{basura:?} debería dar «no es un respaldo»"
            );
        }
    }

    /// **Un respaldo del futuro se rechaza, no se intenta.**
    ///
    /// Leerlo «lo que se pueda» restauraría una base incompleta sin decirlo, que
    /// es el fallo que toda la Fase 8 existe para evitar.
    ///
    /// Verificada el 05/10/2026 quitando la comprobación: el formato 99 pasa a
    /// leerse y sale como «corrupto», que manda a buscar otra copia a quien sólo
    /// necesita actualizar ARLES.
    #[test]
    fn un_respaldo_de_una_version_futura_no_se_intenta_leer() {
        let mut c = cabecera();
        c.formato = 99;
        let texto = c.redactar();
        assert!(matches!(
            Cabecera::leer(&texto),
            Err(CoreError::RespaldoDeVersionFutura {
                formato: 99,
                admitido: 1
            })
        ));
    }

    /// Un alcance que no existe no cae en «esencial».
    ///
    /// Restaurar un completo creyendo que era esencial —o al revés— cambiaría
    /// lo que el preflight le dijo al usuario antes de confirmar.
    #[test]
    fn un_alcance_desconocido_no_se_aproxima() {
        assert!(Alcance::parse("casi-todo").is_err());
        assert!(Cifrado::parse("rot13").is_err());
    }

    /// **Un archivo cortado se detecta por el final que falta.**
    ///
    /// Es lo que pasa cuando se llena el disco a mitad de escribir o cuando un
    /// correo corta un adjunto: todo lo anterior se lee perfectamente.
    #[test]
    fn un_archivo_cortado_se_caza_por_la_suma_que_falta() {
        let completo = format!(
            "{}{}{}",
            cabecera().redactar(),
            redactar_fila("contact", &[Some("Ana")]),
            redactar_cierre("abc123")
        );
        assert_eq!(leer_cierre(&completo).expect("hay cierre"), "abc123");

        // Se corta justo antes del cierre, que es el caso real.
        let cortado = completo.trim_end_matches("\nsuma\tabc123\n");
        assert!(matches!(
            leer_cierre(cortado),
            Err(CoreError::RespaldoCorrupto { .. })
        ));
    }

    /// **La suma cambia si cambia un solo carácter.**
    ///
    /// Verificada el 05/10/2026 comparando la suma de un cuerpo con la del
    /// mismo cuerpo con una letra distinta: no coinciden.
    #[test]
    fn la_suma_caza_un_cambio_de_un_caracter() {
        let suma = |t: &str| {
            let mut s = SumaEnCurso::new();
            s.añadir(t);
            s.terminar()
        };
        let a = suma("contact\tAna\n");
        let b = suma("contact\tAnA\n");
        assert_ne!(a, b, "la suma no distingue dos cuerpos distintos");
        assert!(comprobar_suma(&a, &a).is_ok());
        assert!(comprobar_suma(&a, &b).is_err());
    }

    /// Alimentarla a trozos da lo mismo que de una vez.
    ///
    /// Es lo que permite calcularla mientras se escriben 500 000 filas sin
    /// tenerlas todas en memoria.
    #[test]
    fn la_suma_a_trozos_es_la_misma_que_de_golpe() {
        let entero = {
            let mut s = SumaEnCurso::new();
            s.añadir("contact\tAna\ncontact\tBeto\n");
            s.terminar()
        };
        let a_trozos = {
            let mut s = SumaEnCurso::new();
            s.añadir("contact\tAna\n");
            s.añadir("contact\tBeto\n");
            s.terminar()
        };
        assert_eq!(entero, a_trozos);
    }

    /// Una barra al final, sin nada detrás, no se deja pasar.
    ///
    /// Significa que el archivo se cortó o que alguien lo editó, y en los dos
    /// casos restaurar sería restaurar algo que no se sabe qué es.
    #[test]
    fn una_barra_que_no_escapa_nada_es_corrupcion() {
        assert!(desescapar("acaba en barra\\").is_err());
        assert!(desescapar("escape\\qinventado").is_err());
    }

    /// El recuento de la cabecera sirve para el preflight sin leer el cuerpo.
    #[test]
    fn la_cabecera_dice_cuantas_filas_trae_sin_leer_el_cuerpo() {
        let c = cabecera();
        assert_eq!(c.filas_de("contact"), 2);
        assert_eq!(c.filas_de("suppression_entry"), 1);
        assert_eq!(c.filas_de("no_existe"), 0);
        assert_eq!(c.total_de_filas(), 3);
    }

    /// Un archivo entero, escrito y leído de punta a punta.
    ///
    /// Las pruebas de arriba miran cada pieza; ésta mira que encajen. Es el
    /// hueco del §9 en versión pequeña: tres piezas correctas que no se juntan.
    #[test]
    fn un_respaldo_entero_se_escribe_y_se_vuelve_a_leer() {
        let c = cabecera();
        let mut archivo = c.redactar();
        let mut suma = SumaEnCurso::new();
        suma.añadir(&archivo);

        for (nombre, apellido) in [("Ana", Some("Ruiz")), ("Beto\there", None)] {
            let fila = redactar_fila("contact", &[Some(nombre), apellido]);
            suma.añadir(&fila);
            archivo.push_str(&fila);
        }
        archivo.push_str(&redactar_cierre(&suma.terminar()));

        // Y ahora al revés, como lo hará la 8.3.
        let (leida, desde) = Cabecera::leer(&archivo).expect("cabecera");
        assert_eq!(leida, c);

        // `skip` y `get` en vez de rebanar por índice: una cabecera que
        // devolviera una línea de más reventaría la prueba con un pánico de
        // índice en vez de decir qué esperaba.
        let cuerpo: Vec<&str> = archivo
            .lines()
            .skip(desde)
            .take_while(|l| !l.is_empty())
            .collect();
        assert_eq!(cuerpo.len(), 2, "el cuerpo no trae las dos filas escritas");

        let (_, primera) = leer_fila(cuerpo.first().expect("primera fila")).expect("fila");
        assert_eq!(
            primera,
            vec![Some("Ana".to_owned()), Some("Ruiz".to_owned())]
        );
        let (_, segunda) = leer_fila(cuerpo.get(1).expect("segunda fila")).expect("fila");
        assert_eq!(segunda, vec![Some("Beto\there".to_owned()), None]);

        // La suma declarada cuadra con la del contenido.
        let declarada = leer_cierre(&archivo).expect("cierre");
        let cierre = redactar_cierre(&declarada);
        let sin_cierre = archivo
            .strip_suffix(&cierre)
            .expect("el archivo termina con su línea de cierre");
        let mut recalculada = SumaEnCurso::new();
        recalculada.añadir(sin_cierre);
        comprobar_suma(&recalculada.terminar(), &declarada).expect("la suma cuadra");
    }
}
