# Pedido de compilación de revisión

**Edita este archivo y guárdalo para lanzar una compilación de revisión.**
No hace falta escribir nada concreto: basta con que el archivo cambie.

## Cómo, desde el navegador y sin consola

1. Abre este archivo en GitHub, en la rama `claude/loving-faraday-5chmmy`.
2. Pulsa el **lápiz** (✏️ *Edit this file*), arriba a la derecha.
3. Añade una línea al final con la fecha y quién lo pide.
4. Abajo, botón verde **«Commit changes…»** → **«Commit changes»**.
5. Ve a la pestaña **Actions**: verás arrancar **«Revisión visual»**.

Tarda entre 15 y 25 minutos. Cuando acabe, los instaladores están al final de
la ejecución, en **Artifacts**.

El resto —cómo abrirlos saltándose los avisos de Windows y macOS, qué mirar y
qué capturar— está en `documentacion/06-calidad/REVISION_VISUAL.md`.

## Por qué existe este archivo

El botón **«Run workflow»** de la pestaña Actions solo aparece si el flujo está
en la **rama por defecto** del repositorio. Hoy `main` no tiene ni carpeta
`.github/`, así que ese botón no existe y no se puede pulsar.

Tocar este archivo es la puerta que sí funciona desde la rama de trabajo.
Cuando el proyecto se integre en `main`, el botón aparecerá y este archivo
dejará de hacer falta.

---

## Registro de pedidos

<!-- Añade tu línea aquí abajo. -->

- 2026-09-14 · Primera compilación de revisión (Fase 2 → 3.1)
- 2026-09-14 · Verificación del flujo tras corregir las rutas de tauri.conf.json
- 2026-09-14 · Segunda verificación: iconos declarados para el empaquetado de Windows
- 2026-09-15 · **Entrega 3.1**, a petición de Dirección: Inicio con la lista de
  alta, Ajustes con la configuración de empresa y la barra lateral fija y
  plegable (P-11). Incluye los siete arreglos de la auditoría de funcionamiento
- 2026-09-15 · **3.1, segunda pasada.** La compilación anterior se lanzó antes
  de mirar las capturas, y mirarlas dejó dos arreglos más: los desplegables
  enseñaban «MX» y «America/Mexico_City» en vez de «México» y «Ciudad de
  México», y los pasos que aún no se pueden hacer pesaban más en la pantalla
  que el único accionable. **Usa los instaladores de esta ejecución, no los de
  la anterior.**
