# Qué probar, en qué orden, y qué no se puede todavía

> **Para quién es esto.** Para Dirección, con el instalador delante y una tarde
> por delante. Escrito el **11/11/2026**.
>
> **Qué NO es.** No es la guía de cómo abrir ARLES —eso es
> [`COMO-ABRIR-ARLES.md`](COMO-ABRIR-ARLES.md)— ni la de conectar los
> proveedores —eso es [`GUIA-DE-CONEXION-REAL.md`](GUIA-DE-CONEXION-REAL.md)—.
> Esto es la lista de **qué mirar** y en qué orden, para que ninguna función
> construida se quede sin probar por no saber que estaba.
>
> **De dónde sale.** Del router —diecinueve rutas— y de los comandos
> registrados, no de la memoria. Las listas escritas a mano se quedan atrás, y
> en este proyecto ya ha pasado cinco veces.

---

## 0 · Antes de empezar: las tres cosas que van a pasar y no son defectos

| Qué vas a ver | Por qué |
|---|---|
| **Windows dice «editor desconocido»** | El instalador no está firmado. Falta el certificado (**FIRMA-WIN**), que es un trámite y no código. *Más información → Ejecutar de todas formas* |
| **La letra no es la de la marca** | Va sin Mont mientras **P-01** siga abierto. Los tamaños y pesos sí son los definitivos |
| **En macOS no abre** | Gatekeeper se niega a ejecutar una aplicación sin notarizar, y eso necesita **FIRMA-MAC**. No es un aviso que se pueda saltar |

**Así que esta tanda se prueba en Windows.**

---

## 1 · El recorrido del primer día · una hora

Es el orden en que lo recorre cualquiera, y el único que atraviesa el producto
entero. Si algo falla aquí, falla lo importante.

| # | Qué | Dónde | Qué mirar |
|---|---|---|---|
| 1 | Configurar la empresa | `/inicio/configuracion` | Que al volver a Inicio los pasos del alta cambien de **0 de 6** a **1 de 6** |
| 2 | Cargar contactos | `/contactos/importar` | Que los **choques se enseñen antes** de importar, no después. Que «Teléfono» a secas se reconozca **y avise** |
| 3 | Escribir una plantilla | `/campanas/plantillas` | Que las variables `{{nombre}}` se vean en la lista de las que existen |
| 4 | Dar de alta el remitente | `/remitentes` | Que con un dominio conocido **sólo pida dos campos**. Que «Probar conexión» diga qué falla si falla |
| 5 | Crear la campaña | `/campanas/nueva` | Los **ocho pasos**. En el 4, que la previsualización salga **con los datos de un contacto de verdad**. En el 6, el plazo estimado. En el 7, la comprobación previa y el envío de prueba |
| 6 | Activarla y verla enviar | `/campanas` → `/actividad` | Que Actividad diga **cuántos salieron** y, si no sale nada, **cuál de las seis razones** es, con su cifra |

> **El paso 6 es el que importa.** Es lo único que no se puede verificar desde
> aquí: que un correo de verdad llegue a una bandeja de verdad. Lo demás lo
> cubren 1 344 pruebas.

---

## 2 · Lo que se construyó esta semana y nunca se ha probado a mano

Esto no estaba en la versión del **05/11**, así que es la primera vez que se
puede tocar.

### 2.1 · La baja automática por correo (R-25 y R-30)

| # | Qué hacer | Qué tiene que pasar |
|---|---|---|
| 1 | En `/remitentes`, bajar a «Bajas que llegan por correo» y marcar la casilla | Que proponga el servidor de lectura cambiando `smtp.` por `imap.`, **editable** |
| 2 | Mandarte una campaña a ti mismo | Que el correo lleve el **botón de cancelar suscripción** de Gmail u Outlook arriba, y la frase con la palabra **BAJA** en el pie |
| 3 | Pulsar ese botón, o contestar `BAJA` | — |
| 4 | Pulsar «Revisar el buzón ahora» | Que diga **«1 correo nuevo revisado · 1 baja aplicada»**, pluralizado y sin «(s)» |
| 5 | Mirar `/contactos/supresion` | Que esa dirección esté ahí, con origen **«lo pidió por correo»** |
| 6 | **Esperar quince minutos sin tocar nada** y volver a `/remitentes` | Que debajo de la casilla diga **«Se revisa solo cada 15 minutos. Última vez, hace N minutos»** |
| 7 | Escribir un servidor de lectura **mal** a propósito y esperar | Que diga **«No se pudo revisar»** con el motivo — y no que todo va bien |

> **El 6 y el 7 son los que valen.** El 6 prueba que el automatismo existe; el 7,
> que **no miente cuando falla**, que es lo que distingue un indicador útil de
> uno decorativo.

### 2.2 · Elegir a quién va una campaña (R-27)

| # | Qué hacer | Qué tiene que pasar |
|---|---|---|
| 1 | En `/contactos`, filtrar por texto o por canal | Que el filtro **no busque mientras escribes** —busca al terminar—, porque buscar algo que no está cuesta segundos |
| 2 | Marcar tres contactos y guardarlos como lista | Que pida un nombre y la cree sin tope de cuántos |
| 3 | Crear una campaña y elegir esa lista en el paso 2 | Que el paso 6 diga **3 destinatarios** y no toda la empresa |

### 2.3 · Borrar un borrador (H-22)

Crear una campaña, dejarla en borrador, borrarla. Y después **activar una y
probar a borrarla**: tiene que negarse. Una campaña con historial no se borra
nunca.

---

## 3 · Lo que se puede probar, agrupado por pantalla

Las diecinueve rutas del router. Las que no aparecen arriba:

| Pantalla | Qué mirar |
|---|---|
| `/inicio` | Los seis pasos del alta, y que **ninguno lleve a una pantalla que no existe** |
| `/contactos/derechos` | Los cuatro derechos ARCO: acceso, rectificación, cancelación y oposición |
| `/remitentes/whatsapp` | El alta de un número. **Sin cuenta de Meta no se puede completar** (bloqueo **META**) |
| `/actividad/registro` | Los seis filtros combinables. Que **viajen en la dirección**: al volver atrás, el filtro sigue puesto |
| `/ajustes` | El tema claro y oscuro. Que al cambiarlo **no se reinicie nada** |
| `/ajustes/respaldos` | Exportar con contraseña y sin ella |
| `/ajustes/restaurar` | Restaurar el respaldo que acabas de hacer. Que **diga qué se pierde antes** de que confirmes |
| `/ajustes/archivar` | El preflight del archivado. **No archives de verdad** si no tienes un disco externo a mano |
| `/recuperar` | **No se puede provocar a mano sin romper el llavero.** Se prueba al cerrar R-07 |
| `/catalogo` | El catálogo del sistema de diseño. Es una herramienta nuestra, no una pantalla del producto |

---

## 4 · Lo que NO se puede probar todavía, y por qué

**No son defectos. Si te los encuentras, no hace falta anotarlos.**

Cinco de los siete son bloqueos que esperan a alguien de fuera, y
**los seis impiden entregar** la v1.2.0 — el sexto es **R-07**, la revisión en macOS, que
no estorba a esta tanda porque se prueba en Windows. Están en
[`REGISTRO-DE-REQUISITOS.md`](../10-requisitos/REGISTRO-DE-REQUISITOS.md) con
quién tiene cada uno. Los otros dos son funciones no construidas.

| Qué | Por qué |
|---|---|
| **Una campaña entera por WhatsApp** | El motor la recorre desde el 26/10, pero hace falta una cuenta de Meta con Coexistencia: es el bloqueo **META** y no depende de programar |
| **Que ARLES abra en macOS** | **FIRMA-MAC** |
| **Un instalador sin advertencia** | **FIRMA-WIN** |
| **La tipografía de la marca** | **P-01** |
| **Los textos legales definitivos** | **P-09**. Lo que se ve va marcado como provisional |
| **Editar una campaña ya creada** | No está construido. Hay que crear otra |
| **Un filtro dentro del paso 2** de la campaña | A quién va se elige antes (§2.2). Lo que no hay es un buscador dentro del asistente |
| **Contar las respuestas de WhatsApp** | Necesita el «relay ciego», que no existe |
| **Que el registro se recorte solo** | Hay que pulsarlo en `/actividad/registro`. Ver §5 |

---

## 5 · Dos cosas medidas que conviene saber antes de verlas

**Activar una campaña grande tarda.** 73,5 s con 451 196 destinatarios, y se
empieza a notar hacia los 30 000. Con una lista de prueba no se nota. La ventana
**no se congela**: dice en qué va y se puede cancelar.

**Buscar texto en el registro tarda cuando hay mucho historial.** Medido el
11/11/2026 con 120 000 sucesos: entre 3,5 y 4,5 s. Mientras busca, la pantalla
lo dice y atenúa la tabla. Dirección decidió ese día **mantener el total
exacto** aceptando esa espera, en vez de cambiarlo por un «más de 500» que
costaría 13 ms.

> **Con una base nueva ninguna de las dos se nota**, porque las dos dependen de
> tener muchos datos. Se dicen para que no se lean como una avería el día que
> aparezcan.

---

## 6 · Cómo anotar lo que encuentres

En [`documentacion/06-calidad/BITACORA-DE-PRUEBAS.md`](../06-calidad/BITACORA-DE-PRUEBAS.md),
que tiene su formato. Lo que más sirve, por orden:

1. **Qué esperabas que pasara** y qué pasó.
2. **En qué pantalla** y con qué datos.
3. Si se puede repetir.

Y lo que vale más que un defecto: **algo que se entienda a medias**. Un error
que no dice qué hacer, un botón apagado que no explica por qué, una cifra sin
contexto. Eso no lo caza ninguna prueba y es lo que las trece revisiones a ojo
de este proyecto han encontrado una y otra vez con todo en verde.
