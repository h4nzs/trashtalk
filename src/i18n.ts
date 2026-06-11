import i18n from 'i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

// Import translation files
import en from '../src-tauri/locales/en.json';
import id from '../src-tauri/locales/id.json';

i18n
  .use(LanguageDetector)
  .init({
    resources: {
      en: { translation: en },
      id: { translation: id }
    },
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false // not needed for xss
    }
  });

export default i18n;
