//! Framing for boxes on a byte stream, via [`tokio_util::codec`].
//!
//! Enabled by the `tokio` feature.

use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::{AmpBox, Error};

/// Splits a byte stream into boxes and writes boxes back out.
///
/// Decoding yields an [`AmpBox`] per packet, and encoding writes one.
///
/// ```
/// use amp_protocol::codec::AmpCodec;
/// use tokio_util::codec::Framed;
/// # fn framed<S: tokio::io::AsyncRead + tokio::io::AsyncWrite>(stream: S) {
/// let framed = Framed::new(stream, AmpCodec::new());
/// # }
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct AmpCodec {
    _private: (),
}

impl AmpCodec {
    /// Creates a codec.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Decoder for AmpCodec {
    type Item = AmpBox;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<AmpBox>, Error> {
        match AmpBox::decode(src) {
            Ok((amp_box, consumed)) => {
                src.advance(consumed);
                Ok(Some(amp_box))
            }
            Err(Error::Incomplete) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl Encoder<AmpBox> for AmpCodec {
    type Error = Error;

    fn encode(&mut self, item: AmpBox, dst: &mut BytesMut) -> Result<(), Error> {
        dst.reserve(item.encoded_len());
        for (key, value) in item.iter() {
            dst.put_u16(key.len() as u16);
            dst.put_slice(key);
            dst.put_u16(value.len() as u16);
            dst.put_slice(value);
        }
        dst.put_u16(0);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const SUM_REQUEST: &[u8] =
        b"\x00\x04_ask\x00\x011\x00\x08_command\x00\x03Sum\x00\x01a\x00\x0213\x00\x01b\x00\x0281\x00\x00";

    #[test]
    fn decodes_boxes_as_bytes_arrive() {
        let mut codec = AmpCodec::new();
        let mut buf = BytesMut::new();

        buf.extend_from_slice(&SUM_REQUEST[..5]);
        assert!(codec.decode(&mut buf).unwrap().is_none());
        assert_eq!(buf.len(), 5, "an incomplete box is left in the buffer");

        buf.extend_from_slice(&SUM_REQUEST[5..]);
        buf.extend_from_slice(&SUM_REQUEST[..3]);
        let amp_box = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(amp_box.get("_command"), Some(&b"Sum"[..]));
        assert_eq!(buf.len(), 3, "the next box's bytes are kept");

        buf.extend_from_slice(&SUM_REQUEST[3..]);
        assert!(codec.decode(&mut buf).unwrap().is_some());
        assert!(buf.is_empty());
        assert!(codec.decode(&mut buf).unwrap().is_none());
    }

    #[test]
    fn decode_surfaces_malformed_input() {
        let mut codec = AmpCodec::new();
        let mut buf = BytesMut::from(&b"\x01\x00"[..]);
        assert!(matches!(
            codec.decode(&mut buf),
            Err(Error::KeyTooLong { len: 256 })
        ));
    }

    #[test]
    fn decode_eof_with_partial_box_is_an_error() {
        let mut codec = AmpCodec::new();
        let mut buf = BytesMut::from(&SUM_REQUEST[..5]);
        assert!(matches!(codec.decode_eof(&mut buf), Err(Error::Io(_))));
    }

    #[test]
    fn encodes_box() {
        let mut codec = AmpCodec::new();
        let mut buf = BytesMut::new();
        let (amp_box, _) = AmpBox::decode(SUM_REQUEST).unwrap();
        codec.encode(amp_box, &mut buf).unwrap();
        assert_eq!(&buf[..], SUM_REQUEST);
    }
}
