//! Tests for the textinput component.

use super::*;

#[cfg(test)]
mod textinput_tests {
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

        // With empty value, should show placeholder (remainder after cursor)
        let view_empty = input.view();
        // The current implementation shows cursor + remainder, so we check for remainder
        assert!(view_empty.contains("nter text"), "Should contain placeholder remainder");

        // With value, should not show placeholder
        input.set_value("actual text");
        let view_with_text = input.view();
        assert!(!view_with_text.contains("Enter text"));
        assert!(!view_with_text.contains("nter text"));
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

    /// Tests specifically for placeholder rendering bug fix and regression prevention
    mod placeholder_rendering_tests {
        use super::*;

        #[test]
        fn test_placeholder_no_duplication_basic() {
            // Test core fix: placeholder should not duplicate first character
            let mut input = new();
            input.set_placeholder("Nickname");
            let _ = input.focus(); // Focus to show cursor on first char

            let view = input.view();
            
            // Should show: "> " + cursor with 'N' + remaining "ickname" 
            // NOT: "> " + cursor with 'N' + full "Nickname"
            assert!(view.starts_with("> "), "Should start with prompt");
            
            // Count occurrences of 'N' - should be exactly 1 (in cursor position)
            let n_count = view.chars().filter(|&c| c == 'N').count();
            assert_eq!(n_count, 1, "Should have exactly one 'N' character, found {} in: '{}'", n_count, view);
            
            // Should contain the remaining part of placeholder
            assert!(view.contains("ickname"), "Should contain remaining placeholder 'ickname' in: '{}'", view);
        }

        #[test] 
        fn test_placeholder_specific_examples() {
            // Test the specific examples from the original bug report
            let test_cases = [
                ("Nickname", "ickname"),
                ("Email", "mail"), 
                ("Password", "assword"),
            ];

            for (placeholder, expected_remainder) in test_cases {
                let mut input = new();
                input.set_placeholder(placeholder);
                let _ = input.focus();

                let view = input.view();
                
                // Should start with prompt
                assert!(view.starts_with("> "), "Placeholder '{}' should start with prompt", placeholder);
                
                // Should contain remaining part after first character
                assert!(view.contains(expected_remainder), 
                    "Placeholder '{}' should contain remainder '{}' but view is: '{}'", 
                    placeholder, expected_remainder, view);
                
                // Should NOT contain the full placeholder string duplicated
                let first_char = placeholder.chars().next().unwrap();
                let full_placeholder_occurrences = view.matches(placeholder).count();
                assert_eq!(full_placeholder_occurrences, 0, 
                    "Placeholder '{}' should not appear in full in view: '{}'", 
                    placeholder, view);
                
                // First character should appear exactly once (in cursor)
                let first_char_count = view.chars().filter(|&c| c == first_char).count();
                assert_eq!(first_char_count, 1, 
                    "First character '{}' should appear exactly once, found {} in: '{}'", 
                    first_char, first_char_count, view);
            }
        }

        #[test]
        fn test_placeholder_with_different_cursor_modes() {
            use crate::cursor::Mode;
            
            let mut input = new();
            input.set_placeholder("Test");
            let _ = input.focus();

            // Test with different cursor modes
            let modes = [Mode::Blink, Mode::Static, Mode::Hide];
            
            for mode in modes {
                let _ = input.cursor.set_mode(mode);
                let view = input.view();
                
                // Regardless of cursor mode, should not duplicate
                let t_count = view.chars().filter(|&c| c == 'T').count();
                assert!(t_count <= 1, "With cursor mode {:?}, should have at most one 'T', found {} in: '{}'", 
                    mode, t_count, view);
                
                // Should contain remainder
                assert!(view.contains("est") || mode == Mode::Hide, 
                    "With cursor mode {:?}, should contain 'est' or be hidden, view: '{}'", 
                    mode, view);
            }
        }

        #[test]
        fn test_placeholder_blurred_vs_focused() {
            let mut input = new();
            input.set_placeholder("Example");
            
            // When blurred, should show placeholder content (cursor + remainder)
            input.blur();
            let blurred_view = input.view();
            
            // The placeholder_view always shows cursor + remainder regardless of focus state
            assert!(blurred_view.contains("xample"), "Blurred view should show placeholder content: '{}'", blurred_view);
            
            // When focused, should show cursor + remainder only
            let _ = input.focus();
            let focused_view = input.view();
            
            // Should NOT show full "Example" duplicated 
            let example_count = focused_view.matches("Example").count();
            assert_eq!(example_count, 0, "Focused view should not contain full 'Example': '{}'", focused_view);
            
            // Should show remainder
            assert!(focused_view.contains("xample"), "Focused view should contain 'xample': '{}'", focused_view);
        }

        #[test]
        fn test_placeholder_edge_cases() {
            // Single character placeholder
            let mut input = new();
            input.set_placeholder("A");
            let _ = input.focus();
            let view = input.view();
            
            let a_count = view.chars().filter(|&c| c == 'A').count();
            assert_eq!(a_count, 1, "Single char placeholder should have exactly one 'A': '{}'", view);
            
            // Empty placeholder
            let mut input2 = new();
            input2.set_placeholder("");
            let _ = input2.focus();
            let view2 = input2.view();
            // Should not panic and should show just prompt + cursor space
            assert!(view2.starts_with("> "), "Empty placeholder should show prompt");
            
            // Unicode placeholder
            let mut input3 = new();
            input3.set_placeholder("测试"); // Chinese characters
            let _ = input3.focus();
            let view3 = input3.view();
            
            // Should handle Unicode correctly without duplication
            let first_char_count = view3.chars().filter(|&c| c == '测').count();
            assert!(first_char_count <= 1, "Unicode placeholder should not duplicate first char: '{}'", view3);
        }

        #[test] 
        fn test_placeholder_transitions() {
            let mut input = new();
            input.set_placeholder("Username");
            let _ = input.focus();
            
            // Focused empty input - should show cursor + remainder
            let empty_focused = input.view();
            assert!(empty_focused.contains("sername"), "Should show remainder when focused and empty");
            
            // Add some text - placeholder should disappear
            input.set_value("user");
            let with_text = input.view();
            assert!(with_text.contains("user"), "Should show actual text");
            assert!(!with_text.contains("Username"), "Should not show placeholder when text present");
            assert!(!with_text.contains("sername"), "Should not show placeholder remainder when text present");
            
            // Clear text - placeholder should return
            input.set_value("");
            let cleared = input.view();
            assert!(cleared.contains("sername"), "Should show remainder again when cleared");
        }

        #[test]
        fn test_placeholder_with_width_constraints() {
            let mut input = new();
            input.set_placeholder("VeryLongPlaceholderText");
            input.set_width(10);
            let _ = input.focus();
            
            let view = input.view();
            
            // Should not duplicate first character even with width constraints
            let v_count = view.chars().filter(|&c| c == 'V').count();
            assert_eq!(v_count, 1, "Should have exactly one 'V' even with width constraints: '{}'", view);
            
            // Should handle width properly
            assert!(view.len() >= 10, "Should respect minimum width");
        }

        #[test]
        fn test_regression_original_bug_would_fail() {
            // This test would fail with the original bug (p[0..] instead of p[1..])
            let mut input = new();
            input.set_placeholder("Bug");
            let _ = input.focus();
            
            let view = input.view();
            
            // The original bug would produce "> B" + "Bug" = "> BBug"
            // The fix should produce "> B" + "ug" = "> Bug"
            assert!(!view.contains("BBug"), "Should not show duplicated 'BBug' - this indicates the original bug: '{}'", view);
            
            // Should show correct format
            let b_count = view.chars().filter(|&c| c == 'B').count();
            assert_eq!(b_count, 1, "Should have exactly one 'B': '{}'", view);
            assert!(view.contains("ug"), "Should contain remainder 'ug': '{}'", view);
        }

        #[test]
        fn test_placeholder_styling_preserved() {
            use lipgloss::{Style, Color};
            
            let mut input = new();
            input.set_placeholder("Styled");
            
            // Set custom placeholder style
            input.placeholder_style = Style::new().foreground(Color::from("blue"));
            let _ = input.focus();
            
            let view = input.view();
            
            // Should not duplicate regardless of styling
            let s_count = view.chars().filter(|&c| c == 'S').count();
            assert_eq!(s_count, 1, "Styled placeholder should not duplicate: '{}'", view);
            
            // Should contain remainder
            assert!(view.contains("tyled"), "Should contain styled remainder: '{}'", view);
        }
    }
}
