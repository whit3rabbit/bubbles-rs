//! Tests for the textinput component.

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_default_values() {
        // Test Go's: New()
        let input = new();

        assert_eq!(input.prompt, "> ");
        assert_eq!(input.placeholder, "");
        assert_eq!(input.echo_character, '*');
        assert_eq!(input.char_limit, 0);
        assert_eq!(input.width, 0);
        assert_eq!(input.value(), "");
        assert_eq!(input.position(), 0);
        assert!(!input.focused());
        assert_eq!(input.echo_mode, EchoMode::EchoNormal);
        assert!(input.err.is_none());
    }

    #[test]
    fn test_deprecated_new_model() {
        // Test Go's deprecated: NewModel
        #[allow(deprecated)]
        let input = new_model();
        assert_eq!(input.prompt, "> ");
        assert_eq!(input.value(), "");
    }

    #[test]
    fn test_set_value() {
        // Test Go's: SetValue(s string)
        let mut input = new();
        input.set_value("hello world");

        assert_eq!(input.value(), "hello world");
        assert_eq!(input.position(), input.value().len());
    }

    #[test]
    fn test_set_value_with_char_limit() {
        // Test character limit enforcement
        let mut input = new();
        input.set_char_limit(5);
        input.set_value("hello world"); // Should be truncated

        assert_eq!(input.value(), "hello");
        assert_eq!(input.value().len(), 5);
    }

    #[test]
    fn test_position() {
        // Test Go's: Position() int
        let mut input = new();
        input.set_value("test");

        assert_eq!(input.position(), 4); // Cursor at end

        input.set_cursor(2);
        assert_eq!(input.position(), 2);
    }

    #[test]
    fn test_set_cursor() {
        // Test Go's: SetCursor(pos int)
        let mut input = new();
        input.set_value("hello");

        input.set_cursor(2);
        assert_eq!(input.position(), 2);

        // Test bounds checking
        input.set_cursor(100); // Beyond end
        assert_eq!(input.position(), 5); // Should be clamped to end

        input.set_cursor(0);
        assert_eq!(input.position(), 0);
    }

    #[test]
    fn test_cursor_start_end() {
        // Test Go's: CursorStart() and CursorEnd()
        let mut input = new();
        input.set_value("hello world");
        input.set_cursor(5);

        input.cursor_start();
        assert_eq!(input.position(), 0);

        input.cursor_end();
        assert_eq!(input.position(), 11);
    }

    #[test]
    fn test_focused() {
        // Test Go's: Focused() bool, Focus() tea.Cmd, Blur()
        let mut input = new();

        assert!(!input.focused());

        std::mem::drop(input.focus());
        assert!(input.focused());

        input.blur();
        assert!(!input.focused());
    }

    #[test]
    fn test_reset() {
        // Test Go's: Reset()
        let mut input = new();
        input.set_value("some text");
        input.set_cursor(5);

        input.reset();

        assert_eq!(input.value(), "");
        assert_eq!(input.position(), 0);
    }

    #[test]
    fn test_echo_modes() {
        let mut input = new();
        input.set_value("secret");

        // Test EchoNormal
        input.set_echo_mode(EchoMode::EchoNormal);
        let view_normal = input.view();
        assert!(view_normal.contains("secret"));

        // Test EchoPassword
        input.set_echo_mode(EchoMode::EchoPassword);
        let view_password = input.view();
        assert!(view_password.contains("******")); // Should show asterisks
        assert!(!view_password.contains("secret")); // Should not show actual text

        // Test EchoNone
        input.set_echo_mode(EchoMode::EchoNone);
        let view_none = input.view();
        assert!(!view_none.contains("secret"));
        assert!(!view_none.contains("*"));
    }

    #[test]
    fn test_placeholder() {
        // Test placeholder functionality
        let mut input = new();
        input.set_placeholder("Enter text...");

        // With empty value, should show placeholder
        let view_empty = input.view();
        assert!(view_empty.contains("Enter text"));

        // With value, should not show placeholder
        input.set_value("actual text");
        let view_with_text = input.view();
        assert!(!view_with_text.contains("Enter text"));
        assert!(view_with_text.contains("actual"));
    }

    #[test]
    fn test_width_setting() {
        // Test width setting
        let mut input = new();
        input.set_width(50);

        assert_eq!(input.width, 50);
    }

    #[test]
    fn test_char_limit() {
        // Test character limit functionality
        let mut input = new();
        input.set_char_limit(10);

        assert_eq!(input.char_limit, 10);

        // Should enforce limit on set_value
        input.set_value("this is a very long string");
        assert!(input.value().len() <= 10);
    }

    #[test]
    fn test_suggestions() {
        // Test Go's: SetSuggestions, AvailableSuggestions, MatchedSuggestions, etc.
        let mut input = new();
        let suggestions = vec![
            "apple".to_string(),
            "application".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ];

        input.set_suggestions(suggestions.clone());

        assert_eq!(input.available_suggestions(), suggestions);

        // Test matching
        input.set_value("app");
        input.update_suggestions();

        let matched = input.matched_suggestions();
        assert_eq!(matched.len(), 2);
        assert!(matched.contains(&"apple".to_string()));
        assert!(matched.contains(&"application".to_string()));
        assert!(!matched.contains(&"banana".to_string()));
    }

    #[test]
    fn test_current_suggestion() {
        // Test Go's: CurrentSuggestion, CurrentSuggestionIndex
        let mut input = new();
        input.set_suggestions(vec!["apple".to_string(), "application".to_string()]);
        input.set_value("app");
        input.update_suggestions();

        assert_eq!(input.current_suggestion_index(), 0);
        assert_eq!(input.current_suggestion(), "apple");

        input.next_suggestion();
        assert_eq!(input.current_suggestion_index(), 1);
        assert_eq!(input.current_suggestion(), "application");

        input.previous_suggestion();
        assert_eq!(input.current_suggestion_index(), 0);
        assert_eq!(input.current_suggestion(), "apple");
    }

    #[test]
    fn test_default_trait_implementation() {
        // Test Default trait implementation
        let input = Model::default();
        assert_eq!(input.value(), "");
        assert_eq!(input.prompt, "> ");
        assert!(!input.focused());
    }

    // Tests matching Go's textinput_test.go exactly

    #[test]
    fn test_current_suggestion_go_compat() {
        // Test Go's: Test_CurrentSuggestion
        let mut textinput = new();

        let suggestion = textinput.current_suggestion();
        let expected = "";
        assert_eq!(
            suggestion, expected,
            "Error: expected no current suggestion but was {}",
            suggestion
        );

        textinput.set_suggestions(vec![
            "test1".to_string(),
            "test2".to_string(),
            "test3".to_string(),
        ]);
        let suggestion = textinput.current_suggestion();
        let expected = "";
        assert_eq!(
            suggestion, expected,
            "Error: expected no current suggestion but was {}",
            suggestion
        );

        textinput.set_value("test");
        textinput.update_suggestions();
        textinput.next_suggestion();
        let suggestion = textinput.current_suggestion();
        let expected = "test2";
        assert_eq!(
            suggestion, expected,
            "Error: expected first suggestion but was {}",
            suggestion
        );

        textinput.blur();
        let view = textinput.view();
        assert!(!view.ends_with("test2"), "Error: suggestions should not be rendered when input isn't focused. expected \"> test\" but got \"{}\"", view);
    }

    #[test]
    fn test_slicing_outside_cap() {
        // Test Go's: Test_SlicingOutsideCap
        let mut textinput = new();
        textinput.set_placeholder("作業ディレクトリを指定してください"); // Japanese text
        textinput.set_width(32);

        // Should not panic when rendering with Unicode characters and width constraints
        let _view = textinput.view();
        // Test passes if no panic occurs
    }

    #[test]
    fn test_validate_func_credit_card_example() {
        // Test Go's: ExampleValidateFunc (converted to test)
        let mut credit_card_number = new();
        credit_card_number.set_placeholder("4505 **** **** 1234");
        std::mem::drop(credit_card_number.focus());
        credit_card_number.set_char_limit(20);
        credit_card_number.set_width(30);
        credit_card_number.prompt = "".to_string();

        // Credit card validation function matching grouped format: XXXX XXXX XXXX XXXX
        let credit_card_validator: ValidateFunc = Box::new(|s: &str| {
            // Max length: 19 (16 digits + 3 spaces)
            if s.len() > 19 {
                return Err("CCN is too long".to_string());
            }

            let chars: Vec<char> = s.chars().collect();
            for (i, ch) in chars.iter().enumerate() {
                // Require spaces at positions 4, 9, and 14 if those positions exist
                if i == 4 || i == 9 || i == 14 {
                    if *ch != ' ' {
                        return Err("CCN must separate groups with spaces".to_string());
                    }
                } else if !ch.is_ascii_digit() {
                    return Err("Invalid number format".to_string());
                }
            }

            Ok(())
        });

        credit_card_number.set_validate(credit_card_validator);

        // Test valid credit card format
        credit_card_number.set_value("4505 1234 5678 1234");
        assert!(
            credit_card_number.err.is_none(),
            "Valid credit card should not have error"
        );

        // Test invalid - too long
        credit_card_number.set_value("4505 1234 5678 1234 5678");
        assert!(credit_card_number.err.is_some());
        assert!(credit_card_number
            .err
            .as_ref()
            .unwrap()
            .contains("too long"));

        // Test invalid - missing space
        credit_card_number.set_value("45051234");
        assert!(credit_card_number.err.is_some());
        assert!(credit_card_number
            .err
            .as_ref()
            .unwrap()
            .contains("separate groups"));

        // Test invalid - non-numeric
        credit_card_number.set_value("450a 1234 5678 1234");
        assert!(credit_card_number.err.is_some());
    }
}
