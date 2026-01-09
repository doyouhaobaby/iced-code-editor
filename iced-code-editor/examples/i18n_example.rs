// Example: Using rust-i18n with the code editor
//
// This example demonstrates how to use the internationalization features
// of the iced-code-editor crate with different languages.

use iced_code_editor::{Language, Translations};

fn main() {
    println!("=== Iced Code Editor - Internationalization Example ===\n");

    // Example 1: Using Translations API
    println!("1. Using the Translations API:");

    let mut translations = Translations::new(Language::English);
    println!("  English: {}", translations.search_placeholder());

    translations.set_language(Language::French);
    println!("  French:  {}", translations.search_placeholder());

    translations.set_language(Language::Spanish);
    println!("  Spanish: {}", translations.search_placeholder());

    println!();

    // Example 2: All translations for French
    println!("2. All French translations:");
    let fr = Translations::new(Language::French);
    println!("  Search placeholder:    {}", fr.search_placeholder());
    println!("  Replace placeholder:   {}", fr.replace_placeholder());
    println!("  Case sensitive label:  {}", fr.case_sensitive_label());
    println!("  Previous match:        {}", fr.previous_match_tooltip());
    println!("  Next match:            {}", fr.next_match_tooltip());
    println!("  Close search:          {}", fr.close_search_tooltip());
    println!("  Replace current:       {}", fr.replace_current_tooltip());
    println!("  Replace all:           {}", fr.replace_all_tooltip());

    println!();

    // Example 3: Comparing languages
    println!("3. Comparing 'Replace all' across languages:");
    for lang in [Language::English, Language::French, Language::Spanish] {
        let t = Translations::new(lang);
        println!("  {:?}: {}", lang, t.replace_all_tooltip());
    }

    println!("\n=== Example Complete ===");
}
