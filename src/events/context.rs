use super::markers::{Mut, Ref};

pub trait Context<M = Mut> {
    type Borrow<'a>
    where
        Self: 'a;

    fn borrow<'b, 'a>(cx: &'b mut Self::Borrow<'a>) -> Self::Borrow<'b>
    where
        'a: 'b;
}

impl<C> Context for C {
    type Borrow<'a>
        = &'a mut C
    where
        Self: 'a;

    fn borrow<'b, 'a>(cx: &'b mut Self::Borrow<'a>) -> Self::Borrow<'b>
    where
        'a: 'b,
    {
        &mut **cx
    }
}
impl<C> Context<Ref> for C {
    type Borrow<'a>
        = &'a C
    where
        Self: 'a;

    fn borrow<'b, 'a>(cx: &'b mut Self::Borrow<'a>) -> Self::Borrow<'b>
    where
        'a: 'b,
    {
        &**cx
    }
}

macro_rules! ref_type {
    (Ref $generic:ident) => { &'a $generic };
    (Mut $generic:ident) => { &'a mut $generic };
}
macro_rules! borrow_type {
    (Ref $cx:ident $idx:tt) => {
        &*$cx.$idx
    };
    (Mut $cx:ident $idx:tt) => {
        &mut *$cx.$idx
    };
}

macro_rules! impl_context_tuple_inner {
    ([$($idx:tt), +][$($generic:ident),+][$($marker:ident),+]) => {
        impl<$($generic),+> Context<($($marker),+)> for ($($generic),+) {
            type Borrow<'a>
                = ($(ref_type!($marker $generic)), +)
            where
                Self: 'a;


            fn borrow<'b, 'a>(cx: &'b mut Self::Borrow<'a>) -> Self::Borrow<'b>
            where
                'a: 'b,
            {
                ($(borrow_type!($marker cx $idx)), +)
            }
        }
    }
}

macro_rules! impl_context_tuple {
    ([$A:ident, $B:ident]) => { impl_context_tuple_inner!([0, 1][A, B][$A, $B]); };
    ([$A:ident, $B:ident, $C:ident]) => { impl_context_tuple_inner!([0, 1, 2][A, B, C][$A, $B, $C]); };
}

impl_context_tuple!([Mut, Mut]);
impl_context_tuple!([Mut, Ref]);
impl_context_tuple!([Mut, Mut, Mut]);
impl_context_tuple!([Mut, Mut, Ref]);
impl_context_tuple!([Mut, Ref, Ref]);
