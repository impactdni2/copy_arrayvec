use std::{marker::PhantomData, ops::Deref};

use serde::{de::Visitor, Deserialize, Serialize};

use crate::CopyArrayVec;

impl<T: Copy + Serialize, const C: usize> Serialize for CopyArrayVec<T, C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.deref().serialize(serializer)
    }
}

impl<'de, T: Copy + Deserialize<'de>, const C: usize> Deserialize<'de> for CopyArrayVec<T, C> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visit<T, const C: usize>(PhantomData<fn() -> T>);
        impl<'de, T: Copy + Deserialize<'de>, const C: usize> Visitor<'de> for Visit<T, C> {
            type Value = CopyArrayVec<T, C>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an array of max length {C}")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut arr = Self::Value::default();
                while let Some(el) = seq.next_element()? {
                    if arr.try_push(el).is_err() {
                        return Err(serde::de::Error::invalid_length(
                            arr.capacity() + 1,
                            &"fewer elements in array",
                        ));
                    }
                }
                Ok(arr)
            }
        }
        deserializer.deserialize_seq(Visit(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_de_tokens_error, assert_tokens, Token};

    use crate::CopyArrayVec;

    #[test]
    fn serialize_ints() {
        let mut arr = CopyArrayVec::<_, 3>::new();
        arr.push(0);
        arr.push(1);
        assert_tokens(
            &arr,
            &[
                Token::Seq { len: Some(2) },
                Token::I32(0),
                Token::I32(1),
                Token::SeqEnd,
            ],
        );
    }

    #[test]
    fn fails_to_deserialize_too_large() {
        assert_de_tokens_error::<CopyArrayVec<i32, 1>>(
            &[
                Token::Seq { len: Some(2) },
                Token::I32(0),
                Token::I32(1),
                Token::SeqEnd,
            ],
            "invalid length 2, expected fewer elements in array",
        );
    }
}
