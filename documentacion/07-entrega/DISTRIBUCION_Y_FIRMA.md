# Distribución y firma

**Proyecto:** ARLES RELAY I · v1.2.0
**Alcance de v1.2.0:** despliegue interno TELEMETRY (D-4)

---

## 1. Lo urgente, primero

**Los trámites de firma de código arrancan en la Fase 1**, aunque D-4 difiera el lanzamiento comercial.

Son plazos externos que no se pueden comprimir después:

| Trámite | Plazo típico | Bloquea |
|---|---|---|
| Certificado OV/EV de Windows | **1–3 semanas** | Instalación sin advertencia de SmartScreen |
| Apple Developer ID + notarización | Alta + trámite por build | **Que la aplicación abra en macOS** |
| Verificación OAuth de Google (D-1, P-05) | **4–8 semanas** | `GoogleProvider` en v1.2.x |

Descubrir cualquiera de estos al final del desarrollo congela el release con el producto terminado. **Riesgo R-14.**

---

## 2. Artefactos

| Plataforma | Formato | Notas |
|---|---|---|
| Windows x64 | `.msi` (WiX) + `.exe` (NSIS) | MSI para despliegue corporativo por directiva de grupo; NSIS para instalación manual |
| macOS universal | `.dmg` | Binario universal (Apple Silicon + Intel) en un solo artefacto |

Nombres: `ArlesRelay-1.2.0-x64.msi`, `ArlesRelay-1.2.0-universal.dmg`.

**Sin sufijos informales** (§3). Nunca `final`, `nuevo`, `v2`, `def`.

---

## 3. Firma en Windows

### Por qué no es opcional

Sin firma, **SmartScreen muestra una advertencia de «editor desconocido»** que en la práctica bloquea la instalación en un entorno corporativo: el usuario tiene que pulsar «Más información» y luego «Ejecutar de todos modos», y la mayoría no lo hace. Un departamento de TI directamente lo prohíbe.

### OV frente a EV

| | OV | EV |
|---|---|---|
| Coste | Menor | Mayor |
| Reputación en SmartScreen | **Se acumula con el tiempo** | **Inmediata** |
| Almacenamiento de la clave | HSM o token (desde 2023) | Token físico o HSM en la nube |

**Recomendación: OV para v1.2.0, EV para el lanzamiento comercial.**

El razonamiento: con D-4 (despliegue interno) la reputación de SmartScreen no importa — son máquinas de TELEMETRY donde TI puede aprobar el instalador. Cuando llegue v1.3 y el producto salga a clientes, la reputación acumulada por el certificado OV ya habrá empezado a construirse, y ahí sí conviene EV para que el primer cliente no vea una advertencia.

> **Nota:** desde junio de 2023 todos los certificados de firma de código requieren almacenamiento de la clave en hardware certificado. Esto afecta al pipeline de CI: hay que usar un servicio de firma en la nube o un agente autoalojado con el token. **Conviene resolverlo al contratar el certificado, no al configurar el CI.**

---

## 4. Firma y notarización en macOS

### Sin notarizar, Gatekeeper no abre la aplicación

No es una advertencia: es un bloqueo. Desde macOS Catalina, una aplicación descargada sin notarizar simplemente no se ejecuta.

### Pasos

1. **Apple Developer Program** (cuenta de organización).
2. **Developer ID Application** para firmar el binario y **Developer ID Installer** para el `.dmg`.
3. **Hardened Runtime** activado.
4. **Notarización**: se envía a Apple, se espera el resultado, y se **grapa** el ticket al `.dmg` (`xcrun stapler staple`). El grapado es lo que permite que Gatekeeper verifique sin conexión.

### Entitlements

**Mínimos.** ARLES necesita red saliente y acceso al llavero. Nada más.

No se activa el sandbox de App Store: ARLES no se distribuye por la App Store, y el sandbox complicaría el acceso al llavero sin aportar nada en este modelo de distribución.

---

## 5. Actualizaciones (§79)

Plugin updater de Tauri. Verificación de **firma criptográfica antes de aplicar** cualquier actualización.

### Custodia de claves (§81)

| Clave | Dónde |
|---|---|
| **Privada de firma del updater** | **Secretos del CI.** Nunca en el repositorio, ni cifrada, ni en una rama privada |
| Pública | Empaquetada con la aplicación |

**Procedimiento de rotación escrito antes de necesitarlo.** Si la privada se compromete, hay que publicar una versión con la nueva pública y comunicarlo — y eso no se improvisa durante un incidente.

### Comportamiento

Comprobación al arrancar y cada 24 h. **Nunca se actualiza automáticamente sin avisar**: el usuario decide cuándo.

**Y una regla propia de ARLES:**

> **Con una campaña en ejecución, no se propone actualizar.**

Actualizar reinicia la aplicación, y aunque el motor reanuda correctamente, interrumpir una campaña activa para una actualización no urgente es una mala decisión de producto. Se espera a que termine o a que el usuario la pause.

### Canal

En v1.2.0, con D-4, la distribución es interna: un servidor de TELEMETRY o incluso una carpeta compartida. La infraestructura pública de actualizaciones entra con v1.3.

---

## 6. Versionado (§3)

**SemVer** `MAYOR.MENOR.PARCHE`.

| Componente | Se incrementa cuando |
|---|---|
| MAYOR | Cambio incompatible: migración irreversible, cambio de formato de respaldo |
| MENOR | Funcionalidad nueva compatible |
| PARCHE | Correcciones |

`CHANGELOG.md` en la raíz, formato Keep a Changelog, con: versión, fecha, añadido, cambiado, corregido, **seguridad**, migraciones, cambios incompatibles, build y commit.

**La aplicación muestra su versión** en el pie y en «Acerca de». Con el numeral «I» separado del número (ADR-0010).

---

## 7. Pipeline de release

```
1. Tag vX.Y.Z en la rama de release
2. CI completo (lint, tests, seguridad, contraste, rendimiento, E2E)
3. Build Windows  → firma OV/EV
4. Build macOS    → firma Developer ID → notarización → grapado
5. Firma del manifiesto del updater
6. Generación de checksums SHA-256
7. Publicación + CHANGELOG
```

**Ningún artefacto se publica sin pasar el CI completo.** No hay build manual de emergencia: una build sin tests es exactamente la que rompe una instalación de cliente.

---

## 8. Instalación

### Windows
Por usuario por defecto (sin exigir administrador), con opción de instalación por máquina para despliegue corporativo. El MSI admite instalación silenciosa (`/qn`) para directivas de grupo.

### macOS
`.dmg` con arrastrar a Aplicaciones. El patrón que todo el mundo conoce.

### Desinstalación
**Limpia y honesta:** se eliminan la aplicación y sus entradas del llavero. **Los datos del usuario se conservan** y se le dice dónde están, con la opción explícita de eliminarlos.

Borrar silenciosamente la base de datos de contactos de alguien al desinstalar sería indefendible.

---

## 9. Lo que no se hace en v1.2.0

| No se hace | Por qué |
|---|---|
| Distribución por Microsoft Store o App Store | Sandbox y proceso de revisión; no aporta con D-4 |
| Instalador para Linux | §4 — sólo Windows y macOS |
| Telemetría de uso | No hay datos que salgan de la máquina del cliente |
| Actualizaciones automáticas silenciosas | El usuario decide |
| Canal beta público | D-4 |
