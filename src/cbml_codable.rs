// use crate::CbmlValue;

// /// ```
// ///
// /// use cbml::*;
// ///
// /// #[derive(CbmlCodable, Debug, PartialEq, Clone)]
// /// struct MyConfig {
// ///     name: String,
// ///     age: f64,
// ///     b: bool,
// /// }
// ///
// /// fn asdfsadf() {
// ///     let a = MyConfig {
// ///         name: "namasdfsadfsadfe".to_string(),
// ///         age: 99.0,
// ///         b: false,
// ///     };
// ///
// ///    let cbml_code: String = a.clone().to_cbml();
// ///    let b = MyConfig::from_cbml_value(&cbml_code);
// ///
// ///     assert_eq!(&a, &b);
// /// }
// /// ```
// pub trait CbmlCodable {
//     fn from_cbml_value(val: CbmlValue) -> Result<Self, ()>
//     where
//         Self: Sized;

//     fn to_cbml_value(self) -> CbmlValue;
// }

// pub trait CbmlCodable2 {
//     fn from_cbml(str: &str);
//     fn to_cbml() -> String;
// }

// #[derive(Debug, Clone, PartialEq)]
// pub struct AnyCbmlValue(pub CbmlValue);

// impl CbmlCodable for AnyCbmlValue {
//     fn from_cbml_value(val: CbmlValue) -> Result<Self, ()>
//     where
//         Self: Sized,
//     {
//         Ok(Self(val))
//     }

//     fn to_cbml_value(self) -> CbmlValue {
//         self.0
//     }
// }

// impl CbmlCodable for String {
//     fn from_cbml_value(val: CbmlValue) -> Result<Self, ()>
//     where
//         Self: Sized,
//     {
//         match val {
//             CbmlValue::String(s) => Ok(s),
//             _ => Err(()),
//         }
//     }

//     fn to_cbml_value(self) -> CbmlValue {
//         CbmlValue::String(self)
//     }
// }

// impl CbmlCodable for bool {
//     fn from_cbml_value(val: CbmlValue) -> Result<Self, ()> {
//         match val {
//             CbmlValue::Boolean(s) => Ok(s),
//             // CbmlValue::None => Ok(None),
//             _ => Err(()),
//         }
//     }

//     fn to_cbml_value(self) -> CbmlValue {
//         CbmlValue::Boolean(self)
//     }
// }

// impl CbmlCodable for f64 {
//     fn from_cbml_value(val: CbmlValue) -> Result<Self, ()> {
//         match val {
//             CbmlValue::Number(s) => Ok(s),

//             _ => Err(()),
//         }
//     }

//     fn to_cbml_value(self) -> CbmlValue {
//         CbmlValue::Number(self)
//     }
// }

// impl<T: CbmlCodable + Sized> CbmlCodable for Option<T> {
//     fn from_cbml_value(val: CbmlValue) -> Result<Self, ()>
//     where
//         Self: Sized,
//     {
//         if val.cbml_none().is_some() {
//             return Ok(None);
//         }

//         let re = T::from_cbml_value(val);
//         match re {
//             Ok(v) => Ok(Some(v)),
//             Err(_) => Err(()),
//         }
//     }

//     fn to_cbml_value(self) -> CbmlValue {
//         match self {
//             Some(v) => v.to_cbml_value(),
//             None => CbmlValue::None,
//         }
//     }
// }

// impl<T: CbmlCodable + Sized> CbmlCodable for Vec<T> {
//     fn from_cbml_value(val: CbmlValue) -> Result<Self, ()>
//     where
//         Self: Sized,
//     {
//         let re = val.cbml_array();
//         match re {
//             Some(v) => {
//                 let mut re: Vec<T> = Vec::new();
//                 for x in v {
//                     let Ok(inner_val) = T::from_cbml_value(x.clone()) else {
//                         return Err(());
//                     };

//                     re.push(inner_val);
//                 }

//                 return Ok(re);
//             }
//             None => Err(()),
//         }
//     }

//     fn to_cbml_value(self) -> CbmlValue {
//         let mut re: Vec<CbmlValue> = Vec::new();

//         for x in self {
//             re.push(x.to_cbml_value());
//         }

//         return CbmlValue::Array(re);
//     }
// }

// impl<T: CbmlCodable + Sized> CbmlCodable for Box<T> {
//     fn from_cbml_value(val: CbmlValue) -> Result<Self, ()>
//     where
//         Self: Sized,
//     {
//         let a = T::from_cbml_value(val)?;
//         return Ok(Box::new(a));
//     }

//     fn to_cbml_value(self) -> CbmlValue {
//         (*self).to_cbml_value()
//     }
// }
