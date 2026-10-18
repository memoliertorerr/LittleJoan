import { describe, expect, it } from "vitest";
import { clean, isTracking } from "./logic.js";

const url = (address: string) => clean(address)?.url;

describe("clean link", () => {
  it("strips campaign tags and click ids everywhere", () => {
    expect(clean("https://example.com/post?utm_source=news&utm_medium=email&utm_campaign=oct&id=42")).toEqual({
      url: "https://example.com/post?id=42",
      removed: ["utm_source", "utm_medium", "utm_campaign"],
      unwrapped: null,
    });
    expect(url("https://shop.example/item?fbclid=IwAR0abc")).toBe("https://shop.example/item");
    expect(url("https://example.com/?gclid=x&msclkid=y&mc_eid=z&mc_cid=w&igshid=q&_hsenc=a&mkt_tok=b")).toBe("https://example.com/");
    expect(url("https://example.com/a?UTM_Source=caps&q=keep")).toBe("https://example.com/a?q=keep");
    // An encoded name is still that name.
    expect(url("https://example.com/a?utm%5Fsource=x&q=1")).toBe("https://example.com/a?q=1");
    expect(url("https://example.com/a?ref_src=twsrc%5Etfw")).toBe("https://example.com/a");
  });

  it("keeps what makes the link work, as written", () => {
    for (const address of [
      "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=42s&list=PL123",
      "https://www.google.com/search?q=little+joan&hl=en",
      "https://duckduckgo.com/?q=rust%20book&ia=web",
      "https://example.com/page?id=7&page=2&sort=new#comments",
      "https://example.com/?ref=main",
      "https://example.com/?si=keep-elsewhere&feature=on",
      "https://www.amazon.com/dp/B000123?th=1&psc=1",
      "https://medium.com/@me/post-123?sk=friend-token",
    ]) {
      expect(clean(address)).toEqual({ url: address, removed: [], unwrapped: null });
    }
    // Encoding and order of what stays don't change.
    expect(url("https://example.com/?b=%20x&utm_term=t&a=1+2")).toBe("https://example.com/?b=%20x&a=1+2");
  });

  it("knows the share ids of particular sites, and only there", () => {
    expect(url("https://youtu.be/dQw4w9WgXcQ?si=AbC123&t=10")).toBe("https://youtu.be/dQw4w9WgXcQ?t=10");
    expect(url("https://www.youtube.com/watch?v=abc&feature=share&pp=ygU")).toBe("https://www.youtube.com/watch?v=abc");
    expect(url("https://open.spotify.com/track/123?si=xyz&context=spotify%3Aplaylist")).toBe("https://open.spotify.com/track/123");
    expect(url("https://x.com/someone/status/1?s=20&t=AbCd")).toBe("https://x.com/someone/status/1");
    expect(url("https://www.instagram.com/p/Cx/?igsh=MWQ1")).toBe("https://www.instagram.com/p/Cx/");
    expect(url("https://www.reddit.com/r/rust/comments/1/x/?share_id=abc&utm_name=iossmf")).toBe("https://www.reddit.com/r/rust/comments/1/x/");
    expect(url("https://www.linkedin.com/posts/x?trk=public_post&lipi=urn%3Ali")).toBe("https://www.linkedin.com/posts/x");
    expect(url("https://www.tiktok.com/@a/video/1?is_from_webapp=1&sender_device=pc&web_id=9")).toBe("https://www.tiktok.com/@a/video/1");
    expect(url("https://www.google.com/search?q=joan&sca_esv=1&ei=x&ved=y&oq=joa&sclient=gws")).toBe("https://www.google.com/search?q=joan");
    expect(url("https://www.bing.com/search?q=joan&form=QBLH&cvid=abc")).toBe("https://www.bing.com/search?q=joan");
    expect(isTracking("si", "music.youtube.com")).toBe(true);
    expect(isTracking("si", "example.com")).toBe(false);
    expect(isTracking("ref", "notyoutube.com")).toBe(false);
  });

  it("cleans shop links, path included", () => {
    expect(url("https://www.amazon.com/Some-Book/dp/B000123/ref=sr_1_1?crid=2X&keywords=book&qid=1&sprefix=bo&sr=8-1&tag=aff-20")).toBe(
      "https://www.amazon.com/Some-Book/dp/B000123?keywords=book",
    );
    expect(clean("https://www.amazon.de/dp/B0?pd_rd_w=1&pf_rd_p=2&pd_rd_r=3")?.removed).toEqual(["pd_rd_w", "pf_rd_p", "pd_rd_r"]);
    expect(url("https://www.ebay.com/itm/123?_trkparms=x&_trksid=y&hash=item1&var=55")).toBe("https://www.ebay.com/itm/123?var=55");
    expect(url("https://www.aliexpress.com/item/1.html?spm=a2g0o&pvid=9&aff_fcid=1")).toBe("https://www.aliexpress.com/item/1.html");
  });

  it("takes the real address out of redirect wrappers", () => {
    expect(clean("https://www.google.com/url?sa=t&url=https%3A%2F%2Fexample.com%2Fa%3Futm_source%3Dg%26id%3D1&ved=2")).toEqual({
      url: "https://example.com/a?id=1",
      removed: ["utm_source"],
      unwrapped: "www.google.com",
    });
    expect(url("https://www.google.co.uk/url?q=https://example.org/")).toBe("https://example.org/");
    expect(url("https://l.facebook.com/l.php?u=https%3A%2F%2Fexample.com%2F%3Ffbclid%3D1&h=AT0")).toBe("https://example.com/");
    expect(url("https://out.reddit.com/?url=https%3A%2F%2Fexample.com%2Fx&token=1")).toBe("https://example.com/x");
    expect(url("https://www.youtube.com/redirect?event=video&q=https%3A%2F%2Fexample.com%2F")).toBe("https://example.com/");
    // Never to anything but the web.
    expect(url("https://www.google.com/url?q=javascript%3Aalert(1)")).toBe("https://www.google.com/url?q=javascript%3Aalert(1)");
    // A look-alike host isn't a wrapper.
    expect(url("https://google.com.evil.example/url?q=https://example.org/")).toBe("https://google.com.evil.example/url?q=https://example.org/");
  });

  it("refuses what isn't a web address", () => {
    expect(clean("not a url")).toBeNull();
    expect(clean("ftp://example.com/?utm_source=x")).toBeNull();
    expect(clean("javascript:alert(1)")).toBeNull();
    expect(url("  https://example.com/?utm_source=x  ")).toBe("https://example.com/");
  });
});
