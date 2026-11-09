# Registro de requisitos

**Última actualización:** 2026-10-28

**Este es el único sitio donde vive el estado de un requisito.** Los
identificadores se usaban dispersos en siete documentos, y un requisito que
aparece dos veces con dos estados distintos es peor que uno que falta.

## Cómo se lee

| Estado | Significa |
|---|---|
| **abierto** | Se pidió y no está hecho. Se puede trabajar |
| **bloqueado** | No depende de nosotros. La columna «quién» dice de quién |
| **espera decisión** | Hace falta que Dirección elija. La alternativa está planteada |
| **hecho** | Construido y verificado, con la entrega que lo cerró |
| **descartado** | Se decidió no hacerlo, y el motivo está escrito |

| Prefijo | Qué es |
|---|---|
| **D-** | Decisión de Dirección |
| **P-** | Pendiente de alguien de fuera |
| **R-** | Requisito o riesgo |
| **H-** | Hallazgo encontrado usando ARLES |
| **F-** | Fallo de diseño que ya costó datos |

> **Un requisito nuevo se anota aquí el día que se pide**, no el día que se
> construye. Lo que no está anotado se olvida: pasó con `contact_list`, que
> estuvo en el esquema desde la V1 con su motivo escrito en «pendientes
> conocidos» y sin escritor hasta la D-15, dos años después.

---

## Lo que impide entregar la v1.2.0

Ninguno impide construir. **Los seis impiden entregar**, y dos de ellos tienen
plazo externo que no se comprime.

| Id | Qué | Quién | Estado | Qué bloquea exactamente |
|---|---|---|---|---|
| **P-01** | Licencia de la tipografía Mont | Dirección | bloqueado | **No se puede distribuir un instalador** con la tipografía dentro. Plan B listo: una línea en `--arles-font-family` (D-5, `TIPOGRAFIA.md` §3) |
| **FIRMA-WIN** | Certificado de firma de código para Windows | Dirección | bloqueado | Que el `.msi` se instale **sin la advertencia de «editor desconocido»** de SmartScreen, que en un entorno corporativo bloquea la instalación en la práctica. **Plazo externo de 1 a 3 semanas.** OV para la v1.2.0, EV para el lanzamiento comercial |
| **FIRMA-MAC** | Apple Developer Program de organización | Dirección | bloqueado | **Que ARLES abra en macOS.** Desde Catalina, una aplicación descargada sin notarizar no se ejecuta: no es una advertencia, es un bloqueo de Gatekeeper. Alta de la cuenta más notarización por build |
| **P-09** | Revisión jurídica | Abogado | bloqueado | Los textos legales que lee el usuario, y el primer envío real a alguien de fuera de TELEMETRY. Las dieciocho preguntas están en `08-legal/CONSULTA-JURIDICA.md`. La construcción **no** se detiene (ADR-0013) |
| **META** | Cuenta de WhatsApp Business con Coexistencia | Operaciones | bloqueado | Todo WhatsApp está probado contra un servidor de mentira. **Nunca ha hablado con Meta.** Por dentro el canal está completo desde D-WA (27/10/2026): campañas enteras incluidas |
| **R-07** | Revisión en macOS | quien tenga un Mac | bloqueado | Nadie ha abierto ARLES en WKWebView. La vista previa no cuenta: ahí el motor es el del navegador. **Lo arrastra FIRMA-MAC**: el mismo Mac hace las dos cosas |
| **R-24** | Un servidor de correo real | Dirección | **hecho** 22/10/2026 | Cerrado: ARLES manda por SMTP con TLS desde una cuenta de Titan, y llega a la bandeja principal de Outlook, Gmail y un corporativo |

> **Los dos trámites de firma se anotaron el 28/10/2026, y llevaban declarados
> desde la Fase 0.** `DISTRIBUCION_Y_FIRMA.md` §1 los pone como trámites que
> **arrancan en la Fase 1** precisamente porque son plazos externos, D-4 lo
> repite, y la matriz de riesgos los tiene como **R-14** con la mitigación
> escrita: *«trámite iniciado en Fase 1»*. No hay rastro de que se iniciara, y
> **ninguna de las tres listas de bloqueos del proyecto los nombraba**, así que
> nadie los estaba esperando.
>
> Es el décimo documento que afirmaba de más, y la frase exacta era de esta
> misma tabla: P-01 decía ser *«lo único entre ARLES y el `.exe`»*. Entre ARLES
> y un `.exe` que se instala sin advertencia hay también un certificado, y entre
> ARLES y un `.dmg` que **abre** hay un Developer ID. Una tabla que se lee para
> saber qué falta y deja fuera dos plazos de semanas no es una tabla incompleta:
> es la que hace que se descubran al final, que es el riesgo R-14 literal.
>
> Lo vigila `los_bloqueos_se_cuentan_en_un_solo_sitio` en la fase 0.

---

## Lo que espera una decisión

| Id | Qué | Planteado | Estado |
|---|---|---|---|
| **R-05** | Imagen en la firma | Embebida dispara filtros de spam y engorda cada correo; alojada hay que alojarla, y el saneado tendría que admitir `<img>` | espera decisión |
| **R-25** | Baja automática por correo | La recomendación es IMAP: leer la bandeja y dar de baja a quien responda BAJA | espera decisión |
| **D-19** | ¿El registro tiene que dar el total exacto? | C-2 midió que **el único cambio que baja el tiempo del filtro es dejar de contar todas las coincidencias**: traer las 500 filas cuesta la mitad. Pero el total al lado es D-7C, y se pidió para que el tope no escondiera nada. Las opciones y las cifras, abajo | espera decisión |
| **Fase 9** | Sacar la activación al hilo del motor | Activar tarda ~73 s con 451 196 destinatarios y el presupuesto de 2 s se rompe **alrededor de los 30 000**. Desde el 28/10 la ventana **sí responde**, dice en qué va y se puede cancelar; lo que queda es que no corra dentro de un comando | abierto · medido en la auditoría C |

---

## Decisiones tomadas

Las que cambian cómo se construye. **No se vuelven a discutir** salvo que
Dirección lo pida.

| Id | Fecha | Decisión |
|---|---|---|
| **D-7A** | 29/09/2026 | Las respuestas de WhatsApp **se cuentan, nunca se guarda qué dijeron** |
| **D-7B** | — | La parada de emergencia vive en Campañas |
| **D-7C** | — | El registro lleva seis criterios de filtro combinables |
| **D-8A** | 05/10/2026 | El respaldo va **cifrado**, con opción explícita de desactivarlo |
| **D-8B** | 05/10/2026 | **Dos alcances**, «Esencial» por defecto |
| **D-8C** | 05/10/2026 | Restaurar **reemplaza, nunca mezcla** — y las bajas se **unen** |
| **D-8D** | 05/10/2026 | Se puede **archivar** el historial a un disco externo y sólo entonces borrarlo, con cuatro condiciones |
| **D-13** | 22/10/2026 | **El límite diario lo fija la campaña**, hacia arriba o hacia abajo. El `min` con la cuenta se quitó |
| **D-15** | 23/10/2026 | Cada campaña carga **su propia lista** de contactos. `None` es «toda la empresa», y la pantalla lo ofrece con esas palabras |
| **D-16** | 23/10/2026 | **Ventana de descanso** de 14 días entre campañas al mismo contacto |
| **D-WA** | 26/10/2026 | El canal de WhatsApp entra en la v1.2.0, **con campañas enteras** y no sólo el envío de prueba |
| **D-18** | 28/10/2026 | **[ADR-0015](../03-arquitectura/adr/0015-un-escritor-y-lectores-separados.md) aceptado.** Un escritor con cerrojo y lectores en conexión aparte. El **hilo dedicado con canal de ADR-0002 queda descartado por escrito**, no pendiente: su premisa era Tokio y este proyecto no lo usa. La conexión de lectura queda **programada**, y no empieza hasta medir el crecimiento del `-wal` |
| **DMARC** | 27/10/2026 | El DMARC se queda en `p=none` **dos o tres semanas** leyendo informes, y después se sube `pct=25` → 50 → 100. Saltar a `quarantine` manda correo legítimo a spam **en silencio**: sin rebote, sin error y sin aviso |
| **Baja en WhatsApp** | 27/10/2026 | **No se exige** que la plantilla lleve la salida escrita, porque ARLES no podría atenderla: no recibe los mensajes y no lee su texto (D-7A). Se avisa **a mano** en Contactos › Bajas, y se dice con esas palabras en la pantalla, la guía y el manual |

---

## Requisitos de Dirección, cerrados

| Id | Qué | Cerrado |
|---|---|---|
| **R-01** | Dar de alta un correo era «demasiado complicado» | ✅ 14/10/2026 · dos campos, con `PROVEEDORES` deduciendo servidor, puerto y cifrado de cinco dominios |
| **R-02** | Lo mismo con WhatsApp | ✅ 26/10/2026 · **un dato y una elección**: se pega el token y ARLES le pregunta a Meta cuáles son tus números |
| **R-06** | Una plantilla creada no se podía corregir | ✅ 17/10/2026 |
| **H-01** | Remitentes decía «no has configurado tu empresa» después de configurarla | ✅ |
| **H-02** | Las tres partes de un error salían pegadas | ✅ |
| **H-03** | No se pudo dar de alta una cuenta de Gmail | ✅ 14/10/2026 · cerrado por R-01, y conviene saber hasta dónde: `PROVEEDORES` trae Gmail con `smtp.gmail.com:587` y STARTTLS, exige los **16** caracteres de una contraseña de aplicación, enlaza a dónde se crea y **le quita los espacios** con que Google la enseña — las tres causas plausibles del fallo original. Lo que **no** se ha hecho es dar de alta una cuenta de Gmail real desde entonces: R-24 se cerró contra Titan. Así que la causa está cubierta y la verificación de punta a punta no |
| **H-05** | Error al dar de alta un contacto | ✅ |
| **H-06** | El horario de envío salía vacío, sin salida | ✅ |
| **H-12** | **La firma elegida nunca se guardaba.** Todas las campañas salían con la misma, y nada fallaba | ✅ 22/10/2026 |
| **H-07** | El límite diario se escribía y no se aplicaba: se pedía 300 y salían 50 | ✅ 22/10/2026 · D-13 |
| **H-08** | Dos campañas con el mismo remitente salían 25 y 25, por orden de llegada | ✅ 22/10/2026 · hoy salen las dos enteras, y el paso 6 avisa de que el plazo cambia |
| **H-13** | **Escribir `%` o `_` en el filtro del registro devolvía todas las filas.** El texto del usuario entraba dentro del patrón del `LIKE`, así que era sintaxis | ✅ 02/11/2026 · lo encontró medir C-2 |
| **H-14** | **El error de «fallo al migrar» afirmaba que se había hecho una copia de la base**, y no hay ninguna copia en todo el proyecto. De las tres partes del §95, la falsa era la tercera: la que existe para decidir qué se puede borrar sin perder nada | ✅ 03/11/2026 |
| **H-15** | **Un fallo del generador de aleatoriedad se diagnosticaba como «tu base es de una versión anterior»**, y mandaba a apartar un archivo que está bien — en el primer arranque, donde ni existe | ✅ 03/11/2026 · `DbError::SinAleatoriedad` |
| **R-26** | **La lista de contactos no se podía filtrar**: `listar_contactos` sólo tomaba página y tamaño. Cuatro criterios combinables desde el 03/11/2026 — nombre/apellido/empresa, dirección completa, canal y si tiene alguna baja. Las cifras y lo que se descartó, en su sección | ✅ 03/11/2026 |
| **H-16** | **Buscar «TÉLLEZ» no encontraba a Téllez.** `lower()` de SQLite sólo baja ASCII, así que la `É` quedaba intacta y el filtro decía que esa persona no está — **sin ningún error**. En un producto mexicano no es un caso raro | ✅ 03/11/2026 · lo encontró una prueba con un acento |
| **H-17** | **El filtro del registro usaba un índice del lado inútil.** La forma obvia de buscar por dirección —un `JOIN` con la empresa en el `WHERE`— nombra el índice correcto en el plan y tarda **609 ms** donde la forma buena tarda **0.0**: SQLite conduce desde `contact` por el `ORDER BY` y prueba contacto por contacto | ✅ 03/11/2026 · lo cazó el plan, no el resultado |
| **R-27** | **Una campaña sólo podía ir a toda la empresa o a una lista importada de un archivo.** Dirección dio de alta cincuenta contactos a mano, quiso probar con tres y no pudo: *«no seleccioné una tabla y no me permitió seleccionar contactos de mi lista de contactos»*. Las dos tablas estaban en el esquema desde la V1 y la campaña sabía ir a una lista desde D-15; lo único que las llenaba era la importación — el patrón del §9 | ✅ 05/11/2026 · casilla por fila, un botón que marca lo que el filtro dejó, y la bandeja que las guarda con un nombre. **Sin tope**, y con el mismo nombre se añade a la lista que ya hay |
| **H-22** | **Una campaña a medias se quedaba en la lista para siempre.** Al salir del asistente en el paso 6, la campaña ya está creada como borrador —hace falta para guardar lo que se lleva escrito— y no había forma de quitarla: *«no me gustaría ver esa campaña, me gustaría tener la oportunidad de poder eliminarla, pero no sé qué tan recomendable sea hacer eso»* | ✅ 05/11/2026 · se borra **sólo un borrador**, con tres puertas —el tipo del núcleo, el `status` en el `WHERE` y el recuento de envíos—. Lo que ya envió no se borra: su historial es con lo que se contesta un ARCO |
| **H-21** | **ARLES decía «Meta no acepta ese token» con la configuración correcta.** Dirección recorrió la guía entera —cuenta de empresa, aplicación de tipo Negocios, usuario del sistema con la aplicación y la cuenta de WhatsApp en **acceso total**— y el alta seguía fallando | `debug_token` sólo devuelve los `target_ids` del permiso de gestión cuando está concedido **sobre un activo concreto**, y hay tokens válidos a los que Meta no se los pone. ARLES tenía **una sola vía** para encontrar la cuenta, y al venir vacía acusaba al token. **El token era bueno.** Es el diagnóstico equivocado de DOC-14 en el primer contacto del producto con la API real | ✅ 05/11/2026 · segunda vía por los negocios del token, y el error partido en dos: `NingunaCuenta` ya no manda a generar otro token |

---

## C-2 · el filtro del registro, cerrado al contrario de lo previsto

**02/11/2026.** La auditoría C lo declaró con **3 709 ms de un presupuesto de
1 s** y escribió la solución: FTS5. **Se construyó y es 18 veces peor.**

Medido sobre 451 196 sucesos, con tres corridas de cada camino y las dos
búsquedas que importan — la de **una persona**, que es la que alguien hace, y
la de un dominio entero:

| Camino | Una persona (1 coincidencia) | Un dominio (119 532) |
|---|---|---|
| `LIKE '%…%'`, el de hoy | 1 257, 1 192, 1 151 ms | 1 212, 1 222, 1 146 ms |
| La pregunta al revés, sin índice | 1 015, 1 010, 1 145 ms | 1 192, 1 221, 1 217 ms |
| Índice FTS5 `trigram` | 21 556, 21 906, 20 653 ms | 23 242, 23 793, 24 448 ms |

**Cuatro cosas que cambian lo que se sabía:**

1. **El presupuesto se incumple por un 20 %, no por 3.7×.** La cifra de la
   auditoría no se reproduce, y no lo explica un cambio de esquema: el índice
   que usa esa consulta es de la V10, anterior a aquella medición. No se
   inventa la causa — aquella corrida no se guardó.
2. **Ningún índice sirve.** El FTS5 `trigram` empeora cuanto más largo es el
   texto, porque cada trigrama de una dirección está en casi todas las filas.
   Un índice cubriente no cambia el plan de consulta, así que se descartó sin
   medirlo.
3. **La pregunta al revés gana un 13 %**: no justifica cambiar una consulta
   correcta.
4. **El tiempo está en el total**, no en la búsqueda. Traer las 500 filas de la
   página cuesta la mitad. De ahí **D-19**, arriba: el total exacto es lo único
   que baja esta cifra, y es una decisión de producto.

Y los tres costes que la migración traía aparte del rendimiento, cada uno
suficiente por sí solo: una segunda copia de una dirección de correo que hay
que mantener sincronizada para que un ARCO no la deje encontrable; romper
`la_cascada_calculada_es_la_que_borra_sqlite`, que es la defensa contra **F1**;
e invalidar todos los respaldos anteriores, con un instalador ya fuera.

> **Lo que esto deja construido**, aunque el índice se haya ido: el banco mide
> los tres caminos y cualquiera puede volver a correr la comparación, en vez de
> creerse esta tabla; H-13 arreglado; y
> `ninguna_tabla_se_queda_fuera_del_respaldo` ya ve una tabla virtual — no la
> veía, y la fase 8 pasó en verde con una tabla nueva sin clasificar.

---

## R-26 · el filtro de contactos, y los tres defectos que encontró medirlo

**03/11/2026.** Cuatro criterios combinables en Contactos. Lo entregado es lo
de menos: lo que esta entrega deja son **tres defectos que llevaban tiempo ahí
y pasaban todas las pruebas**, y una cifra de ayer corregida.

### Qué cuesta cada criterio, sobre 455 470 contactos

| Criterio | Milisegundos | Veredicto |
|---|---|---|
| Dirección completa | **0.2** · no crece con la tabla | ✅ |
| Canal | 0.1 | ✅ |
| Texto, cuando lo encuentra | 1.4 | ✅ |
| **Texto, cuando NO lo encuentra** | **~2 600** | ❌ 2.6× el presupuesto, declarado |

**El texto cuesta lo que no encuentra**, y es lo contrario de la intuición:
para poder decir «no hay nada» hay que visitar las 455 470 filas. Es el caso
que se toca mientras se escribe, así que el campo no busca al teclear.

**Y un trozo de dirección no se ofrece**, con su cifra: 2.1 s, porque un sufijo
no lo acota ningún índice ordenado por el principio de la cadena. La pared de
C-2. También se descartó «empieza por» sobre el nombre: sin índice cuesta igual
que `%x%`, y con uno habría que pagar una migración que deja sin restaurar todo
respaldo anterior.

### Los tres defectos

**H-17 · Un índice usado del lado inútil.** La forma obvia de buscar por
dirección —un `JOIN` con la empresa en el `WHERE`— **nombra el índice correcto
en el plan** y tarda 609 ms donde la forma buena tarda 0.0: SQLite conduce desde
`contact` por el `ORDER BY` y prueba contacto por contacto, y como una dirección
casa una sola fila el tope de cien no se cumple nunca. Seis mil veces, con el
mismo índice en el plan y el mismo resultado devuelto.

Y enseña el límite de lo que ya había: `las_consultas_criticas_usan_su_indice`
pregunta si el plan **nombra** el índice, no por qué lado.

**H-16 · Buscar «TÉLLEZ» no encontraba a Téllez.** `lower()` de SQLite sólo
baja ASCII. El patrón se baja ahora en Rust.

**Y el tercer «hallazgo» de esa tarde era mío y lo retiré el mismo día.** Dije
que la conexión de lectura es 12× más lenta recorriendo una tabla, con 302 ms
por el escritor y 3 742 por la lectora. **No se reproduce.**

Cinco corridas del mismo experimento, en el mismo contenedor y sobre la misma
base: la lectora 12× más lenta, 10× más rápida, 11× más rápida, 11× más rápida
y 11× más lenta — y el signo cambió al añadir dos `count(*)` antes de
cronometrar, que no debería influir en nada. **Lo que domina en este contenedor
es la caché, no la conexión**, y por eso aquí no se puede medir. Las cinco
corridas y lo que sí quedó descartado están en
[`PRESUPUESTO_RENDIMIENTO.md`](../06-calidad/PRESUPUESTO_RENDIMIENTO.md) §8.

> **Y lo que costó publicarla antes de repetirla:** con esa proporción moví dos
> lecturas de la lectora al escritor y dije que el coste real del filtro del
> registro era de catorce segundos. **Las dos cosas se deshicieron**: las
> lecturas vuelven a la lectora —cuyo motivo sí está medido, 0 ms contra 650
> con el cerrojo del motor tomado— y los 1 151–1 257 ms de C-2 se mantienen.
>
> **Una cifra que no se reproduce no puede mover una decisión que sí está
> medida.** Es el precedente del 52/1 del validador: se publica el mecanismo
> que se midió, no la causa que se sospecha.

### Lo que esta entrega enseña del método

**Repetir el experimento cambiando algo que no debería importar.** Es lo que
distinguió la medición mala de la buena, y lo que faltó antes de publicar el
12×: las cinco corridas eran cada una limpia, plana y creíble por dentro —nueve
de nueve a 95 ms contra nueve de nueve a 1 150—, y sólo compararlas entre sí
enseñó que el instrumento medía el orden.

**Medir los candidatos no es medir lo entregado.** Los nueve caminos candidatos
daban 899 ms en el peor caso; el camino real daba 22 000. La diferencia no
estaba en ninguno de los nueve: estaba en la junta —qué consultas de más, qué
orden— que ninguno de los nueve recorría. Es el patrón del §9 aplicado a una
medición.

---

## V11→V12 · restaurar un respaldo de un esquema anterior, cerrado el 04/11/2026

Era la limitación más cara que la Fase 8 dejó declarada (8.3b) y la V11 la
disparó por primera vez: ARLES rechazaba todo `.arles` cuyo esquema no fuera el
de hoy, así que **cada migración mataba las copias de seguridad anteriores**.

El archivo seguía en el disco, la pantalla podía revisarlo y contar lo que
traía, y el día que hiciera falta no entraba. Es el peor sitio donde puede vivir
una limitación: una copia de seguridad que existe, parece correcta y no abre.

### Cómo se cerró, y la salida que estaba descartada por escrito

La 8.3b dejó dos caminos y uno descartado: *«restaurar lo que encaje»*, que
dejaría un apellido nulo y una fecha basura en una base que arranca y parece
funcionar. El otro era correr las migraciones sobre los datos del respaldo, y la
forma de hacerlo **sin escribir una línea de SQL nueva** es levantar una base
desechable en la versión que el respaldo declara, meterle sus filas, y
aplicarle las migraciones que faltan.

El SQL que transforma los datos es **el mismo que ya corrió en producción**. No
hay una segunda traducción que mantener: ésa es la parte que se habría quedado
atrás en la primera migración posterior a esta entrega.

La base desechable **va cifrada** con una clave nueva de usar y tirar —lleva los
contactos y las bajas de una empresa, y §29 no admite «menos en este caso»— y se
borra al soltarla, lo que comprueba una prueba.

### Donde más importaba: el llavero perdido

En `/recuperar` no hay otra salida. El cifrado no tiene puerta de atrás y la
pantalla lo dice con esas palabras, así que un respaldo de antes de la última
migración era, **ahí**, ningún respaldo.

Y la puesta al día ocurre **antes** de apartar la base ilegible y de crear la
clave nueva, que es la regla que la auditoría F estableció el 07/10: *nada
irreversible ocurre hasta que todo lo comprobable está comprobado*. Leer el
archivo entero y migrar sus datos es la comprobación más fuerte que se puede
hacer sin tocar nada, y hacerla después habría añadido un disparador nuevo al
F2 —una migración que no se puede aplicar a esos datos—.

### Lo que cuesta

Treinta mil contactos (60 002 filas), tres corridas, 04/11/2026: poner al día
**1 134–1 255 ms** y restaurar **1 117–1 671**, contra **1 410 ms** de restaurar
directo en la misma máquina. Un 70 % más en total, una vez, y sólo si el
respaldo es viejo. Memoria 16 → 23 MB. Está en `PRESUPUESTO_RENDIMIENTO.md` §9.

### Lo que sigue sin poderse

| Qué | Por qué, y qué se dice |
|---|---|
| Un respaldo de una versión **posterior** | Sus migraciones no están en este binario y no se inventan. Error aparte, que manda a **actualizar ARLES** en vez de a buscar otra copia |
| Un **esencial** puesto al día puede no dar lo mismo que la base completa migrada | Una migración que rellene una columna leyendo una tabla que el esencial no lleva no tiene de dónde leer. Es inherente al alcance |
| Espacio en disco | Hace falta para una base del tamaño del respaldo. Un disco lleno falla con la base real intacta |

### Lo que esta entrega enseña del método

**La prueba se eligió midiendo, no suponiendo.** De la V11 a la V16 ninguna
migración mueve datos que viajen en un respaldo —son columnas nuevas e
índices—, así que una prueba V11 → V16 habría pasado en verde con un módulo que
sólo copiara filas. La prueba de verdad arranca en la **V2**, donde la V3
traslada el correo del contacto a su canal y la V4 convierte cada campaña en una
con etapa: dos tablas que **no existían** cuando se tomó ese respaldo.

**Y la comprobación nueva nació con el defecto que persigue, y van trece.** Su
primera ruptura salió **en verde**: buscaba el nombre de la función en el
archivo entero, y la llamada de **la prueba del mismo archivo** la satisfacía.
La pieza se habría quedado sin llamador en producción con todo verde — el patrón
del §9 dentro de la comprobación escrita para cazarlo. Es la misma familia que
el comentario que se caza a sí mismo: **medir en un sitio donde la respuesta no
cuenta**. De ahí `solo_produccion` en el validador, que corta las pruebas además
de los comentarios.

**Y un defecto que cazó mi propia prueba antes de entrar:** `version_de("V-3")`
devolvía `-3`, porque `parse::<i32>` acepta negativos. Con eso, la base
desechable habría nacido vacía y el fallo habría llegado dos pasos después
diciendo que *el respaldo trae una tabla que esta base no tiene* — un
diagnóstico equivocado, que manda a hacer algo (DOC-14).

**Y un `assert` que no podía fallar.** La primera versión de la prueba de ida y
vuelta comprobaba que `template.retired_at` existiera **en la base de destino**,
que la tiene siempre. Lo que distingue una puesta al día de un paseo es comparar
las columnas que el respaldo **declaraba** con las que entrega puesto al día.

---

## Auditoría H · funcionalidad y flujo, 04/11/2026

Dirección pidió auditar la aplicación entera, verificar la funcionalidad y
corregir lo que apareciera. Primera pasada, con las diez fases en verde al
empezar. **Tres hallazgos, los tres arreglados**, dos descartados por falsos y
dos defectos del instrumento de medida.

| Id | Qué | Estado |
|---|---|---|
| **H-18** | Tres documentos decían que **no se puede lanzar una campaña por WhatsApp**, y se puede desde D-WA. Faltaban además las pruebas del motor del canal | ✅ |
| **H-19** | El paso 6 del asistente bloqueaba los pasos 7, 8 y 9 **en la demostración**, con una cifra pegada de otro escenario | ✅ |
| **H-20** | «Activar campaña» apagado **sin decir por qué** a un lector de pantalla, y la sonda medía 1 de los 9 pasos | ✅ |

### H-18 · el §9 al revés: el documento promete de menos

`CLAUDE.md` afirmaba dos cosas contradictorias — que el canal está completo
«campañas enteras incluidas» y que «el motor solo recorre `Canal::Correo`».
Medido en las cuatro capas, la segunda es falsa desde el 26/10: la consulta de
etapas **no filtra por canal** y nunca lo hizo, la fábrica monta el adaptador
de Meta, y el motor arma su contenido con los valores de cada persona.

**Es la primera vez en el proyecto que un documento promete de menos**, y el
coste no es menor: la misma frase estaba en el manual que lee Dirección, así
que una función construida, probada y pagada se quedaba sin usar porque el
producto decía que no existía.

Lo que faltaba de verdad era la red: las diecisiete pruebas de WhatsApp del
taller miden el núcleo, el alta, la frontera y el adaptador —cada capa la
suya— y **ninguna tocaba el motor**. Ya hay dos, y la primera manda a **dos**
contactos a propósito: con uno no se ve que cada quien reciba sus datos, que
es lo que D-WA vino a arreglar.

### H-19 · un decorado que impide recorrer la demostración

Las cifras del paso 6 estaban **pegadas** de otro escenario —1 303 días y
386 046 sin enviar, que salen de los 451 196 contactos del banco—, así que con
la audiencia del decorado la pantalla decía a la vez «esta campaña enviará 3
correos» y «386 046 mensajes se quedarían sin enviar».

Y como el paso 6 bloquea «Continuar» cuando el desenlace impide activar
(8.6 E), **los pasos 7, 8 y 9 no se podían revisar**. Es el defecto de la 7.3
otra vez. Ahora la estimación **se calcula** de la audiencia: una cifra
derivada no se puede desalinear de aquello de lo que deriva.

### H-20 · el botón más importante, mudo para un lector de pantalla

El texto «Vuelve al paso 7» estaba desde D-04, **pintado y sin enlazar**. No lo
cazó nada porque la sonda abre cada ruta y mide **el estado inicial**: en el
asistente, el paso 1 de 9, con la ruta contada como pantalla medida. Es el
defecto del 28/10 un nivel más abajo — allí faltaban rutas, aquí faltan
estados dentro de una ruta.

La sonda **recorre** ahora las pantallas con pasos, detectándolas por «Paso N
de M» y no por una lista de rutas. Verificado quitando el `aria-describedby`:
lo caza en las cuatro ventanas, nombrando el paso.

### Lo que salió limpio

**76 comandos y 108 argumentos**: lo que las pantallas mandan es exactamente lo
que los comandos declaran, en las dos direcciones — y la auditoría G lo midió a
mano el 22/10 **sin dejar comprobación**, así que llevaba dos semanas sin
mirarse. Ningún comando sin llamador. La máquina de estados de una campaña, con
sus trece pruebas y el estado comprobado dentro de la transacción que escribe.
Los pasos 1 a 8 del asistente. Las tres sondas: 18 pantallas por cuatro
ventanas, 1 980 textos, 147 paradas de tabulador.

### Los dos hallazgos falsos, y los dos defectos del instrumento

Descartados antes de anotarlos: que el simulador del motor no pudiera afirmar
qué sale por WhatsApp (sí puede) y que el paso 6 no explicara su botón apagado
(lo explica, con `aria-describedby`).

Y el medidor de argumentos nació diciendo que `activar_campana` «no existe en
Rust» y que ocho comandos no los llama nadie. Las dos falsas: no cubría
`#[tauri::command(async)]` ni los atributos intermedios, y sólo veía llamadas
con el nombre literal. Al corregirlo, los comandos pasaron de 74 a 76.

**Tercera vez que el instrumento es el defecto** —la 8.5 medía un camino que el
usuario no recorre, el 03/11 medía el orden en vez de la conexión—, y la
conclusión se repite: antes de creerle a un instrumento nuevo hay que comprobar
sus cifras contra algo que ya se sepa.

---

## Fallos que ya costaron datos

Se guardan porque **cada uno ha vuelto**, y la defensa que funcionó es la misma
las tres veces: leer la cascada del esquema y contarla, no tocarla.

| Id | Qué pasó | Veces |
|---|---|---|
| **F1** | Un `DROP TABLE` de una tabla padre se llevó en cascada la audiencia congelada y el registro de envíos | 3 — Fase 1, restaurar un esencial (8.3a), archivar el historial (8.3 bis) |
| **F2** | Perder la entrada del llavero y crear una clave nueva al arrancar deja la base **ilegible para siempre** | 2 — Fase 1, y la recuperación de la 8.4 que escribía la clave antes de leer el respaldo |

---

## Documentos que afirmaban de más

El patrón número 4 de [`RELEVO.md`](../00-ANTES-DE-EMPEZAR/RELEVO.md), y el más
barato de cometer: **un documento no falla, se cree.** Se anotan aquí porque un
documento equivocado cuesta trabajo rehecho igual que un defecto, y porque la
defensa que funciona es siempre la misma — una comprobación que lee el código,
no una relectura con buena voluntad.

| Id | Qué decía | Lo que había | Cerrado |
|---|---|---|---|
| **DOC-1** | `DESIGN_SYSTEM.md`: «la barra lateral es fija, **no colapsable**» | Se plega, con el umbral medido por `sonda:plegado` | ✅ 27/10/2026 |
| **DOC-2** | `DESIGN_SYSTEM.md`: la altura de fila es «**preferencia del usuario**» | Es una opción de cada tabla; esa pantalla no existe | ✅ 27/10/2026 |
| **DOC-3** | `DESIGN_SYSTEM.md`: el anillo de foco usa el oro de relleno | Usa la tinta de acento. En tema claro, el relleno daría **1.32:1** | ✅ 27/10/2026 |
| **DOC-4** | `DESIGN_SYSTEM.md`: los iconos vienen de «**Lucide (ISC)**» | 24 trazos propios. No es una dependencia del proyecto | ✅ 27/10/2026 |
| **DOC-5** | `DESIGN_SYSTEM.md`: «**cada** insignia lleva icono además del color» | El tono neutro no lo lleva, a propósito desde el 29/09/2026 | ✅ 27/10/2026 |
| **DOC-6** | `THREAT_MODEL.md` §8 pedía «la capa de redacción de `tracing`» | El proyecto no usa `tracing` | ✅ 06/10/2026 · auditoría B |
| **DOC-7** | `PRESUPUESTO_RENDIMIENTO.md` §3 describía una arquitectura con Tokio | No hay Tokio; la conexión es un `Mutex<Connection>` | ✅ 07/10/2026 · auditoría C |
| **DOC-8** | El `ROADMAP` ponía los derechos ARCO en la Fase 8 | Construidos desde la 3.4. La auditoría B perdió tiempo buscándolos | ✅ 07/10/2026 |
| **DOC-9** | `CLAUDE.md` §1 bis: «Todo documento nuevo se nombra en `00-INDICE.md`: **lo vigila la fase 0**» | No lo vigilaba nadie. La fase 0 tenía una lista de catorce documentos esperados y comprobaba que existieran; la dirección contraria no la miraba nada | ✅ 28/10/2026 · ahora la vigila `todo_documento_esta_en_el_indice` |
| **DOC-10** | Esta misma tabla: P-01 era «**lo único entre ARLES y el `.exe`**», y los bloqueos eran «los cuatro» | Son cinco. Faltaban los **dos trámites de firma**, declarados en `DISTRIBUCION_Y_FIRMA.md` §1 desde la Fase 0 como plazos externos que arrancan en la Fase 1, y ausentes de las **tres** listas de bloqueos del proyecto | ✅ 28/10/2026 · ahora los cuenta `los_bloqueos_se_cuentan_en_un_solo_sitio` |
| **DOC-11** | `avance.json`: **89 % a 2026-10-22**, con F3W al 55 %, F5 al 70 % y F7 al 92 % | Doce tramos cerrados después de esa fecha —la 7.5 entera y D-WA entera— dejaban las tres fichas describiendo un estado que ya no existía. F7 decía «falta sólo la 7.5» con la 7.5 construida. Recalculado: **95 %** | ✅ 28/10/2026 |
| **DOC-12** | `ADR-0002`: la base «vive en un hilo dedicado que recibe peticiones por canal», y «los lectores usan conexiones separadas» | Ni una cosa ni la otra. Cero `thread::spawn` y cero `mpsc` en `arles-db`, y **una sola** `Connection::open`: lo que hay es un `Mutex<Connection>`. Su decisión tecnológica sí está construida y es correcta; lo que no se construyó es una **mitigación** de sus consecuencias negativas | ✅ 28/10/2026 · acotado por [ADR-0015](../03-arquitectura/adr/0015-un-escritor-y-lectores-separados.md) |
| **DOC-13** | `actividad.rs` desde la 7.4, y la auditoría C repitiéndolo el 07/10: «si algún día hiciera falta más, **la respuesta es FTS5**» | Se construyó y es **18 veces más lento** (21 556 ms contra 1 257). Ningún índice sirve para este filtro, y el tiempo está en contar el total, no en buscar. Es el peor sitio donde dejar una recomendación sin medir: un comentario del código, que el siguiente lee como una decisión tomada | ✅ 02/11/2026 · la cabecera del módulo lleva ahora las nueve cifras, y el banco repite la comparación |
| **DOC-14** | `ESTADO_ACTUAL.md` §10 punto 3 y `RELEVO.md`: «el mensaje de **fallo al migrar** no cumple el §95, manda a cerrar y reabrir, y no dice dónde vive la base» | El del arranque se arregló el **14/10/2026** y cumple el §95 con la ruta dentro. Los dos documentos lo siguieron pidiendo tres semanas, y el registro de tropiezos sí tenía el arreglo escrito. El hallazgo de verdad estaba en otro sitio y era peor (H-14): **lo tapó una descripción desfasada que sonaba a lo mismo** | ✅ 03/11/2026 |

| **DOC-15** | `conexion.rs` desde el 28/10/2026: las cascadas de borrado son seguras con dos conexiones «porque la cascada necesita *triggers* recursivos y aquí están activados» | **No están activados**, y la cascada salta igual: `PRAGMA recursive_triggers` no se toca en ninguna línea del proyecto, y un borrado en cascada se midió disparando su `AFTER DELETE`. La garantía es cierta; el mecanismo con que se explicaba, no. Lo que de verdad depende de ese ajuste es `INSERT OR REPLACE`, del que hay **cero** | ✅ 03/11/2026 · el comentario explica ahora el mecanismo medido |
| **DOC-16** | La auditoría C y `ESTADO_ACTUAL.md` §10 punto 1: «**sacar la activación de campañas al hilo del motor**» | Mover el trabajo de hilo **no suelta el cerrojo de escritura**: lo tomaría el hilo del motor en vez del del comando. Medido con una escritura de 800 ms en curso: por la conexión de lectura **0 ms**, por la de escritura **650**. Lo que descongela las pantallas es terminar ADR-0015, y es lo que se hizo — 17 de 24 lecturas movidas. Segunda recomendación de auditoría en dos días que se cae al medirla (la otra es DOC-13) | ✅ 03/11/2026 · lo vigila `toda_lectura_de_pantalla_va_por_su_conexion` |
| **DOC-17** | `MAPA-DE-PANTALLAS.md`: «**las trece pantallas**», y «**Actividad es andamio** hasta su fase» | Son dieciocho desde la Fase 8, y Actividad la construyó la 7.3 el 30/09/2026 llevándose `PantallaPendiente.vue`. Faltaban el Registro, Respaldos, Restaurar, Archivar y Recuperar — **las cuatro últimas son todo lo que construyó la Fase 8**. Es el defecto de las sondas del 28/10 en el documento que se lee **para cambiar el diseño** | ✅ 03/11/2026 · lo vigila `el_mapa_de_pantallas_nombra_todas_las_pantallas` |
| **DOC-18** | **Mío, el 03/11/2026 por la tarde**: «la conexión de lectura es 12× más lenta recorriendo una tabla», y con eso que el filtro del registro costaba **catorce segundos** y no 1 251 | **No se reproduce.** Cinco corridas del mismo experimento dieron el signo en los dos sentidos, y cambió al añadir dos `count(*)` que no debían influir: lo que domina es la caché del contenedor. Las dos conclusiones se deshicieron el mismo día —las lecturas vuelven a la lectora y la cifra de C-2 se mantiene—. Es el único DOC que escribí yo y retiré yo, y el más instructivo: **cada corrida era limpia por dentro; sólo compararlas entre sí la delató** | ✅ 03/11/2026 · retirada, con las cinco corridas en `PRESUPUESTO_RENDIMIENTO.md` §8 |
| **DOC-19** | La plantilla de las notas del instalador (`entrega-de-prueba.yml`): «Las dieciocho pantallas, y ARLES manda de verdad. **Al 29/10/2026**, con las diez fases del validador en verde» | La versión del **04/11/2026** se publicó con esa frase y cinco días de trabajo dentro: el filtro de contactos, el del registro, restaurar un respaldo de una versión anterior y la auditoría H. **Segunda vez en seis días** —el 29/10 se arreglaron tres frases de esta misma nota, a mano y sin dejar red—, y la fecha estaba a mano **en la plantilla**, así que no se desfasaba una vez sino en cada publicación. Es lo primero que lee quien descarga el `.exe` | ✅ 04/11/2026 · la fecha y el commit salen de `git log -1`, y lo vigilan dos comprobaciones en `--fase 1` |
| **DOC-20** | El encabezado de `entrega-de-prueba.yml`: el flujo sólo se puede lanzar con una etiqueta mientras viva en una rama de desarrollo, porque GitHub no enseña el botón «Run workflow» | Lo del botón es cierto; la conclusión no. El mismo `workflow_dispatch` **pedido por la API**, con `ref` en la rama de trabajo, construyó la versión del 04/11 — ejecución 14, `head_branch` = `claude/loving-faraday-5chmmy`. Leer sólo esa frase hace creer que **hay que fusionar a `main` para construir un instalador** | ✅ 04/11/2026 · el encabezado lleva ahora la tercera forma, con la ejecución que la midió |
| **DOC-21** | `ESTADO_ACTUAL.md` §10: «el avance está al **95 % a 2026-10-28**, y ese 5 % restante es exactamente el peso de la Fase 9» | **99 % a 2026-11-04**, y el README lo decía desde la portada del repositorio: seis días de trabajo cerrados en medio. La comprobación que ya existía compara el README con `avance.json` —las dos fuentes del §1 bis— y **este documento no entraba en esa comparación**. Es el documento por el que empieza quien retoma el trabajo, y ya había hecho esto con la 7.5 | ✅ 04/11/2026 · lo vigila `el_estado_cita_el_avance_que_hay`, verificada con tres rupturas |
| **DOC-22** | `CLAUDE.md`, en la tabla de la ronda de Dirección del 13 y 14/10: H-03 «No se pudo dar de alta una cuenta de Gmail» marcado como **«faltan tres datos de Dirección»** | **Esta tabla lo da por cerrado por R-01 desde el 14/10/2026**, así que los dos documentos llevaban **veintiún días** diciendo cosas contrarias del mismo hallazgo. Lo encontró una pregunta de Dirección —«¿en funcionalidad estamos completos?»— que obligó a mirar la lista en vez de recordarla, igual que con los bloqueos el 28/10: **lo que ve falsa una lista es usarla para algo** | ✅ 04/11/2026 · lo vigila `ningun_hallazgo_esta_abierto_y_cerrado_a_la_vez` |

**Las cinco primeras llevaban ahí desde la Fase 2** y ninguna de las cuatro
auditorías del §140 las vio, porque una auditoría mide el producto y esto es un
documento. Las caza ahora `el_sistema_de_diseno_dice_lo_que_hay` en
`validar.py --fase 2`, que compara el documento con los dos CSS y con la carpeta
de componentes, en las dos direcciones.

**DOC-6 y DOC-7 son el mismo defecto a una pasada de distancia:** dos documentos
de la Fase 0 nombrando bibliotecas que nunca se adoptaron. No tienen todavía
ninguna comprobación que los vigile, y está dicho así en vez de darlos por
seguros.

> **DOC-9 es el que más duele de los nueve, y conviene ver por qué.** Los otros
> ocho dicen algo falso sobre el producto. Éste decía falsamente que **había una
> red**, y una red que se cree es una red que hace dejar de mirar: quien leyera
> esa frase daría por comprobado algo que nadie comprobaba.
>
> Medido el día que se escribió la comprobación: **114 de 115 documentos** sí
> estaban en el índice. La regla se venía cumpliendo a mano, así que el hueco no
> se notaba — y un hueco que no se nota es el que un día se cuela.

> **DOC-10 y DOC-11 son los dos únicos de la lista que afirman de más sobre el
> propio estado del proyecto**, y por eso son los que más caro salen: no mandan
> a buscar una función que no existe, mandan a no buscar algo que falta. DOC-11
> dejaba la portada del repositorio diciendo **89 %** con el 95 % construido,
> que es el error en la dirección cómoda de ver y la incómoda de cometer —
> presenta menos trabajo hecho del que hay—; DOC-10 dejaba fuera de la lista de
> bloqueos **dos plazos de semanas** que nadie estaba esperando.
>
> **Los dos los encontró contestar una pregunta de Dirección**, no una
> auditoría: «qué sigue en el roadmap y qué falta para una beta». Las cinco
> auditorías corridas en este proyecto miden el producto; para que un documento
> de estado se vea falso hay que ir a usarlo para algo.

> **DOC-12 es el tercero que nombra algo que nunca se adoptó** —tras `tracing`
> en el `THREAT_MODEL.md` (DOC-6) y Tokio en el presupuesto (DOC-7)—, y es el
> **primero que es un ADR**. La diferencia importa: un ADR es el documento al
> que se va a preguntar *por qué* algo se decidió así, así que quien lo leyera
> construiría el hilo dedicado con canal creyendo que arregla algo.
>
> **Y de las dos mitades que describía, una estaba equivocada.** No es que
> faltara el hilo: es que su premisa escrita —«`rusqlite` es síncrono, no encaja
> en Tokio»— dejó de existir cuando el proyecto no adoptó Tokio, y construirlo
> ahora bajaría la calidad. Es el primer caso de los doce en que lo correcto no
> era construir lo prometido sino **descartarlo por escrito y decir por qué**.
>
> Los dos de la misma familia tienen además el mismo origen: la auditoría C
> corrigió esta frase **en el presupuesto** el 07/10/2026 y **no en el ADR**.
> Corregir una copia y dejar la fuente es cómo un documento falso sobrevive a su
> propio hallazgo.

---

## Dónde vive lo que no está aquí

| Qué | Dónde |
|---|---|
| Los defectos encontrados **usando** ARLES, con su estado | `06-calidad/BITACORA-DE-PRUEBAS.md` |
| Los tropiezos de instalación y arranque | `07-entrega/REGISTRO-DE-TROPIEZOS.md` |
| Qué contarme al reportar un error | `06-calidad/QUE-INFORMACION-NECESITO.md` |
| Los hallazgos de las seis auditorías | `09-fases/AUDITORIA-*.md` |
| El estado del proyecto | `07-entrega/ESTADO_ACTUAL.md` |
