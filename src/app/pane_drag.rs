// SPDX-License-Identifier: MPL-2.0

//! Alt-drag rearranges panes with the same tree moves `Ctrl-w H/J/K/L` make.
//!
//! A press with Alt held picks up the pane under the pointer; releasing it
//! lays the pane along a side of the layout, splits another pane to make room
//! for it, or swaps contents with another pane. Shared borders keep resizing
//! whether Alt is held or not, so the two gestures never compete for a cell.

use std::collections::HashMap;

use super::{
    App, Axis, Edge, InputGrammar, Layout, Modifiers, PointerButton, PointerDrag, PointerEvent,
    PointerEventKind, PreparedView, Rect, pointer_pane, pointer_resize_pair, rect_contains,
};

/// Columns along the left and right of the layout that lay a pane along
/// that whole side. Cells are about twice as tall as they are wide, so the
/// side bands are twice as wide as the top and bottom ones are tall.
const SIDE_BAND_COLUMNS: u16 = 2;
const SIDE_BAND_ROWS: u16 = 1;

/// Where an Alt-dragged pane would land if it were released now.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PaneDrop {
    /// Along one whole side of the layout, spanning everything else.
    Edge { axis: Axis, edge: Edge },
    /// Into one half of another pane, which is split to make room.
    Beside {
        target: usize,
        axis: Axis,
        edge: Edge,
    },
    /// Into another pane's place, exchanging contents with it.
    Swap { target: usize },
}

impl App {
    /// Whether a release needs to resolve a pane drop against its visible frame.
    pub fn pointer_pane_drag_active(&self) -> bool {
        matches!(self.pointer_drag, Some(PointerDrag::MovePane { .. }))
    }

    /// Keyboard and semantic commands cancel the move, but cannot hand the
    /// remainder of its physical gesture to a terminal or another pane.
    pub(super) fn cancel_pointer_drag_preserving_pane_release(&mut self) {
        let owned = self.pane_pointer_owned;
        self.cancel_pointer_drag();
        self.pane_pointer_owned = owned;
    }

    pub(super) fn consume_cancelled_pane_pointer(&mut self, event: PointerEvent) -> bool {
        if !self.pane_pointer_owned || self.pointer_pane_drag_active() {
            return false;
        }
        match event.kind {
            PointerEventKind::Down(_) => {
                self.pane_pointer_owned = false;
                false
            }
            PointerEventKind::Up(_) => {
                self.pane_pointer_owned = false;
                true
            }
            _ => true,
        }
    }

    /// Starts, follows, or completes an Alt-drag of a pane.
    ///
    /// Returns whether the event belonged to one, in which case nothing else
    /// may interpret it: not even a terminal child that asked for the mouse.
    pub(super) fn handle_pane_drag(&mut self, event: PointerEvent, view: &PreparedView) -> bool {
        if let Some(PointerDrag::MovePane { pane, .. }) = self.pointer_drag {
            match event.kind {
                PointerEventKind::Drag(PointerButton::Left) => {
                    let drop = self.pane_drop_at(view, pane, event.column, event.row);
                    self.pointer_drag = Some(PointerDrag::MovePane { pane, drop });
                    self.describe_pane_drop(drop);
                }
                PointerEventKind::Up(_) => {
                    self.pane_pointer_owned = false;
                    self.pointer_drag = None;
                    let drop = self.pane_drop_at(view, pane, event.column, event.row);
                    self.drop_pane(pane, drop);
                }
                _ => {}
            }
            return true;
        }
        if event.kind != PointerEventKind::Down(PointerButton::Left)
            || event.modifiers != Modifiers::ALT
            || pointer_resize_pair(view, event.column, event.row).is_some()
        {
            return false;
        }
        let Some(pane) = pointer_pane(view, event.column, event.row) else {
            return false;
        };
        self.pane_pointer_owned = true;
        self.grammar.reset();
        self.jump = None;
        if let Some(maximized) = self.maximized {
            self.action_failed(format!(
                "{} keeps the current pane maximized",
                maximized.view.label()
            ));
            return true;
        }
        if matches!(self.layout, Layout::Pane(_)) {
            self.status("only one pane");
            return true;
        }
        self.activate_pane_from_pointer(pane);
        self.dismiss_popups();
        self.pointer_drag = Some(PointerDrag::MovePane { pane, drop: None });
        self.describe_pane_drop(None);
        true
    }

    /// The area the dragged pane would occupy if it were released now, found
    /// by making the move on a copy of the layout.
    pub(crate) fn pane_drop_preview(&self, view: &PreparedView) -> Option<Rect> {
        let Some(PointerDrag::MovePane {
            pane,
            drop: Some(drop),
        }) = self.pointer_drag
        else {
            return None;
        };
        if let PaneDrop::Swap { target } = drop {
            return view.pane(target).map(|prepared| prepared.area);
        }
        let mut layout = self.layout.clone();
        if !apply_drop(&mut layout, pane, drop) {
            return None;
        }
        let mut areas = HashMap::new();
        layout.rectangles(view.layout_area(), &mut areas);
        areas.get(&pane).copied()
    }

    /// Where `pane` would land at this cell, or nothing when releasing there
    /// would leave the layout as it is.
    fn pane_drop_at(
        &self,
        view: &PreparedView,
        pane: usize,
        column: u16,
        row: u16,
    ) -> Option<PaneDrop> {
        let drop = drop_at(view, pane, column, row)?;
        if let PaneDrop::Swap { target } = drop {
            return self.panes.contains_key(&target).then_some(drop);
        }
        apply_drop(&mut self.layout.clone(), pane, drop).then_some(drop)
    }

    /// Says on the status line what releasing the pane here would do.
    fn describe_pane_drop(&mut self, drop: Option<PaneDrop>) {
        let message = match drop {
            None => "moving pane: release on a layout side, a pane's side, or a pane's middle"
                .to_owned(),
            Some(PaneDrop::Edge { axis, edge }) => {
                format!("release to move pane to the {} edge", edge_name(axis, edge))
            }
            Some(PaneDrop::Beside { axis, edge, .. }) => format!(
                "release to move pane into the {} half of this pane",
                edge_name(axis, edge)
            ),
            Some(PaneDrop::Swap { .. }) => "release to swap pane contents".to_owned(),
        };
        self.status(message);
    }

    fn drop_pane(&mut self, pane: usize, drop: Option<PaneDrop>) {
        let Some(drop) = drop else {
            self.status("pane left where it was");
            return;
        };
        if !self.panes.contains_key(&pane) {
            return;
        }
        if self.active_pane != pane {
            self.activate_pane(pane);
        }
        match drop {
            PaneDrop::Edge { axis, edge } => self.move_pane_to_edge(axis, edge),
            PaneDrop::Beside { target, axis, edge } => self.move_pane_beside(target, axis, edge),
            PaneDrop::Swap { target } => self.swap_pane_contents(target),
        }
    }
}

impl PreparedView {
    /// The area the split tree is laid out in: the editor area less the
    /// directory tree sidebar.
    pub(crate) fn layout_area(&self) -> Rect {
        let tree = self.tree_area.map_or(0, |tree| tree.width);
        Rect {
            x: self.geometry.editor.x.saturating_add(tree),
            width: self.geometry.editor.width.saturating_sub(tree),
            ..self.geometry.editor
        }
    }
}

/// How a status message names one side of the layout or of a pane.
pub(super) fn edge_name(axis: Axis, edge: Edge) -> &'static str {
    match (axis, edge) {
        (Axis::Horizontal, Edge::Start) => "left",
        (Axis::Horizontal, Edge::End) => "right",
        (Axis::Vertical, Edge::Start) => "top",
        (Axis::Vertical, Edge::End) => "bottom",
    }
}

fn apply_drop(layout: &mut Layout, pane: usize, drop: PaneDrop) -> bool {
    match drop {
        PaneDrop::Edge { axis, edge } => layout.move_to_edge(pane, axis, edge),
        PaneDrop::Beside { target, axis, edge } => layout.move_beside(pane, target, axis, edge),
        PaneDrop::Swap { target } => target != pane,
    }
}

/// Reads a cell as a drop target, before asking whether it changes anything.
///
/// The bands along the outside of the layout come first, so a pane against
/// a side can still be carried along that whole side. Inside another pane,
/// the quarter nearest each of its sides splits it on that side, and the
/// middle swaps; a corner belongs to whichever side it is relatively nearer.
fn drop_at(view: &PreparedView, dragged: usize, column: u16, row: u16) -> Option<PaneDrop> {
    let area = view.layout_area();
    if !rect_contains(area, column, row) {
        return None;
    }
    let right = area.x.saturating_add(area.width);
    let bottom = area.y.saturating_add(area.height);
    let edge = if column < area.x.saturating_add(SIDE_BAND_COLUMNS) {
        Some((Axis::Horizontal, Edge::Start))
    } else if column >= right.saturating_sub(SIDE_BAND_COLUMNS) {
        Some((Axis::Horizontal, Edge::End))
    } else if row < area.y.saturating_add(SIDE_BAND_ROWS) {
        Some((Axis::Vertical, Edge::Start))
    } else if row >= bottom.saturating_sub(SIDE_BAND_ROWS) {
        Some((Axis::Vertical, Edge::End))
    } else {
        None
    };
    if let Some((axis, edge)) = edge {
        return Some(PaneDrop::Edge { axis, edge });
    }

    let target = pointer_pane(view, column, row).filter(|target| *target != dragged)?;
    let pane = view.pane(target)?.area;
    // Distances to each side as a share of the pane's own extent, measured
    // from cell centres so a pane's first and last cells weigh the same.
    let across = |offset: u16, extent: u16| (f32::from(offset) + 0.5) / f32::from(extent.max(1));
    let x = across(column - pane.x, pane.width);
    let y = across(row - pane.y, pane.height);
    let (distance, axis, edge) = [
        (x, Axis::Horizontal, Edge::Start),
        (1.0 - x, Axis::Horizontal, Edge::End),
        (y, Axis::Vertical, Edge::Start),
        (1.0 - y, Axis::Vertical, Edge::End),
    ]
    .into_iter()
    .min_by(|left, right| left.0.total_cmp(&right.0))?;
    Some(if distance < 0.25 {
        PaneDrop::Beside { target, axis, edge }
    } else {
        PaneDrop::Swap { target }
    })
}
