# Cómo se trabaja en ARLES

**Última actualización:** 2026-10-27

Cuatro cosas, y ninguna es ceremonia: las cuatro están escritas porque su
ausencia costó algo concreto.

---

## 1 · La regla de la ruptura

> **Una comprobación que nunca se ha visto fallar no está verificada.**

Cada garantía que se afirme —en una prueba, en el validador, en una sonda— se
rompe a propósito para ver si la comprobación la caza. Si no la caza, la
comprobación no vale y hay que arreglarla **antes** de seguir.

### El procedimiento, y no se improvisa

1. **Copia el archivo antes de la primera ruptura.** No es opcional y no es
   `git checkout`: un `git checkout` sobre un archivo con trabajo sin confirmar
   revierte el archivo **entero**. Ha costado trabajo rehecho dos veces —la
   8.3 bis y D-16—, y saberlo no bastó: la segunda vez fue después de escribir
   la lección.
2. **Comprueba que la ruptura se aplicó.** Que el archivo cambió, de verdad:
   un escapado del *shell* se comió una sustitución una vez y el verde no
   significaba nada.
3. **Comprueba que compila.** Una ruptura que no compila no prueba nada: lo que
   hay que ver caer es la comprobación, no el compilador.
4. **Nombra la prueba que tiene que fallar**, y comprueba que falla **ésa**.
   Buscar «algún rojo» da por cazada una ruptura que tumbó otra cosa.
5. **Restaura desde la copia** y compara el archivo **entero**, byte a byte.
   Nunca con un `grep` de un identificador: casa el comentario que explica la
   ruptura. Pasó, y estuve trabajando sobre un archivo roto creyéndolo bueno.
6. **Déjalo escrito en el comentario de la prueba**: qué se rompió y qué se vio.

### Lo que una ruptura mal elegida parece

- **Que deja el síntoma en pie no prueba nada.** Renombrar la declaración de una
  constante deja el identificador en sus usos.
- **Que no deja síntoma visible tampoco.** Un margen negativo en un aviso que
  sólo se pinta al escribir una dirección: la sonda salió en verde sobre código
  roto.
- **Que apunta a la prueba equivocada da un falso hallazgo propio.** Pasó con el
  WABA del número: la garantía sí estaba cazada, por otra prueba.

### Las nueve veces que una comprobación se cazó a sí misma

Buscaba una cadena en un archivo y la encontraba **en su propio comentario**, o
en la cabecera que explica por qué existe. Pasó en la 5.3, la 8.5, dos veces en
la 8.3 bis, en la 8.6 A, en la 8.6 D, en la 8.6 E, en la 8.6 F y en la guía de
arranque.

> **Conclusión, que ya no es una anécdota: saberlo no basta.** Se quitan los
> comentarios **siempre, por defecto**, en cualquier comprobación que busque una
> cadena en un archivo. Y al buscar en Markdown, **los bloques citados son
> bloques**: `^\s*cargo run` no casa `> cargo run`.

---

## 2 · Un tramo, y se pregunta

Dirección lo ha pedido muchas veces, y es la forma del trabajo:

    un tramo → se verifica → se empuja → se explica → se pregunta

**Un tramo es lo que se puede verificar entero.** No «un archivo» ni «un día»:
lo que tiene una garantía que se puede romper y ver caer.

Y se explica en los términos de Dirección, que no es programador. Al cuestionar
una decisión, con esta estructura y no con una opinión suelta:

    Problema → Riesgo → Impacto → Alternativa → Trade-off → Recomendación

Al informar: si algo falla, se enseña la salida; si un paso se saltó, se dice.
Los porcentajes llevan fecha. Las cifras son cifras y no adjetivos.

---

## 3 · La verificación, en orden y completa

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings   # cero advertencias
cargo test --workspace
cargo deny check licenses advisories                    # antes de añadir dependencias

cd app && npm test && npm run lint && npx vue-tsc --noEmit -p tsconfig.json

# Y ANTES DE EMPUJAR, las diez. Cuesta unos ocho minutos:
for f in 0 1 2 3 4 5 6 W 7 8; do python3 herramientas/validar/validar.py --fase $f --rapido; done
```

**Correr sólo la fase en curso no basta.** En la 5.2 y la 5.3 se verificó con
`--fase 5`, que pasaba entera, y se rompieron cuatro comprobaciones de las
fases 2, 3 y 4 sin que nadie lo viera. Y vuelve a pasar: el tramo 3a de D-WA
rompió una comprobación de las fases 3, 4 y 5 —ninguna de las que tocaba—.

**Las cifras que se publican salen de una corrida secuencial**, de una en una.
Tres validadores a la vez dieron una vez un 52/1 que no existía, y la salida no
se guardó: **una corrida que no se guarda no se puede diagnosticar.**

### Y antes de cerrar: se mira

Las sondas miden contrastes, focos, desbordes y textos. **Nada de eso caza un
dibujo que miente ni una pantalla que empieza en otro sitio.** Antes de cerrar
una entrega se abre la demostración y se mira, pantalla por pantalla. Las
revisiones a ojo llevan encontradas más de veinte cosas con todas las sondas en
verde.

```bash
python3 herramientas/vista-previa/generar.py   # deja vista-previa/ARLES-vista-previa.html
```

---

## 4 · Los poka-yokes

> **Un poka-yoke es una comprobación que hace imposible el error, no un aviso
> de que ocurrió.**

Cuando algo falla, la pregunta no es sólo «¿cómo lo arreglo?» sino **«¿qué
comprobación habría hecho imposible esto?»** — y se escribe. Es la diferencia
entre este proyecto y uno que repite sus fallos.

### Los tres sitios donde vive uno

| Sitio | Cuándo | Ejemplo |
|---|---|---|
| **El tipo** | Cuando el estado imposible se puede dejar de representar | `PorDondeSale` no permite una etapa de WhatsApp con plantilla de correo |
| **El validador** | Cuando es una regla del proyecto que se puede leer del código | `ningun_comando_se_queda_sin_quien_lo_llame` |
| **La prueba** | Cuando es una garantía del comportamiento | `la_cascada_calculada_es_la_que_borra_sqlite` |

**El tipo gana siempre que se pueda.** Una variante con el dato dentro
—`Cifrado::Argon2idXChaCha20 { sal, nonce }`, `RecortadaEnElEsencial { fecha }`,
`CuentaDeLaEtapa`— no necesita que nadie se acuerde, y además **obliga al
compilador a exigir una decisión** en cada sitio que daba lo anterior por
supuesto. Un `_ =>` ahí anula la protección entera.

### Al escribir un poka-yoke, los cinco fallos propios conocidos

1. **La lista a mano.** Once de los diecisiete hallazgos de las cuatro
   auditorías eran esto: una comprobación que decía cubrir una regla y cubría
   una parte. Se lee del código, del esquema o de la carpeta — nunca de una
   lista que alguien mantiene.
2. **El alcance corto.** `nada_que_cruce_la_frontera_lleva_contrasena` leía un
   módulo de once.
3. **La hoja y no la ruta.** Comparar el nombre final de una clave y no su ruta
   entera dejó pasar una clave guardada en la rama equivocada.
4. **El comentario que se caza a sí mismo.** Nueve veces. Ver arriba.
5. **Escribir en el repositorio para medirlo.** Da rojo sobre código correcto
   bajo concurrencia. Se genera a un temporal y se comparan contenidos.

---

## 5 · Al terminar: lo que no se olvida

Antes de empujar, estas cuatro:

- [ ] **¿Quién lo llama?** La pieza nueva tiene un llamador de verdad, no una
      prueba. Si no lo tiene todavía, **no se registra** y queda dicho dónde.
- [ ] **¿Qué documento deja de ser verdad?** El estado, el índice, el manual, el
      mapa de pantallas. Un documento que afirma de más es peor que no tenerlo.
- [ ] **¿Qué poka-yoke hace falta?** Y se verifica rompiéndolo.
- [ ] **¿Está todo en su carpeta?** Ver [`DONDE-VA-CADA-COSA.md`](DONDE-VA-CADA-COSA.md).
