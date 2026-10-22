# Checking the name "Little Joan"

A note for the owner (2026-10-22), for before a wider launch or a store listing. **It isn't legal
advice, and it lists where to look, not what you'll find there** — none of these searches has been
run. If anything close turns up, or before spending money on a launch, a trademark attorney's
clearance search is the reliable answer.

Why it's worth doing first: a rename later touches the product name in
`src-tauri/tauri.conf.json`, the installers' file names, the desktop files, the login items
(`Little Joan.plist`, `Little Joan.desktop`), the docs and screenshots, and any store listing. The
app id `io.github.memoliertorerr.littlejoan` would stay, since it comes from the GitHub account,
not the name.

## What to search for

- The name as written and run together: **Little Joan**, **LittleJoan**, **Little-Joan**.
- **Joan** on its own: marks are compared as a whole, and a shared distinctive word can be enough
  for two names to count as similar.
- Names that sound or look close (a different spelling, "Lil Joan").
- Every status: live marks matter most, but pending applications count too, and dead ones show
  who has used the name.

Keeping a record of what you searched, where and when (exported result lists or screenshots) is
useful if the name is ever questioned.

## Classes that matter

Trademarks are registered per class of the Nice Classification
([NCLPUB](https://nclpub.wipo.int/), WIPO's official edition, looks up any term):

- **Class 9** — downloadable software, which is where a browser app sits. The main one.
- **Class 42** — software services: software as a service, software design and development,
  hosting. It matters if a website or hosted service ever comes with Little Joan, and because
  software marks in 9 and 42 are often treated as related.
- A match in another class can still matter when the goods are related, so read the hits in
  neighbouring classes (38, telecommunications, for example) rather than filtering to 9 and 42
  only.

## Trademark registers

| Where | What it covers |
|---|---|
| [USPTO Trademark Search](https://tmsearch.uspto.gov/) | United States: pending, registered and dead marks. It replaced TESS, which was retired on 30 November 2023. [Trademark Center](https://trademarkcenter.uspto.gov/) is the USPTO's filing system (since January 2025) — for applying, not searching. |
| [EUIPO eSearch plus](https://euipo.europa.eu/eSearch/) | European Union trade marks (valid in every EU country). |
| [TMview](https://www.tmdn.org/tmview/) | One search across EUIPO and many national and regional offices (run by EUIPO). |
| [WIPO Global Brand Database](https://branddb.wipo.int/) | International registrations under the Madrid System, plus many national collections ([coverage](https://branddb.wipo.int/en/coverage)). |
| Your own country's office | Wherever you live and sell from. WIPO's [Directory of Intellectual Property Offices](https://www.wipo.int/directory/en/urls.jsp) lists every national office; check whether yours is already in TMview or the Brand Database, and search it directly if not. |

In some countries — the United States among them — a name that is simply in use can carry rights
without being registered, so the store and web searches below matter as well.

## Stores and names in use

- **Apple App Store / Mac App Store:** search the store itself. App Store Connect only says a name
  is taken when you create the app record, which needs an Apple Developer Program membership. The
  [App Store Review Guidelines](https://developer.apple.com/app-store/review/guidelines/) ask for a
  unique name and no other apps' names or trademarked terms in the listing (2.3.7), and cover
  intellectual property in 5.2.
- **Microsoft Store:** Partner Center → New product → **Check availability**
  ([reserving a name](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/reserve-your-apps-name),
  the EXE/MSI route, since Little Joan ships an installer). A reserved name is held for three
  months before you publish.
- **Flathub:** search [flathub.org](https://flathub.org/). Little Joan's app id already follows
  Flathub's rule for GitHub-hosted apps (`io.github.` and at least four parts — see its
  [requirements](https://docs.flathub.org/docs/for-app-authors/requirements)).
- **Snap Store:** search [snapcraft.io/store](https://snapcraft.io/store), if you ever publish a
  snap.
- **A plain web search** for "Little Joan" with app, browser and software.

## Domains and handles

- Domains such as `littlejoan.com`, `.app`, `.dev`, `.org`: any registrar's search, or
  [ICANN Lookup](https://lookup.icann.org/) for who holds a registered one.
- GitHub: whether `github.com/littlejoan` (a user or an organization) is free. The project lives
  at `memoliertorerr/LittleJoan` today.

## Staying clear of Arc

What Little Joan already does:

- Its own name and an original icon, drawn from original geometry in
  `scripts/icon-geometry.mts` and rendered by `scripts/make-icons.mts`. No Arc name, logo or
  artwork is used as branding — a ground rule in `docs/BUILD_PLAN.md`.
- The README and Settings → About say, as plain text with no link or logo: "Inspired by Little
  Arc in the Arc browser. Little Joan is an independent project and is not affiliated with or
  endorsed by The Browser Company."
- The app id comes from the GitHub account (`io.github.memoliertorerr.littlejoan`).

For you to weigh before a store listing:

- Arc's name still appears in descriptive comparisons — the README and CHANGELOG ("the Little
  Arc-style command bar and toolbar", "as in Little Arc") and a few code comments. Stores review
  the listing itself (Apple's 2.3.7 above), so a listing's name, subtitle, keywords and description
  are best kept free of "Arc".
- The "Little …" pattern is the closest point of contact with Little Arc. Searching the registers
  above by owner, **The Browser Company**, shows which of its names are registered and for what.
