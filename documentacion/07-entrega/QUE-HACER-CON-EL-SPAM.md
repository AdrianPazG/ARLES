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

### Opción B · Titan, el correo que **ya tienes** — sin coste extra

**Medido el 21/10/2026 con mxtoolbox:** los servidores de correo de
`telemetrymx.com` son `mx1.titan.email` y `mx2.titan.email`. Es decir, el
dominio **ya tiene correo profesional con Titan**, que es lo que HostGator
vende como «correo del dominio».

Eso cambia la recomendación: no hace falta contratar Google Workspace para
tener un remitente con el dominio propio. Ya está.

**Pasos:**

1. En el panel de Titan (desde HostGator, o en `app.titan.email`), crear el
   buzón `contacto@telemetrymx.com`
2. **Activar el acceso de terceros** en los ajustes de Titan. Sin eso, Titan
   bloquea cualquier programa que no sea el suyo — y ARLES es uno
3. Los registros SPF y DKIM de la §5
4. En ARLES, dar de alta la cuenta con **Configuración avanzada**:
   - Servidor: `smtp.titan.email`
   - Puerto `465` con cifrado directo, **o** `587` con STARTTLS
   - Usuario: la dirección completa

> **Aviso de Titan que conviene saber antes:** con la verificación en dos pasos
> activada, Titan **bloquea** el acceso desde programas de terceros. Si ARLES
> no consigue autenticarse, es lo primero que hay que mirar.

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
| Valor (Titan) | `v=spf1 include:spf.titan.email ~all` |
| Valor (Workspace) | `v=spf1 include:_spf.google.com ~all` |

Y si se manda desde los dos, **se unen en uno solo**, no se ponen dos:

```text
v=spf1 include:spf.titan.email include:_spf.google.com ~all
```

**Sólo puede haber un registro SPF.** Si ya hay uno, se **edita** para añadir el
`include:`, no se crea otro: con dos, los dos dejan de valer.

### DKIM — la firma

No se inventa: **lo genera quien manda**.

- Titan: lo genera al configurar el dominio; está en su panel, en los ajustes
  del dominio
- Google Workspace: consola de administración → Gmail → Autenticar correo

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

## 5 bis · Cómo leer un informe de mxtoolbox

**Dirección pasó `telemetrymx.com` por mxtoolbox el 21/10/2026** y salieron
«4 errores y 6 avisos». De los diez, **uno es un problema de verdad**. Conviene
saber separarlos, porque una herramienta que marca en rojo lo que es una
recomendación lleva a cambiar cosas que estaban bien.

### El único que hay que arreglar

| | |
|---|---|
| **`spf` · More than one record found** | 🔴 **Rompe el SPF entero** |

**Dos registros SPF no son «SPF por duplicado»: son SPF roto.** La norma
(RFC 7208) dice que un dominio con más de uno produce un `PermError`, y quien
recibe el correo lo trata como si **no hubiera ninguno**. Es decir: hoy el
dominio está peor que si no tuviera SPF.

Se arregla **uniendo los dos en uno**, no borrando uno al azar: cada `include:`
autoriza a un remitente distinto, y quitar el equivocado deja sin autorizar al
que de verdad manda.

```text
v=spf1 include:spf.titan.email include:loDelOtro ~all
```

### Lo que NO es un error aunque salga en rojo

| | Qué dice | Por qué está bien |
|---|---|---|
| `dmarc` | *Quarantine/Reject policy not enabled* | Es `p=none`, **a propósito**. Es el modo de observación, y subirlo antes de tiempo hace que tu propio correo deje de llegar (§5) |
| `spf` / `mx` | *[SUPPLEMENTAL] …required to enable BIMI* | **BIMI es el logotipo** junto al remitente. Es cosmético, exige DMARC estricto y un certificado de marca de pago |

### Lo que no es tuyo

| | Qué dice | De quién es |
|---|---|---|
| `smtp` ×2 | *Reverse DNS does not match SMTP Banner* | De los servidores de **Titan**. No se puede cambiar desde el DNS del dominio |
| `dns` ×3 | *Name Servers on the Same Subnet*, *SOA Refresh/Expire* | Del **hosting**. Son recomendaciones de robustez del DNS, sin efecto en que el correo llegue |

**El resumen honesto: 194 comprobaciones pasadas y una cosa que arreglar.**

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
- Con Gmail como destino **llega a la bandeja principal** (21/10/2026)
- Con Outlook como destino y `@gmail.com` como remitente, **no** — y eso **no
  se arregló**: se evitó cambiando de remitente

---

## 9 · Resuelto el 22/10/2026 · funcionó, y así quedó

**Desde `enlace@telemetrymx.com` (Titan), el correo de prueba de ARLES llegó a
la BANDEJA PRINCIPAL de los tres destinos: Outlook, Gmail y un corporativo.**

Se tomó la **opción B** de la §4 —Titan, el correo que el dominio ya tenía—, sin
contratar nada. Lo que hizo falta, por orden:

| | Qué | Dónde |
|---|---|---|
| 1 | Crear el buzón `enlace@` y ponerle contraseña | Panel de Titan, desde HostGator |
| 2 | Unir los **dos SPF** en uno — dos registros no son «SPF duplicado», son **SPF roto** (RFC 7208: `PermError`, y quien recibe lo trata como si no hubiera ninguno) | Zone Editor de HostGator |
| 3 | Dejar **un solo DMARC**, el que empieza por `v=DMARC1` | Zone Editor |
| 4 | Dar de alta la cuenta en ARLES — dos campos desde el 14/10 | Remitentes |

**El DKIM de Titan ya estaba** y entero, que es lo que más falla: pegarlo
cortado no da error, simplemente no firma.

### Qué cambió de verdad, y qué no

**Lo que cambió:** el remitente. Nada del código de ARLES, que ya construía el
correo bien desde el 21/10 — las dos versiones del mensaje, el HTML como
documento entero y la cabecera de baja.

**Lo que NO cambió:** mandar desde `@gmail.com` sigue cayendo en no deseados de
Outlook. No se arregló; se dejó de hacer.

> **Y la lección que vale para la próxima campaña:** el dominio no se arriesga
> por usarlo, se arriesga por el **volumen**. Un dominio nuevo que arranca con
> dos mil correos en un día es el caso malo; cincuenta al día durante dos
> semanas es el bueno. Ese freno es el límite diario de la campaña, que desde el
> 22/10/2026 se fija al crearla y **se aplica de verdad**.
