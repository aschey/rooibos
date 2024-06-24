use reactive_graph::computed::suspense::SuspenseContext;
use reactive_graph::computed::ArcMemo;
use reactive_graph::effect::RenderEffect;
use reactive_graph::owner::provide_context;
use reactive_graph::signal::ArcRwSignal;
use reactive_graph::traits::{Get, With};
use slotmap::{DefaultKey, SlotMap};
use tachys::reactive_graph::OwnedView;
use tachys::view::either::{EitherKeepAlive, EitherKeepAliveState};
use tachys::view::{Mountable, Render};

use crate::{AnyView, IntoView, RenderAny, RooibosDom, ViewFnOnce};

#[macro_export]
macro_rules! suspense {
    ($fallback:expr, $sus:expr, $errors:expr) => {
        $crate::suspense(
            move || $fallback,
            move || {
                $crate::error_boundary(
                    move || $crate::__tachys_reactive::Suspend(async move { $sus }),
                    $errors,
                )
            },
        )
    };
}

#[macro_export]
macro_rules! transition {
    ($fallback:expr, $sus:expr, $errors:expr) => {
        $crate::transition(
            move || $fallback,
            move || {
                $crate::error_boundary(
                    move || $crate::__tachys_reactive::Suspend(async move { $sus }),
                    $errors,
                )
            },
        )
    };
}

pub fn suspense<F, R>(fallback: impl Into<ViewFnOnce>, children: F) -> impl IntoView
where
    F: Fn() -> R,
    R: RenderAny,
    SuspenseBoundary<false, AnyView, F>: IntoView,
{
    let fallback = fallback.into().run();

    let tasks = ArcRwSignal::new(SlotMap::<DefaultKey, ()>::new());
    provide_context(SuspenseContext {
        tasks: tasks.clone(),
    });
    let none_pending = ArcMemo::new(move |_| tasks.with(SlotMap::is_empty));
    OwnedView::new(SuspenseBoundary::<false, _, _> {
        none_pending,
        fallback,
        children,
    })
}

pub fn transition<F, R>(fallback: impl Into<ViewFnOnce>, children: F) -> impl RenderAny
where
    F: Fn() -> R,
    R: RenderAny,
    SuspenseBoundary<true, AnyView, F>: IntoView,
{
    let fallback = fallback.into().run();
    let tasks = ArcRwSignal::new(SlotMap::<DefaultKey, ()>::new());
    provide_context(SuspenseContext {
        tasks: tasks.clone(),
    });
    let none_pending = ArcMemo::new(move |_| tasks.with(SlotMap::is_empty));
    OwnedView::new(SuspenseBoundary::<true, _, _> {
        none_pending,
        fallback,
        children,
    })
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
    type State = RenderEffect<EitherKeepAliveState<Chil::State, Fal::State>>;

    fn build(self) -> Self::State {
        let mut children = Some(self.children);
        let mut fallback = Some(self.fallback);
        let none_pending = self.none_pending;
        let mut nth_run = 0;

        RenderEffect::new(
            move |prev: Option<EitherKeepAliveState<Chil::State, Fal::State>>| {
                // show the fallback if
                // 1) there are pending futures, and
                // 2) we are either in a Suspense (not Transition), or it's the first fallback
                //    (because we initially render the children to register Futures, the "first
                //    fallback" is probably the 2nd run
                let show_b = !none_pending.get() && (!TRANSITION || nth_run < 2);
                nth_run += 1;
                let this = EitherKeepAlive {
                    a: children.take(),
                    b: fallback.take(),
                    show_b,
                };

                if let Some(mut state) = prev {
                    this.rebuild(&mut state);
                    state
                } else {
                    this.build()
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
