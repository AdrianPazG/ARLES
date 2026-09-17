/**
 * GENERADO. No editar a mano.
 *
 * Sale de `documentacion/07-entrega/avance.json` con
 * `python3 herramientas/avance/calcular.py --escribir`.
 *
 * Dirección pidió ver el avance en GitHub y en la pantalla de Inicio. Son dos
 * sitios; dos números escritos a mano divergen en la primera prisa. El
 * validador recalcula y falla si este archivo, el README y la fuente dejan de
 * coincidir.
 */
export const AVANCE_PORCENTAJE = 31

/** La fecha del dato, no la de hoy: un porcentaje sin fecha no dice nada. */
export const AVANCE_ACTUALIZADO = '2026-09-17'

export interface FaseDeAvance {
  id: string
  nombre: string
  /** Fracción de 0 a 1. */
  hecho: number
}

export const AVANCE_FASES: readonly FaseDeAvance[] = [
  { id: 'F0', nombre: 'Discovery y auditoría', hecho: 1.0 },
  { id: 'F1', nombre: 'Cimientos', hecho: 1.0 },
  { id: 'F2', nombre: 'Design System', hecho: 1.0 },
  { id: 'F3', nombre: 'Empresa y contactos', hecho: 0.2 },
  { id: 'F3W', nombre: 'Canales: esquema de contactos, consentimiento y etapas', hecho: 0.55 },
  { id: 'F4', nombre: 'Motor de ejecución', hecho: 0.0 },
  { id: 'F5', nombre: 'Proveedores de envío', hecho: 0.0 },
  { id: 'F6', nombre: 'Campañas y mensajes', hecho: 0.05 },
  { id: 'F7', nombre: 'Actividad y entregabilidad', hecho: 0.0 },
  { id: 'F8', nombre: 'Respaldos y endurecimiento', hecho: 0.0 },
  { id: 'F9', nombre: 'Release v1.2.0', hecho: 0.0 },
]
