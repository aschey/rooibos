use std::cell::RefCell;
use std::fmt::Debug;
use std::future::Future;
use std::rc::Rc;

use any_spawner::Executor;
use futures::FutureExt;
use reactive_graph::computed::{ArcMemo, ScopedFuture};
use reactive_graph::owner::{provide_context, use_context};
use reactive_graph::signal::ArcRwSignal;
use reactive_graph::traits::{Get, Update, With, Writeable};
use slotmap::{DefaultKey, SlotMap};
use tachys::reactive_graph::RenderEffectState;
use tachys::view::either::{EitherKeepAlive, EitherKeepAliveState};
use tachys::view::iterators::OptionState;
use tachys::view::{Mountable, Render};

use crate::{DomNode, IntoView, RenderAny, RooibosDom, TypedChildren, ViewFnOnce};

pub fn suspense<Chil>(fallback: ViewFnOnce, children: TypedChildren<Chil>) -> impl IntoView
where
    Chil: RenderAny + Send + 'static,
{
    let fallback = fallback.run();
    let children = children.into_inner()();
    let tasks = ArcRwSignal::new(SlotMap::<DefaultKey, ()>::new());
    provide_context(SuspenseContext {
        tasks: tasks.clone(),
    });
    let none_pending = ArcMemo::new(move |_| tasks.with(SlotMap::is_empty));
    SuspenseBoundary::<false, _, _> {
        none_pending,
        fallback,
        children,
    }
}

pub fn transition<Chil>(
    fallback: impl Into<ViewFnOnce>,
    children: impl Into<TypedChildren<Chil>>,
) -> impl RenderAny
where
    Chil: RenderAny + Send + 'static,
{
    let fallback = fallback.into().run();
    let children = children.into().into_inner()();
    let tasks = ArcRwSignal::new(SlotMap::<DefaultKey, ()>::new());
    provide_context(SuspenseContext {
        tasks: tasks.clone(),
    });
    let none_pending = ArcMemo::new(move |_| tasks.with(SlotMap::is_empty));
    SuspenseBoundary::<true, _, _> {
        none_pending,
        fallback,
        children,
    }
}

pub struct SuspenseBoundary<const TRANSITION: bool, Fal, Chil> {
    pub none_pending: ArcMemo<bool>,
    pub fallback: Fal,
    pub children: Chil,
}

impl<const TRANSITION: bool, Fal, Chil> Render<RooibosDom>
    for SuspenseBoundary<TRANSITION, Fal, Chil>
where
    Fal: RenderAny + Send + 'static,
    Chil: RenderAny + Send + 'static,
{
    type State = RenderEffectState<EitherKeepAliveState<Chil::State, Fal::State, RooibosDom>>;

    fn build(self) -> Self::State {
        let mut children = Some(self.children);
        let mut fallback = Some(self.fallback);
        let none_pending = self.none_pending;
        let mut nth_run = 0;

        (move || {
            // show the fallback if
            // 1) there are pending futures, and
            // 2) we are either in a Suspense (not Transition), or it's the first fallback (because
            //    we initially render the children to register Futures, the "first fallback" is
            //    probably the 2nd run
            let show_b = !none_pending.get() && (!TRANSITION || nth_run < 2);
            nth_run += 1;
            EitherKeepAlive {
                a: children.take(),
                b: fallback.take(),
                show_b,
            }
        })
        .build()
    }

    fn rebuild(self, _state: &mut Self::State) {}
}

#[derive(Clone, Debug)]
pub(crate) struct SuspenseContext {
    pub tasks: ArcRwSignal<SlotMap<DefaultKey, ()>>,
}

impl SuspenseContext {
    pub fn task_id(&self) -> TaskHandle {
        let key = self.tasks.write().insert(());
        TaskHandle {
            tasks: self.tasks.clone(),
            key,
        }
    }
}

/// A unique identifier that removes itself from the set of tasks when it is dropped.
#[derive(Debug)]
pub(crate) struct TaskHandle {
    tasks: ArcRwSignal<SlotMap<DefaultKey, ()>>,
    key: DefaultKey,
}

impl Drop for TaskHandle {
    fn drop(&mut self) {
        self.tasks.update(|tasks| {
            tasks.remove(self.key);
        });
    }
}

pub trait FutureViewExt: Sized {
    fn wait(self) -> Suspend<Self>
    where
        Self: Future,
    {
        Suspend(self)
    }
}

impl<F> FutureViewExt for F where F: Future + Sized {}

/* // TODO remove in favor of Suspend()?
#[macro_export]
macro_rules! suspend {
    ($fut:expr) => {
        move || $crate::prelude::FutureViewExt::wait(async move { $fut })
    };
}*/

pub struct Suspend<Fut>(pub Fut);

impl<Fut> Debug for Suspend<Fut> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Suspend").finish()
    }
}

pub struct SuspendState<T>
where
    T: RenderAny,
{
    inner: Rc<RefCell<OptionState<T::State, RooibosDom>>>,
}

impl<T> Mountable<RooibosDom> for SuspendState<T>
where
    T: RenderAny,
{
    fn unmount(&mut self) {
        self.inner.borrow_mut().unmount();
    }

    fn mount(&mut self, parent: &DomNode, marker: Option<&DomNode>) {
        self.inner.borrow_mut().mount(parent, marker);
    }

    fn insert_before_this(&self, parent: &DomNode, child: &mut dyn Mountable<RooibosDom>) -> bool {
        self.inner.borrow_mut().insert_before_this(parent, child)
    }
}

impl<Fut> Render<RooibosDom> for Suspend<Fut>
where
    Fut: Future + 'static,
    Fut::Output: RenderAny,
{
    type State = SuspendState<Fut::Output>;

    // TODO cancellation if it fires multiple times
    fn build(self) -> Self::State {
        // poll the future once immediately
        // if it's already available, start in the ready state
        // otherwise, start with the fallback
        let mut fut = Box::pin(ScopedFuture::new(self.0));
        let initial = fut.as_mut().now_or_never();
        let initially_pending = initial.is_none();
        let inner = Rc::new(RefCell::new(initial.build()));

        // get a unique ID if there's a SuspenseContext
        let id = use_context::<SuspenseContext>().map(|sc| sc.task_id());

        // if the initial state was pending, spawn a future to wait for it
        // spawning immediately means that our now_or_never poll result isn't lost
        // if it wasn't pending at first, we don't need to poll the Future again
        if initially_pending {
            Executor::spawn_local({
                let state = Rc::clone(&inner);
                async move {
                    let value = fut.as_mut().await;
                    drop(id);
                    Some(value).rebuild(&mut *state.borrow_mut());
                }
            });
        }

        SuspendState { inner }
    }

    fn rebuild(self, state: &mut Self::State) {
        // get a unique ID if there's a SuspenseContext
        let fut = ScopedFuture::new(self.0);
        let id = use_context::<SuspenseContext>().map(|sc| sc.task_id());

        // spawn the future, and rebuild the state when it resolves
        Executor::spawn_local({
            let state = Rc::clone(&state.inner);
            async move {
                let value = fut.await;
                drop(id);
                Some(value).rebuild(&mut *state.borrow_mut());
            }
        });
    }
}
