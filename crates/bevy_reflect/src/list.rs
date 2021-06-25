use std::any::Any;

use crate::{serde::Serializable, Array, ArrayIter, DynamicArray, Reflect, ReflectMut, ReflectRef};

/// An ordered, mutable list of [Reflect] items. This corresponds to types like [std::vec::Vec].
/// This is a sub-trait of [`Array`] as it implements a `push` function, allowing it's internal
/// size to grow.
pub trait List: Array {
    fn push(&mut self, value: Box<dyn Reflect>);
    fn clone_dynamic_list(&self) -> DynamicList {
        DynamicList {
            name: self.type_name().to_string(),
            values: self.iter().map(|value| value.clone_value()).collect(),
        }
    }
}

#[derive(Default)]
pub struct DynamicList {
    name: String,
    values: Vec<Box<dyn Reflect>>,
}

impl DynamicList {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn push<T: Reflect>(&mut self, value: T) {
        self.values.push(Box::new(value));
    }

    pub fn push_box(&mut self, value: Box<dyn Reflect>) {
        self.values.push(value);
    }
}

impl Array for DynamicList {
    fn get(&self, index: usize) -> Option<&dyn Reflect> {
        self.values.get(index).map(|value| &**value)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.values.get_mut(index).map(|value| &mut **value)
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn iter(&self) -> ArrayIter {
        ArrayIter {
            array: self,
            index: 0,
        }
    }

    fn clone_dynamic_array(&self) -> DynamicArray {
        DynamicArray {
            name: self.name.clone(),
            values: self
                .values
                .iter()
                .map(|value| value.clone_value())
                .collect(),
        }
    }
}

impl List for DynamicList {
    fn push(&mut self, value: Box<dyn Reflect>) {
        DynamicList::push_box(self, value);
    }

    fn clone_dynamic_list(&self) -> DynamicList {
        DynamicList {
            name: self.name.clone(),
            values: self
                .values
                .iter()
                .map(|value| value.clone_value())
                .collect(),
        }
    }
}

// SAFE: any and any_mut both return self
unsafe impl Reflect for DynamicList {
    #[inline]
    fn type_name(&self) -> &str {
        self.name.as_str()
    }

    #[inline]
    fn any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn apply(&mut self, value: &dyn Reflect) {
        list_apply(self, value);
    }

    #[inline]
    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        *self = value.take()?;
        Ok(())
    }

    #[inline]
    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::List(self)
    }

    #[inline]
    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::List(self)
    }

    #[inline]
    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone_dynamic_list())
    }

    #[inline]
    fn reflect_hash(&self) -> Option<u64> {
        crate::array_hash(self)
    }

    fn reflect_partial_eq(&self, value: &dyn Reflect) -> Option<bool> {
        list_partial_eq(self, value)
    }

    fn serializable(&self) -> Option<Serializable> {
        Some(Serializable::Borrowed(self))
    }
}

impl serde::Serialize for DynamicList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::array_serialize(self, serializer)
    }
}

#[inline]
pub fn list_apply<L: List>(a: &mut L, b: &dyn Reflect) {
    if let ReflectRef::List(list_value) = b.reflect_ref() {
        for (i, value) in list_value.iter().enumerate() {
            if i < a.len() {
                if let Some(v) = a.get_mut(i) {
                    v.apply(value);
                }
            } else {
                List::push(a, value.clone_value());
            }
        }
    } else {
        panic!("Attempted to apply a non-list type to a list type.");
    }
}

#[inline]
pub fn list_partial_eq<L: List>(a: &L, b: &dyn Reflect) -> Option<bool> {
    let list = if let ReflectRef::List(list) = b.reflect_ref() {
        list
    } else {
        return Some(false);
    };

    if a.len() != list.len() {
        return Some(false);
    }

    for (a_value, b_value) in a.iter().zip(list.iter()) {
        if let Some(false) | None = a_value.reflect_partial_eq(b_value) {
            return Some(false);
        }
    }

    Some(true)
}
