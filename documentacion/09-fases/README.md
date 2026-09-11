# Fases

Una carpeta por fase del [roadmap](../07-entrega/ROADMAP.md). Cada una responde a tres preguntas: **qué se construyó**, **cómo se comprobó** y **qué quedó pendiente**.

Existen porque los ADR explican *por qué* se decidió algo y el código muestra *qué* quedó, pero nadie guarda el registro de **qué se verificó de verdad y qué se dio por bueno sin comprobar**. Esa distinción es la que importa cuando alguien pregunta, seis meses después, si algo está probado.

---

## Estado

| Fase | Nombre | Estado | Comprobaciones |
|---|---|---|---|
| [00](FASE-00-DISCOVERY.md) | Discovery, auditoría y arquitectura | ✅ cerrada | 4 / 4 |
| [01](FASE-01-CIMIENTOS.md) | Cimientos | ✅ cerrada y revisada | 32 / 32 |
| [02](FASE-02-DESIGN-SYSTEM.md) | Design System | ✅ cerrada y revisada | 18 / 18 |
| 03 | Empresa y contactos | ⬜ pendiente | — |
| 04 | Motor de ejecución | ⬜ pendiente | — |
| 05 | Proveedores de correo | ⬜ pendiente | — |
| 06 | Campañas y mensajes | ⬜ pendiente | — |
| 07 | Actividad y entregabilidad | ⬜ pendiente | — |
| 08 | Respaldos y endurecimiento | ⬜ pendiente | — |
| 09 | Release v1.2.0 | ⬜ pendiente | — |

---

## Cómo se valida una fase

```bash
python3 herramientas/validar/validar.py            # todas las fases cerradas
python3 herramientas/validar/validar.py --fase 1   # solo una
python3 herramientas/validar/validar.py --rapido   # omite lo que tarda
```

El mismo script corre en CI ([`validacion`](../../.github/workflows/ci.yml)) y publica su informe como artefacto.

Cada comprobación dice **qué verifica y por qué importa**, para que un fallo se entienda sin abrir el código:

```
✗   0.4s  arles-core no hace I/O
    por qué importa: Regla de frontera 3.2: si el dominio toca I/O, deja de
    poder probarse sin infraestructura.
```

---

## Qué cuenta como fase cerrada

1. Todas las comprobaciones de su sección del validador pasan.
2. **Ninguna queda omitida sin justificación.** Un omitido no es un fallo, pero tampoco es una validación: el resumen los lista aparte precisamente para que no se confundan con lo verificado.
3. El documento de la fase registra qué quedó pendiente y por qué.

---

## Estructura de cada documento

- **Qué se construyó** — con rutas reales, no descripciones.
- **Decisiones tomadas durante la fase** — las que no estaban en los ADR previos.
- **Qué se verificó** — la tabla de comprobaciones, con su método.
- **Qué NO se verificó** — lo más importante del documento.
- **Problemas encontrados** — incluidos los diagnósticos equivocados, porque el siguiente que se tope con lo mismo agradece saberlo.
- **Pendiente**.

Cuando una fase tiene consecuencias que Dirección debe entender sin leer código,
lleva además un documento `FASE-NN-PARA-DIRECCION.md`: el mismo contenido contado
sin tecnicismos. La Fase 1 tiene el suyo en
[FASE-01-PARA-DIRECCION.md](FASE-01-PARA-DIRECCION.md) y la Fase 2 en
[FASE-02-PARA-DIRECCION.md](FASE-02-PARA-DIRECCION.md).
