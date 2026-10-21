# Qué hacer para que el correo no caiga en no deseados

**21/10/2026.** Escrito a partir de las cabeceras reales del correo que Outlook
mandó a la carpeta de no deseados, no de suposiciones.

---

## 1 · Qué dijo Outlook, exactamente

Dirección mandó el «Mostrar original» del correo. Esto es lo que contestó el
servidor de Microsoft:

```text
spf=pass          la IP que envió está autorizada
dkim=pass         la firma del mensaje verifica
dmarc=pass        el dominio del remitente cuadra con la firma
compauth=pass     reason=100 — la autenticación compuesta, perfecta
BCL:0             no lo considera envío masivo problemático

X-MS-Exchange-Organization-SCL: 5     ← esto es lo que lo manda a Junk
RF:JunkEmail
```

**Las cuatro comprobaciones de identidad pasan.** Eso significa dos cosas, y la
segunda importa más que la primera:

1. El correo que ARLES construye está bien hecho.
2. **SPF, DKIM y DMARC no son el problema**, así que lo que el ROADMAP tenía
   apuntado como la solución (la entrega 7.5) no iba a resolver esto.

Lo que lo manda a la carpeta es el `SCL: 5` — Spam Confidence Level. Microsoft
lo calcula del **contenido y de la reputación del remitente**, no de la
autenticación.

---

## 2 · Por qué un correo que pasa todas las comprobaciones sigue cayendo

Tres cosas, por orden de peso:

### a) Una dirección de Gmail firmando como marca

El correo sale así:

```text
From: ARLES <pruebas.telemetry@gmail.com>
```

Un nombre de empresa delante de una dirección personal de Gmail es, para un
filtro, **el patrón exacto de un intento de suplantación**. Da igual que en
este caso sea legítimo: el filtro no distingue, y es la señal que más pesa.

Esto no se arregla con código. Se arregla mandando desde una dirección del
dominio propio.

### b) La cuenta no tiene historial

Una cuenta que empieza a mandar correo no tiene reputación. Se construye con
semanas de envío normal y poco volumen. No hay atajo.

### c) Un primer correo sin relación previa

Quien lo recibe nunca ha escrito a ese remitente. Es la situación donde todo
filtro desconfía por diseño.

---

## 3 · Lo que ya hizo ARLES (y que no hay que repetir)

| | Qué | Cuándo |
|---|---|---|
| ✅ | Las dos versiones del mensaje: HTML **y** texto plano | 20/10 |
| ✅ | `List-Unsubscribe` en los correos de campaña | 19/10 |
| ✅ | …y **fuera** de los envíos individuales, que no son de ninguna lista | 20/10 |
| ✅ | El HTML, como documento entero con su idioma y su título | 21/10 |

---

## 4 · Las tres salidas, con su precio

### Opción A · Google Workspace con `telemetrymx.com` — la mejor si quieres seguir en Gmail

Unos **6 USD al mes por buzón**. Es Gmail, con la misma interfaz que ya usas,
pero con tu dominio.

Lo que te da y Gmail gratuito **no** puede darte: Google firma tus correos con
**DKIM de `telemetrymx.com`**, así que la identidad del correo es tuya de punta
a punta.

**Pasos:**

1. Contrata Google Workspace en `workspace.google.com` con `telemetrymx.com`
2. Google te pide verificar el dominio: te da un registro TXT que copias en
   HostGator (ver §5)
3. En la consola de administración → Aplicaciones → Gmail → **Autenticar
   correo**: Google genera tu clave DKIM. Copia ese registro en HostGator
4. Añade el SPF y el DMARC de la §5
5. En ARLES, da de alta `contacto@telemetrymx.com` con su contraseña de
   aplicación — el alta reconoce Gmail y rellena el servidor sola

### Opción B · El correo que ya pagas en HostGator — sin coste extra

HostGator incluye buzones del dominio con el hosting. El buzón se gestiona en
cPanel y se lee por webmail, no por la interfaz de Gmail.

**Pasos:**

1. cPanel → **Cuentas de correo** → Crear `contacto@telemetrymx.com`
2. cPanel → **Email Deliverability**: HostGator genera **SPF y DKIM solo** y te
   dice si están bien puestos. Ese botón hace la parte difícil
3. Añade el DMARC de la §5 a mano
4. En ARLES, da de alta la cuenta con:
   - Servidor: `mail.telemetrymx.com`
   - Puerto `465` con cifrado directo, o `587` con STARTTLS
   - Usuario: la dirección completa

### Opción C · Seguir con `@gmail.com`

**No se arregla.** Lo único que se puede hacer es reducir el daño:

- Que quien recibe pulse **«No es correo no deseado»** y añada el remitente a
  sus contactos. Eso enseña al filtro de ese buzón, y sólo de ése
- Quitar el nombre de marca del remitente: `Adrián Paz <pruebas.telemetry@…>`
  levanta menos sospecha que `ARLES <pruebas.telemetry@…>`
- Empezar con poco volumen y subir despacio

Esto no escala: cada destinatario nuevo vuelve a empezar de cero.

---

## 5 · Los tres registros DNS, para copiar y pegar en HostGator

**Dónde:** cPanel → **Editor de Zona DNS** (o *Zone Editor*) → tu dominio →
**Agregar registro**.

> HostGator sólo te deja editar la zona si los *nameservers* del dominio
> apuntan a HostGator. Si los compraste ahí y no los cambiaste, es el caso.

### SPF — quién puede enviar en tu nombre

| Campo | Valor |
|---|---|
| Tipo | `TXT` |
| Nombre | `telemetrymx.com` (o `@`) |
| Valor (Workspace) | `v=spf1 include:_spf.google.com ~all` |
| Valor (HostGator) | el que genera *Email Deliverability* — **no lo escribas a mano** |

**Sólo puede haber un registro SPF.** Si ya hay uno, se **edita** para añadir el
`include:`, no se crea otro: con dos, los dos dejan de valer.

### DKIM — la firma

No se inventa: **lo genera quien manda**.

- Google Workspace: consola de administración → Gmail → Autenticar correo
- HostGator: cPanel → Email Deliverability

Te dan un nombre (algo como `google._domainkey`) y un valor largo. Se copian tal
cual.

### DMARC — qué hacer con lo que no cuadre

| Campo | Valor |
|---|---|
| Tipo | `TXT` |
| Nombre | `_dmarc` |
| Valor | `v=DMARC1; p=none; rua=mailto:contacto@telemetrymx.com` |

**Empieza con `p=none`.** Significa «vigila y mándame informes, pero no rechaces
nada». Es el modo de observación: durante unas semanas recibes informes de quién
manda en tu nombre, y cuando veas que todo cuadra se sube a `p=quarantine`.

Poner `p=reject` el primer día es la forma más rápida de que tu propio correo
deje de llegar.

---

## 6 · Cuánto tarda en notarse

- **El DNS** tarda de minutos a 48 horas en propagarse
- **La reputación** del dominio se construye en **semanas**. Un dominio nuevo
  mandando a desconocidos arranca sin historial, aunque esté todo bien
  configurado
- Lo que **sí** cambia de inmediato es el patrón «marca + gmail personal», que
  es la señal más fuerte de las tres

---

## 7 · Cómo comprobar que quedó bien

1. **mail-tester.com** — te da una dirección, le mandas el correo de prueba
   desde ARLES, y te puntúa del 1 al 10 diciendo qué falta. Es lo más rápido
2. **El «Mostrar original»** del correo recibido, igual que hoy: `spf=pass`,
   `dkim=pass` con `header.d=telemetrymx.com` (tu dominio, no gmail.com) y
   `dmarc=pass`
3. La entrega **7.5 de ARLES**, cuando se construya, comprobará estos tres
   registros desde la aplicación y dirá qué poner si falta algo

---

## 8 · Lo que este documento no promete

**Nadie puede garantizar la bandeja de entrada.** Los filtros son privados,
cambian, y deciden por buzón. Lo que sí se puede afirmar es lo que hoy está
medido:

- El correo que ARLES construye **pasa las cuatro comprobaciones de identidad**
- Con Gmail como destino **ya llega a la bandeja principal** (21/10/2026)
- Con Outlook como destino y `@gmail.com` como remitente, **no**
- Lo que falta por probar es el mismo correo desde `@telemetrymx.com`, y eso
  depende de los pasos de la §4
