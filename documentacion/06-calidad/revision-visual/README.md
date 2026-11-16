# Revisiones visuales · entregas recibidas

Aquí queda el material de cada revisión: las capturas, la plantilla rellenada,
las grabaciones y **mi análisis**. Es el registro de qué se miró, cuándo, en
qué equipo y qué salió.

Los manuales y las plantillas en blanco están un nivel más arriba, en
[`../`](..) — empieza por [MANUAL_DE_ENTREGA.md](../MANUAL_DE_ENTREGA.md).

---

## Entregas

| Fecha | Sistema | Estado | Hallazgos |
|---|---|---|---|
| 2026-09-14 | 🪟 Windows 11 | 8 de 9 capturas · NVDA pendiente | [HALLAZGOS.md](2026-09-14-windows/HALLAZGOS.md) — **1 defecto**, 2 peticiones |
| — | 🍎 Mac | pendiente | — |

---

## Cómo se organiza

```
revision-visual/
├── README.md                     ← esto
├── videos/                       ← grabaciones, con sus límites explicados
│   └── README.md                 ← LÉELO antes de grabar nada
└── AAAA-MM-DD-sistema/
    ├── PLANTILLA-…-rellenada.docx
    ├── HALLAZGOS.md              ← lo escribo yo al analizar
    └── capturas/
        └── W-01-ventana.png …
```

Una carpeta por entrega, con su fecha delante. Así no se pisan cuando la misma
persona revise dos veces, ni cuando revisen dos personas distintas.

---

## Antes de subir vídeos

**Lee [`videos/README.md`](videos/README.md).** Resumen de lo que hay dentro,
para que no grabes en balde:

- **Veo la imagen de un vídeo** — extraigo fotogramas y los miro.
- **NO oigo el audio.** Un vídeo de NVDA leyendo no me dice nada.
  Para eso está el **Visor de voz** de NVDA, que escribe en pantalla lo que
  dice: eso sí lo leo.
- **25 MB por archivo** si lo subes desde el navegador. Recorta el trozo que
  importa en vez de mandar la grabación entera.

---

## Por qué esto se versiona

Una captura de un fallo es la prueba de que existía. Cuando dentro de tres
fases alguien pregunte «¿esto se revisó alguna vez?», la respuesta tiene que
ser un archivo con fecha, no un recuerdo.

Y sirve para lo contrario: **R-01 lo encontré yo en una captura donde el
revisor había marcado «no lo noté»**. Sin la captura guardada, ese defecto se
habría perdido en el hueco entre lo que se miró y lo que se vio.
