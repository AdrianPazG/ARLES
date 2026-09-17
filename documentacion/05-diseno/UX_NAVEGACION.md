# UX y navegación

**Proyecto:** ARLES RELAY I · v1.2.0

---

## 1. Crítica de la navegación propuesta

El §23 propone siete secciones. Dos se solapan:

- **MESSAGES** (plantillas y firmas) es un **insumo de las campañas**, no un destino por derecho propio. Nadie abre ARLES para «ir a las plantillas»: va a crear una campaña y necesita una plantilla.
- **ACTIVITY** duplica en buena parte **EMAIL > Deliverability Health**. Ambas responden a «¿cómo va el envío?».

## 2. Propuesta: seis secciones

| Sección | Contiene |
|---|---|
| **INICIO** | Panel: campañas activas, progreso de hoy, próximos envíos, errores, avisos |
| **CAMPAÑAS** | Activas · Programadas · Historial · **Plantillas y firmas** |
| **CONTACTOS** | Todos · Listas · Etiquetas · Importaciones · **Supresiones** |
| **REMITENTES** | Cuentas conectadas · Salud de envío · Límites |
| **ACTIVIDAD** | Ejecución en vivo · Eventos · Errores |
| **AJUSTES** | Empresa · Preferencias · Respaldos · Actualizaciones · Licencia |

> 🔵 **Pendiente de revisión por la logística de los dos canales.**
> [LOGISTICA_DE_CAMPANAS](../01-producto/LOGISTICA_DE_CAMPANAS.md) propone dos
> cambios sobre esta tabla, ninguno decidido todavía: **REMITENTES** pasaría a
> llamarse **CANALES** —ahí viven también los números de WhatsApp, y
> «remitente» ya no los nombra— y aparecería una séptima sección,
> **CONVERSACIONES**, **sólo cuando hay WhatsApp configurado**. Se decide en
> P-13. Mientras tanto, las seis de abajo son las vigentes.

**«REMITENTES» en vez de «EMAIL»** porque nombra lo que el usuario administra ahí —sus cuentas de envío— y no una tecnología. Todo en ARLES es «email»; ese nombre no distingue nada.

**Las supresiones viven en CONTACTOS**, no escondidas en ajustes. Son una decisión sobre personas, y el usuario debe encontrarlas donde están las personas.

---

## 3. INICIO — la pantalla que más se mira

> **Rediseñada el 17/09/2026.** Dirección pidió que dejara de ser una lista de
> configuración y pasara a ser modular, con acciones a mano, y que la primera
> vez guiara a configurar la empresa. Capturas en `imagenes/tema-inicio-*.png`.

Debe responder a tres preguntas en dos segundos: **¿está corriendo? ¿va bien?
¿hay algo que atender?**

### 3.1 Son dos pantallas, no una con variantes

Las dos cosas que pidió Dirección parecen una y son opuestas.

Quien abre ARLES **por primera vez** no necesita un panel: no tiene nada que ver
en él. Necesita **una sola cosa que hacer**, grande y sin competencia. Un panel
de seis módulos vacíos en el primer arranque es la forma más rápida de que
alguien cierre la aplicación sin configurar nada.

Quien ya la tiene configurada necesita lo contrario: estado de un vistazo y
acciones a un clic.

```
PRIMERA VEZ                          YA CONFIGURADA
┌───────────────────────────┐        ┌──────────────────┬──────────┐
│ ARLES RELAY I             │        │ Alta      1 / 6  │ Accesos  │
│ Empieza por aquí          │        │ ...              │ · ...    │
│                           │        │ [Acción]  Ver 6  │ · ...    │
│ ┌───────────────────────┐ │        └──────────────────┴──────────┘
│ │ Configurar tu empresa │ │        ┌─────────────────────────────┐
│ │ [Configurar] 0 de 6   │ │        │ Lo que falta por construir  │
│ └───────────────────────┘ │        └─────────────────────────────┘
└───────────────────────────┘
```

La frontera es `empresa.configurada`, que el núcleo **deriva de los datos**.

### 3.2 La regla que impide que un panel modular parezca roto

=> **Un módulo sólo se dibuja cuando puede decir algo cierto.**

La tentación del panel modular es maquetar hoy las ocho cajas que habrá algún
día y dejarlas esperando datos. El resultado es una pantalla que parece
estropeada y que, peor, **anuncia capacidades que no existen**.

Los módulos de campañas en marcha, salud de los canales y actividad reciente
—los que pide la logística— **no están maquetados**. Aparecerán cuando haya algo
que poner dentro. En su lugar hay un módulo que dice exactamente eso, porque es
la respuesta a «¿por qué mi panel está tan vacío?».

Lo vigila una prueba: con datos ilegibles, Inicio no dibuja ningún módulo.

### 3.3 La configuración se fue a Ajustes

La lista de los seis pasos **vivía en Inicio y se mudó a Ajustes**, que es donde
se configura. En Inicio queda la cifra —«1 de 6»— y la acción siguiente.

**Lo que no se mudó es la razón por la que la lista enseña los seis desde el
primer día.** Con sólo los pasos construidos, alguien la vería completa al
terminar el primero y concluiría que ya puede enviar. Por eso los que aún no
existen siguen apareciendo, con su entrega, y sin ser enlaces.

La prueba que lo vigilaba **se repartió entre las dos pantallas en vez de
borrarse**: lo que se comprueba no es dónde está la lista —eso es composición y
puede volver a cambiar— sino que las dos afirmaciones sigan siendo ciertas.

### 3.4 La rejilla áurea

`--arles-aureo-fr: 1.618fr`. El módulo principal y el secundario reparten el
ancho en 1.618 : 1; partirlo por la mitad haría que pesaran lo mismo, que es lo
contrario de lo que un panel tiene que decir.

!x **Cuidado al usarla:** `fr` **no entra en `calc()`**. `calc(var(--arles-aureo)
* 1fr)` es inválido, CSS descarta la declaración entera sin decir nada y la
rejilla se cae a una columna — con aspecto de decisión de diseño, no de error.
Ya pasó una vez. Por eso hay un token con la unidad puesta.

Por debajo de los **988 px** medidos (`UMBRAL_DE_PLEGADO.md`) la columna
estrecha cae por debajo de su medida legible y la rejilla se apila. Inicio
declara ese suelo en `--arles-medida: 684px` —el ancho por debajo del cual el
panel deja de funcionar—, y de ahí sale el umbral de plegado: 684 + 240 de
barra + 64 de márgenes.

!i **Ese cambio movió el umbral de 984 a 988 px.** Un panel no tiene ancho de
lectura, así que la sonda dejó de medir el `max-width` y pasó a medir el suelo
declarado. Se cambió el criterio porque cambió lo medido: la prueba es que con
el criterio nuevo la sonda **falló** y el número se subió al medido, no al
revés.

### 3.5 Lo que no va aquí

Gráficas decorativas, contadores totales sin contexto («12 450 contactos»),
tarjetas de bienvenida que no se pueden cerrar, y **módulos vacíos**.

**El dato más valioso del panel futuro es «faltan 1 240 · ~25 días».** Combina
estado y consecuencia, y es lo que hace que alguien reconsidere su
configuración.

## 4. Onboarding

Nada de tours emergentes de veinte pasos (§25). Una **lista de verificación persistente**, que vive en **AJUSTES** desde el rediseño de §3.3 —antes estaba en INICIO— y de la que INICIO enseña la cifra y la acción siguiente.

```
Para poder enviar tu primera campaña                 1 de 6
✓ Configurar tu empresa
  Conectar una cuenta remitente        Llega en la entrega 5
  Cargar tus contactos                 Llega en la entrega 3.2
  Escribir una plantilla               Llega en la entrega 6
  Definir una ventana de envío         Llega en la entrega 6
  Crear tu primera campaña             Llega en la entrega 6
```

Cada paso es funcional: al pulsar se va a hacer la cosa, no a leer sobre ella.

**Los pasos no se bloquean entre sí.** Un usuario que quiera importar contactos antes de conectar una cuenta puede hacerlo. La lista sugiere un orden; no impone un embudo.

### 4.1 Dos decisiones de la entrega 3.1

**El estado de cada paso se deriva de los datos, no se guarda.** Lo cómodo sería
un booleano por paso que se marca al terminarlo. Entonces basta que alguien
borre su única cuenta remitente para que la lista siga diciendo, para siempre,
que ese paso está hecho. Aquí cada paso es una consulta —¿hay alguna cuenta?
¿hay algún contacto sin borrar?—, así que **no puede mentir**: si el dato
desaparece, el paso vuelve a estar pendiente. Ver `arles_core::onboarding`.

**Los seis pasos se enseñan desde el primer día, incluidos los que aún no
existen**, cada uno con la entrega del roadmap que lo trae. La alternativa
—enseñar sólo lo construido y que la lista crezca sola— hace que alguien vea
la lista completa al terminar el primer paso y concluya que ya puede enviar.
Un paso que avisa de que llega más adelante es información; un paso ausente es
una promesa implícita de que no hace falta.

---

## 4.2 La barra lateral · fija y plegable (P-11)

> **Rediseñada el 17/09/2026** con los puntos 1, 2, 3 y 7 del brief de
> Dirección. Capturas: `imagenes/tema-plegada-*.png`.

### 4.2.1 La cabecera tiene altura fija, y por eso los iconos ya no saltan

Dirección señaló que «los iconos saltan» al pulsar el botón de plegar. **Era
cierto y estaba medido: 40 px.** El logotipo desaparecía al plegar y arrastraba
hacia arriba todo lo que venía debajo.

```
DESPLEGADA                     PLEGADA
┌────────────────────────┐     ┌──────┐
│ ARLES RELAY I          │     │  A   │  ← misma altura reservada
│                    [«] │     │ [»]  │  ← misma fila, mismo alto
├────────────────────────┤     ├──────┤
│ ⌂  Inicio              │     │  ⌂   │  ← y = 124 px en los dos
```

La corrección es reservar la altura en los dos estados. Es una afirmación
geométrica, así que **se mide**: `sonda:cabecera` falla si la navegación se
desplaza más de un píxel. Probada rompiéndola —devolviendo el `v-if` de la
marca— y devuelve los 40 px originales.

### 4.2.2 Plegada va el isotipo, no el logotipo recortado

«ARLES RELAY» en Mont Black no entra en 64 px, y el §21 prohíbe condensarlo. En
su lugar va **la misma «A» que el sistema operativo enseña en la barra de
tareas**, con su placa y sus colores.

=> Esos dos colores —`--arles-marca-fondo` y `--arles-marca-tinta`— **no se
invierten con el tema**, y son los únicos del sistema que no lo hacen. El icono
de la barra de tareas no cambia cuando cambias el tema de Windows; éste es ese
icono.

!i Primero se probó con la placa en `--arles-surface`, que en tema oscuro **es
el mismo color que la barra lateral**: la placa desaparecía y quedaba una «A»
flotando. Ahora usa el color real de la placa del icono (`#045686`), que
contrasta con la barra en los dos temas.

El nombre accesible no cambia: quien usa lector de pantalla oye «ARLES RELAY»,
no la letra «A» suelta.

### 4.2.3 El botón de plegar vive siempre en el mismo sitio

Antes cambiaba de alineación al plegar —de la derecha al centro— y había que
buscarlo dos veces. Ahora tiene **su propia fila**, pegado al borde donde está
la barra.

### 4.2.4 Los iconos ya no heredan el cuerpo del texto

Estaban a `1em`, es decir **14 px**, que en una barra de 240 px se ve de
juguete y plegada es lo único que hay. Pasan a `--arles-icono-nav: 20px`, igual
en los dos estados.

Las filas llevan además alto explícito: con `padding` y un icono de 20 px medían
36 px desplegadas y 34 plegadas, y la lista se descuadraba al plegar.

### 4.2.5 El numeral «ARLES RELAY I»

Arriba a la izquierda, como pidió Dirección. **El numeral no es parte del
logotipo**: el logotipo dice ARLES RELAY (§21) y el «I» pertenece al nombre
comercial, así que va en una propiedad aparte del componente y con tinta
apagada.

La regla de ADR-0010 que sigue en pie: **nunca adyacente al número de versión.**
«ARLES RELAY I v1.2.0» hace pensar que el «I» es la versión 1. Aquí el numeral
está arriba y la versión en el pie, con toda la navegación en medio.

### 4.2.6 El pie: el logotipo de TELEMETRY, enlazado

Sustituye a «Software desarrollado por TELEMETRY INSIGHT» en texto. El logotipo
y la versión **comparten fila y línea de base**; antes iban apilados y la
versión colgaba sin alinearse con nada, que es lo que Dirección señaló en el
punto 4.2 de su brief.

Abre **telemetrymx.com en el navegador del sistema**, no dentro de la ventana:
una WebView que navega a internet deja de ser una aplicación y pasa a ser un
navegador sin barra de direcciones, donde el usuario no puede saber dónde está.

=> **El comando de Rust no recibe la URL.** Ver `MARCA_TELEMETRY.md` §5.

## 4.3 Proporciones y espacio muerto

> Punto 5 del brief de Dirección, cerrado el 17/09/2026. Capturas a 1600 px:
> `imagenes/tema-ancha-*.png`.

### 4.3.1 El problema no era que sobrara aire, era que estaba a un lado

El contenido se pegaba al borde izquierdo y dejaba el resto de la ventana
vacío: en un monitor de 1920 px, la mitad derecha era hueco. **Aire simétrico
es respiración; aire todo a un lado se lee como una pantalla sin terminar.**

Dos correcciones distintas, porque son dos problemas distintos:

| | |
|---|---|
| **Toda pantalla** | se limita a `--arles-ancho-pagina` (1160 px) y **se centra**. La regla vive en el armazón, no en cada pantalla: una que se olvidara volvería a pegarse al borde |
| **Ajustes** | a partir de 1240 px reparte el ancho entre el formulario y **su contexto** |

### 4.3.2 El aire no se quita estirando: se llena con lo que ya había

Lo fácil habría sido dejar que el formulario creciera hasta llenar la ventana.
Un campo de texto de 1200 px de ancho **es peor de rellenar, no mejor**: el ojo
pierde la relación entre la etiqueta y el campo.

Así que el formulario conserva su medida de 62 ch y lo que sobra pasa a llevar
lo que antes estaba debajo: el aviso de la zona horaria y los seis pasos del
alta. Mismo contenido, dos columnas, cero hueco.

=> La segunda columna aparece **a partir de 1240 px, no en cuanto cabe**. A 988
cabría a duras penas y empujaría el umbral de plegado automático hasta casi el
ancho mínimo de la ventana.

### 4.3.3 Una pantalla puede pedir todo el ancho

`--arles-ancho-pagina: none` sobre la pantalla. Está pensado para lo que
todavía no existe: una tabla de 500 000 filas quiere cada píxel, y limitarla a
1160 sería el error contrario.

### 4.3.4 Lo que sigue vacío, y por qué no se rellena

En Inicio queda espacio **vertical** libre. Es consecuencia directa de la regla
de §3.2: los módulos de campañas, canales y actividad no se maquetan hasta que
tengan algo que decir.

!i Se podría rellenar con una gráfica decorativa o con contadores sin contexto.
Sería mentir sobre lo que la aplicación sabe hacer hoy, y el módulo «Lo que
falta por construir» dice la verdad en el mismo sitio.

---

## 5. Flujo de campaña (§40)

Nueve pasos, en un asistente con pasos visitables hacia atrás y borrador guardado automáticamente.

```
1 Información   → nombre, descripción
2 Audiencia     → listas, etiquetas, filtros  [muestra el conteo tras supresión]
3 Remitente     → cuenta, nombre visible, responder-a
4 Mensaje       → plantilla o redacción, variables, firma
5 Límites       → diario, horario, días y horas    [aviso >50 aquí]
6 Simulación    → duración estimada real
7 Preflight     → validación, bloquea si falla
8 Prueba        → envío obligatorio a dirección propia
9 Activación    → confirmación con resumen
```

### Tres momentos que definen el producto

**Paso 2 — el conteo es honesto.** «1 240 contactos (87 excluidos por supresión)». El número excluido se muestra siempre, aunque sea cero.

**Paso 6 — la simulación en lenguaje llano.**

> Esta campaña enviará **1 240 correos**.
> Con tu límite de 50 diarios, de lunes a viernes de 9:00 a 18:00, terminará alrededor del **jueves 14 de enero de 2027**.

Frecuentemente ésta es la información más valiosa de todo ARLES: es lo que hace que alguien reconsidere **antes** de lanzar.

**Paso 8 — la prueba es obligatoria** (§45). No se puede saltar. Es barata y evita la clase de error más cara del producto.

---

## 6. Aviso de más de 50 diarios (§48)

Aparece en el paso 5, en línea, no como modal.

> ⚠ **Enviar más de 50 correos diarios desde una cuenta aumenta el riesgo.**
> Los proveedores vigilan el volumen y la reputación del dominio. Un volumen alto desde una cuenta sin historial puede provocar rebotes, filtrado a spam o la suspensión de la cuenta.
>
> ARLES no bloquea este límite: la decisión es tuya.
>
> ☐ Entiendo el riesgo y quiero continuar con 120 diarios.

**No se bloquea** (§154: informar, no dictar). Se registra la aceptación en `audit_log`.

Y la interfaz **no rota remitentes** para ayudar a sortearlo (T-1, ADR-0008). Si el usuario pregunta por qué, la respuesta está en el propio texto de la pausa.

---

## 7. Ejecución — pausar, reanudar, detener

Tres acciones **visualmente distintas** (§62). Nunca un solo botón que cambia de significado.

| Acción | Aspecto | Confirmación |
|---|---|---|
| Pausar | Secundario | No |
| Reanudar | Primario | No |
| **Detener** | **Peligro** | **Sí, con número concreto** |

> **¿Detener «Clientes Q1»?**
> **1 240 mensajes quedarán sin enviar** y no se podrán reanudar. Tendrás que crear una campaña nueva.
> Los 87 ya aceptados por el proveedor **no se pueden recuperar**.
> `[Cancelar]` `[Detener campaña]`

Un número concreto en vez de «¿estás seguro?».

**Parada de emergencia:** acción global permanentemente accesible desde la barra de estado. Detiene **todo** envío nuevo de inmediato.

### Cerrar la ventana no detiene nada

El motor es independiente (§51). Al cerrar con una campaña activa:

> **ARLES seguirá enviando en segundo plano.**
> «Clientes Q1» continuará su ejecución. Encontrarás ARLES en la bandeja del sistema.
> ☐ No volver a mostrar

No saber si una campaña sigue corriendo es exactamente la clase de ambigüedad que erosiona la confianza en un producto que maneja envíos reales.

---

## 8. Importación (§34)

```
Archivo → Lectura → Detección → Mapeo → Validación → Vista previa → Deduplicación → Importar
```

**El mapeo se presenta ya resuelto**, con la detección automática aplicada y editable. No se pide al usuario que empiece de cero.

**La vista previa muestra filas reales**, no un resumen abstracto. Y muestra explícitamente lo que **no** se va a importar y por qué.

```
1 240 filas leídas
  ✓ 1 087 se importarán
  ⊘   87 duplicados (ya existen)
  ⊘   54 inválidos (correo mal formado)
  ⊘   12 suprimidos (no se reactivarán)
```

Los 12 suprimidos merecen una nota: se importan como contactos, pero **la supresión permanece** (§39). Decirlo evita la sorpresa de que alguien crea haber «recuperado» un contacto.

**Afirmación de origen** antes de confirmar (riesgo R-13, `04-seguridad/PRIVACIDAD_LFPDPPP.md`).

---

## 9. Teclado (§100)

| Atajo | Acción |
|---|---|
| `Ctrl/Cmd + K` | Paleta de comandos |
| `Ctrl/Cmd + 1…6` | Ir a sección |
| `Ctrl/Cmd + N` | Nueva campaña |
| `Ctrl/Cmd + F` | Buscar en la vista actual |
| `Esc` | Cerrar modal, cancelar edición |
| `↑ ↓` | Navegar filas |
| `Espacio` | Seleccionar fila |
| `Mayús + ↑↓` | Selección por rango |

Todo flujo debe completarse sin ratón. Se verifica en la auditoría de accesibilidad (§140-D).

---

## 10. Densidad y pantalla

De 1366×768 a 4K (§22). Ventana mínima **1120 × 720**.

Preferencia de densidad de tabla —compacta 36 px, cómoda 44 px— en AJUSTES.

**Nada de patrones móviles** (§4): sin hamburguesa, sin gestos, sin pestañas inferiores, sin reorganización adaptable. Por debajo de la ventana mínima se recorta, no se reorganiza.

---

## 11. Lo que deliberadamente no se hace

| No se hace | Por qué |
|---|---|
| Tour emergente al abrir | §25 |
| Notificaciones de escritorio por cada envío | Cientos al día. Sólo errores y fin de campaña |
| Barra de progreso del total de la campaña en la barra de título | Una campaña de 100 días no tiene «progreso» útil minuto a minuto |
| Confirmación al pausar | Es reversible. Confirmar lo reversible entrena a ignorar las confirmaciones |
| Guardar explícito en borradores | Se guarda solo. El guardado explícito es una oportunidad de perder trabajo |
| Números en insignias de navegación | Ruido permanente. Sólo lo accionable, y en INICIO |
