//! File-derived Wayfinder maps. The original Chartr reader is the format reference.
use anyhow::{Context as _, Result, bail};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{Read as _, Write as _},
    path::{Path, PathBuf},
};

const MAX_FILE: u64 = 2 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Open,
    Claimed,
    Resolved,
    OutOfScope,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ticket {
    pub number: u32,
    pub path: PathBuf,
    pub title: String,
    pub kind: String,
    pub raw: String,
    pub body: String,
    pub answer: String,
    pub blockers: Vec<u32>,
    pub undermined: Vec<u32>,
    pub assets: Vec<String>,
    pub claimed_by: String,
    pub status: Status,
    pub frontier: bool,
    pub warnings: Vec<String>,
}

impl Ticket {
    pub fn state(&self) -> &'static str {
        match self.status {
            Status::Resolved => "Resolved",
            Status::OutOfScope => "Out of scope",
            Status::Claimed => "Claimed",
            Status::Open if self.frontier => "Ready",
            Status::Open => "Blocked",
        }
    }
    pub fn method(&self) -> Option<&'static str> {
        match self.kind.as_str() {
            "grilling" => Some("grill"),
            "prototype" => Some("prototype"),
            "research" => Some("research"),
            "task" => Some("implement"),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fog {
    pub title: String,
    pub clears_with: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Map {
    pub slug: String,
    pub directory: PathBuf,
    pub title: String,
    pub raw: String,
    pub body: String,
    pub destination: String,
    pub fog: Vec<Fog>,
    pub tickets: Vec<Ticket>,
    pub warnings: Vec<String>,
}

impl Map {
    pub fn ticket(&self, number: u32) -> Option<&Ticket> {
        self.tickets.iter().find(|t| t.number == number)
    }
    pub fn finished(&self) -> bool {
        !self.tickets.is_empty()
            && self
                .tickets
                .iter()
                .all(|t| matches!(t.status, Status::Resolved | Status::OutOfScope))
    }
    pub fn frontier(&self) -> usize {
        self.tickets.iter().filter(|t| t.frontier).count()
    }
}

/// Fixed discovery depth; malformed neighbors never suppress a readable map.
pub fn discover(root: &Path) -> Result<Vec<Map>> {
    let root = root.canonicalize().context("The space folder is unavailable")?;
    let maps_root = root.join(".plan/maps");
    let entries = match fs::read_dir(&maps_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error).context("Reading .plan/maps"),
    };
    let mut maps = Vec::new();
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let directory = entry.path();
        if !directory.join("map.md").exists() {
            continue;
        }
        let slug = entry.file_name().to_string_lossy().into_owned();
        let mut map = Map {
            title: slug.clone(),
            slug,
            directory: directory.clone(),
            raw: String::new(),
            body: String::new(),
            destination: String::new(),
            fog: Vec::new(),
            tickets: Vec::new(),
            warnings: Vec::new(),
        };
        match read_contained(&root, &directory.join("map.md")) {
            Ok(raw) => {
                let scan = structural_lines(&raw);
                map.title = title(&scan).unwrap_or_else(|| map.slug.clone());
                map.body = body(&raw, &scan);
                map.destination = section(&raw, &scan, "Destination").unwrap_or_default();
                map.fog = section(&raw, &scan, "Not yet specified")
                    .unwrap_or_default()
                    .lines()
                    .filter_map(|line| {
                        let rest = line.strip_prefix("- **")?;
                        let (title, tail) = rest.split_once("**")?;
                        let clears_with =
                            tail.split_once("clears-with:").and_then(|(_, number)| {
                                number
                                    .trim()
                                    .split(|c: char| !c.is_ascii_digit())
                                    .next()?
                                    .parse()
                                    .ok()
                            });
                        Some(Fog { title: title.trim_end_matches('.').into(), clears_with })
                    })
                    .collect();
                if map.destination.is_empty() {
                    map.warnings.push("map.md has no Destination.".into());
                }
                map.raw = raw;
            }
            Err(error) => map.warnings.push(format!("map.md: {error:#}")),
        }
        match fs::read_dir(directory.join("tickets")) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().is_none_or(|ext| ext != "md") {
                        continue;
                    }
                    match read_contained(&root, &path)
                        .and_then(|raw| parse_ticket(path.clone(), raw))
                    {
                        Ok(ticket) => map.tickets.push(ticket),
                        Err(error) => map
                            .warnings
                            .push(format!("{}: {error:#}", entry.file_name().to_string_lossy())),
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => map.warnings.push(format!("tickets: {error}")),
        }
        map.tickets.sort_by_key(|ticket| ticket.number);
        derive(&mut map);
        maps.push(map);
    }
    maps.sort_by(|left, right| (left.finished(), &left.slug).cmp(&(right.finished(), &right.slug)));
    Ok(maps)
}

fn read_contained(root: &Path, path: &Path) -> Result<String> {
    let path = path.canonicalize()?;
    if !path.starts_with(root) {
        bail!("Path leaves this space.");
    }
    let file = fs::File::open(&path)?;
    if !file.metadata()?.is_file() {
        bail!("Not a regular file.");
    }
    let mut text = String::new();
    file.take(MAX_FILE + 1).read_to_string(&mut text)?;
    if text.len() as u64 > MAX_FILE {
        bail!("File exceeds 2 MiB.");
    }
    Ok(text)
}

pub fn parse_ticket(path: PathBuf, raw: String) -> Result<Ticket> {
    let name =
        path.file_name().and_then(|name| name.to_str()).context("Invalid ticket filename")?;
    let (number, slug) = name
        .strip_suffix(".md")
        .and_then(|name| name.split_once('-'))
        .context("Expected NN-slug.md")?;
    let number: u32 = number.parse().context("Invalid ticket number")?;
    if number == 0 || slug.is_empty() {
        bail!("Expected a positive ticket number and a slug.");
    }
    let (fields, _) = frontmatter(&raw);
    let scan = structural_lines(&raw);
    let mut warnings = Vec::new();
    if fields.is_empty() {
        warnings.push("Use YAML frontmatter for type and blockers.".into());
    }
    let get = |key: &str| fields.get(key).cloned().unwrap_or_default();
    let legacy = |key: &str| {
        scan.iter()
            .find_map(|line| line.strip_prefix(key))
            .map(|s| s.trim().to_owned())
            .unwrap_or_default()
    };
    let kind =
        if fields.contains_key("type") { get("type") } else { legacy("Type:") }.to_lowercase();
    if !matches!(kind.as_str(), "grilling" | "prototype" | "research" | "task") {
        warnings.push(format!("Unknown ticket type: {kind}"));
    }
    let title = title(&scan).unwrap_or_default();
    if title.is_empty() {
        warnings.push("No ticket title.".into());
    }
    for heading in ["Question", "Done when"] {
        if section(&raw, &scan, heading).is_none() {
            warnings.push(format!("Missing {heading} section."));
        }
    }
    let answer = section(&raw, &scan, "Answer");
    let ruled_out = section(&raw, &scan, "Ruled out");
    if answer.is_some() && ruled_out.is_some() {
        warnings.push("Both Answer and Ruled out are present.".into());
    }
    if answer.as_ref().is_some_and(String::is_empty)
        || ruled_out.as_ref().is_some_and(String::is_empty)
    {
        warnings.push("An empty closing heading does not close this ticket.".into());
    }
    if fields.contains_key("status") {
        warnings.push("Stored status is ignored; status comes from the body.".into());
    }
    let claimed_by = get("claimed_by");
    let answer = answer.unwrap_or_default();
    let status = if !answer.is_empty() {
        Status::Resolved
    } else if ruled_out.is_some_and(|text| !text.is_empty()) {
        Status::OutOfScope
    } else if !claimed_by.is_empty() {
        Status::Claimed
    } else {
        Status::Open
    };
    let blockers = numbers(&if fields.contains_key("blocked_by") {
        get("blocked_by")
    } else {
        legacy("Blocked by:")
    })?;
    let undermined = numbers(&get("undermined_by"))?;
    Ok(Ticket {
        number,
        path,
        title,
        kind,
        body: body(&raw, &scan),
        raw,
        answer,
        blockers,
        undermined,
        assets: list(&get("assets")),
        claimed_by,
        status,
        frontier: false,
        warnings,
    })
}

fn derive(map: &mut Map) {
    let resolved: HashSet<_> =
        map.tickets.iter().filter(|t| t.status == Status::Resolved).map(|t| t.number).collect();
    let mut counts = BTreeMap::new();
    for ticket in &map.tickets {
        *counts.entry(ticket.number).or_insert(0) += 1;
    }
    let duplicate = counts.values().any(|count| *count > 1);
    if duplicate {
        map.warnings.push("Duplicate ticket numbers; resolve them before launching.".into());
    }
    let edges: BTreeMap<_, _> =
        map.tickets.iter().map(|t| (t.number, t.blockers.clone())).collect();
    for ticket in &mut map.tickets {
        let mut pending = ticket.blockers.clone();
        let mut visited = HashSet::new();
        let mut cyclic = false;
        while let Some(number) = pending.pop() {
            if number == ticket.number {
                cyclic = true;
                break;
            }
            if visited.insert(number)
                && let Some(blockers) = edges.get(&number)
            {
                pending.extend(blockers);
            }
        }
        if cyclic {
            ticket.warnings.push("This ticket is in a dependency cycle.".into());
        }
        for blocker in &ticket.blockers {
            if !counts.contains_key(blocker) {
                ticket.warnings.push(format!("Missing blocker {blocker:02}."));
            }
        }
        ticket.frontier = !duplicate
            && !cyclic
            && ticket.status == Status::Open
            && ticket.blockers.iter().all(|number| resolved.contains(number));
    }
}

pub fn structural_lines(text: &str) -> Vec<String> {
    let mut fence: Option<(char, usize)> = None;
    text.lines()
        .map(|line| {
            let trimmed = line.trim_start();
            let delimiter = trimmed.chars().next().filter(|c| matches!(c, '`' | '~'));
            let run = delimiter
                .map(|c| trimmed.chars().take_while(|next| *next == c).count())
                .unwrap_or(0);
            if let Some((character, length)) = fence {
                if delimiter == Some(character) && run >= length {
                    fence = None;
                }
                String::new()
            } else if run >= 3 {
                fence = Some((delimiter.unwrap(), run));
                String::new()
            } else {
                line.to_owned()
            }
        })
        .collect()
}

fn title(scan: &[String]) -> Option<String> {
    scan.iter().find_map(|line| line.strip_prefix("# ").map(|s| s.trim().to_owned()))
}
fn body(raw: &str, scan: &[String]) -> String {
    let start = scan.iter().position(|line| line.starts_with("# ")).map(|i| i + 1).unwrap_or(0);
    raw.lines().skip(start).collect::<Vec<_>>().join("\n").trim().to_owned()
}
pub fn section(raw: &str, scan: &[String], name: &str) -> Option<String> {
    let start = scan.iter().position(|line| line.trim() == format!("## {name}"))? + 1;
    let end = (start..scan.len()).find(|i| scan[*i].starts_with("## ")).unwrap_or(scan.len());
    Some(raw.lines().skip(start).take(end - start).collect::<Vec<_>>().join("\n").trim().to_owned())
}

pub fn frontmatter(raw: &str) -> (BTreeMap<String, String>, usize) {
    let lines: Vec<_> = raw.lines().collect();
    if lines.first().is_none_or(|line| line.trim() != "---") {
        return (BTreeMap::new(), 0);
    }
    let Some(end) = (1..lines.len()).find(|i| lines[*i].trim() == "---") else {
        return (BTreeMap::new(), 0);
    };
    let mut fields = BTreeMap::<String, String>::new();
    let mut key = String::new();
    for line in &lines[1..end] {
        let line = line.split_once(" #").map(|(line, _)| line).unwrap_or(line).trim();
        if let Some(value) = line.strip_prefix("- ") {
            if let Some(previous) = fields.get_mut(&key) {
                if !previous.is_empty() {
                    previous.push(',');
                }
                previous.push_str(value);
            }
        } else if let Some((name, value)) = line.split_once(':') {
            key = name.trim().to_owned();
            fields.insert(key.clone(), value.trim().trim_matches(['\'', '"']).to_owned());
        }
    }
    (fields, end + 1)
}
fn list(value: &str) -> Vec<String> {
    value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|s| s.trim().trim_matches(['\'', '"']))
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("none"))
        .map(str::to_owned)
        .collect()
}
fn numbers(value: &str) -> Result<Vec<u32>> {
    list(value)
        .iter()
        .map(|number| {
            let number = number.parse::<u32>().context("Invalid ticket reference")?;
            if number == 0 {
                bail!("Ticket references must be positive.");
            }
            Ok(number)
        })
        .collect()
}

/// Revalidate the preview under an OS lock shared by every Wayfinder instance.
/// The claim is written only after a real terminal has been prepared.
pub fn claim(root: &Path, expected: &Map, number: u32, session: &str) -> Result<()> {
    let _lock = lock(root)?;
    let maps = discover(root)?;
    if let Some(ticket) =
        maps.iter().flat_map(|map| &map.tickets).find(|ticket| ticket.status == Status::Claimed)
    {
        bail!(
            "“{}” is already claimed in this space. Finish it or release its claim first.",
            ticket.title
        );
    }
    let current =
        maps.iter().find(|map| map.slug == expected.slug).context("The map was removed")?;
    if current != expected {
        bail!("The map changed after preview. Review the refreshed prompt and launch again.");
    }
    let ticket = current.ticket(number).context("The ticket was removed")?;
    if !ticket.frontier || ticket.method().is_none() {
        bail!("This ticket is not ready to launch.");
    }
    write_claim(ticket, Some(session))
}

pub fn release(root: &Path, path: &Path, session: &str) -> Result<()> {
    let _lock = lock(root)?;
    let root = root.canonicalize()?;
    let ticket = parse_ticket(path.to_owned(), read_contained(&root, path)?)?;
    if ticket.claimed_by != session {
        bail!("The claim changed. Refresh before releasing it.");
    }
    write_claim(&ticket, None)
}

struct LaunchLock(fs::File);

impl Drop for LaunchLock {
    fn drop(&mut self) {
        // A concurrent fork can inherit this descriptor until exec. Closing
        // only our copy would leave its lock held on Linux in the meantime.
        let _ = self.0.unlock();
    }
}

fn lock(root: &Path) -> Result<LaunchLock> {
    let root = root.canonicalize()?;
    let plan = root.join(".plan").canonicalize()?;
    if !plan.starts_with(&root) {
        bail!("The plan directory leaves this space.");
    }
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(plan.join(".wayfinder-launch.lock"))?;
    file.try_lock().context("Another Wayfinder launch is being prepared")?;
    Ok(LaunchLock(file))
}

fn write_claim(ticket: &Ticket, session: Option<&str>) -> Result<()> {
    let (_, end) = frontmatter(&ticket.raw);
    if end == 0 {
        bail!("Migrate this ticket to YAML frontmatter before launching.");
    }
    let lines: Vec<_> = ticket.raw.lines().collect();
    let mut result = String::from("---\n");
    for line in &lines[1..end - 1] {
        if !matches!(
            line.split_once(':').map(|(key, _)| key.trim()),
            Some("claimed_by" | "claimed_at")
        ) {
            result.push_str(line);
            result.push('\n');
        }
    }
    if let Some(session) = session {
        if session.contains(['\n', '\r', '\0']) {
            bail!("Invalid session identifier.");
        }
        result.push_str(&format!(
            "claimed_by: {}\nclaimed_at: {}\n",
            serde_json::to_string(session)?,
            chrono::Utc::now().to_rfc3339()
        ));
    }
    result.push_str("---\n");
    result.push_str(&lines[end..].join("\n"));
    if ticket.raw.ends_with('\n') {
        result.push('\n');
    }
    let mut temporary =
        tempfile::NamedTempFile::new_in(ticket.path.parent().context("No ticket directory")?)?;
    temporary.as_file().set_permissions(fs::metadata(&ticket.path)?.permissions())?;
    temporary.write_all(result.as_bytes())?;
    temporary.as_file().sync_all()?;
    if fs::read_to_string(&ticket.path)? != ticket.raw {
        bail!("The ticket changed while updating its claim.");
    }
    temporary.persist(&ticket.path).map_err(|error| error.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ticket(body: &str) -> Ticket {
        parse_ticket("01-question.md".into(), format!("---\ntype: research\nblocked_by: []\n---\n# Question\n\n## Question\nWhy?\n\n## Done when\nKnown.\n{body}")).unwrap()
    }

    #[test]
    fn quoted_and_empty_answers_do_not_close_tickets() {
        assert_eq!(ticket("```markdown\n## Answer\nExample\n```\n").status, Status::Open);
        assert_eq!(ticket("## Answer\n\n").status, Status::Open);
        assert_eq!(ticket("## Answer\n```\nresult\n```\n").status, Status::Resolved);
        assert_eq!(ticket("## Ruled out\nOutside scope.\n").status, Status::OutOfScope);
    }

    #[test]
    fn closure_precedes_a_leftover_claim_and_lists_accept_legacy_numbers() {
        let raw = "---\ntype: task\nclaimed_by: session\nblocked_by:\n  - 01\n  - 002\n---\n# Work\n## Answer\nDone.\n";
        let ticket = parse_ticket("03-work.md".into(), raw.into()).unwrap();
        assert_eq!(ticket.status, Status::Resolved);
        assert_eq!(ticket.blockers, vec![1, 2]);
    }

    #[test]
    fn frontier_claims_and_stale_previews_share_the_same_file_truth() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join(".plan/maps/test");
        fs::create_dir_all(dir.join("tickets")).unwrap();
        fs::write(dir.join("map.md"), "# Map\n## Destination\nDecide.\n").unwrap();
        let path = dir.join("tickets/01-question.md");
        fs::write(&path, ticket("").raw).unwrap();
        let expected = discover(root.path()).unwrap().remove(0);
        assert!(expected.ticket(1).unwrap().frontier);
        claim(root.path(), &expected, 1, "terminal-1").unwrap();
        assert!(claim(root.path(), &expected, 1, "terminal-2").is_err());
        assert!(release(root.path(), &path, "terminal-2").is_err());
        release(root.path(), &path, "terminal-1").unwrap();
        fs::write(dir.join("map.md"), "# Changed\n## Destination\nNew.\n").unwrap();
        assert!(claim(root.path(), &expected, 1, "terminal-2").is_err());
        assert_eq!(discover(root.path()).unwrap()[0].tickets[0].status, Status::Open);
    }

    #[test]
    fn launch_lock_releases_even_while_an_inherited_descriptor_remains_open() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join(".plan")).unwrap();
        let guard = lock(root.path()).unwrap();
        assert!(lock(root.path()).is_err());

        let inherited = guard.0.try_clone().unwrap();
        drop(guard);
        let next = lock(root.path()).unwrap();
        drop(next);
        drop(inherited);
    }
}
