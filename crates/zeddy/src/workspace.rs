//! Chartr's workspace, pane, and item ownership model.
//!
//! The names and responsibilities follow Zed's `Workspace`, `PaneGroup`, and
//! `Pane`, but this module contains only Chartr's product-neutral state. GPUI
//! entities and rendering live above it. Keeping the mutations here makes the
//! invariant observable at one seam: an item belongs to exactly one pane in
//! exactly one workspace.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};

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

    pub fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
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
    pub maximized: Option<PaneId>,
}

impl PaneGroup {
    pub fn new(root: PaneId) -> Self {
        Self { root: Member::pane(root), maximized: None }
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
                if self.maximized == Some(pane) {
                    self.maximized = None;
                }
                Ok(true)
            }
        }
    }

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

    pub fn toggle_maximized(&mut self, pane: PaneId) -> Result<(), ModelError> {
        if !self.contains(pane) {
            return Err(ModelError::PaneNotFound(pane));
        }
        self.maximized = (self.maximized != Some(pane)).then_some(pane);
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

    pub fn panes(&self) -> impl Iterator<Item = &Pane> {
        self.center.panes().into_iter().filter_map(|id| self.panes.get(&id))
    }

    pub fn pane_for_item(&self, item: ItemId) -> Option<PaneId> {
        self.panes_by_item.get(&item).copied()
    }

    pub fn item_ids(&self) -> impl Iterator<Item = ItemId> + '_ {
        self.panes_by_item.keys().copied()
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
        }
        self.active_pane = destination_pane;
        Ok(())
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
        if self.center.remove(source)? {
            self.panes.remove(&source);
        }
        self.active_pane = destination;
        Ok(())
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    PaneNotFound(PaneId),
    ItemNotFound(ItemId),
    DuplicateItem(ItemId),
    ItemIndexMismatch(ItemId),
    InvalidActiveItem(PaneId),
    InvalidPaneTree,
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
    fn an_item_has_one_owner_even_when_moved_and_readded() {
        let mut workspace = Workspace::new();
        let left = workspace.active_pane();
        let right = workspace.split_pane(left, SplitDirection::Right).unwrap();
        let item = workspace.alloc_item();

        workspace.add_item(item, Some(left), None).unwrap();
        workspace.add_item(item, Some(right), None).unwrap();
        workspace.add_item(item, Some(right), Some(0)).unwrap();

        assert_eq!(workspace.pane_for_item(item), Some(right));
        assert!(!workspace.pane(left).unwrap().items().contains(&item));
        assert_eq!(workspace.pane(right).unwrap().items(), &[item]);
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
        workspace.center.toggle_maximized(down).unwrap();

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
