# Modelo de secretos

**Proyecto:** ARLES RELAY I · v1.2.0
**Decisión base:** ADR-0011

---

## 1. La regla

**Ningún secreto toca la base de datos, la configuración, `localStorage`, los registros ni los respaldos.**

Sin excepciones, sin modo compatibilidad, sin casilla avanzada.

---

## 2. Inventario de secretos

| Secreto | Dónde vive | Quién lo lee |
|---|---|---|
| Contraseña de aplicación SMTP | Llavero del SO | `arles-email` en el momento del envío |
| Token OAuth de acceso (v1.2.x) | Llavero del SO | `arles-email` |
| **Token OAuth de refresco** (v1.2.x) | Llavero del SO | `arles-email` al renovar |
| Clave maestra de SQLCipher | Llavero del SO | `arles-db` al abrir |
| Clave privada de firma del updater | **Secretos del CI** | Sólo el pipeline de release |

El *refresh token* merece mención aparte: es el secreto de mayor valor, porque no caduca y permite regenerar accesos indefinidamente. Nunca toca la base de datos.

---

## 3. El llavero

`keyring-rs` sobre los almacenes nativos:

| Plataforma | Almacén | Control de acceso |
|---|---|---|
| macOS | Keychain | Por aplicación, respaldado por el sistema |
| Windows | Credential Manager | Por cuenta de usuario |

Convención de nombres:

```
servicio:  com.telemetryinsight.arlesrelay
cuenta:    db-master-key
           email-account:<uuid>
```

### Si el llavero no está disponible, la aplicación no arranca

Esta es la decisión menos cómoda del modelo y la más importante.

La alternativa —«si no hay llavero, guarda la clave en un archivo»— convierte el cifrado en **teatro**: un atacante con acceso al sistema de archivos obtiene la base de datos y su clave del mismo directorio. El cifrado deja de proteger y sólo aporta la sensación de protección. Y el fallo es **silencioso**: nadie se entera de que su instalación está degradada.

El §85 dice priorizar la seguridad sobre la conveniencia. Es exactamente este caso, y el coste es real: un usuario con el llavero roto no puede usar ARLES hasta arreglarlo. Lo aceptamos.

El mensaje de error sigue el §95 — qué pasó, cómo arreglarlo, qué está a salvo:

> **No se puede acceder al almacén de credenciales del sistema.**
> ARLES necesita el Llavero de macOS para proteger tus datos. Sin él no puede abrir la base de datos cifrada.
> **Qué hacer:** abre Acceso a Llaveros y verifica que el llavero «Inicio de sesión» esté desbloqueado.
> **Tus datos están intactos.** La base de datos no se ha modificado.

---

## 4. `Secret<T>` en código

```rust
pub struct Secret<T>(T);

impl<T> Secret<T> {
    pub fn new(v: T) -> Self { Secret(v) }
    pub fn expose_secret(&self) -> &T { &self.0 }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[REDACTADO]")
    }
}
// Display idéntico.
```

Tres propiedades deliberadas:

- **Sin `impl Deref`.** Exponer el valor exige llamar a `expose_secret()`, que es visible en revisión de código y buscable con grep.
- **`Debug` redactado.** Un `tracing::debug!("{:?}", account)` no puede filtrar nada.
- **Se limpia al soltarse** (`zeroize`), para reducir la ventana en que el secreto queda en memoria liberada.

### Y además, la capa de redacción en `tracing`

`Secret<T>` protege de lo que pasa por él. La capa de redacción protege de lo que no. Las dos cosas, no una: una defensa depende de que todo esté envuelto correctamente, y esa suposición se rompe con el tiempo.

---

## 5. La frontera IPC

**Ningún comando Tauri devuelve un secreto.** Ni cifrado, ni truncado, ni «sólo para mostrar los últimos cuatro caracteres».

El frontend recibe:

```typescript
interface EmailAccountView {
  id: string;
  displayName: string;
  emailAddress: string;
  providerKind: 'smtp' | 'google';
  status: 'connected' | 'needs_reauth' | 'circuit_open';
  // Sin credencial. Sin token. Sin referencia al llavero.
}
```

Cuando el usuario introduce una contraseña de aplicación, viaja **del frontend a Rust una sola vez**, va directa al llavero, y no vuelve nunca. La interfaz muestra a partir de ahí sólo el estado.

---

## 6. Ciclo de vida

**Alta.** El usuario introduce la credencial → se valida con «Probar conexión» → si funciona, se guarda en el llavero y se persiste `credential_ref`. **Si no funciona, no se guarda nada**: una credencial rota almacenada sólo produce fallos confusos después.

**Uso.** Se lee del llavero en el momento del envío. No se cachea en memoria más allá de la operación.

**Renovación (OAuth, v1.2.x).** Al caducar el token de acceso se usa el de refresco. Si el refresco falla —revocado, caducado—, la cuenta pasa a `needs_reauth`, se abre el circuito y se avisa.

**Baja.** Al desconectar una cuenta, **se borra la entrada del llavero**, no sólo la fila de la base de datos. Y se registra en `audit_log`.

---

## 7. Migración entre equipos (§75)

El llavero es **local por diseño**. Un respaldo `.arles` restaurado en otra máquina **no trae las credenciales**, y eso es correcto: un respaldo que llevara credenciales sería un archivo con acceso a las cuentas de correo del cliente.

La restauración debe detectarlo con elegancia:

> **Respaldo restaurado correctamente.**
> Tus contactos, campañas y plantillas están disponibles.
> **3 cuentas de correo necesitan reconectarse.** Por seguridad, las credenciales no se incluyen en los respaldos y se guardan en el almacén de tu sistema.

Las cuentas aparecen en estado `needs_reauth` con un botón para reconectar. **No se falla de forma opaca ni se muestra un error técnico.**

---

## 8. Claves de firma del updater

Viven en los **secretos del sistema de CI**. Nunca en el repositorio, ni siquiera cifradas, ni siquiera en una rama privada (§81).

La **clave pública** se empaqueta con la aplicación y se usa para verificar cada actualización antes de aplicarla (§79).

Rotación: si la privada se compromete, hay que publicar una versión con la nueva pública y comunicarlo. Conviene tener ese procedimiento escrito **antes** de necesitarlo.

---

## 9. Lo que NO hacemos

| No hacemos | Por qué |
|---|---|
| Cifrar credenciales con una clave del binario | Ofuscación, no cifrado. La clave está en el binario |
| Guardar credenciales en `localStorage` | §30, y es accesible desde la webview |
| Enviar credenciales por IPC para mostrarlas | No hay razón legítima |
| Un «modo sin llavero» | Cifrado como teatro (§3) |
| Incluir credenciales en los respaldos | Un respaldo se copia, se envía por correo, se sube a la nube |
| Registrar credenciales «sólo en depuración» | Las compilaciones de depuración se distribuyen por accidente |
