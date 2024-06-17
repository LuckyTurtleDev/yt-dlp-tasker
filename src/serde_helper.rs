use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum EVecOrOne<T> {
	One(T),
	Vec(Vec<T>)
}

/// create a none empty vec from sequence or a singlie value
/// can be used with `#[serde(deserialize_with = "vec_or_one")]`
pub fn vec_or_one<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
	D: Deserializer<'de>,
	T: Deserialize<'de>
{
	let value = EVecOrOne::<T>::deserialize(deserializer)?;
	let vec = match value {
		EVecOrOne::Vec(vec) => vec,
		EVecOrOne::One(value) => vec![value]
	};
	if vec.is_empty() {
		return Err(serde::de::Error::custom(
			"cannot deserialize from an empty sequence"
		));
	}
	Ok(vec)
}
