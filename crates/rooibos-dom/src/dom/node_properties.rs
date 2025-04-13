use core::fmt::Debug;
use std::cell::RefCell;
use std::mem;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use educe::Educe;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::widgets::{
    Block, Clear, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
    WidgetRef,
};
#[cfg(feature = "effects")]
use tachyonfx::{EffectRenderer, Shader};
use terminput::ScrollDirection;

use super::node_tree::{DomNodeKey, NodeTree};
use super::{ContentRect, NodeId, NodeType, next_node_id, refresh_dom};
use crate::events::EventHandlers;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusScope {
    last_focused: Option<DomNodeKey>,
    contain: bool,
    id: u32,
}

impl FocusScope {
    pub(crate) fn new() -> Self {
        Self {
            last_focused: None,
            contain: false,
            id: next_node_id(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FocusMode {
    Tab,
    List,
    #[default]
    None,
}

#[derive(Educe)]
#[educe(Debug, Default)]
pub struct NodeProperties {
    pub(crate) node_type: NodeType,
    pub(crate) name: String,
    pub(crate) original_display: taffy::Display,
    pub(crate) children: Vec<DomNodeKey>,
    pub(crate) parent: Option<DomNodeKey>,
    pub(crate) id: Option<NodeId>,
    pub(crate) class: Vec<String>,
    pub(crate) focus_mode: FocusMode,
    #[educe(Debug(ignore))]
    pub(crate) event_handlers: EventHandlers,
    pub(crate) rect: RefCell<Rect>,
    pub(crate) z_index: Option<i32>,
    pub(crate) block: Option<Block<'static>>,
    pub(crate) clear: bool,
    pub(crate) scroll_offset: Position,
    pub(crate) ancestor_scroll_offset: Position,
    pub(crate) max_scroll_offset: Position,
    #[cfg(feature = "effects")]
    pub(crate) effects: Option<RefCell<EffectProperties>>,
    #[educe(Default = true)]
    pub(crate) enabled: bool,
    #[educe(Default = true)]
    pub(crate) parent_enabled: bool,
    pub(crate) unmounted: Arc<AtomicBool>,
    #[educe(Default = AtomicBool::new(true))]
    pub(crate) visible: AtomicBool,
}

pub(crate) struct RenderProps<'a, 'b> {
    pub(crate) frame: &'a mut Frame<'b>,
    pub(crate) window: Rect,
    pub(crate) parent_bounds: Rect,
    pub(crate) parent_scroll_offset: Position,
    pub(crate) key: DomNodeKey,
    pub(crate) dom_nodes: &'a NodeTree,
}

#[cfg(feature = "effects")]
#[derive(Debug, Clone)]
pub struct EffectProperties {
    effect: tachyonfx::Effect,
    last_tick: std::time::Instant,
}

#[cfg(feature = "effects")]
impl EffectProperties {
    pub(crate) fn new(effect: tachyonfx::Effect) -> Self {
        Self {
            effect,
            last_tick: std::time::Instant::now(),
        }
    }

    fn tick(&mut self) -> tachyonfx::Duration {
        let elapsed = self.last_tick.elapsed();
        self.last_tick = std::time::Instant::now();
        elapsed.into()
    }
}

impl NodeProperties {
    pub(crate) fn enabled(&self) -> bool {
        // If a node is disabled, all children should also be disabled
        self.enabled && self.parent_enabled
    }

    pub(crate) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub(crate) fn tab_focusable(&self) -> bool {
        self.focus_mode == FocusMode::Tab && self.enabled()
    }

    pub(crate) fn list_focusable(&self) -> bool {
        self.focus_mode == FocusMode::List && self.enabled()
    }

    pub(crate) fn focusable(&self) -> bool {
        self.tab_focusable() || self.list_focusable()
    }

    pub(crate) fn set_focus_mode(&mut self, focus_mode: FocusMode) {
        self.focus_mode = focus_mode;
    }

    pub(crate) fn set_parent_enabled(&mut self, enabled: bool) {
        self.parent_enabled = enabled;
    }

    pub(crate) fn visible(&self) -> bool {
        self.visible.load(Ordering::Relaxed)
    }

    pub(crate) fn position(&self) -> Rect {
        let mut rect = *self.rect.borrow();
        rect.x = rect
            .x
            .saturating_sub(self.scroll_offset.x)
            .saturating_sub(self.ancestor_scroll_offset.x);
        rect.y = rect
            .y
            .saturating_sub(self.scroll_offset.y)
            .saturating_sub(self.ancestor_scroll_offset.y);
        rect
    }

    pub(crate) fn scroll(&mut self, direction: ScrollDirection) -> taffy::Point<i32> {
        let delta = direction.delta();
        let mut x = self.scroll_offset.x as i32 - delta.x;
        let mut y = self.scroll_offset.y as i32 - delta.y;

        x = x.min(self.max_scroll_offset.x as i32).max(0);
        y = y.min(self.max_scroll_offset.y as i32).max(0);

        let x_change = x - self.scroll_offset.x as i32;
        let y_change = y - self.scroll_offset.y as i32;
        self.scroll_offset.x = x as u16;
        self.scroll_offset.y = y as u16;
        taffy::Point {
            x: x_change,
            y: y_change,
        }
    }

    pub(crate) fn update_ancestor_scroll_offsets(&mut self, change: taffy::Point<i32>) {
        self.ancestor_scroll_offset.x =
            (self.ancestor_scroll_offset.x as i32 + change.x).max(0) as u16;
        self.ancestor_scroll_offset.y =
            (self.ancestor_scroll_offset.y as i32 + change.y).max(0) as u16;
    }

    pub(crate) fn render(&self, props: RenderProps) {
        let RenderProps {
            frame,
            window,
            key,
            parent_bounds,
            parent_scroll_offset,
            dom_nodes,
        } = props;

        let prev_rect = *self.rect.borrow();
        let Some(content_rect) = dom_nodes.try_rect(key) else {
            self.children.iter().for_each(|key| {
                dom_nodes[*key].render(RenderProps {
                    frame,
                    window,
                    parent_bounds,
                    parent_scroll_offset,
                    key: *key,
                    dom_nodes,
                });
            });
            return;
        };

        if self.focusable() {
            dom_nodes.add_focusable(key);
        }

        let render_bounds = content_rect.render_bounds();

        let height_above_parent = (render_bounds.y + render_bounds.height)
            .saturating_sub(parent_bounds.y + parent_bounds.height)
            .saturating_sub(parent_scroll_offset.y);

        let mut temp_buf = if content_rect.can_scroll() || height_above_parent > 0 {
            Some(Buffer::empty(Rect::default()))
        } else {
            None
        };
        if let Some(temp_buf) = &mut temp_buf {
            content_rect.resize_for_render(temp_buf);
        }

        let visible_bounds = content_rect.visible_bounds();
        let needs_render = (visible_bounds.x
            < parent_bounds.x + parent_bounds.width + parent_scroll_offset.x)
            && (visible_bounds.y < parent_bounds.y + parent_bounds.height + parent_scroll_offset.y);

        if self.clear {
            Clear.render(render_bounds, frame.buffer_mut());
        }

        if let NodeType::Widget(widget) = &self.node_type {
            widget.recompute_done();
        }
        if needs_render {
            self.visible.store(true, Ordering::Relaxed);
            match &self.node_type {
                NodeType::Layout => {
                    if let Some(temp_buf) = &mut temp_buf {
                        temp_buf.area.x = visible_bounds.x;
                        temp_buf.area.y = visible_bounds.y;

                        mem::swap(frame.buffer_mut(), temp_buf);

                        let child_bounds = content_rect.child_bounds();

                        let mut all_child_bounds = Rect::default();
                        let mut init = false;
                        self.children.iter().for_each(|key| {
                            let visible_child_bounds = dom_nodes.rect(*key).total_size();
                            if visible_child_bounds.width > 0 && visible_child_bounds.height > 0 {
                                if !init {
                                    all_child_bounds = visible_child_bounds;
                                    init = true;
                                } else {
                                    all_child_bounds = all_child_bounds.union(visible_child_bounds);
                                }
                            }

                            dom_nodes[*key].render(RenderProps {
                                frame,
                                window,
                                parent_bounds: child_bounds,
                                parent_scroll_offset: content_rect.scroll_offset(),
                                key: *key,
                                dom_nodes,
                            });
                        });
                        content_rect.apply_scroll(frame.buffer_mut().area, frame.buffer_mut());

                        mem::swap(frame.buffer_mut(), temp_buf);

                        let buf = frame.buffer_mut();
                        for row in child_bounds.rows().take(
                            (child_bounds.height.saturating_sub(height_above_parent)) as usize,
                        ) {
                            for col in row.columns() {
                                let pos: Position = col.into();
                                if pos.x < buf.area.x + buf.area.width
                                    && pos.y < buf.area.y + buf.area.height
                                {
                                    buf[pos] = temp_buf[pos].clone();
                                }
                            }
                        }
                        self.render_block(render_bounds, frame.buffer_mut());
                        self.render_scrollbar(&content_rect, all_child_bounds, child_bounds, frame);
                    } else {
                        self.children.iter().for_each(|key| {
                            dom_nodes[*key].render(RenderProps {
                                frame,
                                window,
                                parent_bounds: content_rect.child_bounds(),
                                parent_scroll_offset: content_rect.scroll_offset(),
                                key: *key,
                                dom_nodes,
                            });
                        });
                        if let Some(block) = &self.block {
                            block.render_ref(render_bounds, frame.buffer_mut());
                        };
                    }
                }
                NodeType::FocusScope(_) => {
                    print!("")
                }
                NodeType::Widget(widget) => {
                    self.visible.store(true, Ordering::Relaxed);
                    let mut inner = content_rect.inner_bounds();
                    if let Some(temp_buf) = &mut temp_buf {
                        let total_area = temp_buf.area;

                        temp_buf.area.x = inner.x;
                        temp_buf.area.y = inner.y;

                        mem::swap(frame.buffer_mut(), temp_buf);

                        widget.render(inner, frame);

                        content_rect.apply_scroll(frame.buffer_mut().area, frame.buffer_mut());

                        mem::swap(frame.buffer_mut(), temp_buf);

                        inner.height = inner.height.saturating_sub(height_above_parent);
                        for row in inner.rows() {
                            for col in row.columns() {
                                let pos: Position = col.into();
                                frame.buffer_mut()[pos] = temp_buf[pos].clone();
                            }
                        }
                        self.render_block(render_bounds, frame.buffer_mut());
                        self.render_scrollbar(&content_rect, total_area, inner, frame);
                    } else {
                        widget.render(inner, frame);
                        self.render_block(render_bounds, frame.buffer_mut());
                    }
                }
                NodeType::Placeholder => {}
            }
            #[cfg(feature = "effects")]
            if let Some(effects) = &self.effects {
                let mut effects = effects.borrow_mut();

                if effects.effect.running() {
                    let tick = effects.tick();

                    frame.render_effect(&mut effects.effect, render_bounds, tick);
                    refresh_dom();
                }
            }
        } else {
            self.visible.store(false, Ordering::Relaxed);
        }

        *self.rect.borrow_mut() = render_bounds;
        if render_bounds != prev_rect {
            for on_size_change in &dom_nodes[key].event_handlers.on_size_change {
                on_size_change.borrow_mut()(render_bounds);
            }
        }
    }

    fn render_block(&self, bounds: Rect, buf: &mut Buffer) {
        if let Some(block) = &self.block {
            block.render_ref(bounds, buf);
        };
    }

    fn render_scrollbar(
        &self,
        content_rect: &ContentRect,
        total_area: Rect,
        render_bounds: Rect,
        frame: &mut Frame,
    ) {
        if !content_rect.can_scroll() {
            return;
        }
        let pct = content_rect.scroll_offset().y as f64
            / (total_area.height - render_bounds.height) as f64;

        Scrollbar::new(ScrollbarOrientation::VerticalRight).render(
            render_bounds,
            frame.buffer_mut(),
            &mut ScrollbarState::new(total_area.height as usize)
                .position((total_area.height as f64 * pct).floor() as usize),
        );
    }

    pub(crate) fn replace_with(&mut self, new: &NodeProperties) {
        // This is annoyingly verbose, but we use destructuring here to ensure we account for
        // any new properties that get added to DomNodeInner
        let NodeProperties {
            node_type,
            name,
            children: _children,
            parent: _parent,
            id,
            class,
            focus_mode,
            event_handlers,
            rect,
            original_display,
            block,
            clear,
            enabled,
            scroll_offset,
            ancestor_scroll_offset: _ancestor_scroll_offset,
            max_scroll_offset,
            #[cfg(feature = "effects")]
            effects,
            z_index: _z_index,
            unmounted: _unmounted,
            parent_enabled: _parent_enabled,
            visible: _visible,
        } = new;

        let name = name.clone();
        let node_type = node_type.clone();
        let focus_mode = *focus_mode;
        let id = id.clone();
        let class = class.clone();
        let event_handlers = event_handlers.clone();
        let rect = rect.clone();
        let original_display = *original_display;
        let block = block.clone();
        let clear = *clear;
        let enabled = *enabled;
        let scroll_offset = *scroll_offset;
        let max_scroll_offset = *max_scroll_offset;
        #[cfg(feature = "effects")]
        let effects = effects.clone();

        self.name = name;
        self.node_type = node_type;
        self.focus_mode = focus_mode;
        self.id = id;
        self.class = class;
        self.event_handlers = event_handlers;
        self.rect = rect;
        self.original_display = original_display;
        self.block = block;
        self.clear = clear;
        self.enabled = enabled;
        self.scroll_offset = scroll_offset;
        self.max_scroll_offset = max_scroll_offset;
        #[cfg(feature = "effects")]
        {
            self.effects = effects;
        }
    }
}
