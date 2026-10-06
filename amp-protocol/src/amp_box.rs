use crate::{Error, MAX_KEY_LENGTH, MAX_VALUE_LENGTH};

/// One AMP packet: an ordered collection of key/value pairs, each a byte
/// string.
///
/// Keys are unique within a box, between 1 and 255 bytes long, and values are
/// at most 65,535 bytes long. Both limits are enforced when a pair is
/// inserted, so a box that exists can always be encoded. Pairs keep the order
/// they were inserted in, which is the order they are written to the wire.
///
/// On the wire, each pair is a two-byte big-endian key length, the key, a
/// two-byte big-endian value length, and the value. A key length of zero ends
/// the box.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AmpBox {
    entries: Vec<(Vec<u8>, Vec<u8>)>,
}

impl AmpBox {
    /// Creates an empty box.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a box from key/value pairs, in order.
    pub(crate) fn from_pairs<I, K, V>(pairs: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<Vec<u8>>,
        V: Into<Vec<u8>>,
    {
        let mut amp_box = Self::new();
        for (key, value) in pairs {
            amp_box.insert(key, value)?;
        }
        Ok(amp_box)
    }

    /// Inserts a pair, replacing the value of an existing key in place.
    ///
    /// Returns the replaced value, if any. Fails if the key is empty or
    /// longer than 255 bytes, or the value is longer than 65,535 bytes, in
    /// which case the box is unchanged.
    pub fn insert<K, V>(&mut self, key: K, value: V) -> Result<Option<Vec<u8>>, Error>
    where
        K: Into<Vec<u8>>,
        V: Into<Vec<u8>>,
    {
        let key = key.into();
        let value = value.into();
        if key.is_empty() {
            return Err(Error::EmptyKey);
        }
        if key.len() > MAX_KEY_LENGTH {
            return Err(Error::KeyTooLong { len: key.len() });
        }
        if value.len() > MAX_VALUE_LENGTH {
            return Err(Error::ValueTooLong { len: value.len() });
        }
        Ok(self.insert_unchecked(key, value))
    }

    fn insert_unchecked(&mut self, key: Vec<u8>, value: Vec<u8>) -> Option<Vec<u8>> {
        match self.entries.iter_mut().find(|(k, _)| *k == key) {
            Some((_, existing)) => Some(std::mem::replace(existing, value)),
            None => {
                self.entries.push((key, value));
                None
            }
        }
    }

    /// Returns the value stored under `key`.
    pub fn get<K: AsRef<[u8]>>(&self, key: K) -> Option<&[u8]> {
        let key = key.as_ref();
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_slice())
    }

    /// Removes `key` and returns its value, if it was present.
    pub fn remove<K: AsRef<[u8]>>(&mut self, key: K) -> Option<Vec<u8>> {
        let key = key.as_ref();
        let index = self.entries.iter().position(|(k, _)| k == key)?;
        Some(self.entries.remove(index).1)
    }

    /// The number of pairs in the box.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the box has no pairs.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterates over the pairs in insertion order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&[u8], &[u8])> + '_ {
        self.entries
            .iter()
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
    }

    /// The number of bytes [`encode`](Self::encode) will produce.
    pub(crate) fn encoded_len(&self) -> usize {
        self.entries
            .iter()
            .map(|(k, v)| 2 + k.len() + 2 + v.len())
            .sum::<usize>()
            + 2
    }

    /// Encodes the box, including its terminator.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.encoded_len());
        for (key, value) in &self.entries {
            out.extend_from_slice(&(key.len() as u16).to_be_bytes());
            out.extend_from_slice(key);
            out.extend_from_slice(&(value.len() as u16).to_be_bytes());
            out.extend_from_slice(value);
        }
        out.extend_from_slice(&[0, 0]);
        out
    }

    /// Decodes one box from the front of `bytes`.
    ///
    /// Returns the box and the number of bytes it occupied, so a caller
    /// framing a stream can advance past it. Bytes after the box are left
    /// alone. Returns [`Error::Incomplete`] if `bytes` ends before the
    /// terminator, and [`Error::KeyTooLong`] if a key's length prefix exceeds
    /// 255. A key that repeats within one box keeps the last value, matching
    /// the reference implementation.
    pub fn decode(bytes: &[u8]) -> Result<(Self, usize), Error> {
        let mut amp_box = Self::new();
        let mut pos = 0;
        loop {
            let key_len = read_u16(bytes, pos)?;
            pos += 2;
            if key_len == 0 {
                return Ok((amp_box, pos));
            }
            if key_len > MAX_KEY_LENGTH {
                return Err(Error::KeyTooLong { len: key_len });
            }
            let key = read_slice(bytes, pos, key_len)?;
            pos += key_len;

            let value_len = read_u16(bytes, pos)?;
            pos += 2;
            let value = read_slice(bytes, pos, value_len)?;
            pos += value_len;

            amp_box.insert_unchecked(key.to_vec(), value.to_vec());
        }
    }
}

fn read_u16(bytes: &[u8], pos: usize) -> Result<usize, Error> {
    match bytes.get(pos..pos + 2) {
        Some(&[high, low]) => Ok(u16::from_be_bytes([high, low]) as usize),
        _ => Err(Error::Incomplete),
    }
}

fn read_slice(bytes: &[u8], pos: usize, len: usize) -> Result<&[u8], Error> {
    bytes.get(pos..pos + len).ok_or(Error::Incomplete)
}

impl IntoIterator for AmpBox {
    type Item = (Vec<u8>, Vec<u8>);
    type IntoIter = std::vec::IntoIter<(Vec<u8>, Vec<u8>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn sum_request() -> AmpBox {
        AmpBox::from_pairs([("_ask", "1"), ("_command", "Sum"), ("a", "13"), ("b", "81")]).unwrap()
    }

    const SUM_REQUEST: &[u8] =
        b"\x00\x04_ask\x00\x011\x00\x08_command\x00\x03Sum\x00\x01a\x00\x0213\x00\x01b\x00\x0281\x00\x00";

    #[test]
    fn insert_replaces_in_place() {
        let mut amp_box = AmpBox::new();
        assert_eq!(amp_box.insert("a", "1").unwrap(), None);
        assert_eq!(amp_box.insert("b", "2").unwrap(), None);
        assert_eq!(amp_box.insert("a", "3").unwrap(), Some(b"1".to_vec()));
        assert_eq!(amp_box.len(), 2);
        assert_eq!(
            amp_box.iter().collect::<Vec<_>>(),
            vec![(&b"a"[..], &b"3"[..]), (&b"b"[..], &b"2"[..])]
        );
    }

    #[test]
    fn get_contains_remove() {
        let mut amp_box = sum_request();
        assert_eq!(amp_box.get("a"), Some(&b"13"[..]));
        assert_eq!(amp_box.get(b"b"), Some(&b"81"[..]));
        assert_eq!(amp_box.get("missing"), None);
        assert_eq!(amp_box.remove("_ask"), Some(b"1".to_vec()));
        assert_eq!(amp_box.remove("_ask"), None);
        assert_eq!(amp_box.get("_ask"), None);
        assert_eq!(amp_box.len(), 3);
    }

    #[test]
    fn rejects_empty_key() {
        let mut amp_box = AmpBox::new();
        assert!(matches!(amp_box.insert("", "x"), Err(Error::EmptyKey)));
        assert!(amp_box.is_empty());
    }

    #[test]
    fn enforces_key_length_limit() {
        let mut amp_box = AmpBox::new();
        amp_box.insert(vec![b'k'; MAX_KEY_LENGTH], "").unwrap();
        assert!(matches!(
            amp_box.insert(vec![b'k'; MAX_KEY_LENGTH + 1], ""),
            Err(Error::KeyTooLong { len: 256 })
        ));
    }

    #[test]
    fn enforces_value_length_limit() {
        let mut amp_box = AmpBox::new();
        amp_box.insert("k", vec![0; MAX_VALUE_LENGTH]).unwrap();
        assert!(matches!(
            amp_box.insert("k", vec![0; MAX_VALUE_LENGTH + 1]),
            Err(Error::ValueTooLong { len: 65536 })
        ));
        // The failed insert left the old value alone.
        assert_eq!(amp_box.get("k").unwrap().len(), MAX_VALUE_LENGTH);
    }

    #[test]
    fn encodes_reference_request() {
        assert_eq!(sum_request().encode(), SUM_REQUEST);
        assert_eq!(sum_request().encoded_len(), SUM_REQUEST.len());
    }

    #[test]
    fn encodes_empty_box_as_terminator() {
        assert_eq!(AmpBox::new().encode(), b"\x00\x00");
    }

    #[test]
    fn encodes_empty_value() {
        let amp_box = AmpBox::from_pairs([("k", "")]).unwrap();
        assert_eq!(amp_box.encode(), b"\x00\x01k\x00\x00\x00\x00");
    }

    #[test]
    fn encodes_largest_lengths() {
        let amp_box =
            AmpBox::from_pairs([(vec![b'k'; MAX_KEY_LENGTH], vec![b'v'; MAX_VALUE_LENGTH])])
                .unwrap();
        let bytes = amp_box.encode();
        assert_eq!(&bytes[..2], &[0x00, 0xFF]);
        assert_eq!(
            &bytes[2 + MAX_KEY_LENGTH..4 + MAX_KEY_LENGTH],
            &[0xFF, 0xFF]
        );
        assert_eq!(AmpBox::decode(&bytes).unwrap(), (amp_box, bytes.len()));
    }

    #[test]
    fn decodes_reference_request() {
        let (amp_box, consumed) = AmpBox::decode(SUM_REQUEST).unwrap();
        assert_eq!(amp_box, sum_request());
        assert_eq!(consumed, SUM_REQUEST.len());
    }

    #[test]
    fn decode_reports_consumed_and_leaves_the_rest() {
        let mut stream = SUM_REQUEST.to_vec();
        stream.extend_from_slice(b"\x00\x07_answer\x00\x011\x00\x05total\x00\x0294\x00\x00");
        let (first, consumed) = AmpBox::decode(&stream).unwrap();
        assert_eq!(first, sum_request());
        let (second, consumed_second) = AmpBox::decode(&stream[consumed..]).unwrap();
        assert_eq!(second.get("total"), Some(&b"94"[..]));
        assert_eq!(consumed + consumed_second, stream.len());
    }

    #[test]
    fn decode_is_incomplete_at_every_truncation() {
        for len in 0..SUM_REQUEST.len() {
            assert!(
                matches!(AmpBox::decode(&SUM_REQUEST[..len]), Err(Error::Incomplete)),
                "truncated to {len} bytes"
            );
        }
    }

    #[test]
    fn decode_rejects_key_longer_than_255() {
        let bytes = b"\x01\x00";
        assert!(matches!(
            AmpBox::decode(bytes),
            Err(Error::KeyTooLong { len: 256 })
        ));
    }

    #[test]
    fn decode_empty_box() {
        let (amp_box, consumed) = AmpBox::decode(b"\x00\x00rest").unwrap();
        assert!(amp_box.is_empty());
        assert_eq!(consumed, 2);
    }

    #[test]
    fn decode_keeps_last_duplicate() {
        let (amp_box, _) = AmpBox::decode(b"\x00\x01k\x00\x011\x00\x01k\x00\x012\x00\x00").unwrap();
        assert_eq!(amp_box.len(), 1);
        assert_eq!(amp_box.get("k"), Some(&b"2"[..]));
    }

    #[test]
    fn decode_accepts_non_utf8() {
        let (amp_box, _) = AmpBox::decode(b"\x00\x02\xff\xfe\x00\x02\x00\x01\x00\x00").unwrap();
        assert_eq!(amp_box.get(b"\xff\xfe"), Some(&b"\x00\x01"[..]));
    }

    #[test]
    fn round_trips() {
        let amp_box = AmpBox::from_pairs([
            ("empty", b"".to_vec()),
            ("binary", vec![0, 1, 2, 255]),
            ("text", b"h\xc3\xa9llo".to_vec()),
        ])
        .unwrap();
        let bytes = amp_box.encode();
        assert_eq!(AmpBox::decode(&bytes).unwrap(), (amp_box, bytes.len()));
    }
}
