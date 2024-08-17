use penrose::{
    builtin::{
        actions::{
            exit, modify_with, send_layout_message, spawn, key_handler,
        },
        layout::messages::{ ExpandMain, ShrinkMain },
        layout::MainAndStack,
        layout::transformers::{ ReserveTop, Gaps },
    },
    core::{
        layout::LayoutStack,
        bindings::{
            parse_keybindings_with_xmodmap, KeyEventHandler,
        },
        Config, WindowManager, State,
    },
    extensions::{
        actions::{ toggle_fullscreen },
        hooks::{ add_ewmh_hooks },
    },
    x::{ XConn, XConnExt, query::AppName },
    pure::geometry::Rect,
    Xid,
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
        "M-S-f" => toggle_floating_focused_remember(),
        "M-n" => toggle_fullscreen(),
        "M-o" => modify_with(|cs| cs.focus_down()),
        "M-a" => modify_with(|cs| cs.focus_up()),
        "M-S-o" => modify_with(|cs| cs.swap_down()),
        "M-S-a" => modify_with(|cs| cs.swap_up()),
        // "M-bracketright" => modify_with(|cs| cs.next_screen()),
        // "M-bracketleft" => modify_with(|cs| cs.previous_screen()),
        "M-S-k" => send_layout_message(|| ExpandMain),
        "M-S-p" => send_layout_message(|| ShrinkMain),
    };

    for tag in &["g", "m", "l", "w"] {
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

fn layouts() -> LayoutStack {
    let gap_outer = 2;
    let gap_inner = 4;
    let bar_height = 24;
    stack!(
        MainAndStack::side(1, 0.5, 0.05)
    )
    .map(|l| ReserveTop::wrap(Gaps::wrap(l, gap_outer, gap_inner), bar_height))
}

fn bar_hook<X: XConn + 'static>(id: Xid, state: &mut State<X>, x: &X) -> Result<()> {
    if x.query_or(false, &AppName("shapebar"), id)
    {
        let _ = x.set_client_border_color(id, Color::new_from_hex(0x000000FF));
        // let _ = x.modify_and_refresh(state, |cs| { cs.remove_client(&id); });
        // let mut geo = x.client_geometry(id)?;
        // geo.x = 0;
        // geo.y = 0;
        // let new_geo = Rect { x: 0, y: 0, ..geo };
        // x.position_client(id, geo)?;
        let _ = x.refresh(state);
    }
    Ok(())
}

#[derive(Debug, Default)]
struct OgWindowSize {
    pub map: HashMap<Xid, (u32, u32)>,
}

fn og_window_size_manage<X: XConn + 'static>(id: Xid, state: &mut State<X>, x: &X) -> Result<()> {
    let rect = x.client_geometry(id)?;
    let ows = state.extension::<OgWindowSize>()?;
    ows.borrow_mut().map.insert(id, (rect.w, rect.h));
    Ok(())
}

pub fn toggle_floating_focused_remember<X: XConn>() -> Box<dyn KeyEventHandler<X>> {
    key_handler(|state, x: &X| {
        let id = match state.client_set.current_client() {
            Some(&id) => id,
            None => return Ok(()),
        };
        let screen_rect = state.client_set.current_screen().geometry();

        let ows = state.extension::<OgWindowSize>()?;
        let (w, h) = *ows.borrow().map.get(&id).unwrap_or(&(512, 512));
        let r = Rect { x: 0, y: 0, w, h };
        let r = r.centered_in(&screen_rect).unwrap_or(r);

        x.modify_and_refresh(state, |cs| {
            let _ = cs.toggle_floating_state(id, r);
        })
    })
}

fn main() -> Result<()> {
    let conn = RustConn::new()?;
    let key_bindings = parse_keybindings_with_xmodmap(raw_key_bindings())?;

    let mut config = add_ewmh_hooks(Config{
        normal_border: Color::new_from_hex(0x414868FF),
        focused_border: Color::new_from_hex(0xF7768EFF),
        border_width: 2,
        focus_follow_mouse: true,
        tags: vec!["g".to_string(), "m".to_string(), "l".to_string(), "w".to_string()],
        floating_classes: vec!["ffplay".to_string(), "shapebar".to_string()],
        default_layouts: layouts(),
        // startup_hook: Some(SpawnOnStartup::boxed("~/scripts/.theme/run-shapebar")),
        ..Default::default()
    });

    config.compose_or_set_manage_hook(og_window_size_manage);
    config.compose_or_set_manage_hook(bar_hook);

    let mut wm = WindowManager::new(config, key_bindings, HashMap::new(), conn)?;
    wm.state.add_extension(OgWindowSize::default());

    wm.run()
}
