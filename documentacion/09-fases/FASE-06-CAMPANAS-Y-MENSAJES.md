# Fase 6 · Campañas y mensajes

> **Estado: cinco entregas de siete.** 6.1 a 6.5 construidas y verificadas;
> falta el asistente de nueve pasos (6.6) y el cierre de fase (6.7).
> Última actualización: 29/09/2026.

Esta fase es donde ARLES deja de ser un motor que sabe enviar y pasa a ser un
programa al que se le puede decir **qué** enviar y **a quién**.

---

## 1 · Lo que se construyó, entrega a entrega

| Entrega | Qué | Dónde vive |
|---|---|---|
| **6.1** | El motor de sustitución de variables, no Turing-completo | `arles-core/src/plantilla.rs` |
| **6.2** | El saneado del HTML con lista blanca, en Rust y dos veces | `arles-core/src/sanear.rs` |
| **6.3** | Plantillas con versiones que no se editan, y firmas | `arles-db/src/plantillas.rs` |
| **6.4** | Crear campañas, y el acto que no se deshace: activar | `arles-db/src/alta_de_campana.rs` |
| **6.5** | El preflight de ocho puntos, enchufado de verdad | `arles-db/src/revision.rs` · `arles-app/src/comandos.rs` |
| 6.6 | El asistente de nueve pasos, y el aviso de más de 50 diarios | *pendiente* |
| 6.7 | Cierre: auditoría, documento, informe y PDF | *pendiente* |

---

## 2 · Las tres decisiones que gobiernan la fase

### 2.1 · El motor de plantillas no sabe hacer nada más que sustituir

No hay condicionales, ni bucles, ni funciones, ni filtros. Sólo `{{clave}}`
contra una lista **cerrada** de cuatro variables (ADR-0006).

La tentación de usar Handlebars o Tera es real y se rechazó a propósito: esas
bibliotecas evalúan expresiones, y aquí las plantillas llevan datos de contacto
importados de un XLSX que no escribimos nosotros. Un motor con lógica sobre
datos ajenos es una vía de ejecución.

Y una regla que cierra la misma puerta por dentro: **lo que se sustituye no se
vuelve a mirar**. Si el nombre de un contacto fuera literalmente `{{empresa}}`,
se queda así en el mensaje. Sin esto, un archivo de un tercero podría escribir
plantilla.

### 2.2 · El HTML se sanea en Rust, con lista blanca, y dos veces

Al guardar y al enviar. El saneado del navegador no sirve para dos cosas a la
vez: se salta escribiendo en la base por cualquier otro camino, y **no protege a
quien recibe el correo** — el correo sale del proceso de Rust, no de la webview.

`sanear_dos_veces_no_cambia_nada` es lo que hace seguro aplicarlo dos veces.

### 2.3 · Una versión de plantilla no se edita

Una versión **es lo que se envió**. Editarla cambiaría, a posteriori, el texto
de correos que ya salieron: el registro diría una cosa y los destinatarios
habrían leído otra, y nada fallaría. Por eso se crean versiones nuevas y no se
toca ninguna, y por eso el validador busca cualquier `UPDATE template_version`
en todo `arles-db`.

---

## 3 · Activar una campaña es el acto que no se deshace

Al activar pasan tres cosas, **en una sola transacción**:

1. Se congela la audiencia: quién recibe queda escrito, y ya no depende de la
   lista de contactos.
2. Se llena la cola de envío.
3. La campaña pasa a `running`.

Las tres juntas o ninguna. Si la audiencia se congelara y la cola fallara,
quedaría una campaña activada que no envía y que no se puede volver a activar.

**Quién entra en la audiencia**, y el orden importa: contacto activo, canal de
correo **activo**, y sin baja en la lista de supresión. Lo del canal activo
faltaba en la primera versión de la consulta y lo cazó una prueba escrita a
propósito para ello: un canal archivado habría recibido correo, y nada habría
fallado.

---

## 4 · El preflight, y lo que significa «sin comprobar»

Los ocho puntos del §63 tienen **tres** respuestas, no dos: pasa, falla, y **no
se ha podido mirar**. La tercera bloquea igual que un fallo, y se cuenta aparte
porque lo que hay que hacer con ella es distinto: uno que falla se arregla, uno
sin comprobar se comprueba.

La 6.5 los rellena desde donde toca, y el reparto es deliberado:

| Punto | De dónde sale |
|---|---|
| La audiencia no está vacía | La base, después de aplicar las bajas |
| Las variables se resuelven | Se rellena de verdad, contacto a contacto, con el motor del núcleo |
| El horario alcanza | La estimación, con el **más bajo** de los topes de la etapa y de la cuenta |
| La credencial sirve | El disyuntor, si marcó la cuenta como `needs_reauth` |
| Se hizo el envío de prueba | La marca de la 5.5, **comparando la cuenta** |
| La cuenta autentica | La sonda al servidor, ahora |
| Hay conexión | La misma sonda |
| Los registros del dominio | Nadie todavía (P-16). Es el único que no bloquea |

**`arles-db` no habla con nadie de fuera**, así que devuelve lo que sabe y no un
preflight resuelto. Quien completa los tres puntos de red es `arles-app`, que sí
tiene el adaptador delante. Rellenarlos a mano con `Some(true)` «para que pase»
sería mentirle al usuario sobre su propia campaña, y la mentira se descubriría
en el primer envío — con los correos ya saliendo. Hay una comprobación del
validador que lo vigila.

### 4.1 · Un puerto cerrado no acusa a la contraseña

De los seis diagnósticos de la sonda, **sólo dos dicen algo sobre la
contraseña**: `Correcto` y `LaContrasenaNoSirve`. En los otros cuatro no se
llegó a intentar autenticar, así que ese punto queda **sin comprobar**.

Marcarlos todos como fallidos sería el error cómodo: el botón quedaría apagado
igual —así que nada fallaría— pero el texto diría «la cuenta no autentica» ante
un cable desenchufado, y quien lo lea se pasará la tarde generando contraseñas
de aplicación nuevas para un problema de red.

### 4.2 · El envío de prueba vale sólo con el remitente de hoy

La marca se anota desde la 5.5 con la cuenta que se usó. Si la etapa cambia de
remitente, la prueba **deja de valer**: demostró que otra cuenta podía enviar.

Una marca que sobreviviera a ese cambio daría luz verde apoyándose en la prueba
de otra cosa, y es exactamente el tipo de fallo que no se ve hasta que la
campaña ya salió.

---

## 5 · Cómo se verifica esta fase

```bash
python3 herramientas/validar/validar.py --fase 6 --rapido    # 12/12
cargo test --workspace                                        # 781
cd app && npm test                                            # 122
```

Seis invariantes, y **las seis se rompieron a propósito para ver que la
comprobación las caza** (§2):

| Invariante | Rotura probada | ¿La caza? |
|---|---|---|
| El motor de plantillas no evalúa nada | añadir `handlebars` al núcleo | sí |
| El HTML se sanea en Rust | traer DOMPurify a la webview | sí |
| Una versión de plantilla no se edita | un `UPDATE template_version` real | sí |
| El preflight no se rellena a mano | `hay_conexion: Some(true)` | sí |
| La revisión descuenta las bajas | quitar el `NOT EXISTS` de la supresión | sí |
| El envío de prueba llega al preflight | dejar de pasar la marca | sí |

Y quince pruebas de la 6.5 —nueve en `revision.rs`, cinco en `tests/preflight.rs`,
dos en `arles-email`— con su rotura anotada en el comentario de cada una.

---

## 6 · Lo que esta fase no cubre

- **El asistente de nueve pasos y el simulador** son la 6.6. Hoy el preflight
  existe y se puede llamar, pero no hay pantalla que lo pinte.
- **El aviso de más de 50 diarios** (§48) tiene su umbral en
  `Limites::AVISO_DIARIO` desde la Fase 4 y todavía no se enseña.
- **El canal de WhatsApp** es de la Fase 5 y no está empezado: hay una decisión
  pendiente de Dirección en `FASE-05-PARA-DIRECCION.md`.
- **Los registros SPF/DKIM/DMARC** (P-16) no se consultan. Es el único punto del
  preflight que avisa en vez de bloquear, así que no impide nada — y se dice
  como lo que es: sin comprobar.
