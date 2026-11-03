//! Where a handoff goes (LJ46), pure: the user's main browser, the OS default, or another browser
//! when Little Joan is the default itself — never Little Joan. Tested in browsers/tests.rs.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use url::Url;

/// One browser the OS lists for web links (platform/*/browsers.rs reads them; never from a page).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Browser {
    /// macOS: its bundle id; Windows: its `StartMenuInternet` key; Linux: its desktop file id.
    pub id: String,
    pub name: String,
    pub launch: Launch,
}

/// How to open a URL with one browser, without a shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Launch {
    /// macOS: `open -b <id> <url>`.
    Bundle,
    /// Windows: its registered `shell\open\command`, the URL as one more argument.
    Command(String),
    /// Linux: `gio launch <path> <url>`, or its `Exec` with the URL when there is no gio.
    Desktop { path: PathBuf, exec: String },
}

/// What the OS says, read now: its browsers in its own order (Little Joan among them, maybe), its
/// default for https, and which ids are Little Joan.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Browsers {
    pub installed: Vec<Browser>,
    /// The default's id (`None`: it couldn't be read, or there is none).
    pub os_default: Option<String>,
    /// Little Joan's own ids here (its bundle id, ProgId and key, desktop files).
    pub self_ids: Vec<String>,
}

impl Browsers {
    pub fn is_self(&self, id: &str) -> bool {
        self.self_ids.iter().any(|own| own.eq_ignore_ascii_case(id))
    }

    /// The browsers the user may pick: never Little Joan.
    pub fn others(&self) -> impl Iterator<Item = &Browser> {
        self.installed.iter().filter(|b| !self.is_self(&b.id))
    }

    /// `self`, `other` or `none` (unknown): the smoke log's and Settings' word for the default.
    pub fn default_kind(&self) -> &'static str {
        match &self.os_default {
            Some(id) if self.is_self(id) => "self",
            Some(_) => "other",
            None => "none",
        }
    }

    /// The default's name, when it is a browser the OS listed.
    pub fn default_name(&self) -> Option<&str> {
        let id = self.os_default.as_deref()?;
        self.installed
            .iter()
            .find(|b| b.id.eq_ignore_ascii_case(id))
            .map(|b| b.name.as_str())
    }
}

/// Where a page goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target<'a> {
    /// One browser, opened directly.
    Browser(&'a Browser),
    /// The OS default (it isn't Little Joan, or can't be read): the opener, as before LJ46.
    OsDefault,
    /// Nowhere: Little Joan is the default and no other browser is installed.
    Nowhere,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision<'a> {
    pub target: Target<'a>,
    /// A rule pointed at Little Joan itself and was passed over (`handoff-self-refused`).
    pub refused_self: bool,
}

/// A chosen main browser that is installed wins; else the OS default unless it is Little Joan;
/// else (Little Joan is the default) the first other installed browser in the OS's order; else
/// nowhere. A target that is Little Joan is always refused, whatever chose it.
pub fn handoff_target<'a>(main_browser: Option<&str>, browsers: &'a Browsers) -> Decision<'a> {
    let mut refused_self = false;
    if let Some(main) = main_browser {
        if browsers.is_self(main) {
            refused_self = true;
        } else if let Some(browser) = browsers
            .installed
            .iter()
            .find(|b| b.id.eq_ignore_ascii_case(main))
        {
            return Decision {
                target: Target::Browser(browser),
                refused_self,
            };
        }
    }
    match &browsers.os_default {
        Some(id) if browsers.is_self(id) => refused_self = true,
        _ => {
            return Decision {
                target: Target::OsDefault,
                refused_self,
            };
        }
    }
    Decision {
        target: browsers
            .others()
            .next()
            .map_or(Target::Nowhere, Target::Browser),
        refused_self,
    }
}

/// How long a page Little Joan handed on counts as just handed on.
pub const BOUNCE_WINDOW: Duration = Duration::from_secs(10);
const MAX_RECENT: usize = 16;

/// The loop guard's memory (LJ46): the pages Little Joan just handed on. A link that comes back
/// from another app within BOUNCE_WINDOW is one the OS sent straight back — the default it handed
/// it to was Little Joan after all — and doesn't open again (links.rs: `link-refused loop`).
#[derive(Debug, Default)]
pub struct Recent {
    pages: Vec<(String, Instant)>,
}

impl Recent {
    pub const fn new() -> Self {
        Recent { pages: Vec::new() }
    }

    pub fn handed_off(&mut self, url: &Url, now: Instant) {
        self.forget_old(now);
        if self.pages.len() >= MAX_RECENT {
            self.pages.remove(0);
        }
        self.pages.push((url.as_str().to_owned(), now));
    }

    /// Whether `url` is a page handed on within the window (it is forgotten then: one bounce).
    pub fn bounced(&mut self, url: &Url, now: Instant) -> bool {
        self.forget_old(now);
        match self.pages.iter().position(|(page, _)| page == url.as_str()) {
            Some(at) => {
                self.pages.remove(at);
                true
            }
            None => false,
        }
    }

    fn forget_old(&mut self, now: Instant) {
        self.pages
            .retain(|(_, at)| now.saturating_duration_since(*at) < BOUNCE_WINDOW);
    }
}
