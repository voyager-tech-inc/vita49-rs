// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Data structures and methods related to context association lists
(ANSI/VITA-49.2-2017 section 9.13.2).
*/

use deku::prelude::*;

/// Base context association lists structure.
///
/// The Context Association Lists Section (§9.13.2) associates other packet streams with the
/// described signal by carrying their Stream IDs in up to four lists — Source, System,
/// Vector-Component, and Asynchronous-Channel — plus an optional Asynchronous-Channel Tag
/// list. The first two 32-bit words carry the list sizes (Figure 9.13.2-1); the lists follow.
///
/// The setters keep each list's `Vec` and its size bitfield in sync while preserving the
/// other lists' sizes, so the structure always serializes to a well-formed section.
///
/// # Example
///
/// ```
/// use vita49::ContextAssociationLists;
///
/// // Associate two source streams and one system stream with the described signal.
/// let mut cal = ContextAssociationLists::default();
/// cal.set_source_list(&[0x0000_0001, 0x0000_0002]);
/// cal.set_system_list(&[0x0000_0010]);
///
/// assert_eq!(cal.source_list(), &[0x0000_0001, 0x0000_0002]);
/// assert_eq!(cal.system_list(), &[0x0000_0010]);
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContextAssociationLists {
    w1: u32,
    w2: u32,
    #[deku(count = "((w1 >> 16) & 0x1FF)")]
    source_list: Vec<u32>,
    #[deku(count = "(w1 & 0x1FF)")]
    system_list: Vec<u32>,
    #[deku(count = "(w2 >> 16)")]
    vector_component_list: Vec<u32>,
    #[deku(count = "(w2 & 0x7FFF)")]
    async_channel_list: Vec<u32>,
    #[deku(cond = "(w2 & (1 << 15)) != 0", count = "(w2 & 0x7FFF)")]
    async_channel_tag_list: Vec<u32>,
}

impl ContextAssociationLists {
    /// Get the size of the lists in 32-bit words.
    pub fn size_words(&self) -> u16 {
        // Start with the 2 top words
        let mut ret = 2;
        ret += self.source_list.len();
        ret += self.system_list.len();
        ret += self.vector_component_list.len();
        ret += self.async_channel_list.len();
        ret += self.async_channel_tag_list.len();
        ret as u16
    }

    /// The Source Context Association List — Stream IDs of the input data / upstream
    /// processes (§9.13.2.1).
    pub fn source_list(&self) -> &[u32] {
        &self.source_list
    }

    /// Set the Source Context Association List, updating the Source List Size (word 1 bits
    /// 24:16) and preserving the System List Size.
    ///
    /// # Panics
    ///
    /// Panics if `ids.len() > 511`; Rule 9.13.2-2 caps the Source List Size at 511.
    pub fn set_source_list(&mut self, ids: &[u32]) {
        assert!(
            ids.len() <= 511,
            "source list size must be 0..=511 (Rule 9.13.2-2)"
        );
        self.w1 = (self.w1 & !(0x1FF << 16)) | ((ids.len() as u32) << 16);
        self.source_list = ids.to_vec();
    }

    /// The System Context Association List — Stream IDs of associated system streams
    /// (§9.13.2.2).
    pub fn system_list(&self) -> &[u32] {
        &self.system_list
    }

    /// Set the System Context Association List, updating the System List Size (word 1 bits
    /// 8:0) and preserving the Source List Size.
    ///
    /// # Panics
    ///
    /// Panics if `ids.len() > 511`; Rule 9.13.2-3 caps the System List Size at 511.
    pub fn set_system_list(&mut self, ids: &[u32]) {
        assert!(
            ids.len() <= 511,
            "system list size must be 0..=511 (Rule 9.13.2-3)"
        );
        self.w1 = (self.w1 & !0x1FF) | (ids.len() as u32);
        self.system_list = ids.to_vec();
    }

    /// The Vector-Component Context Association List — Stream IDs of associated
    /// vector-component streams (§9.13.2.3).
    pub fn vector_component_list(&self) -> &[u32] {
        &self.vector_component_list
    }

    /// Set the Vector-Component Context Association List, updating the Vector-Component List
    /// Size (word 2 bits 31:16) and preserving the Asynchronous-Channel fields.
    ///
    /// # Panics
    ///
    /// Panics if `ids.len() > 65_535`; Rule 9.13.2-4 caps the Vector-Component List Size at
    /// 65,535.
    pub fn set_vector_component_list(&mut self, ids: &[u32]) {
        assert!(
            ids.len() <= 65_535,
            "vector-component list size must be 0..=65535 (Rule 9.13.2-4)"
        );
        self.w2 = (self.w2 & 0x0000_FFFF) | ((ids.len() as u32) << 16);
        self.vector_component_list = ids.to_vec();
    }

    /// The Asynchronous-Channel Context Association List — Stream IDs of associated
    /// asynchronous channels (§9.13.2.4).
    pub fn async_channel_list(&self) -> &[u32] {
        &self.async_channel_list
    }

    /// Set the Asynchronous-Channel Context Association List, updating the
    /// Asynchronous-Channel List Size (word 2 bits 14:0) and preserving the Vector-Component
    /// List Size and the "A" bit.
    ///
    /// # Panics
    ///
    /// Panics if `ids.len() > 32_767`; Rule 9.13.2-5 caps the Asynchronous-Channel List Size
    /// at 32,767 (a 15-bit field, just below the "A" bit at bit 15).
    pub fn set_async_channel_list(&mut self, ids: &[u32]) {
        assert!(
            ids.len() <= 32_767,
            "async-channel list size must be 0..=32767 (Rule 9.13.2-5)"
        );
        self.w2 = (self.w2 & !0x7FFF) | (ids.len() as u32);
        self.async_channel_list = ids.to_vec();
    }

    /// The Asynchronous-Channel Tag List — present only when the "A" bit is set (§9.13.2.4);
    /// empty otherwise.
    pub fn async_channel_tag_list(&self) -> &[u32] {
        &self.async_channel_tag_list
    }

    /// Set the Asynchronous-Channel Tag List and the "A" (tag-list-present) bit (word 2 bit
    /// 15).
    ///
    /// Passing a non-empty slice sets the "A" bit; passing an empty slice clears it and drops
    /// the tag list. Per Rule 9.13.2-6, when present the tag list must be the same length as
    /// the Asynchronous-Channel List, so set that list first.
    ///
    /// # Panics
    ///
    /// Panics if `ids` is non-empty and its length differs from the current
    /// Asynchronous-Channel List length.
    pub fn set_async_channel_tag_list(&mut self, ids: &[u32]) {
        if ids.is_empty() {
            self.w2 &= !(1 << 15);
            self.async_channel_tag_list.clear();
        } else {
            assert!(
                ids.len() == self.async_channel_list.len(),
                "tag list length must equal the async-channel list length (Rule 9.13.2-6)"
            );
            self.w2 |= 1 << 15;
            self.async_channel_tag_list = ids.to_vec();
        }
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
    fn to_be_bytes(c: &ContextAssociationLists) -> Vec<u8> {
        let mut out = Vec::new();
        {
            let mut writer = Writer::new(Cursor::new(&mut out));
            c.to_writer(&mut writer, Endian::Big).unwrap();
            writer.finalize().unwrap();
        }
        out
    }

    /// Parse back the big-endian wire bytes.
    fn from_be_bytes(bytes: &[u8]) -> ContextAssociationLists {
        let mut cursor = Cursor::new(bytes);
        let mut reader = Reader::new(&mut cursor);
        ContextAssociationLists::from_reader_with_ctx(&mut reader, Endian::Big).unwrap()
    }

    #[test]
    fn default_is_empty() {
        let c = ContextAssociationLists::default();
        assert!(c.source_list().is_empty());
        assert!(c.system_list().is_empty());
        assert!(c.vector_component_list().is_empty());
        assert!(c.async_channel_list().is_empty());
        assert!(c.async_channel_tag_list().is_empty());
        assert_eq!(c.size_words(), 2);
    }

    #[test]
    fn source_and_system_lists_round_trip() {
        let mut c = ContextAssociationLists::default();
        c.set_source_list(&[0x0000_0001, 0x0000_0002]);
        c.set_system_list(&[0x0000_0010]);

        // 2 header words + 2 source + 1 system.
        assert_eq!(c.size_words(), 5);

        let bytes = to_be_bytes(&c);
        // Word 1 = Source Size (bits 25:16) | System Size (bits 9:0) = (2 << 16) | 1.
        assert_eq!(&bytes[0..4], &0x0002_0001u32.to_be_bytes());
        // Word 2 has no sizes set yet.
        assert_eq!(&bytes[4..8], &0u32.to_be_bytes());

        let parsed = from_be_bytes(&bytes);
        assert_eq!(parsed, c);
        assert_eq!(parsed.source_list(), &[0x0000_0001, 0x0000_0002]);
        assert_eq!(parsed.system_list(), &[0x0000_0010]);
    }

    #[test]
    fn vector_and_async_lists_round_trip() {
        let mut c = ContextAssociationLists::default();
        c.set_vector_component_list(&[0xAAAA_0001, 0xAAAA_0002, 0xAAAA_0003]);
        c.set_async_channel_list(&[0xBBBB_0001, 0xBBBB_0002]);
        // Tag list present: same length as the async-channel list (Rule 9.13.2-6).
        c.set_async_channel_tag_list(&[0xCCCC_0001, 0xCCCC_0002]);

        // 2 header + 3 vector + 2 async + 2 tag.
        assert_eq!(c.size_words(), 9);

        let bytes = to_be_bytes(&c);
        // Word 2 = Vector Size (bits 31:16) | A bit (15) | Async Size (bits 14:0).
        let expected_w2 = (3u32 << 16) | (1 << 15) | 2;
        assert_eq!(&bytes[4..8], &expected_w2.to_be_bytes());

        let parsed = from_be_bytes(&bytes);
        assert_eq!(parsed, c);
        assert_eq!(
            parsed.vector_component_list(),
            &[0xAAAA_0001, 0xAAAA_0002, 0xAAAA_0003]
        );
        assert_eq!(parsed.async_channel_list(), &[0xBBBB_0001, 0xBBBB_0002]);
        assert_eq!(parsed.async_channel_tag_list(), &[0xCCCC_0001, 0xCCCC_0002]);
    }

    #[test]
    fn clearing_tag_list_clears_the_a_bit() {
        let mut c = ContextAssociationLists::default();
        c.set_async_channel_list(&[0xBBBB_0001]);
        c.set_async_channel_tag_list(&[0xCCCC_0001]);
        assert_eq!(c.async_channel_tag_list(), &[0xCCCC_0001]);

        c.set_async_channel_tag_list(&[]);
        assert!(c.async_channel_tag_list().is_empty());

        // "A" bit cleared, so the parsed value omits the tag list.
        let parsed = from_be_bytes(&to_be_bytes(&c));
        assert_eq!(parsed, c);
        assert!(parsed.async_channel_tag_list().is_empty());
        assert_eq!(parsed.async_channel_list(), &[0xBBBB_0001]);
    }

    #[test]
    #[should_panic(expected = "source list size must be 0..=511")]
    fn source_list_over_cap_panics() {
        // Rule 9.13.2-2 caps the Source List Size at 511.
        ContextAssociationLists::default().set_source_list(&vec![0u32; 512]);
    }

    #[test]
    fn async_list_over_511_round_trips() {
        // Regression: the Asynchronous-Channel List Size is a 15-bit field (Rule 9.13.2-5),
        // so a list longer than 511 must survive serialization. A 9-bit size mask would
        // truncate the parsed count to len % 512 (600 -> 88) and corrupt the round trip.
        let ids: Vec<u32> = (0..600).map(|i| 0xDD00_0000 | i).collect();
        let mut c = ContextAssociationLists::default();
        c.set_async_channel_list(&ids);
        let parsed = from_be_bytes(&to_be_bytes(&c));
        assert_eq!(parsed.async_channel_list().len(), 600);
        assert_eq!(parsed.async_channel_list(), ids.as_slice());
    }

    #[test]
    #[should_panic(expected = "async-channel list size must be 0..=32767")]
    fn async_list_over_cap_panics() {
        // Rule 9.13.2-5 caps the Asynchronous-Channel List Size at 32,767.
        ContextAssociationLists::default().set_async_channel_list(&vec![0u32; 32_768]);
    }
}
