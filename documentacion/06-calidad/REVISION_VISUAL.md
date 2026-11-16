# Revisión visual en Windows y macOS

**Para quien va a mirar la aplicación** · No hace falta saber programar ni instalar herramientas de desarrollo.

> **Qué se cierra con esto.** La Fase 2 quedó con cuatro cosas sin verificar
> porque el servidor donde se construye no tiene pantalla: la ventana nativa,
> el motor de macOS, el escalado de Windows y el lector de pantalla. **Esta
> revisión las cierra las cuatro.** Es la puerta de la entrega 3.1.

---

## 1. Antes de empezar

| Necesitas | Por qué |
|---|---|
| Un equipo con **Windows 10 u 11** | Es el motor WebView2, que no es el mismo que usamos para construir |
| Un **Mac** con macOS 11 o posterior | Es WKWebView, el motor de Safari. Dibuja distinto, y ése es el riesgo R-07 |
| Acceso al repositorio en GitHub | Para descargar los instaladores |

**Tiempo estimado:** unos 40 minutos por sistema, sin prisa.

**No hace falta** instalar Rust, Node, ni ninguna herramienta de desarrollo.
Los instaladores se compilan solos en GitHub.

---

## 2. Conseguir los instaladores

### Atajo: ya están compilados

**No hace falta que compiles nada.** Los dos instaladores **de la entrega 3.1**
están listos aquí:

👉 **https://github.com/AdrianPazG/ARLES/actions/runs/35004815794**

Baja hasta **«Artifacts»**, al final de la página, y descarga el de tu sistema.
Caducan el **29 de septiembre de 2026**; pasada esa fecha hay que volver a
compilar con el procedimiento de abajo.

| Artefacto | Qué lleva dentro | Tamaño |
|---|---|---|
| `arles-revision-windows` | un `.exe` instalador | 4,4 MB |
| `arles-revision-macos` | un `.dmg` **universal** (Intel y Apple Silicon) | 11,2 MB |

GitHub los entrega dentro de un `.zip`: descomprímelo primero.

> **Si ves dos ejecuciones de «Revisión visual» del 15 de septiembre, usa la
> segunda** (la de arriba, la número 5). La primera se lanzó antes de mirar las
> capturas, y mirarlas dejó dos arreglos más.
>
> Estos instaladores corresponden al commit `db4d722`. El único cambio
> posterior —que Inicio enseñe el error en vez de la lista cuando no puede leer
> los datos— **no se ve en una revisión normal**: sólo aparece si la base de
> datos está corrupta.

---

### Compilar una versión nueva

Sólo hace falta cuando haya cambios que quieras ver, o si los artefactos de
arriba caducaron.

> **Si te dijeron «entra a Actions y pulsa Run workflow»: eso no funciona, y
> era un error mío.** Ese botón solo aparece cuando el flujo está en la rama
> por defecto del repositorio, y hoy `main` está vacía. Lo de abajo sí
> funciona, desde el navegador y sin consola.

1. Entra al repositorio en GitHub y cambia a la rama
   **`claude/loving-faraday-5chmmy`** (el desplegable que pone `main`).
2. Abre el archivo **`.github/revision-visual/PEDIDO.md`**.
3. Pulsa el **lápiz** (✏️) de arriba a la derecha.
4. Al final del archivo, añade una línea con la fecha. Da igual qué diga: lo
   que dispara la compilación es que el archivo cambie.
5. Botón verde **«Commit changes…»** → **«Commit changes»**.
6. Ve a la pestaña **Actions**. Ahora sí verás **«Revisión visual»** en marcha.
7. Espera. **macOS tarda unos 20 minutos y Windows unos 22**, medido en la
   ejecución real; compilan a la vez. El de macOS sale universal.
8. Cuando termine, entra a la ejecución y baja hasta **«Artifacts»**.

Si una de las dos plataformas falla, la otra se sube igual. Mándame el enlace
de la ejecución y lo miro.

**Si «Artifacts» sale vacío, es un fallo y hay que decírmelo**: la ejecución
comprueba que el instalador existe antes de subirlo, así que un artefacto
vacío no debería poder ocurrir.

---

## 3. Abrirlos: las advertencias son normales

**Estas compilaciones no están firmadas.** Los certificados llegan en la Fase 9
(riesgo R-14). Los dos sistemas van a protestar, y hay que saltárselo a
propósito:

### Windows

Al abrir el `.exe`, aparece una pantalla azul de SmartScreen:

> *Windows protegió su PC*

Pulsa **«Más información»** y después **«Ejecutar de todas formas»**.

### macOS

Al abrir el `.dmg` y arrastrar la aplicación, macOS dirá que no puede
verificar el desarrollador.

**Clic derecho sobre la aplicación → «Abrir»** → confirmar. Con doble clic
normal no deja; con clic derecho sí. Sólo hace falta la primera vez.

> Si algo de esto te incomoda, es la reacción correcta: es exactamente lo que
> un cliente sentiría. Por eso los certificados están en el roadmap.

---

## 4. Qué mirar, y qué capturar

La aplicación abre en **Inicio**. En la barra lateral, **abajo del todo y con
borde punteado**, hay un enlace: **«Catálogo del sistema»**. Ahí están todas
las piezas.

Ese enlace **sólo aparece en esta compilación de revisión.** En la que se
instalaría en casa de un cliente no se pinta, y el catálogo **no va dentro del
programa** — lo comprueba una verificación automática que inspecciona el
paquete compilado, no el código.

### 4.0 Lo nuevo de la entrega 3.1 · **empieza por aquí**

Esta compilación trae **las dos primeras pantallas reales del producto** y la
barra lateral que pediste. Es lo que no se ha visto nunca en una ventana de
verdad, así que es lo primero que conviene mirar.

#### Inicio · la lista de alta

Abre directamente ahí. Debe decir **«Para poder enviar tu primera campaña»** y
listar **seis pasos**, con «0 de 6» a la derecha.

| Comprueba | Qué debe pasar |
|---|---|
| El primer paso, «Configurar tu empresa» | Es un **enlace**: al pulsarlo lleva a Ajustes |
| Los otros cinco | Llevan una etiqueta gris: «Llega en la entrega 5», «3.2», «6»… y **no** son enlaces |
| El icono de la izquierda de cada paso | Un reloj gris en los pendientes. Estado con icono **y** texto, nunca sólo color |

Que los cinco últimos no se puedan pulsar **es lo correcto**: esas pantallas
todavía no existen, y la lista lo dice en vez de llevarte a una pantalla vacía.

📸 **Captura A:** Inicio recién abierto.

#### Ajustes · la configuración de empresa

| Comprueba | Qué debe pasar |
|---|---|
| Pulsa **«Guardar»** con todo vacío | Deben marcarse **dos campos en rojo, con su mensaje debajo**, y salir un aviso arriba. **No** debe quedarse sin hacer nada |
| Escribe un correo mal, p. ej. `hola` | Debe decir «Revisa que el correo tenga la forma nombre@dominio.com» |
| Rellena nombre y correo bien, y guarda | Debe salir **«Configuración guardada»** |
| Empieza a escribir otra vez en cualquier campo | El «Configuración guardada» debe **desaparecer** |
| Vuelve a **Inicio** | Ahora debe decir **«1 de 6»** y el primer paso, hecho |
| Cierra ARLES y ábrelo otra vez | Tus datos deben seguir ahí |
| El desplegable de **zona horaria** | Doce opciones, todas de México más UTC |

> **El caso del correo mal escrito es el importante.** Justo ese error hacía que
> la pantalla dejara de dibujarse, en silencio. Está corregido; esto es
> comprobarlo en el motor real.

📸 **Captura B:** el formulario con los campos en rojo.
📸 **Captura C:** después de guardar bien.

#### La barra lateral · lo que pediste (P-11)

| Comprueba | Qué debe pasar |
|---|---|
| El botón de los **dos galones** `«` arriba de la barra | Pliega la barra a sólo iconos |
| Estando plegada | Se sigue viendo en qué sección estás; al pasar el ratón por un icono sale su nombre |
| **Cierra ARLES y vuelve a abrirlo** | Debe abrir **como la dejaste** |
| Desplaza una pantalla larga | La barra **no se mueve**; sólo se desplaza el contenido |
| Pon el escalado de Windows al **200 %** | La barra debe plegarse **sola**, y el botón queda apagado explicando por qué |

📸 **Captura D:** la barra plegada.
📸 **Captura E:** a 200 % de escala.

---

### 4.1 La ventana nativa · **las dos plataformas**

Lo primero es lo más simple y nunca se ha visto: **que la ventana abra bien**.

| Comprueba | Qué buscar |
|---|---|
| La ventana aparece centrada y con su barra de título | Que diga «ARLES RELAY» |
| El icono de la aplicación | En la barra de tareas (Windows) o el Dock (Mac) |
| Arrastrar el borde para hacerla pequeña | **No debe poder bajar de 1120 × 720.** Si baja más, es un fallo |
| Maximizar | Que el contenido se reparta, no que se quede en una esquina |

📸 **Captura 1:** la ventana recién abierta, **con la barra de título visible**.
No recortes: la barra es parte de lo que hay que ver.

📸 **Captura 2:** la ventana maximizada.

### 4.2 Las piezas · **las dos plataformas**

En el catálogo hay cuatro pestañas. Recórrelas.

| Pestaña | Qué mirar con atención |
|---|---|
| **Primitivas** | Los botones: ¿el amarillo se lee bien? ¿el deshabilitado se distingue del normal **y aún se puede leer**? |
| **Los cuatro estados** | ¿El bloque de «Cargando» respira suavemente, sin parpadear? |
| **Tabla virtualizada** | Desplázate rápido. ¿Va fluido o da tirones? |
| **Tipografía** | ¿La letra se ve igual de gruesa que en la otra plataforma? |

📸 **Captura 3, 4, 5 y 6:** una por pestaña, **pantalla completa**.

**Lo que más me interesa que mires en el Mac**, porque es donde esperamos
diferencias:

- El **desplegable «Proveedor»**: macOS dibuja las listas a su manera. Es una
  divergencia esperada, no un fallo — pero quiero verla.
- El **texto claro sobre fondo oscuro**: macOS lo adelgaza. Si se ve
  notablemente más fino que en Windows, hay que ajustarlo.
- Las **sombras** del modal y del menú.

### 4.3 Escalado de Windows · **sólo Windows**

Esto es el §22 y es donde más se rompen las interfaces.

1. **Cierra ARLES.**
2. Windows: **Configuración → Sistema → Pantalla → Escala**.
3. Ponlo al **125 %**. Windows puede pedir cerrar sesión: hazlo.
4. Abre ARLES. Recorre las cuatro pestañas.
5. Repite con **150 %** y con **200 %**.
6. Devuelve la escala a lo que tenías.

**Qué buscar:** texto cortado, botones que se salen de su caja, dos cosas
encima de la otra, o barras de desplazamiento que aparecen donde no había.

📸 **Captura 7, 8 y 9:** la pestaña «Primitivas» a 125 %, 150 % y 200 %.

### 4.4 El lector de pantalla

Aquí no hay que ser experto. **Basta con escuchar y decirme si lo que dice
tiene sentido.**

#### Windows — NVDA

1. Descarga **NVDA** de `nvaccess.org` — es gratuito y de código abierto.
   Instálalo, o usa la versión portable si prefieres no instalar.
2. Arráncalo. Empezará a leer en voz alta lo que tengas enfocado.
3. Abre ARLES y **navega sólo con el tabulador**. Nada de ratón.
4. Escucha.

Para apagarlo: `Insert + Q`.

#### Mac — VoiceOver

Ya viene incluido. **`Cmd + F5`** lo enciende y lo apaga.

Abre ARLES y recorre con el tabulador.

#### Qué escuchar, en las dos

| Al llegar a | Debe decir algo como |
|---|---|
| El logotipo | *«ARLES RELAY, imagen»* — una sola cosa, no «ARLES» y «RELAY» por separado |
| Un botón | Su texto y la palabra «botón» |
| El campo con error | El texto del error **al entrar**, no sólo al salir |
| Una pestaña | *«pestaña, seleccionada»* o *«2 de 4»* |
| La tabla | **«fila 40 de 5001»** o parecido — el total real, no lo que se ve |

🎙️ **En vez de captura:** graba el audio o la pantalla mientras recorres, o
apúntame en una lista lo que oyes mal. Cualquiera de las dos me sirve.

**Lo que hay que reportar**, aunque parezca menor:

- Algo que **no dice nada** al llegar.
- Algo que dice **«botón botón»** o repite.
- Un sitio donde **te pierdes** y no sabes dónde estás.
- La tabla anunciando **20 filas** en vez de las 5 001 reales.

---

## 5. Cómo mandármelo

Con esto me vale:

1. **Las nueve capturas** (seis de Mac, nueve de Windows si hiciste el escalado).
2. **Qué versión** de Windows y de macOS.
3. **Cualquier cosa que te haya chirriado**, aunque no sepas explicar por qué.
   *«Se ve raro»* es información útil; yo busco el motivo.
4. Si algo **no abrió o se cerró solo**, dímelo con lo que hacías en ese momento.

> **No hace falta que esté bien.** Al contrario: si todo sale perfecto,
> significa que esta revisión no encontró nada, y la experiencia de las dos
> fases anteriores dice que eso es improbable. Lo que encuentres ahora es lo
> que no vamos a arrastrar durante siete pantallas más.

---

## 6. Lo que yo ya comprobé, para que no lo repitas

Para que sepas dónde **no** hace falta que gastes tiempo. Esto está medido en
un navegador con el mismo motor que Windows, y automatizado:

| Ya verificado | Cómo |
|---|---|
| Contraste de todos los colores | Calculado, no a ojo. Verificado en cada compilación |
| La tabla virtualiza de verdad | 18 nodos para 5 000 filas — se midió porque **estaba mal** |
| El foco no se pierde ni se escapa del modal | Sonda de teclado automatizada |
| Ninguna parada de tabulación sin su anillo | 15 paradas, todas con señal visible |
| La política de seguridad no necesita relajarse | Recorriendo la aplicación bajo la política real |

**Lo que sólo tú puedes ver** es precisamente lo de las secciones 4.1 a 4.4:
si en un sistema operativo de verdad, con su motor de verdad, esto se ve y se
oye como debe.
