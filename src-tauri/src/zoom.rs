//! Page zoom (LJ16), the pure part: the levels Ctrl/Cmd + / − step through and the one Ctrl/Cmd+0
//! goes back to. The page webview gets the level through `Webview::set_zoom` (bar/zoom.rs) — never
//! through Tauri's zoom-hotkeys option on the webview builder, which on macOS and Linux injects a
//! script into the page that needs IPC (Landmines), and on Windows would turn on WebView2's own
//! zoom keys and pinch zoom beside ours. A test below keeps its name out of the sources.

/// The zoom levels, in percent (as browsers have them).
pub const LEVELS: [u16; 13] = [50, 67, 75, 80, 90, 100, 110, 125, 150, 175, 200, 250, 300];
/// No zoom: every summon starts here.
pub const DEFAULT: u16 = 100;

/// A zoom key's step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    In,
    Out,
    Reset,
}

/// The level after `step` from `current` (any value, not only a level): the next level up or
/// down, stopping at the ends; Reset is 100 %.
pub fn step(current: u16, step: Step) -> u16 {
    let (lowest, highest) = (LEVELS[0], LEVELS[LEVELS.len() - 1]);
    match step {
        Step::In => LEVELS
            .into_iter()
            .find(|level| *level > current)
            .unwrap_or(highest),
        Step::Out => LEVELS
            .into_iter()
            .rev()
            .find(|level| *level < current)
            .unwrap_or(lowest),
        Step::Reset => DEFAULT,
    }
}

/// The engines' zoom factor for `percent` (1.0 = 100 %).
pub fn factor(percent: u16) -> f64 {
    f64::from(percent) / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zooming_in_and_out_walks_the_levels_and_stops_at_the_ends() {
        let mut level = DEFAULT;
        let mut up = Vec::new();
        for _ in 0..8 {
            level = step(level, Step::In);
            up.push(level);
        }
        assert_eq!(up, [110, 125, 150, 175, 200, 250, 300, 300]);
        let mut down = Vec::new();
        for _ in 0..14 {
            level = step(level, Step::Out);
            down.push(level);
        }
        assert_eq!(
            down,
            [
                250, 200, 175, 150, 125, 110, 100, 90, 80, 75, 67, 50, 50, 50
            ]
        );
    }

    #[test]
    fn any_value_steps_to_the_next_level() {
        assert_eq!(step(105, Step::In), 110);
        assert_eq!(step(105, Step::Out), 100);
        assert_eq!(step(110, Step::In), 125);
        assert_eq!(step(110, Step::Out), 100);
        assert_eq!(step(30, Step::In), 50);
        assert_eq!(step(30, Step::Out), 50);
        assert_eq!(step(400, Step::In), 300);
        assert_eq!(step(400, Step::Out), 300);
        assert_eq!(step(0, Step::In), 50);
        assert_eq!(step(u16::MAX, Step::Out), 300);
    }

    #[test]
    fn reset_goes_to_100_from_anywhere() {
        for current in [50, 67, 100, 105, 300, 999] {
            assert_eq!(step(current, Step::Reset), 100);
        }
    }

    #[test]
    fn tauris_zoom_hotkeys_are_never_turned_on() {
        // The option's name, built so this file doesn't hold it either.
        let option = concat!("zoom_hotkeys", "_enabled");
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut stack = vec![src];
        let mut files = 0;
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).expect("src is readable").flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|ext| ext == "rs") {
                    files += 1;
                    let text = std::fs::read_to_string(&path).expect("a source file");
                    assert!(!text.contains(option), "{} names {option}", path.display());
                }
            }
        }
        assert!(files > 50, "the sources were read ({files} files)");
    }

    #[test]
    fn the_levels_rise_and_hold_100() {
        assert!(LEVELS.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(LEVELS.contains(&DEFAULT));
        assert!((factor(110) - 1.1).abs() < f64::EPSILON);
        assert!((factor(DEFAULT) - 1.0).abs() < f64::EPSILON);
    }
}
