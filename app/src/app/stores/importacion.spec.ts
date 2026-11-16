/**
 * El aviso de la columna de teléfono sin aclarar.
 *
 * ── POR QUÉ (22/10/2026) ────────────────────────────────────────────────────
 *
 * Dirección preguntó si ARLES podía cargar una tabla del cliente con nueve
 * columnas y quedarse con lo que necesita. Podía — con un agujero: **una
 * columna «Teléfono» se ignoraba**, así que su tabla real entraba con ocho
 * columnas reconocidas y sin un solo número, sin que nada lo dijera.
 *
 * El arreglo fue reconocerla, y eso abrió la pregunta que el núcleo ya tenía
 * documentada: «Teléfono» no dice si ese número recibe WhatsApp, y mandar
 * mensajes a fijos se paga en calidad del número ante Meta. Así que se acierta
 * **y se avisa**.
 *
 * Esto comprueba la parte que es del frontend: que el aviso cruce lo que dijo
 * el archivo con el mapeo que el usuario tiene **ahora**, y que desaparezca en
 * cuanto hace lo que el aviso pedía. Un aviso que sigue ahí después de
 * atenderlo enseña a no leer los avisos.
 */
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import type { ArchivoLeido } from '@/app/stores/importacion'
import { useImportacionStore } from '@/app/stores/importacion'

/** La tabla que mandó Dirección, con sus nueve columnas. */
const TABLA_DE_DIRECCION: ArchivoLeido = {
  nombre: 'empresas.xlsx',
  encabezados: [
    'Nombre',
    'Razon Social',
    'Giro',
    'Actividad',
    'Tamaño',
    'Telefono',
    'Correo',
    'Sitio Web',
    'Ubicacion',
  ],
  muestra: [],
  mapeo: [
    'nombre',
    'empresa',
    'ignorar',
    'ignorar',
    'ignorar',
    'whatsapp',
    'correo',
    'ignorar',
    'ignorar',
  ],
  totalDeFilas: 1200,
  // La posición de «Telefono». La calcula Rust; aquí se da como la daría él.
  telefonosSinAclarar: [5],
}

describe('el aviso del teléfono sin aclarar', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('no dice nada antes de elegir archivo', () => {
    const s = useImportacionStore()
    expect(s.columnasPorAclarar).toEqual([])
  })

  it('nombra la columna que el archivo no aclaró', () => {
    const s = useImportacionStore()
    s.archivo = TABLA_DE_DIRECCION
    s.mapeo = [...TABLA_DE_DIRECCION.mapeo]
    expect(s.columnasPorAclarar).toEqual(['Telefono'])
  })

  /**
   * **El caso que justifica cruzar las dos cosas.** Si el aviso mirara sólo lo
   * que dijo el archivo, seguiría ahí después de que el usuario haga lo que le
   * pedía — y entonces no es un aviso, es decorado.
   */
  it('se calla en cuanto esa columna deja de ir a WhatsApp', () => {
    const s = useImportacionStore()
    s.archivo = TABLA_DE_DIRECCION
    s.mapeo = [...TABLA_DE_DIRECCION.mapeo]
    s.mapeo[5] = 'ignorar'
    expect(s.columnasPorAclarar).toEqual([])
  })

  /**
   * Y vuelve si se deshace. No es una comprobación de más: una implementación
   * que apagara el aviso con una bandera en vez de calcularlo pasaría la
   * anterior y fallaría ésta.
   */
  it('y vuelve si se deshace el cambio', () => {
    const s = useImportacionStore()
    s.archivo = TABLA_DE_DIRECCION
    s.mapeo = [...TABLA_DE_DIRECCION.mapeo]
    s.mapeo[5] = 'ignorar'
    s.mapeo[5] = 'whatsapp'
    expect(s.columnasPorAclarar).toEqual(['Telefono'])
  })

  it('no avisa de una columna que ya dice ser un móvil', () => {
    const s = useImportacionStore()
    s.archivo = {
      ...TABLA_DE_DIRECCION,
      encabezados: ['Nombre', 'Correo', 'Celular'],
      mapeo: ['nombre', 'correo', 'whatsapp'],
      // Rust no la marca: «Celular» sí dice de qué número habla.
      telefonosSinAclarar: [],
    }
    s.mapeo = ['nombre', 'correo', 'whatsapp']
    expect(s.columnasPorAclarar).toEqual([])
  })

  it('nombra las dos cuando son dos', () => {
    const s = useImportacionStore()
    s.archivo = {
      ...TABLA_DE_DIRECCION,
      encabezados: ['Nombre', 'Telefono', 'Numero'],
      mapeo: ['nombre', 'whatsapp', 'whatsapp'],
      telefonosSinAclarar: [1, 2],
    }
    s.mapeo = ['nombre', 'whatsapp', 'whatsapp']
    expect(s.columnasPorAclarar).toEqual(['Telefono', 'Numero'])
  })
})

describe('la descarga de la plantilla', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  /**
   * Sin núcleo —la vista previa en un navegador suelto— no se inventa un
   * archivo guardado: se dice que no hay con qué. Un acuse de algo que no
   * ocurrió es peor que un error.
   */
  it('sin núcleo no acusa ningún archivo', async () => {
    const s = useImportacionStore()
    await s.descargarPlantilla()
    expect(s.plantillaGuardada).toBeNull()
    expect(s.errorGeneral).toBe('error.app.sin_nucleo')
  })

  /** Descargar la plantilla **no** empieza ninguna importación. */
  it('no mueve el paso del asistente', async () => {
    const s = useImportacionStore()
    await s.descargarPlantilla()
    expect(s.paso).toBe('elegir')
  })
})
