import { createPinia } from 'pinia'
import { createApp } from 'vue'

// El orden importa: los tokens definen las variables que base.css consume,
// y las @font-face tienen que estar declaradas antes de que body las use.
import '@tokens/arles-tokens.css'
import '@/design/tipografia.css'
import '@/design/base.css'

import App from '@/app/App.vue'
import { i18n } from '@/app/i18n'
import { router } from '@/app/router'

createApp(App).use(createPinia()).use(router).use(i18n).mount('#app')
