# Fase 8 · Respaldos y endurecimiento — plan

**Estado:** 📋 planificada, sin empezar · **Escrito:** 2026-10-05
**Peso en la v1.2.0:** 7 de 100 (`avance.json`) · **Depende de:** Fase 7 ✅
**Lo que la sigue:** Fase 9 · Release, que no puede empezar sin las cuatro
auditorías de aquí.

> **En una frase.** Las fases 1 a 7 construyeron un producto que **acumula datos
> que no se pueden reponer**: contactos importados, el consentimiento de cada
> uno, la lista de bajas y el registro de todo lo enviado. La Fase 8 es la que
> responde a «¿y si se pierde?» — y la que comprueba, midiendo, que lo
> construido aguanta el tamaño que prometimos.

---

## 1 · Los tres hallazgos que ordenan la fase

Antes de planificar nada miré qué existe. Salieron tres cosas, y cambian el
orden de las entregas.

### 1.1 · El camino de recuperación **nunca se ha recorrido**

La base está cifrada con SQLCipher y **la clave vive en el llavero del sistema**
(`mx.telemetryinsight.arlesrelay` / `db-master-key`). El ADR-0011 decide que si
el llavero no está, **la aplicación no arranca** — y eso es correcto: lo
contrario sería degradar a texto plano en silencio.

Pero eso deja una consecuencia que nadie ha probado:

> Si se pierde la entrada del llavero, el archivo `arles.db` **sigue ahí y es
> ilegible para siempre**. No hay segunda copia de la clave. El único camino de
> vuelta es un respaldo… que todavía no existe.

Hay una prueba (`F2`, Fase 1) que comprueba que **no** se genera una clave nueva
en silencio, que era el defecto peligroso. Lo que no hay es la otra mitad: qué
hace el usuario a continuación. Hoy la respuesta honesta es «nada».

**Consecuencia para el plan:** el respaldo no es una función de conveniencia. Es
el único seguro contra una pérdida total, y por eso va primero.

### 1.2 · El respaldo `.arles` está decidido **sin cifrar**, y conviene volver a mirarlo

Tres documentos lo dicen, y dicen lo mismo:

- `FUERA_DE_ALCANCE.md` — «la base de datos está cifrada en reposo, pero el
  archivo `.arles` exportado **no lo está** en v1.2.0» (T-6, R-10).
- `PRIVACIDAD_LFPDPPP.md` — «**Hueco declarado** … contiene datos personales. Se
  advierte al usuario; el cifrado entra en v1.3».
- `GLOSARIO.md` — «⚠️ **No cifrado en v1.2.0**».

La decisión está tomada y documentada. **La vuelvo a plantear igualmente**,
porque se tomó en la Fase 0 —cuando el producto no existía— y las dos frases,
juntas, describen algo que entonces no se veía: *el único camino de recuperación
es un archivo sin cifrar con todos los datos personales de los clientes del
cliente*. Va como **D-8A**, con su estructura, en §4.

### 1.3 · El paso de rendimiento del CI lleva desde la Fase 4 «pendiente»

En `.github/workflows/ci.yml`, al final:

```yaml
# ── Pendientes de sus fases del roadmap ──
# Fase 4 · rendimiento: presupuestos con 500 000 contactos sintéticos
```

No se construyó en la Fase 4 y nadie lo movió de sitio. **No hay un solo
`bench`, ni un generador de datos sintéticos, ni una sola cifra medida** contra
las trece de `PRESUPUESTO_RENDIMIENTO.md`. Todas son estimaciones.

Es el §2 en su forma más cara: trece garantías de rendimiento que **nunca se han
visto fallar porque nunca se han comprobado**. Y la auditoría C del §140 exige
«presupuestos medidos, **no estimados**», así que sin esto la fase no cierra.

---

## 2 · Lo que hay hoy, medido

Verificado leyendo el código y el esquema el 05/10/2026.

### Ya existe y sirve

| Qué | Dónde | Para qué sirve en la Fase 8 |
|---|---|---|
| **Entregar un archivo sin cruzar una ruta** | `comandos.rs::entregar_informe_de_acceso` | **El patrón exacto** que debe seguir exportar: el diálogo lo abre el núcleo, se escribe con `std::fs::write`, y lo único que vuelve es el **nombre** del archivo. Regla 4 de la frontera |
| **Decir antes de tocar nada** | `registro.rs::que_se_recortaria` (7.4) | El precedente de un preflight destructivo: primero se dice cuánto se va y cuánto se queda, después se pulsa |
| `audit_log` append-only, por disparador | V1 | «restaurar respaldos» ya está en la lista de acciones críticas del glosario |
| `ElLlaveroDelSistema` y su contrato | `arles-app/src/llavero.rs` | Nombre del servicio fijado por una prueba, precisamente para no dejar credenciales huérfanas (§75) |
| 33 tablas, 10 migraciones (V1–V10) | `arles-db/migrations/` | El inventario de lo que un respaldo tiene que llevar |
| Derechos ARCO completos | 3.4 · `arco.rs`, `PantallaDerechos.vue` | **El ROADMAP los pone en esta fase y ya están.** Ver §3.1 |
| Lectura en streaming con topes | `arles-import` | El patrón de memoria acotada, ya probado con archivos maliciosos |
| `ningun_comando_se_queda_sin_quien_lo_llame` | validador, fase W (7.7) | Los comandos nuevos entran vigilados desde el primer día |

### No existe, y hay que construirlo

| Qué falta | Por qué importa |
|---|---|
| **El formato `.arles`** | No está definido en ningún sitio: ni su estructura, ni su versión, ni qué lleva dentro |
| **Exportar y restaurar** | Nada. Cero líneas |
| **El camino del llavero perdido** | §1.1. Hoy la aplicación no arranca y no hay nada que hacer |
| **Datos sintéticos de 500 000** | §1.3. `ESTRATEGIA_QA.md` §7 los especifica con detalle y no existen |
| **Un solo `bench`** | Trece presupuestos, cero medidos |
| **Las cuatro auditorías** | Nunca se han corrido como las cuatro pasadas aisladas que el §140 pide |

---

## 3 · Qué es la Fase 8, y qué no

El ROADMAP la define en una línea:

> Exportación e importación `.arles` · migración entre sistemas con manejo
> elegante del llavero (§75) · derechos ARCO · **las cuatro auditorías** (§140)
> · presupuestos de rendimiento a 500 k verificados.

### 3.1 · Una de las cinco ya está hecha

**Los derechos ARCO se construyeron en la 3.4**, dos fases antes: Acceso
(`entregar_informe_de_acceso`), Cancelación (`que_se_borraria` +
`cancelar_datos_de`), la pantalla `/contactos/derechos` y la lista de
supresión. El ROADMAP no se actualizó.

No es trabajo perdido: **la auditoría B de §140 vuelve a pasar por encima**, que
es donde toca verificarlos, no reconstruirlos. Lo anoto para que nadie planifique
dos veces lo mismo y para corregir el ROADMAP al cerrar la fase.

### 3.2 · Las tres preguntas que la fase contesta

En este orden, porque es el orden en que importan:

1. **¿Qué pasa si se pierde todo?** → hay un archivo, se restaura, y la
   aplicación dice con precisión qué volvió y qué no.
2. **¿Aguanta lo que prometimos?** → trece cifras medidas en lugar de trece
   estimaciones.
3. **¿Está lo construido a la altura de sus propias reglas?** → las cuatro
   auditorías, en cuatro pasadas separadas.

### 3.3 · Lo que la fase NO hace, y hay que escribirlo antes de que alguien lo pida

- **No es un respaldo automático ni programado.** Es una acción que el usuario
  pulsa. Un respaldo automático necesita decidir dónde, cada cuánto y qué pasa
  cuando el disco se llena — tres decisiones que no están tomadas y que no
  bloquean entregar.
- **No es sincronización entre equipos.** Un `.arles` se exporta y se restaura a
  mano. No hay servidor, y no lo habrá: los datos no salen de la máquina del
  cliente (`PRIVACIDAD_LFPDPPP.md` §6).
- **No lleva credenciales**, y es deliberado (§75). Un respaldo que las llevara
  sería un archivo con acceso a las cuentas de correo del cliente. La
  restauración lo dice con el texto que el modelo de secretos ya redactó.
- **No restaura «a medias».** O entra el respaldo entero o no entra nada. Una
  restauración parcial deja una base en un estado que nadie ha pensado.
- **No es el enforcement de licencias**, que está fuera de alcance por T-6 y D-4
  aunque el esquema se prepare.

---

## 4 · Las tres decisiones que esperan a Dirección

Ninguna impide empezar: la 8.0 y la 8.1 se construyen igual con cualquier
respuesta. **D-8A sí bloquea la 8.2**, que es la tercera entrega.

---

### D-8A · ¿El respaldo va cifrado?

**Problema.** La decisión vigente (T-6) es que el `.arles` de la v1.2.0 **no va
cifrado**, con un aviso al usuario. Al mismo tiempo, los documentos dicen que ese
archivo es **el único camino de recuperación si se pierde la clave del llavero**.
Las dos cosas juntas describen un archivo que contiene todos los contactos, su
consentimiento, la lista de bajas y el registro de envíos — sin cifrar — y que
además es el que el usuario tiene más incentivo en copiar a muchos sitios.

**Riesgo.** Un respaldo se manda por correo, se sube a Drive y se deja en una USB
en un cajón. Cualquiera de esos sitios convierte una fuga de un archivo en una
fuga de la base de clientes entera. Bajo LFPDPPP eso es una vulneración de
seguridad notificable, y el aviso en la interfaz **no la evita**: documenta que
sabíamos.

**Impacto.** Si ocurre, afecta a los titulares de los datos —los clientes de
nuestro cliente—, no a nosotros. Es el tipo de incidente que se descubre tarde y
no tiene marcha atrás.

**Alternativa.** Cifrar el `.arles` con una contraseña que el usuario elige al
exportar, derivada con Argon2id. La dependencia ya está en el árbol por
SQLCipher; el trabajo es de una entrega, no de una fase.

**Trade-off honesto.** Una contraseña que el usuario olvida convierte el respaldo
en inservible — y el respaldo es justo lo que se usa cuando ya se perdió algo. Se
cambia el riesgo de *fuga* por el riesgo de *no poder restaurar*. Por eso la
contraseña se pide dos veces, se avisa con todas las letras de que no hay forma
de recuperarla, y **el cifrado se puede desactivar a propósito** en el mismo
diálogo, con un aviso que diga qué significa.

**Recomendación.** **Cifrar, con opción explícita de no hacerlo.** El riesgo de
olvidar una contraseña lo decide y lo asume quien exporta, en el momento, con la
advertencia delante. El riesgo de la fuga lo asumen terceros que no eligieron
nada. Entre un riesgo que alguien elige y uno que nadie eligió, se protege el
segundo.

*Si la respuesta es «no», se construye sin cifrar y el aviso de la interfaz pasa
a ser obligatorio y no omitible — y se anota como riesgo aceptado con fecha.*

---

### D-8B · ¿Qué se lleva el respaldo?

**Problema.** `message_attempt` guarda **un renglón por destinatario y envío, y
no se borra jamás** (D-7C). En una instalación de dos años con campañas
mensuales a 50 000 contactos son millones de filas. Un respaldo que lo lleve
todo es enorme y lento; uno que no lo lleve pierde la prueba de qué se envió a
quién.

**Riesgo.** Si el respaldo tarda veinte minutos y produce un archivo de 2 GB,
nadie lo hace. Un respaldo que no se hace no protege de nada.

**Impacto.** Directo sobre el valor de toda la fase.

**Alternativa.** Dos alcances en el mismo diálogo:
- **Completo** — todo, incluido el registro de envíos. Para migrar de equipo.
- **Esencial** — contactos, consentimiento, **bajas**, plantillas, campañas y
  configuración; del registro de envíos, sólo los desenlaces definitivos de los
  últimos doce meses. Para el respaldo del día a día.

En los dos, las **bajas van completas y nunca se recortan**: son el control
anti-abuso del producto y perderlas significa volver a escribir a quien pidió que
no.

**Trade-off.** Dos modos son dos caminos que probar y dos que explicar. La
alternativa —un solo modo completo— es más simple de construir y peor de usar.

**Recomendación.** **Los dos, con «Esencial» por defecto**, y el diálogo diciendo
en cifras qué se lleva cada uno antes de elegir (§94). La 8.0 declara el
manifiesto de forma que añadir el segundo modo sea una línea.

---

### D-8C · Restaurar sobre una base que ya tiene datos

**Problema.** El caso normal de restaurar es un equipo nuevo y vacío. El caso que
rompe cosas es restaurar sobre una instalación **que ya tiene datos**: ¿se
fusiona, se reemplaza, o se rechaza?

**Riesgo.** Fusionar sin querer duplica contactos y, peor, puede **resucitar una
baja**: alguien que pidió no recibir vuelve a estar en la lista porque el
respaldo es de antes de que lo pidiera. Reemplazar sin avisar destruye datos que
no estaban en el respaldo.

**Impacto.** El primero es el daño más caro que este producto puede hacer:
escribir a quien dijo que no.

**Alternativa.** **Reemplazar, nunca fusionar**, con el mismo patrón que el
recorte bimestral de la 7.4: un preflight que dice **antes de tocar nada** qué
hay ahora, qué trae el respaldo y qué se va a perder, con cifras. Y una regla
que no se negocia: **las bajas se unen, no se reemplazan** — la lista final es la
unión de las dos, porque una baja nunca se deshace sola.

**Trade-off.** Fusionar sería más flexible. También sería imposible de explicar y
de verificar, y la flexibilidad aquí no vale lo que cuesta.

**Recomendación.** **Reemplazar con preflight, bajas unidas.** Y queda anotado en
`audit_log`, que ya lista «restaurar respaldos» entre sus acciones críticas.

---

## 5 · Las entregas, en orden

El orden no es arbitrario: cada una desbloquea la siguiente, y las dos de
auditoría van al final porque **auditan lo que las anteriores construyeron**.

---

### 8.0 · El manifiesto del respaldo, y la comprobación que impide olvidar una tabla

**Qué.** Una declaración, en `arles-core`, de qué tablas entran en un respaldo y
en qué modo (completo / esencial / nunca). Más la comprobación que la vigila.

**Por qué va primero, y por qué es la entrega más importante de la fase.**

Es el patrón del §9 esperando a ocurrir, en su forma más cara. El día que
alguien añada una tabla en la V11 y no la ponga en el respaldo, **nada falla**:
se exporta, se restaura, y los datos de esa tabla desaparecen sin un solo error.
Lo descubre el usuario, meses después, cuando ya no hay de dónde sacarlos.

La comprobación —`ninguna_tabla_se_queda_fuera_del_respaldo`— lee las tablas
**del esquema**, no de una lista a mano, y exige que cada una esté clasificada.
Es hermana de `ninguna_tabla_se_queda_sin_escritor`, que ya existe, y se escribe
**antes** que el exportador para verla en rojo (§2).

Una tabla puede declararse «nunca» —`ui_preference` es de este equipo, no de
estos datos— pero hay que **decirlo y por qué**. El olvido no tiene dónde
esconderse.

**Hecho cuando:** las 33 tablas están clasificadas con su motivo, la comprobación
falla al añadir una tabla sin clasificar, y se ha visto fallar.

---

### 8.1 · El formato `.arles`, en el núcleo

**Qué.** La estructura del respaldo y su serialización. **Cero I/O**: entra una
estructura, sale un texto; entra un texto, sale una estructura o un error
tipado.

**Decisiones de formato, y por qué:**

| Decisión | Por qué |
|---|---|
| Lleva un **número de versión del formato**, separado de la versión de la aplicación | Una v1.3 tiene que poder leer un respaldo de la v1.2. Sin el número, la única forma de saberlo es adivinar |
| Lleva la **versión del esquema** (`V10`) | Restaurar un respaldo más nuevo que la aplicación tiene que **rechazarse con un mensaje claro**, no intentarse |
| Lleva una **suma de verificación** del contenido | Un archivo truncado por un disco lleno o un correo que lo cortó se detecta **antes** de tocar la base, no a mitad |
| Lleva **cuántas filas de cada tipo** declara | Permite enseñar el preflight de la restauración sin leer el archivo entero |
| **No** lleva rutas, ni nombres de equipo, ni nada del sistema | Regla 4 de la frontera, y un respaldo no debe decir de qué máquina salió |

**Hecho cuando:** un respaldo se redacta y se vuelve a leer sin perder nada
(ida y vuelta probada sobre un caso con acentos, comillas y campos vacíos), un
archivo truncado se rechaza con su clave de error, y uno de una versión futura
también.

---

### 8.2 · Exportar

**Qué.** El comando que lee la base, arma la estructura y escribe el archivo.

**Depende de D-8A** (cifrado) y **D-8B** (alcance).

- Sigue **el patrón del informe ARCO**: el diálogo lo abre el núcleo, lo único
  que vuelve a la webview es el **nombre** del archivo (regla 4).
- **Memoria acotada**: se escribe por tandas, como lee `arles-import`. Exportar
  500 000 contactos no puede cargarlos todos en memoria.
- **Avance visible**: con medio millón de filas, una barra parada veinte segundos
  es indistinguible de un programa colgado. Cifras, no una rueda (§94).
- Queda en `audit_log`.

**Hecho cuando:** se exporta una base poblada, el archivo se vuelve a leer, y la
memoria del proceso se mide y se queda bajo el techo.

---

### 8.3 · Restaurar, con su preflight

**Qué.** Leer un `.arles`, decir **qué va a pasar**, y sólo entonces hacerlo.

**Depende de D-8C.**

El preflight dice, antes de tocar nada y con cifras:

- Qué trae el respaldo: contactos, bajas, campañas, plantillas, y de cuándo es.
- Qué hay ahora, y **qué se va a perder**.
- Qué **no** trae: las credenciales. Con el texto que `MODELO_DE_SECRETOS.md` §7
  ya redactó, palabra por palabra.
- Si el respaldo es de una versión que esta aplicación no sabe leer, lo dice
  aquí y no deja continuar.

Y cuando se confirma: se restaura **entero o nada** (una transacción), las
cuentas aparecen en `needs_reauth` con su botón de reconectar, y queda en
`audit_log`.

**Hecho cuando:** restaurar sobre una base vacía funciona; restaurar sobre una
poblada avisa de lo que se pierde y, confirmado, lo reemplaza; **las bajas de las
dos se unen**; y una restauración interrumpida a mitad deja la base como estaba.
Lo último se prueba **provocando el fallo**, no suponiéndolo.

---

### 8.4 · El llavero perdido: el camino que nadie ha recorrido

**Qué.** La otra mitad del ADR-0011.

Hoy, sin la entrada del llavero, la aplicación no arranca — correcto — y punto.
Esta entrega hace que a continuación haya algo que hacer:

- **Decir qué pasó, con las tres partes del §95**: qué pasó, cómo se arregla y
  **qué está a salvo**. La tercera es la que importa: los datos no se han
  borrado, están cifrados con una clave que ya no está.
- Ofrecer **empezar de cero restaurando un respaldo**, que es la única salida
  real.
- **No ofrecer nada que no funcione.** Si no hay respaldo, se dice que no hay
  salida, con esas palabras. Un botón que no puede funcionar es peor que ninguno.

**Hecho cuando:** se borra la entrada del llavero a propósito, se arranca, y la
pantalla dice las tres cosas y lleva a restaurar. Se verifica **rompiéndolo**.

> **Límite declarado:** esto se puede probar entero en Linux, donde el llavero se
> simula. En Windows y macOS se prueba cuando se cierre **R-07**. Se dirá así, y
> no «probado» a secas.

---

### 8.5 · Los 500 000 contactos sintéticos, y trece cifras medidas

**Qué.** El generador, el banco de pruebas y el paso de CI que lleva pendiente
desde la Fase 4 (§1.3).

**El generador**, tal como `ESTRATEGIA_QA.md` §7 ya lo especificó: semilla fija y
reproducible, con acentos, correos en mayúsculas y con `+`, campos vacíos,
duplicados exactos y por normalización, direcciones inválidas y **nombres que
parecen fórmulas**. Nunca datos reales, ni anonimizados.

**Las trece cifras** de `PRESUPUESTO_RENDIMIENTO.md` se miden. Las que no se
puedan medir desde aquí —las que necesitan la máquina de referencia o una
pantalla— **se declaran sin medir**, que no es lo mismo que cumplidas. Es la
misma regla que el validador aplica a las sondas omitidas.

**Y se espera que algo falle.** Trece presupuestos estimados en la Fase 0 y nunca
comprobados: que salgan los trece a la primera sería la señal de que el banco de
pruebas no está midiendo lo que dice. Lo que se mida se escribe con su fecha,
pase o no.

**Hecho cuando:** el conjunto se genera dos veces y sale idéntico; las cifras
están medidas y escritas; y lo que no se pudo medir dice por qué.

---

### 8.6 · Las cuatro auditorías (§140)

**Qué.** Cuatro pasadas **separadas**, no una revisión general. Es la condición
del §140 y tiene motivo: una revisión general encuentra lo que salta a la vista,
cuatro pasadas con una pregunta cada una encuentran lo que no.

| | Pasada | Qué se mira |
|---|---|---|
| **A** | Arquitectura | Fronteras entre crates · `arles-core` sin I/O · la webview sin lógica de negocio · sin `unwrap()` ni `any` indiscriminados (§138) |
| **B** | Seguridad | `THREAT_MODEL.md` §8 entero · ningún comando devuelve un secreto · capabilities denegadas por defecto · CSP · sanitizado en Rust · topes de importación · `cargo audit` y `npm audit` · **y los derechos ARCO de la 3.4, que es donde tocaba verificarlos** |
| **C** | QA y rendimiento | Casos frontera · **los presupuestos de la 8.5, medidos y no estimados** · sin fugas de memoria en ejecuciones largas del motor |
| **D** | UX y accesibilidad | Contraste AA calculado · foco visible · teclado · ningún estado sólo por color · `prefers-reduced-motion` · objetivos ≥32 px al 200 % · **los cuatro estados en cada pantalla (§97)** · errores con las tres partes (§95) |

**C depende de la 8.5**, por eso va después.

**Cada hallazgo se arregla o se declara**, con su motivo y su fecha. Una auditoría
que produce una lista y no produce cambios no es una auditoría: es un documento.

> **Lo que la auditoría D no puede hacer desde aquí:** mirar las pantallas en
> WKWebView. Eso es **R-07** y sigue esperando a alguien con un Mac. La vista
> previa no cuenta: ahí el motor es el del navegador.

---

### 8.7 · Cierre de fase

Documento de fase, informe para Dirección, avance recalculado, y **la revisión a
ojo** en Chromium que la 7.7 volvió a demostrar que hace falta: tres de los siete
hallazgos de aquella sólo se vieron mirando, con las sondas en verde.

Y una corrección pendiente: **el ROADMAP pone los derechos ARCO en esta fase y
están desde la 3.4** (§3.1).

---

## 6 · Escalabilidad: lo que esta fase decide para las que vienen

Esta fase toca la frontera entre lo que el producto guarda y el mundo exterior.
Lo que se decida aquí es caro de cambiar después, porque habrá archivos `.arles`
en discos de clientes.

**1 · El formato tiene versión desde el primer archivo.** No se añade después:
un archivo sin número de versión es indistinguible de uno de la versión 1, y en
cuanto exista uno de esos en un disco ajeno, la compatibilidad se vuelve
adivinación. La regla: **la aplicación lee su versión y las anteriores; una
posterior se rechaza con un mensaje que dice qué versión hace falta.**

**2 · El manifiesto hace que crecer sea barato.** Con la 8.0 en su sitio, añadir
una tabla en la V11 obliga a clasificarla. Sin ella, cada migración futura es una
oportunidad de perder datos en silencio. Es la diferencia entre un sistema que
aguanta crecer y uno que acumula deuda invisible.

**3 · El cifrado del respaldo es reversible en una dirección y no en la otra.**
Si la v1.2 exporta sin cifrar y la v1.3 cifra, la v1.3 tiene que seguir leyendo
los archivos viejos — fácil. Si la v1.2 cifra, no hay nada que mantener después.
Es un argumento más para D-8A, y es de ingeniería, no de seguridad.

**4 · El alcance «Esencial» es lo que mantiene el respaldo usable con el
tiempo.** `message_attempt` crece sin techo por diseño (un desenlace no se borra
jamás). Sin dos alcances, el respaldo se vuelve impracticable justo en las
instalaciones con más que perder.

**5 · Los presupuestos medidos son la línea base de todo lo que venga.** Trece
cifras con fecha permiten detectar degradación lenta — tres regresiones del 15 %
no rompen nada y duplican el tiempo. Sin la primera medición no hay contra qué
comparar, y la v1.3 empezaría igual de ciega.

**6 · Lo que esta fase deja preparado para la v1.3**, sin construirlo: el
cifrado del respaldo (si D-8A dice que no), el respaldo programado, y el
`FeatureEntitlement` que el esquema ya contempla para el licenciamiento.

---

## 7 · Riesgos de la fase

| Riesgo | Probabilidad | Impacto | Mitigación |
|---|---|---|---|
| Una restauración destruye datos que no estaban en el respaldo | **Alta** si no se controla | **Muy alto**, irreversible | D-8C: preflight con cifras antes de tocar nada, y bajas unidas |
| Se añade una tabla y nadie la mete en el respaldo | **Alta** con el tiempo | Alto y silencioso | La 8.0, escrita antes que el exportador |
| El `.arles` sin cifrar se filtra | Media | Muy alto, y de terceros | D-8A |
| Los presupuestos no se pueden medir aquí como en la máquina de referencia | **Alta** | Medio | Se mide lo que se pueda y **se declara qué no**, como con las sondas omitidas |
| El llavero sólo se prueba en Linux | **Alta** (ya pasa) | Medio | Se declara; se cierra con R-07 |
| Las cuatro auditorías producen más trabajo del que cabe en la fase | Media | Medio | Cada hallazgo se arregla **o se declara con fecha**. No se tapan |
| Las pruebas de 500 k hacen el CI lento | Media | Bajo | Van en su propio trabajo, no en cada push |

---

## 8 · Cómo se verifica la fase

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check licenses advisories        # si D-8A añade dependencia de cifrado

cd app && npm test && npm run lint && npx vue-tsc --noEmit -p tsconfig.json

# Las DIEZ fases, con sondas. Correr sólo la fase en curso no basta: está
# contado en ESTADO_ACTUAL.md §8 bis y volvió a pasar en la 7.7.
for f in 0 1 2 3 4 5 6 W 7 8; do python3 herramientas/validar/validar.py --fase $f; done
```

Y lo que ninguna orden hace: **abrir la demostración y mirar**. Va en la 8.7 y no
es opcional.

**La fase no está cerrada mientras:**

- [ ] Una tabla pueda quedarse fuera del respaldo sin que nada falle
- [ ] Restaurar pueda destruir datos sin avisar antes, con cifras
- [ ] Los presupuestos sigan siendo estimaciones
- [ ] Alguna de las cuatro auditorías no se haya corrido **aislada**
- [ ] Un hallazgo de auditoría esté sin arreglar **y sin declarar**

---

## 9 · Resumen para quien tenga prisa

| | Entrega | Depende de | Bloquea |
|---|---|---|---|
| **8.0** | El manifiesto, y la comprobación que impide olvidar una tabla | — | todo lo demás |
| **8.1** | El formato `.arles` en el núcleo | 8.0 | 8.2, 8.3 |
| **8.2** | Exportar | 8.1 · **D-8A**, **D-8B** | 8.3 |
| **8.3** | Restaurar, con su preflight | 8.2 · **D-8C** | 8.4 |
| **8.4** | El llavero perdido | 8.3 | — |
| **8.5** | 500 k sintéticos y trece cifras medidas | — *(se puede adelantar)* | 8.6 C |
| **8.6** | Las cuatro auditorías (§140) | 8.5 | Fase 9 |
| **8.7** | Cierre: documento, informe, avance y **mirar** | 8.6 | Fase 9 |

**Se puede empezar hoy por la 8.0**, que no depende de ninguna decisión.
