# Consulta jurídica · lo que necesitamos de un abogado

> **Para:** el despacho o abogado que contrate TELEMETRY INSIGHT
> **De:** el equipo de ARLES RELAY I
> **Fecha:** 17 de septiembre de 2026 · **Resuelve:** la pregunta abierta P-09

Este documento está escrito para **mandarse tal cual**. No pide una opinión
general sobre protección de datos: pide **ocho respuestas concretas**, y en cada
una dice qué vamos a hacer con ella. Si alguna pregunta no se puede contestar
sin más contexto, lo que falta está en la sección 5.

---

## 1 · Qué es ARLES, en un párrafo

Una aplicación **de escritorio** (Windows y macOS) que ejecuta campañas de
correo electrónico y de WhatsApp. **Todo corre en el equipo del cliente**: la
base de datos está cifrada en su disco, y ARLES no tiene servidores ni recibe
copia de los contactos. Los correos salen de **la propia cuenta del cliente**
(su SMTP o su Gmail), no de una infraestructura nuestra.

Quien usa ARLES —el cliente— importa una lista de contactos que ya tiene, o que
obtiene de un directorio público, y les escribe. **Muchos de esos contactos no
han dado consentimiento previo**: es correo comercial en frío dirigido a
empresas.

**El primer usuario es TELEMETRY INSIGHT**, internamente. La comercialización a
terceros llega en una versión posterior.

---

## 2 · Lo que ya averiguamos, para no hacerle perder tiempo

Esto lo dimos por bueno a partir de fuentes secundarias fiables. **Le pedimos que
lo confirme o lo corrija**, no que lo investigue de cero:

| | Lo que entendemos | Fuente |
|---|---|---|
| **a** | La **LFPDPPP de 2010 fue abrogada**. Rige una **LFPDPPP nueva**, publicada en el DOF el **20 de marzo de 2025**, en vigor desde el **21 de marzo de 2025** | DOF, y análisis de Garrigues, EY e IDC |
| **b** | El **INAI se extinguió**. La autoridad en materia de datos en posesión de particulares es ahora la **Secretaría Anticorrupción y Buen Gobierno** | Decreto constitucional del 20/12/2024 |
| **c** | El **reglamento de la ley nueva sigue sin publicarse** | Análisis de firmas, a la fecha de este documento |
| **d** | El medio de defensa pasa a ser el **juicio de amparo** ante tribunales especializados, en lugar del juicio de nulidad | Análisis de firmas |

=> Si alguno de estos cuatro puntos es incorrecto, **eso ya es una respuesta
valiosa** y cambia lo que escribimos.

---

## 3 · Las ocho preguntas

### Bloque A · Qué norma aplicamos

**P-1 · ¿Sigue vigente el Reglamento de 2011 mientras no se publique el nuevo?**

- *Por qué preguntamos:* el Reglamento de 2011 es el que detalla el contenido
  mínimo del aviso de privacidad y los plazos de respuesta a los derechos ARCO.
  Si sigue aplicando, tenemos una guía concreta; si no, hay que cumplir sólo con
  el texto de la ley, que es más escueto.
- *Qué haremos con la respuesta:* decide **qué citamos** en el aviso de
  privacidad y **qué plazos** programamos en la aplicación.

**P-2 · ¿Hay que registrar algo, notificar algo o inscribirse ante la Secretaría
Anticorrupción y Buen Gobierno antes de operar?**

- *Por qué preguntamos:* con el INAI no había registro previo para particulares.
  Queremos confirmar que sigue sin haberlo.
- *Qué haremos:* si lo hay, es un trámite que hay que iniciar **antes** de la
  primera campaña, no después.

### Bloque B · El correo y el WhatsApp en frío

**P-3 · Datos obtenidos de una fuente de acceso público** —por ejemplo, el
**DENUE del INEGI**, que publica el correo y el teléfono de establecimientos—:
**¿se pueden usar para enviar publicidad comercial sin consentimiento previo?**

- *Por qué preguntamos:* es **el supuesto central del producto**. La ley
  anterior lo permitía en su artículo 10, fracción II. Necesitamos saber si la
  ley nueva mantiene esa excepción, con qué límites, y si cambia algo cuando el
  destinatario es una persona física con actividad empresarial en vez de una
  persona moral.
- *Qué haremos:* si la respuesta es no, o es «sí pero», **cambia el producto**,
  no sólo un texto: habría que exigir prueba de consentimiento antes de permitir
  una campaña.

**P-4 · ¿Qué tiene que decir, palabra por palabra, el aviso de privacidad que
el destinatario recibe cuando sus datos se obtuvieron indirectamente?**

- *Por qué preguntamos:* nosotros no tratamos los datos — los trata el cliente.
  Pero ARLES es quien **redacta el texto que sale en el correo**, y queremos que
  sea correcto por defecto.
- *Qué haremos:* será el texto de plantilla que ARLES propone, editable por el
  cliente. Idealmente nos entrega **el texto redactado**.

**P-5 · ¿El WhatsApp comercial no solicitado tiene requisitos distintos a los
del correo?**

- *Por qué preguntamos:* WhatsApp es un canal más intrusivo y está asociado a
  un número personal. Queremos saber si existe alguna obligación adicional en
  México —de la ley de datos, de PROFECO o de cualquier otra— más allá de las
  reglas contractuales de Meta, que ya conocemos y cumplimos.
- *Qué haremos:* decide si el modo «Prospección Directa» necesita una
  advertencia distinta, o si hay que restringirlo.

**P-6 · ¿Existe obligación de consultar el REPEP** (Registro Público para Evitar
Publicidad) **o algún registro equivalente antes de enviar?**

- *Por qué preguntamos:* el REPEP cubre telefonía. No sabemos si alcanza al
  correo o al WhatsApp, ni si es exigible cuando el destinatario es una empresa.
- *Qué haremos:* si es exigible, **es una funcionalidad nueva**: habría que
  cruzar la lista antes de cada campaña. Eso no está construido y hay que
  planearlo.

### Bloque C · Los derechos del destinatario

**P-7 · Plazos y forma de atender los derechos ARCO** (acceso, rectificación,
cancelación y oposición) **con la ley nueva.**

- *Por qué preguntamos:* los tenemos programados contra los plazos de la ley
  anterior. Si cambiaron, cambia el código.
- *Qué haremos:* ajustar los plazos que la aplicación vigila y avisa.

**P-8 · Cuando alguien pide que borren sus datos, ¿podemos conservar su
dirección en una lista de supresión para no volver a escribirle?**

- *Por qué preguntamos:* hay una tensión real. Para **cumplir** con «no me
  vuelvas a escribir» hay que **conservar** la dirección; si la borramos del
  todo, la siguiente importación la vuelve a meter y se le escribe otra vez.
- *Qué haremos:* está construido así —se conserva la dirección y nada más, sin
  nombre ni empresa—. Necesitamos confirmación de que es la lectura correcta, y
  si hay que documentarlo de alguna forma concreta.

---

## 4 · Qué NO le estamos preguntando

Para acotar el encargo:

- **No** pedimos revisar el contrato de licencia de ARLES con sus clientes. Eso
  es otra consulta y llega con la versión comercial.
- **No** pedimos revisar los términos de Meta ni de Google. Los conocemos y los
  cumplimos; son contractuales, no legales.
- **No** pedimos auditoría de seguridad. La aplicación cifra en reposo y guarda
  las credenciales en el llavero del sistema operativo.

---

## 5 · Lo que le podemos entregar si hace falta

- El **aviso de privacidad actual** en borrador.
- El **texto de la afirmación de origen lícito** que el usuario acepta al
  importar una lista.
- La **estructura de datos** que guardamos de cada contacto y de cada permiso,
  con sus fechas y pruebas.
- Una **demostración** de la aplicación.

---

## 6 · Qué necesitamos de vuelta, y en qué orden

Si hay que priorizar, **P-3 es la que más pesa**: de ella depende si el producto
funciona como está planteado. Las demás cambian textos y plazos.

| Prioridad | Preguntas | Por qué |
|---|---|---|
| **1** | **P-3** | Si la respuesta es no, cambia el producto entero |
| **2** | P-4, P-8 | Son los textos y la lista de supresión: se construyen ya |
| **3** | P-1, P-7 | Plazos y citas normativas |
| **4** | P-2, P-5, P-6 | Trámites y posibles funcionalidades nuevas |
