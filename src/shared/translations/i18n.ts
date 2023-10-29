import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import ISO6391 from 'iso-639-1';

import nl from './nl.json';
import en from './en.json';

const resources = {
	nl: {
		translation: nl,
	},
	en: {
		translation: en,
	},
};

const languages: string[] = Object.keys(resources);

interface LanguageInfo {
	code: string;
	englishName: string;
	nativeName: string;
}

export const languageMap: LanguageInfo[] = languages.map((code) => ({
	code,
	englishName: ISO6391.getName(code),
	nativeName: ISO6391.getNativeName(code),
}));

i18n.use(initReactI18next) // passes i18n down to react-i18next
	.init({
		debug: false,
		resources,
		keySeparator: '^',
		nsSeparator: '^',
		lng: 'en',
		fallbackLng: 'nl',
		interpolation: {
			escapeValue: false,
		},
	});

export default i18n;
