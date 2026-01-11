import en from "./i18n.en";
import es from "./i18n.es";
import fr from "./i18n.fr";
import de from "./i18n.de";
import ja from "./i18n.ja";
import ko from "./i18n.ko";
import ru from "./i18n.ru";
import zhCn from "./i18n.zh-CN";
import zhTw from "./i18n.zh-TW";

export type Locale =
  | "en"
  | "es"
  | "fr"
  | "de"
  | "ru"
  | "ko"
  | "zh-CN"
  | "zh-TW"
  | "ja";

type TranslationDict = Record<string, string>;

const STORAGE_KEY = "truthbyte.locale";

const isLocale = (value: string): value is Locale =>
  value === "en" ||
  value === "es" ||
  value === "fr" ||
  value === "de" ||
  value === "ru" ||
  value === "ko" ||
  value === "zh-CN" ||
  value === "zh-TW" ||
  value === "ja";

const translations: Record<Locale, TranslationDict> = {
  en,
  es,
  fr,
  de,
  ru,
  ko,
  "zh-CN": zhCn,
  "zh-TW": zhTw,
  ja
};

export const localeOptions: Array<{ value: Locale; label: string }> = [
  { value: "en", label: "English" },
  { value: "es", label: "Español" },
  { value: "fr", label: "Français" },
  { value: "de", label: "Deutsch" },
  { value: "ru", label: "Русский" },
  { value: "ko", label: "한국어" },
  { value: "zh-CN", label: "简体中文" },
  { value: "zh-TW", label: "繁體中文" },
  { value: "ja", label: "日本語" }
];

export const getInitialLocale = (): Locale => {
  if (typeof localStorage !== "undefined") {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored && isLocale(stored)) {
      return stored;
    }
  }

  if (typeof navigator === "undefined") return "en";
  const locale = navigator.language.toLowerCase();
  if (locale.startsWith("zh-cn") || locale.startsWith("zh-hans")) {
    return "zh-CN";
  }
  if (
    locale.startsWith("zh-tw") ||
    locale.startsWith("zh-hant") ||
    locale.startsWith("zh-hk")
  ) {
    return "zh-TW";
  }
  if (locale.startsWith("ja")) {
    return "ja";
  }
  if (locale.startsWith("ko")) {
    return "ko";
  }
  if (locale.startsWith("es")) {
    return "es";
  }
  if (locale.startsWith("fr")) {
    return "fr";
  }
  if (locale.startsWith("de")) {
    return "de";
  }
  if (locale.startsWith("ru")) {
    return "ru";
  }
  return "en";
};

export const persistLocale = (locale: Locale) => {
  if (typeof localStorage !== "undefined") {
    localStorage.setItem(STORAGE_KEY, locale);
  }
};

export const createTranslator =
  (locale: Locale) =>
  (key: string, vars: Record<string, string> = {}) => {
    const template = translations[locale][key] ?? translations.en[key] ?? key;
    return template.replace(/\{(\w+)\}/g, (_, token: string) => {
      return vars[token] ?? `{${token}}`;
    });
  };
