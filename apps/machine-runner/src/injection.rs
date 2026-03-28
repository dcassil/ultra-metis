/// Format a message for injection into a supervised process's stdin.
///
/// The `injection_type` determines the framing applied to the message:
///
/// - `"normal"` — the message is written as-is with a trailing newline.
/// - `"side_note"` — prefixed with `[Note from user]: `.
/// - `"interrupt"` — prefixed with `[URGENT]: `.
/// - Anything else falls back to `"normal"` behavior.
pub fn format_injection(message: &str, injection_type: &str) -> String {
    match injection_type {
        "normal" => format!("{message}\n"),
        "side_note" => format!("[Note from user]: {message}\n"),
        "interrupt" => format!("[URGENT]: {message}\n"),
        _ => format!("{message}\n"),
    }
}

/// Format an approval response to write to a supervised process's stdin.
///
/// The choice string (e.g. `"yes"`, `"no"`, `"always"`) is written verbatim
/// followed by a newline, matching what Claude Code expects on stdin when
/// waiting for a human approval decision.
pub fn format_approval_response(choice: &str) -> String {
    format!("{choice}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- format_injection tests ----

    #[test]
    fn test_format_injection_normal() {
        let result = format_injection("hello world", "normal");
        assert_eq!(result, "hello world\n");
    }

    #[test]
    fn test_format_injection_side_note() {
        let result = format_injection("please check the tests", "side_note");
        assert_eq!(result, "[Note from user]: please check the tests\n");
    }

    #[test]
    fn test_format_injection_interrupt() {
        let result = format_injection("stop immediately", "interrupt");
        assert_eq!(result, "[URGENT]: stop immediately\n");
    }

    #[test]
    fn test_format_injection_unknown_type_falls_back_to_normal() {
        let result = format_injection("fallback message", "unknown_type");
        assert_eq!(result, "fallback message\n");
    }

    #[test]
    fn test_format_injection_empty_message() {
        assert_eq!(format_injection("", "normal"), "\n");
        assert_eq!(format_injection("", "side_note"), "[Note from user]: \n");
        assert_eq!(format_injection("", "interrupt"), "[URGENT]: \n");
    }

    #[test]
    fn test_format_injection_multiline_message() {
        let result = format_injection("line1\nline2", "normal");
        assert_eq!(result, "line1\nline2\n");
    }

    // ---- format_approval_response tests ----

    #[test]
    fn test_format_approval_response_yes() {
        assert_eq!(format_approval_response("yes"), "yes\n");
    }

    #[test]
    fn test_format_approval_response_no() {
        assert_eq!(format_approval_response("no"), "no\n");
    }

    #[test]
    fn test_format_approval_response_always() {
        assert_eq!(format_approval_response("always"), "always\n");
    }

    #[test]
    fn test_format_approval_response_empty() {
        assert_eq!(format_approval_response(""), "\n");
    }
}
