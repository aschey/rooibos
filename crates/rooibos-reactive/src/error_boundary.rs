use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use reactive_graph::computed::ArcMemo;
use reactive_graph::effect::RenderEffect;
use reactive_graph::signal::ArcRwSignal;
use reactive_graph::traits::{Get, Update, With};
use rustc_hash::FxHashMap;
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
    ErrorBoundaryView {
        hook,
        errors_empty,
        children,
        fallback,
        errors,
    }
}

struct ErrorBoundaryView<Chil, FalFn> {
    hook: Arc<dyn ErrorHook>,
    errors_empty: ArcMemo<bool>,
    children: Chil,
    fallback: FalFn,
    errors: ArcRwSignal<Errors>,
}

struct ErrorBoundaryViewState<Chil, Fal>
where
    Chil: Mountable<RooibosDom>,
    Fal: Mountable<RooibosDom>,
{
    // both the children and the fallback are always present, and we toggle between the two of them
    // as needed
    children: Chil,
    fallback: Option<Fal>,
}

impl<Chil, Fal> Mountable<RooibosDom> for ErrorBoundaryViewState<Chil, Fal>
where
    Chil: Mountable<RooibosDom>,
    Fal: Mountable<RooibosDom>,
{
    fn unmount(&mut self) {
        if let Some(fallback) = &mut self.fallback {
            fallback.unmount();
        } else {
            self.children.unmount();
        }
    }

    fn mount(&mut self, parent: &DomNode, marker: Option<&DomNode>) {
        if let Some(fallback) = &mut self.fallback {
            fallback.mount(parent, marker);
        } else {
            self.children.mount(parent, marker);
        }
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<RooibosDom>) -> bool {
        if let Some(fallback) = &self.fallback {
            fallback.insert_before_this(child)
        } else {
            self.children.insert_before_this(child)
        }
    }
}

impl<Chil, FalFn, Fal> Render<RooibosDom> for ErrorBoundaryView<Chil, FalFn>
where
    Chil: RenderAny + 'static,
    FalFn: FnMut(ArcRwSignal<Errors>) -> Fal + Send + 'static,
    Fal: RenderAny + 'static,
{
    type State = RenderEffect<ErrorBoundaryViewState<Chil::State, Fal::State>>;

    fn build(mut self) -> Self::State {
        let hook = Arc::clone(&self.hook);
        let _hook = throw_error::set_error_hook(Arc::clone(&hook));
        let mut children = Some(self.children.build());
        RenderEffect::new(
            move |prev: Option<ErrorBoundaryViewState<Chil::State, Fal::State>>| {
                let _hook = throw_error::set_error_hook(Arc::clone(&hook));
                if let Some(mut state) = prev {
                    match (self.errors_empty.get(), &mut state.fallback) {
                        // no errors, and was showing fallback
                        (true, Some(fallback)) => {
                            fallback.insert_before_this(&mut state.children);
                            fallback.unmount();
                            state.fallback = None;
                        }
                        // yes errors, and was showing children
                        (false, None) => {
                            state.fallback = Some((self.fallback)(self.errors.clone()).build());
                            state.children.insert_before_this(&mut state.fallback);
                            state.children.unmount();
                        }
                        // either there were no errors, and we were already showing the children
                        // or there are errors, but we were already showing the fallback
                        // in either case, rebuilding doesn't require us to do anything
                        _ => {}
                    }
                    state
                } else {
                    let fallback = (!self.errors_empty.get())
                        .then(|| (self.fallback)(self.errors.clone()).build());
                    ErrorBoundaryViewState {
                        children: children.take().unwrap(),
                        fallback,
                    }
                }
            },
        )
    }

    fn rebuild(self, state: &mut Self::State) {
        let new = self.build();
        let mut old = std::mem::replace(state, new);
        old.insert_before_this(state);
        old.unmount();
    }
}

#[derive(Debug)]
struct ErrorBoundaryErrorHook {
    errors: ArcRwSignal<Errors>,
    id: AtomicUsize,
}

impl ErrorBoundaryErrorHook {
    pub fn new(initial_errors: impl IntoIterator<Item = (ErrorId, Error)>) -> Self {
        Self {
            id: AtomicUsize::new(0),
            errors: ArcRwSignal::new(Errors(initial_errors.into_iter().collect())),
        }
    }
}

impl ErrorHook for ErrorBoundaryErrorHook {
    fn throw(&self, error: Error) -> ErrorId {
        // generate a unique ID
        let key = ErrorId::from(self.id.fetch_add(1, Ordering::Relaxed));

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
