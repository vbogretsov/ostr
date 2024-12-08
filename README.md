# ostr

Owned const str

## Rationale

This library provides owned `str` instance. It can be used in complex keys
in `HashMap` or `HashSet` without leveraging unstable raw entry API.

Example:

```rust
use ostr::Str;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct SchemaKey {
    subject: Str,
    version: i32,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct SchemaKeyRef<'a> {
    subject: &'a str,
    version: i32,
}

impl<'a> Borrow<SchemaKeyRef<'a>> for SchemaKey {
    fn borrow(&self) -> &SchemaKeyRef<'a> {
        unsafe {
            &*(self as *const SchemaKey as *const SchemaKeyRef)
        }
    }
}

fn main() {
    let mut cache: HashMap<SchemaKey, String> = HashMap::new();
    cache.insert(
        SchemaKey{subject: Str::new("User"), version: 1},
        "User:1".to_string(),
    );
    cache.insert(
        SchemaKey{subject: Str::new("User"), version: 2},
        "User:2".to_string(),
    );

    let key = SchemaKeyRef{subject: "User", version: 1};
    assert_eq!(cache.get(&key), Some(&"User:1".to_string()));
}
```

It is guarantied size of `Str` to be equal to size of `&str`.
