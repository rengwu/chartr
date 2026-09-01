//! Chartr's workspace, pane, and item ownership model.
//!
//! The names and responsibilities follow Zed's `Workspace`, `PaneGroup`, and
//! `Pane`, but this module contains only Chartr's product-neutral state. GPUI
//! entities and rendering live above it. Keeping the mutations here makes the
//! invariant observable at one seam: an item belongs to exactly one pane in
//! exactly one workspace.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PaneId(u64);

impl PaneId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ItemId(u64);

impl ItemId {
    pub fn get(self) -> u64 {
        self.0
    }
}

/// One entry in a space's outer tab strip. A workspace tab may be a standalone
/// item or a pane group; that distinction is derived from its contents rather
/// than stored as a second source of truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WorkspaceTabId(u64);

impl WorkspaceTabId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitDirection {
    Up,
    Down,
    Left,
    Right,
}

impl SplitDirection {
    pub fn axis(self) -> Axis {
        match self {
            Self::Up | Self::Down => Axis::Vertical,
            Self::Left | Self::Right => Axis::Horizontal,
        }
    }

    pub fn increasing(self) -> bool {
        matches!(self, Self::Down | Self::Right)
    }
}

/// One leaf or split axis in a pane tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Member {
    Pane { pane: PaneId },
    Axis(PaneAxis),
}

impl Member {
    fn pane(pane: PaneId) -> Self {
        Self::Pane { pane }
    }

    fn new_axis(old: PaneId, new: PaneId, direction: SplitDirection) -> Self {
        let members = if direction.increasing() {
            vec![Self::pane(old), Self::pane(new)]
        } else {
            vec![Self::pane(new), Self::pane(old)]
        };
        Self::Axis(PaneAxis::new(direction.axis(), members))
    }

    fn first_pane(&self) -> PaneId {
        match self {
            Self::Pane { pane } => *pane,
            Self::Axis(axis) => axis.members[0].first_pane(),
        }
    }

    fn collect_panes(&self, panes: &mut Vec<PaneId>) {
        match self {
            Self::Pane { pane } => panes.push(*pane),
            Self::Axis(axis) => {
                for member in &axis.members {
                    member.collect_panes(panes);
                }
            }
        }
    }

    fn contains(&self, needle: PaneId) -> bool {
        match self {
            Self::Pane { pane } => *pane == needle,
            Self::Axis(axis) => axis.members.iter().any(|member| member.contains(needle)),
        }
    }

    fn collect_bounds(&self, bounds: UnitBounds, output: &mut BTreeMap<PaneId, UnitBounds>) {
        match self {
            Self::Pane { pane } => {
                output.insert(*pane, bounds);
            }
            Self::Axis(axis) => {
                let total = axis.flexes.iter().copied().sum::<f32>().max(f32::EPSILON);
                let mut offset = 0.;
                for (index, member) in axis.members.iter().enumerate() {
                    let share = axis.flexes.get(index).copied().unwrap_or(1.) / total;
                    let child = match axis.axis {
                        Axis::Horizontal => UnitBounds {
                            x: bounds.x + bounds.width * offset,
                            y: bounds.y,
                            width: bounds.width * share,
                            height: bounds.height,
                        },
                        Axis::Vertical => UnitBounds {
                            x: bounds.x,
                            y: bounds.y + bounds.height * offset,
                            width: bounds.width,
                            height: bounds.height * share,
                        },
                    };
                    member.collect_bounds(child, output);
                    offset += share;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct UnitBounds {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl UnitBounds {
    fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

/// A same-axis run inside the recursive split tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaneAxis {
    pub axis: Axis,
    pub members: Vec<Member>,
    /// Relative sizes. As in Zed, inserting or removing a member resets the
    /// containing axis to equal shares; direct divider resizing changes them.
    pub flexes: Vec<f32>,
}

impl PaneAxis {
    fn new(axis: Axis, members: Vec<Member>) -> Self {
        let flexes = vec![1.; members.len()];
        Self { axis, members, flexes }
    }

    #[cfg(test)]
    fn valid_flexes(&self) -> bool {
        self.flexes.len() == self.members.len()
            && self.flexes.iter().all(|flex| flex.is_finite() && *flex > 0.)
    }

    fn reset_flexes(&mut self) {
        self.flexes = vec![1.; self.members.len()];
    }

    fn split(&mut self, old: PaneId, new: PaneId, direction: SplitDirection) -> bool {
        for (mut index, member) in self.members.iter_mut().enumerate() {
            match member {
                Member::Axis(axis) => {
                    if axis.split(old, new, direction) {
                        return true;
                    }
                }
                Member::Pane { pane } if *pane == old => {
                    if self.axis == direction.axis() {
                        if direction.increasing() {
                            index += 1;
                        }
                        self.members.insert(index, Member::pane(new));
                        self.reset_flexes();
                    } else {
                        *member = Member::new_axis(old, new, direction);
                    }
                    return true;
                }
                Member::Pane { .. } => {}
            }
        }
        false
    }

    /// Remove a pane and return the sole surviving child when this axis should
    /// collapse into its parent.
    fn remove(&mut self, target: PaneId) -> Result<Option<Member>, ModelError> {
        let mut remove_at = None;
        let mut found = false;

        for (index, member) in self.members.iter_mut().enumerate() {
            match member {
                Member::Pane { pane } if *pane == target => {
                    remove_at = Some(index);
                    found = true;
                    break;
                }
                Member::Axis(axis) => match axis.remove(target) {
                    Ok(replacement) => {
                        if let Some(replacement) = replacement {
                            *member = replacement;
                        }
                        found = true;
                        break;
                    }
                    Err(ModelError::PaneNotFound(_)) => {}
                    Err(error) => return Err(error),
                },
                Member::Pane { .. } => {}
            }
        }

        if !found {
            return Err(ModelError::PaneNotFound(target));
        }
        if let Some(index) = remove_at {
            self.members.remove(index);
            self.reset_flexes();
        }
        if self.members.len() == 1 { Ok(self.members.pop()) } else { Ok(None) }
    }
}

/// One or more panes arranged in a recursive axis tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaneGroup {
    pub root: Member,
}

impl PaneGroup {
    pub fn new(root: PaneId) -> Self {
        Self { root: Member::pane(root) }
    }

    pub fn panes(&self) -> Vec<PaneId> {
        let mut panes = Vec::new();
        self.root.collect_panes(&mut panes);
        panes
    }

    pub fn contains(&self, pane: PaneId) -> bool {
        self.root.contains(pane)
    }

    /// Find the pane immediately across the requested edge. This mirrors
    /// Zed's pane-group navigation: use the active pane's center on the
    /// perpendicular axis and sample just beyond its bounding box.
    pub fn pane_in_direction(&self, active: PaneId, direction: SplitDirection) -> Option<PaneId> {
        let mut bounds = BTreeMap::new();
        self.root.collect_bounds(UnitBounds { x: 0., y: 0., width: 1., height: 1. }, &mut bounds);
        let active_bounds = *bounds.get(&active)?;
        let epsilon = 0.0001;
        let center_x = active_bounds.x + active_bounds.width / 2.;
        let center_y = active_bounds.y + active_bounds.height / 2.;
        let (target_x, target_y) = match direction {
            SplitDirection::Left => (active_bounds.x - epsilon, center_y),
            SplitDirection::Right => (active_bounds.x + active_bounds.width + epsilon, center_y),
            SplitDirection::Up => (center_x, active_bounds.y - epsilon),
            SplitDirection::Down => (center_x, active_bounds.y + active_bounds.height + epsilon),
        };
        bounds
            .into_iter()
            .find_map(|(pane, bounds)| bounds.contains(target_x, target_y).then_some(pane))
    }

    pub fn split(&mut self, old: PaneId, new: PaneId, direction: SplitDirection) {
        let found = match &mut self.root {
            Member::Pane { pane } if *pane == old => {
                self.root = Member::new_axis(old, new, direction);
                true
            }
            Member::Axis(axis) => axis.split(old, new, direction),
            Member::Pane { .. } => false,
        };

        // Zed falls back to splitting the first pane when a stale caller names
        // a pane that is no longer present. Preserve that convention here.
        if !found {
            let first = self.root.first_pane();
            match &mut self.root {
                Member::Pane { .. } => self.root = Member::new_axis(first, new, direction),
                Member::Axis(axis) => {
                    let _ = axis.split(first, new, direction);
                }
            }
        }
    }

    /// Remove a pane, retaining the sole root pane invariant.
    pub fn remove(&mut self, pane: PaneId) -> Result<bool, ModelError> {
        match &mut self.root {
            Member::Pane { pane: root } => {
                if *root == pane {
                    Ok(false)
                } else {
                    Err(ModelError::PaneNotFound(pane))
                }
            }
            Member::Axis(axis) => {
                if let Some(replacement) = axis.remove(pane)? {
                    self.root = replacement;
                }
                Ok(true)
            }
        }
    }

    #[cfg(test)]
    pub fn set_flexes(&mut self, axis_path: &[usize], flexes: Vec<f32>) -> Result<(), ModelError> {
        let mut member = &mut self.root;
        for &index in axis_path {
            member = match member {
                Member::Axis(axis) => axis.members.get_mut(index).ok_or(ModelError::BadAxisPath)?,
                Member::Pane { .. } => return Err(ModelError::BadAxisPath),
            };
        }
        let Member::Axis(axis) = member else {
            return Err(ModelError::BadAxisPath);
        };
        let old = std::mem::replace(&mut axis.flexes, flexes);
        if !axis.valid_flexes() {
            axis.flexes = old;
            return Err(ModelError::InvalidFlexes);
        }
        Ok(())
    }

    pub fn resize_divider(
        &mut self,
        axis_path: &[usize],
        divider: usize,
        fraction: f32,
    ) -> Result<(), ModelError> {
        let mut member = &mut self.root;
        for &index in axis_path {
            member = match member {
                Member::Axis(axis) => axis.members.get_mut(index).ok_or(ModelError::BadAxisPath)?,
                Member::Pane { .. } => return Err(ModelError::BadAxisPath),
            };
        }
        let Member::Axis(axis) = member else {
            return Err(ModelError::BadAxisPath);
        };
        if divider + 1 >= axis.flexes.len() || !fraction.is_finite() {
            return Err(ModelError::InvalidFlexes);
        }
        let total: f32 = axis.flexes.iter().sum();
        let before: f32 = axis.flexes[..divider].iter().sum();
        let pair = axis.flexes[divider] + axis.flexes[divider + 1];
        let minimum = pair * 0.1;
        let left = (fraction.clamp(0., 1.) * total - before).clamp(minimum, pair - minimum);
        axis.flexes[divider] = left;
        axis.flexes[divider + 1] = pair - left;
        Ok(())
    }
}

/// Ordered items and activation history for one pane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pane {
    pub id: PaneId,
    items: Vec<ItemId>,
    active: Option<ItemId>,
    activation_history: Vec<ItemId>,
}

impl Pane {
    fn new(id: PaneId) -> Self {
        Self { id, items: Vec::new(), active: None, activation_history: Vec::new() }
    }

    pub fn items(&self) -> &[ItemId] {
        &self.items
    }

    pub fn active(&self) -> Option<ItemId> {
        self.active
    }

    fn activate(&mut self, item: ItemId) -> Result<(), ModelError> {
        if !self.items.contains(&item) {
            return Err(ModelError::ItemNotFound(item));
        }
        self.active = Some(item);
        self.activation_history.retain(|entry| *entry != item);
        self.activation_history.push(item);
        Ok(())
    }

    fn insert(&mut self, item: ItemId, destination: Option<usize>) {
        if let Some(old_index) = self.items.iter().position(|candidate| *candidate == item) {
            self.items.remove(old_index);
        }
        let index = destination.unwrap_or_else(|| {
            self.active
                .and_then(|active| self.items.iter().position(|candidate| *candidate == active))
                .map_or(self.items.len(), |active| active + 1)
        });
        self.items.insert(index.min(self.items.len()), item);
        let _ = self.activate(item);
    }

    fn remove(&mut self, item: ItemId) -> Result<usize, ModelError> {
        let index = self
            .items
            .iter()
            .position(|candidate| *candidate == item)
            .ok_or(ModelError::ItemNotFound(item))?;
        self.items.remove(index);
        self.activation_history.retain(|entry| *entry != item);

        if self.active == Some(item) {
            self.active = self
                .activation_history
                .iter()
                .rev()
                .find(|candidate| self.items.contains(candidate))
                .copied()
                .or_else(|| self.items.get(index.min(self.items.len().saturating_sub(1))).copied());
        }
        Ok(index)
    }
}

/// The complete layout and ownership state of one Chartr space.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workspace {
    pub center: PaneGroup,
    panes: BTreeMap<PaneId, Pane>,
    panes_by_item: HashMap<ItemId, PaneId>,
    active_pane: PaneId,
    next_pane_id: u64,
    next_item_id: u64,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    pub fn new() -> Self {
        let root = PaneId(1);
        Self {
            center: PaneGroup::new(root),
            panes: BTreeMap::from([(root, Pane::new(root))]),
            panes_by_item: HashMap::new(),
            active_pane: root,
            next_pane_id: 2,
            next_item_id: 1,
        }
    }

    pub fn active_pane(&self) -> PaneId {
        self.active_pane
    }

    pub fn pane(&self, id: PaneId) -> Option<&Pane> {
        self.panes.get(&id)
    }

    pub fn pane_for_item(&self, item: ItemId) -> Option<PaneId> {
        self.panes_by_item.get(&item).copied()
    }

    pub fn item_ids(&self) -> impl Iterator<Item = ItemId> + '_ {
        self.panes_by_item.keys().copied()
    }

    pub fn item_count(&self) -> usize {
        self.panes_by_item.len()
    }

    pub fn is_empty(&self) -> bool {
        self.panes_by_item.is_empty()
    }

    pub fn is_grouped(&self) -> bool {
        self.item_count() > 1 || self.center.panes().len() > 1
    }

    pub fn activate_pane(&mut self, pane: PaneId) -> Result<(), ModelError> {
        if !self.center.contains(pane) {
            return Err(ModelError::PaneNotFound(pane));
        }
        self.active_pane = pane;
        Ok(())
    }

    pub fn activate_pane_in_direction(&mut self, direction: SplitDirection) -> Option<PaneId> {
        let pane = self.center.pane_in_direction(self.active_pane, direction)?;
        self.active_pane = pane;
        Some(pane)
    }

    pub fn pane_in_direction(&self, direction: SplitDirection) -> Option<PaneId> {
        self.center.pane_in_direction(self.active_pane, direction)
    }

    #[cfg(test)]
    pub fn alloc_item(&mut self) -> ItemId {
        let id = ItemId(self.next_item_id);
        self.next_item_id += 1;
        id
    }

    pub fn add_item(
        &mut self,
        item: ItemId,
        pane: Option<PaneId>,
        destination: Option<usize>,
    ) -> Result<(), ModelError> {
        if let Some(source) = self.panes_by_item.get(&item).copied() {
            if Some(source) != pane && pane.is_some() {
                self.move_item(item, pane.expect("checked"), destination)?;
                return Ok(());
            }
            self.panes
                .get_mut(&source)
                .ok_or(ModelError::PaneNotFound(source))?
                .insert(item, destination);
            self.active_pane = source;
            return Ok(());
        }

        let pane = pane.unwrap_or(self.active_pane);
        self.panes.get_mut(&pane).ok_or(ModelError::PaneNotFound(pane))?.insert(item, destination);
        self.panes_by_item.insert(item, pane);
        self.active_pane = pane;
        Ok(())
    }

    pub fn activate_item(&mut self, item: ItemId) -> Result<(), ModelError> {
        let pane = self.panes_by_item.get(&item).copied().ok_or(ModelError::ItemNotFound(item))?;
        self.panes.get_mut(&pane).ok_or(ModelError::PaneNotFound(pane))?.activate(item)?;
        self.active_pane = pane;
        Ok(())
    }

    pub fn remove_item(&mut self, item: ItemId) -> Result<(), ModelError> {
        let pane = self.panes_by_item.remove(&item).ok_or(ModelError::ItemNotFound(item))?;
        self.panes.get_mut(&pane).ok_or(ModelError::PaneNotFound(pane))?.remove(item)?;
        self.remove_pane_if_empty(pane)?;
        Ok(())
    }

    pub fn move_item(
        &mut self,
        item: ItemId,
        destination_pane: PaneId,
        destination_index: Option<usize>,
    ) -> Result<(), ModelError> {
        if !self.panes.contains_key(&destination_pane) {
            return Err(ModelError::PaneNotFound(destination_pane));
        }
        let source =
            self.panes_by_item.get(&item).copied().ok_or(ModelError::ItemNotFound(item))?;
        if source == destination_pane {
            self.panes.get_mut(&source).expect("known pane").insert(item, destination_index);
        } else {
            self.panes.get_mut(&source).expect("known pane").remove(item)?;
            self.panes
                .get_mut(&destination_pane)
                .expect("checked pane")
                .insert(item, destination_index);
            self.panes_by_item.insert(item, destination_pane);
            self.remove_pane_if_empty(source)?;
        }
        self.active_pane = destination_pane;
        Ok(())
    }

    /// Zed removes a split pane when its last item leaves. The sole root pane
    /// is retained so an empty workspace still has a drop/open target.
    pub fn remove_empty_pane(&mut self, pane: PaneId) -> Result<bool, ModelError> {
        let empty = self.panes.get(&pane).ok_or(ModelError::PaneNotFound(pane))?.items.is_empty();
        if !empty {
            return Ok(false);
        }
        self.remove_pane_if_empty(pane)
    }

    /// Normalize restored layouts from older builds that retained every empty
    /// split. Keep all panes containing items, or one active root when the
    /// whole workspace is empty.
    pub fn prune_empty_panes(&mut self) -> Result<(), ModelError> {
        let ordered = self.center.panes();
        let keep = ordered
            .iter()
            .copied()
            .find(|pane| self.panes.get(pane).is_some_and(|pane| !pane.items.is_empty()))
            .or_else(|| ordered.contains(&self.active_pane).then_some(self.active_pane))
            .or_else(|| ordered.first().copied());
        let empty: Vec<_> = ordered
            .into_iter()
            .filter(|pane| {
                Some(*pane) != keep
                    && self.panes.get(pane).is_some_and(|pane| pane.items.is_empty())
            })
            .collect();
        for pane in empty {
            self.remove_pane_if_empty(pane)?;
        }
        Ok(())
    }

    fn remove_pane_if_empty(&mut self, pane: PaneId) -> Result<bool, ModelError> {
        if !self.panes.get(&pane).ok_or(ModelError::PaneNotFound(pane))?.items.is_empty() {
            return Ok(false);
        }
        let ordered = self.center.panes();
        if ordered.len() == 1 {
            return Ok(false);
        }
        let index = ordered
            .iter()
            .position(|candidate| *candidate == pane)
            .ok_or(ModelError::PaneNotFound(pane))?;
        let focus = ordered
            .get(index + 1)
            .or_else(|| index.checked_sub(1).and_then(|i| ordered.get(i)))
            .copied();
        if self.center.remove(pane)? {
            self.panes.remove(&pane);
            if self.active_pane == pane {
                self.active_pane = focus.expect("a split pane always has a neighbor");
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub fn split_pane(
        &mut self,
        pane: PaneId,
        direction: SplitDirection,
    ) -> Result<PaneId, ModelError> {
        if !self.panes.contains_key(&pane) {
            return Err(ModelError::PaneNotFound(pane));
        }
        let new = PaneId(self.next_pane_id);
        self.next_pane_id += 1;
        self.panes.insert(new, Pane::new(new));
        self.center.split(pane, new, direction);
        self.active_pane = new;
        Ok(new)
    }

    /// Join `source` into `destination`, moving every item in order and then
    /// collapsing the recursive group.
    pub fn join_pane(&mut self, source: PaneId, destination: PaneId) -> Result<(), ModelError> {
        if source == destination {
            return Ok(());
        }
        if !self.center.contains(source) || !self.center.contains(destination) {
            return Err(ModelError::PaneNotFound(if !self.center.contains(source) {
                source
            } else {
                destination
            }));
        }
        let items = self.panes.get(&source).ok_or(ModelError::PaneNotFound(source))?.items.clone();
        for item in items {
            self.move_item(item, destination, None)?;
        }
        if self.center.contains(source) && self.center.remove(source)? {
            self.panes.remove(&source);
        }
        self.active_pane = destination;
        Ok(())
    }

    #[cfg(test)]
    pub fn validate(&self) -> Result<(), ModelError> {
        let tree_panes = self.center.panes();
        let tree_set: HashSet<_> = tree_panes.iter().copied().collect();
        let model_set: HashSet<_> = self.panes.keys().copied().collect();
        if tree_panes.len() != tree_set.len() || tree_set != model_set {
            return Err(ModelError::InvalidPaneTree);
        }
        if !tree_set.contains(&self.active_pane) {
            return Err(ModelError::PaneNotFound(self.active_pane));
        }

        let mut seen = HashSet::new();
        for (pane_id, pane) in &self.panes {
            if pane.id != *pane_id {
                return Err(ModelError::InvalidPaneTree);
            }
            if pane.active.is_some_and(|active| !pane.items.contains(&active)) {
                return Err(ModelError::InvalidActiveItem(*pane_id));
            }
            for item in &pane.items {
                if !seen.insert(*item) {
                    return Err(ModelError::DuplicateItem(*item));
                }
                if self.panes_by_item.get(item) != Some(pane_id) {
                    return Err(ModelError::ItemIndexMismatch(*item));
                }
            }
        }
        if seen.len() != self.panes_by_item.len() {
            return Err(ModelError::ItemIndexMismatch(
                self.panes_by_item
                    .keys()
                    .find(|item| !seen.contains(item))
                    .copied()
                    .unwrap_or(ItemId(0)),
            ));
        }
        Ok(())
    }
}

/// One outer tab and the Zed-style pane workspace shown when it is active.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceTab {
    pub id: WorkspaceTabId,
    pub layout: Workspace,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl WorkspaceTab {
    pub fn is_grouped(&self) -> bool {
        self.layout.is_grouped()
    }

    pub fn active_item(&self) -> Option<ItemId> {
        self.layout.pane(self.layout.active_pane()).and_then(Pane::active)
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// The item outer chrome can use to identify this tab even when its active
    /// pane is an empty Zed-style drop target.
    pub fn representative_item(&self) -> Option<ItemId> {
        self.active_item().or_else(|| {
            self.layout
                .center
                .panes()
                .into_iter()
                .find_map(|pane| self.layout.pane(pane)?.items().first().copied())
        })
    }
}

/// The outer tab collection for one Chartr space.
///
/// Zed's pane model remains intact inside each [`WorkspaceTab`]. This layer is
/// Chartr's presentation model: a one-item tab is standalone, while a tab with
/// multiple items or panes is presented as one grouped entry.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WorkspaceTabs {
    tabs: Vec<WorkspaceTab>,
    active: Option<WorkspaceTabId>,
    activation_history: Vec<WorkspaceTabId>,
    next_tab_id: u64,
    next_item_id: u64,
}

#[derive(Deserialize)]
struct WorkspaceTabsFields {
    #[serde(default)]
    tabs: Vec<WorkspaceTab>,
    #[serde(default)]
    active: Option<WorkspaceTabId>,
    #[serde(default)]
    activation_history: Vec<WorkspaceTabId>,
    #[serde(default)]
    next_tab_id: u64,
    #[serde(default)]
    next_item_id: u64,
    #[serde(default)]
    center: Option<PaneGroup>,
    #[serde(default)]
    panes: BTreeMap<PaneId, Pane>,
    #[serde(default)]
    panes_by_item: HashMap<ItemId, PaneId>,
    #[serde(default)]
    active_pane: Option<PaneId>,
    #[serde(default)]
    next_pane_id: u64,
}

impl<'de> Deserialize<'de> for WorkspaceTabs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let fields = WorkspaceTabsFields::deserialize(deserializer)?;
        let mut tabs = if let Some(center) = fields.center {
            let layout = Workspace {
                center,
                panes: fields.panes,
                panes_by_item: fields.panes_by_item,
                active_pane: fields.active_pane.unwrap_or(PaneId(1)),
                next_pane_id: fields.next_pane_id,
                next_item_id: fields.next_item_id,
            };
            if layout.is_empty() {
                Self::new()
            } else {
                let id = WorkspaceTabId(1);
                Self {
                    tabs: vec![WorkspaceTab { id, layout, name: None }],
                    active: Some(id),
                    activation_history: vec![id],
                    next_tab_id: 2,
                    next_item_id: 1,
                }
            }
        } else {
            Self {
                tabs: fields.tabs,
                active: fields.active,
                activation_history: fields.activation_history,
                next_tab_id: fields.next_tab_id,
                next_item_id: fields.next_item_id,
            }
        };
        tabs.normalize();
        Ok(tabs)
    }
}

impl Default for WorkspaceTabs {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceTabs {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active: None,
            activation_history: Vec::new(),
            next_tab_id: 1,
            next_item_id: 1,
        }
    }

    fn normalize(&mut self) {
        for tab in &mut self.tabs {
            tab.name = tab.name.take().and_then(|name| {
                let name = name.trim();
                (!name.is_empty()).then(|| name.to_owned())
            });
        }
        let known: HashSet<_> = self.tabs.iter().map(|tab| tab.id).collect();
        self.activation_history.retain(|tab| known.contains(tab));
        if self.active.is_none_or(|active| !known.contains(&active)) {
            self.active = self.tabs.last().map(|tab| tab.id);
        }
        if let Some(active) = self.active {
            self.activation_history.retain(|tab| *tab != active);
            self.activation_history.push(active);
        }
        self.next_tab_id =
            self.next_tab_id.max(self.tabs.iter().map(|tab| tab.id.0 + 1).max().unwrap_or(1));
        self.next_item_id =
            self.next_item_id.max(self.item_ids().map(|item| item.0 + 1).max().unwrap_or(1));
    }

    pub fn tabs(&self) -> &[WorkspaceTab] {
        &self.tabs
    }

    pub fn active_tab_id(&self) -> Option<WorkspaceTabId> {
        self.active
    }

    pub fn active_tab(&self) -> Option<&WorkspaceTab> {
        self.active.and_then(|active| self.tab(active))
    }

    pub fn active_workspace(&self) -> Option<&Workspace> {
        self.active_tab().map(|tab| &tab.layout)
    }

    pub fn active_workspace_mut(&mut self) -> Option<&mut Workspace> {
        let active = self.active?;
        self.tab_mut(active).map(|tab| &mut tab.layout)
    }

    pub fn tab(&self, id: WorkspaceTabId) -> Option<&WorkspaceTab> {
        self.tabs.iter().find(|tab| tab.id == id)
    }

    pub fn tab_mut(&mut self, id: WorkspaceTabId) -> Option<&mut WorkspaceTab> {
        self.tabs.iter_mut().find(|tab| tab.id == id)
    }

    pub fn workspace(&self, id: WorkspaceTabId) -> Option<&Workspace> {
        self.tab(id).map(|tab| &tab.layout)
    }

    pub fn workspace_mut(&mut self, id: WorkspaceTabId) -> Option<&mut Workspace> {
        self.tab_mut(id).map(|tab| &mut tab.layout)
    }

    pub fn active_item(&self) -> Option<ItemId> {
        self.active_tab().and_then(WorkspaceTab::active_item)
    }

    pub fn alloc_item(&mut self) -> ItemId {
        let id = ItemId(self.next_item_id);
        self.next_item_id += 1;
        id
    }

    pub fn item_ids(&self) -> impl Iterator<Item = ItemId> + '_ {
        self.tabs.iter().flat_map(|tab| tab.layout.item_ids())
    }

    pub fn location(&self, item: ItemId) -> Option<(WorkspaceTabId, PaneId)> {
        self.tabs.iter().find_map(|tab| tab.layout.pane_for_item(item).map(|pane| (tab.id, pane)))
    }

    pub fn activate_tab(&mut self, id: WorkspaceTabId) -> Result<(), ModelError> {
        if self.tab(id).is_none() {
            return Err(ModelError::WorkspaceTabNotFound(id));
        }
        self.active = Some(id);
        self.activation_history.retain(|tab| *tab != id);
        self.activation_history.push(id);
        Ok(())
    }

    pub fn activate_item(&mut self, item: ItemId) -> Result<(), ModelError> {
        let (tab, _) = self.location(item).ok_or(ModelError::ItemNotFound(item))?;
        self.workspace_mut(tab).expect("known workspace tab").activate_item(item)?;
        self.activate_tab(tab)
    }

    pub fn push_standalone(&mut self, item: ItemId) -> Result<WorkspaceTabId, ModelError> {
        self.push_standalone_at(item, self.tabs.len())
    }

    pub fn push_standalone_at(
        &mut self,
        item: ItemId,
        index: usize,
    ) -> Result<WorkspaceTabId, ModelError> {
        if self.location(item).is_some() {
            return Err(ModelError::DuplicateItem(item));
        }
        let id = WorkspaceTabId(self.next_tab_id);
        self.next_tab_id += 1;
        let mut layout = Workspace::new();
        layout.add_item(item, None, None)?;
        self.tabs.insert(index.min(self.tabs.len()), WorkspaceTab { id, layout, name: None });
        self.activate_tab(id)?;
        Ok(id)
    }

    pub fn rename_tab(
        &mut self,
        id: WorkspaceTabId,
        name: Option<String>,
    ) -> Result<(), ModelError> {
        let tab = self.tab_mut(id).ok_or(ModelError::WorkspaceTabNotFound(id))?;
        tab.name = name.and_then(|name| {
            let name = name.trim();
            (!name.is_empty()).then(|| name.to_owned())
        });
        Ok(())
    }

    pub fn move_tab(&mut self, tab: WorkspaceTabId, destination: usize) -> Result<(), ModelError> {
        let source = self
            .tabs
            .iter()
            .position(|candidate| candidate.id == tab)
            .ok_or(ModelError::WorkspaceTabNotFound(tab))?;
        let tab = self.tabs.remove(source);
        self.tabs.insert(destination.min(self.tabs.len()), tab);
        Ok(())
    }

    /// Expand one grouped workspace back into standalone outer tabs.
    ///
    /// Items follow pane-tree order and retain their order within each pane,
    /// so ungrouping is deterministic even for recursively nested splits.
    pub fn ungroup_tab(&mut self, id: WorkspaceTabId) -> Result<(), ModelError> {
        let index = self
            .tabs
            .iter()
            .position(|candidate| candidate.id == id)
            .ok_or(ModelError::WorkspaceTabNotFound(id))?;
        let tab = &self.tabs[index];
        if !tab.is_grouped() {
            return Ok(());
        }

        let items: Vec<_> = tab
            .layout
            .center
            .panes()
            .into_iter()
            .flat_map(|pane| {
                tab.layout.pane(pane).into_iter().flat_map(|pane| pane.items().iter().copied())
            })
            .collect();
        if items.is_empty() {
            return Ok(());
        }

        let was_active = self.active == Some(id);
        let previously_active = self.active;
        let representative = tab.representative_item();
        self.tabs.remove(index);
        self.activation_history.retain(|candidate| *candidate != id);
        if was_active {
            self.active = None;
        }

        let mut representative_tab = None;
        for (offset, item) in items.into_iter().enumerate() {
            let standalone = self.push_standalone_at(item, index + offset)?;
            if Some(item) == representative {
                representative_tab = Some(standalone);
            }
        }

        if was_active {
            if let Some(active) = representative_tab {
                self.activate_tab(active)?;
            }
        } else if let Some(active) = previously_active {
            self.activate_tab(active)?;
        }
        Ok(())
    }

    pub fn remove_item(&mut self, item: ItemId) -> Result<(), ModelError> {
        let (tab, _) = self.location(item).ok_or(ModelError::ItemNotFound(item))?;
        self.workspace_mut(tab).expect("known workspace tab").remove_item(item)?;
        self.remove_tab_if_empty(tab);
        Ok(())
    }

    pub fn move_item(
        &mut self,
        item: ItemId,
        source_tab: WorkspaceTabId,
        source_pane: PaneId,
        target_tab: WorkspaceTabId,
        target_pane: PaneId,
        destination_index: Option<usize>,
    ) -> Result<(), ModelError> {
        if self.location(item) != Some((source_tab, source_pane)) {
            return Err(ModelError::ItemNotFound(item));
        }
        if self.workspace(target_tab).and_then(|layout| layout.pane(target_pane)).is_none() {
            return Err(ModelError::PaneNotFound(target_pane));
        }
        if source_tab == target_tab {
            self.workspace_mut(target_tab).expect("known workspace tab").move_item(
                item,
                target_pane,
                destination_index,
            )?;
        } else {
            self.workspace_mut(source_tab)
                .expect("known source workspace tab")
                .remove_item(item)?;
            self.workspace_mut(target_tab).expect("known target workspace tab").add_item(
                item,
                Some(target_pane),
                destination_index,
            )?;
            self.remove_tab_if_empty(source_tab);
        }
        self.activate_tab(target_tab)?;
        Ok(())
    }

    pub fn prune_empty(&mut self) -> Result<(), ModelError> {
        for tab in &mut self.tabs {
            tab.layout.prune_empty_panes()?;
        }
        let empty: Vec<_> =
            self.tabs.iter().filter(|tab| tab.layout.is_empty()).map(|tab| tab.id).collect();
        for tab in empty {
            self.remove_tab_if_empty(tab);
        }
        Ok(())
    }

    fn remove_tab_if_empty(&mut self, id: WorkspaceTabId) {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == id && tab.layout.is_empty())
        else {
            return;
        };
        self.tabs.remove(index);
        self.activation_history.retain(|tab| *tab != id);
        if self.active == Some(id) {
            self.active = self
                .activation_history
                .iter()
                .rev()
                .find(|candidate| self.tabs.iter().any(|tab| tab.id == **candidate))
                .copied()
                .or_else(|| {
                    self.tabs.get(index.min(self.tabs.len().saturating_sub(1))).map(|tab| tab.id)
                });
            if let Some(active) = self.active {
                self.activation_history.retain(|tab| *tab != active);
                self.activation_history.push(active);
            }
        }
    }

    #[cfg(test)]
    pub fn validate(&self) -> Result<(), ModelError> {
        let mut tabs = HashSet::new();
        let mut items = HashSet::new();
        for tab in &self.tabs {
            if !tabs.insert(tab.id) || tab.layout.is_empty() {
                return Err(ModelError::InvalidWorkspaceTabs);
            }
            tab.layout.validate()?;
            for item in tab.layout.item_ids() {
                if !items.insert(item) {
                    return Err(ModelError::DuplicateItem(item));
                }
            }
        }
        if self.active.is_some_and(|active| !tabs.contains(&active)) {
            return Err(ModelError::WorkspaceTabNotFound(self.active.expect("checked")));
        }
        if self.tabs.is_empty() != self.active.is_none() {
            return Err(ModelError::InvalidWorkspaceTabs);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    PaneNotFound(PaneId),
    WorkspaceTabNotFound(WorkspaceTabId),
    ItemNotFound(ItemId),
    DuplicateItem(ItemId),
    #[cfg(test)]
    ItemIndexMismatch(ItemId),
    #[cfg(test)]
    InvalidActiveItem(PaneId),
    #[cfg(test)]
    InvalidPaneTree,
    #[cfg(test)]
    InvalidWorkspaceTabs,
    BadAxisPath,
    InvalidFlexes,
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ModelError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_tabs_mix_standalone_items_with_one_pane_group() {
        let mut tabs = WorkspaceTabs::new();
        let items: Vec<_> = (0..5).map(|_| tabs.alloc_item()).collect();
        let outer: Vec<_> = items.iter().map(|item| tabs.push_standalone(*item).unwrap()).collect();
        let target_pane = tabs.workspace(outer[2]).unwrap().active_pane();
        let right = tabs
            .workspace_mut(outer[2])
            .unwrap()
            .split_pane(target_pane, SplitDirection::Right)
            .unwrap();

        tabs.move_item(items[3], outer[3], PaneId(1), outer[2], right, None).unwrap();
        tabs.move_item(items[4], outer[4], PaneId(1), outer[2], right, None).unwrap();

        assert_eq!(tabs.tabs().iter().map(|tab| tab.id).collect::<Vec<_>>(), outer[..3]);
        assert!(!tabs.tab(outer[0]).unwrap().is_grouped());
        assert!(!tabs.tab(outer[1]).unwrap().is_grouped());
        assert!(tabs.tab(outer[2]).unwrap().is_grouped());
        assert_eq!(tabs.workspace(outer[2]).unwrap().pane(right).unwrap().items(), &items[3..]);
        assert_eq!(tabs.active_tab_id(), Some(outer[2]));
        tabs.validate().unwrap();
        let restored: WorkspaceTabs =
            serde_json::from_str(&serde_json::to_string(&tabs).unwrap()).unwrap();
        assert_eq!(restored, tabs);
        restored.validate().unwrap();
    }

    #[test]
    fn ungrouping_restores_items_as_outer_tabs_in_pane_and_tab_order() {
        let mut tabs = WorkspaceTabs::new();
        let items: Vec<_> = (0..5).map(|_| tabs.alloc_item()).collect();
        let outer: Vec<_> = items.iter().map(|item| tabs.push_standalone(*item).unwrap()).collect();
        let left = tabs.workspace(outer[1]).unwrap().active_pane();
        let right =
            tabs.workspace_mut(outer[1]).unwrap().split_pane(left, SplitDirection::Right).unwrap();

        tabs.move_item(items[2], outer[2], PaneId(1), outer[1], left, None).unwrap();
        tabs.move_item(items[3], outer[3], PaneId(1), outer[1], right, None).unwrap();
        tabs.ungroup_tab(outer[1]).unwrap();

        assert_eq!(
            tabs.tabs().iter().filter_map(WorkspaceTab::representative_item).collect::<Vec<_>>(),
            items,
        );
        assert!(tabs.tabs().iter().all(|tab| !tab.is_grouped()));
        assert_eq!(tabs.active_item(), Some(items[3]));
        tabs.validate().unwrap();
    }

    #[test]
    fn ungrouping_an_inactive_group_preserves_the_active_outer_tab() {
        let mut tabs = WorkspaceTabs::new();
        let items: Vec<_> = (0..3).map(|_| tabs.alloc_item()).collect();
        let outer: Vec<_> = items.iter().map(|item| tabs.push_standalone(*item).unwrap()).collect();
        let target_pane = tabs.workspace(outer[0]).unwrap().active_pane();
        tabs.move_item(items[1], outer[1], PaneId(1), outer[0], target_pane, None).unwrap();
        tabs.activate_tab(outer[2]).unwrap();

        tabs.ungroup_tab(outer[0]).unwrap();

        assert_eq!(tabs.active_tab_id(), Some(outer[2]));
        assert_eq!(tabs.active_item(), Some(items[2]));
        assert!(tabs.tabs().iter().all(|tab| !tab.is_grouped()));
        tabs.validate().unwrap();
    }

    #[test]
    fn ungrouping_a_single_item_with_an_empty_split_makes_it_standalone() {
        let mut tabs = WorkspaceTabs::new();
        let item = tabs.alloc_item();
        let grouped = tabs.push_standalone(item).unwrap();
        let occupied = tabs.workspace(grouped).unwrap().active_pane();
        tabs.workspace_mut(grouped).unwrap().split_pane(occupied, SplitDirection::Right).unwrap();

        tabs.ungroup_tab(grouped).unwrap();

        assert_eq!(tabs.tabs().len(), 1);
        assert_eq!(tabs.active_item(), Some(item));
        assert!(!tabs.tabs()[0].is_grouped());
        tabs.validate().unwrap();
    }

    #[test]
    fn standalone_outer_tabs_drop_into_a_group_center_or_any_edge() {
        for direction in [
            None,
            Some(SplitDirection::Up),
            Some(SplitDirection::Right),
            Some(SplitDirection::Down),
            Some(SplitDirection::Left),
        ] {
            let mut tabs = WorkspaceTabs::new();
            let target_item = tabs.alloc_item();
            let source_item = tabs.alloc_item();
            let target_tab = tabs.push_standalone(target_item).unwrap();
            let source_tab = tabs.push_standalone(source_item).unwrap();
            let target_pane = tabs.location(target_item).unwrap().1;
            let source_pane = tabs.location(source_item).unwrap().1;
            let destination = direction.map_or(target_pane, |direction| {
                tabs.workspace_mut(target_tab).unwrap().split_pane(target_pane, direction).unwrap()
            });

            tabs.move_item(source_item, source_tab, source_pane, target_tab, destination, None)
                .unwrap();

            assert_eq!(tabs.tabs().len(), 1, "{direction:?}");
            assert_eq!(tabs.active_tab_id(), Some(target_tab), "{direction:?}");
            assert!(tabs.tab(target_tab).unwrap().is_grouped(), "{direction:?}");
            assert_eq!(tabs.workspace(target_tab).unwrap().item_count(), 2, "{direction:?}");
            assert_eq!(
                tabs.workspace(target_tab).unwrap().center.panes().len(),
                direction.map_or(1, |_| 2),
                "{direction:?}",
            );
            tabs.validate().unwrap();
        }
    }

    #[test]
    fn workspace_tabs_round_trip_and_continue_allocating_unique_ids() {
        let mut tabs = WorkspaceTabs::new();
        let first = tabs.alloc_item();
        let second = tabs.alloc_item();
        tabs.push_standalone(first).unwrap();
        tabs.push_standalone(second).unwrap();
        let json = serde_json::to_string(&tabs).unwrap();
        let mut restored: WorkspaceTabs =
            serde_json::from_str(&json).unwrap_or_else(|error| panic!("{error}: {json}"));

        assert_eq!(restored, tabs);
        assert_eq!(restored.alloc_item(), ItemId(3));
        restored.validate().unwrap();
    }

    #[test]
    fn workspace_tab_names_round_trip_and_empty_names_restore_the_default() {
        let mut tabs = WorkspaceTabs::new();
        let first = tabs.alloc_item();
        let second = tabs.alloc_item();
        let group = tabs.push_standalone(first).unwrap();
        let source = tabs.push_standalone(second).unwrap();
        let target_pane = tabs.workspace(group).unwrap().active_pane();
        tabs.move_item(second, source, PaneId(1), group, target_pane, None).unwrap();

        tabs.rename_tab(group, Some("  Build Logs  ".to_owned())).unwrap();

        assert_eq!(tabs.tab(group).unwrap().name(), Some("Build Logs"));
        let json = serde_json::to_string(&tabs).unwrap();
        let mut restored: WorkspaceTabs = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, tabs);
        assert_eq!(restored.tab(group).unwrap().name(), Some("Build Logs"));

        restored.rename_tab(group, Some("   ".to_owned())).unwrap();

        assert_eq!(restored.tab(group).unwrap().name(), None);
        assert!(!serde_json::to_string(&restored).unwrap().contains("\"name\""));
        restored.validate().unwrap();
    }

    #[test]
    fn legacy_single_workspace_state_becomes_one_outer_workspace_tab() {
        let mut legacy = Workspace::new();
        let item = legacy.alloc_item();
        legacy.add_item(item, None, None).unwrap();
        let json = serde_json::to_string(&legacy).unwrap();

        let restored: WorkspaceTabs = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.tabs().len(), 1);
        assert_eq!(restored.active_item(), Some(item));
        restored.validate().unwrap();
    }

    #[test]
    fn moving_the_last_item_removes_its_empty_source_pane_like_zed() {
        let mut workspace = Workspace::new();
        let left = workspace.active_pane();
        let right = workspace.split_pane(left, SplitDirection::Right).unwrap();
        let item = workspace.alloc_item();

        workspace.add_item(item, Some(left), None).unwrap();
        workspace.add_item(item, Some(right), None).unwrap();
        workspace.add_item(item, Some(right), Some(0)).unwrap();

        assert_eq!(workspace.pane_for_item(item), Some(right));
        assert!(workspace.pane(left).is_none());
        assert_eq!(workspace.center.panes(), vec![right]);
        assert_eq!(workspace.pane(right).unwrap().items(), &[item]);
        workspace.validate().unwrap();
    }

    #[test]
    fn closing_the_last_item_removes_a_split_but_retains_the_root() {
        let mut workspace = Workspace::new();
        let root = workspace.active_pane();
        let split = workspace.split_pane(root, SplitDirection::Right).unwrap();
        let root_item = workspace.alloc_item();
        let split_item = workspace.alloc_item();
        workspace.add_item(root_item, Some(root), None).unwrap();
        workspace.add_item(split_item, Some(split), None).unwrap();

        workspace.remove_item(split_item).unwrap();

        assert_eq!(workspace.center.panes(), vec![root]);
        assert!(workspace.pane(split).is_none());
        assert_eq!(workspace.active_pane(), root);
        workspace.remove_item(root_item).unwrap();
        assert_eq!(workspace.center.panes(), vec![root]);
        assert!(workspace.pane(root).unwrap().items().is_empty());
        workspace.validate().unwrap();
    }

    #[test]
    fn restored_empty_splits_are_pruned_to_the_only_useful_pane() {
        let mut workspace = Workspace::new();
        let root = workspace.active_pane();
        let useful = workspace.split_pane(root, SplitDirection::Right).unwrap();
        let empty = workspace.split_pane(useful, SplitDirection::Down).unwrap();
        let item = workspace.alloc_item();
        workspace.add_item(item, Some(useful), None).unwrap();
        workspace.activate_pane(empty).unwrap();

        workspace.prune_empty_panes().unwrap();

        assert_eq!(workspace.center.panes(), vec![useful]);
        assert_eq!(workspace.active_pane(), useful);
        assert_eq!(workspace.pane(useful).unwrap().items(), &[item]);
        workspace.validate().unwrap();
    }

    #[test]
    fn an_empty_active_pane_does_not_hide_its_outer_group() {
        let mut tabs = WorkspaceTabs::new();
        let item = tabs.alloc_item();
        let tab = tabs.push_standalone(item).unwrap();
        let occupied = tabs.location(item).unwrap().1;
        let empty =
            tabs.workspace_mut(tab).unwrap().split_pane(occupied, SplitDirection::Right).unwrap();

        let grouped = tabs.tab(tab).unwrap();
        assert_eq!(grouped.layout.active_pane(), empty);
        assert_eq!(grouped.active_item(), None);
        assert_eq!(grouped.representative_item(), Some(item));
        tabs.validate().unwrap();
    }

    #[test]
    fn closing_an_empty_active_pane_focuses_its_neighbor() {
        let mut workspace = Workspace::new();
        let occupied = workspace.active_pane();
        let item = workspace.alloc_item();
        workspace.add_item(item, Some(occupied), None).unwrap();
        let empty = workspace.split_pane(occupied, SplitDirection::Right).unwrap();

        assert!(workspace.remove_empty_pane(empty).unwrap());
        assert_eq!(workspace.active_pane(), occupied);
        assert_eq!(workspace.center.panes(), vec![occupied]);
        workspace.validate().unwrap();
    }

    #[test]
    fn same_axis_splits_extend_the_axis_and_cross_axis_splits_nest() {
        let mut workspace = Workspace::new();
        let root = workspace.active_pane();
        let right = workspace.split_pane(root, SplitDirection::Right).unwrap();
        let far_right = workspace.split_pane(right, SplitDirection::Right).unwrap();
        let below = workspace.split_pane(right, SplitDirection::Down).unwrap();

        assert_eq!(workspace.center.panes(), vec![root, right, below, far_right]);
        let Member::Axis(horizontal) = &workspace.center.root else {
            panic!("horizontal root");
        };
        assert_eq!(horizontal.axis, Axis::Horizontal);
        assert_eq!(horizontal.members.len(), 3);
        assert!(matches!(
            horizontal.members[1],
            Member::Axis(PaneAxis { axis: Axis::Vertical, .. })
        ));
        workspace.validate().unwrap();
    }

    #[test]
    fn joining_moves_items_and_collapses_the_axis() {
        let mut workspace = Workspace::new();
        let left = workspace.active_pane();
        let right = workspace.split_pane(left, SplitDirection::Right).unwrap();
        let first = workspace.alloc_item();
        let second = workspace.alloc_item();
        workspace.add_item(first, Some(left), None).unwrap();
        workspace.add_item(second, Some(right), None).unwrap();

        workspace.join_pane(right, left).unwrap();

        assert_eq!(workspace.center.panes(), vec![left]);
        assert_eq!(workspace.pane(left).unwrap().items(), &[first, second]);
        assert_eq!(workspace.pane_for_item(second), Some(left));
        workspace.validate().unwrap();
    }

    #[test]
    fn closing_uses_activation_history_before_position() {
        let mut workspace = Workspace::new();
        let pane = workspace.active_pane();
        let a = workspace.alloc_item();
        let b = workspace.alloc_item();
        let c = workspace.alloc_item();
        workspace.add_item(a, Some(pane), None).unwrap();
        workspace.add_item(b, Some(pane), None).unwrap();
        workspace.add_item(c, Some(pane), None).unwrap();
        workspace.activate_item(a).unwrap();
        workspace.activate_item(c).unwrap();

        workspace.remove_item(c).unwrap();

        assert_eq!(workspace.pane(pane).unwrap().active(), Some(a));
        workspace.validate().unwrap();
    }

    #[test]
    fn the_root_pane_cannot_be_removed() {
        let mut group = PaneGroup::new(PaneId(1));
        assert!(!group.remove(PaneId(1)).unwrap());
        assert_eq!(group.panes(), vec![PaneId(1)]);
    }

    #[test]
    fn persisted_layout_round_trips_with_ownership_intact() {
        let mut workspace = Workspace::new();
        let root = workspace.active_pane();
        let down = workspace.split_pane(root, SplitDirection::Down).unwrap();
        let item = workspace.alloc_item();
        workspace.add_item(item, Some(down), None).unwrap();
        workspace.center.set_flexes(&[], vec![1.5, 0.5]).unwrap();

        let encoded = serde_json::to_string(&workspace).unwrap();
        let restored: Workspace = serde_json::from_str(&encoded).unwrap();

        assert_eq!(restored, workspace);
        restored.validate().unwrap();
    }

    #[test]
    fn invalid_flexes_do_not_replace_a_working_layout() {
        let mut workspace = Workspace::new();
        let root = workspace.active_pane();
        workspace.split_pane(root, SplitDirection::Right).unwrap();
        assert_eq!(workspace.center.set_flexes(&[], vec![0., 2.]), Err(ModelError::InvalidFlexes));
        let Member::Axis(axis) = &workspace.center.root else {
            panic!("split root");
        };
        assert_eq!(axis.flexes, vec![1., 1.]);
    }

    #[test]
    fn divider_resizing_changes_only_the_adjacent_pair() {
        let mut workspace = Workspace::new();
        let first = workspace.active_pane();
        let second = workspace.split_pane(first, SplitDirection::Right).unwrap();
        workspace.split_pane(second, SplitDirection::Right).unwrap();
        workspace.center.set_flexes(&[], vec![1., 1., 2.]).unwrap();

        workspace.center.resize_divider(&[], 0, 0.4).unwrap();

        let Member::Axis(axis) = &workspace.center.root else {
            panic!("axis");
        };
        assert_eq!(axis.flexes[2], 2.);
        assert!((axis.flexes[0] - 1.6).abs() < 0.001);
        assert!((axis.flexes[1] - 0.4).abs() < 0.001);
    }

    #[test]
    fn directional_focus_uses_the_center_of_nested_panes_like_zed() {
        let mut workspace = Workspace::new();
        let left = workspace.active_pane();
        let top_right = workspace.split_pane(left, SplitDirection::Right).unwrap();
        let bottom_right = workspace.split_pane(top_right, SplitDirection::Down).unwrap();

        workspace.activate_pane(bottom_right).unwrap();
        assert_eq!(workspace.activate_pane_in_direction(SplitDirection::Left), Some(left));
        assert_eq!(workspace.activate_pane_in_direction(SplitDirection::Right), Some(top_right));
        assert_eq!(workspace.activate_pane_in_direction(SplitDirection::Down), Some(bottom_right));
        assert_eq!(workspace.activate_pane_in_direction(SplitDirection::Down), None);
    }

    #[test]
    fn direction_at_an_outer_edge_has_no_neighbor() {
        let mut workspace = Workspace::new();
        let left = workspace.active_pane();
        workspace.split_pane(left, SplitDirection::Right).unwrap();
        workspace.activate_pane(left).unwrap();

        assert_eq!(workspace.pane_in_direction(SplitDirection::Left), None);
        assert_eq!(workspace.pane_in_direction(SplitDirection::Up), None);
    }

    #[test]
    fn tab_drop_indices_match_zeds_before_and_after_target_semantics() {
        let mut workspace = Workspace::new();
        let pane = workspace.active_pane();
        let a = workspace.alloc_item();
        let b = workspace.alloc_item();
        let c = workspace.alloc_item();
        let d = workspace.alloc_item();
        for item in [a, b, c, d] {
            workspace.add_item(item, Some(pane), None).unwrap();
        }

        // A dragged onto C lands after C because it approached from the left.
        workspace.move_item(a, pane, Some(2)).unwrap();
        assert_eq!(workspace.pane(pane).unwrap().items(), &[b, c, a, d]);

        // D dragged onto C lands before C because it approached from the right.
        workspace.move_item(d, pane, Some(1)).unwrap();
        assert_eq!(workspace.pane(pane).unwrap().items(), &[b, d, c, a]);
        assert_eq!(workspace.pane(pane).unwrap().active(), Some(d));
        workspace.validate().unwrap();
    }

    #[test]
    fn edge_drop_split_inserts_beside_the_target_and_preserves_other_panes() {
        let mut workspace = Workspace::new();
        let left = workspace.active_pane();
        let right = workspace.split_pane(left, SplitDirection::Right).unwrap();
        let left_a = workspace.alloc_item();
        let left_b = workspace.alloc_item();
        let right_a = workspace.alloc_item();
        workspace.add_item(left_a, Some(left), None).unwrap();
        workspace.add_item(left_b, Some(left), None).unwrap();
        workspace.add_item(right_a, Some(right), None).unwrap();

        let dropped = workspace.split_pane(right, SplitDirection::Left).unwrap();
        workspace.move_item(left_a, dropped, Some(0)).unwrap();

        assert_eq!(workspace.center.panes(), vec![left, dropped, right]);
        assert_eq!(workspace.pane(left).unwrap().items(), &[left_b]);
        assert_eq!(workspace.pane(dropped).unwrap().items(), &[left_a]);
        assert_eq!(workspace.pane(right).unwrap().items(), &[right_a]);
        assert_eq!(workspace.active_pane(), dropped);
        workspace.validate().unwrap();
    }

    #[test]
    fn center_drop_of_a_last_tab_collapses_only_its_empty_source() {
        let mut workspace = Workspace::new();
        let left = workspace.active_pane();
        let right = workspace.split_pane(left, SplitDirection::Right).unwrap();
        let left_item = workspace.alloc_item();
        let right_item = workspace.alloc_item();
        workspace.add_item(left_item, Some(left), None).unwrap();
        workspace.add_item(right_item, Some(right), None).unwrap();

        workspace.move_item(left_item, right, Some(0)).unwrap();

        assert_eq!(workspace.center.panes(), vec![right]);
        assert!(workspace.pane(left).is_none());
        assert_eq!(workspace.pane(right).unwrap().items(), &[left_item, right_item]);
        workspace.validate().unwrap();
    }

    #[test]
    fn a_long_edit_sequence_preserves_tree_and_item_ownership() {
        let mut workspace = Workspace::new();
        let root = workspace.active_pane();
        let right = workspace.split_pane(root, SplitDirection::Right).unwrap();
        let down = workspace.split_pane(right, SplitDirection::Down).unwrap();
        let items: Vec<_> = (0..12).map(|_| workspace.alloc_item()).collect();
        for (index, item) in items.iter().copied().enumerate() {
            let pane = [root, right, down][index % 3];
            workspace.add_item(item, Some(pane), None).unwrap();
            workspace.validate().unwrap();
        }
        for (index, item) in items.iter().copied().enumerate() {
            let pane = [down, root, right][index % 3];
            workspace.move_item(item, pane, Some(0)).unwrap();
            workspace.validate().unwrap();
        }
        workspace.center.resize_divider(&[], 0, 0.6).unwrap();
        workspace.join_pane(down, right).unwrap();
        for item in items.iter().step_by(2) {
            workspace.remove_item(*item).unwrap();
            workspace.validate().unwrap();
        }
        let unique: std::collections::HashSet<_> = workspace.item_ids().collect();
        assert_eq!(unique.len(), items.len() / 2);
        let restored: Workspace =
            serde_json::from_str(&serde_json::to_string(&workspace).unwrap()).unwrap();
        restored.validate().unwrap();
    }
}
