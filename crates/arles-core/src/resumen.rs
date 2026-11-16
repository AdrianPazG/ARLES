//! El resumen de una campaña, para llevárselo fuera de ARLES · entrega 7.6.
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! QUÉ ES ESTO, Y POR QUÉ NO ES UNA HOJA DE CÁLCULO
//!
//! Alguien termina una campaña y quiere enseñar cómo fue: a Dirección, a un
//! cliente, a sí mismo dentro de seis meses. Esto arma ese archivo.
//!
//! **Y va con el método de cada número al lado**, que es la decisión que
//! define este módulo. Un CSV con «aceptados: 1 342» es exactamente lo que el
//! §65 existe para impedir: alguien lo pega en una presentación, y para cuando
//! llega a la tercera reunión la cifra ya se llama «entregados». Nadie mintió;
//! el número viajó sin su significado.
//!
//! Aquí cada cifra lleva la frase que dice qué es **y qué no es**. Ocupa más y
//! no se puede sumar en Excel, y las dos cosas son a propósito.
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! CERO I/O, COMO TODO `arles-core`
//!
//! Esto arma un `String`. No abre archivos, no pregunta a la base y no sabe
//! dónde se va a guardar — de eso se encarga `arles-app`, con el diálogo del
//! sistema, igual que el informe de acceso ARCO (regla 4 de la frontera:
//! ninguna ruta la cruza).

/// Cuántas veces falló algo, por su causa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FalloPorCausa {
    /// La clave del error, tal como la guardó el motor en `error_kind`.
    pub clave: String,
    pub cuantos: u32,
}

/// Todo lo que hace falta para redactar el resumen de una campaña.
///
/// Es un montón de datos sin lógica a propósito: quien lo llena es `arles-app`,
/// leyendo de `arles-db`, y quien lo lee es `redactar`. Ninguna de las dos
/// tiene que saber de la otra.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResumenDeCampana {
    pub campana: String,
    pub empresa: String,
    /// La fecha en que se genera el archivo, ya legible. El núcleo no tiene
    /// reloj: se la pasan.
    pub fecha: String,
    /// El rótulo del estado, ya en español. No la clave.
    pub estado: String,
    /// `"correo electrónico"` o `"WhatsApp"`.
    pub canal: String,

    pub audiencia: u32,
    pub salio: u32,
    pub pendiente: u32,
    pub fallo: u32,
    pub dado_de_baja: u32,
    pub cancelado: u32,
    pub respondieron: u32,

    /// Los fallos agrupados por causa, de más a menos.
    pub causas: Vec<FalloPorCausa>,
}

impl ResumenDeCampana {
    /// Cuántos intentos hubo en total.
    #[must_use]
    pub const fn intentos(&self) -> u32 {
        self.salio + self.pendiente + self.fallo + self.dado_de_baja + self.cancelado
    }
}

/// La frase que explica una causa de fallo, en español.
///
/// ───────────────────────────────────────────────────────────────────────────
/// POR QUÉ ESTOS TEXTOS ESTÁN EN RUST Y NO EN `es.ts`
///
/// Porque esto **no es una pantalla**: es un archivo que alguien abre en el
/// Bloc de notas, fuera de ARLES. El §139 manda las claves a la interfaz
/// porque ahí hay un catálogo que traduce; aquí no hay ninguno, y mandar la
/// clave cruda —`reintento.direccionInexistente`— a un archivo que va a leer
/// Dirección sería peor que cualquier duplicación.
///
/// Es el mismo criterio que sigue `arco.rs` con el informe de acceso, que
/// escribe «correo electrónico» y «WhatsApp» directamente.
///
/// Devuelve `None` si la clave no se conoce, y quien redacta decide qué hacer.
/// **No se inventa una frase**: un archivo que explica mal una causa es peor
/// que uno que dice que no la conoce.
#[must_use]
pub fn causa_legible(clave: &str) -> Option<&'static str> {
    Some(match clave {
        "reintento.seVolveraAIntentar" => {
            "Falló y se volvió a intentar. No es un fallo definitivo."
        }
        "reintento.seAgotaronLosIntentos" => {
            "Se agotaron los intentos. El servidor no aceptó el mensaje \
             después de reintentarlo."
        }
        "reintento.direccionInexistente" => {
            "La dirección no existe. Se añadió a la lista de bajas y no se le \
             volverá a escribir."
        }
        "reintento.laCuentaNoAutentica" => {
            "La cuenta remitente dejó de autenticar. Se detuvo el envío para \
             no seguir fallando."
        }
        "reintento.pidioLaBaja" => {
            "Esa persona pidió no recibir. No es un fallo: es una baja \
             respetada."
        }
        _ => return None,
    })
}

/// El resumen, en texto.
///
/// ───────────────────────────────────────────────────────────────────────────
/// EL ORDEN NO ES CASUAL
///
/// Primero lo que se hizo, después lo que salió, después lo que no y por qué.
/// Quien abre este archivo busca una de dos cosas: «¿cómo fue?» —y con las
/// primeras diez líneas le basta— o «¿por qué falló tanto?», que está justo
/// debajo con sus causas.
///
/// Lo que **no** lleva, y consta por escrito al final: a quién se le escribió.
/// Eso está en el registro, dentro de ARLES, y sacarlo a un archivo que se
/// manda por correo es convertir una lista de contactos en un adjunto.
#[must_use]
pub fn redactar(r: &ResumenDeCampana) -> String {
    let mut t = String::with_capacity(2048);

    t.push_str("RESUMEN DE CAMPAÑA\n");
    t.push_str("==================\n\n");
    t.push_str(&format!("Campaña: {}\n", r.campana));
    if !r.empresa.is_empty() {
        t.push_str(&format!("Empresa: {}\n", r.empresa));
    }
    t.push_str(&format!("Canal: {}\n", r.canal));
    t.push_str(&format!("Estado: {}\n", r.estado));
    t.push_str(&format!("Generado el: {}\n\n", r.fecha));

    // ── LAS CIFRAS, CADA UNA CON SU MÉTODO ─────────────────────────────────
    t.push_str("LO QUE PASÓ\n");
    t.push_str("-----------\n\n");

    t.push_str(&format!("Audiencia congelada: {}\n", r.audiencia));
    t.push_str(
        "  Cuánta gente entró en la campaña al activarla. A partir de ese\n  \
         momento la lista no cambia, aunque se añadan contactos después.\n\n",
    );

    t.push_str(&format!("Aceptados por el proveedor: {}\n", r.salio));
    // §65, escrito donde se lee la cifra y no en un pie de página.
    // ── SIN ASTERISCOS, Y ES EL MISMO ERROR DOS VECES ──────────────────
    //
    // Esto llevaba `**Esto no quiere decir…**`, y un archivo de texto plano no
    // interpreta Markdown: los asteriscos salían literales, en la frase más
    // importante del archivo. Es el mismo defecto que apareció en los textos
    // de la pantalla el mismo día — allí porque `vue-i18n` tampoco lo
    // interpreta. Se vio leyendo el archivo generado, no en ninguna prueba.
    //
    // En texto plano lo que resalta es una línea corta y sola.
    t.push_str(
        "  El servidor se hizo cargo del mensaje.\n  \
         NO quiere decir que llegara a una bandeja de entrada: ARLES no puede\n  \
         saberlo, y por eso no lo dice.\n\n",
    );

    t.push_str(&format!("Por enviar: {}\n", r.pendiente));
    t.push_str("  Todavía en la cola, incluidos los que se reintentarán.\n\n");

    t.push_str(&format!("Fallaron: {}\n", r.fallo));
    t.push_str("  No salieron y no van a salir. Las causas están más abajo.\n\n");

    t.push_str(&format!(
        "No se les escribió por estar de baja: {}\n",
        r.dado_de_baja
    ));
    t.push_str("  Habían pedido no recibir. No es un fallo: es una baja respetada.\n\n");

    t.push_str(&format!("Cancelados: {}\n", r.cancelado));
    t.push_str("  Estaban en la cola cuando se detuvo la campaña.\n\n");

    if r.respondieron > 0 {
        t.push_str(&format!("Contestaron: {}\n", r.respondieron));
        // D-7A, con sus palabras: la cifra, nunca el contenido.
        t.push_str(
            "  Cuántas personas respondieron. ARLES no guarda lo que dijeron:\n  \
             eso vive en la bandeja de quien atiende.\n\n",
        );
    }

    // ── LA SUMA, DICHA ─────────────────────────────────────────────────────
    //
    // Se escribe el total y **de qué se compone**, porque si no cuadra con la
    // audiencia alguien va a pensar que faltan datos. Y a veces no cuadra por
    // un motivo correcto: entre congelar la audiencia y crear los intentos,
    // una supresión puede dejar a alguien fuera.
    t.push_str(&format!("Intentos en total: {}\n", r.intentos()));
    t.push_str(
        "  Aceptados + por enviar + fallaron + bajas + cancelados.\n  \
         Puede no coincidir con la audiencia: entre congelarla y crear los\n  \
         intentos, una baja puede dejar a alguien fuera.\n\n",
    );

    // ── POR QUÉ FALLÓ ──────────────────────────────────────────────────────
    t.push_str("POR QUÉ FALLÓ LO QUE FALLÓ\n");
    t.push_str("--------------------------\n\n");

    if r.causas.is_empty() {
        t.push_str("No hay fallos registrados en esta campaña.\n\n");
    } else {
        for c in &r.causas {
            // Envuelta: el resto del archivo está ajustado a mano a unas 72
            // columnas, y una causa de 90 se sale en un Bloc de notas sin
            // ajuste de línea. Se vio leyendo el archivo generado.
            t.push_str(&format!("{} · ", c.cuantos));
            t.push_str(&envolver(&c.clave_o_frase(), ANCHO, "     "));
            t.push_str("\n\n");
        }
    }

    // ── LO QUE ESTE ARCHIVO NO LLEVA ───────────────────────────────────────
    t.push_str("QUÉ NO ESTÁ EN ESTE ARCHIVO\n");
    t.push_str("---------------------------\n\n");
    t.push_str(
        "A quién se le escribió. Esa lista está en ARLES, en Actividad ›\n\
         Registro, y no sale en un archivo que se manda por correo.\n\n",
    );
    t.push_str(
        "Si un correo se leyó, o si llegó a la bandeja principal o a spam.\n\
         ARLES no puede saberlo sin leer un buzón, y no lo hace.\n",
    );

    t
}

/// A cuántas columnas se ajusta el archivo.
///
/// Setenta y dos porque es lo que cabe en un Bloc de notas sin maximizar y en
/// un correo pegado como texto. El resto del archivo está escrito a mano a este
/// ancho; esto es para lo que se compone en tiempo de ejecución.
const ANCHO: usize = 72;

/// Parte un texto en líneas de como mucho `ancho`, sangrando las siguientes.
///
/// Sin dependencias: es cortar por espacios. Una caja de texto envuelta a mano
/// en el código no serviría aquí, porque la frase se elige al redactar.
fn envolver(texto: &str, ancho: usize, sangria: &str) -> String {
    let mut fuera = String::with_capacity(texto.len() + 16);
    let mut columna = sangria.len();

    for (i, palabra) in texto.split_whitespace().enumerate() {
        if i > 0 && columna + 1 + palabra.chars().count() > ancho {
            fuera.push('\n');
            fuera.push_str(sangria);
            columna = sangria.len();
        } else if i > 0 {
            fuera.push(' ');
            columna += 1;
        }
        fuera.push_str(palabra);
        columna += palabra.chars().count();
    }
    fuera
}

impl FalloPorCausa {
    /// La frase de la causa, o la clave si no se conoce.
    ///
    /// Enseñar la clave es feo y es **honesto**: dice que ARLES registró algo
    /// que este resumen todavía no sabe explicar. Inventar una frase genérica
    /// —«error del servidor»— sería peor: parecería una explicación.
    fn clave_o_frase(&self) -> String {
        causa_legible(&self.clave).map_or_else(
            || format!("Causa sin descripción: {}", self.clave),
            ToString::to_string,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn una_campana() -> ResumenDeCampana {
        ResumenDeCampana {
            campana: "Clientes de agosto".into(),
            empresa: "TELEMETRY".into(),
            fecha: "30 de septiembre de 2026".into(),
            estado: "Enviando".into(),
            canal: "correo electrónico".into(),
            audiencia: 1342,
            salio: 1200,
            pendiente: 100,
            fallo: 30,
            dado_de_baja: 12,
            cancelado: 0,
            respondieron: 23,
            causas: vec![FalloPorCausa {
                clave: "reintento.direccionInexistente".into(),
                cuantos: 30,
            }],
        }
    }

    /// **Ninguna cifra sale sola: cada una lleva su método.**
    ///
    /// Es la razón de ser de este módulo. Un archivo con «1 200» y nada más es
    /// el que acaba en una presentación llamándose «entregados».
    ///
    /// Verificada rompiéndola el 30/09/2026 quitando la explicación de
    /// «Aceptados»: falla porque la línea siguiente no empieza con sangría.
    #[test]
    fn cada_cifra_va_con_su_explicacion() {
        let t = redactar(&una_campana());
        let lineas: Vec<&str> = t.lines().collect();

        // Cada línea «Algo: 123» tiene debajo al menos una línea sangrada, que
        // es la explicación. Se comprueba la forma y no un texto concreto: los
        // textos se reescriben, la regla no.
        for (i, l) in lineas.iter().enumerate() {
            let es_cifra = l.contains(": ")
                && l.rsplit(": ")
                    .next()
                    .is_some_and(|v| v.parse::<u32>().is_ok())
                && !l.starts_with(' ');
            // La cabecera («Campaña: …») no lleva cifras, así que no entra.
            if !es_cifra {
                continue;
            }
            let siguiente = lineas.get(i + 1).copied().unwrap_or("");
            assert!(
                siguiente.starts_with("  "),
                "la cifra «{l}» sale sin explicación debajo"
            );
        }
    }

    /// **§65 · el archivo dice que «aceptado» no es «entregado».**
    ///
    /// Y no dice «entregado» en ninguna parte.
    #[test]
    fn el_archivo_no_promete_una_entrega() {
        let t = redactar(&una_campana());
        assert!(t.contains("NO quiere decir que llegara"));
        assert!(
            !t.to_lowercase().contains("entregado"),
            "§65: ARLES no sabe si un correo llegó, así que no lo dice"
        );
    }

    /// **Ninguna línea se sale del ancho de un Bloc de notas.**
    ///
    /// El archivo se lee en un Bloc de notas sin ajuste de línea y se pega en
    /// correos. Una línea de noventa columnas obliga a desplazarse en
    /// horizontal para leer una frase.
    ///
    /// Verificada rompiéndola el 30/09/2026 quitando el `envolver` de las
    /// causas: la línea de la dirección inexistente mide 91 y falla.
    #[test]
    fn ninguna_linea_se_sale_del_ancho() {
        let t = redactar(&una_campana());
        for l in t.lines() {
            assert!(
                l.chars().count() <= ANCHO + 4,
                "línea de {} columnas: «{l}»",
                l.chars().count()
            );
        }
    }

    /// **Y no lleva Markdown: un `.txt` no lo interpreta.**
    ///
    /// Los asteriscos salían literales en la frase más importante del archivo
    /// —la del §65— y se vio leyéndolo, no en ninguna prueba. Es el mismo
    /// defecto que apareció el mismo día en los textos de la pantalla.
    #[test]
    fn el_archivo_no_lleva_markdown() {
        let t = redactar(&una_campana());
        assert!(
            !t.contains("**"),
            "un archivo de texto no interpreta Markdown"
        );
    }

    /// **D-7A · cuántos contestaron, y consta que no se guarda qué dijeron.**
    #[test]
    fn las_respuestas_son_una_cifra_y_nada_mas() {
        let t = redactar(&una_campana());
        assert!(t.contains("Contestaron: 23"));
        assert!(t.contains("no guarda lo que"));
    }

    /// **El archivo no lleva ni una dirección, y lo dice.**
    ///
    /// Un resumen que se manda por correo con la lista de destinatarios dentro
    /// convierte una campaña en una filtración. Aquí no hay ningún campo donde
    /// pudiera entrar una, y además consta por escrito para que quien lo reciba
    /// sepa dónde sí está.
    #[test]
    fn el_archivo_no_lleva_destinatarios() {
        let t = redactar(&una_campana());
        assert!(!t.contains('@'), "no puede salir ninguna dirección");
        assert!(t.contains("A quién se le escribió"));
    }

    /// **Una causa desconocida se dice, no se maquilla.**
    ///
    /// Verificada rompiéndola: devolviendo «Error del servidor» para lo
    /// desconocido, la prueba falla porque la clave desaparece del archivo.
    #[test]
    fn una_causa_que_no_se_conoce_sale_con_su_clave() {
        let mut r = una_campana();
        r.causas = vec![FalloPorCausa {
            clave: "reintento.algoQueNadieHaVisto".into(),
            cuantos: 4,
        }];

        let t = redactar(&r);
        assert!(t.contains("reintento.algoQueNadieHaVisto"));
        assert!(t.contains("sin descripción"));
    }

    /// **Sin fallos, se dice; no se deja la sección vacía.**
    ///
    /// Un título sin nada debajo se lee como un archivo cortado.
    #[test]
    fn sin_fallos_lo_dice() {
        let mut r = una_campana();
        r.causas.clear();
        r.fallo = 0;

        let t = redactar(&r);
        assert!(t.contains("No hay fallos registrados"));
    }

    /// **Toda causa que el motor puede escribir tiene su frase.**
    ///
    /// ── LO QUE ESTO CAZA ──
    ///
    /// `error_kind` guarda la clave que decide `Decision::clave_i18n()`. Si se
    /// añade una decisión nueva y nadie escribe su frase aquí, el resumen la
    /// enseña como clave cruda — y eso lo lee Dirección, no un programador.
    ///
    /// Las claves se leen de las cinco decisiones que existen, y **el `match`
    /// de `clave_i18n` no tiene comodín**, así que una decisión nueva no
    /// compila hasta que alguien le dé su clave; esta prueba exige el segundo
    /// paso, que es darle su frase.
    #[test]
    fn toda_causa_del_motor_tiene_frase() {
        use crate::reintento::Decision;
        use core::time::Duration;

        let todas = [
            Decision::Reintentar {
                espera: Duration::from_secs(1),
                cual: 1,
                de: 3,
            },
            Decision::Rendirse,
            Decision::RendirseYDarDeBaja,
            Decision::PararLaCuenta,
            Decision::NoEsUnFallo,
        ];

        for d in todas {
            let clave = d.clave_i18n();
            assert!(
                causa_legible(clave).is_some(),
                "{clave}: el resumen la enseñaría como clave cruda"
            );
        }
    }
}
