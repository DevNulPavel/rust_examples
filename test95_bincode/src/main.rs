use serde::{
    de::{Deserializer, Visitor},
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};

////////////////////////////////////////////////////////////////////////////////////////////

/// Информация об именованном селекте в базе данных
#[derive(Debug, PartialEq, Eq)]
struct NamedSelect<'a> {
    platform: &'a str,
    name: &'a str,
    request: &'a str,
}

////////////////////////////////////////////////////////////////////////////////////////////

/// Используем собственную реализацию сериализации и десереализации для
/// сохранения одного и того же порядка полей всегда,
/// а также для того, чтобы хранить компактно поля.
impl<'a> Serialize for NamedSelect<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Создаем сериализатор
        let mut struct_serializer = serializer.serialize_struct("NS", 3)?;
        // Сериализуем поля с короткими названиями
        struct_serializer.serialize_field("p", self.platform)?;
        struct_serializer.serialize_field("n", self.name)?;
        struct_serializer.serialize_field("r", self.request)?;
        // Завершение
        struct_serializer.end()
    }
}

/// Используем собственную реализацию сериализации и десереализации для
/// сохранения одного и того же порядка полей всегда,
/// а также для того, чтобы хранить компактно поля.
impl<'a, 'de: 'a> Deserialize<'de> for NamedSelect<'a> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Десереализация с помощью Visitor из-за того, что разные реализации сериализуют по-разному
        deserializer.deserialize_struct("NS", &["p", "n", "r"], StructVisitor {})
    }
}

////////////////////////////////////////////////////////////////////////////////////////////

struct StructVisitor {}

impl<'de> Visitor<'de> for StructVisitor {
    type Value = NamedSelect<'de>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "map or seq with fields 'p', 'n', 'r'")
    }

    /// Подряд идут просто элементы (которые указаны при вызове deserialize_struct?)
    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let platform = seq
            .next_element::<&'de str>()?
            .ok_or(serde::de::Error::missing_field("p"))?;
        let name = seq
            .next_element::<&'de str>()?
            .ok_or(serde::de::Error::missing_field("n"))?;
        let request = seq
            .next_element::<&'de str>()?
            .ok_or(serde::de::Error::missing_field("r"))?;

        let res = NamedSelect {
            platform,
            name,
            request,
        };

        Ok(res)
    }

    /// Реализация сериализации в виде мапы
    fn visit_map<A>(self, mut map: A) -> std::result::Result<NamedSelect<'de>, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut platform: Option<&'de str> = None;
        let mut name: Option<&'de str> = None;
        let mut request: Option<&'de str> = None;

        while let Some(entry) = map.next_entry::<&'de str, &'de str>()? {
            match entry.0 {
                "p" => {
                    platform = Some(entry.1);
                }
                "n" => {
                    name = Some(entry.1);
                }
                "r" => {
                    request = Some(entry.1);
                }
                other => {
                    return Err(serde::de::Error::unknown_field(other, &["p", "n", "r"]));
                }
            }
        }

        let res = NamedSelect {
            platform: platform.ok_or(serde::de::Error::missing_field("p"))?,
            name: name.ok_or(serde::de::Error::missing_field("n"))?,
            request: request.ok_or(serde::de::Error::missing_field("r"))?,
        };

        Ok(res)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////

/// Информация об именованном селекте в базе данных
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct NamedSelectSrc<'a> {
    name: &'a str,
    platform: &'a str,
    request: &'a str,
}

////////////////////////////////////////////////////////////////////////////////////////////

/// Информация об именованном селекте в базе данных
#[derive(Debug, PartialEq, Eq, Deserialize)]
struct NamedSelectReordered<'a> {
    request: &'a str,
    name: &'a str,
    platform: &'a str,
}

////////////////////////////////////////////////////////////////////////////////////////////

fn test_1() {
    let src = NamedSelect {
        name: "name_val",
        platform: "platform_val",
        request: "req_val",
    };

    let serialized_data = bincode::serialize(&src).unwrap();

    let dst: NamedSelect = bincode::deserialize(serialized_data.as_slice()).unwrap();

    assert_eq!(src, dst);
}

fn test_2() {
    let src = NamedSelect {
        name: "name_val",
        platform: "platform_val",
        request: "req_val",
    };

    let serialized_data = serde_json::to_string(&src).unwrap();

    // println!("{serialized_data}");

    let dst: NamedSelect = serde_json::from_str(serialized_data.as_str()).unwrap();

    assert_eq!(src, dst);
}

fn test_3() {
    let src = NamedSelectSrc {
        name: "name_val",
        platform: "platform_val",
        request: "req_val",
    };

    let serialized_data = bincode::serialize(&src).unwrap();

    let dst: NamedSelectSrc = bincode::deserialize(serialized_data.as_slice()).unwrap();

    assert_eq!(src.name, dst.name);
    assert_eq!(src.platform, dst.platform);
    assert_eq!(src.request, dst.request);
}

fn test_4() {
    // ВАЖНО!
    // Bincode сериализует структуру не как мапу, а в виде массива.
    // Поэтому порядок полей важен при разных изменениях.

    let src = NamedSelectSrc {
        name: "name_val",
        platform: "platform_val",
        request: "req_val",
    };

    let serialized_data = bincode::serialize(&src).unwrap();

    let dst: NamedSelectReordered = bincode::deserialize(serialized_data.as_slice()).unwrap();

    dbg!(&dst);

    assert_eq!(src.name, dst.name);
    assert_eq!(src.platform, dst.platform);
    assert_eq!(src.request, dst.request);
}

////////////////////////////////////////////////////////////////////////////////////////////

fn main() {
    test_1();
    test_2();
    test_3();
    test_4();
}
