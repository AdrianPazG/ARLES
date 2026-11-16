# Fase 6 · Campañas y mensajes

> **Estado: cerrada.** Las siete entregas construidas, verificadas y auditadas.
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
| **6.6** | El asistente de nueve pasos, y el aviso de más de 50 diarios | `arles-db/src/ventanas.rs` · `arles-app/src/campanas.rs` · `app/.../AsistenteDeCampana.vue` |
| **6.7** | Cierre: auditoría, documento, informe y PDF | este documento |

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

## 4 bis · El asistente, y las tres cosas que no decide

Nueve pasos (`UX_NAVEGACION.md` §5). Tres de ellos **no son campos que rellenar
sino cosas que leer** —la simulación, el preflight y el resumen de activación—,
y ésa es la razón de que sea un asistente y no un formulario largo: en un
formulario se bajan de un scroll sin verlos, y son justo los que hacen que
alguien reconsidere antes de mandar mil correos.

**Dónde empieza lo irreversible.** Los pasos 1 a 5 no escriben nada. La campaña
se crea al salir del 5, **en borrador**, porque el simulador y el preflight leen
de la base y el envío de prueba cuelga de la etapa. Crear no manda nada; lo que
manda correos es activar, en el paso 9.

Y tres reglas que la pantalla **no** decide:

| Regla | Quién la impone | Qué hace la pantalla |
|---|---|---|
| Si se puede activar | `revisar_campana` | Apaga el botón y dice a dónde volver |
| El aviso de más de 50 diarios | `crear_campana` | Enseña el aviso antes de pedirlo |
| La prueba es obligatoria (§45) | El preflight | Enseña el punto que falla |

### El aviso de volumen (§48) no es un límite

ARLES **no bloquea** el volumen: §154 pide informar, no dictar. Lo que no deja
es crear la campaña sin que conste que se avisó, y la constancia va a
`audit_log` —no a un campo que alguien pueda cambiar después—.

La diferencia es entre «alguien decidió esto sabiendo el riesgo» y «no sabemos
qué pasó», el día que el proveedor marque la cuenta como spam.

El umbral es **más de 50**, no 50 o más. Está en `Limites::AVISO_DIARIO` y
repetido en la pantalla —que necesita enseñarlo antes de llamar a nadie—, con
una prueba a cada lado del umbral en los dos sitios.

---

## 5 · Cómo se verifica esta fase

```bash
python3 herramientas/validar/validar.py --fase 6 --rapido    # 20/20
cargo test --workspace                                        # 786
cd app && npm test                                            # 132
```

Nueve invariantes, y **las nueve se rompieron a propósito para ver que la
comprobación las caza** (§2):

| Invariante | Rotura probada | ¿La caza? |
|---|---|---|
| El motor de plantillas no evalúa nada | añadir `handlebars` al núcleo | sí |
| El HTML se sanea en Rust | traer DOMPurify a la webview | sí |
| Una versión de plantilla no se edita | un `UPDATE template_version` real | sí |
| El preflight no se rellena a mano | `hay_conexion: Some(true)` | sí |
| La revisión descuenta las bajas | quitar el `NOT EXISTS` de la supresión | sí |
| El envío de prueba llega al preflight | dejar de pasar la marca | sí |
| El aviso de volumen lo exige el núcleo | quitar la línea de bitácora | sí |
| La pantalla no dice «entregado» | poner «Entregados» en la columna | sí |
| El botón de activar lo apaga el preflight | quitarle el candado | sí |

Una de las nueve **falló en su primera ejecución por estar mal escrita**, y
conviene dejarlo anotado: la de «entregado» leía los archivos `.vue` y cazó la
clave `campanas.aceptadoNoEsEntregado`, que existe justamente para explicar que
no se dice. Mirar los `.vue` además no habría servido de nada: los componentes
no llevan literales (§17), así que la palabra nunca estaría ahí y la
comprobación habría pasado siempre sin verificar nada. Ahora mira los textos.

Y las pruebas de las dos entregas, con su rotura anotada en el comentario de
cada una: quince de la 6.5 —nueve en `revision.rs`, cinco en
`tests/preflight.rs`, dos en `arles-email`— y de la 6.6, tres en `ventanas.rs`,
dos del umbral del §48 en `alta_de_campana.rs`, y diez de pantalla en
`campanas.spec.ts`.

**Dos fallos de la 6.6 los encontraron los avisos de una tanda que estaba en
verde**, y por eso se miran: faltaba el título del paso 7 —que habría salido en
pantalla como `asistente.paso.preflight.titulo`— y las pruebas simulaban
`$router` cuando las pantallas usan `useRouter()`, que lee de otro sitio. Las
diez pruebas pasaban con las dos cosas mal.

---

## 6 · La auditoría del cierre

Se recorrió la fase buscando lo que **no falla**: cosas que funcionan mal sin dar
error. Cinco hallazgos, y conviene distinguirlos porque no todos eran fallos.

### A-1 · La cifra del paso 6 y la que se congela son dos consultas distintas

**El más serio, y resultó estar bien.** La audiencia que enseña el preflight
(`contactos_de_la_audiencia`, entrega 6.5) y la que congela `activar_campana`
(entrega 6.4) son **dos consultas escritas en dos entregas distintas**, con sus
propios filtros, y nadie las había comparado.

Si divergieran no fallaría nada: el paso 6 diría «1 342 correos», alguien
decidiría con esa cifra, pulsaría activar, y se congelarían otros tantos. La
diferencia se descubre días después contando correos.

Coinciden, y ahora hay una prueba que lo sostiene. **Y la prueba hizo falta
arreglarla**: la primera versión pasaba igual con la revisión sin el filtro de
canal archivado, porque en los datos de prueba no había ninguno. Se añadió una
dirección archivada, y entonces sí caza la divergencia.

### A-2 · Un tope diario de cero deja la campaña colgada

El asistente deja escribir el tope a mano, así que un `0` llega hasta la base. Y
un tope de cero **no da error**: la campaña se crea, se activa, el motor la mira
cada cinco segundos, no le toca enviar, y vuelve a mirar. Para siempre, con la
pantalla diciendo «enviando».

Lo caza la cadena entera —tope 0 → techo diario 0 → la estimación no avanza → el
punto «el horario alcanza» falla y bloquea—, que son cuatro piezas de cuatro
entregas y **nunca se habían visto funcionar juntas**. Ahora hay una prueba.

### A-3 · El asistente sondeaba el servidor dos veces por nada · **corregido**

Pedía la revisión al salir del paso 5 y otra vez al salir del 6, y entre el 6 y
el 7 no cambia nada. Era una conexión SMTP de más, con su espera de hasta 60
segundos (`ESPERA_MAXIMA`), para volver a calcular lo mismo — y en macOS, una
lectura del llavero de más, o sea otro diálogo del sistema delante.

### A-4 · Un paso podía tardar un minuto diciendo «Calculando» · **corregido**

El paso 6 habla con el servidor de correo. Con un servidor lento, la pantalla se
quedaba con una palabra durante hasta un minuto, que es indistinguible de una
avería. Ahora usa `EstadoCargando`, que a partir de los 2 s explica que está
comprobando la cuenta contra el servidor y que puede tardar.

### A-5 · El respaldo de un estado desconocido no existía · **corregido**

La lista de campañas cae en `campana.estado.desconocido` cuando un estado
guardado no tiene rótulo — y esa clave no estaba escrita. La red de seguridad
habría enseñado la clave en crudo. Ahora existe, dice «Estado no reconocido» en
vez de inventarse un rótulo, y hay una prueba que ata las siete claves de Rust
más el respaldo.

### Lo que la auditoría **no** puede cerrar

**R-07 · macOS.** Nadie ha abierto esto en WKWebView. Y esta fase tiene un motivo
propio para que importe: el llavero se lee en cada revisión del preflight, y en
macOS eso puede sacar un diálogo del sistema. Con la corrección de A-3 son dos
por asistente en vez de tres, pero siguen siendo dos, y **no se pueden evitar**
sin cachear la contraseña, que es justo lo que ADR-0011 prohíbe.

**R-24 · Ningún servidor real.** Todo lo de esta fase que toca la red está
probado contra el servidor de mentira. Se cierra en la máquina del usuario.

---

## 7 · Lo que esta fase no cubre

- **La audiencia no se puede filtrar.** El paso 2 lo dice con todas las letras:
  llegan todos los contactos activos con correo, menos las bajas. Las listas,
  etiquetas y filtros del §40 no están, y enseñar un filtro que no filtra sería
  peor que no tenerlo.
- **No se puede editar una campaña ya creada**, ni borrarla. El asistente crea;
  pausar, reanudar y detener son de la Fase 4 y viven en Actividad.
- **Las plantillas y las firmas se guardan por comando pero no hay pantalla
  para escribirlas.** Los comandos están y probados; el editor de texto
  enriquecido es trabajo pendiente.
- **El canal de WhatsApp** es de la Fase 5 y no está empezado: hay una decisión
  pendiente de Dirección en `FASE-05-PARA-DIRECCION.md`.
- **Los registros SPF/DKIM/DMARC** (P-16) no se consultan. Es el único punto del
  preflight que avisa en vez de bloquear, así que no impide nada — y se dice
  como lo que es: sin comprobar.
