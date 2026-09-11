/**
 * Textos en español de México.
 *
 * Reglas de UX_WRITING.md que este archivo debe respetar:
 *
 * - Tono profesional y claro. Nada de «¡Ups!» ni celebraciones (§94).
 * - Todo error dice **qué pasó, cómo arreglarlo y qué está a salvo** (§95).
 * - «Aceptado» nunca es «Entregado» (§65).
 * - Sin concatenación: interpolación con parámetros nombrados.
 */
export const es = {
  producto: {
    // ADR-0010: «I» y el número de versión nunca en el mismo renglón.
    nombre: 'ARLES RELAY',
    nombreComercial: 'ARLES RELAY I',
    atribucion: 'Software desarrollado por TELEMETRY INSIGHT',
  },

  nav: {
    inicio: 'Inicio',
    campanas: 'Campañas',
    contactos: 'Contactos',
    remitentes: 'Remitentes',
    actividad: 'Actividad',
    ajustes: 'Ajustes',
  },

  /** Estados de un intento de envío. Ver arles-core::AttemptState. */
  intento: {
    queued: 'En cola',
    claimed: 'En cola',
    sending: 'Enviando',
    // NO «Entregado»: el proveedor solo confirma que se hizo cargo.
    sent: 'Aceptado',
    failed: 'Reintentando',
    permanently_failed: 'Falló',
    suppressed: 'Suprimido',
    cancelled: 'Cancelado',
    presumed_sent: 'Envío no confirmado',
  },

  honestidad: {
    porQueAceptado: '¿Por qué «aceptado» y no «entregado»?',
    porQueAceptadoDetalle:
      'Tu proveedor de correo confirma que recibió el mensaje y que intentará ' +
      'entregarlo, pero no informa si llegó al buzón del destinatario. ARLES ' +
      'solo muestra lo que puede verificar.',
    rebotesParciales:
      'La detección automática de rebotes es parcial en esta versión. ARLES ' +
      'detecta los rechazos que el servidor de destino comunica durante el ' +
      'envío, pero no los que llegan después como correo a tu bandeja.',
    rebotesParcialesQueHacer:
      'Revisa tu bandeja periódicamente y suprime a mano las direcciones que reboten.',
    respaldosSinCifrar:
      'Los respaldos no están cifrados en esta versión. El archivo .arles ' +
      'contiene los datos de tus contactos: guárdalo en un lugar seguro.',
  },

  error: {
    // Cada error lleva sus tres partes (§95).
    email_invalido: {
      que: 'La dirección de correo no es válida.',
      como: 'Revisa que tenga la forma nombre@dominio.com.',
      salvo: 'No se guardó ningún cambio.',
    },
    id_invalido: {
      que: 'El identificador no es válido.',
      como: 'Vuelve a abrir la pantalla desde el menú.',
      salvo: 'Tus datos están intactos.',
    },
    transicion_invalida: {
      que: 'Esa acción no es posible en el estado actual del envío.',
      como: 'Actualiza la vista para ver el estado más reciente.',
      salvo: 'No se modificó ningún envío.',
    },
    db: {
      clave_incorrecta: {
        que: 'No se pudo abrir la base de datos con las credenciales guardadas.',
        como: 'Verifica que el almacén de credenciales del sistema esté desbloqueado.',
        salvo: 'La base de datos no se ha modificado.',
      },
      clave_mal_formada: {
        que: 'La credencial guardada en el sistema no tiene el formato esperado.',
        como: 'Contacta con soporte antes de continuar.',
        salvo: 'La base de datos no se ha modificado.',
      },
      migracion: {
        que: 'No se pudo actualizar la estructura de la base de datos.',
        como: 'Cierra ARLES y vuelve a abrirlo. Si persiste, restaura tu último respaldo.',
        salvo: 'Se hizo una copia de tu base de datos antes de intentarlo.',
      },
      sqlite: {
        que: 'Ocurrió un error al acceder a los datos.',
        como: 'Cierra ARLES y vuelve a abrirlo.',
        salvo: 'Las operaciones incompletas se revirtieron por completo.',
      },
    },
  },

  vacio: {
    campanas: {
      titulo: 'Aún no hay campañas.',
      cuerpo:
        'Una campaña envía un mensaje a una lista de contactos, respetando los ' +
        'límites que definas.',
      accion: 'Crear campaña',
    },
    supresiones: {
      titulo: 'No hay direcciones suprimidas.',
      cuerpo:
        'Las direcciones suprimidas quedan excluidas de todos los envíos futuros. ' +
        'Se añaden cuando alguien se da de baja, cuando un correo rebota de forma ' +
        'permanente, o cuando las excluyes a mano.',
      accion: 'Suprimir una dirección',
    },
  },

  comun: {
    cancelar: 'Cancelar',
    cargando: 'Cargando…',
  },
} as const
