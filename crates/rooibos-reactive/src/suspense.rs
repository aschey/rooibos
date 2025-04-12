use reactive_graph::computed::ArcMemo;
use reactive_graph::computed::suspense::SuspenseContext;
use reactive_graph::effect::RenderEffect;
use reactive_graph::owner::{Owner, provide_context};
use reactive_graph::signal::ArcRwSignal;
use reactive_graph::traits::{Get, Track, With};
use slotmap::{DefaultKey, SlotMap};
use tachys::reactive_graph::{OwnedView, OwnedViewState};
use tachys::view::either::{EitherKeepAlive, EitherKeepAliveState};
use tachys::view::{Mountable, Render};

use crate::dom::{RenderAny, RooibosDom, ViewFnOnce};

#[macro_export]
macro_rules! suspense {
    ($fallback:expr, $sus:expr, $errors:expr) => {
        $crate::suspense(
            move || $fallback,
            move || {
                $crate::error_boundary(
                    move || $crate::__tachys_reactive::Suspend::new(async move { $sus }),
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
                    move || $crate::__tachys_reactive::Suspend::new(async move { $sus }),
                    $errors,
                )
            },
        )
    };
}

pub fn suspense<F, VF, R>(fallback: VF, children: F) -> impl RenderAny
where
    F: Fn() -> R,
    F: RenderAny,
    VF: Into<ViewFnOnce>,
    R: RenderAny,
{
    suspense_internal(fallback, children, false)
}

pub fn transition<F, VF, R>(fallback: VF, children: F) -> impl RenderAny
where
    F: Fn() -> R,
    F: RenderAny,
    VF: Into<ViewFnOnce>,
    R: RenderAny,
{
    suspense_internal(fallback, children, true)
}

fn suspense_internal<F, VF, R>(fallback: VF, children: F, transition: bool) -> impl RenderAny
where
    F: Fn() -> R,
    F: RenderAny,
    VF: Into<ViewFnOnce>,
    R: RenderAny,
{
    let owner = Owner::new();
    owner.with(|| {
        let fallback = fallback.into().run();

        let tasks = ArcRwSignal::new(SlotMap::<DefaultKey, ()>::new());
        provide_context(SuspenseContext {
            tasks: tasks.clone(),
        });
        let none_pending = ArcMemo::new(move |prev: Option<&bool>| {
            tasks.track();
            if prev.is_none() {
                false
            } else {
                tasks.with(SlotMap::is_empty)
            }
        });

        OwnedView::new(SuspenseBoundary {
            none_pending,
            fallback,
            children,
            transition,
        })
    })
}

pub struct SuspenseBoundary<Fal, Chil> {
    pub none_pending: ArcMemo<bool>,
    pub fallback: Fal,
    pub children: Chil,
    transition: bool,
}

impl<Fal, Chil> Render<RooibosDom> for SuspenseBoundary<Fal, Chil>
where
    Fal: RenderAny,
    Chil: RenderAny,
{
    type State =
        RenderEffect<OwnedViewState<EitherKeepAliveState<Chil::State, Fal::State>, RooibosDom>>;

    fn build(self) -> Self::State {
        let mut children = Some(self.children);
        let mut fallback = Some(self.fallback);
        let none_pending = self.none_pending;
        let mut nth_run = 0;
        let outer_owner = Owner::new();
        let transition = self.transition;

        RenderEffect::new(move |prev| {
            // show the fallback if
            // 1) there are pending futures, and
            // 2) we are either in a Suspense (not Transition), or it's the first fallback (because
            //    we initially render the children to register Futures, the "first fallback" is
            //    probably the 2nd run
            let show_b = !none_pending.get() && (!transition || nth_run < 2);
            nth_run += 1;
            let this = OwnedView::new_with_owner(
                EitherKeepAlive {
                    a: children.take(),
                    b: fallback.take(),
                    show_b,
                },
                outer_owner.clone(),
            );

            if let Some(mut state) = prev {
                this.rebuild(&mut state);
                state
            } else {
                this.build()
            }
        })
    }

    fn rebuild(self, state: &mut Self::State) {
        let new = self.build();
        let mut old = std::mem::replace(state, new);
        old.insert_before_this(state);
        old.unmount();
    }
}
