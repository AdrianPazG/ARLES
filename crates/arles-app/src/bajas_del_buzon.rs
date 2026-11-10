//! El cable de R-25: del buzón a la lista de supresión (tramo 3 · D-21).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ ES ESTE MÓDULO, Y POR QUÉ ES TAN CORTO
//!
//! Los dos tramos anteriores construyeron las dos piezas y **ninguna tenía
//! llamador**: `arles_core::baja_por_correo` decide qué cuenta como una baja y
//! `arles_email::buzon` trae los correos. Esto es el cable, y no hace nada
//! más: lee, pregunta, escribe y registra. Toda la decisión está en el núcleo
//! y toda la conversación con el servidor está en el adaptador.
//!
//! Lo vigila `Las bajas del buzón tienen quien las aplique` en `--fase 1`,
//! escrita **antes** de este archivo y vista fallar nombrando los seis tramos.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! EL ORDEN NO SE PUEDE INVERTIR, Y NO ES UNA PREFERENCIA
//!
//! Se pregunta al núcleo **antes** de escribir la supresión. Al revés sería dar
//! de baja primero y leer después qué decía el correo, y una baja **no se
//! puede levantar con un botón** (`MotivoDeSupresion::se_puede_levantar`): la
//! pidió la persona, así que el producto no la deshace. Un error en esa
//! dirección no tiene arreglo dentro de ARLES.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE NO SE TOCA SI ALGO FALLA
//!
//! La marca del buzón (`inbox_last_uid`) avanza **en la misma transacción** que
//! las bajas. Si avanzara aparte y la escritura fallara, esos correos quedarían
//! por debajo de la marca y **no se volverían a mirar nunca**: quien pidió no
//! recibir seguiría en la lista de envío sin que nada protestara.
//!
//! Y si un buzón de varios falla, los demás **sí** se revisan: el resultado
//! dice qué pasó con cada uno. Parar todo porque un servidor no contesta
//! dejaría sin aplicar bajas que ya estaban leídas.

use serde::Serialize;

use arles_core::baja_por_correo::{self, ComoLoPidio, Veredicto};
use arles_core::ids::EmailAccountId;
use arles_db::buzon::{BajaQueLlego, BuzonQueSeLee};
use arles_email::buzon::{CorreoDelBuzon, CuentaDeLectura};

use crate::error::{AppError, ErrorIpc};
use crate::estado::EstadoApp;
use crate::fabrica::FuenteDeContrasenas;

/// Cuántos días hacia atrás se mira.
///
/// Siete. El tope real de no repetir trabajo lo pone `inbox_last_uid`; esto
/// sólo acota lo que el servidor tiene que buscar. Una semana cubre de sobra
/// un buzón que se revisa a diario y perdona que nadie lo abra en unos días.
const DIAS_QUE_SE_MIRAN: u16 = 7;

/// Lo que pasó al revisar un buzón. **Sin una sola dirección dentro.**
///
/// Es la misma regla que el resumen exportable de la 7.6: la estructura que
/// cruza la frontera **no tiene dónde meter** una dirección de correo, así que
/// no es cuestión de acordarse. Quien quiera ver a quién se dio de baja lo mira
/// en Contactos › Bajas, que es la pantalla que existe para eso.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LoQueTrajoUnBuzon {
    /// El nombre visible de la cuenta, para decir qué buzón se revisó.
    pub buzon: String,
    /// Cuántos correos se miraron.
    pub mirados: u32,
    /// Cuántas direcciones entraron en la lista por primera vez.
    pub dadas_de_baja: u32,
    /// Cuántas ya estaban. No es un error: suprimir es idempotente.
    pub ya_estaban: u32,
    /// Cuántos correos nombran la baja sin pedirla claramente. **No se toca
    /// ninguno**: los mira una persona en su buzón.
    pub para_mirar: u32,
    /// Si el buzón no se pudo revisar, la clave del motivo (§139).
    pub no_se_pudo: Option<String>,
}

/// Cómo está la lectura del buzón de cada cuenta, para la pantalla.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComoEstaUnBuzonEnPantalla {
    pub cuenta: String,
    pub nombre_visible: String,
    pub direccion: String,
    /// De aquí sale el servidor de lectura que la pantalla **propone**.
    pub servidor_de_envio: String,
    pub servidor_de_lectura: Option<String>,
    pub encendido: bool,
    pub ya_se_leyo: bool,
}

/// Cómo está la lectura del buzón en cada cuenta de correo.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada o si la base falla.
#[tauri::command]
pub fn como_estan_los_buzones(
    estado: tauri::State<'_, EstadoApp>,
) -> Result<Vec<ComoEstaUnBuzonEnPantalla>, ErrorIpc> {
    let db = estado.db();
    let empresa = db
        .empresa()
        .map_err(|e| ErrorIpc::from(AppError::Db(e)))?
        .ok_or_else(|| ErrorIpc::from(AppError::EmpresaNoConfigurada))?;

    Ok(db
        .como_estan_los_buzones(empresa.id)
        .map_err(|e| ErrorIpc::from(AppError::Db(e)))?
        .into_iter()
        .map(|b| ComoEstaUnBuzonEnPantalla {
            cuenta: b.cuenta.to_string(),
            nombre_visible: b.nombre_visible,
            direccion: b.direccion,
            servidor_de_envio: b.servidor_de_envio,
            servidor_de_lectura: b.servidor_de_lectura,
            encendido: b.encendido,
            ya_se_leyo: b.ya_se_leyo,
        })
        .collect())
}

/// Quien va al buzón de verdad.
///
/// Es un rasgo por lo mismo que [`FuenteDeContrasenas`]: sin él, probar este
/// cable exigiría un servidor IMAP de verdad, y entonces **sólo se probaría el
/// día que alguien lo encendiera con el buzón de un cliente**. La
/// implementación de producción es [`ElBuzonDeVerdad`]; la de las pruebas habla
/// con el servidor de mentira de `arles-email`.
pub trait LeeUnBuzon {
    /// Trae los correos recientes de ese buzón.
    ///
    /// # Errores
    ///
    /// El de `arles-email`, traducido a [`AppError`] por quien llama.
    fn correos_de(
        &self,
        cuenta: &CuentaDeLectura,
        contrasena: &str,
        dias: u16,
    ) -> Result<Vec<CorreoDelBuzon>, arles_email::ErrorDeCorreo>;
}

/// El buzón de verdad, por IMAP sobre TLS.
#[derive(Debug)]
pub struct ElBuzonDeVerdad;

impl LeeUnBuzon for ElBuzonDeVerdad {
    fn correos_de(
        &self,
        cuenta: &CuentaDeLectura,
        contrasena: &str,
        dias: u16,
    ) -> Result<Vec<CorreoDelBuzon>, arles_email::ErrorDeCorreo> {
        arles_email::buzon::correos_recientes(cuenta, contrasena, dias)
    }
}

/// Revisa todos los buzones encendidos y aplica las bajas que encuentre.
///
/// Un comando **grueso** (regla 1 de la frontera): revisa, decide, escribe y
/// registra en una sola llamada. Partirlo en «lee» y «aplica» dejaría a la
/// webview decidiendo qué se da de baja, y eso es del núcleo.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada o si la base falla. **Un buzón
/// que no contesta no es un error del comando**: viaja dentro de su
/// [`LoQueTrajoUnBuzon::no_se_pudo`], porque los demás buzones sí se revisaron
/// y sus bajas sí se aplicaron.
#[tauri::command]
pub fn revisar_el_buzon(
    estado: tauri::State<'_, EstadoApp>,
) -> Result<Vec<LoQueTrajoUnBuzon>, ErrorIpc> {
    revisar_con(
        estado.db(),
        &ElBuzonDeVerdad,
        &crate::fabrica::ElLlaveroDelSistema,
    )
}

/// El cuerpo de verdad, con el lector y el llavero inyectados.
///
/// # Errores
///
/// Igual que [`revisar_el_buzon`].
pub fn revisar_con(
    db: &arles_db::Db,
    lector: &impl LeeUnBuzon,
    contrasenas: &impl FuenteDeContrasenas,
) -> Result<Vec<LoQueTrajoUnBuzon>, ErrorIpc> {
    let empresa = db
        .empresa()
        .map_err(|e| ErrorIpc::from(AppError::Db(e)))?
        .ok_or_else(|| ErrorIpc::from(AppError::EmpresaNoConfigurada))?;

    let buzones = db
        .buzones_que_se_leen(empresa.id)
        .map_err(|e| ErrorIpc::from(AppError::Db(e)))?;

    let mut salida = Vec::new();
    for buzon in buzones {
        salida.push(revisar_uno(db, lector, contrasenas, empresa.id, &buzon)?);
    }
    Ok(salida)
}

/// Un buzón, de principio a fin.
fn revisar_uno(
    db: &arles_db::Db,
    lector: &impl LeeUnBuzon,
    contrasenas: &impl FuenteDeContrasenas,
    empresa: arles_core::ids::CompanyId,
    buzon: &BuzonQueSeLee,
) -> Result<LoQueTrajoUnBuzon, ErrorIpc> {
    let fallo = |clave: &str| LoQueTrajoUnBuzon {
        buzon: buzon.nombre_visible.clone(),
        mirados: 0,
        dadas_de_baja: 0,
        ya_estaban: 0,
        para_mirar: 0,
        no_se_pudo: Some(clave.to_owned()),
    };

    // La contraseña **es la misma que la del envío**: ésa es la mitad de la
    // razón por la que esta alternativa se eligió el 10/11/2026. Si no está, la
    // cuenta tampoco puede enviar, y el texto de ese error ya manda a
    // configurarla.
    let Some(contrasena) = contrasenas
        .contrasena_de(&buzon.referencia_credencial)
        .map_err(ErrorIpc::from)?
    else {
        return Ok(fallo("error.app.cuenta.sin_contrasena"));
    };

    let cuenta = CuentaDeLectura {
        servidor: buzon.servidor.clone(),
        puerto: None,
        usuario: buzon.direccion.clone(),
    };

    // ── 1 · LEER ──
    let correos = match lector.correos_de(&cuenta, &contrasena, DIAS_QUE_SE_MIRAN) {
        Ok(c) => c,
        // Un buzón que no contesta no tumba la revisión de los demás: su
        // motivo viaja dentro y los otros siguen.
        Err(e) => return Ok(fallo(e.clave_i18n())),
    };

    // ── 2 · PREGUNTAR AL NÚCLEO ──
    //
    // Antes de escribir nada. Ver la cabecera del módulo.
    let nuevos: Vec<&CorreoDelBuzon> = correos
        .iter()
        .filter(|c| buzon.ultimo_uid.is_none_or(|visto| c.uid > visto))
        .collect();

    let mut bajas = Vec::new();
    let mut para_mirar = 0_u32;
    for correo in &nuevos {
        match baja_por_correo::leer(&correo.como_lo_lee_el_nucleo()) {
            Veredicto::EsUnaBaja { como } => {
                // El cómo no se guarda con la baja —la tabla no tiene dónde—
                // pero sí va a la bitácora: el botón de cancelar suscripción y
                // una respuesta a mano son el mismo acto por dos caminos, y
                // saber cuál fue es lo que permite contestar «¿cómo pidió la
                // baja esta persona?» meses después.
                bajas.push(BajaQueLlego {
                    direccion: correo.de.clone(),
                    uid: correo.uid,
                    por_el_asunto: matches!(como, ComoLoPidio::ElAsunto),
                });
            }
            Veredicto::MejorQueLoMireAlguien => para_mirar += 1,
            Veredicto::NoEsUnaBaja => {}
        }
    }

    // ── 3 · ESCRIBIR, Y 4 · REGISTRAR ──
    //
    // Las dos cosas van dentro de `aplicar_bajas_del_buzon`, en una sola
    // transacción con el avance de la marca.
    //
    // `hasta_uid` es el **mayor UID mirado**, no el mayor que dio una baja: si
    // fuera el segundo, los correos que no eran bajas se volverían a leer en
    // cada pasada para nada.
    let hasta_uid = nuevos.iter().map(|c| c.uid).max();
    let resumen = db
        .aplicar_bajas_del_buzon(empresa, buzon.cuenta, &bajas, para_mirar, hasta_uid)
        .map_err(|e| ErrorIpc::from(AppError::Db(e)))?;

    Ok(LoQueTrajoUnBuzon {
        buzon: buzon.nombre_visible.clone(),
        mirados: u32::try_from(nuevos.len()).unwrap_or(u32::MAX),
        dadas_de_baja: resumen.dadas_de_baja,
        ya_estaban: resumen.ya_estaban,
        para_mirar: resumen.para_mirar,
        no_se_pudo: None,
    })
}

/// Enciende o apaga la lectura del buzón de una cuenta.
///
/// El servidor llega como texto porque **ARLES no lo deduce**: la pantalla
/// propone el valor que saldría de cambiar `smtp.` por `imap.` y quien lo mira
/// lo confirma o lo corrige. Ver la cabecera de la migración V18.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa, si esa cuenta no es de esta empresa, o si la
/// base falla.
#[tauri::command]
pub fn configurar_la_lectura_del_buzon(
    estado: tauri::State<'_, EstadoApp>,
    cuenta: String,
    servidor: String,
    encendido: bool,
) -> Result<(), ErrorIpc> {
    let db = estado.db();
    let empresa = db
        .empresa()
        .map_err(|e| ErrorIpc::from(AppError::Db(e)))?
        .ok_or_else(|| ErrorIpc::from(AppError::EmpresaNoConfigurada))?;

    let id: EmailAccountId = cuenta.parse().map_err(|_| {
        ErrorIpc::from(AppError::CuentaNoSirve(
            crate::fabrica::LaCuentaNoSirve::NoExiste,
        ))
    })?;

    let limpio = servidor.trim();
    db.configurar_la_lectura_del_buzon(
        empresa.id,
        id,
        // Vacío es `NULL`: «no lo sé». Guardar una cadena vacía dejaría una
        // cuenta encendida con un servidor que no existe, y el error llegaría
        // al intentar conectarse en vez de al guardar.
        if limpio.is_empty() {
            None
        } else {
            Some(limpio)
        },
        encendido,
    )
    .map_err(|e| ErrorIpc::from(AppError::Db(e)))?;
    Ok(())
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use arles_email::servidor_imap::{CorreoGuardado, GuionDelBuzon, ServidorImap};

    /// El lector que habla con el servidor de mentira.
    ///
    /// Confía **además** en su certificado: no desactiva la verificación. Ver
    /// `arles_email::buzon::correos_recientes_confiando_en`.
    struct BuzonDeMentira {
        puerto: u16,
        certificado: arles_email::buzon::CertificadoDePrueba<'static>,
    }

    impl LeeUnBuzon for BuzonDeMentira {
        fn correos_de(
            &self,
            cuenta: &CuentaDeLectura,
            contrasena: &str,
            dias: u16,
        ) -> Result<Vec<CorreoDelBuzon>, arles_email::ErrorDeCorreo> {
            let apuntando = CuentaDeLectura {
                servidor: "localhost".to_owned(),
                puerto: Some(self.puerto),
                usuario: cuenta.usuario.clone(),
            };
            arles_email::buzon::correos_recientes_confiando_en(
                &apuntando,
                contrasena,
                dias,
                self.certificado.clone(),
            )
        }
    }

    struct UnaContrasena;
    impl FuenteDeContrasenas for UnaContrasena {
        fn contrasena_de(&self, _id: &str) -> Result<Option<String>, AppError> {
            Ok(Some("la-contrasena".to_owned()))
        }

        // El buzón no usa tokens de WhatsApp. Devolver `None` y no un valor
        // cualquiera: si algún día este cable los pidiera, la prueba fallaría
        // en vez de pasar con un secreto inventado.
        fn token_del_numero(&self, _id: &str) -> Result<Option<String>, AppError> {
            Ok(None)
        }
    }

    struct SinContrasena;
    impl FuenteDeContrasenas for SinContrasena {
        fn contrasena_de(&self, _id: &str) -> Result<Option<String>, AppError> {
            Ok(None)
        }

        fn token_del_numero(&self, _id: &str) -> Result<Option<String>, AppError> {
            Ok(None)
        }
    }

    /// Una base con empresa, una cuenta de correo y el buzón encendido.
    fn base_con_buzon(puerto: u16) -> (tempfile::TempDir, arles_db::Db, EmailAccountId) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = arles_db::ClaveMaestra::generar().expect("genera");
        let db = arles_db::Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        let empresa = db
            .guardar_empresa(
                &arles_core::empresa::DatosDeEmpresa::validar(
                    &arles_core::empresa::BorradorDeEmpresa {
                        nombre_comercial: "TELEMETRY".into(),
                        pais: "MX".into(),
                        zona_horaria: "America/Mexico_City".into(),
                        correo_corporativo: "hola@telemetrymx.com".into(),
                        sitio_web: String::new(),
                    },
                )
                .expect("válidos"),
            )
            .expect("empresa");

        let cuenta = EmailAccountId::nuevo();
        let datos =
            arles_core::correo::DatosDeLaCuenta::validar(&arles_core::correo::BorradorDeCuenta {
                servidor: "smtp.telemetrymx.com".into(),
                puerto: "587".into(),
                cifrado: "starttls".into(),
                usuario: "enlace@telemetrymx.com".into(),
                remitente: "enlace@telemetrymx.com".into(),
                nombre_visible: "Enlace".into(),
            })
            .expect("cuenta válida");
        db.alta_de_cuenta_de_correo(
            empresa,
            cuenta,
            &datos,
            arles_db::correo::LimitesDeLaCuenta::default(),
            "llavero://cuenta-1",
        )
        .expect("alta");

        // El puerto va en el lector de mentira, no aquí: lo que se guarda es
        // el servidor de verdad, como en producción.
        let _ = puerto;
        db.configurar_la_lectura_del_buzon(empresa, cuenta, Some("imap.telemetrymx.com"), true)
            .expect("enciende");
        (dir, db, cuenta)
    }

    fn lector(s: &ServidorImap) -> BuzonDeMentira {
        BuzonDeMentira {
            puerto: s.direccion().port(),
            certificado: s.certificado(),
        }
    }

    /// Las bajas que hay en la lista, leídas por la API pública.
    fn bajas_de(db: &arles_db::Db) -> Vec<arles_db::supresion::SupresionGuardada> {
        let empresa = db.empresa().expect("lee").expect("hay").id;
        db.listar_supresiones(empresa, 0, 100).expect("lista")
    }

    fn cuantas_bajas(db: &arles_db::Db) -> usize {
        bajas_de(db).len()
    }

    /// ── LA PRUEBA QUE EL TRAMO EXISTÍA PARA TENER ─────────────────────────
    ///
    /// Un correo en el buzón, y una dirección en la lista de supresión. Las
    /// tres piezas tenían sus pruebas —el núcleo juzga, el lector trae, la base
    /// escribe— y **nadie comprobaba que lo que sale de una entre en la
    /// siguiente**. Es el patrón del §9, y esto es lo único que lo cierra.
    #[test]
    fn un_correo_de_baja_en_el_buzon_acaba_en_la_lista_de_supresion() {
        let s = ServidorImap::arrancar(GuionDelBuzon::con(vec![CorreoGuardado::de(
            7,
            "\"Ana Pérez\" <ana@ejemplo.mx>",
            "BAJA",
            "",
        )]))
        .expect("arranca");
        let (_d, db, _cuenta) = base_con_buzon(s.direccion().port());

        let salida = revisar_con(&db, &lector(&s), &UnaContrasena).expect("revisa");

        assert_eq!(salida.len(), 1, "un buzón");
        let uno = salida.first().expect("uno");
        assert_eq!(uno.no_se_pudo, None, "el buzón contestó");
        assert_eq!(uno.mirados, 1);
        assert_eq!(uno.dadas_de_baja, 1);
        assert_eq!(uno.para_mirar, 0);
        assert_eq!(cuantas_bajas(&db), 1, "la baja está en la base");
    }

    /// Lo dudoso **no da de baja a nadie** y sale en su cifra. Es el caso que
    /// justifica que el núcleo tenga tres veredictos y no dos.
    #[test]
    fn una_frase_dudosa_no_da_de_baja_y_queda_para_mirar() {
        let s = ServidorImap::arrancar(GuionDelBuzon::con(vec![CorreoGuardado::de(
            9,
            "beto@ejemplo.mx",
            "Re: Oferta",
            "No quiero darme de baja, me encanta lo que mandan\r\n",
        )]))
        .expect("arranca");
        let (_d, db, _cuenta) = base_con_buzon(s.direccion().port());

        let salida = revisar_con(&db, &lector(&s), &UnaContrasena).expect("revisa");
        let uno = salida.first().expect("uno");
        assert_eq!(uno.dadas_de_baja, 0);
        assert_eq!(uno.para_mirar, 1);
        assert_eq!(cuantas_bajas(&db), 0, "nadie salió de la lista de envío");
    }

    /// Un aviso de ausencia con la palabra dentro: ni baja, ni dudoso.
    #[test]
    fn un_aviso_de_ausencia_no_cuenta_para_nada() {
        let s = ServidorImap::arrancar(GuionDelBuzon::con(vec![CorreoGuardado::automatico(
            11,
            "dani@ejemplo.mx",
            "Automatic reply",
            "Estoy de baja por maternidad hasta marzo.\r\n",
        )]))
        .expect("arranca");
        let (_d, db, _cuenta) = base_con_buzon(s.direccion().port());

        let salida = revisar_con(&db, &lector(&s), &UnaContrasena).expect("revisa");
        let uno = salida.first().expect("uno");
        assert_eq!((uno.dadas_de_baja, uno.para_mirar), (0, 0));
        assert_eq!(cuantas_bajas(&db), 0);
    }

    /// ── LA MARCA DEL BUZÓN, MEDIDA DE PUNTA A PUNTA ───────────────────────
    ///
    /// La segunda pasada sobre el mismo buzón **no vuelve a mirar nada**. Sin
    /// esto, cada revisión registraría otra vez las mismas bajas en la
    /// bitácora, y el registro de por qué alguien está en la lista se llenaría
    /// de copias del mismo correo.
    #[test]
    fn la_segunda_pasada_no_vuelve_a_mirar_lo_mismo() {
        let s = ServidorImap::arrancar(GuionDelBuzon::con(vec![CorreoGuardado::de(
            7,
            "ana@ejemplo.mx",
            "BAJA",
            "",
        )]))
        .expect("arranca");
        let (_d, db, _cuenta) = base_con_buzon(s.direccion().port());

        let primera = revisar_con(&db, &lector(&s), &UnaContrasena).expect("primera");
        let segunda = revisar_con(&db, &lector(&s), &UnaContrasena).expect("segunda");

        assert_eq!(primera.first().expect("uno").mirados, 1);
        assert_eq!(
            segunda.first().expect("uno").mirados,
            0,
            "el UID ya estaba procesado"
        );
        assert_eq!(cuantas_bajas(&db), 1, "sigue habiendo una baja, no dos");
    }

    /// **Un buzón que no se puede abrir no tumba la revisión**, y lo dice con
    /// su clave (§139) en vez de con un error que pare todo.
    #[test]
    fn sin_contrasena_lo_dice_y_no_toca_nada() {
        let s = ServidorImap::arrancar(GuionDelBuzon::con(Vec::new())).expect("arranca");
        let (_d, db, _cuenta) = base_con_buzon(s.direccion().port());

        let salida = revisar_con(&db, &lector(&s), &SinContrasena).expect("revisa");
        let uno = salida.first().expect("uno");
        assert_eq!(
            uno.no_se_pudo.as_deref(),
            Some("error.app.cuenta.sin_contrasena")
        );
        assert_eq!(uno.dadas_de_baja, 0);
        // Y no se habló con ningún servidor: sin contraseña no hay a qué entrar.
        assert!(!s.dijo("LOGIN"), "intentó entrar sin contraseña");
    }

    /// Una contraseña que el servidor rechaza tiene su propia clave, porque el
    /// arreglo es otro: activar IMAP en el proveedor.
    #[test]
    fn una_contrasena_rechazada_lo_dice_con_su_clave() {
        let s = ServidorImap::arrancar(GuionDelBuzon::contrasena_rechazada()).expect("arranca");
        let (_d, db, _cuenta) = base_con_buzon(s.direccion().port());

        let salida = revisar_con(&db, &lector(&s), &UnaContrasena).expect("revisa");
        assert_eq!(
            salida.first().expect("uno").no_se_pudo.as_deref(),
            Some("error.correo.buzon_rechazo_contrasena")
        );
    }

    /// Un buzón apagado no se revisa, y por eso la lista vuelve vacía.
    #[test]
    fn un_buzon_apagado_no_se_revisa() {
        let s = ServidorImap::arrancar(GuionDelBuzon::con(vec![CorreoGuardado::de(
            7,
            "ana@ejemplo.mx",
            "BAJA",
            "",
        )]))
        .expect("arranca");
        let (_d, db, cuenta) = base_con_buzon(s.direccion().port());
        let empresa = db.empresa().expect("lee").expect("hay").id;
        db.configurar_la_lectura_del_buzon(empresa, cuenta, Some("imap.telemetrymx.com"), false)
            .expect("apaga");

        let salida = revisar_con(&db, &lector(&s), &UnaContrasena).expect("revisa");
        assert!(salida.is_empty(), "no había buzones encendidos");
        assert_eq!(cuantas_bajas(&db), 0);
        assert!(!s.dijo("LOGIN"), "no debería haber entrado a ningún buzón");
    }

    /// ── DOS CORREOS A LA VEZ, Y CADA UNO CON SU DESTINO ───────────────────
    ///
    /// **Dos y no uno a propósito**: con uno solo no se ve que cada veredicto
    /// vaya con su dirección — y cruzarlos aquí daría de baja a quien
    /// preguntaba el precio. Es la lección de D-WA, cuya primera prueba de
    /// WhatsApp manda a dos contactos por lo mismo.
    #[test]
    fn de_dos_correos_sale_de_baja_sólo_el_que_la_pidió() {
        let s = ServidorImap::arrancar(GuionDelBuzon::con(vec![
            CorreoGuardado::de(10, "ana@ejemplo.mx", "Re: Oferta", "BAJA\r\n"),
            CorreoGuardado::de(
                20,
                "beto@ejemplo.mx",
                "Re: Oferta",
                "Gracias, me interesa. ¿Me llamas?\r\n",
            ),
        ]))
        .expect("arranca");
        let (_d, db, _cuenta) = base_con_buzon(s.direccion().port());

        let salida = revisar_con(&db, &lector(&s), &UnaContrasena).expect("revisa");
        let uno = salida.first().expect("uno");
        assert_eq!(uno.mirados, 2);
        assert_eq!(uno.dadas_de_baja, 1);
        assert_eq!(cuantas_bajas(&db), 1);

        // Y es Ana, no Beto. Sin esto, cruzar los dos veredictos pasaría en
        // verde con las cifras correctas.
        let bajas = bajas_de(&db);
        let quien = bajas.first().expect("hay una baja");
        assert_eq!(quien.direccion, "ana@ejemplo.mx");
        // Y con el origen que la distingue de una apuntada a mano.
        assert_eq!(
            quien.origen,
            arles_core::supresion::OrigenDeSupresion::PeticionPorCorreo
        );
    }
}
