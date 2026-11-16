import { createI18n } from 'vue-i18n'

import { es } from '@/app/locales/es'

/**
 * Todo texto visible pasa por aquí desde el día uno (§139), aunque v1.2.0 solo
 * tenga español.
 *
 * Retrofitear i18n sobre una interfaz terminada es mecánico, tedioso y siempre
 * deja cadenas huérfanas. Hacerlo desde el inicio no cuesta casi nada, y es
 * requisito del white labeling futuro (§117).
 */
export const i18n = createI18n({
  legacy: false,
  locale: 'es',
  fallbackLocale: 'es',
  // Una clave que falta es un bug, no algo que deba pasar en silencio.
  missingWarn: true,
  fallbackWarn: true,
  messages: { es },
})

export type Locale = keyof typeof i18n.global.messages.value
