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

Esto no es una lista de deseos: es la lista de los fallos que convierten ARLES
en un problema para el cliente.

> **Cada caso lleva el nombre de su test, y eso es nuevo del 07/10/2026.** Hasta
> ese día esta sección decía «cada uno tiene un test con nombre» **sin decir
> cuál**, así que la afirmación no se podía comprobar — y la auditoría C del
> §140 los buscó uno a uno y encontró que a **dos** les faltaba el test y que
> **tres** describían caminos que el producto no tiene.
>
> Ahora los nombres están escritos, y `validar.py --fase 8` falla si alguno deja
> de existir: `cada_caso_frontera_nombra_su_test`. Una lista que nadie puede
> comprobar envejece sin que se note, que es lo que pasó aquí.

### Motor — los críticos

| Caso | Qué debe pasar | Su test |
|---|---|---|
| **El equipo se suspende a mitad de campaña y despierta al día siguiente** | **No hay ráfaga.** El cubo se reinicia, no se acumula (R-04) | `ocho_horas_dormido_se_detectan_y_no_tocan_el_presupuesto` |
| **La app muere entre el `accept` del proveedor y el commit** | Ambiguo. **Jamás un duplicado** (R-03, ADR-0004) | `un_envio_a_medias_no_se_reenvia_jamas` |
| Doble clic en «Activar» | Una sola campaña activa, un solo conjunto de intentos | `activar_dos_veces_no_duplica_nada` |
| Dos instancias de ARLES a la vez | La segunda detecta el cerrojo y avisa; no corrompe nada | `el_segundo_arles_no_abre_la_misma_base` |
| Reinicio a mitad de envío | La cola se reanuda exactamente donde estaba | `lo_tomado_y_no_enviado_vuelve_a_la_cola` |
| Parada de emergencia con envío en curso | Ningún envío nuevo sale. Se respeta **antes de cada envío**, no entre lotes | `la_parada_se_comprueba_otra_vez_justo_antes_de_enviar` |
| Límite alcanzado a mitad de campaña | **Pausa con aviso. Sin rotación** (T-1) | `sin_cupo_del_dia_se_pausa_y_no_se_busca_otra_cuenta` |
| Contacto suprimido mientras la campaña corre | **No recibe.** Comprobación en el momento del envío | `quien_pidio_la_baja_no_recibe_nada` |

### Tiempo — donde se esconden los bugs sutiles

| Caso | Qué debe pasar | Su test |
|---|---|---|
| Cambio de horario de verano dentro de la ventana | Sin doble envío ni hueco. Ventanas calculadas en la zona de la empresa | `el_cambio_de_horario_no_mueve_la_ventana` (Europe/Madrid: México ya no tiene horario de verano, así que con una zona mexicana este caso **no se podría probar**) |
| Campaña que cruza medianoche | El presupuesto diario cambia a la hora correcta de la empresa | `el_dia_del_presupuesto_cambia_a_medianoche_local` |
| El portátil viaja a otro huso | **La política no cambia.** Se usa `company.timezone`, no la del sistema | `dos_campanas_en_husos_distintos_no_se_pisan` |
| Reloj del sistema ajustado hacia atrás | No se duplican permisos | `atrasar_el_reloj_no_regala_el_cupo_del_dia` |

### Red y credenciales

| Caso | Qué debe pasar | Su test |
|---|---|---|
| Conexión perdida a mitad de envío | Reintento a los 5, 15 y 30 minutos (D-12), **no backoff** | `un_fallo_pasajero_reintenta_a_los_cinco_minutos` |
| Token de Meta revocado con campaña corriendo | Circuito abierto, `needs_reauth`, aviso claro | `una_credencial_caducada_para_la_cuenta_a_la_primera` |
| Contraseña de aplicación caducada | Igual | `una_credencial_caducada_para_la_cuenta_y_no_gasta_reintentos` |
| El proveedor dice cuánto esperar | Se respeta **literalmente** | `cuando_el_proveedor_dice_cuanto_esperar_se_le_hace_caso`. **Ojo:** la espera la decide el adaptador, no una cabecera — ver el §3 bis |
| Servidor devuelve 5 000 caracteres de basura | Se trunca, no se interpola en ningún sitio, no se cae | `cinco_mil_caracteres_de_basura_se_recortan` |
| Servidor acepta la conexión y calla | Timeout a los 60 s, `Pasajero`, reintento | `se_le_puede_ordenar_que_cuelgue_sin_saludar` |

### Importación — la superficie de ataque

| Caso | Qué debe pasar | Su test |
|---|---|---|
| **Zip bomb en XLSX** | Rechazo por ratio de descompresión. Memoria acotada | `una_bomba_de_descompresion_se_corta_antes_de_agotar_la_memoria` |
| 500 000 filas | Se importa, memoria acotada, dentro de presupuesto | **no es un test**: lo mide `arles-banco`, y la cifra está en `PRESUPUESTO_RENDIMIENTO.md` §0 |
| Celda enorme | Rechazo por tamaño de celda | `una_celda_enorme_no_tumba_la_lectura` |
| Fórmulas en celdas | Se importan **como texto**. Nunca se evalúan | `las_formulas_de_un_xlsx_se_leen_como_texto_y_nunca_se_evaluan` |
| Nombre de archivo `../../etc/passwd` | Se escribe con el id del lote. El original sólo como metadato | `un_nombre_de_archivo_hostil_no_sale_de_la_lectura_como_ruta` |
| Archivo con BOM, UTF-16, CRLF mezclado | Se detecta y se lee correctamente | `un_csv_en_utf16_se_lee_con_sus_acentos`, `el_bom_de_utf8_no_se_pega_al_primer_encabezado`, `los_saltos_de_linea_mezclados_no_dejan_un_retorno_en_el_dato` — **los tres escritos el 07/10/2026**, porque no había ninguno |
| Contacto suprimido en el archivo | Se importa el contacto; **la supresión permanece** | `una_direccion_suprimida_entra_pero_se_anuncia` |

### Base de datos

| Caso | Qué debe pasar | Su test |
|---|---|---|
| **Llavero no disponible** | **La aplicación se niega a arrancar** con mensaje accionable (ADR-0011) | `sin_llavero_no_arranca_y_con_llavero_queda_migrada` |
| Migración sobre base poblada | Migración correcta; datos intactos | `migrar_una_base_con_datos_no_los_toca` |
| Restaurar respaldo en otra máquina | Datos restaurados; cuentas en `needs_reauth` (§75) | `un_respaldo_se_escribe_y_se_restaura_en_otra_base` |
| **Disco lleno a mitad de importación** | Transacción revertida. Sin estado a medias | `un_disco_lleno_a_mitad_de_importacion_no_deja_nada_a_medias` — **escrito el 07/10/2026**, con `PRAGMA max_page_count` para provocar el `SQLITE_FULL` de verdad |

---

## 3 bis. Los tres casos que describían caminos que no existen

Los tres estaban en la lista desde la Fase 0, cuando ARLES se imaginaba de otra
forma. **No se borran**: quedan con su fecha y la condición que los devolvería a
contar, igual que los dos puntos del `THREAT_MODEL.md` §8 que corrigió la
auditoría B el mismo día.

Un caso frontera que no se puede cumplir hace dudar de la lista entera, y la
lista es lo único que queda cuando nadie se acuerda.

| Caso | Por qué no aplica (07/10/2026) | Cuándo vuelve a contar |
|---|---|---|
| **Exportar un contacto llamado `=HYPERLINK(...)` → se antepone `'` en el CSV** | **ARLES no exporta CSV.** Escribe `.txt` —el informe ARCO y el resumen de campaña, los dos en prosa— y `.arles`. En un `.txt` que una persona lee, antepoper `'` al apellido de alguien **estropea el documento**: nadie quita el apóstrofo. La función `neutralizar_formula` existe, está probada, y **no la llama nadie a propósito** | El día que haya una salida que una hoja de cálculo abra directamente |
| **Nombre de archivo `CON`, `PRN`, `NUL` en Windows** | El nombre suministrado **no toca el disco nunca**: `arles-db` guarda el archivo con el id del lote, y el original sólo se enseña. Un nombre reservado no puede hacer daño porque no se usa para abrir nada | El día que algo escriba usando el nombre de fuera |
| **Token OAuth revocado** | **No hay OAuth en v1.2.0** (ADR-0003: SMTP primero). El caso equivalente que sí existe —el token de Meta revocado, código 190— está arriba, con su test | Cuando aterrice `GoogleProvider` en v1.2.x |

### Y una cosa que el caso del `Retry-After` no cumple como está escrita

El caso decía «servidor devuelve 429 con `Retry-After` → se respeta
**literalmente**». La mitad de abajo es verdad y está probada: cuando el
adaptador devuelve `NoMeMandesNadaDurante(d)`, el motor espera exactamente `d`.

Lo que **no** existe es la cabecera. `ErrorDeMeta` lleva tres campos —código,
subcódigo y estado HTTP— y **ninguna cabecera de la respuesta se lee**. Ante un
429 el adaptador espera los 60 s de `ESPERA_POR_RITMO`, que es una constante
nuestra, no lo que el servidor pidió. Y SMTP no tiene ninguna cabecera así.

Está anotado como hallazgo **C-5** en
[`AUDITORIA-C-QA.md`](../09-fases/AUDITORIA-C-QA.md), con su recomendación.

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
