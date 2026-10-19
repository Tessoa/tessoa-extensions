//! The rows of a tree node, worked out by the SDK from the items a page
//! gives: which items there are, under which, in what order, which are open.

use std::cell::{Ref, RefCell};
use std::collections::{HashMap, HashSet};

use crate::pages::{Expander, Row};

fn missing_item(method: &str, id: &str) {
    let message = format!("TreeModel.{method}: no item {id:?}; set roots or children before using this id");
    #[cfg(target_arch = "wasm32")]
    crate::log::warn(&message);
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("{message}");
}

/// The rows of a [`crate::pages::tree`] node, worked out from items.
///
/// Which rows a tree shows depends on which of its items are open, and
/// tessoa asks for rows by number. A `TreeModel` keeps the items, each with
/// an id unique in the whole tree and a value of the page's choosing,
/// remembers which are open, and answers by number with the item's depth
/// and expander filled in:
///
/// ```ignore
/// struct Outline {
///     outline: TreeModel<String>,
/// }
///
/// impl Page for Outline {
///     fn open(_page: PageId, _state: Option<Vec<u8>>, _args: Option<Vec<u8>>) -> Self {
///         let mut outline = TreeModel::new();
///         outline.set_roots([("intro", "Introduction".to_string()), ("usage", "Usage".to_string())]);
///         outline.set_children("usage", [("usage/install", "Install".to_string())]);
///         Outline { outline }
///     }
///
///     fn event(&mut self, event: Event) -> Result<(), String> {
///         if let Event::Expand { row, open, .. } = event {
///             self.outline.expand(&row, open);
///         }
///         Ok(())
///     }
///
///     fn render(&self) -> Node {
///         tree(self.outline.count()).key("outline")
///     }
///
///     fn rows(&self, _list: &str, start: usize, count: usize) -> Vec<Row> {
///         self.outline.rows(start, count, |id, title| Row::new(id, [title.as_str()]))
///     }
/// }
/// ```
///
/// An item with children has an expander; one without has none, unless
/// [`TreeModel::set_lazy`] says its children come later. Opening such an
/// item makes [`TreeModel::expand`] return true: the page then sets the
/// children, at once or when a task has found them, perhaps with a row such
/// as "Loading…" meanwhile, which is a child like any other.
#[derive(Debug, Clone)]
pub struct TreeModel<T> {
    /// The items by slot; a removed item leaves an empty slot for the next.
    slots: Vec<Option<Item<T>>>,
    free: Vec<usize>,
    ids: HashMap<String, usize>,
    roots: Vec<usize>,
    /// The rows, worked out when they are asked for after a change.
    rows: RefCell<Option<Vec<Shown>>>,
}

#[derive(Debug, Clone)]
struct Item<T> {
    id: String,
    value: T,
    parent: Option<usize>,
    /// `None` while the children are still to come.
    children: Option<Vec<usize>>,
    open: bool,
}

#[derive(Debug, Clone, Copy)]
struct Shown {
    slot: usize,
    depth: u8,
    expander: Expander,
}

/// One row of a [`TreeModel`]: the item it shows, and how.
#[derive(Debug)]
pub struct TreeRow<'a, T> {
    pub id: &'a str,
    pub value: &'a T,
    /// The level, 0 at the top.
    pub depth: u8,
    pub expander: Expander,
}

// By hand: a derive would ask for `T: Copy`, which a reference does not need.
impl<T> Clone for TreeRow<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for TreeRow<'_, T> {}

impl<T> Default for TreeModel<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TreeModel<T> {
    /// A tree without items.
    pub fn new() -> Self {
        TreeModel {
            slots: Vec::new(),
            free: Vec::new(),
            ids: HashMap::new(),
            roots: Vec::new(),
            rows: RefCell::new(None),
        }
    }

    /// Replace the items at the top, in order, with ids and values. An item
    /// that was in the tree before, matched by id, keeps what is below it
    /// and stays open or closed, wherever it was; the ones left out go, with
    /// everything below them. An id given twice counts once.
    pub fn set_roots<S: Into<String>>(&mut self, items: impl IntoIterator<Item = (S, T)>) {
        let placed = self.place(None, items);
        let old = std::mem::replace(&mut self.roots, placed.clone());
        self.drop_left_out(old, &placed);
        self.changed();
    }

    /// Replace the children of the item `parent`, in order, the way
    /// [`TreeModel::set_roots`] replaces the items at the top. An item
    /// cannot go below itself: an id of `parent` or of an item above it is
    /// left out. Returns false, and changes nothing, when the tree has no
    /// item `parent`.
    pub fn set_children<S: Into<String>>(&mut self, parent: &str, items: impl IntoIterator<Item = (S, T)>) -> bool {
        let Some(&p) = self.ids.get(parent) else {
            missing_item("set_children", parent);
            return false;
        };
        let placed = self.place(Some(p), items);
        let old = self.item_mut(p).children.replace(placed.clone()).unwrap_or_default();
        self.drop_left_out(old, &placed);
        self.changed();
        true
    }

    /// Say that the children of `id` come later: it gets an expander, and
    /// opening it makes [`TreeModel::expand`] return true. Children it had
    /// go, with everything below them, as when its folder is read again.
    /// Returns false when the tree has no item `id`.
    pub fn set_lazy(&mut self, id: &str) -> bool {
        let Some(&s) = self.ids.get(id) else {
            missing_item("set_lazy", id);
            return false;
        };
        if let Some(old) = self.item_mut(s).children.take() {
            for child in old {
                self.remove(child);
            }
        }
        self.changed();
        true
    }

    /// Open or close `id`, as [`crate::pages::Event::Expand`] asks. Returns
    /// true when it opened and its children are still to come (see
    /// [`TreeModel::set_lazy`]). An id the tree does not have is left alone.
    pub fn expand(&mut self, id: &str, open: bool) -> bool {
        let Some(&s) = self.ids.get(id) else {
            return false;
        };
        let item = self.item_mut(s);
        item.open = open;
        let load = open && item.children.is_none();
        self.changed();
        load
    }

    /// Whether `id` is open. Closed items keep the open state of the items
    /// below them.
    pub fn is_open(&self, id: &str) -> bool {
        self.ids.get(id).is_some_and(|&s| self.item(s).open)
    }

    /// The value of the item `id`.
    pub fn get(&self, id: &str) -> Option<&T> {
        self.ids.get(id).map(|&s| &self.item(s).value)
    }

    /// The value of the item `id`, to change it; where the item is in the
    /// tree stays as it is.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut T> {
        let s = *self.ids.get(id)?;
        Some(&mut self.item_mut(s).value)
    }

    /// Remove every item.
    pub fn clear(&mut self) {
        *self = TreeModel::new();
    }

    /// How many rows the tree shows: the items at the top, and below every
    /// open item, its children. The count of the tree node.
    pub fn count(&self) -> usize {
        self.shown().len()
    }

    /// Row `i`, from 0: the item it shows, its depth and its expander.
    pub fn row(&self, i: usize) -> Option<TreeRow<'_, T>> {
        let shown = self.shown().get(i).copied()?;
        let item = self.item(shown.slot);
        Some(TreeRow {
            id: &item.id,
            value: &item.value,
            depth: shown.depth,
            expander: shown.expander,
        })
    }

    /// The rows from `start`, at most `count`, for [`crate::pages::Page::rows`]:
    /// `describe` makes each from the item's id and value, and the tree sets
    /// the row's id, depth and expander.
    pub fn rows(&self, start: usize, count: usize, mut describe: impl FnMut(&str, &T) -> Row) -> Vec<Row> {
        let shown: Vec<Shown> = self.shown().iter().skip(start).take(count).copied().collect();
        shown
            .into_iter()
            .map(|s| {
                let item = self.item(s.slot);
                let mut row = describe(&item.id, &item.value);
                row.id.clone_from(&item.id);
                row.depth = s.depth;
                row.expander = s.expander;
                row
            })
            .collect()
    }

    /// The ids of the rows from `start`, at most `count`, for
    /// [`crate::pages::Page::row_ids`].
    pub fn row_ids(&self, start: usize, count: usize) -> Vec<String> {
        self.shown()
            .iter()
            .skip(start)
            .take(count)
            .map(|s| self.item(s.slot).id.clone())
            .collect()
    }

    fn item(&self, slot: usize) -> &Item<T> {
        self.slots[slot].as_ref().expect("a slot in the tree holds an item")
    }

    fn item_mut(&mut self, slot: usize) -> &mut Item<T> {
        self.slots[slot].as_mut().expect("a slot in the tree holds an item")
    }

    fn changed(&mut self) {
        *self.rows.get_mut() = None;
    }

    /// Put `items` under `parent` (the top when `None`), new or moved from
    /// elsewhere; their slots, in order.
    fn place<S: Into<String>>(&mut self, parent: Option<usize>, items: impl IntoIterator<Item = (S, T)>) -> Vec<usize> {
        let mut placed = Vec::new();
        let mut seen = HashSet::new();
        for (id, value) in items {
            let id = id.into();
            let slot = match self.ids.get(&id) {
                Some(&s) => {
                    if seen.contains(&s) || parent.is_some_and(|p| self.is_at_or_above(s, p)) {
                        continue;
                    }
                    if self.item(s).parent != parent {
                        self.detach(s);
                    }
                    let item = self.item_mut(s);
                    item.value = value;
                    item.parent = parent;
                    s
                }
                None => self.add(id, value, parent),
            };
            seen.insert(slot);
            placed.push(slot);
        }
        placed
    }

    /// Whether `slot` is `of` or an item above it.
    fn is_at_or_above(&self, slot: usize, of: usize) -> bool {
        let mut at = Some(of);
        while let Some(s) = at {
            if s == slot {
                return true;
            }
            at = self.item(s).parent;
        }
        false
    }

    /// Take `slot` out of its parent's children, or out of the top.
    fn detach(&mut self, slot: usize) {
        match self.item(slot).parent {
            Some(p) => {
                if let Some(children) = &mut self.item_mut(p).children {
                    children.retain(|&c| c != slot);
                }
            }
            None => self.roots.retain(|&c| c != slot),
        }
    }

    fn add(&mut self, id: String, value: T, parent: Option<usize>) -> usize {
        let item = Item {
            id: id.clone(),
            value,
            parent,
            children: Some(Vec::new()),
            open: false,
        };
        let slot = match self.free.pop() {
            Some(s) => {
                self.slots[s] = Some(item);
                s
            }
            None => {
                self.slots.push(Some(item));
                self.slots.len() - 1
            }
        };
        self.ids.insert(id, slot);
        slot
    }

    /// Remove the items of `old` that are not in `kept`, with everything
    /// below them.
    fn drop_left_out(&mut self, old: Vec<usize>, kept: &[usize]) {
        let kept: HashSet<usize> = kept.iter().copied().collect();
        for slot in old {
            if !kept.contains(&slot) {
                self.remove(slot);
            }
        }
    }

    /// Remove `slot` and everything below it.
    fn remove(&mut self, slot: usize) {
        let mut stack = vec![slot];
        while let Some(s) = stack.pop() {
            if let Some(item) = self.slots[s].take() {
                self.ids.remove(&item.id);
                stack.extend(item.children.into_iter().flatten());
                self.free.push(s);
            }
        }
    }

    /// The rows, worked out now if a change made them stale.
    fn shown(&self) -> Ref<'_, Vec<Shown>> {
        if self.rows.borrow().is_none() {
            let rows = self.flatten();
            *self.rows.borrow_mut() = Some(rows);
        }
        Ref::map(self.rows.borrow(), |r| r.as_ref().expect("worked out just now"))
    }

    /// The items at the top and, below every open item, its children, in
    /// order.
    fn flatten(&self) -> Vec<Shown> {
        let mut out = Vec::new();
        // Children are pushed in reverse so that they come off the stack in
        // order.
        let mut stack: Vec<(usize, u8)> = self.roots.iter().rev().map(|&s| (s, 0)).collect();
        while let Some((slot, depth)) = stack.pop() {
            let item = self.item(slot);
            let expander = match &item.children {
                Some(children) if children.is_empty() => Expander::None,
                _ if item.open => Expander::Expanded,
                _ => Expander::Collapsed,
            };
            out.push(Shown { slot, depth, expander });
            if item.open
                && let Some(children) = &item.children
            {
                let below = depth.saturating_add(1);
                stack.extend(children.iter().rev().map(|&c| (c, below)));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::{Cell, Tone};

    /// The rows as `id depth expander`, the expander as `-`, `+` or `v`.
    fn shown(tree: &TreeModel<&'static str>) -> Vec<String> {
        (0..tree.count())
            .map(|i| {
                let r = tree.row(i).expect("a row within the count");
                let mark = match r.expander {
                    Expander::None => "-",
                    Expander::Collapsed => "+",
                    Expander::Expanded => "v",
                };
                format!("{} {} {mark}", r.id, r.depth)
            })
            .collect()
    }

    /// a ( a1, a2 ( a2x ) ), b
    fn sample() -> TreeModel<&'static str> {
        let mut tree = TreeModel::new();
        tree.set_roots([("a", "A"), ("b", "B")]);
        assert!(tree.set_children("a", [("a1", "A1"), ("a2", "A2")]));
        assert!(tree.set_children("a2", [("a2x", "A2X")]));
        tree
    }

    #[test]
    fn open_items_show_their_children() {
        let mut tree = sample();
        assert_eq!(shown(&tree), ["a 0 +", "b 0 -"]);
        assert!(!tree.expand("a", true), "its children are there already");
        assert_eq!(shown(&tree), ["a 0 v", "a1 1 -", "a2 1 +", "b 0 -"]);
        tree.expand("a2", true);
        assert_eq!(shown(&tree), ["a 0 v", "a1 1 -", "a2 1 v", "a2x 2 -", "b 0 -"]);
        // Closing an item hides what is below it, which keeps its state.
        tree.expand("a", false);
        assert_eq!(shown(&tree), ["a 0 +", "b 0 -"]);
        assert!(tree.is_open("a2"));
        tree.expand("a", true);
        assert_eq!(tree.count(), 5);
        assert!(tree.row(5).is_none());
        assert!(!tree.expand("nope", true));
        assert_eq!(tree.get("a2x"), Some(&"A2X"));
        *tree.get_mut("a2x").unwrap() = "changed";
        assert_eq!(tree.row(3).unwrap().value, &"changed");
    }

    #[test]
    fn lazy_children_are_asked_for_when_the_item_opens() {
        let mut tree = sample();
        assert!(tree.set_lazy("b"));
        assert_eq!(shown(&tree), ["a 0 +", "b 0 +"]);
        assert!(tree.expand("b", true), "the page sets the children now");
        assert_eq!(shown(&tree), ["a 0 +", "b 0 v"]);
        tree.set_children("b", [("b/loading", "Loading")]);
        assert_eq!(shown(&tree), ["a 0 +", "b 0 v", "b/loading 1 -"]);
        tree.set_children("b", [("b1", "B1"), ("b2", "B2")]);
        assert_eq!(shown(&tree), ["a 0 +", "b 0 v", "b1 1 -", "b2 1 -"]);
        assert_eq!(tree.get("b/loading"), None, "the item left out is gone");
        tree.expand("b", false);
        assert!(!tree.expand("b", true), "the children are known now");
        // Setting no children makes it a leaf.
        tree.set_children("b", Vec::<(&str, &str)>::new());
        assert_eq!(shown(&tree), ["a 0 +", "b 0 -"]);
        // Read again: its children go.
        tree.set_children("b", [("b1", "B1")]);
        tree.set_lazy("b");
        assert_eq!(tree.get("b1"), None);
        assert!(tree.expand("b", true));
        assert!(!tree.set_lazy("nope"));
    }

    #[test]
    fn replacing_items_keeps_the_ones_still_there() {
        let mut tree = sample();
        tree.expand("a", true);
        tree.expand("a2", true);
        // A new order at the top, without b.
        tree.set_roots([("c", "C"), ("a", "A again")]);
        assert_eq!(shown(&tree), ["c 0 -", "a 0 v", "a1 1 -", "a2 1 v", "a2x 2 -"]);
        assert_eq!(tree.get("a"), Some(&"A again"));
        assert_eq!(tree.get("b"), None);
        // a1 goes with what is below it, a2 stays open.
        tree.set_children("a", [("a2", "A2")]);
        assert_eq!(shown(&tree), ["c 0 -", "a 0 v", "a2 1 v", "a2x 2 -"]);
        assert!(tree.get("a1").is_none());
        assert!(!tree.set_children("gone", [("x", "X")]));
    }

    #[test]
    fn an_id_moves_with_what_is_below_it_but_never_below_itself() {
        let mut tree = sample();
        tree.expand("a", true);
        tree.expand("a2", true);
        tree.expand("b", true);
        tree.set_children("b", [("a2", "A2")]);
        assert_eq!(shown(&tree), ["a 0 v", "a1 1 -", "b 0 v", "a2 1 v", "a2x 2 -"]);
        // b is above a2x: it stays where it is.
        tree.set_children("a2x", [("b", "B"), ("z", "Z")]);
        tree.expand("a2x", true);
        assert_eq!(shown(&tree), ["a 0 v", "a1 1 -", "b 0 v", "a2 1 v", "a2x 2 v", "z 3 -"]);
        // To the top, and an id given twice counts once.
        tree.set_roots([("a", "A"), ("z", "Z"), ("b", "B"), ("z", "Z twice")]);
        assert_eq!(shown(&tree), ["a 0 v", "a1 1 -", "z 0 -", "b 0 v", "a2 1 v", "a2x 2 -"]);
        assert_eq!(tree.get("z"), Some(&"Z"));
        tree.clear();
        assert_eq!(tree.count(), 0);
        assert_eq!(tree.get("a"), None);
    }

    #[test]
    fn rows_for_the_list() {
        let mut tree = sample();
        tree.expand("a", true);
        // `describe` may get the id wrong; the tree knows better.
        let rows = tree.rows(1, 2, |id, value| {
            Row::new(format!("wrong-{id}"), [*value]).depth(9).tone(Tone::Dim)
        });
        assert_eq!(
            rows,
            [
                Row {
                    depth: 1,
                    ..Row::new("a1", [Cell::text("A1")]).tone(Tone::Dim)
                },
                Row {
                    depth: 1,
                    expander: Expander::Collapsed,
                    ..Row::new("a2", [Cell::text("A2")]).tone(Tone::Dim)
                },
            ]
        );
        assert_eq!(tree.row_ids(0, 10), ["a", "a1", "a2", "b"]);
        assert_eq!(tree.row_ids(3, 10), ["b"]);
        assert!(tree.rows(10, 5, |id, _| Row::new(id, ["x"])).is_empty());
    }

    #[test]
    fn slots_are_used_again() {
        let mut tree = sample();
        let slots = tree.slots.len();
        for round in 0..10 {
            tree.set_children("a", [("x", "X"), ("y", "Y")]);
            tree.set_children("a", [(format!("z{round}"), "Z")]);
        }
        assert!(tree.slots.len() <= slots + 2, "{} slots", tree.slots.len());
    }
}
