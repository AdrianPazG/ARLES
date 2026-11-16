# Prompt de arranque · para abrir ARLES en una sesión nueva

> **Cómo se usa:** copia todo lo que hay entre las dos líneas de guiones y pégalo
> como primer mensaje en la sesión nueva de Claude Code, con el repositorio
> `AdrianPazG/ARLES` conectado.
>
> Este archivo vive en el repositorio a propósito: si se pierde el mensaje
> original, está aquí.

---

Hola. Vas a continuar el desarrollo de **ARLES RELAY I**, un proyecto con
**nueve fases construidas** —la 8 cerrada entera, con sus seis auditorías— y el
canal de WhatsApp a falta de su última pieza. Vengo de otra sesión y lo que
sigue es todo lo que necesitas para retomarlo sin romper nada.

**Lee esto entero antes de tocar una sola línea de código.**

> **Este archivo decía «cuatro fases y media» hasta el 27/10/2026**, con la Fase
> 8 cerrada. Un documento de arranque que miente sobre el tamaño del proyecto
> hace empezar con la idea equivocada de lo que ya está decidido — así que
> ahora el detalle no vive aquí: vive en
> [`00-ANTES-DE-EMPEZAR/`](../00-ANTES-DE-EMPEZAR/LEEME-PRIMERO.md), que es lo
> que de verdad hay que leer, y en
> [`ESTADO_ACTUAL.md`](ESTADO_ACTUAL.md), que es el único archivo de estado.

---

## 1 · Lo primero que tienes que hacer

En este orden, y sin saltarte ninguno:

1. **Lee `CLAUDE.md`** (raíz del repositorio). Son las reglas del proyecto. No
   son preferencias: casi todas están escritas porque algo salió mal antes.
2. **Lee `documentacion/07-entrega/ESTADO_ACTUAL.md`.** Es el mapa de dónde está
   el trabajo hoy, qué decisiones ya están tomadas y qué es lo siguiente.
3. **Comprueba que el proyecto está sano** antes de cambiar nada:

   ```bash
   for f in 0 1 2 3 4 5; do python3 herramientas/validar/validar.py --fase $f --rapido; done
   cargo test --workspace
   cd app && npm test
   ```

   Lo esperado el 24/09/2026: las seis fases en verde (5, 45, 33, 71, 37 y 22
   comprobaciones), **677 pruebas de Rust** y **110 del frontend**.

4. **Dime lo que has encontrado** y confirma que las cifras coinciden. Si algo no
   cuadra, quiero saberlo antes de que empieces, no después.

**No empieces a programar hasta que me hayas respondido eso.**

---

## 2 · Qué es ARLES, en dos párrafos

Una **aplicación de escritorio para Windows y macOS** que permite a una empresa
ejecutar campañas de correo electrónico de forma controlada, auditable y
honesta, usando **sus propias cuentas de correo y sus propios límites**. La
desarrolla TELEMETRY INSIGHT, que es mi empresa, y la v1.2.0 es despliegue
interno nuestro, no producto comercial todavía.

Lo que define al producto **no es enviar correo** —eso lo hace cualquier cosa—
sino el control sobre el envío: límites que fija el cliente, una simulación que
dice cuánto va a tardar la campaña **antes** de activarla, un motor que respeta
esos límites incluso cuando el equipo se suspende y despierta, idempotencia para
que nadie reciba dos veces el mismo correo, y métricas que no mienten.

**El bucle completo del producto:** configurar empresa → conectar cuenta de
correo → importar contactos → organizar audiencia → crear campaña → redactar y
personalizar → fijar límites → simular → enviar prueba → validar (preflight) →
ejecutar → pausar / reanudar / detener.

---

## 3 · Cómo trabajamos · lo que no se negocia

### Idioma

**Todo en español.** Respuestas, planificación, comentarios del código,
documentación y mensajes de commit. El equipo es mexicano y el producto también.

### Git

- **Rama de trabajo: `claude/loving-faraday-5chmmy`.** Nunca empujes a otra sin
  que yo te lo pida con esas palabras.
- **Nunca abras un pull request** salvo que yo lo pida explícitamente.
- Cada commit termina con la línea de coautoría y la de sesión. **La línea de
  sesión es la de TU sesión** — no la copies de un commit anterior: apuntaría a
  una conversación que no explica ese commit.

### La regla de oro del proyecto

> **Una comprobación que nunca se ha visto fallar no está verificada.**

Cada garantía que afirmes —en una prueba, en el validador, en una sonda— la
rompes a propósito para ver si la comprobación la caza. Si no la caza, la
comprobación no vale y se arregla antes de seguir. Se deja constancia en el
comentario de la prueba.

**Esto no es ceremonia.** En este proyecto ya ha encontrado, entre otras cosas:
una prueba de paginación que devolvía un resultado pidiera lo que pidiera, una
sonda que buscaba un texto con un formato que la pantalla no usa nunca, y —hace
dos días— que una función de la biblioteca de correo que creíamos usar para
detectar fallos de cifrado **no funciona**, y su rama de código no se ejecutaba
jamás.

### Cómo me hablas a mí

**No soy programador.** Explícame las cosas en mis términos, sin esconder lo
técnico cuando importa.

- **Al cuestionar una decisión**, usa esta estructura, no una opinión suelta:
  `Problema → Riesgo → Impacto → Alternativa → Trade-off → Recomendación`.
- **Al informar**, di lo que hay: si algo falla, enséñame la salida; si te
  saltaste un paso, dilo. Las cifras son cifras, no adjetivos: «2 de 6», «34 %»,
  nunca «casi listo».
- **Paso a paso.** Un tramo, lo verificas, lo empujas, me lo explicas, y me
  preguntas antes del siguiente. Esto lo he pedido muchas veces.

---

## 4 · Dónde está el trabajo hoy

**58 % de la v1.2.0 construido y verificado**, a 24/09/2026.

| Fase | Estado |
|---|---|
| F0 Discovery · F1 Cimientos · F2 Design System · F3 Empresa y contactos | ✅ cerradas |
| F3W Canales (WhatsApp, esquema) | 🟡 55 % |
| **F4 Motor de ejecución** | ✅ cerrada |
| **F5 Proveedores de envío** | 🟡 **en curso — 5.1, 5.2 y 5.3 cerradas** |
| F6 a F9 | ⬜ sin empezar |

### Lo siguiente, concreto: **entrega 5.4**

**Enganchar el adaptador de correo al motor.** Hoy el bucle del motor usa el
proveedor **simulado** de la Fase 4. Hay que construir el `ProveedorSmtp` a
partir de la cuenta que ya se guarda —leyendo su contraseña del llavero del
sistema— y dárselo al trabajador.

Es el punto donde, por primera vez, una campaña puede mandar un correo de
verdad.

Después vienen **5.5** (el envío de prueba a una dirección propia, antes de
activar una campaña) y **5.6** (cierre de fase: documento, informe para
Dirección y PDF).

---

## 5 · Las tres reglas de la fase en curso

Están escritas más de una vez en el código a propósito, y hay comprobaciones que
fallan si tocas una. **No las «simplifiques».**

1. **No existe ninguna forma de no cifrar.** Ni un valor, ni una casilla, ni una
   opción avanzada. Está en el tipo del núcleo, en el adaptador y en la
   restricción de la base de datos. Si algún día un servidor no cifra, ARLES no
   le habla — no hay plan B.
2. **La contraseña entra y no sale.** Vive en el llavero del sistema; en la base
   sólo hay una referencia que no sirve para entrar a ningún sitio. Ningún
   comando la devuelve, ninguna estructura que viaje a la interfaz tiene un
   campo para ella.
3. **Nada antes del `RCPT TO` puede dar de baja a nadie.** Ése y el acuse final
   son las únicas puertas a una supresión, y una baja no se deshace.

---

## 6 · Trampas conocidas · te vas a encontrar estas

**1 · Corre SIEMPRE las seis fases antes de empujar, no sólo la que tocas.**
En las dos últimas entregas se verificó sólo la fase 5 —que pasaba entera— y se
rompieron cuatro comprobaciones de las fases 2, 3 y 4 sin que nadie lo viera:
seis nombres de estilo inventados (que el navegador resuelve **a nada, sin dar
error**, así que la pantalla se veía mal), la huella de una migración sin
registrar, el simulador de la vista previa desalineado con el código real, y una
comprobación que se cazaba a sí misma. Cuesta un minuto con `--rapido`.

**2 · El disco de la sesión se llena, y el síntoma engaña.**
Compilar este proyecto ocupa decenas de GB. Cuando se agota, el validador falla
en varias fases con pruebas que parecen rotas y que en realidad **no llegaron a
compilar**. Antes de buscar un fallo en el código, mira el final del error: si
dice `No space left on device`, es esto.

```bash
df -h /
rm -rf target/debug/incremental        # suele bastar
cargo clean                            # si no
CARGO_INCREMENTAL=0 cargo test --workspace
```

**3 · Las comprobaciones que buscan texto se cazan a sí mismas.**
Ya ha pasado **tres veces**: una comprobación busca una palabra prohibida en el
código, y la encuentra… en el comentario que explica que esa palabra está
prohibida. Si escribes una comprobación así, quita los comentarios antes de
mirar.

**4 · Lo que no se puede probar desde aquí.**
El entorno de desarrollo **no tiene salida a servidores de correo** (medido: los
tres puertos agotan el tiempo de espera) y **no tiene llavero del sistema**, que
ARLES exige para arrancar. No intentes probar contra Gmail desde ahí, y **no me
pidas credenciales por el chat**: quedan escritas en el historial. Eso se cierra
en mi máquina, y está explicado en `ESTADO_ACTUAL.md` §7 (riesgo **R-24**).

---

## 7 · Lo que NO debes hacer

- ❌ Empujar a una rama distinta de `claude/loving-faraday-5chmmy`.
- ❌ Abrir un pull request sin que yo lo pida.
- ❌ Tocar las carpetas de sólo lectura: `/RECURSOS`,
  `/REFERENCIA_DE_COLOR`, `/CONCEPTOS_DE_DISEÑO`,
  `/REFERENCIAS_VISUALES_DEL_SITIO`, `/TIPOGRAFIA`. No se borran, mueven,
  renombran, convierten, comprimen ni editan.
- ❌ Editar una migración de base de datos ya aplicada.
- ❌ Guardar credenciales en texto plano, JSON, almacenamiento del navegador,
  bitácoras o tablas sin cifrar. Van al llavero, y si el llavero no está, **la
  aplicación no arranca**: nunca se degrada.
- ❌ Poner nombres de modelo de IA en commits, comentarios o cualquier cosa que
  acabe en el repositorio.
- ❌ Dar por bueno algo que no has ejecutado. Si no lo corriste, dilo.

---

## 8 · Dónde está escrito lo demás

`documentacion/00-INDICE.md` es el mapa completo. Lo que más se consulta:

| Para saber | Mira |
|---|---|
| **Dónde está el trabajo y qué sigue** | `07-entrega/ESTADO_ACTUAL.md` |
| Qué es y qué no es ARLES | `01-producto/VISION_Y_ALCANCE.md` |
| Las decisiones que tomé yo, con su porqué | `01-producto/DECISIONES_DE_DIRECCION.md` |
| Por qué algo se construyó así | `03-arquitectura/adr/` (14 decisiones) |
| Tablas, índices e invariantes de la base | `03-arquitectura/MODELO_DE_DATOS.md` |
| Cómo funciona el motor de envío | `03-arquitectura/MOTOR_DE_EJECUCION.md` |
| El adaptador de correo y sus reglas | `03-arquitectura/ABSTRACCION_EMAIL_PROVIDER.md` |
| Superficie de ataque | `04-seguridad/THREAT_MODEL.md` |
| Dónde viven los secretos | `04-seguridad/MODELO_DE_SECRETOS.md` |
| Riesgos abiertos | `02-auditoria/MATRIZ_DE_RIESGOS.md` |
| Fases y dependencias reales | `07-entrega/ROADMAP.md` |
| Lo que pasó en cada entrega | `CHANGELOG.md` (escrito para que lo lea yo) |

---

## 9 · Lo que quiero que me respondas ahora

Antes de escribir código:

1. **Confirma que leíste** `CLAUDE.md` y `ESTADO_ACTUAL.md`.
2. **Dime el resultado real** de las seis fases del validador y de las pruebas, y
   si coincide con las cifras que te di arriba.
3. **Explícame con tus palabras** qué es lo siguiente (5.4) y cómo piensas
   abordarlo, en tramos.
4. **Pregúntame lo que no esté claro.** Prefiero cinco preguntas ahora que una
   entrega en la dirección equivocada.

Cuando me respondas eso, te digo si arrancamos.

---
