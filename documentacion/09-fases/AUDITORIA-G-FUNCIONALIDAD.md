# Auditoría G · Funcionalidad de punta a punta

**22/10/2026.** Pedida por Dirección: *«analiza la aplicación a detalle,
revísala, audítala y verifica que ya no existan errores de funcionalidad, si
existen corrígelos y se cumplan todos los requisitos que hemos estado
detallando»*.

Las cuatro auditorías del §140 midieron arquitectura, seguridad, rendimiento y
accesibilidad. **Ésta mide otra cosa:** si lo que el usuario escribe llega a
donde tiene que llegar. Es la clase de defecto que las otras cuatro no podían
ver, porque todas sus comprobaciones pasaban.

---

## 0 · El resultado, en una línea

**Seis hallazgos. Seis arreglados.** Las diez fases del validador estaban en
verde al empezar, y siguen en verde.

| | Qué | Gravedad |
|---|---|---|
| **G-1** | El límite diario que se escribe en la campaña **no se aplicaba** | 🔴 Alta |
| **G-2** | Un límite de `0` creaba una campaña que se activa y no manda nada | 🟠 Media |
| **G-3** | La firma elegida nunca se guardaba: todas las campañas usaban la misma | 🟠 Media |
| **G-4** | La demostración no exigía aceptar el aviso de volumen | 🟡 Baja |
| **G-5** | Dos claves de error sin texto: se veía la clave cruda en pantalla | 🟡 Baja |
| **G-6** | Cinco cabeceras de Contactos no caben a 1024 px | 🟡 Baja |

Y **un límite declarado, que no es un defecto sino una fase que falta**: §4.

---

## 1 · G-1 · El límite diario no se aplicaba

**Lo que Dirección vio:** *«tampoco vi que pudiera modificar el límite de
correos por día»*.

**Lo que había:** el campo existe desde la Fase 6, en el paso 5 del asistente,
con su aviso de riesgo y su casilla de aceptación. Lo que no existía era el
efecto.

El motor tomaba **el menor** entre el límite de la campaña y el de la cuenta
remitente. Y toda cuenta nace con `diario: 50, por_hora: 10` escritos en el
código, sin ninguna pantalla para cambiarlos. Medido el 22/10/2026:

| Se escribe en la campaña | La cuenta trae | **Sale de verdad** |
|---|---|---|
| 300 | 50 | **50** |
| 500 | 50 | **50** |
| 20 | 50 | 20 |

Es decir: se escribe 300, se lee el aviso de riesgo, se marca la casilla de
aceptación — **y salen 50, sin que nada lo diga.**

> **Peor que faltar la función: la aplicación dejaba creer que se había usado.**
> Un campo que no hace nada y no lo dice es la forma en que un producto pierde
> la confianza de quien lo usa, porque el siguiente número que enseñe tampoco
> se creerá.

**La decisión fue de Dirección** (22/10/2026): el límite se fija al crear la
campaña. Así que el `min` se quitó y manda la campaña.

### Qué protege ahora en lugar de ese `min`

El `min` existía por una razón buena —no pasarse del límite del proveedor sin
darse cuenta—, así que quitarlo exige decir qué ocupa su sitio:

1. **El presupuesto cuelga de la CUENTA, no de la campaña.** Dos campañas con
   300 sobre el mismo remitente no gastan 600: comparten el contador de
   `rate_budget`. El total de una cuenta en un día es el **mayor** de los
   límites de sus campañas, nunca la suma. Lo ejercita
   `dos_campanas_sobre_una_cuenta_no_suman_sus_topes`.
2. **El aviso del §48 con su casilla**, que queda registrado en `audit_log` con
   su fecha: el rastro de que se informó y de que alguien dijo que sí.
3. **ADR-0008 sigue intacto.** No hay rotación de remitentes. Repartir un envío
   entre cuentas para superar el total agregado es lo que ARLES no hace, y este
   cambio no lo toca.

> **Y el techo que se quitó no era el del proveedor: era una constante.** Titan
> y Workspace admiten muchos más de 50; ése es el suelo de una cuenta de Gmail
> personal. Quien sabe cuánto admite su proveedor es el usuario, y el §154 pide
> informar, no dictar. Un tope que no se puede ver ni cambiar no informa de
> nada.

---

## 2 · G-2 · Un cero creaba una campaña muda

Escribir `0` como máximo se guardaba tal cual. La campaña se creaba, se
activaba, ponía «en marcha» en la pantalla — **y no mandaba nunca nada**. Sin
error que mirar, sin cola que explicara la espera.

Y un negativo era peor: Rust lo descartaba y caía en el límite de la cuenta, así
que el número que se veía en pantalla no era el que se aplicaba.

Arreglado en los dos sitios, a propósito:

- **En el paso 5**, que dice qué pasa y no deja seguir. Es donde se puede
  corregir sin perder lo escrito.
- **En `crear_campana`**, con `DbError::LimiteDeEnviosEnCero`. La frontera no se
  fía de la webview: quien decide qué entra en la base es esa capa.

> **La red que ya existía se quedó, y se dice por qué.** Una validación de
> entrada no protege a las filas que ya están dentro. `un_tope_de_cero_lo_caza_la_estimacion`
> —que cazaba el cero por la cadena entera: techo 0 → la estimación no avanza →
> el preflight bloquea— se reescribió para sembrar el cero **en la etapa**, que
> es el camino que queda abierto: una base anterior a esta guarda, o una fila
> tocada a mano. Borrarla al añadir la validación habría sido el error.

---

## 3 · G-3 · La firma elegida no se guardaba

`campaign_stage.signature_id` existe desde la **migración V4**. La consulta que
arma el correo, `redaccion_de_la_campana`, ya la miraba —con su caída a la firma
por defecto— desde la Fase 6.

**Y nada la escribía.** El asistente no preguntaba por la firma, el comando
mandaba `firma: None` fijo, y la columna llevaba meses vacía.

El resultado: todas las campañas salían con la misma firma, la marcada por
defecto. Nada fallaba. Es el patrón del §9 —una pieza correcta y el cable sin
conectar— en su forma más silenciosa, porque aquí ni siquiera faltaba una
pantalla: faltaba **un valor**.

Arreglado de punta a punta: desplegable en el paso 4 → store → comando →
`CampanaNueva.firma` → `INSERT`. Sin migración: la columna ya estaba.

> **Y dos pruebas, no una, porque ninguna de las dos sola cubre el camino.**
> La de `plantillas.rs` recorre las dos ramas del `COALESCE` sembrando la etapa
> con SQL; la de `alta_de_campana.rs` pasa por `crear_campana`. **La primera
> versión del comentario afirmaba estar verificada quitando el `INSERT` — y era
> falso**: se probó, y pasó en verde, porque siembra la etapa a mano. El hueco
> vivía justo entre las dos pruebas, que es el patrón del §9 aplicado a sus
> propias pruebas.

---

## 4 · Lo que no se puede hacer todavía, y no es un defecto

**No se puede lanzar una campaña por WhatsApp.** Sólo el envío de prueba.

Está declarado desde el cierre del canal W, y esta auditoría lo midió otra vez:
el camino entero está cableado a `channel = 'email'` en **siete sitios** —
`crear_campana`, la consulta de audiencia, la revisión, el preflight, la cola de
trabajo, `redaccion_de_la_campana` y el motor.

No es un cable suelto que se pueda conectar: es una fase de trabajo. Se dice
aquí porque un producto que ofrece «campañas de correo y WhatsApp» tiene que ser
claro sobre cuál de las dos puede lanzar hoy.

**Y R-05 —imagen en la firma— sigue sin construirse.** El editor de firmas
admite negrita y cursiva desde el 21/10; una imagen exige decidir si va
embebida (dispara filtros de spam y engorda cada correo) o alojada (hay que
alojarla en algún sitio), y el saneado del HTML en Rust tendría que admitir
`<img>` con sus reglas. Es una decisión de Dirección, no un olvido.

---

## 5 · Lo que se midió y salió limpio

Conviene dejarlo escrito, porque es la mitad del valor de una auditoría:

- **Los 37 argumentos que las pantallas mandan a los 67 comandos.** Se comparó
  cada clave enviada con los parámetros que el comando declara, resolviendo
  camelCase ↔ snake_case: **ningún desajuste**. Era la familia del H-05 —el enum
  con dos nombres que bloqueó el producto cuatro días— y está limpia.
- **Los comandos sin quien los llame** (`--fase W`): ninguno.
- **Las tablas sin quien las escriba**: ninguna.
- **Las pantallas sin ruta**: ninguna.
- **Las dieciocho pantallas en Chromium**, a cuatro tamaños de ventana: sin un
  solo texto encimado ni cortado tras arreglar G-6.

> **Y un aviso sobre el método.** La primera medición de los argumentos dio
> **130 hallazgos**, todos falsos: la expresión regular capturaba hasta el
> `return {` del final de cada store. Se rehízo contando llaves. Es la lección
> que las auditorías B, C y D aprendieron cada una por su cuenta —**comprobar
> antes de anotar**— y la cuarta vez que hace falta.

---

## 6 · Los defectos de las propias comprobaciones

Tres, y los tres cazados antes de dar nada por bueno:

1. **El aviso del teléfono avisaba de «Teléfono móvil»**, que dice exactamente
   de qué número habla. Usaba el prefijo de los genéricos.
2. **La comprobación de la plantilla medía el primer `cuerpo:` de `es.ts`**,
   donde hay uno por pantalla: salió **en rojo sobre dos textos correctos**. Es
   la hoja y no la ruta, el mismo defecto que la 7.7 arregló en la del §139.
3. **Y su comentario afirmaba una verificación en las dos direcciones** que era
   falsa. Está corregido en el docstring, con por qué esa dirección no se puede
   medir sin hallazgos falsos.

Y uno más, el que vale la pena contar: al restaurar una ruptura del presupuesto,
`grep -c "cuenta-b"` devolvió **1** — y el código estaba bien restaurado: la
única aparición era **dentro de mi propio comentario** describiendo la ruptura.
Es el noveno caso del proyecto del comentario que se caza a sí mismo, y el
primero detectado antes de trabajar sobre un archivo roto.

---

## 7 · Los requisitos de Dirección, uno a uno

| | Requisito | Estado |
|---|---|---|
| R-01 | Alta de correo en dos campos | ✅ 14/10 |
| R-02 | Alta de WhatsApp simplificada | ✅ 14/10, con límite declarado |
| R-03 | Negrita y cursiva en plantillas | ✅ 21/10 |
| R-04 | La firma no lleva datos de quien recibe | ✅ 21/10 |
| R-05 | **Imagen en la firma** | ⬜ Pendiente de decisión (§4) |
| R-06 | Borrar plantillas y firmas sin perderlas | ✅ 21/10 |
| R-09 | Editar y dar de baja desde la fila del contacto | ✅ 21/10 |
| H-01…H-05 | La ronda de correcciones del 13-16/10 | ✅ |
| — | Que el correo no caiga en no deseados | ✅ 22/10, tres bandejas de tres |
| — | Plantilla de contactos descargable | ✅ 22/10 |
| — | Reconocer las columnas de la tabla del cliente | ✅ 22/10 |
| — | **Fijar el límite de correos al día** | ✅ 22/10 (G-1) |
| — | Elegir qué correo usa la campaña | ✅ Ya estaba, paso 3 |
| — | **Lanzar una campaña por WhatsApp** | ⬜ Fase pendiente (§4) |
