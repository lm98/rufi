use crate::path::Path;
use sede::{deserialize_rc_box_any_map, serialize_rc_box_any_map};
use serde::Deserialize;
use serde::Serialize;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use std::str::FromStr;

/// Represents the Result of a query made to the Export.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Abstraction for the result of local computation.
/// It is an AST decorated with the computation value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Export {
    #[serde(
        serialize_with = "serialize_rc_box_any_map",
        deserialize_with = "deserialize_rc_box_any_map"
    )]
    map: HashMap<Path, Rc<Box<dyn Any>>>,
}

#[macro_export]
macro_rules! export {
        ($($x:expr),*) => {{
            let mut temp_map = std::collections::HashMap::new();
            $(
                temp_map.insert($x.0, std::rc::Rc::new(Box::new($x.1) as Box<dyn Any>));
            )*
            Export::from(temp_map)
        }};
    }

impl Export {
    /// Create new Export.
    ///
    /// # Returns
    ///
    /// The new Export.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Inserts a value in the Export at the given Path.
    ///
    /// # Arguments
    ///
    /// * `path` - The Path where to insert the value.
    /// * `value` - The value to insert.
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of the value to insert. It must have a `'static` lifetime.
    pub fn put<A: 'static>(&mut self, path: Path, value: A) {
        self.map.insert(path, Rc::new(Box::new(value)));
    }

    /// Inserts a value in the Export at the given Path. The value is calculated from the provided
    /// function.
    ///
    /// # Arguments
    ///
    /// * `path` - The Path where to insert the value.
    /// * `fun` - The function from which we calculate the value to insert.
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of the value to insert. It must have a `'static` lifetime.
    /// * `F` - The type of the function from which the value is calculated.
    pub fn put_lazy<A: 'static, F>(&mut self, path: Path, fun: F)
    where
        F: FnOnce() -> A,
    {
        let value = fun();
        self.put(path, value);
    }

    /// Inserts a value in the Export at the given Path. The value is calculated from the provided
    /// function and then returned to the caller.
    ///
    /// # Arguments
    ///
    /// * `path` - The Path where to insert the value.
    /// * `fun` - The function from which we calculate the value to insert.
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of the value to insert. It must have a `'static` lifetime.
    /// * `F` - The type of the function from which the value is calculated.
    ///
    /// # Returns
    ///
    /// The calculated value.
    pub fn put_lazy_and_return<A: 'static + Clone, F>(&mut self, path: Path, fun: F) -> A
    where
        F: FnOnce() -> A,
    {
        let value = fun();
        self.put(path, value.clone());
        value
    }

    /// Returns the value at the given Path.
    ///
    /// # Arguments
    ///
    /// * `path` - The Path where to get the value.
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of the value to get  to return. It must have a `'static` lifetime.
    ///
    /// # Returns
    ///
    /// The value at the given Path.
    pub fn get<A: 'static + FromStr + Clone>(&self, path: &Path) -> Result<A> {
        let get_result: Result<&A> = self.get_from_map::<A>(path);

        match get_result {
            Ok(any_val) => Ok(any_val.clone()),
            _ => {
                // get deserialized value
                let str_result = self.get_from_map::<String>(path);
                str_result?.parse::<A>().map_err(|_| "Cannot parse".into())
            }
        }
    }

    fn get_from_map<A>(&self, path: &Path) -> Result<&A>
    where
        A: 'static + FromStr + Clone,
    {
        self.map
            .get(path)
            .and_then(|value| value.downcast_ref::<A>())
            .ok_or("No value at the given Path".into())
    }

    /// Obtain the root value. This function may panic, so it is preferable to use the non-panicking
    /// counterpart [Export::root_as_result]
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of the value to return. It must have a `'static` lifetime.
    ///
    /// # Returns
    ///
    /// The root value.
    ///
    /// # Panics
    /// * Panics if there is not a root value (a value at the empty Path).
    /// * Panics if the type of the root value is not the same as the type of the requested value.
    pub fn root<A: 'static + FromStr + Clone>(&self) -> A {
        self.get(&Path::new()).unwrap()
    }

    /// Obtain the root value. This method is the non-panicking version of [Export::root]
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of the value to return. It must have a `'static` lifetime.
    ///
    /// # Returns
    ///
    /// A Result containing the root value if present and an error otherwise.
    pub fn root_as_result<A: 'static + FromStr + Clone>(&self) -> Result<A> {
        self.get(&Path::new())
    }

    /// Returns the HashMap of the Export.
    ///
    /// # Returns
    ///
    /// The HashMap of the Export.
    pub fn paths(&self) -> &HashMap<Path, Rc<Box<dyn Any>>> {
        &self.map
    }
}

impl Default for Export {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<Path, Rc<Box<dyn Any>>>> for Export {
    fn from(map: HashMap<Path, Rc<Box<dyn Any>>>) -> Self {
        Self { map }
    }
}

impl Display for Export {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = serde_json::to_string(&self);
        write!(f, "{}", string.unwrap())
    }
}

impl PartialEq for Export {
    fn eq(&self, other: &Self) -> bool {
        let keys__len_equality = if self.map.keys().len() == other.map.keys().len() {
            true
        } else {
            false
        };

        let values_equality = self.map.iter().all(|(key, _value)| {
            if let Ok(value) = self.get::<i32>(key) {
                if let Ok(other_value) = other.get::<i32>(key) {
                    value == other_value
                } else {
                    false
                }
            } else if let Ok(value) = self.get::<bool>(key) {
                if let Ok(other_value) = other.get::<bool>(key) {
                    value == other_value
                } else {
                    false
                }
            } else if let Ok(value) = self.get::<String>(key) {
                if let Ok(other_value) = other.get::<String>(key) {
                    value == other_value
                } else {
                    false
                }
            } else if let Ok(value) = self.get::<f64>(key) {
                if let Ok(other_value) = other.get::<f64>(key) {
                    value == other_value
                } else {
                    false
                }
            } else {
                false
            }
        });

        keys__len_equality && values_equality
    }
}

/// This private module is needed to serialize and deserialize the HashMap<Path, Rc<Box<dyn Any>>>.
mod sede {
    use crate::path::Path;
    use serde::de::Visitor;
    use serde::{Deserializer, Serialize, Serializer};
    use std::any::Any;
    use std::collections::HashMap;
    use std::rc::Rc;

    pub fn serialize_rc_box_any_map<S>(
        data: &HashMap<Path, Rc<Box<dyn Any>>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the data and wrap it in a HashMap<Path, [u8]>
        let serializable_data: HashMap<String, String> = data
            .iter()
            .map(|(key, value)| {
                let key_str = serde_json::to_string(key).unwrap();
                // if value is an i32, cast as String, otherwise panic
                if let Some(value) = value.downcast_ref::<i32>() {
                    (key_str, value.clone().to_string())
                } else if let Some(value) = value.downcast_ref::<bool>() {
                    (key_str, value.clone().to_string())
                } else if let Some(value) = value.downcast_ref::<String>() {
                    (key_str, value.clone())
                } else if let Some(value) = value.downcast_ref::<f64>() {
                    (key_str, value.clone().to_string())
                } else {
                    panic!("Cannot serialize type")
                }
            })
            .collect();
        serializable_data.serialize(serializer)
    }

    struct ExportMapVisitor;

    impl<'de> Visitor<'de> for ExportMapVisitor {
        type Value = HashMap<Path, Rc<Box<dyn Any>>>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a map of Paths and Any")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut result = HashMap::new();
            while let Some((key, value)) = map.next_entry::<String, String>()? {
                let path: Path = serde_json::from_str(&key).unwrap();
                let value: Rc<Box<dyn Any>> = Rc::new(Box::new(value) as Box<dyn Any>);
                result.insert(path, value);
            }
            Ok(result)
        }
    }

    pub fn deserialize_rc_box_any_map<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<Path, Rc<Box<dyn Any>>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ExportMapVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path;
    use crate::path::Path;
    use crate::slot::Slot::{Nbr, Rep};

    #[test]
    fn test_new_empty() {
        let export: Export = Export::new();
        assert!(export.map.is_empty())
    }

    #[test]
    fn test_new() {
        /* showing how the macros saves us from writing this:
        let mut map: HashMap<Path, Rc<Box<dyn Any>>> = HashMap::new();
        map.insert(Path::from(vec![Rep(0), Nbr(0)]), Rc::new(Box::new(10)));
        let export = Export::from(map);*/
        let export = export!((path!(Rep(0), Nbr(0)), 10));
        assert_eq!(export.map.len(), 1);
    }

    #[test]
    fn test_put() {
        let mut export = export!((path!(Rep(0)), 10));
        export.put(path!(Rep(0), Nbr(0)), 20);
        export.put(Path::from(vec![Nbr(0)]), "foo");
        assert_eq!(export.paths().len(), 3);
    }

    #[test]
    fn test_get() {
        let export = export!((path!(Nbr(0), Rep(0)), 10));
        assert_eq!(
            //path is written in reverse order in the macro
            export
                .get::<i32>(&Path::from(vec![Rep(0), Nbr(0)]))
                .unwrap(),
            10
        );
    }

    #[test]
    fn test_get_none() {
        let export = export!((path!(Rep(0), Nbr(0)), 10));
        assert!(export
            .get::<String>(&Path::from(vec![Rep(0), Nbr(0)]))
            .is_err());
    }

    #[test]
    fn test_root() {
        let export = export!((Path::new(), 10));
        assert_eq!(export.root::<i32>(), 10);
    }

    #[test]
    #[should_panic]
    fn test_root_panic() {
        let export = export!((Path::new(), 10));
        assert_eq!(export.root::<String>(), "foo");
    }

    #[test]
    fn test_paths() {
        let export = export!((Path::new(), 10));
        let mut map2: HashMap<Path, Rc<Box<dyn Any>>> = HashMap::new();
        map2.insert(Path::new(), Rc::new(Box::new(10)));
        assert!(export.map.keys().eq(map2.keys()));
    }

    #[test]
    fn test_empty_state() {
        let export: Export = Export::new();
        let path = path!(Nbr(0), Rep(0));
        assert!(export.get::<i32>(&Path::new()).is_err());
        assert!(export.get::<i32>(&path).is_err());
    }

    #[test]
    fn test_root_path() {
        let mut export: Export = Export::new();
        export.put(Path::new(), String::from("foo"));
        assert_eq!(
            export.get::<String>(&Path::new()).unwrap(),
            export.root::<String>()
        );
        assert_eq!(
            export.get::<String>(&Path::new()).unwrap(),
            "foo".to_string()
        );
    }

    #[test]
    fn test_non_root_path() {
        let mut export: Export = Export::new();
        let path = path!(Nbr(0), Rep(0));
        export.put(path.clone(), String::from("bar"));
        assert_eq!(export.get::<String>(&path).unwrap(), String::from("bar"));
    }

    #[test]
    fn test_overwriting_with_different_type() {
        let mut export: Export = Export::new();
        export.put(Path::new(), String::from("foo"));
        assert_eq!(
            export.get::<String>(&Path::new()).unwrap(),
            "foo".to_string()
        );
        export.put(Path::new(), 77);
        assert_eq!(export.get::<i32>(&Path::new()).unwrap(), 77);
    }

    #[test]
    fn test_put_lazy() {
        let mut export: Export = Export::new();
        export.put_lazy(path!(Nbr(0)), || 10);
        assert_eq!(export.get::<i32>(&path!(Nbr(0))).unwrap(), 10);
    }

    #[test]
    fn test_put_lazy_and_return() {
        let mut export: Export = Export::new();
        let value = export.put_lazy_and_return(path!(Nbr(0)), || 10);
        assert_eq!(value, 10);
        assert_eq!(export.get::<i32>(&path!(Nbr(0))).unwrap(), 10);
    }

    #[test]
    fn test_partial_eq() {
        //assert the equality of two exports
        let export1 = export!((path!(Rep(0), Nbr(0)), 10));
        let export2 = export!((path!(Rep(0), Nbr(0)), 10));
        assert_eq!(export1, export2);

        //assert the inequality of two exports
        let export3 = export!((path!(Rep(0), Nbr(0)), 100));
        assert_ne!(export1, export3);
    }

    #[test]
    fn test_serialize_and_deserialize() {
        let export = export![
            (path!(Rep(0), Nbr(0)), 10),
            (path!(Nbr(0)), 10),
            (path!(Rep(0)), 10),
            (Path::new(), 10)
        ];
        let export_ser = serde_json::to_string(&export).unwrap();
        let export_des: Export = serde_json::from_str(&export_ser).unwrap();
        assert_eq!(export, export_des);
    }
}
