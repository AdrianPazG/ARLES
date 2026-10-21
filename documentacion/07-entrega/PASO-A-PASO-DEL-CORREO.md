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

Busca los registros de **tipo TXT** cuyo valor empiece por `v=spf1`. Vas a ver
dos. Un ejemplo de lo que suele pasar:

```text
TXT   telemetrymx.com   v=spf1 include:spf.titan.email ~all
TXT   telemetrymx.com   v=spf1 a mx include:websitewelcome.com ~all
```

### 2.3 · Juntarlos en uno

Toma **todos los `include:`, las `a` y las `mx`** de los dos, y ponlos en una
sola línea. Del ejemplo de arriba saldría:

```text
v=spf1 a mx include:spf.titan.email include:websitewelcome.com ~all
```

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

## Paso 3 · El DKIM de Titan

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
