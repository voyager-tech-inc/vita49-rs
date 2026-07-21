// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Data structures and methods related to the ASCII GPS format
(ANSI/VITA-49.2-2017 section 9.4.7).
*/

use deku::prelude::*;

use crate::VitaError;

/// Base ASCII GPS data structure.
///
/// Some GPS devices emit their fixes as formatted ASCII "sentences" (e.g. NMEA-0183); the
/// GPS ASCII field (§9.4.7) carries those sentences verbatim. Word 1 holds the GPS/INS
/// Manufacturer OUI, word 2 the number of 32-bit ASCII words (Rule 9.4.7-3), and the
/// remaining words the packed sentence bytes.
///
/// The sentence bytes are packed big-endian to match the VITA-49 wire convention: per Figure
/// 9.4.7-1 the first character (Byte 1) occupies bits 31:24 of word 3, so serializing the
/// packet big-endian (as `Vrt` does) lays the bytes out in order Byte 1, Byte 2, …. The
/// setters here pack with [`u32::from_be_bytes`] and the getters unpack with
/// [`u32::to_be_bytes`] so this holds regardless of the host's native endianness.
///
/// # Example
///
/// ```
/// use vita49::GpsAscii;
///
/// // A default value is empty; fill in the OUI and the ASCII sentence(s).
/// let mut gps = GpsAscii::default();
/// gps.set_manufacturer_oui(0x0A_1B2C);
/// gps.set_sentences("$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47\r\n");
///
/// assert_eq!(gps.manufacturer_oui(), 0x0A_1B2C);
/// assert_eq!(
///     gps.sentences().unwrap(),
///     "$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47\r\n"
/// );
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpsAscii {
    w1: u32,
    num_words: u32,
    #[deku(count = "num_words")]
    ascii: Vec<u32>,
}

impl GpsAscii {
    /// Gets the size of the ASCII GPS field in 32-bit words.
    pub fn size_words(&self) -> u16 {
        (((std::mem::size_of_val(&self.w1) + std::mem::size_of_val(&self.num_words))
            / std::mem::size_of::<u32>())
            + self.num_words as usize) as u16
    }

    /// The GPS/INS Manufacturer OUI (Rule 9.4.7-2); `FF-FF-FF` when unknown (Perm 9.4.5-2).
    pub fn manufacturer_oui(&self) -> u32 {
        self.w1 & 0x00FF_FFFF
    }
    /// Set the GPS/INS Manufacturer OUI (low 24 bits used).
    pub fn set_manufacturer_oui(&mut self, oui: u32) {
        self.w1 = (self.w1 & 0xFF00_0000) | (oui & 0x00FF_FFFF);
    }

    /// The raw ASCII sentence bytes (words 3…N+2), including any trailing NUL padding.
    ///
    /// Each 32-bit word is unpacked big-endian ([`u32::to_be_bytes`]) so the bytes come back
    /// in wire order Byte 1, Byte 2, … (Figure 9.4.7-1). Internal: callers use
    /// [`sentences`](Self::sentences), which validates UTF-8.
    fn ascii_bytes(&self) -> Vec<u8> {
        self.ascii.iter().flat_map(|w| w.to_be_bytes()).collect()
    }

    /// Set the raw ASCII sentence bytes, updating the Number of Words subfield.
    ///
    /// The bytes are NUL-padded (`0x00`) up to a multiple of four (Rule 9.4.7-5) and packed
    /// four-per-word big-endian ([`u32::from_be_bytes`]) so that, once the packet is
    /// serialized big-endian, Byte 1 lands in bits 31:24 of word 3 (Figure 9.4.7-1).
    /// `num_words` is set to the resulting number of 32-bit words (Rule 9.4.7-3). Internal:
    /// callers use [`set_sentences`](Self::set_sentences), which enforces ASCII.
    fn set_ascii_bytes(&mut self, bytes: &[u8]) {
        let words: Vec<u32> = bytes
            .chunks(4)
            .map(|chunk| {
                // Short final chunk stays NUL-padded via the zero-initialized buffer.
                let mut buf = [0u8; 4];
                buf[..chunk.len()].copy_from_slice(chunk);
                u32::from_be_bytes(buf)
            })
            .collect();
        self.num_words = words.len() as u32;
        self.ascii = words;
    }

    /// Set the ASCII GPS sentence(s), e.g. an NMEA-0183 string (Rule 9.4.7-4).
    ///
    /// Multiple sentences may be concatenated into a single field (Permission 9.4.7-1). The
    /// bytes are NUL-padded to a 32-bit boundary and `num_words` is updated (Rule 9.4.7-3).
    ///
    /// # Panics
    ///
    /// Panics if `s` is not ASCII. Rule 9.4.7-4 requires the field to carry complete ASCII
    /// sentences, so non-ASCII input (e.g. UTF-8 multibyte) is rejected here rather than
    /// written to the wire.
    pub fn set_sentences(&mut self, s: &str) {
        assert!(
            s.is_ascii(),
            "GPS ASCII sentences must be ASCII (Rule 9.4.7-4)"
        );
        self.set_ascii_bytes(s.as_bytes());
    }

    /// The ASCII GPS sentence(s) as a string, with trailing NUL padding stripped.
    ///
    /// # Errors
    ///
    /// Returns [`VitaError::InvalidAscii`] if the payload bytes are not valid UTF-8 — e.g. a
    /// malformed or non-conforming received packet. Conforming ASCII sentences (Rule 9.4.7-4)
    /// always decode.
    pub fn sentences(&self) -> Result<String, VitaError> {
        let bytes = self.ascii_bytes();
        let end = bytes.iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
        Ok(String::from_utf8(bytes[..end].to_vec())?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deku::ctx::Endian;
    use deku::reader::Reader;
    use deku::writer::Writer;
    use std::io::Cursor;

    /// Serialize big-endian, matching the VITA-49 wire convention (`Vrt` is `endian = "big"`).
    fn to_be_bytes(g: &GpsAscii) -> Vec<u8> {
        let mut out = Vec::new();
        {
            let mut writer = Writer::new(Cursor::new(&mut out));
            g.to_writer(&mut writer, Endian::Big).unwrap();
            writer.finalize().unwrap();
        }
        out
    }

    /// Parse back the big-endian wire bytes.
    fn from_be_bytes(bytes: &[u8]) -> GpsAscii {
        let mut cursor = Cursor::new(bytes);
        let mut reader = Reader::new(&mut cursor);
        GpsAscii::from_reader_with_ctx(&mut reader, Endian::Big).unwrap()
    }

    #[test]
    fn default_is_empty() {
        let g = GpsAscii::default();
        assert_eq!(g.manufacturer_oui(), 0);
        assert_eq!(g.sentences().unwrap(), "");
        assert!(g.ascii_bytes().is_empty());
        // Just the two header words.
        assert_eq!(g.size_words(), 2);
    }

    #[test]
    fn nmea_sentence_round_trips_on_the_wire() {
        let sentence = "$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47\r\n";
        let mut g = GpsAscii::default();
        g.set_manufacturer_oui(0x0A_1B2C);
        g.set_sentences(sentence);

        // Rule 9.4.7-3: number of 32-bit words needed to convey the sentence.
        let expected_words = ((sentence.len() + 3) / 4) as u32;
        assert_eq!(g.sentences().unwrap(), sentence);
        assert_eq!(g.size_words(), (2 + expected_words) as u16);

        let bytes = to_be_bytes(&g);
        // Word 1 = OUI, word 2 = num_words, big-endian.
        assert_eq!(&bytes[0..4], &[0x00, 0x0A, 0x1B, 0x2C]);
        assert_eq!(&bytes[4..8], &expected_words.to_be_bytes());
        // Words 3.. carry the ASCII in wire order Byte 1..Byte 4 (Figure 9.4.7-1).
        assert_eq!(&bytes[8..8 + sentence.len()], sentence.as_bytes());

        let parsed = from_be_bytes(&bytes);
        assert_eq!(parsed, g);
        assert_eq!(parsed.manufacturer_oui(), 0x0A_1B2C);
        assert_eq!(parsed.sentences().unwrap(), sentence);
    }

    #[test]
    fn non_word_aligned_sentence_is_nul_padded() {
        // Length 5 is not a multiple of 4, so it pads to 8 bytes / 2 words.
        let sentence = "$GPGG";
        assert_ne!(
            sentence.len() % 4,
            0,
            "guard: sentence length must be non-aligned"
        );
        let mut g = GpsAscii::default();
        g.set_sentences(sentence);

        assert_eq!(g.ascii_bytes(), b"$GPGG\0\0\0");
        assert_eq!(g.sentences().unwrap(), sentence);

        let parsed = from_be_bytes(&to_be_bytes(&g));
        assert_eq!(parsed, g);
        assert_eq!(parsed.sentences().unwrap(), sentence);
        // Padding word count = ceil(5 / 4) = 2.
        assert_eq!(parsed.size_words(), 4);
    }

    #[test]
    #[should_panic(expected = "GPS ASCII sentences must be ASCII")]
    fn non_ascii_sentence_panics() {
        // Rule 9.4.7-4: the field carries complete ASCII sentences only.
        GpsAscii::default().set_sentences("café");
    }

    #[test]
    fn non_utf8_payload_is_flagged() {
        // A received packet carrying non-UTF-8 bytes surfaces an error rather than
        // silently substituting the replacement character.
        let mut g = GpsAscii::default();
        g.set_ascii_bytes(&[0xFF, 0xFE, 0x00, 0x00]);
        assert!(matches!(g.sentences(), Err(VitaError::InvalidAscii(_))));
    }
}
