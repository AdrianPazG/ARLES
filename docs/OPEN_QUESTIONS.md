# Preguntas abiertas para Dirección

Solo lo que no pudo resolverse por análisis propio. Cada una lleva recomendación (§164).

## Bloqueos

### 1. ¿Se comprará la licencia de aplicación de Mont a Fontfabric?
**Importa:** bloquea la Fase 2 y la identidad completa del producto.
**Recomendación:** comprarla. Previsiblemente de bajo costo frente a rehacer la identidad después de
vender. Si no se compra, decidir *ahora* la sustituta libre — cambiarla en la Fase 12 toca todas las
pantallas.

### 2. ¿Cuáles son el logotipo y los datos de contacto oficiales de TELEMETRY INSIGHT?
**Importa:** §118 y §170 los exigen en el pie; §145 prohíbe inventarlos; no existen en el repositorio.
**Recomendación:** entregar SVG en versión clara y oscura, teléfono en formato internacional, correo de
soporte y URL canónica. Si hay varias versiones del logo, indicar cuál es la vigente.

## Estructura y marca

### 3. ¿Consolidamos `/RECURSOS` o trabajamos con la raíz?
**Importa:** `/RECURSOS` tiene 1 de 27 conceptos y 1 de 66 archivos de fuente; el brief lo trata como
autoritativo.
**Recomendación:** que Dirección complete `/RECURSOS` y elimine el duplicado de la raíz. No se ejecuta
automáticamente: §11 lo prohíbe y una subida incompleta es donde una operación automática pierde material.

### 4. ¿Se conserva la «I» del nombre?
**Importa:** afecta instalador, actualizador, dominio, soporte y material comercial.
**Recomendación:** quitarla. El producto es **ARLES RELAY**; la versión la lleva SemVer y el eje comercial
las ediciones (§78). Si Dirección la conserva, se ejecuta sin objeción — solo se pide registrarlo como ADR.

### 5. ¿1.2.0 o 1.0.0?
**Importa:** SemVer estricto (§3) y el changelog nacerían con un hueco sin explicar.
**Recomendación:** 1.0.0. Si "1.2.0" ya se comunicó, mantenerlo y documentar en la primera entrada del
changelog que es la versión pública inicial. Lo importante es que no quede sin explicación.

## Alcance y equipo

### 6. ¿Lanzamos sin Google?
**Importa:** es la decisión de calendario más determinante del proyecto.
**Recomendación:** sí. Código escrito en v1.2.0 tras bandera de función; se libera cuando Google verifique.
Así la verificación no bloquea el lanzamiento. Empezar el trámite de inmediato de todos modos.

### 7. ¿Qué capacidad real de Rust tiene el equipo?
**Importa:** es lo único que puede invertir la recomendación de Tauri. No es pregunta de tecnología: es de
personas.
**Recomendación:** con al menos una persona con Rust productivo o presupuesto de formación, **Tauri**. Sin
ninguna y sin presupuesto, **Electron bien configurado**, sin culpa. Se necesita respuesta franca, no
optimista.

### 8. ¿Las campañas necesitan adjuntos?
**Importa:** el brief no los menciona. Afectan peso, entregabilidad y superficie de malware.
**Recomendación:** no en v1.2.0; enlaces a archivos alojados. Si el negocio los requiere, decirlo ahora:
cambia el modelo de datos y el almacenamiento.

### 9. ¿Qué versiones mínimas de Windows y macOS soportamos?
**Importa:** determina qué CSS y qué APIs se pueden usar. Con Tauri, la versión de macOS decide
directamente qué funciona en WKWebView.
**Recomendación:** Windows 10 22H2 o superior con WebView2; macOS 12 o superior. Se declara en requisitos y
se prueba contra el mínimo, no contra la última.

### 10. ¿Hay presupuesto para el backend mínimo?
**Importa:** sin él no hay actualización firmada (§80) ni licenciamiento (§76).
**Recomendación:** sí, en su forma mínima: almacenamiento estático con CDN para el manifiesto firmado y una
API sin estado para licencias. **Sin datos de clientes, nunca.** Si no hay presupuesto, la única
alternativa honesta es actualización manual y licencia por archivo firmado, y hay que decidirlo ahora.

### 11. ¿Quién revisa el aviso de privacidad para México?
**Importa:** §91 y §92 piden capacidades de cumplimiento. Se pueden construir las capacidades —origen,
consentimiento, oposición, cancelación, trazabilidad—. **No se puede redactar ni validar el texto legal.**
**Recomendación:** abogado especialista en protección de datos en México, involucrado desde la Fase 4. El
marco normativo aplicable se marca **NO VERIFICADO**: debe confirmarlo un especialista.
