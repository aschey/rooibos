use std::fmt::{self, Debug};
use std::sync::Arc;

use tachys::view::any_view::{AnyView, IntoAny};
use tachys::view::fragment::{Fragment, IntoFragment};

use crate::dom::{IntoView, RenderAny, RooibosDom, View};

/// The most common type for the `children` property on components,
/// which can only be called once.
///
/// This does not support iterating over individual nodes within the children.
/// To iterate over children, use [`ChildrenFragment`].
pub type Children = Box<dyn FnOnce() -> AnyView<RooibosDom> + Send>;

/// A type for the `children` property on components that can be called only once,
/// and provides a collection of all the children passed to this component.
pub type ChildrenFragment = Box<dyn FnOnce() -> Fragment<RooibosDom> + Send>;

/// A type for the `children` property on components that can be called
/// more than once.
pub type ChildrenFn = Arc<dyn Fn() -> AnyView<RooibosDom> + Send + Sync>;

/// A type for the `children` property on components that can be called more than once,
/// and provides a collection of all the children passed to this component.
pub type ChildrenFragmentFn = Arc<dyn Fn() -> Fragment<RooibosDom> + Send>;

/// A type for the `children` property on components that can be called
/// more than once, but may mutate the children.
pub type ChildrenFnMut = Box<dyn FnMut() -> AnyView<RooibosDom> + Send>;

/// A type for the `children` property on components that can be called more than once,
/// but may mutate the children, and provides a collection of all the children
/// passed to this component.
pub type ChildrenFragmentMut = Box<dyn FnMut() -> Fragment<RooibosDom> + Send>;

// This is to still support components that accept `Box<dyn Fn() -> AnyView>` as a children.
type BoxedChildrenFn = Box<dyn Fn() -> AnyView<RooibosDom> + Send>;

pub trait ToChildren<F> {
    fn to_children(f: F) -> Self;
}

pub trait IntoChildrenFnMut {
    fn into_children_fn_mut(self) -> ChildrenFnMut;
}

pub trait IntoChildrenFn {
    fn into_children_fn(self) -> ChildrenFn;
}

pub trait IntoChildren {
    fn into_children(self) -> Children;
}

impl<F, C> IntoChildren for F
where
    F: FnOnce() -> C + Send + 'static,
    C: IntoAny<RooibosDom> + Send + 'static,
{
    fn into_children(self) -> Children {
        Box::new(move || self().into_any())
    }
}

impl<F, C> ToChildren<F> for Children
where
    F: FnOnce() -> C + Send + 'static,
    C: IntoAny<RooibosDom> + Send + 'static,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Box::new(move || f().into_any())
    }
}

impl<F, C> ToChildren<F> for ChildrenFn
where
    F: Fn() -> C + Send + Sync + 'static,
    C: IntoAny<RooibosDom> + Send + 'static,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Arc::new(move || f().into_any())
    }
}

impl<F, C> IntoChildrenFnMut for F
where
    F: FnMut() -> C + Send + Sync + 'static,
    C: IntoAny<RooibosDom>,
{
    fn into_children_fn_mut(mut self) -> ChildrenFnMut {
        Box::new(move || self().into_any())
    }
}

impl<F, C> IntoChildrenFn for F
where
    F: Fn() -> C + Send + Sync + 'static,
    C: IntoAny<RooibosDom>,
{
    fn into_children_fn(self) -> ChildrenFn {
        Arc::new(move || self().into_any())
    }
}

impl<F, C> ToChildren<F> for ChildrenFnMut
where
    F: Fn() -> C + Send + 'static,
    C: IntoAny<RooibosDom> + Send + 'static,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Box::new(move || f().into_any())
    }
}

impl<F, C> ToChildren<F> for BoxedChildrenFn
where
    F: Fn() -> C + Send + 'static,
    C: IntoAny<RooibosDom> + Send + 'static,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Box::new(move || f().into_any())
    }
}

impl<F, C> ToChildren<F> for ChildrenFragment
where
    F: FnOnce() -> C + Send + 'static,
    C: IntoFragment<RooibosDom>,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Box::new(move || f().into_fragment())
    }
}

impl<F, C> ToChildren<F> for ChildrenFragmentFn
where
    F: Fn() -> C + Send + 'static,
    C: IntoFragment<RooibosDom>,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Arc::new(move || f().into_fragment())
    }
}

impl<F, C> ToChildren<F> for ChildrenFragmentMut
where
    F: FnMut() -> C + Send + 'static,
    C: IntoFragment<RooibosDom>,
{
    #[inline]
    fn to_children(mut f: F) -> Self {
        Box::new(move || f().into_fragment())
    }
}

/// New-type wrapper for a function that returns a view with `From` and `Default` traits implemented
/// to enable optional props in for example `<Show>` and `<Suspense>`.
#[derive(Clone)]
pub struct ViewFn(Arc<dyn Fn() -> AnyView<RooibosDom> + Send + Sync + 'static>);

impl Default for ViewFn {
    fn default() -> Self {
        Self(Arc::new(|| ().into_any()))
    }
}

impl<F, C> From<F> for ViewFn
where
    F: Fn() -> C + Send + Sync + 'static,
    C: IntoAny<RooibosDom> + Send + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(move || value().into_any()))
    }
}

impl ViewFn {
    /// Execute the wrapped function
    pub fn run(&self) -> AnyView<RooibosDom> {
        (self.0)()
    }
}

/// New-type wrapper for a function, which will only be called once and returns a view with `From`
/// and `Default` traits implemented to enable optional props in for example `<Show>` and
/// `<Suspense>`.
pub struct ViewFnOnce(Box<dyn FnOnce() -> AnyView<RooibosDom> + Send + 'static>);

impl Default for ViewFnOnce {
    fn default() -> Self {
        Self(Box::new(|| ().into_any()))
    }
}

impl<F, C> From<F> for ViewFnOnce
where
    F: FnOnce() -> C + Send + 'static,
    C: IntoAny<RooibosDom> + 'static,
{
    fn from(value: F) -> Self {
        Self(Box::new(move || value().into_any()))
    }
}

impl ViewFnOnce {
    /// Execute the wrapped function
    pub fn run(self) -> AnyView<RooibosDom> {
        (self.0)()
    }
}

/// A typed equivalent to [`Children`], which takes a generic but preserves type information to
/// allow the compiler to optimize the view more effectively.
pub struct TypedChildren<T>(Box<dyn FnOnce() -> View<T> + Send>);

impl<T> TypedChildren<T> {
    pub fn into_inner(self) -> impl FnOnce() -> View<T> + Send {
        self.0
    }
}

impl<F, T> From<F> for TypedChildren<T>
where
    F: FnOnce() -> T + Send + Sync + 'static,
    T: RenderAny,
{
    fn from(value: F) -> Self {
        Self(Box::new(move || value().into_view()))
    }
}

impl<F, C> ToChildren<F> for TypedChildren<C>
where
    F: FnOnce() -> C + Send + 'static,
    C: IntoView,
{
    #[inline]
    fn to_children(f: F) -> Self {
        TypedChildren(Box::new(move || f().into_view()))
    }
}

/// A typed equivalent to [`ChildrenMut`], which takes a generic but preserves type information to
/// allow the compiler to optimize the view more effectively.
pub struct TypedChildrenMut<T>(Box<dyn FnMut() -> View<T> + Send>);

impl<F, T> From<F> for TypedChildrenMut<T>
where
    F: FnMut() -> T + Send + Sync + 'static,
    T: RenderAny,
{
    fn from(mut value: F) -> Self {
        Self(Box::new(move || value().into_view()))
    }
}

impl<T> Debug for TypedChildrenMut<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TypedChildrenMut").finish()
    }
}

impl<T> TypedChildrenMut<T> {
    pub fn into_inner(self) -> impl FnMut() -> View<T> + Send {
        self.0
    }
}

impl<F, C> ToChildren<F> for TypedChildrenMut<C>
where
    F: FnMut() -> C + Send + 'static,
    C: IntoView,
{
    #[inline]
    fn to_children(mut f: F) -> Self {
        TypedChildrenMut(Box::new(move || f().into_view()))
    }
}

/// A typed equivalent to [`ChildrenFn`], which takes a generic but preserves type information to
/// allow the compiler to optimize the view more effectively.
pub struct TypedChildrenFn<T>(Arc<dyn Fn() -> View<T> + Send + Sync>);

impl<T> Debug for TypedChildrenFn<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TypedChildrenFn").finish()
    }
}

impl<T> TypedChildrenFn<T> {
    pub fn into_inner(self) -> Arc<dyn Fn() -> View<T> + Send + Sync> {
        self.0
    }
}

impl<F, C> ToChildren<F> for TypedChildrenFn<C>
where
    F: Fn() -> C + Send + Sync + 'static,
    C: IntoView,
{
    #[inline]
    fn to_children(f: F) -> Self {
        TypedChildrenFn(Arc::new(move || f().into_view()))
    }
}
