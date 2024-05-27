// partially (c) 2016 Google Inc. (Lewin Bormann, lewinb@google.com)
//
// See project root for licensing information.
//
use futures::{
    lock::{
        Mutex
    }
};
use std::{
    io,
    path::{
        Path, 
        PathBuf
    },
    collections::{
        HashMap
    }
};
use serde::{
    Deserialize, 
    Serialize
};
use crate::{
    types::{
        TokenInfo
    }
};

// Слой storage возволяет заполучать токены для scope, которые предварительно были одобрены
// Одним из особенностей является то, что токен одобренный для набора
// скоупов X так же валидный для любого подмножества X скоупов.
// Так что при получении токена для набора скоупов, предоставляемых вызывающей стороной, выгоднее сравнить
// набор предыдущих сохраненных токенов, чтобы увидеть, что это поднабор других существующих токенов.
// Чтобы делать это эффективно, мы храним bloom filter вместе с каждым токеном, который представляет 
// набор скоупов, с которым ассоциирован токен.
// Фильтр блума позволяет эффективно пропускать любы сущности, которые определенно не являеются подмножеством.
// Данная реализация использует 64х битный фильтр блума с 4мя хэш функциями.

/// ScopeHash - это значение хэша, унаследованное из списка скоупов.
/// Значение хэша представляет собой отпечаток набора скоупов, независимо от порядка???
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct ScopeHash(u64);

/////////////////////////////////////////////////////////////////////////////////////////////////////////

/// ScopeFilter представляет собой фильтр для набора скоупов.
/// Он может определенно доказать, что данный список скоупов не поднабор других скоупов.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct ScopeFilter(u64);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum FilterResponse {
    Maybe,
    No,
}

impl ScopeFilter {
    /// Определяем, что ScopeFilter может быть набором предоставленного фильтра
    fn is_subset_of(self, filter: ScopeFilter) -> FilterResponse {
        if self.0 & filter.0 == self.0 {
            FilterResponse::Maybe
        } else {
            FilterResponse::No
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct ScopeSet<'a, T> {
    hash: ScopeHash,
    filter: ScopeFilter,
    scopes: &'a [T],
}

// Реализуем клонирование вручную, авто-наследование фейлится так как нам нужно, чтобы каждый элемент был клонируемым.
// А так, мы создаем новый сет вне зависимости от поддержки клонирования
impl<'a, T> Clone for ScopeSet<'a, T> {
    fn clone(&self) -> Self {
        ScopeSet {
            hash: self.hash,
            filter: self.filter,
            scopes: self.scopes,
        }
    }
}

impl<'a, T> Copy for ScopeSet<'a, T> {
}

impl<'a, T> ScopeSet<'a, T>
where
    T: AsRef<str>,
{
    // implement an inherent from method even though From is implemented. This
    // is because passing an array ref like &[&str; 1] (&["foo"]) will be auto
    // deref'd to a slice on function boundaries, but it will not implement the
    // From trait. This inherent method just serves to auto deref from array
    // refs to slices and proxy to the From impl.
    pub fn from(scopes: &'a [T]) -> Self {
        // Перебираем все скоупы, сладываем их в один хэш с фильтром
        let (hash, filter) = scopes
            .iter()
            .fold((ScopeHash(0), ScopeFilter(0)), |(mut scope_hash, mut scope_filter), scope| {
                let h = seahash::hash(scope.as_ref().as_bytes());

                // Используем первые 4 битных чанка из seahash как 4 хэш значения
                // фильтра блума
                for i in 0..4 {
                    // h is a hash derived value in the range 0..64
                    let h = (h >> (6 * i)) & 0b11_1111;
                    scope_filter.0 |= 1 << h;
                }

                // xor the hashes together to get an order independent fingerprint.
                // Ксорим хэши вместе, чтобы получить порядко-независимый отпечаток
                scope_hash.0 ^= h;
                (scope_hash, scope_filter)
            });
        ScopeSet {
            hash,
            filter,
            scopes,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) enum Storage {
    Memory { 
        tokens: Mutex<JSONTokens> 
    },
    Disk(DiskStorage),
}

impl Storage {
    pub(crate) async fn set<T>(&self,
                               scopes: ScopeSet<'_, T>,
                               token: TokenInfo) -> Result<(), io::Error>
    where
        T: AsRef<str>,
    {
        // В зависимости от типа хранилища устанавливаем куда надо
        match self {
            Storage::Memory { tokens } => {
                tokens
                    .lock()
                    .await
                    .set(scopes, token)
            },
            Storage::Disk(disk_storage) => {
                disk_storage
                    .set(scopes, token)
                    .await
            },
        }
    }

    pub(crate) async fn get<T>(&self, scopes: ScopeSet<'_, T>) -> Option<TokenInfo>
    where
        T: AsRef<str>,
    {
        match self {
            Storage::Memory { tokens } => {
                tokens
                    .lock()
                    .await
                    .get(scopes)
            },
            Storage::Disk(disk_storage) => {
                disk_storage
                    .get(scopes)
                    .await
            },
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////

/// A single stored token.

#[derive(Debug, Clone)]
struct JSONToken {
    scopes: Vec<String>,
    token: TokenInfo,
    hash: ScopeHash,
    filter: ScopeFilter,
}

impl<'de> Deserialize<'de> for JSONToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawJSONToken {
            scopes: Vec<String>,
            token: TokenInfo,
        }
        let RawJSONToken { scopes, token } = RawJSONToken::deserialize(deserializer)?;
        let ScopeSet { hash, filter, .. } = ScopeSet::from(&scopes);
        Ok(JSONToken {
            scopes,
            token,
            hash,
            filter,
        })
    }
}

impl Serialize for JSONToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct RawJSONToken<'a> {
            scopes: &'a [String],
            token: &'a TokenInfo,
        }
        RawJSONToken {
            scopes: &self.scopes,
            token: &self.token,
        }
        .serialize(serializer)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////

/// List of tokens in a JSON object
#[derive(Debug, Clone)]
pub(crate) struct JSONTokens {
    token_map: HashMap<ScopeHash, JSONToken>,
}

impl Serialize for JSONTokens {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.token_map.values())
    }
}

impl<'de> Deserialize<'de> for JSONTokens {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = JSONTokens;

            // Format a message stating what data this Visitor expects to receive.
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of JSONToken's")
            }

            fn visit_seq<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::SeqAccess<'de>,
            {
                let mut token_map = HashMap::with_capacity(access.size_hint().unwrap_or(0));
                while let Some(json_token) = access.next_element::<JSONToken>()? {
                    token_map.insert(json_token.hash, json_token);
                }
                Ok(JSONTokens { token_map })
            }
        }

        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data.
        deserializer.deserialize_seq(V)
    }
}

impl JSONTokens {
    pub(crate) fn new() -> Self {
        JSONTokens {
            token_map: HashMap::new(),
        }
    }

    async fn load_from_file(filename: &Path) -> Result<Self, io::Error> {
        let contents = tokio::fs::read(filename).await?;
        serde_json::from_slice(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn get<T>(
        &self,
        ScopeSet {
            hash,
            filter,
            scopes,
        }: ScopeSet<T>,
    ) -> Option<TokenInfo>
    where
        T: AsRef<str>,
    {
        if let Some(json_token) = self.token_map.get(&hash) {
            return Some(json_token.token.clone());
        }

        let requested_scopes_are_subset_of = |other_scopes: &[String]| {
            scopes
                .iter()
                .all(|s| other_scopes.iter().any(|t| t.as_str() == s.as_ref()))
        };
        // No exact match for the scopes provided. Search for any tokens that
        // exist for a superset of the scopes requested.
        self.token_map
            .values()
            .filter(|json_token| filter.is_subset_of(json_token.filter) == FilterResponse::Maybe)
            .find(|v: &&JSONToken| requested_scopes_are_subset_of(&v.scopes))
            .map(|t: &JSONToken| t.token.clone())
    }

    fn set<T>(
        &mut self,
        ScopeSet {
            hash,
            filter,
            scopes,
        }: ScopeSet<T>,
        token: TokenInfo,
    ) -> Result<(), io::Error>
    where
        T: AsRef<str>,
    {
        use std::collections::hash_map::Entry;
        match self.token_map.entry(hash) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().token = token;
            }
            Entry::Vacant(entry) => {
                let json_token = JSONToken {
                    scopes: scopes.iter().map(|x| x.as_ref().to_owned()).collect(),
                    token,
                    hash,
                    filter,
                };
                entry.insert(json_token.clone());
            }
        }
        Ok(())
    }
}

pub(crate) struct DiskStorage {
    tokens: Mutex<JSONTokens>,
    filename: PathBuf,
}

impl DiskStorage {
    pub(crate) async fn new(filename: PathBuf) -> Result<Self, io::Error> {
        let tokens = match JSONTokens::load_from_file(&filename).await {
            Ok(tokens) => tokens,
            Err(e) if e.kind() == io::ErrorKind::NotFound => JSONTokens::new(),
            Err(e) => return Err(e),
        };

        Ok(DiskStorage {
            tokens: Mutex::new(tokens),
            filename,
        })
    }

    pub(crate) async fn set<T>(
        &self,
        scopes: ScopeSet<'_, T>,
        token: TokenInfo,
    ) -> Result<(), io::Error>
    where
        T: AsRef<str>,
    {
        use tokio::io::AsyncWriteExt;
        let json = {
            use std::ops::Deref;
            let mut lock = self.tokens.lock().await;
            lock.set(scopes, token)?;
            serde_json::to_string(lock.deref())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        };
        let mut f = open_writeable_file(&self.filename).await?;
        f.write_all(json.as_bytes()).await?;
        Ok(())
    }

    pub(crate) async fn get<T>(&self, scopes: ScopeSet<'_, T>) -> Option<TokenInfo>
    where
        T: AsRef<str>,
    {
        self.tokens.lock().await.get(scopes)
    }
}

#[cfg(unix)]
async fn open_writeable_file(
    filename: impl AsRef<Path>,
) -> Result<tokio::fs::File, tokio::io::Error> {
    // Ensure if the file is created it's only readable and writable by the
    // current user.
    use std::os::unix::fs::OpenOptionsExt;
    let opts: tokio::fs::OpenOptions = {
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true).mode(0o600);
        opts.into()
    };
    opts.open(filename).await
}

#[cfg(not(unix))]
async fn open_writeable_file(
    filename: impl AsRef<Path>,
) -> Result<tokio::fs::File, tokio::io::Error> {
    // I don't have knowledge of windows or other platforms to know how to
    // create a file that's only readable by the current user.
    tokio::fs::File::create(filename).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_filter() {
        let foo = ScopeSet::from(&["foo"]).filter;
        let bar = ScopeSet::from(&["bar"]).filter;
        let foobar = ScopeSet::from(&["foo", "bar"]).filter;

        // foo and bar are both subsets of foobar. This condition should hold no
        // matter what changes are made to the bloom filter implementation.
        assert!(foo.is_subset_of(foobar) == FilterResponse::Maybe);
        assert!(bar.is_subset_of(foobar) == FilterResponse::Maybe);

        // These conditions hold under the current bloom filter implementation
        // because "foo" and "bar" don't collide, but if the bloom filter
        // implementations change it could be valid for them to return Maybe.
        assert!(foo.is_subset_of(bar) == FilterResponse::No);
        assert!(bar.is_subset_of(foo) == FilterResponse::No);
        assert!(foobar.is_subset_of(foo) == FilterResponse::No);
        assert!(foobar.is_subset_of(bar) == FilterResponse::No);
    }

    #[tokio::test]
    async fn test_disk_storage() {
        let new_token = |access_token: &str| TokenInfo {
            access_token: access_token.to_owned(),
            refresh_token: None,
            expires_at: None,
        };
        let scope_set = ScopeSet::from(&["myscope"]);
        let tempdir = tempfile::tempdir().unwrap();
        {
            let storage = DiskStorage::new(tempdir.path().join("tokenstorage.json"))
                .await
                .unwrap();
            assert!(storage.get(scope_set).await.is_none());
            storage
                .set(scope_set, new_token("my_access_token"))
                .await
                .unwrap();
            assert_eq!(
                storage.get(scope_set).await,
                Some(new_token("my_access_token"))
            );
        }
        {
            // Create a new DiskStorage instance and verify the tokens were read from disk correctly.
            let storage = DiskStorage::new(tempdir.path().join("tokenstorage.json"))
                .await
                .unwrap();
            assert_eq!(
                storage.get(scope_set).await,
                Some(new_token("my_access_token"))
            );
        }
    }
}
