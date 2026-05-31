use serde::{Deserialize, Deserializer, Serialize};

/// A creature's speed (in feet) on all types of movement.
///
/// Each field is given as a descriptive string, such as "30 ft.".
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Speed {
    /// Basic movement speed.
    #[serde(default, deserialize_with = "deserialize_inner_speed")]
    pub walk: Option<i32>,

    /// Movement speed when moving through sand, earth, mud, or ice.
    #[serde(default, deserialize_with = "deserialize_inner_speed")]
    pub burrow: Option<i32>,

    /// Movement speed when climbing.
    #[serde(default, deserialize_with = "deserialize_inner_speed")]
    pub climb: Option<i32>,

    /// Movement speed when flying.
    #[serde(default, deserialize_with = "deserialize_inner_speed")]
    pub fly: Option<i32>,

    /// Movement speed when swimming.
    #[serde(default, deserialize_with = "deserialize_inner_speed")]
    pub swim: Option<i32>,
}

fn deserialize_inner_speed<'de, D>(d: D) -> Result<Option<i32>, D::Error>
where D: Deserializer<'de>
{
    // api provides speed as: {"walk":"10 ft.","swim":"40 ft."}
    //
    // pretty easy to get what we want
    let result = String::deserialize(d)?
        .split(' ')
        .next()
        .map(|s| s.parse().ok())
        .flatten();
    Ok(result)
}
