// Catalogs are prepended by the Trunk staging hooks from locales/*.json.
(() => {
  const supported = ['en-US', 'es-AR', 'gn-PY', 'pt-BR'];
  const valid = value => supported.includes(value);
  const match = tag => {
    if (typeof tag !== 'string') return null;
    const normalized = tag.trim().toLowerCase();
    return supported.find(locale => locale.toLowerCase() === normalized) ||
      supported.find(locale => locale.split('-')[0] === normalized.split('-')[0]) || null;
  };
  const resolve = (saved, languages, language) => {
    if (valid(saved)) return saved;
    const preferred = Array.isArray(languages) && languages.length ? languages : [language];
    for (const tag of preferred) {
      const locale = match(tag);
      if (locale) return locale;
    }
    return 'en-US';
  };
  let saved;
  try { saved = localStorage.getItem('sudoku_locale'); } catch {}
  const locale = resolve(saved, navigator.languages, navigator.language);
  const api = window.__sudokuI18n = {
    locale,
    saveFailed: false,
    resolve,
    currentLocale() { return this.locale; },
    failedToSave() { return this.saveFailed; },
    text(key) { return window.__sudokuCatalogs[this.locale][key]; },
    setLocale(value) {
      if (!valid(value)) return;
      this.locale = value;
      try { localStorage.setItem('sudoku_locale', value); this.saveFailed = false; }
      catch { this.saveFailed = true; }
      this.refresh();
    },
    refresh() {
      document.documentElement.lang = this.locale;
      document.title = this.text('Title');
      document.querySelectorAll('[data-i18n]').forEach(element => {
        element.textContent = this.text(element.dataset.i18n);
      });
      const status = document.getElementById('share-status');
      if (status && status.dataset.message) status.textContent = this.text(status.dataset.message);
    },
    shareStatus(key) {
      const status = document.getElementById('share-status');
      if (status) { status.dataset.message = key; status.textContent = key ? this.text(key) : ''; }
    },
    playedDate(ms) {
      const options = { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' };
      const locale = this.locale === 'gn-PY' && !Intl.DateTimeFormat.supportedLocalesOf(['gn-PY']).length ? 'es-PY' : this.locale;
      return new Intl.DateTimeFormat(locale, options).format(new Date(ms));
    }
  };
  // Persist detection and the English fallback as explicit choices on first use.
  api.setLocale(locale);
  document.addEventListener('DOMContentLoaded', () => api.refresh());
})();
