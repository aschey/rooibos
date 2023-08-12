//! Side effects!

use super::{create_child_scope, Scope, SignalId};
use slotmap::new_key_type;

new_key_type! { pub(crate) struct EffectId; }

pub(crate) struct EffectState {
    /// The callback of the effect. This is an `Option` so that we can temporarily take the
    /// callback out to call it without holding onto a mutable borrow of all the effects.
    pub callback: Option<Box<dyn FnMut()>>,
    /// A list of signals that will trigger this effect.
    pub dependencies: Vec<SignalId>,
    /// An internal state to prevent an effect from running twice in the same update.
    pub already_run_in_update: bool,
}

/// Creates an effect on signals used inside the effect closure.
///
/// # Example
/// ```
/// # use sycamore_reactive3::*;
/// # create_root(|cx| {
/// let state = create_signal(cx, 0);
///
/// create_effect(cx, move || {
///     println!("State changed. New state value = {}", state.get());
/// });
/// // Prints "State changed. New state value = 0"
///
/// state.set(1);
/// // Prints "State changed. New state value = 1"
/// # });
/// ```
///
/// `create_effect` should only be used for creating **side-effects**. It is generally not
/// recommended to update signal states inside an effect. You probably should be using a
/// [`create_memo`](crate::create_memo) instead.
pub fn create_effect(cx: Scope, mut f: impl FnMut() + 'static) {
    // Run the effect right now so we can get the dependencies.
    let (_, tracker) = cx.root.tracked_scope(&mut f);
    let key = cx.root.effects.borrow_mut().insert(EffectState {
        callback: Some(Box::new(f)),
        dependencies: Vec::new(),
        already_run_in_update: false,
    });
    cx.get_data(|data| data.effects.push(key));
    // Add the dependency links.
    tracker.create_effect_dependency_links(cx.root, key);
}

/// Creates an effect on signals used inside the effect closure.
///
/// Unlike [`create_effect`], this function also provides a new reactive scope instead the
/// effect closure. This scope is created for each new run of the effect.
///
/// # Example
/// ```
/// # use sycamore_reactive3::*;
/// # create_root(|cx| {
/// create_effect_scoped(cx, |cx| {
///     // Use the scoped cx inside here.
///     let _nested_signal = create_signal(cx, 0);
///     // _nested_signal cannot escape out of the effect closure.
/// });
/// # });
/// ```
pub fn create_effect_scoped(cx: Scope, mut f: impl FnMut(Scope) + 'static) {
    let mut child_scope: Option<Scope> = None;
    create_effect(cx, move || {
        if let Some(child_scope) = child_scope {
            child_scope.dispose();
        }
        child_scope = Some(create_child_scope(cx, &mut f));
    });
}
