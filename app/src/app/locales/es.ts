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
    irAlSitio: 'Ir a telemetrymx.com · se abre en tu navegador',
    // ADR-0010: «I» y el número de versión nunca en el mismo renglón.
    nombre: 'ARLES RELAY',
    nombreComercial: 'ARLES RELAY I',
    atribucion: 'Software desarrollado por TELEMETRY INSIGHT',
  },

  nav: {
    principal: 'Navegación principal',
    inicio: 'Inicio',
    campanas: 'Campañas',
    contactos: 'Contactos',
    remitentes: 'Remitentes',
    actividad: 'Actividad',
    ajustes: 'Ajustes',
    // P-11. El botón dice lo que va a hacer, no en qué estado está: «Plegar»
    // cuando está abierta. Al revés obliga a deducir la acción del estado.
    plegar: 'Plegar la navegación',
    expandir: 'Expandir la navegación',
    plegadaPorAncho:
      'La navegación se pliega sola porque la ventana es estrecha. ' +
      'Ensánchala para poder abrirla.',
  },

  inicio: {
    titulo: 'Bienvenido a {producto}',
    // WhatsApp entró en el alcance el 17/09/2026 (P-15). Esta línea decía
    // «campañas de correo» y se quedó corta el mismo día.
    entradilla:
      'ARLES ejecuta campañas de correo y de WhatsApp con tus propias ' +
      'cuentas, respetando los límites que definas.',
    alta: 'Para poder enviar tu primera campaña',
    avance: '{hechos} de {total}',
    verPasos: 'Ver los {n} pasos',

    // Primera vez. Una sola cosa que hacer, sin panel que la rodee: quien abre
    // ARLES por primera vez no tiene nada que mirar en un panel.
    primera: {
      titulo: 'Empieza por aquí',
      entradilla:
        'Antes de enviar nada, ARLES necesita saber de qué empresa salen ' +
        'los correos.',
      porQue:
        'El nombre que verán quienes los reciban, y la zona horaria, que es ' +
        'la que decide a qué hora sale cada uno. Son cinco campos.',
      accion: 'Configurar mi empresa',
      // La cifra evita la conclusión de que con esto ya se puede enviar.
      despues: 'Después vienen cinco pasos más.',
    },

    accesos: {
      titulo: 'Accesos rápidos',
      // Un acceso que aún no lleva a nada lo dice, en vez de desaparecer:
      // quien busca dónde se cargan los contactos necesita leer «aún no»,
      // no encontrarse con el vacío.
      aun: 'aún no',
    },

    enConstruccion: {
      titulo: 'Lo que falta por construir',
      nota: '{n} pasos',
      cuerpo:
        'Los módulos de campañas, canales y actividad aparecerán aquí cuando ' +
        'existan. No se dibujan vacíos a propósito: un panel de cajas en ' +
        'espera anuncia cosas que ARLES todavía no sabe hacer.',
    },
    hecho: 'Hecho',
    pendiente: 'Pendiente',
    llegaEn: 'Llega en la entrega {entrega}',
    sinPasosDisponibles:
      'Los siguientes pasos se construyen en las entregas indicadas. ' +
      'No hay nada más que puedas hacer aquí todavía.',
    paso: {
      empresa: {
        titulo: 'Configurar tu empresa',
        detalle:
          'El nombre, el país y la zona horaria. La zona decide a qué hora ' +
          'sale cada correo.',
      },
      remitente: {
        titulo: 'Conectar una cuenta remitente',
        detalle:
          'La cuenta desde la que se envía. ARLES no envía por su cuenta: ' +
          'usa la tuya, con sus límites.',
      },
      contactos: {
        titulo: 'Cargar tus contactos',
        detalle:
          'Importa tu lista desde un archivo. ARLES no incluye contactos ni ' +
          'los compra.',
      },
      plantilla: {
        titulo: 'Escribir una plantilla',
        detalle: 'El mensaje, con los datos de cada contacto sustituidos.',
      },
      ventana: {
        titulo: 'Definir una ventana de envío',
        detalle:
          'Los días y las horas en que se puede enviar, en la zona horaria ' +
          'de tu empresa.',
      },
      campana: {
        titulo: 'Crear tu primera campaña',
        detalle: 'Une lo anterior y queda lista para revisarse antes de activarse.',
      },
    },
  },

  // C-1 y C-2. Apariencia, no «tema»: quien busca cómo poner la pantalla
  // clara no piensa en la palabra «tema».
  apariencia: {
    titulo: 'Apariencia',
    campo: 'Tema de la interfaz',
    ayuda: 'Se aplica al momento y se recuerda al reabrir ARLES.',
    auto: 'Automático · el de tu sistema',
    oscuro: 'Oscuro',
    claro: 'Claro',
    // No es un detalle cosmético: la elección cambia qué contrastes se aplican,
    // y los dos temas están medidos. Decirlo evita la pregunta de si el claro
    // es «el bueno» o un añadido a medias.
    porQue: '¿Cambia algo más que el color?',
    porQueDetalle:
      'No. Los dos temas se miden con las mismas reglas de contraste, así ' +
      'que ninguno se lee peor que el otro. «Automático» sigue a tu sistema ' +
      'y cambia con él mientras ARLES está abierto.',
  },

  empresa: {
    entradilla: 'Los datos de tu empresa y cómo afectan a los envíos.',
    titulo: 'Configuración de empresa',
    guardar: 'Guardar',
    guardada: 'Configuración guardada.',
    guardadaDetalle: 'Los envíos usarán estos datos a partir de ahora.',
    porQueLaZona: '¿Por qué importa la zona horaria?',
    porQueLaZonaDetalle:
      'Las ventanas de envío se calculan siempre en la zona de tu empresa, ' +
      'nunca en la del equipo donde esté ARLES. Así una campaña programada ' +
      'de 9 a 18 sale a esa hora aunque el equipo viaje a otro huso.',
    // Nombres legibles de lo que ofrecen los desplegables.
    //
    // El **valor** sigue siendo el que exige el núcleo —`MX`,
    // `America/Mexico_City`—; esto es sólo lo que se lee. Enseñar el
    // identificador crudo obliga al usuario a saber qué es un identificador
    // IANA para elegir su ciudad.
    //
    // El validador comprueba que ninguna opción del núcleo se queda sin
    // nombre: si falta, se enseña el valor crudo y nadie se entera.
    pais: {
      MX: 'México',
    },
    zona: {
      'America/Mexico_City': 'Ciudad de México',
      'America/Cancun': 'Cancún · Quintana Roo',
      'America/Merida': 'Mérida · Yucatán',
      'America/Monterrey': 'Monterrey · Nuevo León',
      'America/Matamoros': 'Matamoros · frontera',
      'America/Chihuahua': 'Chihuahua',
      'America/Ojinaga': 'Ojinaga · frontera',
      'America/Mazatlan': 'Mazatlán · Sinaloa',
      'America/Bahia_Banderas': 'Bahía de Banderas · Nayarit',
      'America/Hermosillo': 'Hermosillo · Sonora',
      'America/Tijuana': 'Tijuana · frontera',
      UTC: 'UTC · tiempo universal',
    },
    campo: {
      nombreComercial: 'Nombre comercial',
      pais: 'País',
      zonaHoraria: 'Zona horaria',
      correoCorporativo: 'Correo corporativo',
      sitioWeb: 'Sitio web',
    },
    ayuda: {
      nombreComercial: 'Como quieres que aparezca en tus correos.',
      pais: 'Determina el marco legal aplicable. Esta versión opera en México.',
      zonaHoraria: 'Decide a qué hora sale cada correo.',
      correoCorporativo: 'Para avisos internos de ARLES. No se usa para enviar campañas.',
      sitioWeb: 'Opcional. Con https:// delante.',
    },
    // Claves que devuelve `arles_core::empresa`. Una por motivo, porque «dato
    // inválido» no dice qué corregir.
    error: {
      generico: 'Este dato no es válido.',
      nombreVacio: 'Escribe el nombre comercial de tu empresa.',
      nombreLargo: 'El nombre es demasiado largo: usa 120 caracteres o menos.',
      nombreConSaltos: 'El nombre no puede contener saltos de línea.',
      paisNoSoportado: 'Esta versión de ARLES opera en México.',
      zonaNoSoportada: 'Elige una zona horaria de la lista.',
      // `{'@'}` y no una arroba suelta: en la gramática de vue-i18n `@` abre
      // un enlace a otra clave, y el texto no compila. Reventaba la pantalla
      // entera, en silencio. Lo vigila una prueba que compila todos los textos.
      correoInvalido: "Revisa que el correo tenga la forma nombre{'@'}dominio.com.",
      sitioLargo: 'La dirección es demasiado larga.',
      sitioSinEsquema: 'Escribe la dirección completa, empezando por https://.',
      sitioInvalido: 'Esta dirección no es válida. Debe empezar por https:// o http://.',
    },
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
      como: "Revisa que tenga la forma nombre{'@'}dominio.com.",
      salvo: 'No se guardó ningún cambio.',
    },
    // L-1 · las etapas de una campaña. No los puede provocar quien usa ARLES a
    // ciegas: el asistente ofrece lo que se puede elegir. Están porque el
    // núcleo valida igualmente, y un error sin texto se pinta como su clave.
    etapa: {
      sin_etapas: {
        que: 'La campaña no tiene ninguna etapa.',
        como: 'Añade al menos una: un canal, un remitente y un mensaje.',
        salvo: 'La campaña sigue en borrador. No se envió nada.',
      },
      demasiadas: {
        que: 'Una campaña admite como mucho dos etapas.',
        como: 'Quita una, o crea una segunda campaña para el resto.',
        salvo: 'La campaña sigue como estaba.',
      },
      posiciones: {
        que: 'Las etapas están mal numeradas.',
        como: 'Tienen que ser la 1 y la 2, sin huecos ni repetidas.',
        salvo: 'La campaña sigue como estaba.',
      },
      canal_repetido: {
        que: 'Las dos etapas usan el mismo canal.',
        como:
          'Una etapa por canal: si las dos son de correo, la segunda no ' +
          'enviaría nada — para ARLES sería el mismo envío repetido.',
        salvo: 'La campaña sigue en borrador. No se envió nada.',
      },
      primera_no_espera: {
        que: 'La primera etapa no puede esperar ni depender de otra.',
        como: 'Quita la espera y la condición de la etapa 1.',
        salvo:
          'La campaña sigue como estaba. Si se hubiera activado así, no ' +
          'habría enviado nada y no habría forma de saber por qué.',
      },
      espera_desmesurada: {
        que: 'La espera entre etapas pasa de 30 días.',
        como: 'Ponla en 30 días o menos.',
        salvo: 'La campaña sigue como estaba.',
      },
      condicion_desconocida: {
        que: 'La condición de la etapa no es una de las admitidas.',
        como: 'Vuelve a elegirla en el paso de ritmo de la campaña.',
        salvo: 'La campaña sigue como estaba.',
      },
    },
    telefono_invalido: {
      que: 'El número de teléfono no es válido.',
      como:
        'Escríbelo con lada, por ejemplo 81 1234 5678, o con el prefijo del ' +
        'país si es de fuera de México: +1 415 555 0132.',
      salvo: 'No se guardó ningún cambio.',
    },
    // No lo puede provocar el usuario: el canal lo elige ARLES, no se escribe.
    // Si aparece, es una fila de la base con un canal que no reconocemos.
    canal_desconocido: {
      que: 'Ese contacto tiene un medio de contacto que ARLES no reconoce.',
      como: 'Vuelve a importarlo, o avísanos con el nombre del contacto.',
      salvo:
        'No se le escribió nada. ARLES se detiene antes de enviar cuando no ' +
        'sabe por qué medio hacerlo.',
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
      direccion_en_uso: {
        que: 'Esa dirección ya está registrada en otro contacto.',
        como:
          'Búscala en la lista de contactos para ver de quién es. Si de verdad ' +
          'son la misma persona, edita el contacto que ya existe en vez de ' +
          'crear uno nuevo.',
        salvo:
          'No se guardó nada: ni el contacto ni las direcciones que sí eran ' +
          'nuevas. El contacto que ya existía no se tocó.',
      },
      contacto_no_existe: {
        que: 'Ese contacto ya no está: alguien lo dio de baja.',
        como: 'Actualiza la lista de contactos para ver cómo está ahora.',
        salvo: 'No se modificó nada.',
      },
      dato_invalido: {
        que: 'Un dato guardado no tiene la forma que ARLES espera.',
        como:
          'Vuelve a guardar la configuración desde Ajustes. Si el problema ' +
          'persiste, restaura tu último respaldo.',
        salvo: 'No se modificó ni se borró nada.',
      },
    },
    app: {
      llavero_no_disponible: {
        que: 'No se puede acceder al almacén de credenciales de tu sistema.',
        como:
          'Verifica que esté disponible y desbloqueado, y vuelve a abrir ARLES. ' +
          'Sin él, ARLES no puede descifrar tus datos.',
        salvo: 'Tus datos están intactos: la base de datos no se ha modificado.',
      },
      // Hallazgo F2: se distingue de «llavero no disponible» porque la acción
      // del usuario es completamente distinta. Allí hay que desbloquear el
      // llavero; aquí lo único que recupera los datos es un respaldo.
      clave_maestra_perdida: {
        que: 'La clave que cifraba tu base de datos ya no está en tu sistema.',
        como:
          'Restaura tu último respaldo .arles. Sin esa clave, los datos ' +
          'existentes no se pueden leer: es la propiedad que los protegía.',
        salvo: 'El archivo no se ha modificado ni borrado: sigue donde estaba.',
      },
      // No lo produce el núcleo: lo produce la interfaz cuando corre sin él
      // (`vite dev`, las sondas). Fingir un guardado ahí sería enseñar una
      // pantalla que miente sobre lo que acaba de pasar.
      sin_nucleo: {
        que: 'Esta ventana no está conectada al núcleo de ARLES.',
        como: 'Abre ARLES desde su acceso directo, no desde el navegador.',
        salvo: 'No se guardó ningún cambio.',
      },
      empresa_invalida: {
        que: 'Algunos datos de la empresa no son válidos.',
        como: 'Revisa los campos marcados y vuelve a guardar.',
        salvo: 'No se guardó ningún cambio: la configuración anterior sigue activa.',
      },
      directorio_de_datos: {
        que: 'No se pudo acceder a la carpeta de datos de ARLES.',
        como:
          'Comprueba que tu usuario tiene permiso de escritura en la carpeta ' +
          'de datos de las aplicaciones.',
        salvo: 'No se ha creado ni modificado ningún archivo.',
      },
      sitio_no_abre: {
        que: 'No se pudo abrir tu navegador.',
        como: 'Escribe telemetrymx.com en la barra de direcciones.',
        salvo: 'No cambió nada en ARLES: sólo no se abrió la página.',
      },
      // No lo puede provocar el desplegable, que tiene tres opciones. Si
      // aparece, algo está hablando con ARLES por su cuenta.
      tema_desconocido: {
        que: 'No se pudo guardar el tema.',
        como: 'Vuelve a elegirlo en Ajustes › Apariencia.',
        salvo: 'La pantalla sigue con el tema que ves; sólo no se recordó.',
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
