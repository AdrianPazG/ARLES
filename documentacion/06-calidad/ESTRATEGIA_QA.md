# Estrategia de QA

**Proyecto:** ARLES RELAY I · v1.2.0

---

## 1. Reparto

| Nivel | Herramienta | Cubre | Peso |
|---|---|---|---|
| Unitario Rust | `cargo test` | Lógica de dominio pura (`arles-core`) | **El grueso** |
| Integración Rust | `cargo test` + SQLite temporal | Repositorios, motor con proveedor simulado | Alto |
| Unitario Vue | Vitest | Componentes, stores, formato | Medio |
| E2E | Playwright | Camino dorado y flujos críticos | Bajo pero imprescindible |

**El peso está en Rust deliberadamente.** La regla de frontera 3.1 pone toda la autoridad de negocio en Rust, así que ahí es donde vive lo que puede romperse de forma cara. `arles-core` no hace I/O, lo que permite testear normalización, cálculo de ventanas, transiciones de estado y estimación del simulador sin levantar nada.

---

## 2. El camino dorado (§105)

Un E2E que recorre el bucle central completo:

```
configurar empresa
  → conectar SMTP (servidor de pruebas local)
  → importar 1 000 contactos desde XLSX
  → crear lista, aplicar etiquetas
  → crear campaña
  → redactar con variables
  → fijar límites
  → simular          [verifica que la duración estimada es correcta]
  → preflight        [verifica que devuelve LISTA]
  → envío de prueba
  → activar
  → pausar
  → reanudar
  → completar
  → revisar resultados
  → respaldar y restaurar
```

Corre en CI en Windows y macOS. Un fallo aquí bloquea el merge.

---

## 3. Los casos que de verdad rompen el producto

Esto no es una lista de deseos: es la lista de los fallos que convierten ARLES en un problema para el cliente. Cada uno tiene un test con nombre.

### Motor — los críticos

| Caso | Qué debe pasar |
|---|---|
| **El equipo se suspende a mitad de campaña y despierta al día siguiente** | **No hay ráfaga.** El cubo se reinicia, no se acumula (R-04) |
| **La app muere entre el `accept` del proveedor y el commit** | `presumed_sent`. **Jamás un duplicado** (R-03, ADR-0004) |
| Doble clic en «Activar» | Una sola campaña activa, un solo conjunto de intentos |
| Dos instancias de ARLES a la vez | La segunda detecta el bloqueo y avisa; no corrompe nada |
| Reinicio a mitad de envío | La cola se reanuda exactamente donde estaba |
| Parada de emergencia con envío en curso | Ningún envío nuevo sale. Se respeta **antes de cada envío**, no entre lotes |
| Límite alcanzado a mitad de campaña | **Pausa con aviso. Sin rotación** (T-1) |
| Contacto suprimido mientras la campaña corre | **No recibe.** Comprobación en el momento del envío |

### Tiempo — donde se esconden los bugs sutiles

| Caso | Qué debe pasar |
|---|---|
| Cambio de horario de verano dentro de la ventana | Sin doble envío ni hueco. Ventanas calculadas en la zona de la empresa |
| Campaña que cruza medianoche | El presupuesto diario cambia a la hora correcta de la empresa |
| El portátil viaja a otro huso | **La política no cambia.** Se usa `company.timezone`, no la del sistema |
| Reloj del sistema ajustado hacia atrás | No se duplican permisos |

### Red y credenciales

| Caso | Qué debe pasar |
|---|---|
| Conexión perdida a mitad de envío | Reintento con backoff. Estado coherente |
| Token OAuth revocado con campaña corriendo | Circuito abierto, `needs_reauth`, aviso claro |
| Contraseña de aplicación caducada | Igual |
| Servidor devuelve 429 con `Retry-After` | Se respeta **literalmente** |
| Servidor devuelve 5 000 caracteres de basura | Se trunca, no se interpola en ningún sitio, no se cae |
| Servidor tarda 10 minutos en responder | Timeout, `Transient`, reintento |

### Importación — la superficie de ataque

| Caso | Qué debe pasar |
|---|---|
| **Zip bomb en XLSX** | Rechazo por ratio de descompresión. Memoria acotada |
| 500 000 filas | Se importa, memoria acotada, dentro de presupuesto |
| Celda de 50 MB | Rechazo por tamaño de celda |
| Fórmulas en celdas | Se importan **como texto**. Nunca se evalúan |
| **Exportar contacto llamado `=HYPERLINK(...)`** | Se antepone `'` en el CSV exportado |
| Nombre de archivo `../../etc/passwd` | Se escribe con UUID. El original sólo como metadato |
| Nombre de archivo `CON`, `PRN`, `NUL` (Windows) | Igual |
| Archivo con BOM, UTF-16, CRLF mezclado | Se detecta y se lee correctamente |
| Contacto suprimido en el archivo | Se importa el contacto; **la supresión permanece** |

### Base de datos

| Caso | Qué debe pasar |
|---|---|
| Base bloqueada por otro proceso | Reintento con `busy_timeout`; mensaje claro si persiste |
| **Llavero no disponible** | **La aplicación se niega a arrancar** con mensaje accionable (ADR-0011) |
| Migración sobre base poblada | Copia previa; migración correcta; datos intactos |
| Restaurar respaldo en otra máquina | Datos restaurados; cuentas en `needs_reauth` con mensaje claro (§75) |
| Disco lleno a mitad de importación | Transacción revertida. Sin estado a medias |

---

## 4. Verificación entre plataformas

**Cada pantalla se valida en Windows y macOS antes de darse por terminada.**

No es un pase final: es parte de la definición de «hecho». WebView2 (Chromium) y WKWebView (Safari) divergen en casos límite de flexbox, métricas tipográficas y animación — es el coste aceptado de ADR-0001 y el riesgo R-07.

Además, escalado de Windows al 100 %, 125 %, 150 % y 200 % (§22), y pantallas Retina y no-Retina.

---

## 5. Las cuatro auditorías (§140)

Tras cada hito del roadmap, de forma **aislada** — cuatro pasadas separadas, no una revisión general.

### A · Arquitectura
Fronteras entre crates respetadas · `arles-core` sin I/O · la webview sin lógica de negocio · sin objetos-dios · sin números mágicos · sin `unwrap()` ni `any` indiscriminados (§138)

### B · Seguridad
La lista completa está en `04-seguridad/THREAT_MODEL.md` §8. Resumen: sin concatenación en consultas · `Secret<T>` y capa de redacción activas · ningún comando IPC devuelve credenciales · capabilities denegadas por defecto · CSP estricta · sanitizado en Rust al guardar y al usar · topes de importación probados con archivos reales · `cargo audit` y `npm audit` en CI

### C · QA y rendimiento
Casos frontera de §3 cubiertos · presupuestos de `PRESUPUESTO_RENDIMIENTO.md` medidos, no estimados · sin fugas de memoria en ejecuciones largas del motor

### D · UX y accesibilidad
Contraste AA **calculado** en todos los pares · foco visible siempre · navegación completa por teclado · ningún estado sólo por color · `prefers-reduced-motion` · objetivos ≥32 px a escalado 200 % · los cuatro estados en cada pantalla (§97) · textos de error con las tres partes (§95)

---

## 6. CI

```
1. Formato y lint      rustfmt · clippy -D warnings · eslint · tsc --noEmit
2. Unitarios           cargo test · vitest
3. Integración         cargo test --features integration
4. Seguridad           cargo audit · npm audit
5. Contraste           script de cálculo WCAG sobre los tokens   ← rompe la build
6. Rendimiento         presupuestos con conjunto sintético de 500 k
7. E2E                 Playwright en Windows y macOS
8. Build               instaladores para ambas plataformas
```

**El paso 5 es inusual y deliberado.** Una regresión de contraste debe romper la compilación, no descubrirse meses después en una auditoría. Los tokens y sus ratios se verifican con el mismo cálculo documentado en `COLOR_SYSTEM.md` §6.

---

## 7. Datos de prueba

**Conjunto sintético de 500 000 contactos**, generado con semilla fija y reproducible. Incluye deliberadamente: acentos y caracteres no ASCII · correos en mayúsculas, con espacios, con `+` y con puntos · campos vacíos · duplicados exactos y por normalización · direcciones inválidas · nombres que parecen fórmulas.

**Nunca datos reales de clientes en tests**, ni siquiera anonimizados. Es una regla de privacidad, no de comodidad.

**Servidor SMTP de pruebas local** (MailHog o equivalente) para los E2E. Ninguna prueba envía correo real a internet.

---

## 8. Definición de «hecho»

Una funcionalidad está terminada cuando:

- [ ] Tests unitarios y de integración pasan
- [ ] Casos frontera relevantes cubiertos
- [ ] Verificada en Windows **y** macOS
- [ ] Los cuatro estados implementados (§97)
- [ ] Accesible por teclado, con foco visible
- [ ] Contraste verificado
- [ ] Textos de error con las tres partes (§95)
- [ ] Sin secretos en registros
- [ ] Documentación actualizada si cambia una decisión
