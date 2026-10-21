# Paso a paso: dejar el correo del dominio listo

**Para `telemetrymx.com`, con HostGator y Titan.** Escrito el 21/10/2026.

Esto es la versión «qué pulsar y qué escribir». El porqué de cada cosa está en
[QUE-HACER-CON-EL-SPAM.md](QUE-HACER-CON-EL-SPAM.md); aquí sólo están los
pasos.

> **Antes de empezar, una regla que ahorra disgustos:** en el DNS, **nada se
> borra sin haber copiado antes lo que decía**. Un registro borrado por error
> no se recupera, y el correo deja de llegar hasta que alguien recuerde qué
> ponía.

---

## Paso 0 · Lo que `telemetrymx.com` tiene el 21/10/2026

Leído de la captura del Editor de Zona que mandó Dirección. **Queda aquí para
saber de dónde se parte**, porque dentro de un mes nadie se acordará.

### Lo que está bien

| Registro | Valor | |
|---|---|---|
| `MX` 10 | `mx1.titan.email` | ✅ |
| `MX` 20 | `mx2.titan.email` | ✅ |
| `titan3._domainkey` | `v=DKIM1; k=rsa; p=MIGfMA0…` | ✅ **El DKIM de Titan está puesto y entero** |
| `_acme-challenge` | (cadena larga) | ✅ Es del certificado del sitio web |
| `google-site-verification=…` | | ✅ Es de Search Console |

### Lo que hay que arreglar

| | Qué pasa | Gravedad |
|---|---|---|
| **Dos registros SPF** | `include:spf.titan.email` y `include:_spf.google.com`, por separado | 🔴 **El SPF no vale** |
| **Dos registros DMARC** | uno con `p=none` y otro con `p=quarantine` | 🔴 **El DMARC no vale** |
| El segundo DMARC, además, **empieza por `DMARC1;`** y no por `v=DMARC1;` | Sin la `v=` no es un DMARC, es texto | 🔴 Inválido por sí solo |
| `dns.telemetrymx.com` → `titan3._domainkey` | Un registro con el **nombre** del DKIM como valor | 🟡 No hace nada. Sobra |

> **La misma regla rompe el SPF y el DMARC, y conviene entenderla una vez:**
> las dos normas (RFC 7208 y RFC 7489) dicen que si un dominio publica **más de
> un registro** de ese tipo, quien recibe **no elige uno: los descarta todos**.
>
> No es «se queda con el último» ni «gana el más estricto». Es como no tener
> ninguno. Por eso duplicar un registro es peor que no ponerlo: parece
> configurado y no lo está.

## Paso 1 · Apuntar lo que hay ahora

**Dónde:** [mxtoolbox.com/spf.aspx](https://mxtoolbox.com/spf.aspx) → escribe
`telemetrymx.com` → **SPF Record Lookup**.

Esto enseña **sólo los SPF**, sin el ruido de las otras 200 comprobaciones.

**Qué hacer:** copia a un archivo de texto **todas** las líneas que empiecen
por `v=spf1`. Son las que vamos a tocar, y vas a necesitar lo que decían.

Haz lo mismo con:

- [mxtoolbox.com/dmarc.aspx](https://mxtoolbox.com/dmarc.aspx) → tu DMARC
- [mxtoolbox.com/dkim.aspx](https://mxtoolbox.com/dkim.aspx) → pide un
  «selector»; prueba con `titan1` y con `titan`

---

## Paso 2 · Arreglar el SPF duplicado

Es **lo único urgente**. Con dos registros SPF, quien recibe tu correo actúa
como si no tuvieras ninguno.

### 2.1 · Entrar al editor

**cPanel → Dominios → Editor de Zona** (o *Zone Editor*) → junto a
`telemetrymx.com`, pulsa **Administrar**.

En el buscador de esa página escribe `spf` para ver sólo lo que importa.

### 2.2 · Encontrar los dos

Busca los registros de **tipo TXT** cuyo valor empiece por `v=spf1`. En
`telemetrymx.com`, el 21/10/2026, son exactamente estos dos:

```text
TXT   telemetrymx.com   v=spf1 include:spf.titan.email ~all
TXT   telemetrymx.com   v=spf1 include:_spf.google.com ~all
```

### 2.3 · Juntarlos en uno

Toma **todos los `include:`, las `a` y las `mx`** de los dos y ponlos en una
sola línea. Para este dominio, el valor que hay que dejar es:

```text
v=spf1 include:spf.titan.email include:_spf.google.com ~all
```

**Titan primero**, porque es por donde va a mandar ARLES. El de Google se
conserva —no estorba y ocupa uno de los diez permitidos— por si algún día se
manda también desde Google Workspace con este dominio. Si se tiene la certeza
de que eso no va a pasar, se puede dejar sólo el de Titan.

**Las reglas al juntarlos:**

| | |
|---|---|
| Empieza **siempre** por `v=spf1` | Una vez, al principio |
| Termina **siempre** por `~all` | Una vez, al final. Nunca en medio |
| Cada `include:` aparece **una sola vez** | Si está repetido, se deja uno |
| **Diez `include:` como máximo** | Es el límite de la norma. Si pasas de ahí, el SPF falla |

### 2.4 · Guardar

1. **Edita** uno de los dos registros y pon el valor unido
2. **Borra** el otro
3. Guarda

> **Si te da miedo borrar:** edita primero el que vas a conservar, guarda, y
> comprueba en mxtoolbox que el nuevo valor aparece. Sólo entonces borra el
> otro. Durante unos minutos habrá dos y el SPF seguirá roto — pero ya lo está,
> así que no empeora.

### 2.5 · Comprobar

Vuelve a [mxtoolbox.com/spf.aspx](https://mxtoolbox.com/spf.aspx). Tiene que
decir **un solo registro** y «Pass». Puede tardar hasta media hora.

---

## Paso 2 bis · Dejar un solo DMARC

El dominio tiene **dos**, y con dos no vale ninguno. Además, el segundo está
mal escrito: empieza por `DMARC1;` en vez de `v=DMARC1;`, y sin esa `v=` no es
un registro DMARC aunque lo parezca.

**Qué hacer:**

1. cPanel → Editor de Zona → busca `_dmarc`
2. **Conserva este**, que es el válido y el correcto para empezar:

   ```text
   v=DMARC1;p=none;sp=none;adkim=r;aspf=r;pct=100;fo=0;rf=afrf;ri=86400;rua=mailto:enlace@telemetrymx.com
   ```

3. **Borra el otro**, el que empieza por `DMARC1; p=quarantine`

> **Por qué se queda el de `p=none` y no el de `quarantine`.** No es por ser
> conservador porque sí: `p=quarantine` le dice a quien recibe que mande a no
> deseados todo lo que no cuadre — y mientras el SPF esté roto, **no cuadra
> nada**. Arreglar primero, endurecer después.

> **Comprueba que `enlace@telemetrymx.com` existe.** Es la dirección a donde
> van los informes semanales. Si ese buzón no existe, los informes rebotan y te
> quedas sin la información que justifica subir a `quarantine` más adelante.

---

## Paso 2 ter · Borrar el registro que sobra

Hay un registro TXT llamado `dns.telemetrymx.com` cuyo valor es
`titan3._domainkey`.

Eso es **el nombre del DKIM metido como valor** en un sitio equivocado — pasa
al copiar de un panel que enseña las dos columnas juntas. No hace daño y no
hace nada. **Bórralo**, para que el día que alguien revise esto no pierda media
hora preguntándose qué es.

El DKIM de verdad —el que sí funciona— es el otro: el que se llama
`titan3._domainkey.telemetrymx.com` y cuyo valor empieza por `v=DKIM1;`.

---

## Paso 3 · El DKIM de Titan

> **En `telemetrymx.com` esto YA ESTÁ HECHO** (21/10/2026): el registro
> `titan3._domainkey` está puesto y su valor llega entero hasta `…IDAQAB`, que
> es como termina una clave de este tipo. Este paso queda por si hay que
> rehacerlo o añadir otro dominio.

**Qué es:** la firma que demuestra que el correo salió de verdad de tu dominio.

**Dónde se consigue:** en el panel de Titan (se entra desde HostGator, o en
`app.titan.email`), en los ajustes del dominio. Titan te da **un nombre** y
**un valor** — no se inventan ni se copian de otro sitio.

El nombre suele ser algo como `titan1._domainkey` y el valor una línea muy
larga que empieza por `v=DKIM1;`.

**Cómo ponerlo:** cPanel → Editor de Zona → **Agregar registro**:

| Campo | Qué poner |
|---|---|
| Tipo | `TXT` |
| Nombre | lo que diga Titan, por ejemplo `titan1._domainkey` |
| TTL | `14400` (lo que venga por defecto) |
| Valor | la línea larga de Titan, **entera y sin espacios de más** |

> **El error más común aquí** es copiar el valor cortado. Si al pegarlo ves que
> el final no coincide con lo que enseña Titan, ha quedado a medias — y un DKIM
> a medias falla igual que no tenerlo.

---

## Paso 4 · Revisar el DMARC

Ya lo tienes puesto. Comprueba sólo que dice esto:

```text
v=DMARC1; p=none; rua=mailto:contacto@telemetrymx.com
```

| Parte | Qué significa |
|---|---|
| `p=none` | **Observa, no rechaces.** Es lo correcto ahora |
| `rua=mailto:…` | A dónde te mandan los informes semanales |

**No lo subas a `p=quarantine` todavía.** Primero hay que ver unas semanas de
informes y confirmar que todo tu correo legítimo pasa. Subirlo antes es la
forma más rápida de que tus propios correos dejen de llegar.

---

## Paso 5 · Crear el buzón en Titan

1. Entra al panel de Titan
2. Crea el buzón, por ejemplo `contacto@telemetrymx.com`
3. **Busca los ajustes de acceso y activa el acceso desde programas de
   terceros.** Sin esto, Titan rechaza a ARLES
4. **Si tienes la verificación en dos pasos activada en Titan, desactívala para
   ese buzón** — Titan bloquea los programas externos cuando está puesta

---

## Paso 6 · Dar de alta la cuenta en ARLES

**Remitentes → Agregar cuenta.**

Como `telemetrymx.com` no es un proveedor que ARLES reconozca —no es Gmail ni
Outlook—, se abre sola la **Configuración avanzada**. Rellena:

| Campo | Valor |
|---|---|
| Correo | `contacto@telemetrymx.com` |
| Contraseña | la del buzón de Titan |
| Servidor | `smtp.titan.email` |
| Puerto | `465` |
| Cifrado | **TLS directo** |

> **Si no conecta con el 465**, prueba `587` con **STARTTLS**. Son los dos
> pares que Titan publica, y ARLES no deja cruzarlos: 465 va con directo y 587
> con STARTTLS, nunca al revés.

Después, **Enviar correo de prueba** a tu cuenta de Outlook.

---

## Paso 7 · Comprobar que todo quedó bien

### 7.1 · Con mail-tester

1. Entra en [mail-tester.com](https://www.mail-tester.com)
2. Copia la dirección que te da
3. Mándale el correo de prueba desde ARLES
4. Vuelve a mail-tester y pulsa **Then check your score**

Te puntúa del 1 al 10 y dice qué falta. **Por encima de 8 está bien.**

### 7.2 · Con las cabeceras del correo recibido

En Outlook: abre el correo → los tres puntos → **Ver** → **Ver origen del
mensaje**. Busca la línea `Authentication-Results` y comprueba que diga:

```text
spf=pass      dkim=pass      dmarc=pass
```

Y lo que de verdad importa, que es nuevo: junto a `dkim=pass` tiene que
aparecer **`header.d=telemetrymx.com`**. Hoy dice `header.d=gmail.com`.

**Ese cambio es el objetivo entero de todos estos pasos.**

---

## Si algo no sale

| Lo que ves | Qué suele ser |
|---|---|
| ARLES dice que no puede autenticarse | El acceso de terceros de Titan, o la verificación en dos pasos |
| ARLES dice que no se pudo cifrar | El puerto y el cifrado cruzados: 465→directo, 587→STARTTLS |
| mxtoolbox sigue viendo dos SPF | No ha propagado. Espera 30 minutos y vuelve |
| `dkim=none` en las cabeceras | El valor se pegó cortado, o aún no propagó |
| Sigue cayendo en no deseados | La reputación del dominio se construye en **semanas**. Empieza con poco volumen |
