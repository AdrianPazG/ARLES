# Modelo de amenazas — preliminar

Fase 1. Debe revisarse antes de cada hito y actualizarse cuando cambie una frontera.

Referencias: OWASP ASVS, OWASP Top 10, Secure Software Development Framework.
**ARLES no afirma ser "100 % seguro"** (§83). Se diseña defensa en profundidad y se documentan los límites.

## Fronteras de confianza

| # | Frontera | Amenaza principal | Control |
|---|---|---|---|
| 1 | Webview ↔ núcleo Rust | Una vista comprometida invoca comandos privilegiados | Capacidades de Tauri 2 por ventana. CSP estricta sin `unsafe-inline`. Ninguna API de sistema global. Cada comando valida sus argumentos en Rust |
| 2 | Aplicación ↔ llavero del SO | Robo de token por malware local | `keyring`. Ver límite honesto abajo. Reducción de daño: scopes mínimos, tokens de vida corta, desconexión que revoca |
| 3 | Aplicación ↔ proveedores | Intercepción de OAuth, redirección maliciosa | Authorization Code + **PKCE obligatorio**. Sin *client secret* incrustado. Navegador del sistema, nunca webview embebido. Redirección a `127.0.0.1` con puerto efímero y verificación de `state` |
| 4 | Aplicación ↔ servidor de actualización | Instalar un paquete manipulado | Firma verificada **antes** de ejecutar nada. TLS validado. Versión monótona. Clave privada en el gestor de secretos de CI, jamás en el repositorio |
| 5 | Aplicación ↔ archivos importados | XLSX malicioso, bomba de descompresión, inyección de fórmulas | Ver sección propia |
| 6 | Aplicación ↔ respaldos | Robo del archivo, restauración manipulada | Cifrado autenticado con frase del usuario. Credenciales excluidas. Manifiesto verificado antes de restaurar |

## Límite honesto del llavero del sistema

El Llavero de macOS y el Administrador de Credenciales de Windows protegen un secreto frente a **otros
usuarios del equipo** y frente a quien se lleve el disco. **No protegen frente a código que se ejecuta con
la misma cuenta de usuario.** Si hay malware corriendo como la persona que usa ARLES, ese malware puede
pedirle al sistema el mismo secreto que le pedimos nosotros.

Esto es cierto para toda aplicación de escritorio sin excepción. Se documenta aquí y debe reflejarse en
`SECURITY.md` porque la alternativa es que el material comercial insinúe una protección de grado bóveda que
no existe.

Lo que sí se controla es el tamaño del daño: scope mínimo (solo envío), cero contraseñas almacenadas, y
desconexión que revoca de verdad.

**Decisión relacionada:** no se cifra la base completa en v1.2.0. Se apoya en el cifrado de disco del
sistema (FileVault / BitLocker), se comprueba su estado en Diagnóstico y se recomienda activarlo. Cifrar la
base con una clave que vive en el mismo equipo añade complejidad de respaldo y herramientas sin cambiar el
modelo de atacante.

## Vector específico: inyección de encabezado por variable de personalización

**El hallazgo más importante de esta fase.**

§41 permite `{{first_name}}` en el mensaje y §40 define un *Asunto*. §88 pide prevenir inyección de
encabezados y CRLF. Nadie conectó los dos puntos: **si una variable se interpola en el Asunto, y el Asunto
es un encabezado, entonces un contacto cuyo nombre contenga `\r\nBcc: victima@dominio.com` inyecta un
encabezado real.**

El dato viene de un XLSX que el cliente recibió de un tercero: es entrada no confiable que llega hasta la
construcción del MIME. Es la vulnerabilidad más probable del producto porque cruza tres módulos que
distintas personas escribirán en distintas fases.

**Mitigación, aplicada en el núcleo de Rust y no en el frontend:** todo valor interpolado en un encabezado
pasa por un normalizador que elimina CR, LF, NUL y caracteres de control, y trunca a la longitud del
encabezado. Un test unitario con la carga de inyección forma parte de la Definición de Terminado. Donde sea
posible, la construcción del MIME se delega a `lettre` en lugar de concatenar cadenas (§88).

## Importación de archivos

Única ruta por la que entra un archivo arbitrario de origen desconocido. Controles en orden de ejecución:

1. **Topes duros antes de parsear** — tamaño, filas, columnas, longitud de celda, ratio de descompresión.
   Superar cualquiera **aborta**; no se trunca en silencio. Un XLSX es un ZIP: sin tope de ratio, una bomba
   de descompresión agota la RAM.
2. **Nunca evaluar fórmulas.** Se lee el valor almacenado. Una celda que empieza con `=` es texto, siempre.
3. **Escape de fórmulas al exportar** — toda celda que empiece con `=` `+` `−` `@`, tabulador o retorno de
   carro se prefija. Si no, exportamos un archivo que ejecuta código al abrirse en Excel (§147).
4. **Nombres de archivo neutralizados.** Nunca se usa el nombre proporcionado para construir una ruta.
5. **Normalización Unicode** a NFC; rechazo de anulaciones bidireccionales, que permiten que un correo se
   vea distinto de lo que es.
6. **Fuera del hilo de interfaz**, con progreso y cancelación.

## Vista previa del mensaje

La vista previa muestra el mensaje con las variables ya resueltas —datos de origen desconocido— dentro de
nuestra propia aplicación. Es la ruta de XSS más directa del producto.

Controles: renderizado en `<iframe sandbox>` sin `allow-scripts` y con CSP propia. Saneado por **lista
blanca** de etiquetas y atributos, no lista negra. `href` limitado a `http`, `https` y `mailto`: nada de
`javascript:` ni `data:`. **El saneado ocurre en el núcleo de Rust**, no en el frontend, porque el frontend
es justamente lo que se intenta comprometer.

## Registros y bitácora

**Prohibido registrar:** contraseñas, tokens de acceso, tokens de refresco, credenciales SMTP, cuerpos
completos de mensajes, direcciones de correo en nivel `info`. Los contactos se registran por UUID.

El paquete de soporte se genera con un saneador que corre sobre el resultado, y **la persona ve el
contenido antes de enviarlo** (§152).

La bitácora de auditoría es distinta de los registros: vive en la base, es solo-inserción, y registra quién,
cuándo y qué en las 14 acciones críticas del §90 —incluida la aceptación de la advertencia de volumen del
§48, que es lo que protege a TELEMETRY si un cliente reclama después.

## Uso indebido del producto

Postura: **detectar y advertir, nunca facilitar la evasión.**

**Se construye:** detección de tasas de error anómalas; alerta por proporción alta de rechazos; advertencia
por listas con muchos dominios de un solo uso; advertencia de volumen desproporcionado frente al histórico;
bitácora de todo cambio de límites.

**No se construye, y se documenta el porqué:** rotación automática de cuentas para repartir volumen;
variación de contenido para evadir filtros; encabezados falsificados; suplantación de remitente; imitación
de tiempos humanos; cualquier ayuda para superar un límite impuesto por el proveedor.

## Cadena de suministro

Auditoría de dependencias y escaneo de secretos en cada cambio, no al final. Se evitan paquetes abandonados
y bibliotecas desconocidas para funciones sensibles. SBOM cuando sea práctico.

## Pendiente de verificación

- Clasificación vigente del scope de envío de Gmail y sus requisitos de verificación — **NO VERIFICADO**,
  debe confirmarse contra la documentación de Google antes de comprometer fecha.
- Marco normativo de protección de datos personales aplicable en México — **NO VERIFICADO**, requiere
  abogado especialista.
