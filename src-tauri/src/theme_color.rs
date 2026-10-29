//! A theme's base background colour, `--lj-bg-0`, computed in Rust the way ui/lib/theme.ts does
//! (LJ19.4, the theme flash): the window and the bar's webview get it before anything is on screen,
//! so a summon never shows another colour — the engine's white, or the default theme — while the
//! bar paints its first frame. Only the first background (the gradient's base) is computed: the
//! first pick, its lightness mapped into the scheme's band, its chroma capped at 0.12 × intensity,
//! fitted into sRGB. Pure, unit-tested against values ui/lib/theme.ts computed (and a vitest test
//! pins the same values on that side). Matrices: Björn Ottosson's OKLab (as ui/lib/color.ts).

use crate::settings::Theme;

/// The lightness band of the backgrounds per scheme (OKLCH L): ui/lib/theme.ts's `LIGHTNESS`.
const LIGHT_BAND: (f64, f64) = (0.86, 0.93);
const DARK_BAND: (f64, f64) = (0.22, 0.32);
/// The backgrounds' chroma at full intensity, at most (`MAX_CHROMA`).
const MAX_CHROMA: f64 = 0.12;
/// The default theme's first color (ui/lib/presets.ts' default, settings/theme.rs's).
const FALLBACK: &str = "#8c9bb5";

#[derive(Debug, Clone, Copy)]
struct Oklch {
    l: f64,
    c: f64,
    h: f64,
}

fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

fn parse_hex(hex: &str) -> Option<[f64; 3]> {
    let digits = hex.trim().strip_prefix('#')?;
    let full: String = match digits.len() {
        3 => digits.chars().flat_map(|d| [d, d]).collect(),
        6 => digits.to_owned(),
        _ => return None,
    };
    if !full.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let channel = |at: usize| {
        u8::from_str_radix(&full[at..at + 2], 16)
            .ok()
            .map(|v| f64::from(v) / 255.0)
    };
    Some([channel(0)?, channel(2)?, channel(4)?])
}

fn to_linear(x: f64) -> f64 {
    let a = x.abs();
    x.signum()
        * if a <= 0.04045 {
            a / 12.92
        } else {
            ((a + 0.055) / 1.055).powf(2.4)
        }
}

fn to_gamma(x: f64) -> f64 {
    let a = x.abs();
    x.signum()
        * if a <= 0.003_130_8 {
            12.92 * a
        } else {
            1.055 * a.powf(1.0 / 2.4) - 0.055
        }
}

fn hex_to_oklch(hex: &str) -> Option<Oklch> {
    let [r, g, b] = parse_hex(hex)?.map(to_linear);
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    let lab_l = 0.210_454_255_3 * l + 0.793_617_785 * m - 0.004_072_046_8 * s;
    let lab_a = 1.977_998_495_1 * l - 2.428_592_205 * m + 0.450_593_709_9 * s;
    let lab_b = 0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766 * s;
    let c = lab_a.hypot(lab_b);
    let h = if c < 1e-7 {
        0.0
    } else {
        (lab_b.atan2(lab_a).to_degrees() % 360.0 + 360.0) % 360.0
    };
    Some(Oklch { l: lab_l, c, h })
}

fn oklch_to_rgb(lch: Oklch) -> [f64; 3] {
    let angle = lch.h.to_radians();
    let (a, b) = (lch.c * angle.cos(), lch.c * angle.sin());
    let l = (lch.l + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
    let m = (lch.l - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
    let s = (lch.l - 0.089_484_177_5 * a - 1.291_485_548 * b).powi(3);
    [
        to_gamma(4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s),
        to_gamma(-1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s),
        to_gamma(-0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701 * s),
    ]
}

fn in_gamut(rgb: [f64; 3]) -> bool {
    rgb.iter().all(|&x| (-1e-6..=1.0 + 1e-6).contains(&x))
}

/// `lch` with its chroma lowered (lightness and hue kept) until sRGB can show it (`fitGamut`).
fn fit_gamut(lch: Oklch) -> Oklch {
    let l = clamp01(lch.l);
    let c = lch.c.max(0.0);
    if in_gamut(oklch_to_rgb(Oklch { l, c, h: lch.h })) {
        return Oklch { l, c, h: lch.h };
    }
    let (mut low, mut high) = (0.0, c);
    for _ in 0..24 {
        let mid = f64::midpoint(low, high);
        if in_gamut(oklch_to_rgb(Oklch {
            l,
            c: mid,
            h: lch.h,
        })) {
            low = mid;
        } else {
            high = mid;
        }
    }
    Oklch {
        l,
        c: low,
        h: lch.h,
    }
}

/// A pick as a background (`mute`): its hue kept, its lightness mapped into the band, its chroma
/// capped, fitted into sRGB.
fn mute(color: &str, dark: bool, intensity: f64) -> Oklch {
    let pick = hex_to_oklch(color)
        .or_else(|| hex_to_oklch(FALLBACK))
        .unwrap_or(Oklch {
            l: 0.5,
            c: 0.0,
            h: 0.0,
        });
    let (low, high) = if dark { DARK_BAND } else { LIGHT_BAND };
    fit_gamut(Oklch {
        l: low + (high - low) * clamp01(pick.l),
        c: pick.c.min(MAX_CHROMA) * clamp01(intensity),
        h: pick.h,
    })
}

/// `theme`'s `--lj-bg-0` in its light (`dark` false) or dark scheme, as sRGB bytes.
pub fn base_background(theme: &Theme, dark: bool) -> [u8; 3] {
    let first = theme.colors.first().map_or(FALLBACK, String::as_str);
    let rgb = oklch_to_rgb(fit_gamut(mute(first, dark, theme.intensity)));
    // Clamped into 0–255 first, so the cast cannot truncate.
    rgb.map(|x| (clamp01(x) * 255.0).round() as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::ThemeMode;

    fn theme(colors: &[&str], intensity: f64) -> Theme {
        Theme {
            mode: ThemeMode::System,
            colors: colors.iter().map(|&c| c.to_owned()).collect(),
            intensity,
            grain: 0.0,
        }
    }

    fn hex([r, g, b]: [u8; 3]) -> String {
        format!("#{r:02x}{g:02x}{b:02x}")
    }

    /// The same cases as ui/lib/theme.test.ts' "base background" test: what theme.ts computed.
    #[test]
    fn the_base_background_is_theme_ts_s() {
        let cases: [(&[&str], f64, bool, &str); 10] = [
            (&["#8c9bb5"], 0.45, false, "#dae1ee"),
            (&["#8c9bb5"], 0.45, true, "#262b35"),
            (&["#9d8fd0", "#e0a2a0"], 0.6, false, "#e2dbff"),
            (&["#9d8fd0", "#e0a2a0"], 0.6, true, "#2d2545"),
            (&["#ff0000"], 1.0, false, "#ffd4cc"),
            (&["#00ff00"], 1.0, true, "#003b00"),
            (&["#000000"], 0.5, false, "#d1d1d1"),
            (&["#ffffff"], 0.0, true, "#333333"),
            (&["#1e90ff", "#123456", "#abcdef"], 0.8, true, "#002a54"),
            (&["#2f6f4f"], 0.3, false, "#cfe2d6"),
        ];
        for (colors, intensity, dark, want) in cases {
            assert_eq!(
                hex(base_background(&theme(colors, intensity), dark)),
                want,
                "{colors:?} {intensity} dark={dark}"
            );
        }
    }

    #[test]
    fn the_default_theme_is_its_preset_and_junk_falls_back() {
        assert_eq!(hex(base_background(&Theme::default(), false)), "#dae1ee");
        assert_eq!(hex(base_background(&Theme::default(), true)), "#262b35");
        assert_eq!(hex(base_background(&theme(&[], 0.45), false)), "#dae1ee");
        assert_eq!(
            hex(base_background(&theme(&["teal"], 0.45), true)),
            "#262b35"
        );
        for intensity in [f64::NAN, -2.0, 9.0] {
            let [r, g, b] = base_background(&theme(&["#ff0000"], intensity), false);
            assert!(r > 200 && g > 150 && b > 150, "{intensity}: {r} {g} {b}");
        }
    }
}
