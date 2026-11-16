# ADR-0011 · Cifrado en reposo y custodia de la clave maestra

**Estado:** aceptado · **Fecha:** 2026-09-11 · **Decide:** Security (T-3)

---

## Contexto

T-3 exige cifrado de la base de datos en reposo para proteger datos personales bajo la legislación mexicana (LFPDPPP, T-8). El §30 prohíbe almacenar credenciales en texto plano, JSON, `localStorage`, registros o tablas SQLite sin cifrar, y exige almacenamiento seguro del sistema operativo.

ARLES custodia dos clases de secreto muy distintas:

1. **Credenciales de correo** — contraseñas de aplicación SMTP, y en v1.2.x tokens OAuth.
2. **La clave maestra de la base de datos** — la que cifra todo lo demás.

## Decisión

### Dos almacenes, ninguno es la base de datos

**Credenciales de correo → llavero del sistema operativo**, vía `keyring-rs`: Keychain en macOS, Credential Manager en Windows.

La tabla `email_account` guarda `credential_ref`, un identificador opaco. **Nunca un secreto.**

**Clave maestra → el mismo llavero.** 32 bytes de un generador criptográficamente seguro (`OsRng`), generados en el primer arranque, bajo una entrada propia.

### Si el llavero no está disponible, la aplicación no arranca

Sin degradación a texto plano. Sin «modo compatibilidad». Sin archivo de claves de respaldo.

El mensaje de error explica qué pasó y cómo resolverlo, siguiendo el §95.

### `Secret<T>` en código

```rust
pub struct Secret<T>(T);

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[REDACTADO]")
    }
}
// Display idéntico. Sin `impl Deref`: exponer exige llamar a expose_secret().
```

Más una **capa de redacción obligatoria** en `tracing`. Las dos cosas: el tipo protege del formateo accidental, la capa protege de lo que el tipo no cubra.

### Apertura de la base de datos

```sql
PRAGMA key = <32 bytes>;   -- PRIMERO. Antes de cualquier otra sentencia.
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
```

`PRAGMA key` va antes que nada. Si se ejecuta cualquier sentencia previa, SQLCipher falla con un error que no es evidente.

## Justificación

### Por qué negarse a arrancar

Es la decisión menos cómoda del documento y la más importante.

La alternativa —«si no hay llavero, guarda la clave en un archivo»— convierte el cifrado en **teatro**. Un atacante con acceso al sistema de archivos obtiene la base de datos y la clave del mismo directorio. El cifrado deja de proteger nada y sólo aporta la sensación de que protege.

Y el fallo es silencioso: nadie se entera de que su instalación está en modo degradado.

El §85 dice: priorizar la seguridad sobre la conveniencia. Esto es exactamente ese caso, y el coste es real — un usuario con el llavero roto no puede usar la aplicación hasta arreglarlo. Lo aceptamos.

### Por qué el llavero y no una contraseña del usuario

Una contraseña maestra sería criptográficamente más fuerte —la clave no estaría disponible sin la intervención humana— pero **rompe el motor de ejecución en segundo plano**. ARLES debe poder reanudar una campaña tras un reinicio del equipo sin que haya nadie delante para teclear nada. Es un requisito duro del §51.

El llavero da persistencia sin intervención, con el control de acceso del sistema operativo.

## Consecuencias

**Positivas.** Ningún secreto en disco fuera del llavero · cumplimiento de T-3 y §30 · el motor puede reanudar sin intervención humana · sin secretos en registros, por tipo y por capa.

**Negativas, y una es seria.**

1. **Si se pierde la clave del llavero, la base de datos es irrecuperable.** No hay puerta trasera — y esa es exactamente la propiedad que se buscaba. Ocurre al reinstalar el sistema, cambiar de equipo o corromperse el perfil de usuario.

   **Riesgo R-10.** Mitigaciones:
   - La interfaz lo advierte **en el primer arranque**, no en un manual que nadie lee.
   - El respaldo `.arles` debe permitir restaurar en una máquina nueva.
   - ⚠️ **Pero el cifrado de respaldos está diferido a v1.3 (T-6)**, así que en v1.2.0 el respaldo es el único camino de recuperación **y es un archivo sin cifrar con datos personales**. Hay que decírselo al usuario para que lo custodie en consecuencia.

2. **Sobrecoste de SQLCipher:** 5–15 %. No negociable.

3. **Fricción entre plataformas.** Al migrar de Windows a macOS, las credenciales no viajan: el llavero es local por diseño. El §75 lo contempla — la restauración debe **detectarlo con elegancia** y pedir al usuario que reconecte sus cuentas, no fallar de forma opaca.

## Alternativas descartadas

**Clave derivada de una contraseña del usuario (Argon2id).** Criptográficamente superior, pero impide el arranque desatendido del motor. Incompatible con §51.

**Clave en un archivo con permisos restrictivos.** Es el cifrado como teatro. Rechazado.

**Clave derivada de identificadores de hardware.** Frágil —cambia con una actualización de firmware o un cambio de disco— y no aporta seguridad real: un atacante con acceso al sistema también los tiene.

**Sin cifrado.** Contradice T-3 y debilita la posición frente a la LFPDPPP.

## Nota para v1.3

Al implementar el cifrado de respaldos, la clave **no puede ser la del llavero**: un respaldo que sólo se puede restaurar en la máquina que lo generó no es un respaldo. Lo correcto es una frase de paso elegida por el usuario, con derivación Argon2id, aplicada **sólo al archivo de respaldo**. Ahí sí hay una persona delante tecleando, así que la objeción del arranque desatendido no aplica.
