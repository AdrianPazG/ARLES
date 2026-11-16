# Capabilities de Tauri

**Se deniega por defecto.** Cada permiso que aparezca aquí tiene que justificarse.

Es el argumento central de [ADR-0001](../../../documentacion/03-arquitectura/adr/0001-tauri-2-sobre-electron.md): en Electron, una inyección de HTML en una plantilla de correo —contenido que en ARLES proviene de archivos XLSX ajenos— está a un `require('child_process')` del sistema de archivos. En Tauri, con las capabilities denegadas, **esa capacidad no existe en el proceso**.

La ausencia de una capacidad es una garantía más fuerte que la configuración correcta de una capacidad presente: la configuración se cambia en un commit de viernes por la tarde.

---

## Lo que NO se habilita, y por qué

| Plugin | Motivo |
|---|---|
| `shell` | Ejecutar procesos es exactamente la escalada que la elección de Tauri evita |
| `fs` (amplio) | El acceso al sistema de archivos vive en Rust, con rutas que el frontend nunca ve (regla de frontera 3.4) |
| `http` | El I/O de red pasa por comandos tipados. La webview no habla con el exterior |
| `process` | No hace falta |
| `clipboard-write` sin acotar | Se evaluará cuando una funcionalidad lo pida, no antes |

## Lo que se añadirá, cuando toque

| Plugin | Fase | Para qué |
|---|---|---|
| `dialog` (solo abrir archivo) | 3 | Selector de XLSX/CSV. La ruta se resuelve en Rust y el frontend recibe un identificador opaco |
| `dialog` (guardar) | 8 | Exportar respaldos `.arles` |
| `opener` | 5 | Abrir el navegador del sistema para el consentimiento OAuth — **nunca** una webview embebida |
| `updater` | 9 | Actualizaciones con verificación de firma (§79) |

## Regla

Añadir un permiso aquí es una decisión de seguridad, no de comodidad. Si un permiso resuelve un problema que también se resuelve moviendo lógica a Rust, **se mueve la lógica a Rust**.
