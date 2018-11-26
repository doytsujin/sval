use crate::{
    std::fmt,
    stream,
    value,
};

use super::err;

use serde::ser::{
    Error as SerError,
    Serialize,
    SerializeMap,
    SerializeSeq,
    Serializer,
};

pub(super) struct ToSerialize<V>(pub(super) V);

impl<V> Serialize for ToSerialize<V>
where
    V: value::Value,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut stream = Stream::begin(serializer);
        crate::stream(&self.0, &mut stream).map_err(S::Error::custom)?;

        Ok(stream.expect_ok())
    }
}

struct Stream<S>
where
    S: Serializer,
{
    ok: Option<S::Ok>,
    pos: Option<stream::Pos>,
    current: Option<Current<S>>,
}

enum Current<S>
where
    S: Serializer,
{
    Serializer(S),
    SerializeSeq(S::SerializeSeq),
    SerializeMap(S::SerializeMap),
}

impl<S> Stream<S>
where
    S: Serializer,
{
    fn begin(ser: S) -> Self {
        Stream {
            ok: None,
            pos: Some(stream::Pos::Root),
            current: Some(Current::Serializer(ser)),
        }
    }

    fn expect_ok(self) -> S::Ok {
        self.ok.expect("missing return value")
    }
}

impl<S> Current<S>
where
    S: Serializer,
{
    fn take_serializer(self) -> Result<S, stream::Error> {
        match self {
            Current::Serializer(ser) => Ok(ser),
            _ => Err(stream::Error::msg("invalid serializer value")),
        }
    }

    fn expect_serialize_seq(&mut self) -> Result<&mut S::SerializeSeq, stream::Error> {
        match self {
            Current::SerializeSeq(seq) => Ok(seq),
            _ => Err(stream::Error::msg("invalid serializer value")),
        }
    }

    fn take_serialize_seq(self) -> Result<S::SerializeSeq, stream::Error> {
        match self {
            Current::SerializeSeq(seq) => Ok(seq),
            _ => Err(stream::Error::msg("invalid serializer value")),
        }
    }

    fn expect_serialize_map(&mut self) -> Result<&mut S::SerializeMap, stream::Error> {
        match self {
            Current::SerializeMap(map) => Ok(map),
            _ => Err(stream::Error::msg("invalid serializer value")),
        }
    }

    fn take_serialize_map(self) -> Result<S::SerializeMap, stream::Error> {
        match self {
            Current::SerializeMap(map) => Ok(map),
            _ => Err(stream::Error::msg("invalid serializer value")),
        }
    }
}

impl<S> Stream<S>
where
    S: Serializer,
{
    fn map_serializer<E>(
        &mut self,
        f: impl FnOnce(S) -> Result<Current<S>, E>,
    ) -> Result<(), stream::Error>
    where
        E: serde::ser::Error,
    {
        let serializer = self.take()?.take_serializer()?;
        self.current = Some(f(serializer).map_err(err("error maping serializer"))?);

        Ok(())
    }

    fn take(&mut self) -> Result<Current<S>, stream::Error> {
        self.current
            .take()
            .ok_or(stream::Error::msg("attempt to use an invalid serializer"))
    }

    fn expect(&mut self) -> Result<&mut Current<S>, stream::Error> {
        self.current
            .as_mut()
            .ok_or(stream::Error::msg("attempt to use an invalid serializer"))
    }

    fn primitive(&mut self, v: impl Serialize) -> Result<(), stream::Error> {
        use self::stream::Pos::*;

        match self.pos.take() {
            Some(Key) => {
                let map = self.expect()?.expect_serialize_map()?;
                map.serialize_key(&v)
                    .map_err(err("error map serializing key"))?;

                Ok(())
            }
            Some(Value) => {
                let map = self.expect()?.expect_serialize_map()?;
                map.serialize_value(&v)
                    .map_err(err("error serializing map value"))?;

                Ok(())
            }
            Some(Elem) => {
                let seq = self.expect()?.expect_serialize_seq()?;
                seq.serialize_element(&v)
                    .map_err(err("error serializing sequence element"))?;

                Ok(())
            }
            Some(Root) => {
                let ser = self.take()?.take_serializer()?;
                v.serialize(ser)
                    .map_err(err("error serializing 128bit signed integer"))?;

                Ok(())
            }
            None => Err(stream::Error::msg("attempt to use an invalid serializer")),
        }
    }
}

impl<S> stream::Stream for Stream<S>
where
    S: Serializer,
{
    fn seq_begin(&mut self, len: Option<usize>) -> Result<(), stream::Error> {
        self.map_serializer(|ser| ser.serialize_seq(len).map(|seq| Current::SerializeSeq(seq)))
    }

    fn seq_elem(&mut self) -> Result<(), stream::Error> {
        self.pos = Some(stream::Pos::Elem);

        Ok(())
    }

    fn seq_end(&mut self) -> Result<(), stream::Error> {
        let seq = self.take()?.take_serialize_seq()?;
        self.ok = Some(seq.end().map_err(err("error completing sequence"))?);

        Ok(())
    }

    fn map_begin(&mut self, len: Option<usize>) -> Result<(), stream::Error> {
        self.map_serializer(|ser| ser.serialize_map(len).map(|map| Current::SerializeMap(map)))
    }

    fn map_key(&mut self) -> Result<(), stream::Error> {
        self.pos = Some(stream::Pos::Key);

        Ok(())
    }

    fn map_value(&mut self) -> Result<(), stream::Error> {
        self.pos = Some(stream::Pos::Value);

        Ok(())
    }

    fn map_end(&mut self) -> Result<(), stream::Error> {
        let map = self.take()?.take_serialize_map()?;
        self.ok = Some(map.end().map_err(err("error completing map"))?);

        Ok(())
    }

    fn i64(&mut self, v: i64) -> Result<(), stream::Error> {
        self.primitive(v)
    }

    fn u64(&mut self, v: u64) -> Result<(), stream::Error> {
        self.primitive(v)
    }

    fn i128(&mut self, v: i128) -> Result<(), stream::Error> {
        self.primitive(v)
    }

    fn u128(&mut self, v: u128) -> Result<(), stream::Error> {
        self.primitive(v)
    }

    fn f64(&mut self, v: f64) -> Result<(), stream::Error> {
        self.primitive(v)
    }

    fn bool(&mut self, v: bool) -> Result<(), stream::Error> {
        self.primitive(v)
    }

    fn char(&mut self, v: char) -> Result<(), stream::Error> {
        self.primitive(v)
    }

    fn str(&mut self, v: &str) -> Result<(), stream::Error> {
        self.primitive(v)
    }

    fn none(&mut self) -> Result<(), stream::Error> {
        self.primitive(Option::None::<()>)
    }

    fn fmt(&mut self, v: fmt::Arguments) -> Result<(), stream::Error> {
        self.primitive(v)
    }
}