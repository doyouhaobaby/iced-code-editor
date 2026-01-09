# Internationalization with rust-i18n

This crate uses [rust-i18n](https://github.com/longbridge/rust-i18n) for managing translations in YAML files.

## Directory Structure

```
iced-code-editor/
├── locales/
│   ├── en.yml      # English translations
│   ├── fr.yml      # French translations
│   └── es.yml      # Spanish translations
└── src/
    ├── lib.rs      # rust-i18n initialization
    └── i18n.rs     # Translation wrappers
```

## Translation Files

Each YAML file contains hierarchical translation keys:

### en.yml (English)
```yaml
search:
  placeholder: "Search..."
  close_tooltip: "Close search dialog (Esc)"
  previous_match_tooltip: "Previous match (Shift+F3)"
  next_match_tooltip: "Next match (F3 / Enter)"

replace:
  placeholder: "Replace..."
  current_tooltip: "Replace current match"
  all_tooltip: "Replace all matches"

settings:
  case_sensitive_label: "Case sensitive"
```

### fr.yml (French)
```yaml
search:
  placeholder: "Rechercher..."
  close_tooltip: "Fermer la recherche (Échap)"
  previous_match_tooltip: "Résultat précédent (Maj+F3)"
  next_match_tooltip: "Résultat suivant (F3 / Entrée)"

replace:
  placeholder: "Remplacer..."
  current_tooltip: "Remplacer l'occurrence actuelle"
  all_tooltip: "Tout remplacer"

settings:
  case_sensitive_label: "Sensible à la casse"
```

### es.yml (Spanish)
```yaml
search:
  placeholder: "Buscar..."
  close_tooltip: "Cerrar búsqueda (Esc)"
  previous_match_tooltip: "Coincidencia anterior (Mayús+F3)"
  next_match_tooltip: "Siguiente coincidencia (F3 / Enter)"

replace:
  placeholder: "Reemplazar..."
  current_tooltip: "Reemplazar coincidencia actual"
  all_tooltip: "Reemplazar todo"

settings:
  case_sensitive_label: "Distinguir mayúsculas"
```

## Usage

### Using the Translations API (Recommended)

The crate provides a simple `Translations` struct that manages language switching:

```rust
use iced_code_editor::{Language, Translations};

// Create translations for a specific language
let translations = Translations::new(Language::French);

// Use translations
println!("{}", translations.search_placeholder()); // "Rechercher..."

// Change language
let mut translations = Translations::new(Language::English);
translations.set_language(Language::Spanish);
println!("{}", translations.search_placeholder()); // "Buscar..."
```

See the `examples/i18n_example.rs` file for a complete working example.

## Adding New Languages

1. Create a new YAML file in `iced-code-editor/locales/` (e.g., `de.yml` for German)
2. Add all translation keys following the same structure as existing files
3. Add the language variant to the `Language` enum in `src/i18n.rs`
4. Update the `to_locale()` method to return the correct locale code
5. Update the `Translations` methods to include the new language in match statements

Example for adding German:

```rust
// In src/i18n.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    English,
    French,
    Spanish,
    German,  // New language
}

impl Language {
    pub const fn to_locale(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::French => "fr",
            Self::Spanish => "es",
            Self::German => "de",  // New locale code
        }
    }
}
```

## Features

- **Compile-time validation**: Translation keys are checked at compile time
- **Fallback support**: Missing translations fall back to English
- **Type-safe**: No runtime string errors for translation keys
- **YAML format**: Easy to read and edit translation files
- **Hierarchical keys**: Organized translation structure with dot notation

## Dependencies

- `rust-i18n = "3"` - Internationalization library with YAML support
