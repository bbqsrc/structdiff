use std::fmt::Debug;

pub trait Diff: Debug
where
    Self: Sized,
{
    type Changeset: Debug + Apply<Self>;
    type Action: Debug + Apply<Self>;

    fn changeset(&self, other: &Self) -> Field<Self, Self::Changeset, Self::Action>;
}

pub trait Apply<T: Sized>: Debug {
    fn apply(self, target: &mut T);
}

impl<T> Apply<T> for () {
    #[inline(always)]
    fn apply(self, _target: &mut T) {}
}

#[derive(Debug)]
pub enum Field<V, K, A>
where
    K: Apply<V>,
    A: Apply<V>,
{
    None,
    Set(V),
    Changes(K),
    Actions(Vec<A>),
}

impl<V, K, A> Apply<V> for Field<V, K, A>
where
    V: Debug,
    K: Debug + Apply<V>,
    A: Apply<V>,
{
    fn apply(self, target: &mut V) {
        match self {
            Field::None => {}
            Field::Set(value) => {
                *target = value;
            }
            Field::Changes(changeset) => changeset.apply(target),
            Field::Actions(actions) => {
                actions.into_iter().for_each(|x| x.apply(target));
            }
        }
    }
}

impl<V, K: Apply<V>, A: Apply<V>> std::default::Default for Field<V, K, A> {
    fn default() -> Self {
        Field::None
    }
}

macro_rules! impl_scalar {
    ($ty:ty) => {
        impl $crate::Diff for $ty {
            type Changeset = ();
            type Action = ();

            fn changeset(
                &self,
                other: &Self,
            ) -> $crate::Field<Self, Self::Changeset, Self::Action> {
                if self != other {
                    $crate::Field::Set(*other)
                } else {
                    $crate::Field::None
                }
            }
        }
    };
}

macro_rules! impl_scalar_ref {
    ($ty:ty) => {
        impl $crate::Diff for $ty {
            type Changeset = ();
            type Action = ();

            fn changeset(
                &self,
                other: &Self,
            ) -> $crate::Field<Self, Self::Changeset, Self::Action> {
                if self != other {
                    $crate::Field::Set(other.to_owned())
                } else {
                    $crate::Field::None
                }
            }
        }
    };
}

use types::*;

pub mod types {
    use super::{Apply, Diff, Field};

    impl_scalar!(i8);
    pub type I8Changeset = ();

    impl_scalar!(u8);
    pub type U8Changeset = ();

    impl_scalar!(i16);
    pub type I16Changeset = ();

    impl_scalar!(u16);
    pub type U16Changeset = ();

    impl_scalar!(i32);
    pub type I32Changeset = ();

    impl_scalar!(u32);
    pub type U32Changeset = ();

    impl_scalar!(i64);
    pub type I64Changeset = ();

    impl_scalar!(u64);
    pub type U64Changeset = ();

    impl_scalar!(i128);
    pub type I128Changeset = ();

    impl_scalar!(u128);
    pub type U128Changeset = ();

    impl_scalar!(isize);
    pub type IsizeChangeset = ();

    impl_scalar!(usize);
    pub type UsizeChangeset = ();

    impl_scalar!(f32);
    pub type F32Changeset = ();

    impl_scalar!(f64);
    pub type F64Changeset = ();

    impl_scalar!(bool);
    pub type BoolChangeset = ();

    impl_scalar!(());

    impl_scalar_ref!(String);
    pub type StringChangeset = ();

    #[derive(Debug)]
    pub enum VecAction<T: Diff> {
        Set(usize, Field<T, <T as Diff>::Changeset, <T as Diff>::Action>),
        Push(T),
        Truncate(usize),
        Append(Vec<T>),
    }

    impl<T: Diff> Apply<Vec<T>> for VecAction<T> {
        fn apply(self, target: &mut Vec<T>) {
            use VecAction::*;

            match self {
                Set(index, field) => {
                    field.apply(&mut target[index]);
                }
                Push(value) => target.push(value),
                Truncate(len) => target.truncate(len),
                Append(mut items) => target.append(&mut items),
            }
        }
    }

    #[derive(Debug)]
    pub struct VecChangeset<T: Diff>(Field<T, <T as Diff>::Changeset, <T as Diff>::Action>);

    impl<T: Diff> Apply<Vec<T>> for VecChangeset<T> {
        fn apply(self, target: &mut Vec<T>) {}
    }

    #[derive(Debug)]
    pub enum OptionChangeset<T: Diff> {
        NoneChangeset(Field<(), (), ()>),
        SomeChangeset(Field<T, <T as Diff>::Changeset, <T as Diff>::Action>),
    }

    impl<T: Diff> Apply<Option<T>> for OptionChangeset<T> {
        fn apply(self, target: &mut Option<T>) {
            use OptionChangeset::*;

            match self {
                NoneChangeset(_) => {
                    *target = None;
                }
                SomeChangeset(value) => match target.iter_mut().next() {
                    Some(v) => {
                        value.apply(v);
                    }
                    None => {
                        unreachable!("This is a logic error.");
                    }
                },
            }
        }
    }

    impl<T: Diff + PartialEq + Clone> Diff for Option<T> {
        type Changeset = OptionChangeset<T>;
        type Action = ();

        fn changeset(&self, other: &Self) -> Field<Self, Self::Changeset, Self::Action>
        where
            Self: Sized,
        {
            if self == other {
                return Field::None;
            }

            let changes = match (self, other) {
                (None, None) => OptionChangeset::NoneChangeset(Default::default()),
                (Some(a), Some(b)) => OptionChangeset::SomeChangeset(a.changeset(&b)),
                (_, v) => return Field::Set(v.to_owned()),
            };

            Field::Changes(changes)
        }
    }

    #[derive(Debug)]
    pub enum ResultChangeset<T: Diff, E: Diff> {
        OkChangeset(Field<T, <T as Diff>::Changeset, <T as Diff>::Action>),
        ErrChangeset(Field<E, <E as Diff>::Changeset, <E as Diff>::Action>),
    }

    impl<T: Diff, E: Diff> Apply<Result<T, E>> for ResultChangeset<T, E> {
        fn apply(self, target: &mut Result<T, E>) {
            use ResultChangeset::*;

            match self {
                OkChangeset(x) => match target.as_mut() {
                    Ok(inner) => x.apply(inner),
                    _ => unreachable!("Logic error"),
                },
                ErrChangeset(x) => match target.as_mut() {
                    Err(inner) => x.apply(inner),
                    _ => unreachable!("Logic error"),
                },
            }
        }
    }

    impl<T, E> Diff for Result<T, E>
    where
        T: Diff + PartialEq + Clone,
        E: Diff + PartialEq + Clone,
    {
        type Changeset = ResultChangeset<T, E>;
        type Action = ();

        fn changeset(&self, other: &Self) -> Field<Self, Self::Changeset, Self::Action>
        where
            Self: Sized,
        {
            if self == other {
                return Field::None;
            }

            let changes = match (self, other) {
                (Ok(a), Ok(b)) => ResultChangeset::OkChangeset(a.changeset(&b)),
                (Err(a), Err(b)) => ResultChangeset::ErrChangeset(a.changeset(&b)),
                (_, v) => return Field::Set(v.to_owned()),
            };

            Field::Changes(changes)
        }
    }
}

impl<T> Diff for Vec<T>
where
    T: Clone + PartialEq + Diff,
{
    type Changeset = VecChangeset<T>;
    type Action = VecAction<T>;

    fn changeset(&self, other: &Self) -> Field<Self, Self::Changeset, Self::Action> {
        if self == other {
            return Field::None;
        }

        let mut changes: Vec<Self::Action> = vec![];

        let min = std::cmp::min(self.len(), other.len());

        for i in 0..min {
            let changeset = self[i].changeset(&other[i]);
            match changeset {
                Field::None => {}
                changeset => changes.push(VecAction::Set(i, changeset)),
            }
        }

        if self.len() > other.len() {
            changes.push(VecAction::Truncate(other.len()));
        } else if self.len() < other.len() {
            changes.push(VecAction::Append(other[min..].to_vec()))
        }

        Field::Actions(changes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type SetField<A, B> = Field<A, B, ()>;

    #[derive(Debug, PartialEq, Clone)]
    enum SomeEnum {
        None,
        Field1(String),
        Field2(u32),
        Field3(Bar),
        Field4(u16, u16),
    }

    impl std::default::Default for SomeEnum {
        fn default() -> Self {
            SomeEnum::None
        }
    }

    #[derive(Debug)]
    enum SomeEnumChangeset {
        None(SetField<(), ()>),
        Field1(SetField<String, StringChangeset>),
        Field2(SetField<u32, U32Changeset>),
        Field3(SetField<Bar, BarChangeset>),
        Field4(SetField<u16, U16Changeset>, SetField<u16, U16Changeset>),
    }

    impl Apply<SomeEnum> for SomeEnumChangeset {
        fn apply(self, target: &mut SomeEnum) {
            use SomeEnumChangeset::*;

            match self {
                None(_) => {
                    *target = SomeEnum::None;
                }
                Field1(field) => match target {
                    SomeEnum::Field1(target) => field.apply(target),
                    _ => unreachable!("logic error"),
                },
                Field2(field) => match target {
                    SomeEnum::Field2(target) => field.apply(target),
                    _ => unreachable!("logic error"),
                },
                Field3(field) => match target {
                    SomeEnum::Field3(target) => field.apply(target),
                    _ => unreachable!("logic error"),
                },
                Field4(a, b) => match target {
                    SomeEnum::Field4(t1, t2) => {
                        a.apply(t1);
                        b.apply(t2);
                    }
                    _ => unreachable!("logic error"),
                },
            }
        }
    }

    #[derive(Debug, Default, PartialEq)]
    struct Foo {
        field_a: u32,
        field_b: String,
        enumer: SomeEnum,
        bar1: Bar,
        bar: Option<Bar>,
        vec: Vec<String>,
    }

    impl Diff for SomeEnum {
        type Changeset = SomeEnumChangeset;
        type Action = ();

        fn changeset(&self, other: &Self) -> Field<Self, Self::Changeset, Self::Action>
        where
            Self: Sized,
        {
            if self == other {
                return Field::None;
            }

            use SomeEnum::*;

            let changes = match (self, other) {
                (None, None) => SomeEnumChangeset::None(().changeset(&())),
                (Field1(a), Field1(b)) => SomeEnumChangeset::Field1(a.changeset(&b)),
                (Field2(a), Field2(b)) => SomeEnumChangeset::Field2(a.changeset(&b)),
                (Field3(a), Field3(b)) => SomeEnumChangeset::Field3(a.changeset(&b)),
                (Field4(a1, a2), Field4(b1, b2)) => {
                    SomeEnumChangeset::Field4(a1.changeset(&b1), a2.changeset(&b2))
                }
                (_, v) => return Field::Set(v.to_owned()),
            };

            Field::Changes(changes)
        }
    }
    impl Diff for Foo {
        type Changeset = FooChangeset;
        type Action = ();

        fn changeset(&self, other: &Self) -> Field<Self, Self::Changeset, Self::Action>
        where
            Self: Sized,
        {
            if self == other {
                return Field::None;
            }

            let mut changes = Self::Changeset::default();

            changes.field_a = self.field_a.changeset(&other.field_a);
            changes.field_b = self.field_b.changeset(&other.field_b);
            changes.enumer = self.enumer.changeset(&other.enumer);
            changes.bar = self.bar.changeset(&other.bar);
            changes.bar1 = self.bar1.changeset(&other.bar1);
            changes.vec = self.vec.changeset(&other.vec);

            Field::Changes(changes)
        }
    }

    impl Diff for Bar {
        type Changeset = BarChangeset;
        type Action = ();

        fn changeset(&self, other: &Self) -> Field<Self, Self::Changeset, Self::Action>
        where
            Self: Sized,
        {
            if self == other {
                return Field::None;
            }

            let mut changes = Self::Changeset::default();

            if self.field_d != other.field_d {
                changes.field_d = Field::Set(other.field_d.to_owned())
            }

            Field::Changes(changes)
        }
    }

    #[derive(Debug, Clone, Default, PartialEq)]
    struct Bar {
        field_d: String,
    }

    #[derive(Debug, Default)]
    struct FooChangeset {
        field_a: SetField<u32, U32Changeset>,
        field_b: SetField<String, StringChangeset>,
        enumer: SetField<SomeEnum, SomeEnumChangeset>,
        bar1: SetField<Bar, BarChangeset>,
        bar: SetField<Option<Bar>, OptionChangeset<Bar>>,
        vec: Field<Vec<String>, VecChangeset<String>, VecAction<String>>,
    }

    impl Apply<Foo> for FooChangeset {
        fn apply(self, target: &mut Foo) {
            self.field_a.apply(&mut target.field_a);
            self.field_b.apply(&mut target.field_b);
            self.enumer.apply(&mut target.enumer);
            self.bar1.apply(&mut target.bar1);
            self.bar.apply(&mut target.bar);
            self.vec.apply(&mut target.vec);
        }
    }

    #[derive(Debug, Default)]
    struct BarChangeset {
        field_d: SetField<String, StringChangeset>,
    }

    impl Apply<Bar> for BarChangeset {
        fn apply(self, target: &mut Bar) {
            self.field_d.apply(&mut target.field_d);
        }
    }

    #[test]
    fn basic() {
        println!("{:?}", Foo::default().changeset(&Foo::default()));

        let mut g = Foo::default();
        g.enumer = SomeEnum::Field4(22, 44);
        g.bar = Some(Bar {
            field_d: "Mongol".into(),
        });
        g.vec = vec!["A".into(), "X".into(), "C".into()];

        let mut f = Foo::default();
        f.field_a = 123;
        f.field_b = "ahahah".into();
        f.enumer = SomeEnum::None;
        f.bar = Some(Bar {
            field_d: "Hello".into(),
        });
        f.vec = vec!["A".into(), "B".into(), "C".into(), "D".into(), "D".into()];

        // f.bar = Some(Bar { field_d: "AOWIJEWIOAJE".into() });
        let changeset = g.changeset(&f);
        println!("{:#?}", &changeset);

        changeset.apply(&mut g);
        println!("{:#?}", &g);

        assert_eq!(&f, &g);
    }
}
