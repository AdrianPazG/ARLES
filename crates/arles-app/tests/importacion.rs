//! El camino completo de una importación, de punta a punta.
//!
//! Igual que `empresa.rs`, y por la misma razón: cada capa está probada por su
//! lado —el núcleo mapea, `arles-import` lee, `arles-db` escribe— y **nada
//! recorría el camino entero**. Una frontera mal puesta entre dos capas
//! probadas pasa desapercibida, porque cada test dice que su lado funciona.
//!
//! Se lee un archivo real del disco y se importa contra una base cifrada de
//! verdad, con sus migraciones. Queda fuera el diálogo de Tauri —que sólo elige
//! la ruta— y **el llavero**, a propósito:
//!
//! Las pruebas de `empresa.rs` arrancan con `EstadoApp` y se saltan solas
//! cuando la máquina no tiene llavero, diciéndolo. Es honesto, pero significa
//! que en un servidor de integración continua sin sesión gráfica **no se
//! ejercitan**. Aquí la clave se genera a mano y la base se abre directa: la
//! importación no depende del llavero, así que exigirlo convertiría esta prueba
//! en otra que se salta sola y no comprueba nada.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::io::Write;

use arles_core::{BorradorDeEmpresa, CampoImportable, DatosDeEmpresa, OrigenDeLaLista};
use arles_db::{ClaveMaestra, Db};

fn entorno(_nombre: &str) -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().expect("directorio temporal");
    let clave = ClaveMaestra::generar().expect("genera clave");
    let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre y migra");

    let datos = DatosDeEmpresa::validar(&BorradorDeEmpresa {
        nombre_comercial: "TELEMETRY INSIGHT".into(),
        pais: "MX".into(),
        zona_horaria: "America/Mexico_City".into(),
        correo_corporativo: "hola@telemetrymx.com".into(),
        sitio_web: String::new(),
    })
    .expect("empresa válida");
    db.guardar_empresa(&datos).expect("guarda empresa");

    (dir, db)
}

/// Deja un CSV en disco y devuelve su ruta.
fn csv(dir: &tempfile::TempDir, nombre: &str, contenido: &str) -> std::path::PathBuf {
    let ruta = dir.path().join(nombre);
    let mut f = std::fs::File::create(&ruta).expect("crea");
    f.write_all(contenido.as_bytes()).expect("escribe");
    ruta
}

/// El recorrido entero: leer un archivo, proponer el mapeo, analizar y escribir.
///
/// El archivo es el de un cliente real en lo que importa: encabezados en
/// español con acentos, el Excel en español con **punto y coma**, un móvil
/// escrito con el «1» mexicano, una fila sin forma de contacto y una dirección
/// repetida dentro del propio archivo.
#[test]
fn importar_un_archivo_de_verdad_de_punta_a_punta() {
    let (dir, db) = entorno("camino");

    let ruta = csv(
        &dir,
        "contactos-marzo.csv",
        "Nombre;Apellidos;Razón social;Correo electrónico;Teléfono móvil\n\
         Ana;Ruiz;Empresa SA;Ana@Empresa.MX;+52 1 81 1234 5678\n\
         Luis;Pérez;Otra SA;luis@empresa.mx;\n\
         Sin;Contacto;Tercera SA;;\n\
         Repe;Tida;Cuarta SA;ana@empresa.mx;\n",
    );

    // ── 1 · Leer ────────────────────────────────────────────────────────
    let tabla = arles_import::leer(&ruta).expect("lee el archivo");
    assert_eq!(tabla.nombre, "contactos-marzo.csv");
    assert_eq!(
        tabla.encabezados.len(),
        5,
        "el punto y coma del Excel en español no se detectó: {:?}",
        tabla.encabezados
    );
    assert_eq!(tabla.total_de_filas, 4);

    // ── 2 · El mapeo se propone solo ────────────────────────────────────
    let mapeo = arles_core::proponer_mapeo(&tabla.encabezados);
    assert_eq!(
        mapeo,
        vec![
            CampoImportable::Nombre,
            CampoImportable::Apellido,
            CampoImportable::Empresa,
            CampoImportable::Correo,
            CampoImportable::WhatsApp,
        ],
        "el mapeo automático no reconoció los encabezados en español"
    );

    // ── 3 · Analizar, sin escribir nada ─────────────────────────────────
    let direcciones: Vec<String> = tabla
        .filas
        .iter()
        .filter_map(|f| arles_core::fila_a_borrador(&mapeo, f).ok())
        .filter_map(|b| arles_core::contacto::DatosDeContacto::validar(&b, "MX").ok())
        .flat_map(|d| {
            d.canales
                .into_iter()
                .map(|c| c.valor_normalizado)
                .collect::<Vec<_>>()
        })
        .collect();

    let empresa = db.empresa().expect("lee empresa").expect("hay empresa").id;
    let duenos = db
        .duenos_de_direcciones(empresa, &direcciones)
        .expect("consulta");
    let suprimidas = db
        .motivos_de_supresion(empresa, &direcciones)
        .expect("consulta las supresiones");

    let informe = arles_core::analizar(
        &mapeo,
        &tabla.filas,
        "MX",
        &duenos,
        &suprimidas,
        &Default::default(),
    );

    assert_eq!(informe.cuantas_listas(), 2, "deberían entrar Ana y Luis");
    assert_eq!(informe.rechazadas.len(), 1, "la fila sin contacto");
    assert_eq!(informe.choques.len(), 1, "la repetida dentro del archivo");
    assert!(
        informe
            .choques
            .first()
            .is_some_and(|c| c.dentro_del_archivo),
        "el choque tiene que ser contra el propio archivo, no contra la base"
    );

    // Y la base sigue intacta: analizar no escribe.
    assert_eq!(
        db.listar_contactos(empresa, 0, 100).expect("lista").total,
        0,
        "analizar escribió en la base"
    );

    // ── 4 · Confirmar ───────────────────────────────────────────────────
    let lote = arles_db::DatosDelLote {
        nombre_del_archivo: tabla.nombre.clone(),
        huella_del_archivo: tabla.huella.clone(),
        mapeo_en_json: serde_json::to_string(&mapeo).expect("serializa"),
        texto_del_consentimiento: "Declaro que esta lista tiene origen lícito. [TEXTO PROVISIONAL]"
            .into(),
        origen: OrigenDeLaLista::EventoOFeria,
        total_de_filas: tabla.total_de_filas,
        choques: informe.choques.len(),
        invalidas: informe.invalidas.len() + informe.rechazadas.len(),
        lista: None,
        direcciones: Vec::new(),
    };
    let resumen = db
        .importar_contactos(empresa, &lote, &informe.listas)
        .expect("importa");

    assert_eq!(resumen.importados, 2);

    // ── 5 · Y lo importado es lo que se leyó ────────────────────────────
    let pagina = db.listar_contactos(empresa, 0, 100).expect("lista");
    assert_eq!(pagina.total, 2);

    let ana = pagina
        .contactos
        .iter()
        .find(|c| c.datos.nombre == "Ana")
        .expect("Ana entró");
    assert_eq!(ana.datos.apellido, "Ruiz");
    assert_eq!(ana.datos.empresa, "Empresa SA");

    let correo = ana
        .datos
        .canales
        .iter()
        .find(|c| c.canal == arles_core::Canal::Correo)
        .expect("tiene correo");
    assert_eq!(
        correo.valor_normalizado, "ana@empresa.mx",
        "el correo no se normalizó a minúsculas"
    );
    assert_eq!(
        correo.valor_raw, "Ana@Empresa.MX",
        "se perdió lo que el archivo decía"
    );

    let movil = ana
        .datos
        .canales
        .iter()
        .find(|c| c.canal == arles_core::Canal::WhatsApp)
        .expect("tiene móvil");
    assert_eq!(
        movil.valor_normalizado, "+528112345678",
        "el «1» del móvil mexicano sobrevivió a la importación"
    );
}

/// Reimportar el mismo archivo **no duplica**: la segunda vez, todas sus
/// direcciones chocan contra la base y no entra nadie.
///
/// Es el error que más veces se comete con una importación: volver a subir el
/// archivo «por si acaso». Sin la detección de choques, la lista se duplicaría
/// entera y cada contacto recibiría la campaña dos veces.
#[test]
fn reimportar_el_mismo_archivo_no_duplica_a_nadie() {
    let (dir, db) = entorno("reimportar");
    let ruta = csv(
        &dir,
        "lista.csv",
        "Nombre,Correo\nAna,ana@empresa.mx\nLuis,luis@empresa.mx\n",
    );

    let tabla = arles_import::leer(&ruta).expect("lee");
    let mapeo = arles_core::proponer_mapeo(&tabla.encabezados);
    let empresa = db.empresa().expect("lee").expect("hay").id;

    let lote = |informe: &arles_core::Analisis| arles_db::DatosDelLote {
        nombre_del_archivo: tabla.nombre.clone(),
        huella_del_archivo: tabla.huella.clone(),
        mapeo_en_json: "[]".into(),
        texto_del_consentimiento: "Declaro origen lícito. [PROVISIONAL]".into(),
        origen: OrigenDeLaLista::ClientesExistentes,
        total_de_filas: tabla.total_de_filas,
        choques: informe.choques.len(),
        invalidas: informe.invalidas.len() + informe.rechazadas.len(),
        lista: None,
        direcciones: Vec::new(),
    };

    // Primera vez: entran los dos.
    let primero = arles_core::analizar(
        &mapeo,
        &tabla.filas,
        "MX",
        &Default::default(),
        &Default::default(),
        &Default::default(),
    );
    assert_eq!(primero.cuantas_listas(), 2);
    db.importar_contactos(empresa, &lote(&primero), &primero.listas)
        .expect("primera importación");

    // Segunda vez: el mismo archivo, sin tocar nada.
    let direcciones = vec!["ana@empresa.mx".to_owned(), "luis@empresa.mx".to_owned()];
    let duenos = db
        .duenos_de_direcciones(empresa, &direcciones)
        .expect("consulta");
    let suprimidas = db
        .motivos_de_supresion(empresa, &direcciones)
        .expect("consulta las supresiones");
    let segundo = arles_core::analizar(
        &mapeo,
        &tabla.filas,
        "MX",
        &duenos,
        &suprimidas,
        &Default::default(),
    );

    assert_eq!(
        segundo.cuantas_listas(),
        0,
        "la reimportación iba a duplicar {} contactos",
        segundo.cuantas_listas()
    );
    assert_eq!(segundo.choques.len(), 2);
    assert!(
        segundo.choques.iter().all(|c| !c.dentro_del_archivo),
        "los choques tienen que ser contra la base, no contra el archivo"
    );

    // Y el informe dice con quién choca cada una, no sólo que choca.
    assert!(
        segundo.choques.iter().any(|c| c.con == "Ana"),
        "el choque no nombra al contacto que ya tiene la dirección: {:?}",
        segundo.choques.iter().map(|c| &c.con).collect::<Vec<_>>()
    );

    // La base sigue con dos, no con cuatro.
    assert_eq!(
        db.listar_contactos(empresa, 0, 100).expect("lista").total,
        2
    );
}

/// **El caso que Dirección pidió ver el 21/09/2026.**
///
/// ─────────────────────────────────────────────────────────────────────────
/// LA CIFRA NO PUEDE MENTIR
///
/// Un archivo de la feria donde dos de las cuatro personas ya habían pedido no
/// recibir. Lo que se exige es que el informe **lo diga antes de importar**, y
/// que distinga las dos situaciones que parecen la misma y no lo son:
///
///   - Ana cerró su correo **pero tiene WhatsApp**: sigue siendo alcanzable.
///   - Luis cerró lo único que tenía: no lo es.
///
/// Contar las dos igual daría una cifra tan falsa como callarlas. Y la lista
/// que Dirección quiere —quién canceló, por qué canal— sale de la misma tabla.
/// ─────────────────────────────────────────────────────────────────────────
#[test]
fn el_informe_avisa_de_quien_ya_habia_pedido_no_recibir() {
    use arles_core::supresion::{
        AlcanceDeSupresion, MotivoDeSupresion, OrigenDeSupresion, PeticionDeSupresion,
    };

    let (dir, db) = entorno("supresion");
    let empresa = db.empresa().expect("lee empresa").expect("hay empresa").id;

    // Dos personas que en su día dijeron que no.
    for (correo, motivo) in [
        ("ana@empresa.mx", MotivoDeSupresion::Baja),
        ("luis@empresa.mx", MotivoDeSupresion::Queja),
    ] {
        let valor = arles_core::canal::ValorDeCanal::parse(arles_core::Canal::Correo, correo, "MX")
            .expect("correo válido");
        let canal = arles_core::contacto::CanalValidado {
            canal: valor.canal(),
            valor_raw: valor.raw().to_owned(),
            valor_normalizado: valor.normalized().to_owned(),
            principal: true,
        };
        db.suprimir(
            empresa,
            &PeticionDeSupresion::nueva(
                vec![canal],
                AlcanceDeSupresion::Canal,
                motivo,
                OrigenDeSupresion::Usuario,
                "",
            )
            .expect("petición válida"),
        )
        .expect("suprime");
    }

    // Y meses después llega el archivo de la feria, con las dos dentro.
    let ruta = csv(
        &dir,
        "feria-septiembre.csv",
        "Nombre;Correo electrónico;Teléfono móvil\n\
         Ana;ana@empresa.mx;81 1234 5678\n\
         Luis;luis@empresa.mx;\n\
         Sofía;sofia@empresa.mx;\n\
         Mario;mario@empresa.mx;81 9876 5432\n",
    );

    let tabla = arles_import::leer(&ruta).expect("lee");
    let mapeo = arles_core::proponer_mapeo(&tabla.encabezados);

    let direcciones: Vec<String> = tabla
        .filas
        .iter()
        .filter_map(|f| arles_core::fila_a_borrador(&mapeo, f).ok())
        .filter_map(|b| arles_core::contacto::DatosDeContacto::validar(&b, "MX").ok())
        .flat_map(|d| {
            d.canales
                .into_iter()
                .map(|c| c.valor_normalizado)
                .collect::<Vec<_>>()
        })
        .collect();

    let duenos = db
        .duenos_de_direcciones(empresa, &direcciones)
        .expect("consulta");
    let suprimidas = db
        .motivos_de_supresion(empresa, &direcciones)
        .expect("consulta las supresiones");

    let informe = arles_core::analizar(
        &mapeo,
        &tabla.filas,
        "MX",
        &duenos,
        &suprimidas,
        &Default::default(),
    );

    // Las cuatro entran: el nombre y la empresa valen igual.
    assert_eq!(informe.cuantas_listas(), 4, "las cuatro tenían que entrar");

    // Pero el informe dice quiénes son, y por qué.
    assert_eq!(
        informe.suprimidas.len(),
        2,
        "el informe no avisó de las dos que pidieron no recibir"
    );
    let ana = informe
        .suprimidas
        .iter()
        .find(|s| s.direccion == "ana@empresa.mx")
        .expect("Ana tenía que estar avisada");
    let luis = informe
        .suprimidas
        .iter()
        .find(|s| s.direccion == "luis@empresa.mx")
        .expect("Luis tenía que estar avisado");

    assert_eq!(ana.motivo, MotivoDeSupresion::Baja.clave_i18n());
    assert_eq!(luis.motivo, MotivoDeSupresion::Queja.clave_i18n());

    // Y la distinción: a Ana se le puede escribir por WhatsApp; a Luis, no.
    assert!(
        !ana.sin_ningun_canal_libre,
        "a Ana le queda el WhatsApp y se la dio por incomunicada"
    );
    assert!(
        luis.sin_ningun_canal_libre,
        "Luis no tiene ningún canal libre y se contó como alcanzable"
    );

    assert_eq!(
        informe.cuantas_se_pueden_alcanzar(),
        3,
        "la cifra de a cuántos se les puede escribir no cuadra: entran 4, \
         uno se quedó sin ningún canal"
    );
}
