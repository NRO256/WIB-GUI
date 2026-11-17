#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncodingState {
    Auto,
    Utf8,
    Cp932,
}

/// `\r` と `\n` を区切りとして行単位で文字列を切り出し、非ASCII出現時に UTF-8 / CP932 を確定するデコーダ。
pub struct StreamLineDecoder<F>
where
    F: Fn(&[u8]) -> String,
{
    state: TextEncodingState,
    buffer: Vec<u8>,
    cp932_decoder: F,
}

impl<F> StreamLineDecoder<F>
where
    F: Fn(&[u8]) -> String,
{
    pub fn new(cp932_decoder: F) -> Self {
        Self {
            state: TextEncodingState::Auto,
            buffer: Vec::new(),
            cp932_decoder,
        }
    }

    #[cfg(test)]
    pub fn state(&self) -> TextEncodingState {
        self.state
    }

    /// 受信したバイト列を投入し、確定した行のベクタを返す。
    pub fn push_bytes(&mut self, chunk: &[u8]) -> Vec<String> {
        let mut lines = Vec::new();
        for &b in chunk {
            if b == b'\r' || b == b'\n' {
                if !self.buffer.is_empty() {
                    let line =
                        Self::decode_line_bytes(&mut self.state, &self.cp932_decoder, &self.buffer);
                    self.buffer.clear();
                    lines.push(line);
                }
            } else {
                self.buffer.push(b);
                if self.buffer.len() >= 65_536 {
                    let line =
                        Self::decode_line_bytes(&mut self.state, &self.cp932_decoder, &self.buffer);
                    self.buffer.clear();
                    lines.push(line);
                }
            }
        }
        lines
    }

    /// ストリーム終了時に残ったバッファをフラッシュする。
    pub fn finish(&mut self) -> Option<String> {
        if self.buffer.is_empty() {
            return None;
        }
        let line = Self::decode_line_bytes(&mut self.state, &self.cp932_decoder, &self.buffer);
        self.buffer.clear();
        Some(line)
    }

    fn decode_line_bytes(state: &mut TextEncodingState, cp932_decoder: &F, bytes: &[u8]) -> String {
        let has_non_ascii = bytes.iter().any(|&b| b >= 0x80);
        if !has_non_ascii {
            return String::from_utf8_lossy(bytes).into_owned();
        }

        match *state {
            TextEncodingState::Cp932 => cp932_decoder(bytes),
            TextEncodingState::Utf8 => String::from_utf8_lossy(bytes).into_owned(),
            TextEncodingState::Auto => match std::str::from_utf8(bytes) {
                Ok(s) => {
                    *state = TextEncodingState::Utf8;
                    s.to_string()
                }
                Err(_) => {
                    *state = TextEncodingState::Cp932;
                    cp932_decoder(bytes)
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_cp932(bytes: &[u8]) -> String {
        format!("[CP932:{}bytes]", bytes.len())
    }

    #[test]
    fn test_ascii_does_not_lock_encoding_and_splits_cr_lf() {
        let mut dec = StreamLineDecoder::new(mock_cp932);
        let lines = dec.push_bytes(b"Progress 10%\rProgress 20%\r\nDone");
        assert_eq!(lines, vec!["Progress 10%", "Progress 20%"]);
        assert_eq!(dec.state(), TextEncodingState::Auto);
        assert_eq!(dec.finish(), Some("Done".to_string()));
    }

    #[test]
    fn test_cp932_latches_on_invalid_utf8() {
        let mut dec = StreamLineDecoder::new(mock_cp932);
        // 0x83 0x65 ("テ" in CP932, invalid UTF-8)
        let lines = dec.push_bytes(&[b'A', b'B', b'\n', 0x83, 0x65, b'\n']);
        assert_eq!(lines[0], "AB");
        assert_eq!(lines[1], "[CP932:2bytes]");
        assert_eq!(dec.state(), TextEncodingState::Cp932);

        // Once latched to Cp932, even valid UTF-8 sequences go to cp932_decoder
        let next = dec.push_bytes("あ\n".as_bytes());
        assert_eq!(next[0], "[CP932:3bytes]");
    }

    #[test]
    fn test_utf8_latches_on_valid_non_ascii_utf8() {
        let mut dec = StreamLineDecoder::new(mock_cp932);
        let lines = dec.push_bytes("準備中...\r\n".as_bytes());
        assert_eq!(lines, vec!["準備中..."]);
        assert_eq!(dec.state(), TextEncodingState::Utf8);
    }
}
