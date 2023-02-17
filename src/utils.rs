/// A macro to create new `HashMap`s.
#[macro_export]
macro_rules! hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
            {
                let capacity = hashmap!(@count $($key),*);
                let mut map = std::collections::HashMap::with_capacity(capacity);
                $(
                    let _ = map.insert($key, $value);
                )*
                map
            }
        };
    }
