//! Un contacto y sus canales, con su validación.
//!
//! Decisiones **L-13** (alta y edición a mano) y **L-14** (la ficha enseña
//! todos los canales, la tabla el principal) de
//! `documentacion/01-producto/LOGISTICA_DE_CAMPANAS.md`.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! UN CONTACTO ES UNA PERSONA, NO UNA DIRECCIÓN
//!
//! Desde la migración V3 un contacto **tiene** canales: uno o varios correos,
//! uno o varios móviles, o una mezcla. Este módulo es lo que decide si un
//! conjunto de canales forma un contacto al que se le puede escribir.
//!
//! La validación vive aquí y **no en la pantalla**, por la misma razón que la
//! de empresa: dos validaciones son dos reglas que mantener iguales, y el día
//! que divergen el formulario aprueba lo que el núcleo rechaza.
//! ─────────────────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

use crate::canal::{Canal, ValorDeCanal};
use crate::error::CoreError;

/// Tope del nombre y del apellido. Viajan a la sustitución de plantillas y de
/// ahí a una cabecera de correo; un campo sin tope es un problema de otra capa.
const MAX_NOMBRE: usize = 80;

/// Tope del nombre de la empresa del contacto.
const MAX_EMPRESA: usize = 120;

/// Cuántos canales admite un contacto.
///
/// Diez es holgado —lo normal son uno o dos— y a la vez impide que una fila mal
/// mapeada de un XLSX produzca un contacto con doscientas columnas convertidas
/// en canales.
pub const MAX_CANALES: usize = 10;

/// Qué campo falló, para que la pantalla **señale el campo** y no obligue a
/// revisar el formulario entero (§95).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CampoDeContacto {
    Nombre,
    Apellido,
    Empresa,
    /// El problema está en la lista de canales como conjunto.
    Canales,
    /// El problema está en un canal concreto; el índice va en el error.
    Canal,
}

/// Un canal tal y como llega del formulario o de una importación.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BorradorDeCanal {
    pub canal: Canal,
    /// Lo que se escribió, sin normalizar. Normalizar es cosa del núcleo.
    pub valor: String,
    /// Si es el que se usa por defecto para su tipo.
    ///
    /// **Lo elige el usuario, no el orden.** Si lo decidiera el orden de
    /// importación, cambiaría solo al reimportar el mismo archivo ordenado de
    /// otra forma — y con él cambiaría a qué dirección se le escribe.
    pub principal: bool,
}

/// Un contacto tal y como llega del formulario.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BorradorDeContacto {
    pub nombre: String,
    pub apellido: String,
    pub empresa: String,
    pub canales: Vec<BorradorDeCanal>,
}

/// Un error de validación, con el campo al que pertenece.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDeContacto {
    pub campo: CampoDeContacto,
    /// Posición del canal que falló, cuando el campo es `Canal`.
    pub indice: Option<usize>,
    /// Clave de i18n. La interfaz resuelve el texto (§139).
    pub clave: &'static str,
}

/// Un contacto ya validado, con sus canales normalizados.
///
/// No deriva `Deserialize`: se construye **sólo** con [`DatosDeContacto::validar`].
/// Si se pudiera deserializar directamente, una entrada JSON produciría un
/// contacto con canales sin normalizar, y la deduplicación y la supresión —que
/// comparan la forma normalizada— dejarían de funcionar sin que nada fallara.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatosDeContacto {
    pub nombre: String,
    pub apellido: String,
    pub empresa: String,
    pub canales: Vec<CanalValidado>,
}

/// Un canal validado y normalizado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanalValidado {
    pub canal: Canal,
    /// Lo que escribió el usuario, para poder mostrarlo tal cual.
    pub valor_raw: String,
    /// La forma canónica. Es la clave de la deduplicación y de la supresión.
    pub valor_normalizado: String,
    pub principal: bool,
}

impl DatosDeContacto {
    /// Valida y normaliza un contacto con sus canales.
    ///
    /// `pais_por_defecto` es el de la empresa, y sólo se usa para completar
    /// móviles escritos sin prefijo internacional.
    ///
    /// # Errores
    ///
    /// La **lista completa** de errores, no el primero: la pantalla tiene que
    /// poder marcar todos los campos malos de una vez. Un formulario que
    /// corrige de uno en uno se recorre tantas veces como errores tenga.
    pub fn validar(
        borrador: &BorradorDeContacto,
        pais_por_defecto: &str,
    ) -> Result<Self, Vec<ErrorDeContacto>> {
        let mut errores = Vec::new();

        let nombre = borrador.nombre.trim().to_owned();
        let apellido = borrador.apellido.trim().to_owned();
        let empresa = borrador.empresa.trim().to_owned();

        for (texto, tope, campo, clave) in [
            (
                &nombre,
                MAX_NOMBRE,
                CampoDeContacto::Nombre,
                "contacto.error.nombreLargo",
            ),
            (
                &apellido,
                MAX_NOMBRE,
                CampoDeContacto::Apellido,
                "contacto.error.apellidoLargo",
            ),
            (
                &empresa,
                MAX_EMPRESA,
                CampoDeContacto::Empresa,
                "contacto.error.empresaLarga",
            ),
        ] {
            if texto.chars().count() > tope {
                errores.push(ErrorDeContacto {
                    campo,
                    indice: None,
                    clave,
                });
            }
        }

        // ── Los canales ──
        //
        // Un contacto sin ningún canal es un contacto al que no se le puede
        // escribir. Guardarlo no es inofensivo: engorda la lista, cuenta en el
        // total de la campaña y desaparece del envío sin explicación.
        if borrador.canales.is_empty() {
            errores.push(ErrorDeContacto {
                campo: CampoDeContacto::Canales,
                indice: None,
                clave: "contacto.error.sinCanales",
            });
        }
        if borrador.canales.len() > MAX_CANALES {
            errores.push(ErrorDeContacto {
                campo: CampoDeContacto::Canales,
                indice: None,
                clave: "contacto.error.demasiadosCanales",
            });
        }

        let mut validados: Vec<CanalValidado> = Vec::new();
        for (i, c) in borrador.canales.iter().enumerate() {
            match ValorDeCanal::parse(c.canal, &c.valor, pais_por_defecto) {
                Ok(valor) => {
                    // Dos veces la misma dirección **dentro del mismo contacto**.
                    // La base lo rechazaría por su índice único, pero ahí el
                    // error llega sin decir cuál de los dos sobra.
                    if validados
                        .iter()
                        .any(|v| v.canal == c.canal && v.valor_normalizado == valor.normalized())
                    {
                        errores.push(ErrorDeContacto {
                            campo: CampoDeContacto::Canal,
                            indice: Some(i),
                            clave: "contacto.error.canalRepetido",
                        });
                        continue;
                    }
                    validados.push(CanalValidado {
                        canal: c.canal,
                        valor_raw: valor.raw().to_owned(),
                        valor_normalizado: valor.normalized().to_owned(),
                        principal: c.principal,
                    });
                }
                Err(CoreError::EmailInvalido { .. }) => errores.push(ErrorDeContacto {
                    campo: CampoDeContacto::Canal,
                    indice: Some(i),
                    clave: "contacto.error.correoInvalido",
                }),
                Err(_) => errores.push(ErrorDeContacto {
                    campo: CampoDeContacto::Canal,
                    indice: Some(i),
                    clave: "contacto.error.telefonoInvalido",
                }),
            }
        }

        if !errores.is_empty() {
            return Err(errores);
        }

        marcar_principales(&mut validados);
        ordenar_para_la_ficha(&mut validados);

        Ok(Self {
            nombre,
            apellido,
            empresa,
            canales: validados,
        })
    }

    /// El canal principal de un tipo, que es lo que enseña la tabla (L-14).
    pub fn principal_de(&self, canal: Canal) -> Option<&CanalValidado> {
        self.canales
            .iter()
            .find(|c| c.canal == canal && c.principal)
    }

    /// Cuántos canales de ese tipo tiene. La tabla enseña el principal y, si
    /// hay más, cuántos — nunca tres correos apiñados en una celda.
    pub fn cuantos_de(&self, canal: Canal) -> usize {
        self.canales.iter().filter(|c| c.canal == canal).count()
    }
}

/// Deja **exactamente un principal por tipo**.
///
/// Dos casos que el formulario deja pasar y que aquí se cierran:
///
/// - **Ninguno marcado.** Pasa siempre al importar: un XLSX no trae esa
///   columna. Se marca el primero de su tipo; sin esto, la tabla no tendría
///   qué enseñar y el envío no sabría a cuál escribir.
/// - **Varios marcados.** Pasa al editar a mano. Gana el primero, y los demás
///   se desmarcan: «principal» que apunta a dos sitios no es principal.
fn marcar_principales(canales: &mut [CanalValidado]) {
    for tipo in Canal::TODOS {
        let mut visto = false;
        for c in canales.iter_mut().filter(|c| c.canal == *tipo) {
            if c.principal && !visto {
                visto = true;
            } else {
                c.principal = false;
            }
        }
        if !visto && let Some(primero) = canales.iter_mut().find(|c| c.canal == *tipo) {
            primero.principal = true;
        }
    }
}

/// El orden de la ficha (L-14, reglas 1 y 2).
///
/// Primero los correos y después los móviles; dentro de cada tipo, el principal
/// arriba. **No** el orden en que se importaron: ése cambia al reimportar el
/// mismo archivo ordenado de otra forma, y la ficha se vería distinta cada vez
/// sin que nadie haya tocado nada.
///
/// `sort_by_key` es estable, así que lo que no decide esta clave conserva el
/// orden de entrada — que es lo razonable entre dos correos secundarios.
fn ordenar_para_la_ficha(canales: &mut [CanalValidado]) {
    canales.sort_by_key(|c| {
        let tipo = match c.canal {
            Canal::Correo => 0,
            Canal::WhatsApp => 1,
        };
        (tipo, u8::from(!c.principal))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canal(canal: Canal, valor: &str, principal: bool) -> BorradorDeCanal {
        BorradorDeCanal {
            canal,
            valor: valor.to_owned(),
            principal,
        }
    }

    /// El canal en la posición `i`, o un fallo que dice qué se esperaba.
    ///
    /// El lint del proyecto prohíbe indexar, también en pruebas, y tiene razón:
    /// un índice fuera de rango falla con «index out of bounds», que no dice
    /// nada sobre lo que el test estaba comprobando.
    fn en(d: &DatosDeContacto, i: usize) -> &CanalValidado {
        d.canales
            .get(i)
            .unwrap_or_else(|| panic!("se esperaba un canal en la posición {i}: {:?}", d.canales))
    }

    /// Lo mismo para la lista de errores.
    fn err(e: &[ErrorDeContacto], i: usize) -> &ErrorDeContacto {
        e.get(i)
            .unwrap_or_else(|| panic!("se esperaba un error en la posición {i}: {e:?}"))
    }

    fn borrador(canales: Vec<BorradorDeCanal>) -> BorradorDeContacto {
        BorradorDeContacto {
            nombre: "Ana".into(),
            apellido: "Ruiz".into(),
            empresa: "Empresa SA".into(),
            canales,
        }
    }

    #[test]
    fn un_contacto_con_correo_y_movil_es_valido() {
        let d = DatosDeContacto::validar(
            &borrador(vec![
                canal(Canal::Correo, "Ana@Empresa.MX", true),
                canal(Canal::WhatsApp, "81 1234 5678", true),
            ]),
            "MX",
        )
        .expect("válido");

        assert_eq!(d.canales.len(), 2);
        assert_eq!(en(&d, 0).valor_normalizado, "ana@empresa.mx");
        assert_eq!(en(&d, 1).valor_normalizado, "+528112345678");
        // El original se conserva para poder enseñarlo tal cual.
        assert_eq!(en(&d, 0).valor_raw, "Ana@Empresa.MX");
    }

    /// Un contacto sin ningún canal no es un contacto: es una fila que engorda
    /// la lista, cuenta en el total de la campaña y desaparece del envío sin
    /// que nadie sepa por qué.
    #[test]
    fn un_contacto_sin_canales_se_rechaza() {
        let e = DatosDeContacto::validar(&borrador(vec![]), "MX").expect_err("debe fallar");
        assert_eq!(e.len(), 1);
        assert_eq!(err(&e, 0).campo, CampoDeContacto::Canales);
        assert_eq!(err(&e, 0).clave, "contacto.error.sinCanales");
    }

    /// La pantalla tiene que poder marcar **todos** los campos malos de una
    /// vez. Devolviendo sólo el primero, el formulario se recorre tantas veces
    /// como errores tenga.
    #[test]
    fn se_devuelven_todos_los_errores_no_el_primero() {
        let mut b = borrador(vec![
            canal(Canal::Correo, "no-es-un-correo", true),
            canal(Canal::WhatsApp, "1234", false),
        ]);
        b.nombre = "a".repeat(MAX_NOMBRE + 1);

        let e = DatosDeContacto::validar(&b, "MX").expect_err("debe fallar");
        assert_eq!(e.len(), 3, "faltan errores: {e:?}");
        assert!(e.iter().any(|x| x.campo == CampoDeContacto::Nombre));
        assert_eq!(
            e.iter()
                .filter(|x| x.campo == CampoDeContacto::Canal)
                .count(),
            2
        );
    }

    /// El error de un canal dice **cuál**. Sin el índice, la pantalla tendría
    /// que marcar los cinco canales o ninguno.
    #[test]
    fn el_error_de_un_canal_dice_cual_es() {
        let e = DatosDeContacto::validar(
            &borrador(vec![
                canal(Canal::Correo, "ana@empresa.mx", true),
                canal(Canal::Correo, "esto-no", false),
            ]),
            "MX",
        )
        .expect_err("debe fallar");

        assert_eq!(e.len(), 1);
        assert_eq!(err(&e, 0).indice, Some(1));
        assert_eq!(err(&e, 0).clave, "contacto.error.correoInvalido");
    }

    /// La misma dirección dos veces dentro del mismo contacto. La base lo
    /// rechazaría por su índice único, pero el error llegaría sin decir cuál
    /// de los dos sobra.
    #[test]
    fn la_misma_direccion_dos_veces_en_un_contacto_se_rechaza() {
        let e = DatosDeContacto::validar(
            &borrador(vec![
                canal(Canal::Correo, "ana@empresa.mx", true),
                canal(Canal::Correo, "ANA@EMPRESA.MX", false),
            ]),
            "MX",
        )
        .expect_err("debe fallar");

        assert_eq!(err(&e, 0).clave, "contacto.error.canalRepetido");
        assert_eq!(err(&e, 0).indice, Some(1));
    }

    /// Y el mismo texto en canales distintos **no** es un duplicado: un móvil y
    /// un correo no compiten.
    #[test]
    fn el_mismo_contacto_puede_tener_correo_y_movil_distintos() {
        DatosDeContacto::validar(
            &borrador(vec![
                canal(Canal::Correo, "ana@empresa.mx", true),
                canal(Canal::WhatsApp, "+528112345678", true),
            ]),
            "MX",
        )
        .expect("válido");
    }

    // ── L-14 · el principal y el orden ──

    /// Importar nunca trae la columna «principal». Sin esto, la tabla no
    /// tendría qué enseñar y el envío no sabría a qué dirección escribir.
    #[test]
    fn sin_ninguno_marcado_el_primero_de_cada_tipo_es_el_principal() {
        let d = DatosDeContacto::validar(
            &borrador(vec![
                canal(Canal::Correo, "uno@empresa.mx", false),
                canal(Canal::Correo, "dos@empresa.mx", false),
                canal(Canal::WhatsApp, "+528112345678", false),
            ]),
            "MX",
        )
        .expect("válido");

        assert_eq!(
            d.principal_de(Canal::Correo)
                .map(|c| c.valor_normalizado.as_str()),
            Some("uno@empresa.mx")
        );
        assert_eq!(
            d.principal_de(Canal::WhatsApp)
                .map(|c| c.valor_normalizado.as_str()),
            Some("+528112345678")
        );
    }

    /// «Principal» que apunta a dos sitios no es principal. Gana el primero.
    #[test]
    fn dos_principales_del_mismo_tipo_se_reducen_a_uno() {
        let d = DatosDeContacto::validar(
            &borrador(vec![
                canal(Canal::Correo, "uno@empresa.mx", true),
                canal(Canal::Correo, "dos@empresa.mx", true),
            ]),
            "MX",
        )
        .expect("válido");

        assert_eq!(
            d.canales.iter().filter(|c| c.principal).count(),
            1,
            "quedó más de un principal: {:?}",
            d.canales
        );
        assert_eq!(
            d.principal_de(Canal::Correo)
                .map(|c| c.valor_normalizado.as_str()),
            Some("uno@empresa.mx")
        );
    }

    /// Cada tipo tiene el suyo: marcar el correo principal no deja al móvil sin
    /// principal.
    #[test]
    fn cada_tipo_tiene_su_propio_principal() {
        let d = DatosDeContacto::validar(
            &borrador(vec![
                canal(Canal::WhatsApp, "+528112345678", false),
                canal(Canal::Correo, "ana@empresa.mx", true),
            ]),
            "MX",
        )
        .expect("válido");

        assert!(d.principal_de(Canal::Correo).is_some());
        assert!(d.principal_de(Canal::WhatsApp).is_some());
    }

    /// **La regla 1 y la 2 de L-14.** Correos primero, móviles después, y el
    /// principal arriba dentro de cada tipo — no el orden de importación, que
    /// cambia al reimportar el mismo archivo ordenado de otra forma.
    #[test]
    fn la_ficha_agrupa_por_tipo_y_pone_el_principal_arriba() {
        let d = DatosDeContacto::validar(
            &borrador(vec![
                canal(Canal::WhatsApp, "+528100000002", false),
                canal(Canal::Correo, "secundario@empresa.mx", false),
                canal(Canal::WhatsApp, "+528100000001", true),
                canal(Canal::Correo, "principal@empresa.mx", true),
            ]),
            "MX",
        )
        .expect("válido");

        let orden: Vec<_> = d
            .canales
            .iter()
            .map(|c| (c.canal, c.valor_normalizado.as_str()))
            .collect();
        assert_eq!(
            orden,
            vec![
                (Canal::Correo, "principal@empresa.mx"),
                (Canal::Correo, "secundario@empresa.mx"),
                (Canal::WhatsApp, "+528100000001"),
                (Canal::WhatsApp, "+528100000002"),
            ]
        );
    }

    /// Entre dos secundarios del mismo tipo se respeta el orden de entrada: la
    /// ordenación es estable y no reordena lo que nadie pidió reordenar.
    #[test]
    fn los_secundarios_conservan_su_orden() {
        let d = DatosDeContacto::validar(
            &borrador(vec![
                canal(Canal::Correo, "principal@empresa.mx", true),
                canal(Canal::Correo, "b@empresa.mx", false),
                canal(Canal::Correo, "a@empresa.mx", false),
            ]),
            "MX",
        )
        .expect("válido");

        assert_eq!(en(&d, 1).valor_normalizado, "b@empresa.mx");
        assert_eq!(en(&d, 2).valor_normalizado, "a@empresa.mx");
    }

    /// La tabla enseña el principal y, si hay más, cuántos (L-14, regla 4).
    #[test]
    fn se_puede_saber_cuantos_canales_de_cada_tipo_hay() {
        let d = DatosDeContacto::validar(
            &borrador(vec![
                canal(Canal::Correo, "uno@empresa.mx", true),
                canal(Canal::Correo, "dos@empresa.mx", false),
                canal(Canal::WhatsApp, "+528112345678", true),
            ]),
            "MX",
        )
        .expect("válido");

        assert_eq!(d.cuantos_de(Canal::Correo), 2);
        assert_eq!(d.cuantos_de(Canal::WhatsApp), 1);
    }

    /// Una fila mal mapeada de un XLSX no puede convertir doscientas columnas
    /// en canales.
    #[test]
    fn un_contacto_con_demasiados_canales_se_rechaza() {
        let muchos = (0..=MAX_CANALES)
            .map(|i| canal(Canal::Correo, &format!("c{i}@empresa.mx"), false))
            .collect();
        let e = DatosDeContacto::validar(&borrador(muchos), "MX").expect_err("debe fallar");
        assert!(
            e.iter()
                .any(|x| x.clave == "contacto.error.demasiadosCanales")
        );
    }

    #[test]
    fn los_espacios_de_los_nombres_se_recortan() {
        let mut b = borrador(vec![canal(Canal::Correo, "ana@empresa.mx", true)]);
        b.nombre = "  Ana  ".into();
        b.empresa = "  Empresa SA  ".into();

        let d = DatosDeContacto::validar(&b, "MX").expect("válido");
        assert_eq!(d.nombre, "Ana");
        assert_eq!(d.empresa, "Empresa SA");
    }

    /// Un contacto sin nombre es legítimo: muchas listas traen sólo el correo.
    /// Lo que no es legítimo es un contacto sin canales.
    #[test]
    fn un_contacto_sin_nombre_es_valido() {
        let b = BorradorDeContacto {
            nombre: String::new(),
            apellido: String::new(),
            empresa: String::new(),
            canales: vec![canal(Canal::Correo, "ana@empresa.mx", true)],
        };
        DatosDeContacto::validar(&b, "MX").expect("válido");
    }
}
