use futures::stream::{self, StreamExt};

use super::Masker;
use crate::TextStream;

const MAX_HELD_DIGITS: usize = 10;

/// Restores masked values in a model's output stream, even when a placeholder
/// such as `<IP_1>` arrives split across several chunks.
pub fn unmask_stream(masker: Masker, inner: TextStream) -> TextStream {
    if !masker.has_placeholders() {
        return inner;
    }
    let state = (inner, masker, String::new(), false);
    stream::unfold(state, |(mut inner, masker, mut pending, done)| async move {
        if done {
            return None;
        }
        loop {
            match inner.next().await {
                Some(Ok(chunk)) => {
                    pending.push_str(&chunk);
                    let tail = pending.split_off(held_back_start(&pending));
                    let ready = std::mem::replace(&mut pending, tail);
                    if ready.is_empty() {
                        continue;
                    }
                    let text = masker.unmask(&ready);
                    return Some((Ok(text), (inner, masker, pending, false)));
                }
                Some(Err(e)) => return Some((Err(e), (inner, masker, pending, false))),
                None if pending.is_empty() => return None,
                None => {
                    let text = masker.unmask(&pending);
                    return Some((Ok(text), (inner, masker, String::new(), true)));
                }
            }
        }
    })
    .boxed()
}

/// Byte index from which the text could still be the start of a placeholder.
fn held_back_start(text: &str) -> usize {
    match text.rfind('<') {
        Some(pos) if could_be_placeholder(&text[pos + 1..]) => pos,
        _ => text.len(),
    }
}

fn could_be_placeholder(rest: &str) -> bool {
    ["IP_", "USER_"].iter().any(|prefix| {
        prefix.starts_with(rest)
            || rest.strip_prefix(prefix).is_some_and(|digits| {
                digits.len() <= MAX_HELD_DIGITS && digits.chars().all(|c| c.is_ascii_digit())
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holds_back_partial_placeholders_only() {
        assert_eq!(held_back_start("abc <"), 4);
        assert_eq!(held_back_start("abc <US"), 4);
        assert_eq!(held_back_start("abc <USER_12"), 4);
        assert_eq!(held_back_start("abc <USER_12>"), 13);
        assert_eq!(held_back_start("a < b"), 5);
        assert_eq!(held_back_start("<div"), 4);
    }
}
