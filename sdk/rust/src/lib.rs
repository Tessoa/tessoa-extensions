//! The Rust SDK for tessoa plugins.
//!
//! A plugin is a `cdylib` crate compiled for `wasm32-wasip2`. It implements
//! [`Plugin`] for a type and exports that type:
//!
//! ```ignore
//! use tessoa_plugin_sdk::prelude::*;
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn run_command(command_id: &str) -> Result<(), String> {
//!         match command_id {
//!             "say-hello" => {
//!                 ui::notify("Hello", "The plugin is running.");
//!                 Ok(())
//!             }
//!             other => Err(format!("unknown command {other}")),
//!         }
//!     }
//! }
//!
//! tessoa_plugin_sdk::export!(MyPlugin);
//! ```
//!
//! A plugin with pages (whole tabs it describes and tessoa draws) turns on
//! the SDK's `pages` feature in its `Cargo.toml`:
//!
//! ```toml
//! tessoa-plugin-sdk = { path = "...", features = ["pages"] }
//! ```
//!
//! implements `pages::Page` for each kind of page, and exports them with
//! `export_pages!` instead of `export!`; see the `pages` module.
//!
//! Work that takes longer than a second runs as a task: a type that
//! implements [`Task`], listed in `export!` (or `export_pages!`) and started
//! with [`tasks::start`]. The columns `[[columns]]` in `plugin.toml` adds to
//! tessoa's file list get their values from a type that implements
//! [`Columns`]:
//!
//! ```ignore
//! tessoa_plugin_sdk::export!(MyPlugin; tasks: [("hash", Hash)]; columns: Dims);
//! ```
//!
//! The modules [`log`], [`ui`], [`files`], [`refs`], [`storage`],
//! [`clipboard`], [`settings`], [`images`], [`net`], [`tasks`] and [`task`],
//! and `pages_host` with the `pages` feature, are the host interface with
//! Rust-friendly signatures. The generated bindings they wrap are available
//! as [`raw`] for anything the wrappers do not cover.
//!
//! Rules the host enforces, so that the plugin does not have to:
//!
//! - Handles from [`files::selected`] are valid for the current command
//!   only, and in a task the command started until the task ends.
//! - [`files::read`] needs the `files.read-selected` permission and
//!   [`files::rename`] needs `files.rename-selected`; both are declared in
//!   `plugin.toml`. The other gated functions say which permission they
//!   need.
//! - File references from [`files::keep`] stay valid across commands and
//!   restarts; the plugin sees numbers, never paths.
//! - Dialogs, file pickers, new files and connections wait for their answer
//!   in a command or a task. The host keeps running meanwhile. Pages never
//!   wait: they ask with `pages::confirm`, and call the `_start` twin of a
//!   function that would wait, whose answer arrives as an event of the page.

/// Bindings generated from `wit/tessoa-plugin.wit`.
///
/// The `pages` feature picks the world with pages. Without it a plugin
/// exports no page functions at all: they would cost a plugin without pages
/// about 11 KB of component, the conversions of the page types, and the
/// runner could no longer tell a plugin that forgot its pages from one that
/// has none.
#[doc(hidden)]
pub mod bindings {
    #[cfg(not(feature = "pages"))]
    wit_bindgen::generate!({
        path: "../wit",
        world: "tessoa-plugin",
        pub_export_macro: true,
        default_bindings_module: "tessoa_plugin_sdk::bindings",
        export_macro_name: "export_plugin",
        additional_derives: [PartialEq],
    });

    #[cfg(feature = "pages")]
    wit_bindgen::generate!({
        path: "../wit",
        world: "tessoa-plugin-pages",
        pub_export_macro: true,
        default_bindings_module: "tessoa_plugin_sdk::bindings",
        export_macro_name: "export_plugin",
        additional_derives: [PartialEq],
    });
}

/// The generated host interface, unchanged, the types of the interfaces
/// every plugin exports, and with the `pages` feature the types of the page
/// interface.
pub mod raw {
    #[cfg(feature = "pages")]
    pub use crate::bindings::exports::tessoa::plugin::pages;
    pub use crate::bindings::exports::tessoa::plugin::{columns, plugin, task_runner};
    #[cfg(feature = "pages")]
    pub use crate::bindings::tessoa::plugin::pages_host;
    pub use crate::bindings::tessoa::plugin::{
        clipboard, files, images, log, net, refs, settings, storage, task, tasks, ui,
    };
}

#[cfg(feature = "pages")]
pub mod pages;
#[cfg(feature = "pages")]
mod tree_model;

pub use crate::bindings::exports::tessoa::plugin::columns::{ColumnFile, ColumnValue};

/// What a plugin implements.
pub trait Plugin {
    /// Run the command with the given id, as declared in `plugin.toml`. An
    /// `Err` is written to the plugin log and shown to the user.
    fn run_command(command_id: &str) -> Result<(), String>;

    /// Something that concerns the plugin as a whole: a task reported how
    /// far it is or ended, or the user changed the plugin's settings. Called
    /// in the plugin's own instance, the one that runs its commands and
    /// pages, never in a worker. It does not wait: [`ui::confirm`] answers
    /// false here, and the functions that wait for the user or the network
    /// are refused, and so are their `_start` twins. It can start and cancel
    /// tasks, read settings, store, log and notify.
    ///
    /// With pages, every open page hears the same through
    /// `pages::Page::event`, after this, and every page renders again.
    fn on_event(event: PluginEvent) {
        let _ = event;
    }
}

/// What [`Plugin::on_event`] hears. A plugin with pages hears the same in
/// every open page, as the page events of the same names.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum PluginEvent {
    /// A task reported how far it is: `done` of `total`, `total` 0 when the
    /// total is not known, and its line of text. At most 5 times a second
    /// per task; in between, only the newest report waits.
    TaskProgress {
        task: u64,
        done: f64,
        total: f64,
        text: String,
    },
    /// A task ended: with the bytes its [`Task::run`] returned, with the
    /// reason it failed, or cancelled. Every task the plugin started ends
    /// this way, except when the plugin stops or is reloaded.
    TaskDone { task: u64, outcome: tasks::TaskOutcome },
    /// The user changed settings of the plugin: their ids. The [`settings`]
    /// functions read the new values already.
    SettingsChanged { ids: Vec<String> },
}

/// Work that runs in a worker: another instance of the plugin, started for
/// the task, while the plugin's commands and pages go on working.
///
/// ```ignore
/// struct Hash;
///
/// impl Task for Hash {
///     fn run(&mut self, input: Vec<u8>, task: &TaskCtx) -> Result<Vec<u8>, String> {
///         let files = decode(&input)?;
///         for (i, file) in files.iter().enumerate() {
///             if task.cancelled() {
///                 // A cancelled task ends as cancelled, whatever it returns.
///                 return Ok(Vec::new());
///             }
///             hash_one(*file)?;
///             task.progress((i + 1) as f64, files.len() as f64, &format!("{} of {}", i + 1, files.len()));
///         }
///         Ok(Vec::new())
///     }
/// }
///
/// tessoa_plugin_sdk::export!(MyPlugin; tasks: [("hash", Hash)]);
/// ```
///
/// A command or a page starts it with [`tasks::start`], naming its kind;
/// the expression `export!` pairs with the kind makes a new value for every
/// run. What the worker has to know goes into the input: it shares the
/// plugin's storage, file references, settings, images and picked folders,
/// but not the variables of the page or the command that started it.
///
/// A worker may wait: [`files::pick_files`], [`files::pick_folder`],
/// [`files::create_file`], [`net::fetch`] and [`ui::confirm`] wait for their
/// answer here, and a task a command started reads the command's files with
/// [`files::read`]. It cannot touch pages, start other tasks, rename files or
/// offer web addresses. It has no time limit, but a worker that does not
/// call the host for 60 seconds is stopped: report progress after every
/// item.
pub trait Task {
    /// Run the task. The result, at most 1 MiB, or the error reaches the
    /// plugin as [`PluginEvent::TaskDone`]; a larger result fails the task.
    /// A task that was cancelled ends as cancelled, whatever this returns.
    fn run(&mut self, input: Vec<u8>, task: &TaskCtx) -> Result<Vec<u8>, String>;
}

/// What a running task reports through, and asks whether it was cancelled.
pub struct TaskCtx {
    _private: (),
}

impl TaskCtx {
    /// Report how far the task is: `done` of `total`, and a line of text for
    /// tessoa's transfer panel; `total` 0 means the total is not known.
    /// tessoa shows at most 10 reports a second and the plugin hears at most
    /// 5; text longer than 1 KiB is cut.
    pub fn progress(&self, done: f64, total: f64, text: &str) {
        task::progress(done, total, text)
    }

    /// Whether the task was cancelled: by the user in tessoa's transfer
    /// panel, or by the plugin with [`tasks::cancel`]. Look between items,
    /// clean up and return: a worker that has not returned 2 seconds after
    /// the task was cancelled is stopped.
    pub fn cancelled(&self) -> bool {
        task::cancelled()
    }

    #[doc(hidden)]
    pub fn __new() -> TaskCtx {
        TaskCtx { _private: () }
    }
}

/// The values of the columns `[[columns]]` in `plugin.toml` declares.
///
/// ```ignore
/// struct Dims;
///
/// impl Columns for Dims {
///     fn value(&mut self, column: &str, file: &ColumnFile) -> ColumnValue {
///         match (column, image_size(file.handle)) {
///             ("dims", Some((w, h))) => ColumnValue::sorted_text(format!("{w}×{h}"), f64::from(w * h)),
///             _ => ColumnValue::Empty,
///         }
///     }
/// }
///
/// tessoa_plugin_sdk::export!(MyPlugin; columns: Dims);
/// ```
///
/// tessoa asks only for the files on screen, at most 64 at a time, and
/// keeps the values until a file changes; the SDK asks [`Columns::value`]
/// for each file in turn. The values are computed in the plugin's column
/// worker, an instance of its own, so a slow column never holds up a page.
/// The expression `export!` gives makes the value once per worker, which can
/// keep what it learnt for the next files; the worker is let go after five
/// minutes without a request and after a crash, and the next one starts
/// afresh.
///
/// Reading a file needs the `files.read-listed` permission: [`files::read`]
/// with `file.handle`, which is valid during the request only.
/// `files.read-selected` does not count here. A column cannot ask the user
/// anything: [`ui::confirm`] answers false. A request has the limits of a
/// command: 15 seconds without calling the host, 120 seconds in all.
pub trait Columns {
    /// The value of `column` for `file`. Text longer than 1 KiB is cut, and
    /// a number that is not finite counts as empty.
    fn value(&mut self, column: &str, file: &ColumnFile) -> ColumnValue;
}

impl ColumnValue {
    /// Text, sorted the way tessoa sorts names.
    pub fn text(text: impl Into<String>) -> ColumnValue {
        ColumnValue::Text(text.into())
    }

    /// Text that sorts by a number, such as `1920×1080` by its pixel count.
    pub fn sorted_text(text: impl Into<String>, by: f64) -> ColumnValue {
        ColumnValue::SortedText((text.into(), by))
    }
}

/// Export a [`Plugin`] implementation as the plugin's entry point, with the
/// plugin's tasks and columns when it has them. A plugin with pages uses
/// `export_pages!` instead.
///
/// ```ignore
/// tessoa_plugin_sdk::export!(MyPlugin);
/// tessoa_plugin_sdk::export!(MyPlugin; tasks: [("hash", Hash), ("thumbs", Thumbs::new(64))]);
/// tessoa_plugin_sdk::export!(MyPlugin; columns: Dims);
/// tessoa_plugin_sdk::export!(MyPlugin; tasks: [("hash", Hash)]; columns: Dims);
/// ```
///
/// `tasks` pairs each kind [`tasks::start`] names with an expression that
/// makes a [`Task`], evaluated for every run; `columns` is an expression that
/// makes the [`Columns`]. A task of a kind that is not listed fails, and a
/// plugin without columns answers with empty values.
///
/// The export is made in the WebAssembly build only. A build for the computer
/// itself, which `cargo test` makes for unit tests, has no use for it, and on
/// macOS it would not link: the exported names contain `#`, which the macOS
/// linker reads as the start of a comment in its list of exports.
#[macro_export]
macro_rules! export {
    ($plugin:ty $(; $($section:tt)+)?) => {
        $crate::__export!(@plugin [$plugin] tasks [] columns [] ; $($($section)+)?);
    };
}

// Reads the sections of `export!` one at a time, in any order, then makes
// the exports.
#[doc(hidden)]
#[macro_export]
macro_rules! __export {
    (@plugin [$plugin:ty] tasks [$($tasks:tt)*] columns [$($columns:tt)*] ;) => {
        const _: () = {
            struct TessoaExportedPlugin;

            impl $crate::bindings::exports::tessoa::plugin::plugin::Guest for TessoaExportedPlugin {
                fn run_command(command_id: String) -> Result<(), String> {
                    <$plugin as $crate::Plugin>::run_command(&command_id)
                }

                fn on_event(ev: $crate::bindings::exports::tessoa::plugin::plugin::PluginEvent) {
                    <$plugin as $crate::Plugin>::on_event($crate::__exports::plugin_event(ev))
                }
            }

            $crate::__tasks!(TessoaExportedPlugin; $($tasks)*);
            $crate::__columns!(TessoaExportedPlugin; $($columns)*);
            $crate::__no_pages!(TessoaExportedPlugin);

            #[cfg(target_arch = "wasm32")]
            $crate::bindings::export_plugin!(TessoaExportedPlugin with_types_in $crate::bindings);

            // Without the export nothing would call the plugin, and its code
            // would be reported as unused.
            #[cfg(not(target_arch = "wasm32"))]
            let _ = (
                <TessoaExportedPlugin as $crate::bindings::exports::tessoa::plugin::plugin::Guest>::run_command,
                <TessoaExportedPlugin as $crate::bindings::exports::tessoa::plugin::plugin::Guest>::on_event,
                <TessoaExportedPlugin as $crate::bindings::exports::tessoa::plugin::task_runner::Guest>::run,
                <TessoaExportedPlugin as $crate::bindings::exports::tessoa::plugin::columns::Guest>::values,
            );
        };
    };
    (@plugin [$plugin:ty] tasks [] columns [$($columns:tt)*] ; tasks: [$($task:tt)*] $(; $($rest:tt)*)?) => {
        $crate::__export!(@plugin [$plugin] tasks [$($task)*] columns [$($columns)*] ; $($($rest)*)?);
    };
    (@plugin [$plugin:ty] tasks [$($tasks:tt)*] columns [] ; columns: $columns:expr $(; $($rest:tt)*)?) => {
        $crate::__export!(@plugin [$plugin] tasks [$($tasks)*] columns [$columns] ; $($($rest)*)?);
    };
}

// The worker's entry point: the task of each listed kind, made anew for
// every run. A plugin without tasks says it has none.
#[doc(hidden)]
#[macro_export]
macro_rules! __tasks {
    ($exports:ident;) => {
        impl $crate::bindings::exports::tessoa::plugin::task_runner::Guest for $exports {
            fn run(kind: String, _input: Vec<u8>) -> Result<Vec<u8>, String> {
                Err($crate::__exports::no_task(&kind, &[]))
            }
        }
    };
    ($exports:ident; $(($kind:literal, $task:expr)),+ $(,)?) => {
        impl $crate::bindings::exports::tessoa::plugin::task_runner::Guest for $exports {
            fn run(kind: String, input: Vec<u8>) -> Result<Vec<u8>, String> {
                let ctx = $crate::TaskCtx::__new();
                match kind.as_str() {
                    $($kind => $crate::Task::run(&mut $task, input, &ctx),)+
                    _ => Err($crate::__exports::no_task(&kind, &[$($kind),+])),
                }
            }
        }
    };
}

// The column worker's entry point. A plugin without columns answers with an
// empty value for every file.
#[doc(hidden)]
#[macro_export]
macro_rules! __columns {
    ($exports:ident;) => {
        impl $crate::bindings::exports::tessoa::plugin::columns::Guest for $exports {
            fn values(_column: String, files: Vec<$crate::ColumnFile>) -> Vec<$crate::ColumnValue> {
                $crate::__exports::no_values(&files)
            }
        }
    };
    ($exports:ident; $columns:expr) => {
        impl $crate::bindings::exports::tessoa::plugin::columns::Guest for $exports {
            fn values(column: String, files: Vec<$crate::ColumnFile>) -> Vec<$crate::ColumnValue> {
                $crate::__exports::column_values(&column, &files, || $columns)
            }
        }
    };
}

/// The machinery behind the exports of tasks, columns and plugin events. Not
/// for plugins.
#[doc(hidden)]
pub mod __exports {
    use std::any::Any;
    use std::cell::RefCell;

    use crate::bindings::exports::tessoa::plugin::plugin::PluginEvent as WirePluginEvent;
    use crate::{ColumnFile, ColumnValue, Columns, PluginEvent};

    /// `task-runner.run` for a kind of task the plugin does not list.
    pub fn no_task(kind: &str, known: &[&str]) -> String {
        if known.is_empty() {
            format!("no task named {kind:?}: this plugin defines no tasks")
        } else {
            format!("no task named {kind:?}: the export lists {known:?}")
        }
    }

    /// `columns.values` of a plugin without columns: one empty value per file.
    pub fn no_values(files: &[ColumnFile]) -> Vec<ColumnValue> {
        files.iter().map(|_| ColumnValue::Empty).collect()
    }

    thread_local! {
        /// The plugin's columns, made by the first request this worker
        /// answers.
        static COLUMNS: RefCell<Option<Box<dyn Any>>> = const { RefCell::new(None) };
    }

    /// `columns.values` of a plugin with columns: one value per file, in
    /// order, from the same value of the plugin's type for as long as the
    /// worker lives.
    pub fn column_values<C: Columns + 'static>(
        column: &str,
        files: &[ColumnFile],
        make: impl FnOnce() -> C,
    ) -> Vec<ColumnValue> {
        // Taken out while it works, so that the cell is never borrowed
        // across a call into the plugin's code.
        let kept = COLUMNS
            .with(|c| c.borrow_mut().take())
            .and_then(|b| b.downcast::<C>().ok());
        let mut columns = kept.unwrap_or_else(|| Box::new(make()));
        let values = files.iter().map(|f| columns.value(column, f)).collect();
        COLUMNS.with(|c| *c.borrow_mut() = Some(columns as Box<dyn Any>));
        values
    }

    /// The event as [`crate::Plugin::on_event`] hears it.
    pub fn plugin_event(ev: WirePluginEvent) -> PluginEvent {
        match ev {
            WirePluginEvent::TaskProgress(p) => PluginEvent::TaskProgress {
                task: p.task,
                done: p.done,
                total: p.total,
                text: p.text,
            },
            WirePluginEvent::TaskDone((task, outcome)) => PluginEvent::TaskDone { task, outcome },
            WirePluginEvent::SettingsChanged(ids) => PluginEvent::SettingsChanged { ids },
        }
    }
}

// A plugin without pages, built where the `pages` feature is on: in a cargo
// workspace one member with pages turns the feature on for every member that
// is built with it. Its page functions answer that there are none.
#[cfg(feature = "pages")]
#[doc(hidden)]
#[macro_export]
macro_rules! __no_pages {
    ($exports:ident) => {
        impl $crate::bindings::exports::tessoa::plugin::pages::Guest for $exports {
            fn open(_page: u64, _kind: String, _state: Option<Vec<u8>>, _args: Option<Vec<u8>>) -> Result<(), String> {
                Err($crate::pages::__private::no_pages())
            }

            fn event(
                _page: u64,
                _ev: $crate::bindings::exports::tessoa::plugin::pages::PageEvent,
            ) -> Result<(), String> {
                Err($crate::pages::__private::no_pages())
            }

            fn render(_page: u64) -> Result<$crate::bindings::exports::tessoa::plugin::pages::Tree, String> {
                Err($crate::pages::__private::no_pages())
            }

            fn close(_page: u64) {}

            fn rows(
                _page: u64,
                _list: String,
                _start: u32,
                _count: u32,
            ) -> Result<Vec<$crate::bindings::exports::tessoa::plugin::pages::Row>, String> {
                Err($crate::pages::__private::no_pages())
            }

            fn row_ids(_page: u64, _list: String, _start: u32, _count: u32) -> Result<Vec<String>, String> {
                Err($crate::pages::__private::no_pages())
            }
        }
    };
}

#[cfg(not(feature = "pages"))]
#[doc(hidden)]
#[macro_export]
macro_rules! __no_pages {
    ($exports:ident) => {};
}

// Without the `pages` feature, `export_pages!` says what is missing.
#[cfg(not(feature = "pages"))]
#[doc(hidden)]
#[macro_export]
macro_rules! export_pages {
    ($($anything:tt)*) => {
        compile_error!(
            "export_pages! needs the `pages` feature of the SDK: in Cargo.toml, tessoa-plugin-sdk = { path = \"...\", features = [\"pages\"] }"
        );
    };
}

/// The plugin log. Lines appear in the plugin log panel and the log file.
pub mod log {
    pub use crate::raw::log::Level;

    pub fn log(level: Level, message: &str) {
        crate::raw::log::log(level, message);
    }

    pub fn debug(message: &str) {
        log(Level::Debug, message);
    }

    pub fn info(message: &str) {
        log(Level::Info, message);
    }

    pub fn warn(message: &str) {
        log(Level::Warn, message);
    }

    pub fn error(message: &str) {
        log(Level::Error, message);
    }
}

/// Notifications, dialogs and web addresses, drawn by the host.
pub mod ui {
    /// Show a notification and return at once.
    pub fn notify(title: &str, body: &str) {
        crate::raw::ui::notify(title, body);
    }

    /// Ask a yes/no question with the default button labels, and wait for
    /// the answer. Returns `true` when the user confirmed.
    ///
    /// In a command or a task; a task's question names the plugin. A page,
    /// [`crate::Plugin::on_event`] and a column do not wait: there this
    /// returns false at once. Pages ask with `pages::confirm`.
    pub fn confirm(title: &str, body: &str) -> bool {
        confirm_with(title, body, "OK", "Cancel")
    }

    /// Ask a yes/no question with custom button labels, like [`confirm`].
    pub fn confirm_with(title: &str, body: &str, ok_label: &str, cancel_label: &str) -> bool {
        crate::raw::ui::confirm(title, body, ok_label, cancel_label)
    }

    /// Ask a yes/no question showing one line per item, with the default
    /// button labels, like [`confirm`].
    pub fn confirm_list(title: &str, body: &str, items: &[String]) -> bool {
        confirm_list_with(title, body, items, "OK", "Cancel")
    }

    /// Ask a yes/no question showing one line per item, with custom button
    /// labels, like [`confirm`].
    pub fn confirm_list_with(title: &str, body: &str, items: &[String], ok_label: &str, cancel_label: &str) -> bool {
        crate::raw::ui::confirm_list(title, body, items, ok_label, cancel_label)
    }

    /// The language of the tessoa interface as a short code such as `en`,
    /// `zh`, `ja` or `de`, to pick the plugin's own texts.
    pub fn language() -> String {
        crate::raw::ui::language()
    }

    /// Offer to open a web address in the user's browser. tessoa shows the
    /// whole address and opens it only when the user agrees; the address
    /// need not be declared anywhere. Only `http://` and `https://`
    /// addresses of at most 64 KiB, and only while the plugin handles
    /// something the user just did: a command, or one of the page events
    /// `Click`, `Submit`, `Key`, `Menu`, `Toggle`, `Choose`, `Date`, `Link`,
    /// `Activate`, `Expand`, `Sort`, `RowCheck`, `RowAction`, `DropFiles`,
    /// `DropItem` and `Dialog`. At other times (while the user types or
    /// drags, in a task, in a column) and while the plugin's previous offer
    /// is still on screen, it fails with the reason. `Ok` means the offer is
    /// on screen; whether the user opens the address is not reported. A link
    /// to a web address in rich text asks the same without the page's help.
    pub fn open_url(url: &str) -> Result<(), String> {
        crate::raw::ui::open_url(url)
    }

    /// Open the plugin's page in tessoa's settings, where the settings
    /// `[[settings]]` in `plugin.toml` declares are. Does nothing for a
    /// plugin without settings.
    pub fn open_settings() {
        crate::raw::ui::open_settings()
    }
}

/// The files the user hands to the plugin: the ones selected for the current
/// command, the ones picked in tessoa's file picker, and new files in a
/// folder the user picked.
pub mod files {
    pub use crate::raw::files::{
        CreateOutcome, Error, FileInfo, FileRef, Folder, Handle, RenameOutcome, RenameRequest,
    };

    /// The files selected when the command was triggered. In a task the
    /// command started, the same files, until the task ends.
    pub fn selected() -> Vec<FileInfo> {
        crate::raw::files::selected()
    }

    /// Read up to `max_len` bytes starting at `offset`, at most 16 MiB at a
    /// time. Needs the `files.read-selected` permission; in a column, the
    /// `files.read-listed` permission.
    pub fn read(handle: Handle, offset: u64, max_len: u32) -> Result<Vec<u8>, Error> {
        crate::raw::files::read(handle, offset, max_len)
    }

    /// Read a whole file, with the permission [`read`] needs.
    pub fn read_all(handle: Handle) -> Result<Vec<u8>, Error> {
        const CHUNK: u32 = 4 << 20;
        let mut out = Vec::new();
        loop {
            let chunk = read(handle, out.len() as u64, CHUNK)?;
            let done = chunk.len() < CHUNK as usize;
            out.extend_from_slice(&chunk);
            if done {
                return Ok(out);
            }
        }
    }

    /// Ask the host to rename files. One outcome per request, in order.
    /// Needs the `files.rename-selected` permission. In a command only.
    pub fn rename(items: &[RenameRequest]) -> Result<Vec<RenameOutcome>, Error> {
        crate::raw::files::rename(items)
    }

    /// Keep a selected file beyond the command: the reference stays valid
    /// after the command returns and after tessoa restarts.
    /// Needs the `files.keep` permission.
    pub fn keep(handle: Handle) -> Result<FileRef, Error> {
        crate::raw::files::keep(handle)
    }

    /// Let the user pick files in tessoa's file picker, and wait for the
    /// choice. `extensions` limits the files offered, written without the
    /// dot, such as `"csv"`; empty offers every file. Returns references to
    /// the picked files, which [`crate::refs::read`] reads; empty when the
    /// user cancelled. With `multiple` false, at most one.
    ///
    /// In a command or a task. A page cannot wait: it calls
    /// [`pick_files_start`], and the choice arrives as the page event
    /// `PickedFiles`.
    pub fn pick_files(title: &str, extensions: &[&str], multiple: bool) -> Result<Vec<FileRef>, Error> {
        crate::raw::files::pick_files(title, &strings(extensions), multiple)
    }

    /// [`pick_files`] for a page: returns at once with a call id, and the
    /// choice arrives as the page event `PickedFiles` with that id. Only a
    /// page can call it.
    pub fn pick_files_start(title: &str, extensions: &[&str], multiple: bool) -> Result<u64, Error> {
        crate::raw::files::pick_files_start(title, &strings(extensions), multiple)
    }

    /// Let the user pick a folder, and wait for the choice; `None` when the
    /// user cancelled. The plugin can create files in it with
    /// [`create_file`], and nothing else, until the plugin stops or tessoa
    /// restarts, or until [`release_folder`]. It sees the folder's name
    /// ([`folder_name`]), never its path.
    ///
    /// In a command or a task. A page calls [`pick_folder_start`], and the
    /// choice arrives as the page event `PickedFolder`.
    pub fn pick_folder(title: &str) -> Result<Option<Folder>, Error> {
        crate::raw::files::pick_folder(title)
    }

    /// [`pick_folder`] for a page: returns at once with a call id, and the
    /// choice arrives as the page event `PickedFolder` with that id. Only a
    /// page can call it.
    pub fn pick_folder_start(title: &str) -> Result<u64, Error> {
        crate::raw::files::pick_folder_start(title)
    }

    /// The name of a picked folder, without the folders above it.
    pub fn folder_name(folder: Folder) -> Result<String, Error> {
        crate::raw::files::folder_name(folder)
    }

    /// Forget a picked folder; creating files in it fails afterwards.
    pub fn release_folder(folder: Folder) {
        crate::raw::files::release_folder(folder)
    }

    /// Create a file in a picked folder, and wait until it is written.
    /// `path` is relative to the folder and written with `/`, such as
    /// `"report.csv"` or `"2026/report.csv"`; missing folders on the way are
    /// created. A path with `..`, a drive, a backslash, a name Windows
    /// reserves (such as `con`) or a name that ends in a dot or a space is
    /// refused as [`Error::Invalid`]. At most 64 MiB. An existing file is
    /// never replaced: when the name is taken, tessoa asks the user whether
    /// to keep both, with a number added to the new name, or to skip the new
    /// file. The new file is in tessoa's undo and activity log, and
    /// [`CreateOutcome::Created`] carries a reference to it. Needs the
    /// `files.create` permission.
    ///
    /// In a command or a task. A page calls [`create_file_start`], and the
    /// outcome arrives as the page event `Created`.
    pub fn create_file(folder: Folder, path: &str, data: &[u8]) -> Result<CreateOutcome, Error> {
        crate::raw::files::create_file(folder, path, data)
    }

    /// [`create_file`] for a page: returns at once with a call id, and the
    /// outcome arrives as the page event `Created` with that id. Only a page
    /// can call it. Needs the `files.create` permission.
    pub fn create_file_start(folder: Folder, path: &str, data: &[u8]) -> Result<u64, Error> {
        crate::raw::files::create_file_start(folder, path, data)
    }

    fn strings(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }
}

/// Files the host keeps for the plugin, by reference.
pub mod refs {
    pub use crate::raw::files::{Error, FileRef};
    pub use crate::raw::refs::{RefInfo, TransferOutcome};

    /// Describe references. References not issued to this plugin are left out.
    pub fn info(refs: &[FileRef]) -> Vec<RefInfo> {
        crate::raw::refs::info(refs)
    }

    /// Read up to `max_len` bytes of a referenced file, starting at
    /// `offset`, at most 16 MiB at a time: a file the user dropped on a page
    /// or picked, or one the plugin kept or created. Needs the
    /// `files.read-selected` permission.
    pub fn read(file: FileRef, offset: u64, max_len: u32) -> Result<Vec<u8>, Error> {
        crate::raw::refs::read(file, offset, max_len)
    }

    /// Read a whole referenced file. Needs the `files.read-selected`
    /// permission.
    pub fn read_all(file: FileRef) -> Result<Vec<u8>, Error> {
        const CHUNK: u32 = 4 << 20;
        let mut out = Vec::new();
        loop {
            let chunk = read(file, out.len() as u64, CHUNK)?;
            let done = chunk.len() < CHUNK as usize;
            out.extend_from_slice(&chunk);
            if done {
                return Ok(out);
            }
        }
    }

    /// Forget references. The files are not touched.
    pub fn release(refs: &[FileRef]) {
        crate::raw::refs::release(refs)
    }

    /// Let the user pick a folder and copy the files there. Returns the
    /// transfer id; the outcome arrives as a `transfer-done` event of the
    /// page that made the call. Needs the `files.transfer` permission.
    pub fn copy_to_picked(refs: &[FileRef]) -> Result<u64, Error> {
        crate::raw::refs::copy_to_picked(refs)
    }

    /// Like [`copy_to_picked`], moving instead of copying. Needs the
    /// `files.transfer` permission.
    pub fn move_to_picked(refs: &[FileRef]) -> Result<u64, Error> {
        crate::raw::refs::move_to_picked(refs)
    }

    /// Open the file the way a double click in tessoa would. Needs the
    /// `files.open` permission.
    pub fn open(file: FileRef) -> Result<(), Error> {
        crate::raw::refs::open(file)
    }

    /// Show the file in a new tessoa tab. Needs the `files.open` permission.
    pub fn reveal(file: FileRef) -> Result<(), Error> {
        crate::raw::refs::reveal(file)
    }

    /// Move the files to the recycle bin after tessoa asks the user. Returns
    /// the transfer id; the outcome arrives as a `transfer-done` event of the
    /// page that made the call. Needs the `files.delete` permission.
    pub fn trash(refs: &[FileRef]) -> Result<u64, Error> {
        crate::raw::refs::trash(refs)
    }
}

/// The plugin's own storage: up to 1 MiB of keys and values that survive
/// restarts, shared by its commands, pages and workers. No permission needed.
pub mod storage {
    pub use crate::raw::storage::Error;

    /// The value stored under `key`, if any.
    pub fn get(key: &str) -> Option<Vec<u8>> {
        crate::raw::storage::get(key)
    }

    /// Store `value` under `key`, replacing what was there.
    pub fn set(key: &str, value: &[u8]) -> Result<(), Error> {
        crate::raw::storage::set(key, value)
    }

    /// Remove `key`.
    pub fn remove(key: &str) {
        crate::raw::storage::remove(key)
    }

    /// Every key, in byte order.
    pub fn keys() -> Vec<String> {
        crate::raw::storage::keys()
    }
}

/// The system clipboard.
pub mod clipboard {
    pub use crate::raw::files::Error;

    /// Put text on the clipboard. Needs the `clipboard.write` permission.
    pub fn write_text(text: &str) -> Result<(), Error> {
        crate::raw::clipboard::write_text(text)
    }
}

/// The plugin's settings: the options `[[settings]]` in `plugin.toml`
/// declares, which tessoa shows in its settings and keeps. The plugin only
/// reads them, without a round trip, wherever it runs; a setting the user
/// never changed has the default the manifest gives. When the user changes
/// settings, [`crate::Plugin::on_event`] and every open page hear of it and
/// the pages render again, so a page that reads a setting while it renders
/// shows the new value. An id the manifest does not declare, or a setting of
/// another type, reads as the zero value and writes a line to the plugin
/// log. No permission needed.
pub mod settings {
    /// The type of a declared setting: `"flag"`, `"choice"`, `"number"` or
    /// `"text"`; `None` for an id the manifest does not declare.
    pub fn kind(id: &str) -> Option<String> {
        crate::raw::settings::kind(id)
    }

    /// The value of a `flag` setting; false for any other id.
    pub fn flag(id: &str) -> bool {
        crate::raw::settings::flag(id)
    }

    /// The value of a `number` setting, within its minimum and maximum and
    /// on its step; 0 for any other id.
    pub fn number(id: &str) -> f64 {
        crate::raw::settings::number(id)
    }

    /// The value of a `text` setting, or the id of the chosen option of a
    /// `choice` setting; empty for any other id.
    pub fn text(id: &str) -> String {
        crate::raw::settings::text(id)
    }
}

/// Images the plugin makes, for image nodes and image cells of its pages.
/// The bytes go to tessoa once, when the image is added; pages refer to it
/// by number. The numbers are valid until the plugin stops: a plugin that
/// starts again adds its images again, so a page's saved state should not
/// keep them. The plugin's images take at most 64 MiB once decoded. No
/// permission needed.
pub mod images {
    pub use crate::raw::images::Error;

    /// Add an image from the bytes of a PNG, JPEG, GIF (its first frame),
    /// WebP, BMP or SVG file: at most 8 MiB, and at most 4096 by 4096
    /// pixels. Returns its number.
    pub fn add(data: &[u8]) -> Result<u64, Error> {
        crate::raw::images::add(data)
    }

    /// Add an image from pixels: `width` times `height` times 4 bytes of
    /// red, green, blue and alpha, row by row from the top. At most 4096 by
    /// 4096 pixels. Returns its number.
    pub fn add_rgba(width: u32, height: u32, data: &[u8]) -> Result<u64, Error> {
        crate::raw::images::add_rgba(width, height, data)
    }

    /// Remove an image. Nodes and cells that still name it show an empty
    /// frame.
    pub fn remove(id: u64) {
        crate::raw::images::remove(id)
    }
}

/// Connections to the websites `plugin.toml` lists under `[permissions]
/// net`, which tessoa shows the user when the plugin is installed. Nothing
/// else can be reached, and a listed name that leads to this computer or the
/// local network is refused. The runner makes the requests: tessoa does not
/// see what is sent or received.
///
/// - Redirects are followed, at most 5, each to a listed website.
/// - At most 8 MiB out and 32 MiB in: more is [`Error::TooLarge`], never cut.
///   30 seconds unless the request says otherwise, at most 120.
/// - At most 8 requests of the plugin are under way at once; more wait their
///   turn.
/// - No cookies are kept and nothing is cached. The user agent is tessoa's,
///   whatever the request says. The plugin log gets a line per request: the
///   method, the website, the status and the size, never the query or the
///   content.
pub mod net {
    use std::time::Duration;

    pub use crate::raw::net::{Error, Request, Response};

    impl Request {
        /// A request with `method`: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`
        /// or `HEAD`.
        pub fn new(method: &str, url: impl Into<String>) -> Request {
            Request {
                method: method.to_string(),
                url: url.into(),
                headers: Vec::new(),
                body: Vec::new(),
                timeout_ms: 0,
            }
        }

        /// A `GET` request. A `&str` or a `String` works as one too:
        /// `net::fetch("https://...")`.
        pub fn get(url: impl Into<String>) -> Request {
            Request::new("GET", url)
        }

        /// A `POST` request with a body.
        pub fn post(url: impl Into<String>, body: impl Into<Vec<u8>>) -> Request {
            Request::new("POST", url).body(body)
        }

        /// Add a header. At most 64; the ones the runner writes itself, such
        /// as `Host` and `Content-Length`, are refused.
        pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Request {
            self.headers.push((name.into(), value.into()));
            self
        }

        /// The body, at most 8 MiB.
        pub fn body(mut self, body: impl Into<Vec<u8>>) -> Request {
            self.body = body.into();
            self
        }

        /// How long to wait for the answer, from the moment the request is
        /// sent: 30 seconds without this, at most 120.
        pub fn timeout(mut self, timeout: Duration) -> Request {
            self.timeout_ms = u32::try_from(timeout.as_millis()).unwrap_or(u32::MAX).max(1);
            self
        }
    }

    impl From<&str> for Request {
        fn from(url: &str) -> Request {
            Request::get(url)
        }
    }

    impl From<String> for Request {
        fn from(url: String) -> Request {
            Request::get(url)
        }
    }

    impl From<&String> for Request {
        fn from(url: &String) -> Request {
            Request::get(url.as_str())
        }
    }

    impl Response {
        /// The body as text, with any bytes that are not UTF-8 replaced.
        pub fn text(&self) -> String {
            String::from_utf8_lossy(&self.body).into_owned()
        }

        /// The first value of a header, whatever the case of its name.
        pub fn header(&self, name: &str) -> Option<&str> {
            self.headers
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(name))
                .map(|(_, v)| v.as_str())
        }
    }

    /// Make a request and wait for the answer: `net::fetch(url)` for a
    /// `GET`, or a [`Request`]. A status such as 404 is an answer, not an
    /// error.
    ///
    /// In a command, a task or a column. A page cannot wait: it calls
    /// [`fetch_start`], and the answer arrives as the page event `Fetched`.
    pub fn fetch(req: impl Into<Request>) -> Result<Response, Error> {
        crate::raw::net::fetch(&req.into())
    }

    /// [`fetch`] for a page: returns at once with a request id, and the
    /// answer arrives as the page event `Fetched` with that id. Only a page
    /// can call it.
    pub fn fetch_start(req: impl Into<Request>) -> Result<u64, Error> {
        crate::raw::net::fetch_start(&req.into())
    }

    /// Abandon a request made with [`fetch_start`]; its `Fetched` event does
    /// not arrive.
    pub fn cancel(id: u64) {
        crate::raw::net::cancel(id)
    }
}

/// Long tasks: work that runs in a worker, another instance of the plugin,
/// while its commands and pages go on working. The kinds of task are the
/// [`crate::Task`] types the export lists. tessoa shows every task in its
/// transfer panel, where the user can cancel it. No permission needed.
pub mod tasks {
    pub use crate::raw::tasks::TaskOutcome;

    /// Start a task of the given kind: a worker's [`crate::Task::run`]
    /// receives `input`, at most 1 MiB. `title` is shown in tessoa's
    /// transfer panel after the plugin's name; `kind` and `title` take at
    /// most 1 KiB each. Returns the task id. How far the task is and how it
    /// ended arrive as [`crate::PluginEvent::TaskProgress`] and
    /// [`crate::PluginEvent::TaskDone`], and in every open page as the page
    /// events of the same names. At most 4 tasks of the plugin run at once
    /// and 64 more wait their turn; beyond that the call fails, so let one
    /// task work through many items. A task a command starts reads the
    /// command's files until it ends.
    ///
    /// In a command, a page or [`crate::Plugin::on_event`]; not in a worker.
    pub fn start(kind: &str, title: &str, input: &[u8]) -> Result<u64, String> {
        crate::raw::tasks::start(kind, title, input)
    }

    /// Cancel a task: [`crate::TaskCtx::cancelled`] turns true in its
    /// worker, and a worker that has not returned 2 seconds later is
    /// stopped. A task still waiting for its turn ends at once. A task that
    /// has ended, or that the plugin did not start, is left alone.
    pub fn cancel(task: u64) {
        crate::raw::tasks::cancel(task)
    }
}

/// What a running task reports through; [`crate::TaskCtx`] does the same.
/// Outside a worker these do nothing, and `cancelled` is false.
pub mod task {
    /// Report how far the task is; see [`crate::TaskCtx::progress`].
    pub fn progress(done: f64, total: f64, text: &str) {
        crate::raw::task::progress(done, total, text)
    }

    /// Whether the task was cancelled; see [`crate::TaskCtx::cancelled`].
    pub fn cancelled() -> bool {
        crate::raw::task::cancelled()
    }
}

/// What the host does for a plugin's pages. Pages are rendered again after
/// every event without asking, so most plugins only need
/// [`open`](crate::pages_host::open) and
/// [`set_title`](crate::pages_host::set_title); questions are asked with
/// [`crate::pages::confirm`].
#[cfg(feature = "pages")]
pub mod pages_host {
    pub use crate::pages::PageId;

    /// Ask for a page to be rendered again once the current call returns.
    pub fn invalidate(page: PageId) {
        crate::raw::pages_host::invalidate(page)
    }

    /// Open a page of this plugin, declared in `plugin.toml` under `kind`,
    /// with arguments for its [`crate::pages::Page::open`]. A page declared
    /// `single` that is already open in the window is focused instead. From
    /// a command, the page opens in the window the command ran in.
    pub fn open(kind: &str, args: Option<&[u8]>) -> Result<PageId, String> {
        crate::raw::pages_host::open(kind, args)
    }

    /// Change the title of the page's tab.
    pub fn set_title(page: PageId, title: &str) {
        crate::raw::pages_host::set_title(page, title)
    }

    /// Ask a yes/no question on behalf of a page; the answer arrives as
    /// [`crate::pages::Event::Dialog`]. With items, one line per item.
    /// [`crate::pages::confirm`] and its relatives are the usual way to call
    /// this.
    pub fn ask(page: PageId, title: &str, body: &str, items: &[String], ok_label: &str, cancel_label: &str) -> u64 {
        crate::raw::pages_host::ask(page, title, body, items, ok_label, cancel_label)
    }
}

/// Everything a plugin usually needs. With the `pages` feature, the page
/// building blocks are in `pages`; `use tessoa_plugin_sdk::pages::*;` brings
/// them in.
pub mod prelude {
    pub use crate::{
        ColumnFile, ColumnValue, Columns, Plugin, PluginEvent, Task, TaskCtx, clipboard, export, export_pages, files,
        images, log, net, refs, settings, storage, task, tasks, ui,
    };
    #[cfg(feature = "pages")]
    pub use crate::{pages, pages_host};
}

#[cfg(test)]
mod tests {
    use crate::__exports::{column_values, no_task, no_values, plugin_event};
    use crate::bindings::exports::tessoa::plugin::plugin::PluginEvent as WirePluginEvent;
    use crate::bindings::tessoa::plugin::tasks::{TaskOutcome, TaskProgress};
    use crate::{ColumnFile, ColumnValue, Columns, Plugin, PluginEvent, Task, TaskCtx};

    fn file(name: &str) -> ColumnFile {
        ColumnFile {
            handle: 1,
            name: name.into(),
            size: name.len() as u64,
            modified_ms: 0,
            is_dir: false,
        }
    }

    #[test]
    fn a_plugin_without_tasks_or_columns_says_so() {
        assert_eq!(
            no_task("hash", &[]),
            "no task named \"hash\": this plugin defines no tasks"
        );
        assert_eq!(
            no_task("hush", &["hash", "thumbs"]),
            "no task named \"hush\": the export lists [\"hash\", \"thumbs\"]"
        );
        assert_eq!(
            no_values(&[file("a"), file("b")]),
            [ColumnValue::Empty, ColumnValue::Empty]
        );
        assert!(no_values(&[]).is_empty());
    }

    /// Counts the files it saw, across requests.
    struct Counting {
        seen: u32,
    }

    impl Columns for Counting {
        fn value(&mut self, column: &str, file: &ColumnFile) -> ColumnValue {
            self.seen += 1;
            match column {
                "size" => ColumnValue::Size(file.size),
                _ => ColumnValue::sorted_text(file.name.clone(), f64::from(self.seen)),
            }
        }
    }

    #[test]
    fn the_columns_value_lives_as_long_as_the_worker() {
        let made = std::cell::Cell::new(0);
        let make = || {
            made.set(made.get() + 1);
            Counting { seen: 0 }
        };
        assert_eq!(
            column_values("size", &[file("a"), file("bcd")], make),
            [ColumnValue::Size(1), ColumnValue::Size(3)]
        );
        assert_eq!(
            column_values("seen", &[file("x")], make),
            [ColumnValue::SortedText(("x".into(), 3.0))],
            "the value made for the first request answers the next"
        );
        assert_eq!(made.get(), 1);
        assert!(column_values("seen", &[], make).is_empty());
        assert_eq!(made.get(), 1);
    }

    #[test]
    fn plugin_events_are_translated() {
        let progress = WirePluginEvent::TaskProgress(TaskProgress {
            task: 3,
            done: 1.0,
            total: 4.0,
            text: "1 of 4".into(),
        });
        assert_eq!(
            plugin_event(progress),
            PluginEvent::TaskProgress {
                task: 3,
                done: 1.0,
                total: 4.0,
                text: "1 of 4".into()
            }
        );
        assert_eq!(
            plugin_event(WirePluginEvent::TaskDone((3, TaskOutcome::Done(b"ok".to_vec())))),
            PluginEvent::TaskDone {
                task: 3,
                outcome: TaskOutcome::Done(b"ok".to_vec())
            }
        );
        assert_eq!(
            plugin_event(WirePluginEvent::SettingsChanged(vec!["format".into()])),
            PluginEvent::SettingsChanged {
                ids: vec!["format".into()]
            }
        );
    }

    // The sections of `export!`, in any order or none: what follows only has
    // to compile.
    struct Quiet;

    impl Plugin for Quiet {
        fn run_command(_command_id: &str) -> Result<(), String> {
            Ok(())
        }
    }

    struct Echo;

    impl Task for Echo {
        fn run(&mut self, input: Vec<u8>, _task: &TaskCtx) -> Result<Vec<u8>, String> {
            Ok(input)
        }
    }

    struct Times {
        by: u8,
    }

    impl Times {
        fn new(by: u8) -> Times {
            Times { by }
        }
    }

    impl Task for Times {
        fn run(&mut self, input: Vec<u8>, _task: &TaskCtx) -> Result<Vec<u8>, String> {
            Ok(input.iter().map(|b| b.wrapping_mul(self.by)).collect())
        }
    }

    crate::export!(Quiet);
    crate::export!(Quiet; tasks: [("echo", Echo), ("times", Times::new(2)),]);
    crate::export!(Quiet; columns: Counting { seen: 0 }; tasks: [("echo", Echo)]);
    crate::export!(Quiet; tasks: [("echo", Echo)]; columns: Counting { seen: 0 };);
}
