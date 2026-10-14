# Mapa funcional · qué hace cada pantalla y cómo se conectan

**Escrito el 14 de octubre de 2026**, a petición de Dirección, extrayendo los
datos **del código** —el router, los almacenes y la frontera— y no de la memoria
ni de lo que el ROADMAP prometía.

> **Para qué sirve.** Para probar ARLES **sistemáticamente** en vez de a ojo:
> dice, de cada pantalla, qué hace, qué necesita antes, a dónde lleva y qué
> comandos del motor usa. Sin esta referencia se recorren las pantallas que uno
> recuerda y se escapan caminos enteros.
>
> **No es el manual de usuario**, que se escribe después de probar y cuenta
> *cómo se usa*. Éste cuenta *qué hay y qué hace*.
>
> **Ni es [`MAPA-DE-PANTALLAS.md`](MAPA-DE-PANTALLAS.md)**, que es para
> programadores —qué archivo dibuja cada pantalla, dónde viven sus textos— y se
> quedó en trece pantallas.

---

## 0 · Las cifras, medidas

| | |
|---|---|
| Pantallas con ruta propia | **18** (+ el catálogo del design system, que no viaja en el instalador) |
| Secciones en la barra lateral | **6** · Inicio · Campañas · Contactos · Remitentes · Actividad · Ajustes |
| Comandos registrados en la frontera | **59** |
| Almacenes de estado | **14** |

**Cada comando registrado tiene quien lo llame**, y lo vigila
`ningun_comando_se_queda_sin_quien_lo_llame` en `--fase W`: un comando que nadie
invoca es superficie de ataque que nadie vigila, así que se quita.

---

## 1 · El árbol, de un vistazo

```
/                        → redirige a /inicio
│
├─ /inicio                       INICIO · el panel y la lista de 6 pasos
│  └─ /inicio/configuracion      Datos de la empresa
│
├─ /campanas                     CAMPAÑAS · lista, y el freno
│  ├─ /campanas/plantillas       Plantillas y firmas
│  └─ /campanas/nueva            Asistente de 9 pasos
│
├─ /contactos                    CONTACTOS · tabla, alta y edición
│  ├─ /contactos/importar        Importar CSV o XLSX
│  ├─ /contactos/supresion       Lista de bajas
│  └─ /contactos/derechos        Derechos ARCO
│
├─ /remitentes                   REMITENTES · cuentas de correo
│  └─ /remitentes/whatsapp       Números de WhatsApp
│
├─ /actividad                    ACTIVIDAD · cifras por campaña
│  └─ /actividad/registro        Registro de envíos, con filtros
│
├─ /ajustes                      AJUSTES · apariencia
│  ├─ /ajustes/respaldos         Crear un respaldo
│  ├─ /ajustes/restaurar         Restaurar uno
│  └─ /ajustes/archivar          Archivar el historial
│
└─ /recuperar            FUERA DEL ARMAZÓN · sin barra lateral
                         Sólo aparece si se perdió la llave del llavero
```

**`/recuperar` no está en la barra lateral y es deliberado:** sin base de datos
no hay secciones a las que ir. Es la única pantalla que no pasa por el armazón.

---

## 2 · El orden que impone el producto

ARLES no deja saltarse pasos, y la lista de Inicio lo refleja: **1 de 6**, **2 de
6**… Lo que bloquea a qué:

```
   Configurar la empresa
            │
            │  (todo lo demás la necesita: el alta de un remitente,
            │   los contactos y las campañas piden la empresa y
            │   fallan sin ella)
            ▼
   ┌────────┴────────┬──────────────────┐
   ▼                 ▼                  ▼
Remitentes       Contactos          Plantillas
(cuenta SMTP)    (importar o        (el mensaje)
   │              dar de alta)           │
   └────────┬──────────┴─────────────────┘
            ▼
      Nueva campaña  ── el asistente de 9 pasos exige los tres
            │
            ▼
        Activarla  ── y entonces aparece en Actividad
```

> **Esto es lo que causó el hallazgo H-01.** Entrar en Remitentes sin empresa
> configurada da un aviso correcto; lo que estaba mal es que el aviso **seguía
> ahí** después de configurarla. Está en
> [`BITACORA-DE-PRUEBAS.md`](../06-calidad/BITACORA-DE-PRUEBAS.md).

---

## 3 · Las dieciocho, una por una

Cada fila dice **qué hace**, **qué necesita antes** y **qué comandos del motor
usa** — extraídos de los almacenes, no supuestos.

### INICIO

| | |
|---|---|
| **`/inicio`** | El panel. Si no hay empresa, enseña «Empieza por aquí» y un solo botón. Si la hay, los **6 pasos** con cuántos van |
| Necesita | Nada. Es la puerta |
| Lleva a | `/inicio/configuracion` |
| Comandos | `configuracion_de_empresa` |
| Qué verificar | Que el contador de pasos **cambie** al completar uno. Y que si los datos no se pueden leer, **no** pinte «0 de 6» — eso afirmaría algo que no sabe |

| | |
|---|---|
| **`/inicio/configuracion`** | Nombre comercial, país, zona horaria, correo corporativo y sitio web |
| Necesita | Nada |
| Comandos | `configuracion_de_empresa`, `guardar_empresa` |
| Qué verificar | Que al guardar **se queda guardado** tras cerrar y reabrir ARLES. Que la zona horaria sale en los envíos. Que los campos obligatorios no dejan guardar vacíos |

### CAMPAÑAS

| | |
|---|---|
| **`/campanas`** | La lista de campañas, su estado, y **la parada de emergencia** |
| Necesita | Empresa |
| Comandos | `campanas`, `estado_del_envio`, `parar_todo`, `levantar_parada`, `ventanas`, `guardar_ventana` |
| Qué verificar | Que «Detener» **para de verdad** y que se puede levantar. Que la ventana de envío se guarda |

| | |
|---|---|
| **`/campanas/plantillas`** | Escribir plantillas y firmas. Las versiones **no se editan**: cada cambio crea una nueva |
| Necesita | Empresa |
| Comandos | `plantillas`, `guardar_plantilla`, `firmas`, `guardar_firma` |
| Qué verificar | Que las variables `{{nombre}}` se sustituyen. Que una plantilla guardada **no cambia** al editarla: aparece otra versión |

| | |
|---|---|
| **`/campanas/nueva`** | El asistente de **9 pasos**. Al activar, **congela la audiencia** |
| Necesita | Empresa · remitente · contactos · plantilla |
| Comandos | `crear_campana`, `revisar_campana`, `activar_campana`, `enviar_correo_de_prueba` |
| Qué verificar | Que **no deja pasar** del paso 8 sin que el correo de prueba llegue. Que el paso 6 avisa si la campaña **no puede terminar**. Que avisa si pasa de **50 diarios** |

### CONTACTOS

| | |
|---|---|
| **`/contactos`** | La tabla, con alta, edición y baja |
| Necesita | Empresa |
| Comandos | `listar_contactos`, `crear_contacto`, `editar_contacto`, `borrar_contacto` |
| Qué verificar | Que un correo mal escrito se rechaza **al escribirlo**. Que dar de baja lo saca de futuras campañas |

| | |
|---|---|
| **`/contactos/importar`** | CSV o XLSX, con **los choques revisados antes** de importar |
| Necesita | Empresa |
| Comandos | `elegir_archivo_para_importar`, `analizar_importacion`, `confirmar_importacion`, `cancelar_importacion` |
| Qué verificar | Que enseña los repetidos **antes** y deja cancelar. Que un archivo con acentos, BOM o Excel raro entra bien |
| Lleva a | `/contactos` al terminar |

| | |
|---|---|
| **`/contactos/supresion`** | Las bajas. **Una baja no se borra jamás** |
| Necesita | Empresa |
| Comandos | `listar_supresiones`, `suprimir_contacto`, `levantar_supresion` |
| Qué verificar | Que un contacto dado de baja **no entra** en una campaña nueva |

| | |
|---|---|
| **`/contactos/derechos`** | Los cuatro derechos ARCO: acceso, rectificación, cancelación y oposición |
| Necesita | Empresa |
| Comandos | `entregar_informe_de_acceso`, `que_se_borraria`, `cancelar_datos_de` |
| Qué verificar | Que «qué se borraría» dice la cifra **antes** de borrar nada |

### REMITENTES

| | |
|---|---|
| **`/remitentes`** | Las cuentas de correo. **ARLES no envía por su cuenta**: usa la tuya |
| Necesita | **Empresa** ← el alta falla sin ella |
| Comandos | `remitentes`, `guardar_remitente`, `probar_remitente`, `probar_envio_de_la_cuenta` |
| Qué verificar | Que **«Probar conexión» dice qué falla**, no un error genérico. Que la contraseña **no vuelve** a la pantalla una vez guardada |

> **Desde el 14/10/2026 (R-01) el formulario son dos campos**: correo y
> contraseña. El servidor, el puerto y el cifrado se deducen del dominio con
> `PROVEEDORES` —Gmail, Outlook, Yahoo, Zoho e iCloud— y viven en una
> «Configuración avanzada» que **sólo se abre sola cuando el dominio no se
> reconoce**, que es cuando de verdad hacen falta.
>
> Qué verificar aquí: que escribir `@gmail.com` rellena `smtp.gmail.com`, `587`
> y STARTTLS **y lo dice**; que el usuario sigue a la dirección mientras nadie
> lo toque; y que con Gmail avisa si la contraseña no mide dieciséis letras —la
> de aplicación—, que es lo que impidió el alta en la prueba del 13/10/2026.
>
> Lo vigila `el puerto y el cifrado nunca se cruzan`: 587 con STARTTLS, 465 con
> TLS directo, y ningún tercer puerto.

| | |
|---|---|
| **`/remitentes/whatsapp`** | Los números de WhatsApp, con Coexistencia y la aceptación de riesgo |
| Necesita | Empresa · cuenta de Meta |
| Comandos | `numeros_de_whatsapp`, `guardar_numero_de_whatsapp`, `enviar_whatsapp_de_prueba` |
| Qué verificar | Que el envío de prueba llega al teléfono. **Una campaña entera por WhatsApp no existe** |

> **Aquí NO se puede hacer lo del correo, y el límite está declarado (R-02).**
> El correo se deduce del dominio; los tres identificadores de Meta —WABA ID,
> Phone Number ID y token— **sólo existen en su panel**. Lo que sí se hizo es
> agrupar el formulario en tres bloques con su rótulo —«Tu número», «Lo que se
> copia del panel de Meta», «Cómo se va a usar»— y **cazar el error de pegado**
> con `revisarDatoDeMeta`: los identificadores son dígitos y el token empieza
> por `EAA`. **Avisa, no bloquea.**

### ACTIVIDAD

| | |
|---|---|
| **`/actividad`** | Una tarjeta por campaña con sus cifras, cuánta gente contestó, el cupo de cada remitente — y **por qué no sale nada** |
| Necesita | Al menos una campaña |
| Comandos | `actividad`, `exportar_resumen` |
| Qué verificar | Que «por qué no sale nada» dice **cuál** de las seis razones es, con su cifra. Que el resumen exportado **no lleva ninguna dirección de correo** |

| | |
|---|---|
| **`/actividad/registro`** | Cada intento de envío, con **seis filtros combinables**. Nunca más de 500 filas, con el total al lado |
| Necesita | Al menos un envío |
| Comandos | `registro`, `que_se_recortaria`, `recortar_registro` |
| Qué verificar | Que los filtros **viajan en la dirección** (se puede volver atrás). Que el recorte dice **antes** cuánto se va y cuánto se queda |

### AJUSTES

| | |
|---|---|
| **`/ajustes`** | Apariencia: tema claro u oscuro, barra lateral plegada |
| Necesita | Nada |
| Comandos | `preferencias_de_interfaz`, `guardar_tema`, `guardar_barra_plegada` |
| Qué verificar | Que el tema **sobrevive** a cerrar ARLES |

| | |
|---|---|
| **`/ajustes/respaldos`** | Crea un archivo `.arles` con todo, cifrado con una contraseña que eliges |
| Necesita | Empresa |
| Comandos | `que_llevaria_el_respaldo`, `exportar_respaldo` |
| Qué verificar | Que dice **cuántas filas y de cuántas tablas** antes de crearlo. **Si olvidas la contraseña, el archivo no se abre nunca** |

| | |
|---|---|
| **`/ajustes/restaurar`** | Restaura un `.arles`. **Reemplaza, nunca mezcla** — y las bajas se unen |
| Necesita | Un archivo `.arles` |
| Comandos | `revisar_respaldo`, `restaurar_respaldo` |
| Qué verificar | Que **antes de tocar nada** dice de cuándo es, qué trae y **qué se va a perder**. Que un archivo de historial se **rechaza** |

| | |
|---|---|
| **`/ajustes/archivar`** | Saca los envíos viejos a un archivo externo, y **sólo después** los borra |
| Necesita | Al menos un año de historial |
| Comandos | `revisar_archivado`, `archivar_historial` |
| Qué verificar | Que **vuelve a abrir el archivo y lo verifica** antes de borrar. Que dice «no se puede deshacer». El mínimo es **un año** |

### FUERA DEL ARMAZÓN

| | |
|---|---|
| **`/recuperar`** | El día que Windows pierde la llave que abre tu base |
| Aparece | Sola, si la base existe y la llave no |
| Comandos | `que_paso_con_la_clave`, `revisar_respaldo_para_recuperar`, `recuperar_desde_respaldo` |
| Qué verificar | Que **no borra** la base vieja: la aparta. Que **no crea una llave nueva** hasta que eliges un respaldo y confirmas. Que si no hay respaldo **lo dice con esas palabras** |

---

## 4 · Lo que NO está construido

No son defectos y no hace falta reportarlos:

| Qué | Por qué |
|---|---|
| Una campaña entera **por WhatsApp** | El motor sólo recorre correo |
| **Filtrar la audiencia** de una campaña | No está construido |
| **Editar una campaña** ya creada | No está construido |
| Contar las **respuestas de WhatsApp** | Necesita el «relay ciego», que no existe |

---

## 5 · Cómo usar este mapa para probar

1. **Sigue el orden del §2.** Saltarse la empresa hace fallar todo lo demás, y
   el fallo no siempre lo dice con claridad.
2. **Por cada pantalla, mira su «Qué verificar».** Son las garantías que el
   producto promete; si una no se cumple, es un hallazgo.
3. **Anótalo** como dice
   [`BITACORA-DE-PRUEBAS.md`](../06-calidad/BITACORA-DE-PRUEBAS.md): pantalla,
   qué hiciste, qué pasó, qué esperabas.
4. **No te saltes lo que parezca pequeño.** En este proyecto, la mayoría de los
   defectos graves los ha encontrado alguien mirando, no una comprobación.
