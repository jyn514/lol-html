use crate::harness::unescape::Unescape;

use encoding_rs::Encoding;
use failure::{ensure, Error};
use rand::{thread_rng, Rng};
use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde_json::error::Error as SerdeError;
use std::env;
use std::fmt::{self, Formatter};

#[derive(Debug, Clone)]
pub struct ChunkedInput {
    input: String,
    chunks: Vec<Vec<u8>>,
    initialized: bool,
    encoding: Option<&'static Encoding>,
}

impl From<String> for ChunkedInput {
    fn from(input: String) -> Self {
        ChunkedInput {
            input,
            chunks: Vec::new(),
            initialized: false,
            encoding: None,
        }
    }
}

impl ChunkedInput {
    pub fn init(&mut self, encoding: &'static Encoding) -> Result<usize, Error> {
        let (bytes, _, had_unmappable_chars) = encoding.encode(&self.input);

        // NOTE: Input had unmappable characters for this encoding which were
        // converted to HTML entities by the encoder. This basically means
        // that such input is impossible with the given encoding, so we just
        // bail.
        ensure!(!had_unmappable_chars, "There were unmappable characters");

        // NOTE: Some encodings deviate from ASCII, e.g. in ShiftJIS yen sign (U+00A5) is
        // mapped to 0x5C which makes conversion from UTF8 to it non-roundtrippable despite the
        // abscence of HTML entities replacements inserted by the encoder.
        ensure!(
            self.input == encoding.decode_without_bom_handling(&bytes).0,
            "ASCII characters deviation"
        );

        let len = bytes.len();

        self.encoding = Some(encoding);

        let chunk_size = match env::var("CHUNK_SIZE") {
            Ok(val) => val.parse().unwrap(),
            Err(_) => {
                if len > 1 {
                    thread_rng().gen_range(1, len)
                } else {
                    len
                }
            }
        };

        if chunk_size > 0 {
            self.chunks = bytes.chunks(chunk_size).map(|c| c.to_vec()).collect()
        }

        Ok(chunk_size)
    }

    pub fn encoding(&self) -> Option<&'static Encoding> {
        self.encoding
    }

    pub fn chunks(&self) -> &[Vec<u8>] {
        &self.chunks
    }

    pub fn as_str(&self) -> &str {
        &self.input
    }
}

impl<'de> Deserialize<'de> for ChunkedInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringVisitor;

        impl<'de> Visitor<'de> for StringVisitor {
            type Value = ChunkedInput;

            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str("a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(value.to_owned().into())
            }
        }

        deserializer.deserialize_string(StringVisitor)
    }
}

impl Unescape for ChunkedInput {
    fn unescape(&mut self) -> Result<(), SerdeError> {
        assert!(
            !self.initialized,
            "Input can't be unescaped after initialization"
        );

        self.input.unescape()?;

        Ok(())
    }
}