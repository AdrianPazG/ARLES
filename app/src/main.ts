import { createPinia } from 'pinia'
import { createApp } from 'vue'

// El orden importa: los tokens definen las variables que base.css consume,
// y las @font-face tienen que estar declaradas antes de que body las use.
import '@tokens/arles-tokens.css'
import '@/design/tipografia.css'
import '@/design/base.css'

import App from '@/app/App.vue'
import { i18n } from '@/app/i18n'
import { enRecuperacion } from '@/app/nucleo'
import { router } from '@/app/router'

// ── EL MODO DE RECUPERACIÓN MANDA SOBRE CUALQUIER RUTA (8.4) ───────────────
//
// Si ARLES arrancó sin poder abrir su base, lo único que existe es la pantalla
// de recuperación: el núcleo registra tres comandos y ninguno más. Llegar a
// cualquier otra ruta daría una pantalla que no puede cargar nada y no dice
// por qué.
//
// Va antes de montar, no en un `onMounted`, para que no se llegue a pintar un
// instante la pantalla equivocada.
if (enRecuperacion()) {
  void router.replace('/recuperar')
}

createApp(App).use(createPinia()).use(router).use(i18n).mount('#app')
