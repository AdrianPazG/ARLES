# ADR-0012 · `/RECURSOS` como fuente única de verdad

**Estado:** propuesto — **pendiente de autorización de Dirección** · **Fecha:** 2026-09-11

> ⚠️ **Nada se ha movido, renombrado ni borrado.** El brief marcó estas rutas como sólo lectura (§11) y no se tocan sin autorización explícita. Este ADR es la propuesta.

---

## Contexto

El §11 instruye localizar `/RECURSOS` y `/REFERENCIA_DE_COLOR` y tratarlas como críticas y de sólo lectura. El §12 describe `/RECURSOS` como contenedor de referencias de interfaz, identidad de marca, tipografía y activos gráficos.

La auditoría encontró que **`/RECURSOS` es una copia parcial y obsoleta** de carpetas que viven en la raíz del repositorio:

| Carpeta | En raíz | En `/RECURSOS` |
|---|---:|---:|
| `CONCEPTOS_DE_DISEÑO` | 27 archivos | **1** |
| `TIPOGRAFIA` | 89 archivos | **1** |
| `REFERENCIAS_VISUALES_DEL_SITIO` | 8 archivos | 8 |

Los archivos solapados son idénticos (md5 verificado).

**El riesgo concreto:** quien siga literalmente la instrucción del §12 —«inspecciona `/RECURSOS` antes de proponer la UI»— verá **1 de 27** conceptos de diseño y concluirá que apenas hay material. El brief señala como fuente autoritativa la copia más pobre.

Además, `REFERENCIA_DE_COLOR/farm-lifestyle-digital-art.jpg` **no es un JPEG**: es un PNG de 2320×3080 renombrado (hallazgo A-02). Cualquier herramienta que enrute por extensión fallará o producirá basura en silencio.

## Decisión propuesta

**1 · `/RECURSOS` pasa a ser la fuente única de verdad** y se rehidrata con el contenido completo desde raíz.

**2 · Las carpetas duplicadas de raíz se eliminan** una vez verificado que `/RECURSOS` las contiene íntegras.

**3 · La referencia cromática se renombra a `.png`**, que es lo que realmente es. **No se convierte ni se recomprime:** es el original y se conserva bit a bit.

**4 · Se añade un `README.md` dentro de `/RECURSOS`** que documenta qué es cada carpeta **de verdad**, no lo que el brief suponía:

```
RECURSOS/
├── README.md
├── REFERENCIA_DE_COLOR/
│   ├── farm-lifestyle-digital-art.png   PNG 2320×3080. Stock generado
│   │                                    por IA (IPTC trainedAlgorithmicMedia).
│   │                                    USO INTERNO. No se distribuye.
│   └── ARLES_RELAY-paleta-v1.2.0.png    Lámina de la paleta oficial derivada
│                                        de la anterior. Activo PROPIO de
│                                        TELEMETRY. Referencia rápida; la
│                                        fuente de verdad es COLOR_SYSTEM.md
├── TIPOGRAFIA/                          Mont (Fontfabric). Licencia PENDIENTE (D-3).
├── CONCEPTOS_DE_DISEÑO/                 Carruseles de tips de UX de redes sociales
│                                        (@ux_snacks, @uxwithvamshi). Heurística
│                                        genérica, NO identidad de ARLES.
└── REFERENCIAS_VISUALES_DEL_SITIO/      Mockups de terceros (COINEST, Milray Park).
                                         Referencia de densidad y jerarquía. IP ajena.
```

**5 · Estructura para los activos que aún no existen**, con marcadores explícitos:

```
RECURSOS/
└── MARCA/            ← VACÍA. Ver P-02: no existe ningún activo de marca de ARLES
```

## Justificación

**Una sola ubicación elimina la clase entera de errores** de «trabajé sobre la copia obsoleta».

**Documentar qué es cada carpeta de verdad es lo que más valor aporta.** El brief afirma que `/CONCEPTOS_DE_DISEÑO` contiene referencias de identidad; en realidad son consejos genéricos de UX de Instagram, uno de ellos un anuncio. Sin esa aclaración escrita, alguien intentará derivar la identidad visual de ARLES de un carrusel sobre botones de radio.

**La carpeta `MARCA/` vacía es deliberada.** Un hueco explícito y nombrado comunica «esto falta y alguien debe entregarlo» mucho mejor que una ausencia silenciosa.

## Consecuencias

**Positivas.** Fuente única · el `README` previene malinterpretaciones que ya ocurrieron · la referencia cromática con su extensión real no rompe herramientas · el hueco de marca queda visible.

**Negativas.** Contradice literalmente la instrucción de sólo lectura del §11 — por eso **requiere autorización explícita** (P-08). Y cambia rutas que el brief menciona por nombre, así que el propio brief queda parcialmente desactualizado; este ADR es el registro de por qué.

## Alternativas descartadas

**No tocar nada.** Respeta el §11 al pie de la letra y deja activa la trampa de la copia obsoleta. El coste se paga cada vez que alguien nuevo entra al proyecto.

**Rehidratar `/RECURSOS` y conservar las copias de raíz.** Elimina el problema de la copia incompleta pero mantiene dos ubicaciones que se desincronizarán de nuevo. Media medida.

**Hacer de la raíz la fuente y eliminar `/RECURSOS`.** Funcionalmente equivalente, pero el brief nombra `/RECURSOS` explícitamente en dos secciones. Menos disruptivo conservar el nombre que el brief usa.

**Convertir el PNG a JPEG para que coincida con su extensión.** Al revés: destruiría información del original con una recompresión con pérdida. Se corrige la extensión, no el archivo.

## Autorización pendiente

**P-08.** Hasta que Dirección autorice, el repositorio permanece exactamente como está y esta propuesta queda registrada aquí.
