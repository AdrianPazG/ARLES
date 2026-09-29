# WhatsApp · lo que hay, lo que falta y qué se puede enseñar

**Para Adrián Paz · 29 de septiembre de 2026**

---

## Lo primero: apareció un hueco grande, y lo cerré

Preparando la guía que me pediste fui a buscar qué botón hay que pulsar para
mandar un WhatsApp de prueba. **No existía.**

El canal entero —seis entregas, setenta pruebas en verde— no podía mandar un
solo mensaje. La pieza que habla con Meta estaba construida y probada, pero
nadie la llamaba: el programa principal **ni siquiera la tenía enlazada**.

**Ya está resuelto.** Lo construí el mismo día (entrega W.8): el cable, el
comando de envío de prueba, y una prueba que recorre el camino entero y
comprueba que el mensaje llegó al otro lado — no solo que el botón dijera que
sí.

**Hoy ARLES manda un correo real y un WhatsApp real.** La demostración que
querías sale.

Lo que todavía no sale es una **campaña entera** por WhatsApp: el motor que
reparte los envíos con su ritmo y su horario sigue recorriendo solo el correo.
Para enseñar el producto no estorba; para usarlo con clientes, sí hace falta.

---

## Por qué ninguna prueba lo detectó

Es la parte que más conviene entender, porque va a volver a pasar.

Las pruebas del adaptador de WhatsApp pasan todas, y son buenas: hablan con un
servidor que imita a Meta, incluidos sus errores. El problema es que prueban que
**la pieza funciona**, no que **alguien la use**.

Es la segunda vez en dos fases: en la Fase 6 pasó igual con el editor de
plantillas — los comandos existen, probados, y ninguna pantalla los llama.

Un programa puede tener todas sus piezas correctas y no funcionar, porque nadie
las conectó.

**Ya no depende de que alguien se pregunte «¿y esto quién lo llama?».** Añadí
una comprobación al validador que busca justo eso: piezas construidas que nadie
usa. La escribí antes de arreglar el hueco, para verla marcarlo en rojo — y
marcó exactamente el adaptador de WhatsApp. Ahora está en verde porque el hueco
se cerró, no porque la comprobación se ablandara.

---

## Lo que sí quedó construido

**Seis entregas, 70 pruebas.** Lo que más vale la pena enseñar:

**Las dos cifras que en correo no existen.** Antes de activar una campaña de
WhatsApp, ARLES enseña esto:

```
    847  contactos en la audiencia
  −  12  con permiso registrado
  ─────
   835  recibirían el mensaje sin haberlo pedido
```

Y no lo bloquea. Un número dado de alta para prospección directa puede escribir
en frío — para eso se dio de alta así. Lo que ARLES impide es hacerlo **sin
darse cuenta**: la cifra está siempre a la vista y activar pide escribir una
palabra.

**El permiso se lee bien.** Quien dio permiso en enero y lo retiró en marzo no
tiene permiso. Parece obvio; una consulta escrita con prisa habría dicho que sí,
porque la entrada de enero sigue ahí — las retiradas se añaden, no borran.

**Las dos confirmaciones cuestan un gesto cada una**, a propósito. No son
casillas del formulario de alta, porque las casillas se marcan con el resto de
los campos sin leerlas. Son la constancia de que a alguien se le avisó.

**Lo que dice la gente no se guarda.** Cuando alguien contesta, ARLES cuenta que
contestó y tira el texto en la misma función que lo lee. No viaja a la base ni
aparece en ningún informe de error.

---

## Cómo hacer la demostración

Escribí la guía completa en
[`GUIA-DE-CONEXION-REAL.md`](../07-entrega/GUIA-DE-CONEXION-REAL.md). El resumen:

**Correo — sale entero hoy.**
1. Crea una cuenta de Gmail desechable.
2. Activa la verificación en dos pasos, genera una **contraseña de aplicación**
   (dieciséis letras). Sin el paso 2, la opción ni aparece — es donde más gente
   se atasca.
3. En ARLES: Remitentes › Añadir cuenta. El servidor y el puerto se rellenan
   solos.
4. «Probar conexión» responde en dos segundos, y el asistente manda un correo de
   prueba real en su paso 8.

**WhatsApp — pide los accesos igual, tardan.**
1. Crea una app de tipo Negocios en Meta for Developers y añádele WhatsApp.
2. Meta te da un **número de prueba gratis** y los dos identificadores.
3. Genera un **token permanente** (usuario de sistema, caducidad «nunca»). El
   token fácil que Meta ofrece de entrada **caduca en menos de 24 horas** — a
   media presentación deja de funcionar.
4. **Verifica tu móvil como destinatario**, desde el panel de Meta. Esto es lo
   que más tarda porque depende de que llegue un código, y el número de prueba
   solo escribe a números verificados (hasta cinco).

Un detalle a favor: el número de prueba viene con una plantilla **ya aprobada**
por Meta, `hello_world`. Como ARLES todavía no tiene pantalla para escribir
plantillas, esa nos sirve para demostrar que el mensaje sale y llega.

**Esto se hace en tu equipo, no desde donde yo trabajo.** La red de mi sesión
bloquea los puertos de correo —medido— y ARLES exige el llavero del sistema, que
un servidor Linux sin escritorio no tiene. Es el bloqueo R-24 y por eso sigue
abierto.

---

## Lo que queda pendiente, en orden de lo que desbloquea

| | Qué es | Qué desbloquea |
|---|---|---|
| ~~**W.8**~~ | ~~Enchufar el adaptador y añadir el envío de prueba~~ | ✅ **hecho el 29/09/2026** |
| **W.9** | Que el motor recorra también WhatsApp | Lanzar una campaña entera por WhatsApp, no solo una prueba |
| **6.8** | El editor de plantillas y firmas | Que una campaña se cree entera sin tocar la base |
| **Relay** | El servicio que recibe los avisos de Meta | Entregas, respuestas, fallos y bloqueos |
| **Fase 7** | Actividad: las dos pestañas, la línea de tiempo | Ver una campaña en marcha |

Mi recomendación ahora es **6.8, el editor de plantillas**: es lo que separa
«se puede enseñar» de «se puede usar», y arrastra el mismo problema en los dos
canales.

---

## Lo que no se ha probado

**Nada de WhatsApp se ha probado contra Meta de verdad.** Todo está probado
contra un servidor que imita a Cloud API, con sus códigos de error incluidos.
Eso encuentra muchísimo, pero no encuentra lo que solo aparece hablando con el
sistema real.

Sigue dependiendo de Operaciones conseguir una cuenta con Coexistencia. Con el
número de prueba de Meta se puede avanzar bastante sin ella.

---

## Verificación, con cifras

```
853 pruebas de Rust          0 fallan
139 pruebas de interfaz      0 fallan
7 fases + el canal W         0 fallan
```

Quince garantías rotas a propósito en este canal, para comprobar que las
pruebas las cazan. Todas fallaron con el mensaje correcto antes de darlas por
buenas.

Y tres fallos que esta fase encontró y que no eran suyos: una función imposible
de llamar bien, dos textos que el programa pedía y nadie había escrito, y seis
nombres de color y espaciado inventados que el navegador ignora sin avisar.
