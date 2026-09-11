# ADR-0001 · Tauri 2 sobre Electron

**Estado:** aceptado · **Fecha:** 2026-09-11 · **Decide:** Tech Lead + Security

---

## Contexto

El brief propone Tauri 2 como shell de escritorio y pide explícitamente una comparación formal frente a Electron, evaluando superficie de ataque, RAM, tamaño de bundle, rendimiento, madurez, actualizaciones, distribución en Windows/macOS y seguridad (§9).

ARLES custodia listas de datos personales de terceros bajo la LFPDPPP, credenciales de correo y —de forma indirecta pero muy real— la reputación del dominio del cliente.

## Decisión

**Tauri 2** (línea 2.11.x, estable desde octubre de 2024).

## Comparación

| Criterio | Tauri 2 | Electron | Gana |
|---|---|---|---|
| Superficie de ataque | Sin Chromium ni Node empaquetados. *Capabilities*: denegar por defecto, habilitar por ventana | Chromium + Node completos. `fs`, `child_process` y red a un `require` de distancia si entra XSS | **Tauri** |
| Cadencia de CVEs | Se hereda la del WebView del SO, que el SO parchea | Cada CVE de Chromium o Node obliga a reempaquetar y redistribuir | **Tauri** |
| RAM | 50–75 % menor | Referencia | **Tauri** |
| Bundle | ~3–10 MB | ~96 MB+ | **Tauri** |
| Consistencia de render | WebView2 (Chromium) vs WKWebView (Safari) — **divergen** | Idéntico en todas las plataformas | **Electron** |
| Madurez del ecosistema | Menor, algunos plugins jóvenes | Muy maduro | **Electron** |
| Actualizaciones | Plugin updater con firma propia; hay que custodiar claves | Muy rodado | Empate |
| Encaje con el equipo | Núcleo en Rust (T-10) | Node/TS en backend | **Tauri** |

## El argumento que decide

**No es el tamaño ni la RAM: es la superficie de ataque.**

En Electron, una inyección de HTML en una plantilla de correo —contenido que en ARLES proviene de archivos XLSX ajenos y de texto que el usuario pega— está a un paso del sistema de archivos, porque el runtime de Node está presente en el mismo proceso. Mitigarlo exige aislamiento de contexto, `nodeIntegration: false`, `contextIsolation: true` y disciplina sostenida en cada ventana que alguien añada durante años.

En Tauri, con capabilities denegadas por defecto y sin `shell` ni `fs` amplio expuestos, **ese paso no existe**. La ausencia de capacidad es una garantía más fuerte que la configuración correcta de una capacidad presente.

## Consecuencias

**Positivas.** Superficie de ataque mínima · CVEs de motor los parchea el SO · instaladores pequeños (relevante para despliegue corporativo) · toda la lógica autoritativa en Rust, con la webview como pura presentación.

**Negativas, y hay que decirlas en voz alta.**

1. **WebView2 y WKWebView no renderizan igual.** Es la fuente número uno de bugs visuales en Tauri: casos límite de flexbox, métricas tipográficas, comportamiento de animaciones. **Mitigación:** cada pantalla se valida en Windows y macOS **antes** de darse por terminada, no en un pase final. El presupuesto de QA lo contempla desde el inicio (riesgo R-07).

2. **Ecosistema menor.** Alguna funcionalidad puede exigir escribir un plugin propio en Rust. Aceptable con T-10.

3. **Custodia de claves de firma del updater.** Las claves privadas **nunca** entran al repositorio: viven en los secretos del CI. La pública se empaqueta con la aplicación.

## Alternativas descartadas

**Electron.** Superficie de ataque y peso, ambos incompatibles con el perfil del producto. La consistencia de renderizado no compensa tener un runtime de Node junto al contenido no confiable.

**Nativo (WinUI + SwiftUI).** Mejor rendimiento y sensación de plataforma, pero duplica el frontend entero y no hay equipo para dos interfaces. No es una discusión técnica, es aritmética.

**Flutter Desktop.** Buena consistencia visual, pero aleja el núcleo de Rust —desaprovechando T-10— y su soporte de escritorio en Windows sigue siendo el eslabón débil.

**Aplicación web + navegador.** Elimina la distribución, pero contradice el §4 (aplicación de escritorio) y, sobre todo, imposibilita el motor de ejecución en segundo plano y el acceso al llavero del sistema operativo, que son requisitos duros.
