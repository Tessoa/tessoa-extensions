//! Pages: whole tabs that the plugin describes and tessoa draws.
//!
//! A page is a type that implements [`Page`]. tessoa opens it with
//! [`Page::open`], shows what [`Page::render`] returns, and reports what the
//! user does through [`Page::event`]. The description is the whole page every
//! time; tessoa compares it with the previous one and keeps what the user
//! typed, scrolled and focused, matched by node keys.
//!
//! ```ignore
//! use tessoa_plugin_sdk::pages::*;
//!
//! struct Counter {
//!     count: u32,
//! }
//!
//! impl Page for Counter {
//!     fn open(_page: PageId, state: Option<Vec<u8>>, _args: Option<Vec<u8>>) -> Self {
//!         let count = state
//!             .and_then(|s| String::from_utf8(s).ok())
//!             .and_then(|s| s.parse().ok())
//!             .unwrap_or(0);
//!         Counter { count }
//!     }
//!
//!     fn event(&mut self, event: Event) -> Result<(), String> {
//!         match event {
//!             Event::Click { key } if key == "add" => self.count += 1,
//!             _ => {}
//!         }
//!         Ok(())
//!     }
//!
//!     fn render(&self) -> Node {
//!         col([
//!             text(self.count.to_string()).key("count").size(TextSize::Title),
//!             btn("Add").key("add"),
//!         ])
//!         .pad(Space::M)
//!         .gap(Space::S)
//!     }
//!
//!     fn save(&self) -> Option<Vec<u8>> {
//!         Some(self.count.to_string().into_bytes())
//!     }
//! }
//!
//! tessoa_plugin_sdk::export_pages!(Counter => "counter");
//! ```
//!
//! How a page works:
//!
//! - Every event is followed by a render of every open page of the plugin,
//!   and so is every command when the plugin has pages, and every event of
//!   the plugin as a whole: a task's progress, settings the user changed. A
//!   page never asks to be drawn again, and pages that show shared data stay
//!   in step.
//! - Nodes the user acts on need a key: events name the node by its key.
//!   Keys are unique within the page, so a list of cards uses keys such as
//!   `card-12`, not the position.
//! - A control shows the value the page gives it. When the user changes it,
//!   the page hears an event and shows the new value by rendering it; a page
//!   that keeps the old value puts the control back. A text field is the
//!   exception: it keeps what the user typed until the page gives it another
//!   value.
//! - A page never waits for the user. [`confirm`] returns a dialog id at
//!   once; the answer arrives later as [`Event::Dialog`]. Picking files,
//!   creating files and connections work the same way: the `_start`
//!   functions return an id at once, and the answer arrives as an event
//!   with that id. The blocking dialogs of [`crate::ui`] are refused in a
//!   page and read as cancel, and the functions that wait fail and name the
//!   one to call instead.
//! - Long lists are [`list`], [`table`] and [`tree`] nodes: the node says how
//!   many rows there are, and [`Page::rows`] describes the ones tessoa asks
//!   for, the ones on screen. Beyond two hundred items or so, use one
//!   instead of a column of nodes.
//! - Work that takes more than a second runs as a task ([`crate::Task`]):
//!   its progress and its end arrive in every open page as
//!   [`Event::TaskProgress`] and [`Event::TaskDone`].
//! - What [`Page::save`] returns is kept by tessoa and handed back to
//!   [`Page::open`] when tessoa restarts. At most 16 KiB.
//! - Layout works like a flexible box: `col` stacks its children top to
//!   bottom, `row` left to right; children stretch across the direction
//!   unless `align` says otherwise, and the children that `grow` share the
//!   remaining length equally. A page is at most 64 levels deep.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub use crate::bindings::exports::tessoa::plugin::pages::{
    Accepts, Align, ButtonKind, Cell, CellText, ChoiceOption, Column, Date, Expander, Fit, ImageSrc, Justify, KeyPress,
    Link, MenuItem, Row, RowAction, Selection, SizeHint, SortState, Space, Span, TextAlign, TextSize, Tone,
};
pub use crate::raw::files::{CreateOutcome, FileRef, Folder};
pub use crate::raw::net::Response;
pub use crate::raw::refs::TransferOutcome;
pub use crate::raw::tasks::TaskOutcome;
pub use crate::tree_model::{TreeModel, TreeRow};

use crate::bindings::exports::tessoa::plugin::pages as wire;
use wire::NodeKind;

/// Identifies one open page. tessoa issues it when the page opens.
pub type PageId = u64;

/// Identifies a question asked with [`confirm`]; the answer names it.
pub type DialogId = u64;

/// Where an image comes from: `Img::file("images/logo.png")` in the
/// plugin's directory, or `Img::added(id)` for one the plugin made with
/// [`crate::images::add`]. A plain `&str` works as a file too.
pub type Img = ImageSrc;

/// What a page implements. One value of the type is one open page.
pub trait Page: 'static {
    /// Build the page. `state` is what [`Page::save`] returned before tessoa
    /// restarted or the page was reloaded; `args` are what
    /// [`crate::pages_host::open`] passed. Both are `None` for a new page.
    /// Keep `page` if the page asks questions or sets its title.
    fn open(page: PageId, state: Option<Vec<u8>>, args: Option<Vec<u8>>) -> Self
    where
        Self: Sized;

    /// Something happened on the page, or to the plugin. An `Err` goes to
    /// the plugin log; the page stays open. Every open page is rendered
    /// again afterwards.
    fn event(&mut self, event: Event) -> Result<(), String> {
        let _ = event;
        Ok(())
    }

    /// Describe the whole page. Called after `open`, and after every event
    /// and every command for every open page. It only describes: no
    /// questions, no storage writes.
    fn render(&self) -> Node;

    /// The tab title. `None` keeps the title from `plugin.toml`, or the one
    /// set with [`crate::pages_host::set_title`].
    fn title(&self) -> Option<String> {
        None
    }

    /// Keys the page handles while it has the focus and no text field is
    /// focused, such as `"8"`, `"+"`, `"Enter"` or `"Ctrl+z"`. They arrive as
    /// [`Event::Key`]; every other key works as elsewhere in tessoa.
    fn keys(&self) -> Vec<String> {
        Vec::new()
    }

    /// Bytes tessoa keeps for the page and hands back to [`Page::open`] after
    /// a restart. Called with every render. At most 16 KiB; more is dropped
    /// and logged.
    fn save(&self) -> Option<Vec<u8>> {
        None
    }

    /// Rows of the list, table or tree whose key is `list`: from row
    /// `start`, at most `count` of them, fewer at the end. tessoa asks for
    /// the rows on screen and a screen above and below, at most 2000 at a
    /// time, so a list can have up to a million rows. Describe each row from
    /// the page's data, the same data giving the same row; more than `count`
    /// rows are dropped. A page with a list implements this; without it the
    /// list shows no rows, and the plugin log says why.
    ///
    /// ```ignore
    /// fn rows(&self, _list: &str, start: usize, count: usize) -> Vec<Row> {
    ///     self.files
    ///         .iter()
    ///         .skip(start)
    ///         .take(count)
    ///         .map(|f| Row::new(&f.id, [Cell::file(f.file), Cell::text(&f.note)]))
    ///         .collect()
    /// }
    /// ```
    fn rows(&self, list: &str, start: usize, count: usize) -> Vec<Row> {
        let _ = (list, start, count);
        Vec::new()
    }

    /// The ids of rows, without the rest of them: tessoa asks for them when
    /// the user selects a range of rows, or all of them, some of which were
    /// never on screen, at most 10,000 at a time. The default takes the ids
    /// of what [`Page::rows`] describes; a page whose rows are slow to
    /// describe answers with the ids alone.
    fn row_ids(&self, list: &str, start: usize, count: usize) -> Vec<String> {
        self.rows(list, start, count).into_iter().map(|r| r.id).collect()
    }
}

/// What happened, as [`Page::event`] receives it: what the user did on the
/// page, the answers to what the page asked for, and what concerns the
/// plugin as a whole.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Event {
    /// A button was pressed.
    Click { key: String },
    /// Enter in a text field, or Ctrl+Enter in one with several lines.
    /// `text` is the field's text at that moment.
    Submit { key: String, text: String },
    /// The text of a field changed.
    Input { key: String, text: String },
    /// A checkbox or a switch changed to `checked`, or a fold opened
    /// (`checked` true) or closed.
    Toggle { key: String, checked: bool },
    /// A key the page declared in [`Page::keys`].
    Key(KeyPress),
    /// Files were dropped on a drop zone. They arrive as references issued
    /// to the plugin, ready for [`crate::refs`] and [`file_row`].
    DropFiles { zone: String, files: Vec<FileRef> },
    /// A draggable node was dropped on a drop zone: its payload, and the key
    /// of the draggable it was dropped in front of, none at the end. Only
    /// the zone's own children that are draggable and have a key count, and
    /// never the one being dragged.
    DropItem {
        zone: String,
        payload: String,
        before: Option<String>,
    },
    /// An item of a node's menu was chosen.
    Menu { key: String, item: String },
    /// The user answered [`confirm`]; `ok` is true for the ok button.
    Dialog { id: DialogId, ok: bool },
    /// A copy, a move or a move to the recycle bin that this page started
    /// has ended: one outcome per file, in order.
    TransferDone { id: u64, outcomes: Vec<TransferOutcome> },
    /// The page came into view, or went out of view: as a tab, it became the
    /// visible tab of its pane or stopped being it; in the sidebar, its
    /// section was shown or hidden.
    Activated { visible: bool },
    /// An option of a dropdown, a radio group, segmented buttons or tabs was
    /// chosen: its id.
    Choose { key: String, option: String },
    /// A number input or a slider changed: the new value, within the node's
    /// range and on its step. While a slider is dragged, only the newest
    /// value waits to be delivered.
    Value { key: String, value: f64 },
    /// A colour was picked, as `0xrrggbb`. While the picker is dragged, only
    /// the newest waits to be delivered.
    Color { key: String, color: u32 },
    /// A date was chosen, or cleared (`None`).
    Date { key: String, date: Option<Date> },
    /// The bar of a split was let go: the share of the first child.
    Split { key: String, ratio: f32 },
    /// A link of rich text made with [`action`] was clicked: its id.
    Link { key: String, action: String },
    /// The user closed a popup: Escape, a click outside it or its close
    /// button. The popup stays open until the page renders it closed.
    Dismiss { key: String },
    /// The selected rows of a list, a table or a tree changed: the ids of
    /// the selected rows, in the order of the rows. A row the page took away
    /// stays in the selection until the user selects again.
    Select { list: String, ids: Vec<String> },
    /// A row was opened: double click or Enter.
    Activate { list: String, row: String },
    /// The expander of a tree's row was clicked; `open` is true to show its
    /// children. [`TreeModel::expand`] takes it as it comes.
    Expand { list: String, row: String, open: bool },
    /// A sortable column's header was clicked: first ascending, then the
    /// other way round. The page sorts its rows and renders the node's
    /// `sort` to match.
    Sort {
        list: String,
        column: String,
        descending: bool,
    },
    /// The checkbox of a row's cell was clicked: its new state.
    RowCheck {
        list: String,
        row: String,
        column: String,
        checked: bool,
    },
    /// A button at the end of a row was clicked: the id of the action.
    RowAction { list: String, row: String, action: String },
    /// The user picked files for [`crate::files::pick_files_start`], whose
    /// call id is `id`; empty when the user cancelled. The files are
    /// references issued to the plugin.
    PickedFiles { id: u64, files: Vec<FileRef> },
    /// The user picked a folder for [`crate::files::pick_folder_start`];
    /// `None` when the user cancelled.
    PickedFolder { id: u64, folder: Option<Folder> },
    /// A file of [`crate::files::create_file_start`] was created, skipped or
    /// refused.
    Created {
        id: u64,
        outcome: Result<CreateOutcome, crate::files::Error>,
    },
    /// The answer to [`crate::net::fetch_start`].
    Fetched {
        id: u64,
        response: Result<Response, crate::net::Error>,
    },
    /// A task of the plugin reported how far it is: `done` of `total`,
    /// `total` 0 when the total is not known. Every open page hears it; a
    /// page knows its own tasks by the id [`crate::tasks::start`] returned.
    TaskProgress {
        task: u64,
        done: f64,
        total: f64,
        text: String,
    },
    /// A task of the plugin ended. Every open page hears it, like
    /// [`Event::TaskProgress`].
    TaskDone { task: u64, outcome: TaskOutcome },
    /// The user changed settings of the plugin: their ids. The pages render
    /// again afterwards, so a page that reads its settings while it renders
    /// needs nothing more.
    SettingsChanged { ids: Vec<String> },
}

impl From<crate::PluginEvent> for Event {
    fn from(event: crate::PluginEvent) -> Event {
        match event {
            crate::PluginEvent::TaskProgress {
                task,
                done,
                total,
                text,
            } => Event::TaskProgress {
                task,
                done,
                total,
                text,
            },
            crate::PluginEvent::TaskDone { task, outcome } => Event::TaskDone { task, outcome },
            crate::PluginEvent::SettingsChanged { ids } => Event::SettingsChanged { ids },
        }
    }
}

/// Ask a yes/no question on behalf of a page with the labels OK and Cancel.
/// Returns at once; the answer arrives as [`Event::Dialog`] with the
/// returned id. Returns 0 when the page is not open.
pub fn confirm(page: PageId, title: &str, body: &str) -> DialogId {
    confirm_with(page, title, body, "OK", "Cancel")
}

/// [`confirm`] with custom button labels.
pub fn confirm_with(page: PageId, title: &str, body: &str, ok_label: &str, cancel_label: &str) -> DialogId {
    crate::raw::pages_host::ask(page, title, body, &[], ok_label, cancel_label)
}

/// [`confirm`] showing one line per item between the body and the buttons.
pub fn confirm_list(page: PageId, title: &str, body: &str, items: &[String]) -> DialogId {
    confirm_list_with(page, title, body, items, "OK", "Cancel")
}

/// [`confirm_list`] with custom button labels.
pub fn confirm_list_with(
    page: PageId,
    title: &str,
    body: &str,
    items: &[String],
    ok_label: &str,
    cancel_label: &str,
) -> DialogId {
    crate::raw::pages_host::ask(page, title, body, items, ok_label, cancel_label)
}

/// One node of a page and its children. Build nodes with the functions of
/// this module ([`col`], [`text`], [`btn`] and so on) and adjust them with
/// the methods.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    key: Option<String>,
    kind: NodeKind,
    layout: wire::Layout,
    menu: Vec<MenuItem>,
    tooltip: Option<String>,
    children: Vec<Node>,
    /// Methods called on a node they do not apply to, reported once in the
    /// plugin log.
    misuse: Vec<Misuse>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Misuse {
    method: &'static str,
    applies_to: &'static str,
}

fn layout() -> wire::Layout {
    wire::Layout {
        grow: false,
        w: SizeHint::Auto,
        h: SizeHint::Auto,
        pad: Space::None,
        gap: Space::None,
        align: Align::Stretch,
        justify: Justify::Start,
        wrap: false,
        scroll: false,
    }
}

fn node(kind: NodeKind) -> Node {
    Node {
        key: None,
        kind,
        layout: layout(),
        menu: Vec::new(),
        tooltip: None,
        children: Vec::new(),
        misuse: Vec::new(),
    }
}

fn container(kind: NodeKind, children: impl IntoIterator<Item = Node>) -> Node {
    Node {
        children: children.into_iter().collect(),
        ..node(kind)
    }
}

/// Children top to bottom.
pub fn col(children: impl IntoIterator<Item = Node>) -> Node {
    container(NodeKind::Column, children)
}

/// Children left to right.
pub fn row(children: impl IntoIterator<Item = Node>) -> Node {
    container(NodeKind::Row, children)
}

/// A column with a frame and a background.
pub fn card(children: impl IntoIterator<Item = Node>) -> Node {
    container(NodeKind::Card, children)
}

/// A thin line across the parent.
pub fn divider() -> Node {
    node(NodeKind::Divider)
}

/// Empty space; with [`Node::grow`], pushes its siblings apart.
pub fn spacer() -> Node {
    node(NodeKind::Spacer)
}

/// Text, in the body size and the normal tone.
pub fn text(text: impl Into<String>) -> Node {
    node(NodeKind::Text(wire::TextProps {
        text: text.into(),
        size: TextSize::Body,
        tone: Tone::Normal,
        wrap: false,
        align: TextAlign::Start,
    }))
}

/// A button. Its key arrives in [`Event::Click`], so give it one; a button
/// with a [`Node::menu`] opens the menu instead.
pub fn btn(label: impl Into<String>) -> Node {
    node(NodeKind::Button(wire::ButtonProps {
        label: label.into(),
        kind: ButtonKind::Normal,
        icon: None,
        disabled: false,
    }))
}

/// A button that shows only an icon from tessoa's icon set; the label is its
/// tooltip. The names: `plus`, `close`, `arrow-left`, `arrow-right`,
/// `arrow-up`, `arrow-down`, `chevron-down`, `chevron-right`, `check`,
/// `search`, `menu`, `more`, `reload`, `undo`, `copy`, `cut`, `paste`,
/// `new-folder`, `rename`, `delete`, `play`, `note`, `sliders`, `eye`, `sun`,
/// `gear`, `image`, `keyboard`, `monitor`, `info`, `pin`, `question`, `tag`,
/// `filter`, `puzzle`. A name tessoa does not know shows the puzzle piece;
/// TypeScript catches a wrong name when it builds, Rust does not.
pub fn icon_btn(icon: impl Into<String>, label: impl Into<String>) -> Node {
    node(NodeKind::IconButton(wire::ButtonProps {
        label: label.into(),
        kind: ButtonKind::Normal,
        icon: Some(icon.into()),
        disabled: false,
    }))
}

/// A text field showing `value`. Changes arrive as [`Event::Input`], Enter as
/// [`Event::Submit`]. Give it a key, so that tessoa keeps what the user typed
/// while the page changes around it.
pub fn text_input(value: impl Into<String>) -> Node {
    node(NodeKind::TextInput(wire::InputProps {
        value: value.into(),
        placeholder: String::new(),
        disabled: false,
        multiline: false,
    }))
}

/// A checkbox. Changes arrive as [`Event::Toggle`].
pub fn checkbox(checked: bool, label: impl Into<String>) -> Node {
    node(NodeKind::Checkbox(wire::CheckProps {
        checked,
        label: label.into(),
    }))
}

/// One file, drawn by tessoa: icon, name, size, and a dimmed look when the
/// file is gone. Double click opens it, the user can drag it out, and its
/// context menu is tessoa's file menu. The reference must have been issued
/// to the plugin.
pub fn file_row(file: FileRef) -> Node {
    node(NodeKind::FileRow(wire::FileRowProps {
        file,
        selected: false,
        draggable: true,
    }))
}

/// A target for dragged files or items. Drops arrive as
/// [`Event::DropFiles`] and [`Event::DropItem`] with the zone's key.
pub fn drop_zone(accepts: Accepts, children: impl IntoIterator<Item = Node>) -> Node {
    container(NodeKind::DropZone(wire::DropProps { accepts, hint: None }), children)
}

/// Content the user can drag onto drop zones of the same page that accept
/// items. The payload arrives in [`Event::DropItem`]. Give it a key, such as
/// the id of the item it shows: the event names the draggable an item was
/// dropped in front of by that key, and never names one without a key.
pub fn draggable(payload: impl Into<String>, children: impl IntoIterator<Item = Node>) -> Node {
    container(
        NodeKind::Draggable(wire::DragProps {
            payload: payload.into(),
        }),
        children,
    )
}

/// An option of a [`dropdown`], a [`radio`] group, [`segmented`] buttons or
/// [`tabs`]: an id, unique within the node and not empty, which
/// [`Event::Choose`] names, and a label.
pub fn opt(id: impl Into<String>, label: impl Into<String>) -> ChoiceOption {
    ChoiceOption {
        id: id.into(),
        label: label.into(),
        icon: None,
        disabled: false,
    }
}

impl ChoiceOption {
    /// An icon from tessoa's icon set before the label; the names are
    /// listed at [`icon_btn`].
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Shown, but cannot be chosen.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

fn choice_props(value: impl Into<String>, options: impl IntoIterator<Item = ChoiceOption>) -> wire::ChoiceProps {
    wire::ChoiceProps {
        value: value.into(),
        options: options.into_iter().collect(),
        disabled: false,
    }
}

/// A button showing the chosen option; clicking it opens the options.
/// `value` is the id of the chosen option, or `""` for none; at most 256
/// options. The choice arrives as [`Event::Choose`]: render it as the new
/// value, or the dropdown goes back to the old one.
pub fn dropdown(value: impl Into<String>, options: impl IntoIterator<Item = ChoiceOption>) -> Node {
    node(NodeKind::Dropdown(choice_props(value, options)))
}

/// The options one below the other, each with a round mark; like
/// [`dropdown`] otherwise.
pub fn radio(value: impl Into<String>, options: impl IntoIterator<Item = ChoiceOption>) -> Node {
    node(NodeKind::RadioGroup(choice_props(value, options)))
}

/// The options side by side as joined buttons; like [`dropdown`] otherwise.
pub fn segmented(value: impl Into<String>, options: impl IntoIterator<Item = ChoiceOption>) -> Node {
    node(NodeKind::Segmented(choice_props(value, options)))
}

/// A row of tabs; like [`dropdown`] otherwise. Only the tabs: the page shows
/// what belongs to the chosen one below them.
pub fn tabs(value: impl Into<String>, options: impl IntoIterator<Item = ChoiceOption>) -> Node {
    node(NodeKind::Tabs(choice_props(value, options)))
}

/// An on/off switch. Changes arrive as [`Event::Toggle`].
pub fn switch(on: bool, label: impl Into<String>) -> Node {
    node(NodeKind::Switch(wire::CheckProps {
        checked: on,
        label: label.into(),
    }))
}

/// A number field with up and down arrows, showing `value`. Without
/// [`Node::range`] it takes numbers from -1,000,000,000 to 1,000,000,000;
/// [`Node::step`] is 1 unless set, and it shows as many decimals as the
/// step has unless [`Node::decimals`] says more. The value arrives as
/// [`Event::Value`] on Enter, when the field loses the focus, and on the
/// arrows; not while the user types.
pub fn number_input(value: f64) -> Node {
    node(NodeKind::NumberInput(wire::NumberProps {
        value,
        min: -1e9,
        max: 1e9,
        step: 1.0,
        decimals: 0,
        suffix: String::new(),
        disabled: false,
    }))
}

/// A slider showing `value`, from 0 to 100 in steps of 1 unless
/// [`Node::range`] and [`Node::step`] say otherwise. The value arrives as
/// [`Event::Value`] while it is dragged, and once more when it is let go.
pub fn slider(value: f64) -> Node {
    node(NodeKind::Slider(wire::SliderProps {
        value,
        min: 0.0,
        max: 100.0,
        step: 1.0,
        vertical: false,
        disabled: false,
    }))
}

/// A progress bar: how far, from 0 to 1, or `None` when it is not known,
/// which shows a moving bar.
pub fn progress(value: Option<f32>) -> Node {
    node(NodeKind::Progress(wire::ProgressProps { value }))
}

/// An image: `image("images/logo.png")` from the plugin's directory, or
/// `image(Img::added(id))` for one the plugin made with
/// [`crate::images::add`]. Without [`Node::w`] and [`Node::h`] it takes its
/// own size in pixels; [`Node::fit`] says how it fills the node, and
/// [`Node::alt`] what it shows.
pub fn image(src: impl Into<ImageSrc>) -> Node {
    node(NodeKind::Image(wire::ImageProps {
        src: src.into(),
        fit: Fit::Contain,
        alt: String::new(),
    }))
}

/// Text in several styles, with links: `rich([span("Read "), span("this").bold(),
/// span(" or "), url("the site", "https://example.com"), action("more", "more")])`.
/// At most 2048 spans and 64 KiB of text.
pub fn rich(spans: impl IntoIterator<Item = Span>) -> Node {
    node(NodeKind::RichText(wire::RichProps {
        spans: spans.into_iter().collect(),
        size: TextSize::Body,
        wrap: false,
        align: TextAlign::Start,
    }))
}

/// A stretch of [`rich`] text in the normal style; the methods make it
/// bold, italic, code or coloured.
pub fn span(text: impl Into<String>) -> Span {
    Span {
        text: text.into(),
        bold: false,
        italic: false,
        code: false,
        tone: Tone::Normal,
        link: None,
    }
}

/// A link to a web address in [`rich`] text, `http://` or `https://`. tessoa
/// asks the user before it opens the address in the browser; the page hears
/// nothing of it.
pub fn url(text: impl Into<String>, address: impl Into<String>) -> Span {
    Span {
        link: Some(Link::Url(address.into())),
        ..span(text)
    }
}

/// A link in [`rich`] text that the page handles: a click sends
/// [`Event::Link`] with `id`.
pub fn action(text: impl Into<String>, id: impl Into<String>) -> Span {
    Span {
        link: Some(Link::Action(id.into())),
        ..span(text)
    }
}

impl Span {
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// In a fixed-width font on a faint background.
    pub fn code(mut self) -> Self {
        self.code = true;
        self
    }

    /// The colour, by meaning.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
}

/// A swatch with the colour's code, `0xrrggbb`; clicking it opens a colour
/// picker. The colour arrives as [`Event::Color`].
pub fn color_picker(value: u32) -> Node {
    node(NodeKind::ColorPicker(wire::ColorProps { value, disabled: false }))
}

impl Date {
    /// A day of the calendar: year 1 to 9999, month 1 to 12, day 1 to the
    /// last day of the month. A day that does not exist breaks the page.
    pub fn new(year: u16, month: u8, day: u8) -> Date {
        Date { year, month, day }
    }
}

impl std::fmt::Display for Date {
    /// `2026-10-15`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// A button showing a date, `None` for an empty one; clicking it opens a
/// calendar. [`Node::min`] and [`Node::max`] limit the days the user can
/// choose. The date arrives as [`Event::Date`].
pub fn date_picker(value: Option<Date>) -> Node {
    node(NodeKind::DatePicker(wire::DateProps {
        value,
        min: None,
        max: None,
        disabled: false,
    }))
}

/// A column of a [`table`]: an id, unique within the node and not empty,
/// which [`Event::Sort`] and [`Event::RowCheck`] name, and the title shown
/// in the header. A list or a tree given columns uses the first only.
pub fn column(id: impl Into<String>, title: impl Into<String>) -> Column {
    Column {
        id: id.into(),
        title: title.into(),
        width: SizeHint::Auto,
        align: TextAlign::Start,
        sortable: false,
    }
}

impl Column {
    /// The width the column starts with; the user can drag it, and tessoa
    /// remembers where.
    pub fn width(mut self, width: SizeHint) -> Self {
        self.width = width;
        self
    }

    /// How the cells line up: `Start`, `Center` or `End`.
    pub fn align(mut self, align: Align) -> Self {
        self.align = text_align(align);
        self
    }

    /// A click on the header sends [`Event::Sort`].
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }
}

fn text_align(align: Align) -> TextAlign {
    match align {
        Align::Center => TextAlign::Center,
        Align::End => TextAlign::End,
        Align::Start | Align::Stretch => TextAlign::Start,
    }
}

fn list_props(count: usize, columns: Vec<Column>) -> wire::ListProps {
    wire::ListProps {
        count: u32::try_from(count).unwrap_or(u32::MAX),
        columns,
        selection: Selection::Single,
        selected: None,
        sort: None,
        two_line: false,
        empty: None,
    }
}

/// A list of `count` rows, at most a million, one column; [`Page::rows`]
/// describes the ones tessoa asks for. Give it a key: [`Page::rows`] and the
/// events name the list by it. The user selects one row at a time unless
/// [`Node::selection`] says otherwise, and tessoa draws the selection at
/// once; the page hears it as [`Event::Select`].
pub fn list(count: usize) -> Node {
    node(NodeKind::List(list_props(count, Vec::new())))
}

/// A list with `columns` under a header that can sort and resize them; like
/// [`list`] otherwise. The cells of a row go with the columns, in order.
pub fn table(count: usize, columns: impl IntoIterator<Item = Column>) -> Node {
    node(NodeKind::Table(list_props(count, columns.into_iter().collect())))
}

/// A list whose rows have levels and can expand; like [`list`] otherwise.
/// The page decides which rows exist and describes them with their depth
/// and expander; [`TreeModel`] does that work for it.
pub fn tree(count: usize) -> Node {
    node(NodeKind::Tree(list_props(count, Vec::new())))
}

/// Children in a grid, filled row by row, `columns` to a row (1 to 32). A
/// column is as wide as its widest child; room to spare is shared equally.
pub fn grid(columns: u32, children: impl IntoIterator<Item = Node>) -> Node {
    container(NodeKind::Grid(wire::GridProps { columns }), children)
}

/// A title that opens and closes the children below it. The user's click
/// arrives as [`Event::Toggle`]; render the new `open`. Closed, the
/// children stay in the tree but are not drawn.
pub fn fold(title: impl Into<String>, open: bool, children: impl IntoIterator<Item = Node>) -> Node {
    container(
        NodeKind::Fold(wire::FoldProps {
            title: title.into(),
            open,
        }),
        children,
    )
}

/// Two nodes side by side, or one above the other with [`Node::vertical`],
/// and a bar between them the user can drag. `ratio` is the share of the
/// first, between 0 and 1; tessoa keeps where the user dragged the bar until
/// the page gives another ratio, and reports it as [`Event::Split`] when the
/// bar is let go. A split has exactly these two children.
pub fn split(ratio: f32, first: Node, second: Node) -> Node {
    container(
        NodeKind::Split(wire::SplitProps { ratio, vertical: false }),
        [first, second],
    )
}

/// Children in a panel over the page while `open` is true, centred unless
/// [`Node::anchor`] names a node to show it below. [`Node::title`] adds a
/// title line with a close button, and [`Node::w`] sets its width. Escape,
/// a click outside and the close button send [`Event::Dismiss`]; the popup
/// closes when the page renders it with `open` false. A page shows one open
/// popup at a time.
pub fn popup(open: bool, children: impl IntoIterator<Item = Node>) -> Node {
    container(
        NodeKind::Popup(wire::PopupProps {
            open,
            title: None,
            anchor: None,
            width: SizeHint::Auto,
        }),
        children,
    )
}

/// An item of a node's menu. The id arrives in [`Event::Menu`].
pub fn item(id: impl Into<String>, label: impl Into<String>) -> MenuItem {
    MenuItem {
        id: id.into(),
        label: label.into(),
        disabled: false,
        danger: false,
        separator_after: false,
    }
}

impl MenuItem {
    /// Drawn in the warning colour, for destructive choices.
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    /// Shown, but cannot be chosen.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// A separator line follows the item.
    pub fn separator_after(mut self) -> Self {
        self.separator_after = true;
        self
    }
}

impl ImageSrc {
    /// A PNG, JPEG, GIF, WebP, BMP or SVG file in the plugin's directory,
    /// written with `/`, such as `images/logo.png`.
    pub fn file(path: impl Into<String>) -> ImageSrc {
        ImageSrc::File(path.into())
    }

    /// An image the plugin made with [`crate::images::add`] or
    /// [`crate::images::add_rgba`], by its number.
    pub fn added(id: u64) -> ImageSrc {
        ImageSrc::Added(id)
    }
}

impl From<&str> for ImageSrc {
    fn from(path: &str) -> ImageSrc {
        ImageSrc::file(path)
    }
}

impl From<String> for ImageSrc {
    fn from(path: String) -> ImageSrc {
        ImageSrc::file(path)
    }
}

impl Row {
    /// A row of a list, a table or a tree: an id, at most 128 bytes and
    /// unique within the list, which the events and the selection follow,
    /// and one cell per column, in order. Text works as a cell:
    /// `Row::new("r1", ["Name", "12 KB"])`; other cells are made with
    /// [`Cell`]'s functions.
    pub fn new<C: Into<Cell>>(id: impl Into<String>, cells: impl IntoIterator<Item = C>) -> Row {
        Row {
            id: id.into(),
            cells: cells.into_iter().map(Into::into).collect(),
            detail: None,
            depth: 0,
            expander: Expander::None,
            actions: Vec::new(),
            tone: Tone::Normal,
        }
    }

    /// The second line, dimmed, when the list has [`Node::two_line`].
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// The level in a tree, 0 at the top. Lists and tables ignore it.
    pub fn depth(mut self, depth: u8) -> Self {
        self.depth = depth;
        self
    }

    /// Whether the row of a tree has children, and whether they show.
    pub fn expander(mut self, expander: Expander) -> Self {
        self.expander = expander;
        self
    }

    /// A button at the end of the row, shown while the pointer is over it:
    /// an id, which [`Event::RowAction`] names, an icon from tessoa's icon
    /// set and its tooltip. At most 4; a row with more is refused.
    pub fn action(mut self, id: impl Into<String>, icon: impl Into<String>, label: impl Into<String>) -> Self {
        self.actions.push(RowAction {
            id: id.into(),
            icon: icon.into(),
            label: label.into(),
        });
        self
    }

    /// The colour of the whole row, by meaning.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
}

impl Cell {
    /// Text; the first 1 KiB shows.
    pub fn text(text: impl Into<String>) -> Cell {
        Cell::Text(CellText {
            text: text.into(),
            tone: Tone::Normal,
            icon: None,
        })
    }

    /// A file issued to the plugin, drawn as in tessoa's file list: it opens
    /// on a double click, can be dragged out, and has tessoa's file menu.
    pub fn file(file: FileRef) -> Cell {
        Cell::File(file)
    }

    /// A small picture, as for [`image`].
    pub fn image(src: impl Into<ImageSrc>) -> Cell {
        Cell::Image(src.into())
    }

    /// A checkbox; a click on it sends [`Event::RowCheck`].
    pub fn check(checked: bool) -> Cell {
        Cell::Check(checked)
    }

    /// A progress bar, from 0 to 1.
    pub fn progress(value: f32) -> Cell {
        Cell::Progress(value)
    }

    /// Text cells only: the colour, by meaning. Other cells stay as they are.
    pub fn tone(mut self, tone: Tone) -> Cell {
        if let Cell::Text(t) = &mut self {
            t.tone = tone;
        }
        self
    }

    /// Text cells only: an icon from tessoa's icon set before the text.
    /// Other cells stay as they are.
    pub fn icon(mut self, icon: impl Into<String>) -> Cell {
        if let Cell::Text(t) = &mut self {
            t.icon = Some(icon.into());
        }
        self
    }
}

impl From<&str> for Cell {
    fn from(text: &str) -> Cell {
        Cell::text(text)
    }
}

impl From<String> for Cell {
    fn from(text: String) -> Cell {
        Cell::text(text)
    }
}

impl From<&String> for Cell {
    fn from(text: &String) -> Cell {
        Cell::text(text.as_str())
    }
}

/// The number of decimals `step` needs to show exactly, at most 10: how many
/// times it is multiplied by 10 before it is a whole number, give or take a
/// billionth of itself.
fn decimals_of(step: f64) -> u8 {
    let mut x = step.abs();
    let mut decimals = 0;
    while decimals < 10 && (x - x.round()).abs() > 1e-9 * x {
        x *= 10.0;
        decimals += 1;
    }
    decimals
}

impl Node {
    /// Names the node in events and keeps its state across renders. Unique
    /// within the page. Events name nodes only by key: a node the user acts
    /// on without a key sends nothing.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Append a child. Only `col`, `row`, `card`, `grid`, `fold`, `popup`,
    /// `drop_zone` and `draggable` have children; a `split` has the two it
    /// was made with.
    pub fn child(mut self, child: Node) -> Self {
        self.children.push(child);
        self
    }

    /// Append children; an `Option<Node>` adds one or none.
    pub fn children(mut self, children: impl IntoIterator<Item = Node>) -> Self {
        self.children.extend(children);
        self
    }

    /// A line shown when the pointer rests on the node, at most 1 KiB. An
    /// icon button without one shows its label, an image its `alt`.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Take a share of the length the parent has left along its direction.
    /// Children that grow get equal shares whatever their content, as with
    /// CSS `flex: 1 1 0`, but none gets less than its content needs.
    pub fn grow(mut self) -> Self {
        self.layout.grow = true;
        self
    }

    /// The width: `SizeHint::Auto` (as wide as the content, the default),
    /// `SizeHint::Fill` or `SizeHint::Em(n)`. For a popup, the width of its
    /// panel.
    pub fn w(mut self, w: SizeHint) -> Self {
        match &mut self.kind {
            NodeKind::Popup(p) => p.width = w,
            _ => self.layout.w = w,
        }
        self
    }

    /// The height, like [`Node::w`].
    pub fn h(mut self, h: SizeHint) -> Self {
        self.layout.h = h;
        self
    }

    /// Space inside the node, around its children.
    pub fn pad(mut self, pad: Space) -> Self {
        self.layout.pad = pad;
        self
    }

    /// Space between the children.
    pub fn gap(mut self, gap: Space) -> Self {
        self.layout.gap = gap;
        self
    }

    /// For a container, where the children go across its direction (the
    /// default is `Align::Stretch`). For text and rich text, how its lines
    /// are aligned: `Start`, `Center` or `End`.
    pub fn align(mut self, align: Align) -> Self {
        match &mut self.kind {
            NodeKind::Text(t) => t.align = text_align(align),
            NodeKind::RichText(r) => r.align = text_align(align),
            _ => self.layout.align = align,
        }
        self
    }

    /// Where the children go along the container's direction.
    pub fn justify(mut self, justify: Justify) -> Self {
        self.layout.justify = justify;
        self
    }

    /// For a container, children that do not fit continue on the next line.
    /// For text and rich text, lines break to fit the width instead of being
    /// cut off.
    pub fn wrap(mut self) -> Self {
        match &mut self.kind {
            NodeKind::Text(t) => t.wrap = true,
            NodeKind::RichText(r) => r.wrap = true,
            _ => self.layout.wrap = true,
        }
        self
    }

    /// The node scrolls when its children do not fit.
    pub fn scroll(mut self) -> Self {
        self.layout.scroll = true;
        self
    }

    /// A small menu: right-clicking the node opens it, and so does a left
    /// click on a button or an icon button that has one, which then sends no
    /// [`Event::Click`]. The choice arrives as [`Event::Menu`] with the
    /// node's key, so give the node one. A list's menu acts on its selected
    /// rows.
    pub fn menu(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.menu = items.into_iter().collect();
        self
    }

    fn misused(mut self, method: &'static str, applies_to: &'static str) -> Self {
        self.misuse.push(Misuse { method, applies_to });
        self
    }

    /// Text and rich text only: the size.
    pub fn size(mut self, size: TextSize) -> Self {
        match &mut self.kind {
            NodeKind::Text(t) => t.size = size,
            NodeKind::RichText(r) => r.size = size,
            _ => return self.misused("size", "text and rich text"),
        }
        self
    }

    /// Text only: the colour, by meaning. Rich text has it per span.
    pub fn tone(mut self, tone: Tone) -> Self {
        match &mut self.kind {
            NodeKind::Text(t) => t.tone = tone,
            _ => return self.misused("tone", "text"),
        }
        self
    }

    /// Buttons only: `Primary`, `Danger` or `Quiet` instead of `Normal`.
    pub fn kind(mut self, kind: ButtonKind) -> Self {
        match &mut self.kind {
            NodeKind::Button(b) | NodeKind::IconButton(b) => b.kind = kind,
            _ => return self.misused("kind", "buttons"),
        }
        self
    }

    /// Buttons only: an icon from tessoa's icon set next to the label. The
    /// names are listed at [`icon_btn`].
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        match &mut self.kind {
            NodeKind::Button(b) | NodeKind::IconButton(b) => b.icon = Some(icon.into()),
            _ => return self.misused("icon", "buttons"),
        }
        self
    }

    /// Shown, but cannot be used: buttons, text fields, dropdowns, radio
    /// groups, segmented buttons, tabs, number inputs, sliders, colour
    /// pickers and date pickers.
    pub fn disabled(mut self, disabled: bool) -> Self {
        match &mut self.kind {
            NodeKind::Button(b) | NodeKind::IconButton(b) => b.disabled = disabled,
            NodeKind::TextInput(i) => i.disabled = disabled,
            NodeKind::Dropdown(c) | NodeKind::RadioGroup(c) | NodeKind::Segmented(c) | NodeKind::Tabs(c) => {
                c.disabled = disabled
            }
            NodeKind::NumberInput(n) => n.disabled = disabled,
            NodeKind::Slider(s) => s.disabled = disabled,
            NodeKind::ColorPicker(c) => c.disabled = disabled,
            NodeKind::DatePicker(d) => d.disabled = disabled,
            _ => return self.misused("disabled", "buttons, text fields and controls with a value"),
        }
        self
    }

    /// Text fields only: the grey text shown while the field is empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        match &mut self.kind {
            NodeKind::TextInput(i) => i.placeholder = placeholder.into(),
            _ => return self.misused("placeholder", "text fields"),
        }
        self
    }

    /// Text fields only: several lines. Enter starts a new line; Ctrl+Enter
    /// submits.
    pub fn multiline(mut self) -> Self {
        match &mut self.kind {
            NodeKind::TextInput(i) => i.multiline = true,
            _ => return self.misused("multiline", "text fields"),
        }
        self
    }

    /// File rows only: drawn as selected. A list selects its rows with
    /// [`Node::selected_rows`].
    pub fn selected(mut self, selected: bool) -> Self {
        match &mut self.kind {
            NodeKind::FileRow(f) => f.selected = selected,
            _ => return self.misused("selected", "file rows"),
        }
        self
    }

    /// File rows only: whether the user can drag the file out (the default)
    /// or not.
    pub fn draggable(mut self, draggable: bool) -> Self {
        match &mut self.kind {
            NodeKind::FileRow(f) => f.draggable = draggable,
            _ => return self.misused("draggable", "file rows"),
        }
        self
    }

    /// Drop zones only: shown in the zone while nothing is dragged over it.
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        match &mut self.kind {
            NodeKind::DropZone(d) => d.hint = Some(hint.into()),
            _ => return self.misused("hint", "drop zones"),
        }
        self
    }

    /// Number inputs and sliders only: the smallest and the largest value.
    pub fn range(mut self, min: f64, max: f64) -> Self {
        match &mut self.kind {
            NodeKind::NumberInput(n) => (n.min, n.max) = (min, max),
            NodeKind::Slider(s) => (s.min, s.max) = (min, max),
            _ => return self.misused("range", "number inputs and sliders"),
        }
        self
    }

    /// Number inputs and sliders only: what the arrows and the arrow keys
    /// add or take, and the values a typed number is rounded to; greater
    /// than 0. A number input shows at least as many decimals as the step
    /// has.
    pub fn step(mut self, step: f64) -> Self {
        match &mut self.kind {
            NodeKind::NumberInput(n) => {
                n.step = step;
                n.decimals = n.decimals.max(decimals_of(step));
            }
            NodeKind::Slider(s) => s.step = step,
            _ => return self.misused("step", "number inputs and sliders"),
        }
        self
    }

    /// Number inputs only: the digits shown after the decimal point.
    pub fn decimals(mut self, decimals: u8) -> Self {
        match &mut self.kind {
            NodeKind::NumberInput(n) => n.decimals = decimals,
            _ => return self.misused("decimals", "number inputs"),
        }
        self
    }

    /// Number inputs only: shown dimmed after the number, such as `px` or `%`.
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        match &mut self.kind {
            NodeKind::NumberInput(n) => n.suffix = suffix.into(),
            _ => return self.misused("suffix", "number inputs"),
        }
        self
    }

    /// Sliders and splits only: upright, a slider's minimum at the bottom,
    /// a split's first child on top.
    pub fn vertical(mut self) -> Self {
        match &mut self.kind {
            NodeKind::Slider(s) => s.vertical = true,
            NodeKind::Split(s) => s.vertical = true,
            _ => return self.misused("vertical", "sliders and splits"),
        }
        self
    }

    /// Images only: how the image fills the node, as CSS `object-fit`.
    pub fn fit(mut self, fit: Fit) -> Self {
        match &mut self.kind {
            NodeKind::Image(i) => i.fit = fit,
            _ => return self.misused("fit", "images"),
        }
        self
    }

    /// Images only: what the image shows, in words; its tooltip when the
    /// node has none.
    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        match &mut self.kind {
            NodeKind::Image(i) => i.alt = alt.into(),
            _ => return self.misused("alt", "images"),
        }
        self
    }

    /// Date pickers only: days before this cannot be chosen.
    pub fn min(mut self, min: Date) -> Self {
        match &mut self.kind {
            NodeKind::DatePicker(d) => d.min = Some(min),
            _ => return self.misused("min", "date pickers"),
        }
        self
    }

    /// Date pickers only: days after this cannot be chosen.
    pub fn max(mut self, max: Date) -> Self {
        match &mut self.kind {
            NodeKind::DatePicker(d) => d.max = Some(max),
            _ => return self.misused("max", "date pickers"),
        }
        self
    }

    fn list_like(&mut self) -> Option<&mut wire::ListProps> {
        match &mut self.kind {
            NodeKind::List(l) | NodeKind::Table(l) | NodeKind::Tree(l) => Some(l),
            _ => None,
        }
    }

    /// Lists, tables and trees only: the columns, at most 32. A table gets
    /// them from [`table`]; a list or a tree uses the first only.
    pub fn columns(mut self, columns: impl IntoIterator<Item = Column>) -> Self {
        match self.list_like() {
            Some(l) => l.columns = columns.into_iter().collect(),
            None => return self.misused("columns", "lists, tables and trees"),
        }
        self
    }

    /// Lists, tables and trees only: how rows can be selected,
    /// `Selection::Single` unless set. With `Selection::None`, Enter and a
    /// double click still send [`Event::Activate`].
    pub fn selection(mut self, selection: Selection) -> Self {
        match self.list_like() {
            Some(l) => l.selection = selection,
            None => return self.misused("selection", "lists, tables and trees"),
        }
        self
    }

    /// Lists, tables and trees only: select these rows by id, replacing what
    /// the user selected, when they differ from what the previous render
    /// gave. Without it the selection is the user's. At most 10,000.
    pub fn selected_rows<S: Into<String>>(mut self, ids: impl IntoIterator<Item = S>) -> Self {
        match self.list_like() {
            Some(l) => l.selected = Some(ids.into_iter().map(Into::into).collect()),
            None => return self.misused("selected_rows", "lists, tables and trees"),
        }
        self
    }

    /// Lists, tables and trees only: the column the page sorted its rows by,
    /// shown as an arrow in the header; tessoa does not sort. The column is
    /// one of the node's, and sortable.
    pub fn sort(mut self, column: impl Into<String>, descending: bool) -> Self {
        match self.list_like() {
            Some(l) => {
                l.sort = Some(SortState {
                    column: column.into(),
                    descending,
                })
            }
            None => return self.misused("sort", "lists, tables and trees"),
        }
        self
    }

    /// Lists, tables and trees only: each row has a second, dimmed line, its
    /// [`Row::detail`].
    pub fn two_line(mut self) -> Self {
        match self.list_like() {
            Some(l) => l.two_line = true,
            None => return self.misused("two_line", "lists, tables and trees"),
        }
        self
    }

    /// Lists, tables and trees only: shown when there are no rows.
    pub fn empty(mut self, text: impl Into<String>) -> Self {
        match self.list_like() {
            Some(l) => l.empty = Some(text.into()),
            None => return self.misused("empty", "lists, tables and trees"),
        }
        self
    }

    /// Popups only: a title line with a close button.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        match &mut self.kind {
            NodeKind::Popup(p) => p.title = Some(title.into()),
            _ => return self.misused("title", "popups"),
        }
        self
    }

    /// Popups only: the key of the node to show the popup below.
    pub fn anchor(mut self, key: impl Into<String>) -> Self {
        match &mut self.kind {
            NodeKind::Popup(p) => p.anchor = Some(key.into()),
            _ => return self.misused("anchor", "popups"),
        }
        self
    }
}

/// The node kind as WIT spells it, for messages.
fn kind_name(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Column => "column",
        NodeKind::Row => "row",
        NodeKind::Card => "card",
        NodeKind::Divider => "divider",
        NodeKind::Spacer => "spacer",
        NodeKind::Text(_) => "text",
        NodeKind::Button(_) => "button",
        NodeKind::IconButton(_) => "icon-button",
        NodeKind::TextInput(_) => "text-input",
        NodeKind::Checkbox(_) => "checkbox",
        NodeKind::FileRow(_) => "file-row",
        NodeKind::DropZone(_) => "drop-zone",
        NodeKind::Draggable(_) => "draggable",
        NodeKind::Dropdown(_) => "dropdown",
        NodeKind::RadioGroup(_) => "radio-group",
        NodeKind::Segmented(_) => "segmented",
        NodeKind::Tabs(_) => "tabs",
        NodeKind::Switch(_) => "switch",
        NodeKind::NumberInput(_) => "number-input",
        NodeKind::Slider(_) => "slider",
        NodeKind::Progress(_) => "progress",
        NodeKind::Image(_) => "image",
        NodeKind::RichText(_) => "rich-text",
        NodeKind::ColorPicker(_) => "color-picker",
        NodeKind::DatePicker(_) => "date-picker",
        NodeKind::List(_) => "list",
        NodeKind::Table(_) => "table",
        NodeKind::Tree(_) => "tree",
        NodeKind::Grid(_) => "grid",
        NodeKind::Fold(_) => "fold",
        NodeKind::Split(_) => "split",
        NodeKind::Popup(_) => "popup",
    }
}

/// The tree as the interface carries it: in pre-order, each node naming its
/// parent by position. Also returns the messages about misused methods.
fn flatten(root: Node) -> (Vec<wire::Node>, Vec<String>) {
    let mut nodes = Vec::new();
    let mut notes = Vec::new();
    // Children are pushed in reverse so that they come off the stack in
    // order.
    let mut stack = vec![(None, root)];
    while let Some((parent, n)) = stack.pop() {
        let index = nodes.len() as u32;
        for m in &n.misuse {
            notes.push(format!(
                "{}() is for {}; it was called on a {}{}",
                m.method,
                m.applies_to,
                kind_name(&n.kind),
                n.key.as_deref().map(|k| format!(" (key {k:?})")).unwrap_or_default()
            ));
        }
        nodes.push(wire::Node {
            parent,
            key: n.key,
            kind: n.kind,
            layout: n.layout,
            menu: n.menu,
            tooltip: n.tooltip,
        });
        for child in n.children.into_iter().rev() {
            stack.push((Some(index), child));
        }
    }
    (nodes, notes)
}

/// The lists, tables and trees of a tree that have a key, and how many rows
/// each says it has.
fn lists_of(nodes: &[wire::Node]) -> HashMap<String, usize> {
    nodes
        .iter()
        .filter_map(|n| match (&n.key, &n.kind) {
            (Some(key), NodeKind::List(l) | NodeKind::Table(l) | NodeKind::Tree(l)) => {
                Some((key.clone(), l.count as usize))
            }
            _ => None,
        })
        .collect()
}

/// The text of each text field of a page, by key, for [`Event::Submit`].
#[derive(Debug, Default)]
struct Fields {
    /// Key to (the value the last render gave, the text the field holds).
    map: HashMap<String, (String, String)>,
}

impl Fields {
    /// After a render: a field whose value the page changed shows the new
    /// value; any other keeps what the user typed.
    fn rendered(&mut self, nodes: &[wire::Node]) {
        let mut seen = BTreeSet::new();
        for n in nodes {
            if let (Some(key), NodeKind::TextInput(i)) = (&n.key, &n.kind) {
                seen.insert(key.as_str());
                match self.map.get_mut(key) {
                    Some((value, text)) if *value != i.value => {
                        value.clone_from(&i.value);
                        text.clone_from(&i.value);
                    }
                    Some(_) => {}
                    None => {
                        self.map.insert(key.clone(), (i.value.clone(), i.value.clone()));
                    }
                }
            }
        }
        self.map.retain(|k, _| seen.contains(k.as_str()));
    }

    fn input(&mut self, key: &str, text: &str) {
        match self.map.get_mut(key) {
            Some((_, t)) => *t = text.to_string(),
            None => {
                self.map.insert(key.to_string(), (String::new(), text.to_string()));
            }
        }
    }

    fn text(&self, key: &str) -> String {
        self.map.get(key).map(|(_, t)| t.clone()).unwrap_or_default()
    }
}

/// The event as the page receives it.
fn translate(fields: &mut Fields, ev: wire::PageEvent) -> Event {
    use wire::{DropPayload, PageEvent};
    match ev {
        PageEvent::Click(key) => Event::Click { key },
        PageEvent::Submit(key) => {
            let text = fields.text(&key);
            Event::Submit { key, text }
        }
        PageEvent::Input((key, text)) => {
            fields.input(&key, &text);
            Event::Input { key, text }
        }
        PageEvent::Toggle((key, checked)) => Event::Toggle { key, checked },
        PageEvent::Key(k) => Event::Key(k),
        PageEvent::Drop((zone, DropPayload::Files(files))) => Event::DropFiles { zone, files },
        PageEvent::Drop((zone, DropPayload::Item((payload, before)))) => Event::DropItem { zone, payload, before },
        PageEvent::TransferDone((id, outcomes)) => Event::TransferDone { id, outcomes },
        PageEvent::Dialog((id, ok)) => Event::Dialog { id, ok },
        PageEvent::Menu((key, item)) => Event::Menu { key, item },
        PageEvent::Activated(visible) => Event::Activated { visible },
        PageEvent::Choose((key, option)) => Event::Choose { key, option },
        PageEvent::Value((key, value)) => Event::Value { key, value },
        PageEvent::Color((key, color)) => Event::Color { key, color },
        PageEvent::Date((key, date)) => Event::Date { key, date },
        PageEvent::Split((key, ratio)) => Event::Split { key, ratio },
        PageEvent::Link((key, action)) => Event::Link { key, action },
        PageEvent::Dismiss(key) => Event::Dismiss { key },
        PageEvent::Select((list, ids)) => Event::Select { list, ids },
        PageEvent::Activate((list, row)) => Event::Activate { list, row },
        PageEvent::Expand(e) => Event::Expand {
            list: e.list,
            row: e.row,
            open: e.open,
        },
        PageEvent::Sort(e) => Event::Sort {
            list: e.list,
            column: e.column,
            descending: e.descending,
        },
        PageEvent::RowCheck(e) => Event::RowCheck {
            list: e.list,
            row: e.row,
            column: e.column,
            checked: e.checked,
        },
        PageEvent::RowAction(e) => Event::RowAction {
            list: e.list,
            row: e.row,
            action: e.action,
        },
        PageEvent::PickedFiles((id, files)) => Event::PickedFiles { id, files },
        PageEvent::PickedFolder((id, folder)) => Event::PickedFolder { id, folder },
        PageEvent::Created((id, outcome)) => Event::Created { id, outcome },
        PageEvent::Fetched((id, response)) => Event::Fetched { id, response },
    }
}

/// The machinery behind `export!` and `export_pages!`. Not for plugins.
#[doc(hidden)]
pub mod __private {
    use super::*;

    /// A page of any type.
    pub trait Erased {
        fn event(&mut self, event: Event) -> Result<(), String>;
        fn render(&self) -> Node;
        fn title(&self) -> Option<String>;
        fn keys(&self) -> Vec<String>;
        fn save(&self) -> Option<Vec<u8>>;
        fn rows(&self, list: &str, start: usize, count: usize) -> Vec<Row>;
        fn row_ids(&self, list: &str, start: usize, count: usize) -> Vec<String>;
    }

    impl<P: Page> Erased for P {
        fn event(&mut self, event: Event) -> Result<(), String> {
            Page::event(self, event)
        }
        fn render(&self) -> Node {
            Page::render(self)
        }
        fn title(&self) -> Option<String> {
            Page::title(self)
        }
        fn keys(&self) -> Vec<String> {
            Page::keys(self)
        }
        fn save(&self) -> Option<Vec<u8>> {
            Page::save(self)
        }
        fn rows(&self, list: &str, start: usize, count: usize) -> Vec<Row> {
            Page::rows(self, list, start, count)
        }
        fn row_ids(&self, list: &str, start: usize, count: usize) -> Vec<String> {
            Page::row_ids(self, list, start, count)
        }
    }

    /// Opens a page of one type.
    pub type Make = fn(PageId, Option<Vec<u8>>, Option<Vec<u8>>) -> Box<dyn Erased>;

    pub fn make<P: Page>(page: PageId, state: Option<Vec<u8>>, args: Option<Vec<u8>>) -> Box<dyn Erased> {
        Box::new(P::open(page, state, args))
    }

    struct Open {
        page: Box<dyn Erased>,
        fields: Fields,
        /// The lists of the last render, by key, with their numbers of rows.
        lists: HashMap<String, usize>,
    }

    #[derive(Default)]
    struct Pages {
        open: BTreeMap<PageId, Open>,
        /// Misuse messages already logged, so that a page rendering twenty
        /// times a second does not repeat them.
        noted: BTreeSet<String>,
    }

    thread_local! {
        static PAGES: RefCell<Pages> = RefCell::new(Pages::default());
    }

    // A page is taken out of the table while its own code runs, so that the
    // table is never borrowed across a call into the plugin's code.
    fn take(page: PageId) -> Result<Open, String> {
        PAGES
            .with(|p| p.borrow_mut().open.remove(&page))
            .ok_or_else(|| format!("page {page} is not open"))
    }

    fn put(page: PageId, open: Open) {
        PAGES.with(|p| p.borrow_mut().open.insert(page, open));
    }

    fn open_pages() -> Vec<PageId> {
        PAGES.with(|p| p.borrow().open.keys().copied().collect())
    }

    /// Ask for every open page to be rendered again.
    fn refresh() {
        for page in open_pages() {
            crate::raw::pages_host::invalidate(page);
        }
    }

    pub fn open(
        page: PageId,
        kind: &str,
        state: Option<Vec<u8>>,
        args: Option<Vec<u8>>,
        kinds: &[(&str, Make)],
    ) -> Result<(), String> {
        let Some((_, make)) = kinds.iter().find(|(k, _)| *k == kind) else {
            let known: Vec<&str> = kinds.iter().map(|(k, _)| *k).collect();
            return Err(format!("no page {kind:?} is exported; export_pages! lists {known:?}"));
        };
        let page_value = make(page, state, args);
        put(
            page,
            Open {
                page: page_value,
                fields: Fields::default(),
                lists: HashMap::new(),
            },
        );
        Ok(())
    }

    pub fn event(page: PageId, ev: wire::PageEvent) -> Result<(), String> {
        let mut open = take(page)?;
        let event = translate(&mut open.fields, ev);
        let result = open.page.event(event);
        put(page, open);
        refresh();
        result
    }

    /// Rows of a list of a page, from [`Page::rows`].
    pub fn rows(page: PageId, list: &str, start: u32, count: u32) -> Result<Vec<wire::Row>, String> {
        let open = take(page)?;
        let rows = answer(&open, page, list, start, count, "rows", |p, l, s, c| p.rows(l, s, c));
        put(page, open);
        rows
    }

    /// Row ids of a list of a page, from [`Page::row_ids`].
    pub fn row_ids(page: PageId, list: &str, start: u32, count: u32) -> Result<Vec<String>, String> {
        let open = take(page)?;
        let ids = answer(&open, page, list, start, count, "row_ids", |p, l, s, c| {
            p.row_ids(l, s, c)
        });
        put(page, open);
        ids
    }

    /// What the page describes for a request about the list `list`: at most
    /// `count` of them, and never none where the list has rows.
    fn answer<T>(
        open: &Open,
        page: PageId,
        list: &str,
        start: u32,
        count: u32,
        method: &str,
        describe: impl FnOnce(&dyn Erased, &str, usize, usize) -> Vec<T>,
    ) -> Result<Vec<T>, String> {
        let Some(&total) = open.lists.get(list) else {
            return Err(no_list(page, list));
        };
        let (start, count) = (start as usize, count as usize);
        let mut out = describe(&*open.page, list, start, count);
        out.truncate(count);
        if out.is_empty() && count > 0 && start < total {
            return Err(format!(
                "Page::{method} described nothing from row {start} of list {list:?}, which has {total} rows; a page with a list, a table or a tree describes its rows in Page::rows"
            ));
        }
        Ok(out)
    }

    fn no_list(page: PageId, list: &str) -> String {
        format!("no list named {list:?} on page {page}")
    }

    /// Something that concerns the plugin as a whole: `P`, the commands'
    /// handler, hears it, then every open page, and every page renders
    /// again.
    pub fn plugin_event<P: crate::Plugin>(ev: crate::bindings::exports::tessoa::plugin::plugin::PluginEvent) {
        for problem in deliver::<P>(crate::__exports::plugin_event(ev)) {
            crate::log::warn(&problem);
        }
        refresh();
    }

    /// Hand a plugin event to `P` and to every open page; what the pages
    /// answered with an error, one line each.
    fn deliver<P: crate::Plugin>(event: crate::PluginEvent) -> Vec<String> {
        P::on_event(event.clone());
        let mut problems = Vec::new();
        for page in open_pages() {
            let Ok(mut open) = take(page) else { continue };
            let result = open.page.event(Event::from(event.clone()));
            put(page, open);
            if let Err(e) = result {
                problems.push(format!("page {page}: {e}"));
            }
        }
        problems
    }

    pub fn render(page: PageId) -> Result<wire::Tree, String> {
        let mut open = take(page)?;
        let root = open.page.render();
        let title = open.page.title();
        let keys = open.page.keys();
        let state = open.page.save();
        let (nodes, notes) = flatten(root);
        open.fields.rendered(&nodes);
        open.lists = lists_of(&nodes);
        put(page, open);
        for note in notes {
            let new = PAGES.with(|p| p.borrow_mut().noted.insert(note.clone()));
            if new {
                crate::log::warn(&note);
            }
        }
        Ok(wire::Tree {
            title,
            keys,
            state,
            nodes,
        })
    }

    pub fn close(page: PageId) {
        PAGES.with(|p| p.borrow_mut().open.remove(&page));
    }

    /// Run a command of a plugin that has pages; its pages are rendered
    /// again afterwards, as after an event.
    pub fn command(run: impl FnOnce() -> Result<(), String>) -> Result<(), String> {
        let result = run();
        refresh();
        result
    }

    /// The answer of a plugin exported with `export!` to every page call.
    pub fn no_pages() -> String {
        "this plugin has no pages: it is exported with export!, not export_pages!".to_string()
    }

    /// The command handler of a plugin exported with `export_pages!` and no
    /// commands.
    pub struct NoCommands;

    impl crate::Plugin for NoCommands {
        fn run_command(command_id: &str) -> Result<(), String> {
            Err(format!("no command {command_id:?}: this plugin has pages only"))
        }
    }

    #[cfg(test)]
    pub(super) mod testing {
        use super::*;

        pub fn deliver_to_pages<P: crate::Plugin>(event: crate::PluginEvent) -> Vec<String> {
            deliver::<P>(event)
        }

        /// Render without logging, for tests: the tree and the notes.
        pub fn render_quietly(page: PageId) -> (wire::Tree, Vec<String>) {
            let mut open = take(page).expect("the page is open");
            let (nodes, notes) = flatten(open.page.render());
            open.fields.rendered(&nodes);
            open.lists = lists_of(&nodes);
            put(page, open);
            (
                wire::Tree {
                    title: None,
                    keys: Vec::new(),
                    state: None,
                    nodes,
                },
                notes,
            )
        }

        /// Hand a page event to a page without asking for renders.
        pub fn event_quietly(page: PageId, ev: wire::PageEvent) -> Result<(), String> {
            let mut open = take(page)?;
            let event = translate(&mut open.fields, ev);
            let result = open.page.event(event);
            put(page, open);
            result
        }
    }
}

/// Export page types, and optionally a command handler, tasks and columns,
/// as the plugin's entry points. Each page type is named with the `id` its
/// `[[pages]]` entry in `plugin.toml` has:
///
/// ```ignore
/// tessoa_plugin_sdk::export_pages!(Calculator => "calculator");
/// tessoa_plugin_sdk::export_pages!(Board => "board", Archive => "archive");
/// tessoa_plugin_sdk::export_pages!(Shelf => "shelf"; commands: ShelfCommands);
/// tessoa_plugin_sdk::export_pages!(Hasher => "hasher"; tasks: [("hash", Hash)]; columns: Dims);
/// ```
///
/// After the pages come the optional sections, in any order: `commands`
/// names a type that implements [`crate::Plugin`], without which the plugin
/// has no commands; `tasks` and `columns` are as for [`crate::export!`].
/// Use this macro instead of [`crate::export!`], not in addition to it.
#[macro_export]
macro_rules! export_pages {
    ($($page:ty => $kind:literal),+ $(,)? $(; $($section:tt)+)?) => {
        $crate::__export_pages!(@pages [$($page => $kind),+] commands [] tasks [] columns [] ; $($($section)+)?);
    };
}

// Reads the sections of `export_pages!` one at a time, in any order, then
// makes the exports.
#[doc(hidden)]
#[macro_export]
macro_rules! __export_pages {
    (@pages [$($page:ty => $kind:literal),+] commands [$($commands:ty)?] tasks [$($tasks:tt)*] columns [$($columns:tt)*] ;) => {
        const _: () = {
            struct TessoaExportedPages;
            type TessoaCommands = $crate::__commands_or_none!($($commands)?);

            impl $crate::bindings::exports::tessoa::plugin::plugin::Guest for TessoaExportedPages {
                fn run_command(command_id: String) -> Result<(), String> {
                    $crate::pages::__private::command(|| <TessoaCommands as $crate::Plugin>::run_command(&command_id))
                }

                fn on_event(ev: $crate::bindings::exports::tessoa::plugin::plugin::PluginEvent) {
                    $crate::pages::__private::plugin_event::<TessoaCommands>(ev)
                }
            }

            $crate::__tasks!(TessoaExportedPages; $($tasks)*);
            $crate::__columns!(TessoaExportedPages; $($columns)*);

            impl $crate::bindings::exports::tessoa::plugin::pages::Guest for TessoaExportedPages {
                fn open(page: u64, kind: String, state: Option<Vec<u8>>, args: Option<Vec<u8>>) -> Result<(), String> {
                    $crate::pages::__private::open(
                        page,
                        &kind,
                        state,
                        args,
                        &[$(($kind, $crate::pages::__private::make::<$page> as $crate::pages::__private::Make)),+],
                    )
                }

                fn event(page: u64, ev: $crate::bindings::exports::tessoa::plugin::pages::PageEvent) -> Result<(), String> {
                    $crate::pages::__private::event(page, ev)
                }

                fn render(page: u64) -> Result<$crate::bindings::exports::tessoa::plugin::pages::Tree, String> {
                    $crate::pages::__private::render(page)
                }

                fn close(page: u64) {
                    $crate::pages::__private::close(page)
                }

                fn rows(
                    page: u64,
                    list: String,
                    start: u32,
                    count: u32,
                ) -> Result<Vec<$crate::bindings::exports::tessoa::plugin::pages::Row>, String> {
                    $crate::pages::__private::rows(page, &list, start, count)
                }

                fn row_ids(page: u64, list: String, start: u32, count: u32) -> Result<Vec<String>, String> {
                    $crate::pages::__private::row_ids(page, &list, start, count)
                }
            }

            #[cfg(target_arch = "wasm32")]
            $crate::bindings::export_plugin!(TessoaExportedPages with_types_in $crate::bindings);

            // Outside WebAssembly nothing calls the exports; referring to them
            // keeps the page types from being reported as unused.
            #[cfg(not(target_arch = "wasm32"))]
            let _ = (
                <TessoaExportedPages as $crate::bindings::exports::tessoa::plugin::plugin::Guest>::run_command,
                <TessoaExportedPages as $crate::bindings::exports::tessoa::plugin::plugin::Guest>::on_event,
                <TessoaExportedPages as $crate::bindings::exports::tessoa::plugin::task_runner::Guest>::run,
                <TessoaExportedPages as $crate::bindings::exports::tessoa::plugin::columns::Guest>::values,
                <TessoaExportedPages as $crate::bindings::exports::tessoa::plugin::pages::Guest>::open,
            );
        };
    };
    (@pages [$($pages:tt)*] commands [] tasks [$($tasks:tt)*] columns [$($columns:tt)*] ; commands: $commands:ty $(; $($rest:tt)*)?) => {
        $crate::__export_pages!(@pages [$($pages)*] commands [$commands] tasks [$($tasks)*] columns [$($columns)*] ; $($($rest)*)?);
    };
    (@pages [$($pages:tt)*] commands [$($commands:tt)*] tasks [] columns [$($columns:tt)*] ; tasks: [$($task:tt)*] $(; $($rest:tt)*)?) => {
        $crate::__export_pages!(@pages [$($pages)*] commands [$($commands)*] tasks [$($task)*] columns [$($columns)*] ; $($($rest)*)?);
    };
    (@pages [$($pages:tt)*] commands [$($commands:tt)*] tasks [$($tasks:tt)*] columns [] ; columns: $columns:expr $(; $($rest:tt)*)?) => {
        $crate::__export_pages!(@pages [$($pages)*] commands [$($commands)*] tasks [$($tasks)*] columns [$columns] ; $($($rest)*)?);
    };
}

// The commands' handler of `export_pages!`: the type named, or none.
#[doc(hidden)]
#[macro_export]
macro_rules! __commands_or_none {
    () => {
        $crate::pages::__private::NoCommands
    };
    ($commands:ty) => {
        $commands
    };
}

#[cfg(test)]
mod tests {
    use super::__private::testing::{deliver_to_pages, event_quietly, render_quietly};
    use super::*;

    fn kinds(nodes: &[wire::Node]) -> Vec<(Option<u32>, &'static str, Option<&str>)> {
        nodes
            .iter()
            .map(|n| (n.parent, kind_name(&n.kind), n.key.as_deref()))
            .collect()
    }

    #[test]
    fn a_tree_flattens_in_preorder_with_parents_first() {
        let tree = col([
            text("0").key("display").size(TextSize::Display),
            row([btn("7").key("k7"), btn("8").key("k8")]),
            card([text("a"), row([text("b")])]),
            divider(),
        ]);
        let (nodes, notes) = flatten(tree);
        assert!(notes.is_empty());
        assert_eq!(
            kinds(&nodes),
            [
                (None, "column", None),
                (Some(0), "text", Some("display")),
                (Some(0), "row", None),
                (Some(2), "button", Some("k7")),
                (Some(2), "button", Some("k8")),
                (Some(0), "card", None),
                (Some(5), "text", None),
                (Some(5), "row", None),
                (Some(7), "text", None),
                (Some(0), "divider", None),
            ]
        );
    }

    #[test]
    fn defaults_and_setters() {
        let (nodes, _) = flatten(
            col([
                text("x").align(Align::End).wrap(),
                btn("b").kind(ButtonKind::Primary).disabled(true),
            ])
            .pad(Space::M)
            .gap(Space::S)
            .align(Align::Center)
            .grow()
            .w(SizeHint::Em(10.0)),
        );
        let root = &nodes[0].layout;
        assert_eq!(
            (root.pad, root.gap, root.align, root.grow, root.w, root.h, root.wrap),
            (
                Space::M,
                Space::S,
                Align::Center,
                true,
                SizeHint::Em(10.0),
                SizeHint::Auto,
                false
            )
        );
        let NodeKind::Text(t) = &nodes[1].kind else { panic!() };
        assert_eq!((t.align, t.wrap), (TextAlign::End, true));
        assert_eq!(nodes[1].layout.align, Align::Stretch, "on text, align is the text's");
        assert!(!nodes[1].layout.wrap);
        let NodeKind::Button(b) = &nodes[2].kind else { panic!() };
        assert_eq!((b.kind, b.disabled), (ButtonKind::Primary, true));
        assert!(nodes.iter().all(|n| n.tooltip.is_none()));
    }

    #[test]
    fn misused_methods_are_reported_by_name() {
        let (_, notes) = flatten(col([
            btn("x").key("b").size(TextSize::Title),
            text("t").kind(ButtonKind::Danger),
            text("u").range(0.0, 1.0),
            slider(1.0).decimals(2).key("s"),
            switch(true, "On").disabled(true),
            list(3).title("Results"),
            popup(true, []).sort("name", false),
        ]));
        assert_eq!(
            notes,
            [
                "size() is for text and rich text; it was called on a button (key \"b\")",
                "kind() is for buttons; it was called on a text",
                "range() is for number inputs and sliders; it was called on a text",
                "decimals() is for number inputs; it was called on a slider (key \"s\")",
                "disabled() is for buttons, text fields and controls with a value; it was called on a switch",
                "title() is for popups; it was called on a list",
                "sort() is for lists, tables and trees; it was called on a popup",
            ]
        );
    }

    #[test]
    fn every_node_can_have_a_tooltip() {
        let (nodes, notes) = flatten(col([text("a").tooltip("A line"), spacer()]).tooltip("The page"));
        assert!(notes.is_empty());
        let tips: Vec<Option<&str>> = nodes.iter().map(|n| n.tooltip.as_deref()).collect();
        assert_eq!(tips, [Some("The page"), Some("A line"), None]);
    }

    /// The node kind and the properties of the single node `n` builds.
    fn one(n: Node) -> NodeKind {
        let (mut nodes, notes) = flatten(n);
        assert!(notes.is_empty(), "{notes:?}");
        nodes.remove(0).kind
    }

    #[test]
    fn choices_share_their_options() {
        let options = || {
            [
                opt("iso", "2026-10-15"),
                opt("us", "10/15/2026").icon("tag").disabled(true),
            ]
        };
        let props = wire::ChoiceProps {
            value: "us".into(),
            options: vec![
                ChoiceOption {
                    id: "iso".into(),
                    label: "2026-10-15".into(),
                    icon: None,
                    disabled: false,
                },
                ChoiceOption {
                    id: "us".into(),
                    label: "10/15/2026".into(),
                    icon: Some("tag".into()),
                    disabled: true,
                },
            ],
            disabled: false,
        };
        assert_eq!(one(dropdown("us", options())), NodeKind::Dropdown(props.clone()));
        assert_eq!(one(radio("us", options())), NodeKind::RadioGroup(props.clone()));
        assert_eq!(one(segmented("us", options())), NodeKind::Segmented(props.clone()));
        assert_eq!(
            one(tabs("us", options()).disabled(true)),
            NodeKind::Tabs(wire::ChoiceProps {
                disabled: true,
                ..props
            })
        );
    }

    #[test]
    fn the_value_controls() {
        assert_eq!(
            one(switch(true, "Dark")),
            NodeKind::Switch(wire::CheckProps {
                checked: true,
                label: "Dark".into()
            })
        );
        assert_eq!(
            one(number_input(12.0)),
            NodeKind::NumberInput(wire::NumberProps {
                value: 12.0,
                min: -1e9,
                max: 1e9,
                step: 1.0,
                decimals: 0,
                suffix: String::new(),
                disabled: false
            })
        );
        assert_eq!(
            one(number_input(0.5).range(0.0, 2.0).step(0.25).suffix("px").disabled(true)),
            NodeKind::NumberInput(wire::NumberProps {
                value: 0.5,
                min: 0.0,
                max: 2.0,
                step: 0.25,
                decimals: 2,
                suffix: "px".into(),
                disabled: true
            })
        );
        let NodeKind::NumberInput(n) = one(number_input(1.0).decimals(3).step(0.5)) else {
            panic!()
        };
        assert_eq!(n.decimals, 3, "a step never lowers the decimals asked for");
        let NodeKind::NumberInput(n) = one(number_input(1.0).step(0.1).decimals(0)) else {
            panic!()
        };
        assert_eq!(n.decimals, 0, "the decimals asked for last win");
        assert_eq!(
            one(slider(50.0)),
            NodeKind::Slider(wire::SliderProps {
                value: 50.0,
                min: 0.0,
                max: 100.0,
                step: 1.0,
                vertical: false,
                disabled: false
            })
        );
        assert_eq!(
            one(slider(0.5).range(0.0, 1.0).step(0.01).vertical()),
            NodeKind::Slider(wire::SliderProps {
                value: 0.5,
                min: 0.0,
                max: 1.0,
                step: 0.01,
                vertical: true,
                disabled: false
            })
        );
        assert_eq!(
            one(progress(Some(0.42))),
            NodeKind::Progress(wire::ProgressProps { value: Some(0.42) })
        );
        assert_eq!(
            one(progress(None)),
            NodeKind::Progress(wire::ProgressProps { value: None })
        );
        assert_eq!(
            one(color_picker(0xff8800)),
            NodeKind::ColorPicker(wire::ColorProps {
                value: 0xff8800,
                disabled: false
            })
        );
        let day = Date::new(2026, 10, 15);
        assert_eq!(day.to_string(), "2026-10-15");
        assert_eq!(Date::new(7, 1, 2).to_string(), "0007-01-02");
        assert_eq!(
            one(date_picker(Some(day))
                .min(Date::new(2026, 1, 1))
                .max(Date::new(2026, 12, 31))),
            NodeKind::DatePicker(wire::DateProps {
                value: Some(day),
                min: Some(Date::new(2026, 1, 1)),
                max: Some(Date::new(2026, 12, 31)),
                disabled: false
            })
        );
    }

    #[test]
    fn decimals_follow_the_step() {
        let cases = [
            (1.0, 0),
            (0.5, 1),
            (0.1, 1),
            (0.25, 2),
            (0.01, 2),
            (0.3, 1),
            (5.0, 0),
            (0.125, 3),
        ];
        for (step, decimals) in cases {
            assert_eq!(decimals_of(step), decimals, "{step}");
        }
        assert_eq!(decimals_of(1e-15), 10, "at most 10");
        assert_eq!(decimals_of(f64::NAN), 0);
    }

    #[test]
    fn images_and_rich_text() {
        assert_eq!(
            one(image("images/logo.png")),
            NodeKind::Image(wire::ImageProps {
                src: ImageSrc::File("images/logo.png".into()),
                fit: Fit::Contain,
                alt: String::new()
            })
        );
        assert_eq!(
            one(image(Img::added(7)).fit(Fit::Cover).alt("A chart")),
            NodeKind::Image(wire::ImageProps {
                src: ImageSrc::Added(7),
                fit: Fit::Cover,
                alt: "A chart".into()
            })
        );
        assert_eq!(Img::file("a.svg"), ImageSrc::File("a.svg".into()));
        let NodeKind::RichText(r) = one(rich([
            span("Plain "),
            span("bold").bold(),
            span("odd").italic().code().tone(Tone::Danger),
            url("site", "https://example.com/"),
            action("more", "more").bold(),
        ])
        .size(TextSize::Caption)
        .wrap()
        .align(Align::Center)) else {
            panic!()
        };
        assert_eq!((r.size, r.wrap, r.align), (TextSize::Caption, true, TextAlign::Center));
        type Style<'a> = (&'a str, bool, bool, bool, Tone, Option<Link>);
        let styles: Vec<Style> = r
            .spans
            .iter()
            .map(|s| (s.text.as_str(), s.bold, s.italic, s.code, s.tone, s.link.clone()))
            .collect();
        assert_eq!(
            styles,
            [
                ("Plain ", false, false, false, Tone::Normal, None),
                ("bold", true, false, false, Tone::Normal, None),
                ("odd", false, true, true, Tone::Danger, None),
                (
                    "site",
                    false,
                    false,
                    false,
                    Tone::Normal,
                    Some(Link::Url("https://example.com/".into()))
                ),
                (
                    "more",
                    true,
                    false,
                    false,
                    Tone::Normal,
                    Some(Link::Action("more".into()))
                ),
            ]
        );
    }

    #[test]
    fn lists_tables_and_trees() {
        let list_props = |count, columns| wire::ListProps {
            count,
            columns,
            selection: Selection::Single,
            selected: None,
            sort: None,
            two_line: false,
            empty: None,
        };
        assert_eq!(one(list(12000)), NodeKind::List(list_props(12000, vec![])));
        assert_eq!(one(tree(3)), NodeKind::Tree(list_props(3, vec![])));
        assert_eq!(
            one(list(usize::MAX)),
            NodeKind::List(list_props(u32::MAX, vec![])),
            "the runner refuses more than a million rows"
        );
        let columns = vec![
            column("name", "Name").width(SizeHint::Fill),
            column("size", "Size")
                .width(SizeHint::Em(6.0))
                .align(Align::End)
                .sortable(),
        ];
        assert_eq!(
            columns[1],
            Column {
                id: "size".into(),
                title: "Size".into(),
                width: SizeHint::Em(6.0),
                align: TextAlign::End,
                sortable: true
            }
        );
        assert_eq!(
            one(table(2, columns.clone())
                .selection(Selection::Multi)
                .selected_rows(["r1", "r2"])
                .sort("size", true)
                .two_line()
                .empty("Nothing yet")),
            NodeKind::Table(wire::ListProps {
                selection: Selection::Multi,
                selected: Some(vec!["r1".into(), "r2".into()]),
                sort: Some(SortState {
                    column: "size".into(),
                    descending: true
                }),
                two_line: true,
                empty: Some("Nothing yet".into()),
                ..list_props(2, columns.clone())
            })
        );
        assert_eq!(
            one(list(1).columns([column("name", "Name").align(Align::Center)])),
            NodeKind::List(list_props(
                1,
                vec![Column {
                    align: TextAlign::Center,
                    ..column("name", "Name")
                }]
            ))
        );
    }

    #[test]
    fn the_new_containers() {
        let (nodes, notes) = flatten(col([
            grid(3, [text("a"), text("b")]).key("g"),
            fold("Advanced", true, [text("c")]).key("f"),
            split(0.3, text("left"), text("right")).vertical().key("s"),
            popup(true, [text("d")])
                .title("Rename")
                .anchor("f")
                .w(SizeHint::Em(20.0))
                .key("p"),
        ]));
        assert!(notes.is_empty());
        assert_eq!(
            kinds(&nodes),
            [
                (None, "column", None),
                (Some(0), "grid", Some("g")),
                (Some(1), "text", None),
                (Some(1), "text", None),
                (Some(0), "fold", Some("f")),
                (Some(4), "text", None),
                (Some(0), "split", Some("s")),
                (Some(6), "text", None),
                (Some(6), "text", None),
                (Some(0), "popup", Some("p")),
                (Some(9), "text", None),
            ]
        );
        assert_eq!(nodes[1].kind, NodeKind::Grid(wire::GridProps { columns: 3 }));
        assert_eq!(
            nodes[4].kind,
            NodeKind::Fold(wire::FoldProps {
                title: "Advanced".into(),
                open: true
            })
        );
        assert_eq!(
            nodes[6].kind,
            NodeKind::Split(wire::SplitProps {
                ratio: 0.3,
                vertical: true
            })
        );
        assert_eq!(
            nodes[9].kind,
            NodeKind::Popup(wire::PopupProps {
                open: true,
                title: Some("Rename".into()),
                anchor: Some("f".into()),
                width: SizeHint::Em(20.0)
            })
        );
        assert_eq!(nodes[9].layout.w, SizeHint::Auto, "a popup's width is its panel's");
    }

    #[test]
    fn rows_and_cells() {
        let row = Row::new("r1", ["Name", "12 KB"]);
        assert_eq!(
            row,
            Row {
                id: "r1".into(),
                cells: vec![Cell::text("Name"), Cell::text("12 KB")],
                detail: None,
                depth: 0,
                expander: Expander::None,
                actions: vec![],
                tone: Tone::Normal
            }
        );
        let name = String::from("Report");
        let row = Row::new(
            String::from("r2"),
            [
                Cell::from(&name),
                Cell::text("Failed").tone(Tone::Danger).icon("info"),
                Cell::file(4),
                Cell::image("images/a.png"),
                Cell::check(true).tone(Tone::Dim),
                Cell::progress(0.5),
                Cell::Empty,
            ],
        )
        .detail("Second line")
        .depth(2)
        .expander(Expander::Collapsed)
        .action("open", "eye", "Open")
        .tone(Tone::Dim);
        assert_eq!(
            row.cells,
            [
                Cell::Text(CellText {
                    text: "Report".into(),
                    tone: Tone::Normal,
                    icon: None
                }),
                Cell::Text(CellText {
                    text: "Failed".into(),
                    tone: Tone::Danger,
                    icon: Some("info".into())
                }),
                Cell::File(4),
                Cell::Image(ImageSrc::File("images/a.png".into())),
                Cell::Check(true),
                Cell::Progress(0.5),
                Cell::Empty,
            ]
        );
        assert_eq!(
            (row.detail.as_deref(), row.depth, row.expander, row.tone),
            (Some("Second line"), 2, Expander::Collapsed, Tone::Dim)
        );
        assert_eq!(
            row.actions,
            [RowAction {
                id: "open".into(),
                icon: "eye".into(),
                label: "Open".into()
            }]
        );
        assert_eq!(Row::new("r3", [name.clone()]).cells, [Cell::text("Report")]);
    }

    #[test]
    fn submit_carries_the_text_of_the_field() {
        let mut fields = Fields::default();
        let render = |value: &str| flatten(col([text_input(value).key("f")])).0;
        fields.rendered(&render(""));
        use wire::PageEvent;
        assert_eq!(
            translate(&mut fields, PageEvent::Input(("f".into(), "hel".into()))),
            Event::Input {
                key: "f".into(),
                text: "hel".into()
            }
        );
        // A render with the same value keeps what the user typed.
        fields.rendered(&render(""));
        assert_eq!(
            translate(&mut fields, PageEvent::Submit("f".into())),
            Event::Submit {
                key: "f".into(),
                text: "hel".into()
            }
        );
        // A render that changes the value replaces it.
        fields.rendered(&render("new"));
        assert_eq!(fields.text("f"), "new");
        // A field that left the page is forgotten.
        fields.rendered(&flatten(col([])).0);
        assert_eq!(fields.text("f"), "");
    }

    #[test]
    fn every_event_reaches_the_page_in_its_own_words() {
        use crate::bindings::tessoa::plugin::files::Error as FilesError;
        use wire::{ExpandEvent, PageEvent, RowActionEvent, RowCheckEvent, SortEvent};
        let s = |t: &str| t.to_string();
        let day = Date::new(2026, 10, 20);
        let response = Response {
            status: 200,
            headers: vec![(s("Content-Type"), s("text/plain"))],
            body: b"hello".to_vec(),
            url: s("https://example.com/"),
        };
        let cases = [
            (
                PageEvent::Choose((s("fmt"), s("us"))),
                Event::Choose {
                    key: s("fmt"),
                    option: s("us"),
                },
            ),
            (
                PageEvent::Value((s("vol"), 0.75)),
                Event::Value {
                    key: s("vol"),
                    value: 0.75,
                },
            ),
            (
                PageEvent::Color((s("c"), 0x00aaff)),
                Event::Color {
                    key: s("c"),
                    color: 0x00aaff,
                },
            ),
            (
                PageEvent::Date((s("d"), Some(day))),
                Event::Date {
                    key: s("d"),
                    date: Some(day),
                },
            ),
            (
                PageEvent::Date((s("d"), None)),
                Event::Date {
                    key: s("d"),
                    date: None,
                },
            ),
            (
                PageEvent::Split((s("s"), 0.5)),
                Event::Split {
                    key: s("s"),
                    ratio: 0.5,
                },
            ),
            (
                PageEvent::Link((s("r"), s("more"))),
                Event::Link {
                    key: s("r"),
                    action: s("more"),
                },
            ),
            (PageEvent::Dismiss(s("p")), Event::Dismiss { key: s("p") }),
            (
                PageEvent::Select((s("results"), vec![s("r1"), s("r6003")])),
                Event::Select {
                    list: s("results"),
                    ids: vec![s("r1"), s("r6003")],
                },
            ),
            (
                PageEvent::Activate((s("results"), s("r12"))),
                Event::Activate {
                    list: s("results"),
                    row: s("r12"),
                },
            ),
            (
                PageEvent::Expand(ExpandEvent {
                    list: s("outline"),
                    row: s("n3"),
                    open: true,
                }),
                Event::Expand {
                    list: s("outline"),
                    row: s("n3"),
                    open: true,
                },
            ),
            (
                PageEvent::Sort(SortEvent {
                    list: s("t"),
                    column: s("size"),
                    descending: true,
                }),
                Event::Sort {
                    list: s("t"),
                    column: s("size"),
                    descending: true,
                },
            ),
            (
                PageEvent::RowCheck(RowCheckEvent {
                    list: s("t"),
                    row: s("r1"),
                    column: s("done"),
                    checked: false,
                }),
                Event::RowCheck {
                    list: s("t"),
                    row: s("r1"),
                    column: s("done"),
                    checked: false,
                },
            ),
            (
                PageEvent::RowAction(RowActionEvent {
                    list: s("t"),
                    row: s("r1"),
                    action: s("delete"),
                }),
                Event::RowAction {
                    list: s("t"),
                    row: s("r1"),
                    action: s("delete"),
                },
            ),
            (
                PageEvent::PickedFiles((4, vec![77, 78])),
                Event::PickedFiles {
                    id: 4,
                    files: vec![77, 78],
                },
            ),
            (
                PageEvent::PickedFolder((5, Some(9))),
                Event::PickedFolder { id: 5, folder: Some(9) },
            ),
            (
                PageEvent::Created((6, Ok(CreateOutcome::Created(88)))),
                Event::Created {
                    id: 6,
                    outcome: Ok(CreateOutcome::Created(88)),
                },
            ),
            (
                PageEvent::Created((7, Err(FilesError::Invalid(s("..: no"))))),
                Event::Created {
                    id: 7,
                    outcome: Err(FilesError::Invalid(s("..: no"))),
                },
            ),
            (
                PageEvent::Fetched((8, Ok(response.clone()))),
                Event::Fetched {
                    id: 8,
                    response: Ok(response.clone()),
                },
            ),
            (
                PageEvent::Toggle((s("adv"), true)),
                Event::Toggle {
                    key: s("adv"),
                    checked: true,
                },
            ),
        ];
        let mut fields = Fields::default();
        for (wire_event, event) in cases {
            assert_eq!(translate(&mut fields, wire_event), event);
        }
        assert_eq!(response.text(), "hello");
        assert_eq!(response.header("content-type"), Some("text/plain"));
        assert_eq!(response.header("etag"), None);
    }

    /// A page that shows how many rows its list has, describes them on
    /// request, and writes down what it heard.
    struct Lister {
        count: usize,
        heard: Vec<String>,
    }

    impl Page for Lister {
        fn open(_page: PageId, _state: Option<Vec<u8>>, args: Option<Vec<u8>>) -> Self {
            let count = args
                .and_then(|a| String::from_utf8(a).ok())
                .and_then(|a| a.parse().ok())
                .unwrap_or(0);
            Lister {
                count,
                heard: Vec::new(),
            }
        }

        fn event(&mut self, event: Event) -> Result<(), String> {
            match event {
                Event::TaskDone { task: 13, .. } => Err("task 13 is unlucky".into()),
                other => {
                    self.heard.push(format!("{other:?}"));
                    Ok(())
                }
            }
        }

        fn render(&self) -> Node {
            col([
                list(self.count).key("items"),
                table(2, [column("a", "A")]).key("short"),
                list(4),
                text(self.heard.join("; ")).key("heard"),
            ])
        }

        // Every row from `start` on, however many were asked for: the SDK
        // drops the rest.
        fn rows(&self, list: &str, start: usize, _count: usize) -> Vec<Row> {
            match list {
                "items" => (start..self.count)
                    .map(|i| Row::new(format!("r{i}"), [format!("item {i}")]))
                    .collect(),
                _ => Vec::new(),
            }
        }
    }

    struct Blank;

    impl Page for Blank {
        fn open(_page: PageId, _state: Option<Vec<u8>>, _args: Option<Vec<u8>>) -> Self {
            Blank
        }

        fn render(&self) -> Node {
            col([])
        }
    }

    fn open_page(page: PageId, args: &str) {
        __private::open(
            page,
            "lister",
            None,
            Some(args.as_bytes().to_vec()),
            &[
                ("lister", __private::make::<Lister> as __private::Make),
                ("blank", __private::make::<Blank> as __private::Make),
            ],
        )
        .unwrap();
    }

    #[test]
    fn rows_come_from_the_page_for_the_lists_it_rendered() {
        assert_eq!(
            __private::rows(41, "items", 0, 3).unwrap_err(),
            "page 41 is not open",
            "an unknown page is named first"
        );
        open_page(41, "100");
        assert_eq!(
            __private::rows(41, "items", 0, 3).unwrap_err(),
            "no list named \"items\" on page 41",
            "before the first render the page has no lists"
        );
        render_quietly(41);
        let ids = |rows: Vec<Row>| rows.into_iter().map(|r| r.id).collect::<Vec<_>>();
        assert_eq!(ids(__private::rows(41, "items", 98, 5).unwrap()), ["r98", "r99"]);
        assert_eq!(
            ids(__private::rows(41, "items", 10, 3).unwrap()),
            ["r10", "r11", "r12"],
            "rows beyond the count asked for are dropped"
        );
        assert_eq!(
            __private::row_ids(41, "items", 0, 4).unwrap(),
            ["r0", "r1", "r2", "r3"],
            "the ids default to the rows' ids"
        );
        assert!(__private::rows(41, "items", 100, 5).unwrap().is_empty(), "past the end");
        assert_eq!(
            __private::rows(41, "short", 0, 2).unwrap_err(),
            "Page::rows described nothing from row 0 of list \"short\", which has 2 rows; a page with a list, a table or a tree describes its rows in Page::rows"
        );
        assert_eq!(
            __private::row_ids(41, "nope", 0, 2).unwrap_err(),
            "no list named \"nope\" on page 41",
            "a list without a key, or one the page does not have, has no rows"
        );
        __private::close(41);
        assert!(__private::row_ids(41, "items", 0, 1).unwrap_err().contains("not open"));
    }

    #[test]
    fn a_page_that_shows_no_list_has_no_rows() {
        __private::open(
            42,
            "blank",
            None,
            None,
            &[("blank", __private::make::<Blank> as __private::Make)],
        )
        .unwrap();
        render_quietly(42);
        assert_eq!(
            __private::rows(42, "results", 0, 9).unwrap_err(),
            "no list named \"results\" on page 42"
        );
        __private::close(42);
    }

    /// Counts the plugin events the commands' handler heard.
    struct Commands;

    thread_local! {
        static HEARD: RefCell<Vec<crate::PluginEvent>> = const { RefCell::new(Vec::new()) };
    }

    impl crate::Plugin for Commands {
        fn run_command(_command_id: &str) -> Result<(), String> {
            Ok(())
        }

        fn on_event(event: crate::PluginEvent) {
            HEARD.with(|h| h.borrow_mut().push(event));
        }
    }

    // The forms of `export_pages!` the probes do not use: what follows only
    // has to compile.
    crate::export_pages!(Blank => "blank");
    crate::export_pages!(Blank => "blank", Lister => "lister",);
    crate::export_pages!(Lister => "lister"; commands: Commands);

    #[test]
    fn plugin_events_reach_the_commands_and_every_page() {
        open_page(43, "0");
        open_page(44, "0");
        let done = crate::PluginEvent::TaskDone {
            task: 7,
            outcome: TaskOutcome::Done(b"ok".to_vec()),
        };
        assert!(deliver_to_pages::<Commands>(done.clone()).is_empty());
        assert_eq!(HEARD.with(|h| h.borrow().clone()), [done]);
        for page in [43, 44] {
            let (tree, _) = render_quietly(page);
            let heard = tree
                .nodes
                .iter()
                .find_map(|n| match &n.kind {
                    NodeKind::Text(t) if n.key.as_deref() == Some("heard") => Some(t.text.clone()),
                    _ => None,
                })
                .unwrap();
            assert_eq!(heard, "TaskDone { task: 7, outcome: TaskOutcome::Done([111, 107]) }");
        }
        // A page that answers with an error is reported, and the others
        // still hear the event.
        let unlucky = crate::PluginEvent::TaskDone {
            task: 13,
            outcome: TaskOutcome::Cancelled,
        };
        assert_eq!(
            deliver_to_pages::<Commands>(unlucky),
            ["page 43: task 13 is unlucky", "page 44: task 13 is unlucky"]
        );
        let changed = crate::PluginEvent::SettingsChanged {
            ids: vec!["format".into()],
        };
        assert!(deliver_to_pages::<__private::NoCommands>(changed).is_empty());
        let (tree, _) = render_quietly(43);
        assert!(tree.nodes.iter().any(
            |n| matches!(&n.kind, NodeKind::Text(t) if t.text.ends_with("SettingsChanged { ids: [\"format\"] }"))
        ));
        event_quietly(43, wire::PageEvent::Click("x".into())).unwrap();
        __private::close(43);
        __private::close(44);
    }
}
