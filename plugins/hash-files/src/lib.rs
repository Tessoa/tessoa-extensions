//! Example plugin: hash the selected files and save the hashes as a report.
//!
//! Hashing a few large files takes longer than a command may run, so the
//! command only starts a task and returns. The task runs in a worker, a
//! second instance of the plugin: it reads the command's files in pieces,
//! reports its progress after every piece (tessoa shows it in the transfer
//! panel, with a button that cancels the task) and looks between the pieces
//! whether the user cancelled. Then it asks the user for a folder and creates
//! the report in it, in the format `sha256sum --check` reads. A report of the
//! same name that is there already is never replaced: tessoa asks the user
//! whether to keep both or to skip the new one.
//!
//! The setting `algorithm` (SHA-256 or SHA-512) is the user's, in tessoa's
//! settings; the task reads it when it starts. The plugin reads the selected
//! files and creates files, so the manifest declares `files.read-selected`
//! and `files.create`.

use sha2::{Digest, Sha256, Sha512};
use tessoa_plugin_sdk::prelude::*;

const TITLE: &str = "Hash files";

/// How much of a file the task reads at a time, between two looks at
/// whether it was cancelled.
const PIECE: u32 = 1 << 20;

struct HashFiles;

impl Plugin for HashFiles {
    fn run_command(command_id: &str) -> Result<(), String> {
        match command_id {
            "hash-files" => {
                let count = files::selected().iter().filter(|f| !f.is_dir).count();
                if count == 0 {
                    ui::notify(TITLE, "No files selected.");
                    return Ok(());
                }
                // The task reads the command's files itself, so it needs no
                // input. The command returns now; the task goes on.
                let algorithm = Algorithm::chosen();
                let title = format!("{} of {}", algorithm.label(), plural(count, "file"));
                tasks::start("hash", &title, b"")?;
                Ok(())
            }
            other => Err(format!("unknown command {other}")),
        }
    }
}

/// The task: hash every selected file, then save the report.
struct Hash;

impl Task for Hash {
    fn run(&mut self, _input: Vec<u8>, task: &TaskCtx) -> Result<Vec<u8>, String> {
        let algorithm = Algorithm::chosen();
        let mut selected = Vec::new();
        for file in files::selected() {
            if file.is_dir {
                log::info(&format!("skipping folder {}", file.name));
            } else {
                selected.push(file);
            }
        }
        let total: u64 = selected.iter().map(|f| f.size).sum();

        let mut done = 0;
        let mut lines = Vec::new();
        let mut unread = 0;
        for (i, file) in selected.iter().enumerate() {
            let text = format!("{} ({} of {})", file.name, i + 1, selected.len());
            let mut hasher = algorithm.hasher();
            let mut offset = 0;
            let read = loop {
                if task.cancelled() {
                    // Nothing is written yet: there is nothing to clean up.
                    log::info(&format!(
                        "cancelled after {} of {}",
                        lines.len(),
                        plural(selected.len(), "file")
                    ));
                    return Ok(Vec::new());
                }
                match files::read(file.handle, offset, PIECE) {
                    Ok(piece) if piece.is_empty() => break Ok(()),
                    Ok(piece) => {
                        hasher.update(&piece);
                        offset += piece.len() as u64;
                        done += piece.len() as u64;
                        task.progress(done as f64, total as f64, &text);
                    }
                    Err(e) => break Err(e),
                }
            };
            match read {
                Ok(()) => {
                    let hex = hasher.hex();
                    log::info(&format!("{hex}  {}", file.name));
                    lines.push((hex, file.name.clone()));
                }
                Err(e) => {
                    log::warn(&format!("cannot read {}: {e:?}", file.name));
                    unread += 1;
                }
            }
        }
        if lines.is_empty() {
            ui::notify(TITLE, "No file could be read.");
            return Ok(Vec::new());
        }

        // A task may wait for the user: here, for the folder of the report.
        let picked = files::pick_folder("Save the hashes in").map_err(|e| format!("{e:?}"))?;
        let Some(folder) = picked else {
            ui::notify(TITLE, "The hashes were not saved.");
            return Ok(Vec::new());
        };
        let place = files::folder_name(folder).map_err(|e| format!("{e:?}"))?;
        let outcome = files::create_file(folder, algorithm.report_name(), report(&lines).as_bytes());
        files::release_folder(folder);
        let mut message = match outcome.map_err(|e| format!("{e:?}"))? {
            files::CreateOutcome::Created(report) => {
                // The final name has a number added when the user kept both.
                let name = refs::info(&[report]).pop().map(|i| i.name).unwrap_or_default();
                refs::release(&[report]);
                format!("Saved {name} in {place}: {}.", plural(lines.len(), "file"))
            }
            files::CreateOutcome::Skipped(why) => format!("The hashes were not saved: {why}."),
        };
        if unread > 0 {
            message.push_str(&format!(" {} could not be read.", plural(unread, "file")));
        }
        ui::notify(TITLE, &message);
        Ok(Vec::new())
    }
}

tessoa_plugin_sdk::export!(HashFiles; tasks: [("hash", Hash)]);

/// The algorithms the setting offers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Algorithm {
    Sha256,
    Sha512,
}

impl Algorithm {
    /// The one the user chose in the plugin's settings.
    fn chosen() -> Algorithm {
        Algorithm::from_id(&settings::text("algorithm"))
    }

    /// The algorithm of a choice of the setting; SHA-256 for any other id.
    pub fn from_id(id: &str) -> Algorithm {
        match id {
            "sha512" => Algorithm::Sha512,
            _ => Algorithm::Sha256,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Algorithm::Sha256 => "SHA-256",
            Algorithm::Sha512 => "SHA-512",
        }
    }

    /// The report's name, as the tools that check such reports name them.
    pub fn report_name(self) -> &'static str {
        match self {
            Algorithm::Sha256 => "SHA256SUMS.txt",
            Algorithm::Sha512 => "SHA512SUMS.txt",
        }
    }

    pub fn hasher(self) -> Hasher {
        match self {
            Algorithm::Sha256 => Hasher::Sha256(Sha256::new()),
            Algorithm::Sha512 => Hasher::Sha512(Sha512::new()),
        }
    }
}

/// A hash being computed, one piece of the file at a time.
pub enum Hasher {
    Sha256(Sha256),
    Sha512(Sha512),
}

impl Hasher {
    pub fn update(&mut self, bytes: &[u8]) {
        match self {
            Hasher::Sha256(h) => h.update(bytes),
            Hasher::Sha512(h) => h.update(bytes),
        }
    }

    /// The hash in lower-case hexadecimal.
    pub fn hex(self) -> String {
        let bytes = match self {
            Hasher::Sha256(h) => h.finalize().to_vec(),
            Hasher::Sha512(h) => h.finalize().to_vec(),
        };
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}

/// The report: one line per file, the hash, two spaces and the name. A name
/// with a backslash or a line break in it is escaped, and its line starts
/// with a backslash, as `sha256sum` writes it.
pub fn report(lines: &[(String, String)]) -> String {
    let mut out = String::new();
    for (hex, name) in lines {
        if name.contains(['\\', '\n', '\r']) {
            let escaped = name.replace('\\', "\\\\").replace('\n', "\\n").replace('\r', "\\r");
            out.push_str(&format!("\\{hex}  {escaped}\n"));
        } else {
            out.push_str(&format!("{hex}  {name}\n"));
        }
    }
    out
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(algorithm: Algorithm, pieces: &[&[u8]]) -> String {
        let mut h = algorithm.hasher();
        for p in pieces {
            h.update(p);
        }
        h.hex()
    }

    #[test]
    fn hashes_the_same_in_pieces_as_at_once() {
        assert_eq!(
            hash(Algorithm::Sha256, &[b"abc"]),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hash(Algorithm::Sha256, &[b"a", b"", b"bc"]),
            hash(Algorithm::Sha256, &[b"abc"])
        );
        assert_eq!(
            hash(Algorithm::Sha512, &[b"abc"]),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
             2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
    }

    #[test]
    fn writes_the_report_as_sha256sum_does() {
        let lines = [
            ("ab".to_string(), "a.txt".to_string()),
            ("cd".to_string(), "back\\slash.txt".to_string()),
            ("ef".to_string(), "two\nlines.txt".to_string()),
        ];
        assert_eq!(
            report(&lines),
            "ab  a.txt\n\\cd  back\\\\slash.txt\n\\ef  two\\nlines.txt\n"
        );
    }

    #[test]
    fn names_the_report_after_the_chosen_algorithm() {
        assert_eq!(Algorithm::from_id("sha512").report_name(), "SHA512SUMS.txt");
        assert_eq!(Algorithm::from_id("sha256").report_name(), "SHA256SUMS.txt");
        assert_eq!(Algorithm::from_id("").report_name(), "SHA256SUMS.txt");
    }
}
