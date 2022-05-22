pub trait Id: Copy + Clone + PartialOrd + PartialEq {
    fn new(id: u32) -> Self;
    fn unwrap(&self) -> u32;
}

macro_rules! implement_id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
        pub struct $name(u32);
        impl Id for $name {
            fn new(id: u32) -> Self {
                Self(id)
            }

            fn unwrap(&self) -> u32 {
                self.0
            }
        }
    };
}

implement_id_type!(NodeId);
implement_id_type!(InputId);
implement_id_type!(OutputId);

#[derive(Debug, Clone, PartialEq)]
pub struct IdGenerator<T: Id> {
    last_id: T,
}

impl<T: Id> IdGenerator<T> {
    pub fn new() -> Self {
        Self { last_id: T::new(0) }
    }
    pub fn generate(&mut self) -> T {
        let new_id = self.last_id;
        // TODO(philpax): replace with an actual id generator
        self.last_id = T::new(new_id.unwrap() + 1);
        new_id
    }
}

impl<T: Id> Default for IdGenerator<T> {
    fn default() -> Self {
        Self::new()
    }
}
