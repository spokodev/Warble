/// Clean up transcription text after Whisper returns it.
pub fn cleanup(text: &str) -> String {
    let text = remove_repetitions(text);
    let text = text.trim().to_string();
    text
}

/// Remove consecutive repeated phrases (3+ repeats → keep one).
/// Handles patterns like "як це працює, як це працює, як це працює, як це працює"
fn remove_repetitions(text: &str) -> String {
    let result = remove_repeated_segments(text);

    // Also handle word-level repetition like "так так так так"
    let words: Vec<&str> = result.split_whitespace().collect();
    if words.len() < 3 {
        return result;
    }

    let mut cleaned: Vec<&str> = Vec::new();
    let mut repeat_count = 0;

    for i in 0..words.len() {
        if i > 0 && words[i] == words[i - 1] {
            repeat_count += 1;
            if repeat_count < 2 {
                // Allow one repeat ("так так" is ok, "так так так" is not)
                cleaned.push(words[i]);
            }
        } else {
            repeat_count = 0;
            cleaned.push(words[i]);
        }
    }

    cleaned.join(" ")
}

/// Remove repeated comma-separated or period-separated segments.
/// "як це працює, як це працює, як це працює" → "як це працює"
fn remove_repeated_segments(text: &str) -> String {
    // Try progressively larger segment sizes (3-30 words)
    let mut result = text.to_string();

    for segment_len in 2..=20 {
        result = remove_repeated_ngram(&result, segment_len);
    }

    result
}

fn remove_repeated_ngram(text: &str, n: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < n * 3 {
        return text.to_string();
    }

    let mut result: Vec<&str> = Vec::new();
    let mut i = 0;

    while i < words.len() {
        // Check if segment [i..i+n] repeats starting at i+n
        if i + n * 2 <= words.len() {
            let segment: Vec<&str> = words[i..i + n].to_vec();
            let mut repeats = 1;
            let mut j = i + n;

            while j + n <= words.len() {
                let next: Vec<&str> = words[j..j + n].to_vec();
                // Compare ignoring trailing comma/period
                if segments_match(&segment, &next) {
                    repeats += 1;
                    j += n;
                } else {
                    break;
                }
            }

            if repeats >= 3 {
                // Keep segment once, skip repetitions
                result.extend_from_slice(&segment);
                i = j;
                continue;
            }
        }

        result.push(words[i]);
        i += 1;
    }

    result.join(" ")
}

/// Compare two word segments, ignoring trailing punctuation.
fn segments_match(a: &[&str], b: &[&str]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (wa, wb) in a.iter().zip(b.iter()) {
        let ca = wa.trim_end_matches(|c: char| c == ',' || c == '.' || c == '!' || c == '?');
        let cb = wb.trim_end_matches(|c: char| c == ',' || c == '.' || c == '!' || c == '?');
        if ca != cb {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_triple_repeat() {
        let input = "як це має працювати, як це має працювати, як це має працювати, як це має працювати";
        let result = cleanup(input);
        assert_eq!(result, "як це має працювати,");
    }

    #[test]
    fn test_keep_double() {
        let input = "добре, добре, поїхали";
        let result = cleanup(input);
        assert_eq!(result, "добре, добре, поїхали");
    }

    #[test]
    fn test_word_repeat() {
        let input = "так так так так так";
        let result = cleanup(input);
        assert_eq!(result, "так так");
    }

    #[test]
    fn test_normal_text_unchanged() {
        let input = "Привіт, як справи? Все добре.";
        let result = cleanup(input);
        assert_eq!(result, "Привіт, як справи? Все добре.");
    }
}
