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
        layout::{ Layout, LayoutStack, Message },
        bindings::{
            parse_keybindings_with_xmodmap, KeyEventHandler,
        },
        Config, WindowManager, State,
    },
    extensions::{
        actions::{ toggle_fullscreen },
        hooks::{ add_ewmh_hooks },
    },
    x::{ XConn, XConnExt, XEvent, query::AppName },
    pure::{ Stack, geometry::Rect },
    Xid,
    stack,
    Color,
    map,
    x11rb::RustConn,
    Result,
};

use std::collections::{ HashSet, HashMap };

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
        "M-S-o" => ring_rotate_r(),
        // "M-S-o" => modify_with(|cs| cs.swap_down()),
        "M-S-a" => modify_with(|cs| cs.swap_up()),
        "M-S-q" => modify_with(|cs| cs.next_layout()),
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
        Cols::boxed(),
        MainAndStack::side(1, 0.5, 0.05)
    )
    .map(|l| ReserveTop::wrap(Gaps::wrap(l, gap_outer, gap_inner), bar_height))
}

fn bar_hook<X: XConn + 'static>(id: Xid, state: &mut State<X>, x: &X) -> Result<()> {
    if x.query_or(false, &AppName("shapebar"), id)
    {
        let _ = x.set_client_border_color(id, Color::new_from_hex(0x000000FF));
        let _ = x.modify_and_refresh(state, |cs| { cs.remove_client(&id); });
        // let mut geo = x.client_geometry(id)?;
        // geo.x = 0;
        // geo.y = 0;
        // // let new_geo = Rect { x: 0, y: 0, ..geo };
        // x.position_client(id, geo)?;
        // let _ = x.refresh(state);
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

        let _ = state.client_set.toggle_floating_state(id, r);
        x.refresh(state)
    })
}

#[derive(Debug, Clone, Default)]
pub struct Cols;

impl Cols {
    pub fn boxed() -> Box<dyn Layout> { Box::new(Self) }
}

impl Layout for Cols {
    fn name(&self) -> String { "cols".to_string() }
    fn boxed_clone(&self) -> Box<dyn Layout> { Box::new(self.clone()) }

    fn layout(&mut self, s: &Stack<Xid>, rect: Rect) -> (Option<Box<dyn Layout>>, Vec<(Xid, Rect)>) {
        let mut l: Option<Xid> = None;
        let mut r: Option<Xid> = None;
        let mut ps = Vec::with_capacity(2);

        for id in s {
            if l.is_none() { l = Some(*id); }
            else if r.is_none() { r = Some(*id); }
        }

        if let (Some(l), Some(r)) = (l, r) {
            let (lr, rr) = rect.split_at_width_perc(0.5).expect("could not split rings rec");
            ps.push((l, lr));
            ps.push((r, rr));
        } else if let Some(l) = l {
            ps.push((l, rect));
        }

        (None, ps)
    }

    fn handle_message(&mut self, _: &Message) -> Option<Box<dyn Layout>> {
        None
    }
}

#[derive(Debug, Default, Clone)]
struct Ring {
    ring: Vec<Xid>,
    focus: usize,
}

impl Ring {
    fn focus(&self) -> Option<Xid> {
        if self.ring.is_empty() { None }
        else { Some(self.ring[self.focus]) }
    }

    fn len(&self) -> usize {
        self.ring.len()
    }

    fn insert(&mut self, id: Xid) {
        if self.ring.is_empty() {
            self.ring.push(id);
        } else {
            self.ring.insert(self.focus + 1, id);
            self.focus += 1;
        }
    }

    fn rotate(&mut self) -> Option<Xid> {
        if self.len() < 2 { None }
        else {
            self.focus += 1;
            if self.focus >= self.len() {
                self.focus = 0;
            }
            Some(self.ring[self.focus])
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Rings {
    workspaces: [(Ring, Ring); 4],
    global: HashSet<Xid>,
    map: HashMap<String, usize>,
    last_focus: Option<Xid>,
}

impl Rings {
    fn new() -> Self {
        let mut rings = Self::default();
        rings.map.insert("g".to_string(), 0);
        rings.map.insert("m".to_string(), 1);
        rings.map.insert("l".to_string(), 2);
        rings.map.insert("w".to_string(), 3);
        rings
    }

    // returns (previously focused id needs to minimize, inserted into left column, new right col)
    fn insert(&mut self, id: Xid, focused: Option<Xid>, ws_label: &str) -> (bool, bool, bool) {
        self.global.insert(id);
        if let Some(index) = self.map.get(ws_label) {
            let (l, r) = &mut self.workspaces[*index];
            if l.len() == 0 {
                l.insert(id);
                (false, true, false)
            } else if l.len() == 1 && r.len() == 0 {
                r.insert(id);
                (false, false, true)
            } else if r.focus() == focused {
                r.insert(id);
                (true, false, false)
            } else {
                l.insert(id);
                (true, true, false)
            }
        } else {
            (false, false, false)
        }
    }

    fn rotate(&mut self, focused: Xid, ws_label: &str) -> Option<Xid> {
        if let Some(index) = self.map.get(ws_label) {
            let (l, r) = &mut self.workspaces[*index];
            if l.focus() == Some(focused) {
                l.rotate()
            } else if r.focus() == Some(focused) {
                r.rotate()
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn rings_manage<X: XConn + 'static>(id: Xid, state: &mut State<X>, x: &X) -> Result<()> {
    if x.query_or(false, &AppName("shapebar"), id) { return Ok(()) }
    let rings = state.extension::<Rings>()?;
    let cs = &mut state.client_set;
    let ws = cs.current_workspace();
    let fc = rings.borrow().last_focus;
    let (minimize, into_left, new_right_col) = rings.borrow_mut().insert(id, fc, ws.tag());
    // println!("{}:{:?}:{}", id, fc, ws.tag());
    if let Some(fid) = fc {
        if minimize && fid != id {
            cs.move_client_to_tag(&fid, "m");
        }
    }
    if into_left {
        println!("into left: {}", id);
    } else {
        println!("into right: {}", id);
    }
    if new_right_col {
        cs.rotate_down();
    }
    cs.focus_client(&id);
    rings.borrow_mut().last_focus = Some(id);
    Ok(())
}

pub fn rings_refresh<X: XConn + 'static>(state: &mut State<X>, _: &X) -> Result<()> {
    let rings = state.extension::<Rings>()?;
    let focus = state.client_set.current_client().copied();
    if focus != rings.borrow().last_focus {
        rings.borrow_mut().last_focus = focus;
    }
    Ok(())
}

// pub fn rings_event<X: XConn + 'static>(event: &XEvent, state: &mut State<X>, _: &X) -> Result<bool> {
//     println!("event: {}", event);
//     if let XEvent::FocusIn(id) = event {
//         let rings = state.extension::<Rings>()?;
//         rings.borrow_mut().last_focus = Some(*id);
//         println!("focus: {};", id);
//     }
//     if let XEvent::PropertyNotify(pe) = event {
//         if pe.atom == "_NET_ACTIVE_WINDOW" {
//             println!("active: {};", pe.id);
//         }
//     }
//     Ok(true)
// }

fn ring_rotate_r<X: XConn>() -> Box<dyn KeyEventHandler<X>> {
    key_handler(|state, x: &X| {
        let rings = state.extension::<Rings>()?;
        let cs = &mut state.client_set;
        let ws = cs.current_workspace();
        if ws.layout_name() != "cols" {
            cs.focus_down();
            return x.refresh(state)
        }
        let wstag = ws.tag().to_string();
        let fc = cs.current_client().copied();
        if let Some(fid) = fc {
            if let Some(id) = rings.borrow_mut().rotate(fid, &wstag) {
                cs.move_client_to_tag(&fid, "reikai");
                cs.move_client_to_tag(&id, &wstag);
                cs.focus_client(&id);
                return x.refresh(state);
            }
        }
        Ok(())
    })
}

fn main() -> Result<()> {
    let conn = RustConn::new()?;
    let key_bindings = parse_keybindings_with_xmodmap(raw_key_bindings())?;

    let mut config = add_ewmh_hooks(Config{
        normal_border: Color::new_from_hex(0x414868FF),
        focused_border: Color::new_from_hex(0xF7768EFF),
        border_width: 1,
        focus_follow_mouse: true,
        tags: vec![
            "g".to_string(), "m".to_string(), "l".to_string(), "w".to_string(),
        ],
        floating_classes: vec![
            "ffplay".to_string(),
            "notshapebar".to_string()
        ],
        default_layouts: layouts(),
        // startup_hook: Some(SpawnOnStartup::boxed("~/scripts/.theme/run-shapebar")),
        ..Default::default()
    });

    config.compose_or_set_manage_hook(og_window_size_manage);
    config.compose_or_set_manage_hook(bar_hook);
    config.compose_or_set_manage_hook(rings_manage);
    config.compose_or_set_refresh_hook(rings_refresh);
    // config.compose_or_set_event_hook(rings_event);

    let mut wm = WindowManager::new(config, key_bindings, HashMap::new(), conn)?;
    wm.state.add_extension(OgWindowSize::default());
    wm.state.add_extension(Rings::new());
    wm.state.client_set.add_invisible_workspace("reikai")?;

    wm.run()
}
