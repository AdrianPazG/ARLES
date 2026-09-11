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

**«REMITENTES» en vez de «EMAIL»** porque nombra lo que el usuario administra ahí —sus cuentas de envío— y no una tecnología. Todo en ARLES es «email»; ese nombre no distingue nada.

**Las supresiones viven en CONTACTOS**, no escondidas en ajustes. Son una decisión sobre personas, y el usuario debe encontrarlas donde están las personas.

---

## 3. INICIO — la pantalla que más se mira

Debe responder a tres preguntas en dos segundos: **¿está corriendo? ¿va bien? ¿hay algo que atender?**

```
┌─────────────────────────────────────────────────────┐
│ ● Campaña «Clientes Q1» · en ejecución              │
│   12 de 35 enviados hoy · siguiente a las 11:34     │
│   ████████░░░░░░░░░░░░  faltan 1 240 · ~25 días     │
├─────────────────────────────────────────────────────┤
│ ⚠ 2 asuntos requieren tu atención                   │
│   · La cuenta ventas@… necesita reconectarse        │
│   · 3 envíos sin confirmar de la campaña «Marzo»    │
├──────────────────────┬──────────────────────────────┤
│ Próximos envíos      │ Últimos eventos              │
└──────────────────────┴──────────────────────────────┘
```

**Lo que no va aquí:** gráficas decorativas, contadores totales sin contexto («12 450 contactos»), tarjetas de bienvenida que no se pueden cerrar.

**El dato más valioso es «faltan 1 240 · ~25 días».** Combina estado y consecuencia, y es lo que hace que alguien reconsidere su configuración.

---

## 4. Onboarding

Nada de tours emergentes de veinte pasos (§25). Una **lista de verificación persistente** en INICIO, que se contrae al completarse y se puede volver a abrir desde AJUSTES.

```
Configura ARLES                                  2 de 5
✓ 1. Configurar empresa
✓ 2. Conectar cuenta de correo
  3. Preferencias de ejecución      [Configurar]
  4. Importar contactos
  5. Primera campaña
```

Cada paso es funcional: al pulsar se va a hacer la cosa, no a leer sobre ella.

**Los pasos no se bloquean entre sí.** Un usuario que quiera importar contactos antes de conectar una cuenta puede hacerlo. La lista sugiere un orden; no impone un embudo.

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
