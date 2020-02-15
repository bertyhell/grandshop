import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import nl from './nl.json';
import en from './en.json';

i18n
    .use(initReactI18next) // passes i18n down to react-i18next
    .init({
      debug: true,
      resources: {
        nl: {
          translation: nl,
        },
        en: {
          translation: en,
        },
      },
      keySeparator: '^',
      nsSeparator: '^',
      lng: 'nl',
      fallbackLng: 'nl',
      interpolation: {
        escapeValue: false,
      },
    });

export default i18n;
