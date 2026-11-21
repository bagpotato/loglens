use std::fmt::Display;

use futures::stream::{self, BoxStream, Stream, StreamExt};

use crate::AiError;

pub(crate) type LineStream = BoxStream<'static, Result<String, AiError>>;

struct State<S> {
    inner: S,
    buf: Vec<u8>,
    finished: bool,
}

/// Turns a byte-chunk stream into a stream of lines (without `\n` / `\r\n`).
/// A trailing line without a newline is emitted when the input ends.
pub(crate) fn lines<S, B, E>(inner: S) -> LineStream
where
    S: Stream<Item = Result<B, E>> + Send + Unpin + 'static,
    B: AsRef<[u8]>,
    E: Display,
{
    let state = State {
        inner,
        buf: Vec::new(),
        finished: false,
    };
    stream::unfold(state, |mut st| async move {
        loop {
            if let Some(pos) = st.buf.iter().position(|b| *b == b'\n') {
                let rest = st.buf.split_off(pos + 1);
                let line = std::mem::replace(&mut st.buf, rest);
                return Some((Ok(decode(&line[..pos])), st));
            }
            if st.finished {
                if st.buf.is_empty() {
                    return None;
                }
                let line = std::mem::take(&mut st.buf);
                return Some((Ok(decode(&line)), st));
            }
            match st.inner.next().await {
                Some(Ok(chunk)) => st.buf.extend_from_slice(chunk.as_ref()),
                Some(Err(e)) => {
                    st.finished = true;
                    st.buf.clear();
                    return Some((Err(AiError::Http(e.to_string())), st));
                }
                None => st.finished = true,
            }
        }
    })
    .boxed()
}

fn decode(line: &[u8]) -> String {
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    String::from_utf8_lossy(line).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn collect(chunks: Vec<Result<&'static str, String>>) -> Vec<Result<String, String>> {
        lines(stream::iter(chunks))
            .map(|r| r.map_err(|e| e.to_string()))
            .collect()
            .await
    }

    #[tokio::test]
    async fn splits_lines_across_chunks() {
        let out = collect(vec![Ok("ab"), Ok("c\r\nde"), Ok("f\n\ng"), Ok("h")]).await;
        let got: Vec<String> = out.into_iter().map(|r| r.unwrap()).collect();
        assert_eq!(got, vec!["abc", "def", "", "gh"]);
    }

    #[tokio::test]
    async fn keeps_multibyte_chars_split_between_chunks() {
        let bytes = "héllo\n".as_bytes();
        let chunks: Vec<Result<Vec<u8>, String>> =
            vec![Ok(bytes[..2].to_vec()), Ok(bytes[2..].to_vec())];
        let got: Vec<_> = lines(stream::iter(chunks)).collect().await;
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].as_ref().unwrap(), "héllo");
    }

    #[tokio::test]
    async fn error_ends_stream() {
        let out = collect(vec![Ok("a\nb"), Err("boom".into()), Ok("c\n")]).await;
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], Ok("a".to_string()));
        assert!(out[1].as_ref().unwrap_err().contains("boom"));
    }
}
