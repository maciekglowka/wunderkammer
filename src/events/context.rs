use std::marker::PhantomData;

pub trait Context {
    type Borrow<'a>
    where
        Self: 'a;

    fn borrow<'b, 'a>(cx: &'b mut Self::Borrow<'a>) -> Self::Borrow<'b>
    where
        'a: 'b;
}

pub struct Ref<T>(PhantomData<fn(&T)>);
impl<T> Context for Ref<T> {
    type Borrow<'a>
        = &'a T
    where
        Self: 'a;

    fn borrow<'b, 'a>(cx: &'b mut Self::Borrow<'a>) -> Self::Borrow<'b>
    where
        'a: 'b,
    {
        cx
    }
}

pub struct Mut<T>(PhantomData<fn(&mut T)>);
impl<T> Context for Mut<T> {
    type Borrow<'a>
        = &'a mut T
    where
        Self: 'a;

    fn borrow<'b, 'a>(cx: &'b mut Self::Borrow<'a>) -> Self::Borrow<'b>
    where
        'a: 'b,
    {
        cx
    }
}

macro_rules! impl_single_tuple {
    ($($T:ident),+) => {
        impl<$($T),+> Context for ($($T),+)
        where
            $($T: Context),+

        {
            type Borrow<'a> = ($($T::Borrow<'a>),+)
            where
                Self: 'a;

            #[allow(non_snake_case)]
            fn borrow<'b, 'a>(cx: &'b mut Self::Borrow<'a>) -> Self::Borrow<'b>
            where
                'a: 'b,
            {
                let ($($T,)+) = cx;
                ($($T::borrow($T),)+)
            }
        }

    }
}

macro_rules! impl_tuples {
    ($a:ident, $b:ident) => {
        impl_single_tuple!($a, $b);
    };
    ($a:ident, $($rest:ident),+) => {
        impl_single_tuple!($a, $($rest), +);
        impl_tuples!($($rest),+);
    };
}

impl_tuples!(A, B, C, D, E, F);
