# Fase 7 · Actividad — lo construido, y lo que la auditoría encontró

> **Estado al 01/10/2026.** Siete entregas cerradas (7.0 a 7.4, 7.6 y 7.7). La
> **7.5 —SPF, DKIM y DMARC— está movida al final** por decisión de Dirección del
> 30/09/2026: es lo único de la fase que necesita red y una dependencia nueva, y
> sólo se puede verificar de verdad el día que se cierre R-24.
>
> Esta fase hizo que ARLES **cuente lo que hace**. Antes enviaba y la única
> forma de saber qué pasaba era abrir la base de datos.

---

## 1 · Qué se construyó

| Entrega | Qué es | Dónde vive |
|---|---|---|
| **7.0** | **El freno**: pausar, reanudar, detener y la parada de emergencia, con comando y botón | `arles-app/src/control.rs`, `PantallaCampanas.vue` |
| **7.1** | El registro de actividad: el motor deja de tirar lo que sabe | `arles-db/src/actividad.rs`, migración V10 |
| **7.2** | Las cifras de cada campaña, calculadas en un solo sitio | `arles-db/src/cifras.rs` |
| **7.3** | La pantalla de Actividad: una tarjeta por campaña y **por qué no sale nada** | `PantallaActividad.vue`, `stores/actividad.ts` |
| **7.4** | El registro con sus seis filtros y el recorte bimestral (D-7C) | `arles-app/src/registro.rs`, `PantallaRegistro.vue` |
| **7.6** | El resumen exportable, con el método de cada cifra | `arles-core/src/resumen.rs` |
| **7.7** | Esta auditoría, y lo que salió de ella | ver §3 |

Con Actividad construida, **las seis secciones tienen su pantalla**, así que el
andamio (`PantallaPendiente.vue`) se fue con ella.

---

## 2 · Las tres decisiones de Dirección, ya construidas

- **D-7A · Las respuestas se cuentan, nunca se leen.** Por campaña, y una
  persona que conteste cinco veces suma uno. El texto del mensaje no existe en
  la base: se descarta al recibirlo, antes de escribir nada.
- **D-7B · El freno vive en Campañas**, donde está la lista, y Actividad lo
  enseña porque es la razón número uno por la que no sale nada.
- **D-7C · El registro se filtra por seis criterios** que viajan en la
  dirección, nunca más de 500 filas con el total al lado, y el recorte bimestral
  dice **antes de tocar nada** cuánto se va y cuánto se queda. **Un desenlace de
  envío no se borra jamás.**

---

## 3 · La auditoría de cierre (7.7)

Se hizo con las nueve fases del validador en verde y las 187 pruebas del
frontend pasando. Encontró **siete cosas**, y ninguna la veía ninguna prueba.

### 3.1 · Tres comandos registrados que nadie llamaba

El patrón del §9, visto por fin desde el único sitio que lo ve entero: **comando
registrado → alguien que lo invoque**. Había tres comprobaciones persiguiendo
este patrón y cada una miraba un tramo distinto del cable; el tramo de en medio
—el único que el usuario pisa— no lo miraba nadie salvo para los seis comandos
de `control.rs`.

| Comando | Qué pasaba | Qué se hizo |
|---|---|---|
| `enviar_whatsapp_de_prueba` | Construido en la W.8, probado de punta a punta contra el servidor de mentira, **y sin botón**. Desde ARLES no se podía mandar un WhatsApp | Se construyó el botón (§3.2) |
| `revisar_campana_de_whatsapp` | El preflight del canal (W.5). Es la puerta de un flujo que **no existe**: el motor sólo recorre `Canal::Correo` | Se quitó del registro. La función y sus pruebas se quedan; vuelve el día que el motor recorra WhatsApp |
| `contacto` | La ficha completa de un contacto. `listar_contactos` ya trae todos los canales y la ficha se recalcula de la lista a propósito | Se quitó. `Db::contacto` **se queda**: seis pruebas lo usan, una de ellas la de aislamiento entre empresas |

**No es pulcritud: cada comando registrado es superficie de ataque.** Una puerta
que nadie usa es una puerta que nadie vigila, y las cuatro reglas de la frontera
hay que sostenerlas en todas, no sólo en las que alguien abre.

Lo caza ahora `ningun_comando_se_queda_sin_quien_lo_llame` (fase W), que lee la
lista **de `generate_handler!`** y no a mano. Escribirla tuvo su propia lección:
la primera versión miraba `pantallas/` y `stores/` más dos archivos sueltos, y
dio un falso positivo inmediato con un comando que se invoca desde `App.vue`.
Una lista a mano de dónde mirar, cometiendo el defecto que perseguía.

### 3.2 · El envío de prueba de WhatsApp tenía **dos** capas de hueco

No sólo faltaba el botón. Las **seis claves** que el comando devuelve
—`whatsapp.prueba.*`— **no tenían texto** en el catálogo, así que el día que
alguien pusiera el botón habría visto «whatsapp.prueba.aceptado» en pantalla.

Lo mismo con las cinco de `whatsapp.plantilla.*`, los estados de una plantilla en
Meta. Ésas importan más de lo que parece: **una plantilla que Meta pausa deja de
enviar**, y sin la frase la campaña fallaría sin que nadie supiera por qué. Ya
tienen texto; la pantalla que los enseña sigue pendiente y está en §4.

### 3.3 · La comprobación del §139 cubría otra vez una parte

El §139 dice **toda** clave. La comprobación tenía los dos defectos que ya había
tenido una vez:

1. **Una lista de prefijos escrita a mano** —`campana|motor|reintento|ritmo|…`—
   sin `whatsapp` ni `plantilla`. Once claves sin vigilar.
2. **Comparaba la hoja, no la ruta.** Estaba anotado como límite conocido con el
   argumento de que `errores.spec.ts` resuelve la ruta de verdad — pero ese test
   sólo cubre `error.*`.

Así pasó en verde `plantilla.variable.miEmpresa`, que **no existe**: el catálogo
la tiene como `mi_empresa`. La hoja existía, la ruta no. Nadie lo vio porque las
pantallas no usan ese método: construyen la clave con `clave()`, que era la
buena. El método estaba mal **y no lo llamaba nadie**.

Ahora los prefijos **son los bloques del propio catálogo** y se resuelve la ruta
entera. Verificado moviendo un bloque de rama sin tocar sus hojas: lo caza.

Y en el núcleo, `la_clave_de_i18n_se_deriva_de_la_clave` ata las dos para que no
puedan volver a separarse.

### 3.4 · «Detener» era ilegible: 2.41:1

El botón más destructivo del producto —y el único freno de una campaña en
marcha— llevaba tinta oscura sobre rojo oscuro en el tema claro. Menos de la
mitad del mínimo de AA.

`.v-peligro` usaba `--arles-text-on-accent`, la tinta del dorado. El token
correcto, `--arles-text-on-danger`, **existía desde la Fase 2** con su comentario
—«en oscuro el rojo es claro; en claro es oscuro, la tinta se invierte»— y no lo
usaba nadie en el botón. `AInsignia` sí lo usa, con un comentario que cuenta
este mismo defecto arreglado en la insignia **y no en el botón**.

No se cazaba antes porque hasta la 7.0 no hubo ningún botón de peligro en una
pantalla: la variante existía y nada la usaba. Ahora da **6.64:1**.

### 3.5 · Tres pantallas de catorce no tenían título de página

Remitentes, Bajas y Derechos son un módulo y ya: como `AModulo` siempre pintaba
un `h2`, esas tres **no tenían ningún `h1`**. Para quien usa un lector de
pantalla, «ir al encabezado 1» no llevaba a ninguna parte; para quien mira, once
pantallas se presentan con su nombre y tres empezaban con una tarjeta.

La sonda decía vigilar la jerarquía de encabezados y no lo veía: `previo`
empezaba en 0 y la condición era `if (previo && …)`, así que **el primer
encabezado nunca se comprobaba**. Una pantalla que empieza en `h2` pasaba en
verde, porque no hay salto entre dos encabezados cuando el que falta no está ahí
para saltárselo.

Se encontró **midiendo dónde empieza el `h1` de cada pantalla** y viendo que en
tres no había ninguno.

### 3.6 · Y el arreglo tardó tres intentos, por la cascada

Poner `nivel="h1"` dejó la sonda en verde y **la captura igual de pequeña**. El
selector tenía que vencer a `.modulo.principal .titulo`, y lo que no había
mirado es que `<style scoped>` añade el atributo `[data-v-…]` **sólo al último
selector**: el de `principal` pesa cuatro y el mío pesaba tres. No era cuestión
de orden en ningún momento.

Las tres veces la sonda dio verde con el `h1` ya puesto. **Medir no es mirar**,
que es exactamente la lección del A3.

### 3.7 · Dos cosas más que sólo se vieron en la captura

- **«Meta aceptó el mensaje» salía dos veces seguidas**: como titular y como
  texto de la clave. Las pruebas no lo veían porque las dos frases contienen lo
  que se buscaba.
- **El botón de mandar la prueba se estiraba a todo el ancho** de la tarjeta, lo
  que lo hacía parecer una barra.

---

## 4 · Lo que la Fase 7 NO hace, y conviene no prometer

- **No dice «entregado»** (§65), y no es cautela: ARLES no lee buzones, así que
  no puede saberlo. Dice «aceptado por el servidor», que es lo que ocurrió.
- **No hay tasa de apertura ni clics.** Requieren píxel de rastreo y enlaces
  reescritos. ARLES no rastrea a nadie, y eso es la visión del producto.
- **No hay tasa de rebote real.** Sólo los rechazos síncronos del servidor. Los
  asíncronos necesitan leer un buzón → v1.3 (ADR-0009).
- **No hay cifras de campaña de WhatsApp.** El motor sólo recorre
  `Canal::Correo`: de WhatsApp hay envíos de prueba y respuestas, nada más.
- **Las plantillas aprobadas de Meta no tienen pantalla.** La tabla y el
  repositorio existen (6.8b) y ahora también los textos de sus estados, pero no
  hay comando ni pantalla que los lea. Va con la decisión de §5.
- **SPF, DKIM y DMARC** (7.5): sin empezar, al final, por decisión de Dirección.

---

## 5 · Lo que espera a Dirección

1. **El canal de WhatsApp, entero o no.** Hoy sólo manda envíos de prueba. Para
   lanzar una campaña por WhatsApp hace falta que el motor recorra ese canal, y
   con él vuelven `revisar_campana_de_whatsapp` y la pantalla de plantillas
   aprobadas. Está planteado en `FASE-05-PARA-DIRECCION.md`.
2. **El título dentro de la tarjeta.** Las catorce pantallas tienen ya su título
   al mismo tamaño, pero en tres empieza 28 px más a la derecha porque va dentro
   del módulo. Se puede igualar sacándolo a una cabecera propia, como las otras
   once. Es un cambio de diseño y lo decide Dirección.
3. **R-24.** Mientras no se cierre, todo lo de correo sigue probado contra un
   servidor de mentira, y la 7.5 no se puede verificar de verdad.

---

## 6 · Cómo se verificó

```
cargo fmt --all --check                     limpio
cargo clippy --workspace --all-targets      0 advertencias
cargo test --workspace                      todas en verde
npm test                                    187 pruebas
npm run lint · npx vue-tsc --noEmit         limpios
validar.py --fase {0..6,W,7}  CON SONDAS    298 comprobaciones, 0 fallos, 0 omitidas
```

Y lo que ninguna de esas órdenes hace: **se abrió la demostración en Chromium y
se miraron las catorce pantallas**, una tras otra. De ahí salieron §3.4 a §3.7.
