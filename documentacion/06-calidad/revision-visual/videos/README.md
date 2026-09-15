# Grabaciones de pantalla

Aquí van los vídeos de las revisiones. **Antes de subir nada, lee los dos
límites de abajo:** son reales y te van a morder.

---

## Lo que puedo y lo que no puedo hacer con un vídeo

**Sé honesto con esto o perderás tiempo grabando cosas que no me sirven.**

| | |
|---|---|
| ✅ **Veo la imagen** | Extraigo fotogramas y los miro uno a uno. Para todo lo visual —una ventana que salta, un menú que se dibuja mal, algo que parpadea— un vídeo me vale tanto como una captura, y a veces más |
| ❌ **NO oigo el audio** | No puedo escuchar. Un vídeo de NVDA o VoiceOver leyendo **no me dice nada**: veo el cursor moverse, pero no sé qué dijo |

### Entonces, para el lector de pantalla

Tienes tres formas de que me llegue, de mejor a peor:

1. **Escríbelo en la plantilla.** «Al llegar al logotipo dijo *ARLES, RELAY,
   gráfico*» — con eso trabajo. Es lo más rápido para los dos.
2. **Activa los subtítulos de NVDA** (*Speech Viewer* / Visor de voz, en el menú
   de NVDA → Herramientas). Abre una ventana con **todo lo que dice, en texto**.
   Grábala dentro del vídeo, o captúrala, y ahí sí lo leo.
3. Vídeo con audio a secas. **No me sirve.** No lo grabes sólo para esto.

> El **Visor de voz** de NVDA es el atajo bueno: convierte el problema de audio
> en un problema de imagen, que es el que sí puedo resolver.

---

## Los límites de tamaño

| Ruta | Máximo por archivo |
|---|---|
| **Subir desde el navegador** (arrastrar a GitHub) | **25 MB** |
| Cualquier otra ruta a Git | 100 MB |

Una grabación de 30 minutos a pantalla completa pesa **cientos de MB**. No cabe.

### Qué hacer

**Recorta en vez de comprimir.** No me mandes los 30 minutos: mándame **el
trozo donde pasó lo que quieres enseñarme**, de 10 a 60 segundos.

En Windows, sin instalar nada:

1. Abre el vídeo con la aplicación **Fotos** (Photos).
2. **Editar → Recortar** (*Trim*).
3. Arrastra los dos extremos hasta dejar sólo el trozo.
4. **Guardar como copia**.

Si aun así pasa de 25 MB, bájale la resolución: la app **Clipchamp** —viene con
Windows 11— exporta a 720p, y para ver un fallo de interfaz 720p sobra.

---

## Cómo nombrarlos

Igual que las capturas, con el sistema delante y de qué sección hablan:

```
videos/
├── W-escalado-200-desborda.mp4
├── W-tabla-desplazamiento.mp4
└── M-desplegable-proveedor.mov
```

`W` es Windows, `M` es Mac. **El nombre debe decir qué se ve**, porque es lo
primero que leo.

---

## Y dime el momento

Aunque el clip sea corto, escríbeme en la plantilla **en qué segundo** está lo
que quieres que mire y **qué estabas haciendo**:

> *`W-tabla-desplazamiento.mp4`, hacia el segundo 8: bajo rápido con la rueda y
> la tabla da un tirón.*

Sin eso miro el vídeo entero fotograma a fotograma buscando algo que tú ya
sabías dónde estaba.

---

## Lo recibido

| Vídeo | Duración | Qué confirma |
|---|---|---|
| `W-01-ventana.mp4` | 47 s | El redimensionado: se ve el cursor en el borde y la ventana frenando en su mínimo |
| `W-02-maximizada.mp4` | 16 s | La ventana maximizada |
| `W-03-primitivas.mp4` | 20 s | Recorrido de la pestaña de primitivas |
| `W-04-estados.mp4` | 24 s | Los cuatro estados |

Revisados por fotogramas el 15 de septiembre. **No añaden defectos** sobre las
capturas: confirman lo mismo. Que no aporten nada nuevo es, en sí, un
resultado — significa que las capturas cubrían lo que había que ver.
