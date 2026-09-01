# ADR-0004 — Motor de ejecución en el mismo proceso

**Estado:** Propuesto · **Fecha:** 2026-09-01

## Contexto
§51 pide que el motor pueda seguir ejecutando con la ventana minimizada. La lectura intuitiva es un demonio
separado. §108 exige además probar "SQLite bloqueado" y "dos procesos intentan enviar el mismo mensaje".

## Decisión
El motor es una **tarea asíncrona dentro del núcleo de Rust del proceso de la aplicación**, con la **única
conexión de escritura** del sistema. Cerrar la ventana oculta la interfaz; la aplicación permanece en la
bandeja de Windows o la barra de menú de macOS. Salir de verdad detiene el motor, y ARLES lo dice
explícitamente si hay campaña activa.

## Alternativas
- **Demonio o servicio separado:** dos procesos sobre un mismo SQLite producen contención de escritura y
  abren la puerta al peor escenario del brief. Además duplica instalación, actualización, firma y depuración.
- **Ejecución solo con la ventana abierta:** incumple §51.

## Consecuencias

**Positivas.** *"Dos procesos enviando el mismo mensaje"* deja de ser un caso de prueba y pasa a ser
imposible. Un solo binario que firmar, actualizar y depurar. Sin protocolo de comunicación entre procesos
que asegurar.

**Negativas.** Si el usuario sale de la aplicación, el motor se detiene. Se considera correcto y honesto:
§53 establece que ARLES no ejecuta con el equipo apagado y §52 que nunca debe ocultar si está ejecutando.
La interfaz debe advertirlo con claridad al salir con una campaña activa.
