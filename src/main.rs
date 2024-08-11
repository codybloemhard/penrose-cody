// #[macro_use]
// extern crate penrose;

use penrose::{
    builtin::{
        actions::{
            floating::sink_focused,
            exit, modify_with, send_layout_message, spawn,
        },
        layout::messages::{ ExpandMain, ShrinkMain },
        layout::MainAndStack,
        layout::transformers::{ ReserveTop, Gaps },
    },
    core::{
        layout::LayoutStack,
        bindings::{
            parse_keybindings_with_xmodmap, KeyEventHandler, MouseEventHandler,
            MouseState, MouseEventKind
        },
        Config, WindowManager,
    },
    extensions::{
        actions::{ toggle_fullscreen },
        hooks::add_ewmh_hooks,
    },
    stack,
    Color,
    map,
    x11rb::RustConn,
    Result,
};

use std::collections::HashMap;

fn raw_key_bindings() -> HashMap<String, Box<dyn KeyEventHandler<RustConn>>> {
    let mut raw_bindings = map! {
        map_keys: |k: &str| k.to_string();

        "M-S-t" => exit(),
        "M-h" => spawn("dmenu_run"),
        "M-b" => spawn("st"),
        "M-j" => spawn("firefox"),
        "M-S-j" => spawn("firefox --private-window"),
        "M-f" => modify_with(|cs| cs.kill_focused()),
        // "M-S-f" => modify_with(|cs| cs.sink_focused()),
        "M-o" => modify_with(|cs| cs.focus_down()),
        "M-a" => modify_with(|cs| cs.focus_up()),
        "M-S-o" => modify_with(|cs| cs.swap_down()),
        "M-S-a" => modify_with(|cs| cs.swap_up()),
        // "M-bracketright" => modify_with(|cs| cs.next_screen()),
        // "M-bracketleft" => modify_with(|cs| cs.previous_screen()),
        "M-S-k" => send_layout_message(|| ExpandMain),
        "M-S-p" => send_layout_message(|| ShrinkMain),
    };

    for tag in &["q", "g", "m", "l", "w"] {
        raw_bindings.extend([
            (
                format!("M-{tag}"),
                modify_with(move |client_set| client_set.focus_tag(tag)),
            ),
            (
                format!("M-S-{tag}"),
                modify_with(move |client_set| client_set.move_focused_to_tag(tag)),
            ),
        ]);
    }

    raw_bindings
}

fn mouse_bindings() -> HashMap<(MouseEventKind, MouseState), Box<dyn MouseEventHandler<RustConn>>> {
    map! {}
}

fn layouts() -> LayoutStack {
    let gap_outer = 2;
    let gap_inner = 4;
    let bar_height = 24;
    stack!(
        MainAndStack::side(1, 0.5, 0.05)
    )
    .map(|l| ReserveTop::wrap(Gaps::wrap(l, gap_outer, gap_inner), bar_height))
}

fn main() -> Result<()> {
    let conn = RustConn::new()?;
    let key_bindings = parse_keybindings_with_xmodmap(raw_key_bindings())?;

    let config = add_ewmh_hooks(Config{
        normal_border: Color::new_from_hex(0x000000FF),
        focused_border: Color::new_from_hex(0xFF000000),
        border_width: 2,
        focus_follow_mouse: true,
        tags: vec!["q".to_string(), "g".to_string(), "m".to_string(), "l".to_string(), "w".to_string()],
        floating_classes: vec!["ffplay".to_string()],
        default_layouts: layouts(),
        // startup_hook: Some(SpawnOnStartup::boxed("~/scripts/.theme/run-shapebar")),
        ..Default::default()
    });

    let wm = WindowManager::new(config, key_bindings, mouse_bindings(), conn)?;

    wm.run()
}
