#[macro_use]
extern crate deep_clone_derive;

use std::borrow::Cow;

pub trait DeepClone {
    type DeepCloned: 'static;
    fn deep_clone(&self) -> Self::DeepCloned;
}

impl DeepClone for usize {
    type DeepCloned = usize;
    fn deep_clone(&self) -> usize { *self }
}

impl DeepClone for str {
    type DeepCloned = String;
    fn deep_clone(&self) -> String { self.to_owned() }
}

impl<T> DeepClone for [T]
    where T: DeepClone
{
    type DeepCloned = Vec<T::DeepCloned>;
    fn deep_clone(&self) -> Vec<T::DeepCloned> { self.iter().map(DeepClone::deep_clone).collect() }
}

impl<T> DeepClone for Vec<T>
    where T: DeepClone
{
    type DeepCloned = Vec<T::DeepCloned>;
    fn deep_clone(&self) -> Vec<T::DeepCloned> { self.iter().map(DeepClone::deep_clone).collect() }
}

impl<'a, T: ?Sized> DeepClone for Cow<'a, T>
    where T: 'static + ToOwned,
{
    type DeepCloned = Cow<'static, T>;
    fn deep_clone(&self) -> Cow<'static, T> { Cow::Owned((**self).to_owned()) }
}

#[cfg(test)]
#[derive(DeepClone, PartialEq, Eq, Debug)]
struct TestStruct<'a,T> {
    this: usize,
    that: Cow<'a,str>,
    other: Vec<T>,
}

#[cfg(test)]
#[derive(DeepClone, PartialEq, Eq, Debug)]
struct TestTuple<'a,T> (usize, Cow<'a,str>, Vec<T>);

#[cfg(test)]
#[derive(DeepClone, PartialEq, Eq, Debug)]
enum TestEnum<'a,T> {
    Structy { 
        this: usize,
        that: Cow<'a,str>,
        other: Vec<T>,
    },
    Tuply (usize, Cow<'a,str>, Vec<T>),
    Unity,
}

#[test]
fn test_derive() {
    let value = TestStruct {
        this: 37,
        that: "abc".into(),
        other: vec![
            TestTuple (
                37,
                "abc".into(),
                vec![
                    TestEnum::Structy {
                        this: 37,
                        that: "abc".into(),
                        other: vec![9]
                    },
                    TestEnum::Tuply(37, "abc".into(), vec![]),
                    TestEnum::Unity
                ]
            )
        ]
    };
    assert_eq!(value, value.deep_clone());
}
