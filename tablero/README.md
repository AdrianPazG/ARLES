# Tablero de ARLES

**Abre `ARLES-tablero.html` con doble clic.** No hace falta instalar nada ni
levantar ningún servidor: es un archivo suelto, igual que la vista previa.

Enseña, de un vistazo:

- **el avance**, con una barra por fase y el porcentaje total;
- **las comprobaciones** del validador, fase a fase, con las que fallan y las
  que no se midieron en columnas distintas;
- **los hallazgos** de las pruebas, con su gravedad en color y su estado;
- **lo que espera a alguien de fuera** —licencia, abogado, un Mac, un servidor
  de correo real—, con quién lo tiene en la mano;
- **los tropiezos de instalación** que ya nos han pasado y tienen solución
  escrita.

---

## Para rehacerlo

```bash
python3 herramientas/tablero/generar.py --medir     # corre las diez fases
python3 herramientas/tablero/generar.py             # usa la última medición
```

Con `--medir` tarda unos minutos, porque corre el validador de verdad. Sin
`--medir` es instantáneo y **el tablero enseña de cuándo es la medición** — y
la pinta en amarillo si pasó de una semana.

---

## Por qué este tablero no puede mentir

Aquí no hay **ni una cifra escrita a mano**. Todas salen de donde ya viven:

| Qué | De dónde |
|---|---|
| Porcentaje, fases, bloqueos | `documentacion/07-entrega/avance.json` |
| Hallazgos de las pruebas | La tabla de `documentacion/06-calidad/BITACORA-DE-PRUEBAS.md` |
| Tropiezos | Los títulos de `documentacion/07-entrega/REGISTRO-DE-TROPIEZOS.md` |
| Comprobaciones | Una corrida de verdad de `validar.py`, guardada en `corrida.json` |

La razón no es elegancia. El `CLAUDE.md` §9 estuvo diciendo «entrega 3.3» con
la Fase 4 cerrada y la 5 a medias, y lo cuenta así: *«un archivo que se lee al
abrir el proyecto y que miente sobre el estado es peor que no tenerlo, porque
se cree»*. Un tablero es exactamente eso **pero con colores**, que dan más
confianza todavía.

Y por si alguien cambia `avance.json` sin regenerar, el validador compara los
dos: `el avance del tablero coincide con la fuente`, en la fase 3. Verificada
poniéndole al tablero un 31 % a mano — falla diciendo «tablero 31 vs calculado
89».

---

## Qué NO está aquí

- **Lo que ARLES puede hacer hoy** — eso es el
  [manual de usuario](../documentacion/07-entrega/MANUAL-DE-USUARIO.md).
- **Qué hace cada pantalla por dentro** — eso es el
  [mapa funcional](../documentacion/05-diseno/MAPA-FUNCIONAL.md).
- **Un porcentaje dentro de la aplicación.** Es un dato del proyecto, no del
  producto, y Dirección decidió el 17/09/2026 que va sólo en el repositorio.

---

## Los dos archivos

| | |
|---|---|
| `ARLES-tablero.html` | Lo que se abre. Se **genera**: no se edita a mano |
| `corrida.json` | La última medición de las diez fases, con su fecha y su hora |
