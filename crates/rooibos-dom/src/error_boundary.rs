use std::sync::Arc;

use reactive_graph::computed::ArcMemo;
use reactive_graph::signal::ArcRwSignal;
use reactive_graph::traits::{Get, Update, With};
use rustc_hash::FxHashMap;
use tachys::renderer::Renderer;
use tachys::view::{Mountable, Render};
use throw_error::{Error, ErrorHook, ErrorId};

use crate::{DomNode, IntoView, RenderAny, RooibosDom};

pub fn error_boundary<FalFn, Fal, F, R>(children: F, fallback: FalFn) -> impl IntoView
where
    F: Fn() -> R + Send + 'static,
    R: RenderAny,
    R::State: 'static,
    FalFn: FnMut(ArcRwSignal<Errors>) -> Fal + Clone + Send + 'static,
    Fal: RenderAny + 'static,
{
    let hook = Arc::new(ErrorBoundaryErrorHook::new([]));
    let errors = hook.errors.clone();
    let errors_empty = ArcMemo::new({
        let errors = errors.clone();
        move |_| errors.with(|map| map.is_empty())
    });
    let hook = hook as Arc<dyn ErrorHook>;

    // provide the error hook and render children
    throw_error::set_error_hook(Arc::clone(&hook));
    let mut children = Some(children);

    move || ErrorBoundaryView {
        errors_empty: errors_empty.get(),
        children: children.take(),
        fallback: Some((fallback.clone())(errors.clone())),
    }
}

#[derive(Debug)]
struct ErrorBoundaryView<Chil, Fal> {
    errors_empty: bool,
    children: Option<Chil>,
    fallback: Fal,
}

struct ErrorBoundaryViewState<Chil, Fal>
where
    Chil: RenderAny,
    Fal: RenderAny,
{
    showing_fallback: bool,
    // both the children and the fallback are always present, and we toggle between the two of them
    // as needed
    children: Chil::State,
    fallback: Fal::State,
    placeholder: DomNode,
}

impl<Chil, Fal> Mountable<RooibosDom> for ErrorBoundaryViewState<Chil, Fal>
where
    Chil: RenderAny,
    Fal: RenderAny,
{
    fn unmount(&mut self) {
        if self.showing_fallback {
            self.fallback.unmount();
        } else {
            self.children.unmount();
        }
        self.placeholder.unmount();
    }

    fn mount(&mut self, parent: &DomNode, marker: Option<&DomNode>) {
        if self.showing_fallback {
            self.fallback.mount(parent, marker);
        } else {
            self.children.mount(parent, marker);
        }
        self.placeholder.mount(parent, marker);
    }

    fn insert_before_this(&self, parent: &DomNode, child: &mut dyn Mountable<RooibosDom>) -> bool {
        if self.showing_fallback {
            self.fallback.insert_before_this(parent, child)
        } else {
            self.children.insert_before_this(parent, child)
        }
    }
}

impl<Chil, Fal> Render<RooibosDom> for ErrorBoundaryView<Chil, Fal>
where
    Chil: RenderAny,
    Fal: RenderAny,
{
    type State = ErrorBoundaryViewState<Chil, Fal>;

    fn build(self) -> Self::State {
        let placeholder = RooibosDom::create_placeholder();
        let children = (self
            .children
            .expect("tried to build ErrorBoundary but children were not present"))
        .build();
        let fallback = self.fallback.build();
        ErrorBoundaryViewState {
            showing_fallback: !self.errors_empty,
            children,
            fallback,
            placeholder,
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        match (self.errors_empty, state.showing_fallback) {
            // no errors, and was showing fallback
            (true, true) => {
                state.fallback.unmount();
                RooibosDom::try_mount_before(&mut state.children, state.placeholder.as_ref());
            }
            // yes errors, and was showing children
            (false, false) => {
                state.children.unmount();
                RooibosDom::try_mount_before(&mut state.fallback, state.placeholder.as_ref());
            }
            // either there were no errors, and we were already showing the children
            // or there are errors, but we were already showing the fallback
            // in either case, rebuilding doesn't require us to do anything
            _ => {}
        }
        state.showing_fallback = !self.errors_empty;
    }
}

#[derive(Debug)]
struct ErrorBoundaryErrorHook {
    errors: ArcRwSignal<Errors>,
}

impl ErrorBoundaryErrorHook {
    pub fn new(initial_errors: impl IntoIterator<Item = (ErrorId, Error)>) -> Self {
        Self {
            errors: ArcRwSignal::new(Errors(initial_errors.into_iter().collect())),
        }
    }
}

impl ErrorHook for ErrorBoundaryErrorHook {
    fn throw(&self, error: Error) -> ErrorId {
        // generate a unique ID
        let key = ErrorId::default(); // TODO unique ID...

        // add it to the reactive map of errors
        self.errors.update(|map| {
            map.insert(key.clone(), error);
        });

        // return the key, which will be owned by the Result being rendered and can be used to
        // unregister this error if it is rebuilt
        key
    }

    fn clear(&self, id: &throw_error::ErrorId) {
        self.errors.update(|map| {
            map.remove(id);
        });
    }
}

/// A struct to hold all the possible errors that could be provided by child Views
#[derive(Debug, Clone, Default)]
#[repr(transparent)]
pub struct Errors(FxHashMap<ErrorId, Error>);

impl Errors {
    /// Returns `true` if there are no errors.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Add an error to Errors that will be processed by `<ErrorBoundary/>`
    pub fn insert<E>(&mut self, key: ErrorId, error: E)
    where
        E: Into<Error>,
    {
        self.0.insert(key, error.into());
    }

    /// Add an error with the default key for errors outside the reactive system
    pub fn insert_with_default_key<E>(&mut self, error: E)
    where
        E: Into<Error>,
    {
        self.0.insert(Default::default(), error.into());
    }

    /// Remove an error to Errors that will be processed by `<ErrorBoundary/>`
    pub fn remove(&mut self, key: &ErrorId) -> Option<Error> {
        self.0.remove(key)
    }

    /// An iterator over all the errors, in arbitrary order.
    #[inline(always)]
    pub fn iter(&self) -> Iter<'_> {
        Iter(self.0.iter())
    }
}

impl IntoIterator for Errors {
    type Item = (ErrorId, Error);
    type IntoIter = IntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

/// An owning iterator over all the errors contained in the [`Errors`] struct.
#[repr(transparent)]
pub struct IntoIter(std::collections::hash_map::IntoIter<ErrorId, Error>);

impl Iterator for IntoIter {
    type Item = (ErrorId, Error);

    #[inline(always)]
    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> {
        self.0.next()
    }
}

/// An iterator over all the errors contained in the [`Errors`] struct.
#[repr(transparent)]
pub struct Iter<'a>(std::collections::hash_map::Iter<'a, ErrorId, Error>);

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a ErrorId, &'a Error);

    #[inline(always)]
    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> {
        self.0.next()
    }
}
