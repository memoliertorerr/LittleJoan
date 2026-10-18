// @ts-check
// Clean link's rules, pure and written for Little Joan: which query parameters are tracking
// (campaign tags, click ids, share ids) and which keep a link working, plus the few redirect
// wrappers whose real address is in the link itself. A parameter is removed only when a rule
// names it — everywhere for names nothing functional uses, on its own sites for names that
// could mean something elsewhere (`si`, `ref`, `feature`). Unknown parameters stay.

/**
 * @typedef {object} Cleaned
 * @property {string} url the clean address
 * @property {string[]} removed the parameter names taken out (in order, each once)
 * @property {string | null} unwrapped the redirect host the address was taken out of, if any
 */

/** Tracking parameters on every site: exact names (lower case). */
const ANYWHERE = new Set([
  // Ad and analytics click ids.
  "fbclid",
  "gclid",
  "gclsrc",
  "dclid",
  "gbraid",
  "wbraid",
  "msclkid",
  "yclid",
  "twclid",
  "ttclid",
  "li_fat_id",
  "epik",
  "srsltid",
  "_ga",
  "_gl",
  "s_kwcid",
  "ef_id",
  // Email and marketing tools.
  "mc_cid",
  "mc_eid",
  "_hsenc",
  "_hsmi",
  "__hssc",
  "__hstc",
  "__hsfp",
  "hsctatracking",
  "mkt_tok",
  "vero_id",
  "vero_conv",
  "oly_anon_id",
  "oly_enc_id",
  "rb_clickid",
  "wickedid",
  "ml_subscriber",
  "ml_subscriber_hash",
  // Share ids.
  "igshid",
  "igsh",
  "ref_src",
  "ref_url",
]);

/** Tracking parameters on every site: name prefixes. */
const ANYWHERE_PREFIXES = ["utm_", "pk_campaign", "pk_kwd", "pk_source", "pk_medium", "mtm_"];

/**
 * Parameters that are tracking only on some sites: [host suffixes, exact names, name prefixes].
 * @type {[string[], string[], string[]][]}
 */
const SITES = [
  [["youtube.com", "youtu.be", "youtube-nocookie.com"], ["si", "feature", "pp", "embeds_referring_euri", "embeds_referring_origin", "source_ve_path"], []],
  [["spotify.com"], ["si", "context", "nd", "dlsi"], []],
  [["twitter.com", "x.com"], ["s", "t", "cxt", "ref_src"], []],
  [["instagram.com"], ["igsh", "igshid", "img_index_share"], []],
  [["facebook.com"], ["mibextid", "__tn__", "__cft__", "__xts__", "rdid", "sfnsn", "ref", "notif_id", "notif_t"], ["__cft__", "__xts__"]],
  [["linkedin.com"], ["trk", "trkinfo", "lipi", "originalsubdomain", "refid", "trackingid", "midtoken", "midsig", "eid", "otptoken"], []],
  [["reddit.com"], ["share_id", "rdt", "ref", "ref_source", "ref_campaign", "correlation_id"], []],
  [["tiktok.com"], ["is_from_webapp", "sender_device", "sender_web_id", "is_copy_url", "web_id", "refer", "_r", "_t", "u_code", "preview_pb", "share_app_id", "share_item_id", "share_link_id", "timestamp", "tt_from", "checksum", "user_id"], []],
  [
    ["amazon.com", "amazon.co.uk", "amazon.de", "amazon.fr", "amazon.it", "amazon.es", "amazon.ca", "amazon.co.jp", "amazon.in", "amazon.com.au", "amazon.nl", "amazon.se", "amazon.pl", "amazon.com.tr", "amazon.com.mx", "amazon.com.br"],
    ["ref", "ref_", "_encoding", "content-id", "crid", "sprefix", "dib", "dib_tag", "qid", "sr", "linkcode", "linkid", "tag", "ascsubtag", "creative", "creativeasin", "camp", "spia"],
    ["pd_rd_", "pf_rd_", "_encoding"],
  ],
  [["ebay.com", "ebay.co.uk", "ebay.de", "ebay.fr", "ebay.it", "ebay.es", "ebay.com.au", "ebay.ca"], ["_trkparms", "_trksid", "hash", "amdata", "mkcid", "mkevt", "mkrid", "campid", "toolid", "customid", "siteid", "ufes_redirect"], []],
  [["aliexpress.com", "aliexpress.us"], ["spm", "scm", "pvid", "algo_pvid", "algo_exp_id", "btsid", "ws_ab_test", "pdp_npi", "sk", "dp", "gatewayadapt", "_randl_shipto"], ["aff_", "terminal_id"]],
  [["etsy.com"], ["click_key", "click_sum", "ref", "pro", "frs", "sts", "organic_search_click", "plkey"], []],
  [["medium.com"], ["source"], []],
  [["nytimes.com"], ["smid", "smtyp", "unlocked_article_code"], []],
  [["google.com", "google.co.uk", "google.de", "google.fr", "google.it", "google.es", "google.ca", "google.com.tr", "google.co.jp", "google.com.au", "google.nl", "google.co.in", "google.com.br"], ["ved", "ei", "sxsrf", "oq", "aqs", "gs_lcrp", "gs_lp", "gs_l", "sclient", "sourceid", "uact", "bih", "biw", "rlz", "sca_esv", "sca_upv", "sa", "iflsig", "client", "ie", "oe", "dpr", "fbs"], []],
  [["bing.com"], ["form", "cvid", "sk", "sp", "pq", "sc", "qs", "ghc", "lq", "ghsh", "ghacc", "ghpl"], []],
];

/** Google's hosts in any country (`google.com`, `www.google.co.uk`). */
const GOOGLE = /^(www\.)?google\.(com|[a-z]{2}|com?\.[a-z]{2})$/;

/**
 * Redirect wrappers: [the host, the path, the parameter holding the real address].
 * @type {[(host: string) => boolean, string, string][]}
 */
const WRAPPERS = [
  [(host) => GOOGLE.test(host), "/url", "q"],
  [(host) => GOOGLE.test(host), "/url", "url"],
  [(host) => host === "l.facebook.com" || host === "lm.facebook.com", "/l.php", "u"],
  [(host) => host === "l.messenger.com", "/l.php", "u"],
  [(host) => host === "l.instagram.com", "/", "u"],
  [(host) => host === "out.reddit.com", "/", "url"],
  [(host) => host === "www.youtube.com" || host === "youtube.com", "/redirect", "q"],
  [(host) => host === "t.umblr.com", "/redirect", "z"],
  [(host) => host === "slack-redir.net", "/link", "url"],
  [(host) => host === "away.vk.com", "/away.php", "to"],
];

/**
 * Whether `host` is `domain` or one of its subdomains.
 * @param {string} host
 * @param {string} domain
 */
function within(host, domain) {
  return host === domain || host.endsWith(`.${domain}`);
}

/**
 * Whether the parameter `name` is tracking on `host`.
 * @param {string} name
 * @param {string} host
 */
export function isTracking(name, host) {
  const key = name.toLowerCase();
  if (ANYWHERE.has(key) || ANYWHERE_PREFIXES.some((prefix) => key.startsWith(prefix))) {
    return true;
  }
  return SITES.some(([domains, names, prefixes]) => domains.some((d) => within(host, d)) && (names.includes(key) || prefixes.some((p) => key.startsWith(p))));
}

/**
 * The address inside a known redirect wrapper (http/https only), or null.
 * @param {URL} url
 */
function unwrap(url) {
  const host = url.hostname.toLowerCase();
  for (const [matches, path, param] of WRAPPERS) {
    const inner = matches(host) && url.pathname === path ? url.searchParams.get(param) : null;
    if (inner && /^https?:\/\//i.test(inner)) {
      try {
        return new URL(inner);
      } catch {
        return null;
      }
    }
  }
  return null;
}

/**
 * A query parameter's name, decoded (`utm%5Fsource` → `utm_source`; `+` is a space).
 * @param {string} raw `name=value` as written
 */
function nameOf(raw) {
  const name = raw.split("=")[0] ?? "";
  try {
    return decodeURIComponent(name.replace(/\+/g, " "));
  } catch {
    return name;
  }
}

/**
 * Amazon puts tracking in the path too: `/dp/B000/ref=sr_1_1` → `/dp/B000`.
 * @param {URL} url
 * @returns {boolean} whether the path changed
 */
function cleanPath(url) {
  if (!/(^|\.)amazon\.[a-z.]+$/.test(url.hostname.toLowerCase())) {
    return false;
  }
  const path = url.pathname.replace(/\/ref=[^/]*$/, "");
  if (path === url.pathname) {
    return false;
  }
  url.pathname = path || "/";
  return true;
}

/**
 * The address without its tracking: redirect wrappers unwrapped (up to three deep), tracking
 * parameters removed, functional ones kept in their order, an empty `?` dropped. The fragment
 * stays. Null for anything that isn't an http(s) address.
 * @param {string} address
 * @returns {Cleaned | null}
 */
export function clean(address) {
  /** @type {URL} */
  let url;
  try {
    url = new URL(address.trim());
  } catch {
    return null;
  }
  if (url.protocol !== "http:" && url.protocol !== "https:") {
    return null;
  }
  /** @type {string | null} */
  let unwrapped = null;
  for (let depth = 0; depth < 3; depth += 1) {
    const inner = unwrap(url);
    if (!inner) {
      break;
    }
    unwrapped ??= url.hostname;
    url = inner;
  }
  const host = url.hostname.toLowerCase();
  /** @type {string[]} */
  const removed = [];
  // The kept parameters stay exactly as written (their order and encoding).
  const kept = url.search
    .slice(1)
    .split("&")
    .filter((raw) => {
      if (raw === "") {
        return false;
      }
      const name = nameOf(raw);
      if (!isTracking(name, host)) {
        return true;
      }
      if (!removed.includes(name)) {
        removed.push(name);
      }
      return false;
    });
  if (removed.length > 0) {
    url.search = kept.length > 0 ? `?${kept.join("&")}` : "";
  }
  if (cleanPath(url)) {
    removed.push("ref (in the path)");
  }
  return { url: url.toString(), removed, unwrapped };
}
