use penrose::{
    builtin::{
        actions::{
            exit, modify_with, send_layout_message, spawn, key_handler,
        },
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
    x::{ XConn, XConnExt, XEvent, query::AppName, Atom, Prop },
    pure::{ Stack, StackSet, geometry::Rect },
    Xid,
    stack,
    Color,
    map,
    x11rb::RustConn,
    Result,
};

use std::collections::{ HashMap, HashSet };
use std::sync::Arc;
use std::cell::RefCell;

use tracing_subscriber::{ self, prelude::* };

fn raw_key_bindings() -> HashMap<String, Box<dyn KeyEventHandler<RustConn>>> {
    let mut raw_bindings = map! {
        map_keys: |k: &str| k.to_string();

        "M-h" => spawn("dmenu_run"),
        "M-b" => spawn("st"),
        "M-j" => spawn("firefox"),
        "M-S-j" => spawn("firefox --private-window"),
        "M-f" => modify_with(|cs| cs.kill_focused()),
        "M-S-f" => toggle_floating_focused_remember(),
        "M-n" => toggle_fullscreen(),
        "M-o" => modify_with(|cs| cs.focus_down()),
        "M-a" => modify_with(|cs| cs.focus_up()),
        "M-S-o" => ring_rotate(true),
        "M-S-a" => ring_rotate(false),
        "M-S-e" => swap_cols(),
        "M-comma" => modify_with(|cs| cs.previous_screen()),
        "M-period" => modify_with(|cs| cs.next_screen()),
        "M-S-comma" => swap_ring(false),
        "M-S-period" => swap_ring(true),
        "M-space" => toggle_scratchpad(),
        "M-S-space" => link_scratchpad(),
        "M-S-y" => log_status(),
        "M-S-t" => exit(),
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
        Cols::boxed()
    )
    .map(|l| ReserveTop::wrap(Gaps::wrap(l, gap_outer, gap_inner), bar_height))
}

fn bar_hook<X: XConn + 'static>(id: Xid, state: &mut State<X>, x: &X) -> Result<()> {
    if x.query_or(false, &AppName("shapebar"), id)
    {
        // let _ = x.set_client_border_color(id, Color::new_from_hex(0x000000FF));
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
    fn name(&self) -> String { "2col".to_string() }
    fn boxed_clone(&self) -> Box<dyn Layout> { Box::new(self.clone()) }

    fn layout(&mut self, s: &Stack<Xid>, rect: Rect) -> (Option<Box<dyn Layout>>, Vec<(Xid, Rect)>) {
        let mut l: Option<Xid> = None;
        let mut r: Option<Xid> = None;
        let mut ps = Vec::with_capacity(2);

        for id in s {
            if l.is_none() { l = Some(*id); }
            else if r.is_none() { r = Some(*id); }
            else { break; }
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
    fn len(&self) -> usize {
        self.ring.len()
    }

    fn focus(&self) -> Option<Xid> {
        if self.ring.is_empty() { None }
        else { Some(self.ring[self.focus]) }
    }

    fn insert(&mut self, id: Xid) {
        if self.ring.is_empty() {
            self.ring.push(id);
        } else {
            self.ring.insert(self.focus + 1, id);
            self.focus += 1;
        }
    }

    // returns newly focused on id
    fn rotate(&mut self, right: bool) -> Option<Xid> {
        if self.len() < 2 { None }
        else {
            let mut f = (self.focus as i32) + if right { 1 } else { -1 };
            if f >= self.len() as i32 {
                f = 0;
            }
            if f < 0 {
                f = self.len() as i32 - 1;
            }
            self.focus = f as usize;
            Some(self.ring[self.focus])
        }
    }

    // returns (is empty due to deletion, id if it needs to be switched in)
    fn delete(&mut self, id: Xid) -> (bool, Option<Xid>) {
        let ol = self.ring.len();
        self.ring.retain(|&e| e != id);
        let nl = self.ring.len();
        if ol == nl { return (false, None); }
        let mut f = (self.focus as i32) - 1;
        if f < 0 {
            f = self.len() as i32 - 1;
        }
        if f < 0 {
            f = 0;
        }
        self.focus = f as usize;
        if self.ring.is_empty() {
            (true, None)
        } else {
            (false, Some(self.ring[self.focus]))
        }
    }

    fn swap(&mut self, right: bool) {
        if self.len() < 2 { return; }
        if right {
            if self.focus == self.ring.len() - 1 {
                let x = self.ring.pop().unwrap();
                self.ring.insert(0, x);
                self.focus = 0;
            } else {
                self.ring.swap(self.focus, self.focus + 1);
                self.focus += 1;
            }
        } else if self.focus == 0 {
            let x = self.ring.remove(0);
            self.ring.push(x);
            self.focus = self.ring.len() - 1;
        } else {
            self.ring.swap(self.focus - 1, self.focus);
            self.focus -= 1;
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Rings {
    tags: [(Ring, Ring); 4],
    tag_indices: HashMap<String, usize>,
    tag_names: [String; 4],
    last_focus: Option<Xid>,
    scratchpad: Option<Xid>,
    fullscreen: HashSet<Xid>,
}

impl Rings {
    fn new() -> Self {
        let mut rings = Self::default();
        for (i, tag) in ["g", "m", "l", "w"].iter().enumerate() {
            rings.tag_indices.insert(tag.to_string(), i);
            rings.tag_names[i] = tag.to_string();
        }
        rings
    }

    fn current_view(&self) -> Vec<(String, Vec<Xid>)> {
        let mut res = Vec::new();
        for (i, tname) in self.tag_names.iter().enumerate() {
            let mut tag_windows = Vec::new();
            let (l, r) = &self.tags[i];
            let (l, r) = (l.focus(), r.focus());
            if let Some(l) = l { tag_windows.push(l); }
            if let Some(r) = r { tag_windows.push(r); }
            res.push((tname.clone(), tag_windows));
        }
        res
    }

    fn is_focused_in_a_ring(&self, id: Xid) -> bool {
        for (i, _) in self.tag_names.iter().enumerate() {
            let (l, r) = &self.tags[i];
            if l.focus() == Some(id) || r.focus() == Some(id) {
                return true;
            }
        }
        false
    }

    fn insert(&mut self, id: Xid, focused: Option<Xid>, ws_label: &str) {
        if let Some(index) = self.tag_indices.get(ws_label) {
            let (l, r) = &mut self.tags[*index];
            if l.len() == 0 {
                l.insert(id);
            } else if r.len() == 0 || r.focus() == focused {
                r.insert(id);
            } else {
                l.insert(id);
            }
        }
    }

    // returns newly focused on id
    fn rotate(&mut self, focused: Xid, ws_label: &str, right: bool) -> Option<Xid> {
        if let Some(index) = self.tag_indices.get(ws_label) {
            let (l, r) = &mut self.tags[*index];
            if l.focus() == Some(focused) {
                l.rotate(right)
            } else if r.focus() == Some(focused) {
                r.rotate(right)
            } else {
                None
            }
        } else {
            None
        }
    }

    // returns Option<to be focused id>
    fn delete(&mut self, id: Xid) -> Option<Xid> {
        for (i, _) in self.tag_names.iter().enumerate() {
            let (l, r) = &mut self.tags[i];
            match l.delete(id) {
                // currently it is illegal to have a client in multiple rings
                (false, Some(fid)) => return Some(fid),
                (true, None) => std::mem::swap(l, r),
                _ => {  },
            }
            if let (_, Some(fid)) = r.delete(id) {
                return Some(fid);
            }
        }
        None
    }

    // returns true if a swap occured
    fn swap_cols(&mut self, ws_label: &str) -> bool {
        if let Some(index) = self.tag_indices.get(ws_label) {
            let (l, r) = &mut self.tags[*index];
            if l.len() > 0 && r.len() > 0 {
                std::mem::swap(l, r);
                return true;
            }
        }
        false
    }

    fn swap_ring(&mut self, focused: Option<Xid>, ws_label: &str, right: bool) {
        if focused.is_none() { return; }
        if let Some(index) = self.tag_indices.get(ws_label) {
            let (l, r) = &mut self.tags[*index];
            if l.focus() == focused {
                l.swap(right)
            } else if r.focus() == focused {
                r.swap(right)
            }
        }
    }
}

fn rebuild(rings: Arc<RefCell<Rings>>, cs: &mut StackSet<Xid>) {
    println!("rebuild!");
    let rings_state = rings.borrow().current_view();

    for (tname, tview) in rings_state {
        println!("  {tname}");
        let wcs = cs.workspace(&tname).unwrap().clients().copied().collect::<Vec<_>>();
        for xid in wcs {
            cs.move_client_to_tag(&xid, "reikai");
        }
        let mut put_on_screen = Vec::with_capacity(2);
        for xid in tview.into_iter().rev() {
            if rings.borrow().fullscreen.contains(&xid) {
                put_on_screen.clear();
                put_on_screen.push(xid);
                break;
            }
            put_on_screen.push(xid);
        }
        for xid in put_on_screen {
            cs.move_client_to_tag(&xid, &tname);
            println!("    {xid}");
        }
    }
}

fn rings_manage<X: XConn + 'static>(id: Xid, state: &mut State<X>, x: &X) -> Result<()> {
    if x.query_or(false, &AppName("shapebar"), id) { return Ok(()) }
    let rings = state.extension::<Rings>()?;
    let cs = &mut state.client_set;
    let ws = cs.current_workspace();
    let fc = rings.borrow().last_focus;
    rings.borrow_mut().insert(id, fc, ws.tag());
    rebuild(rings.clone(), cs);
    println!("c: fc rings_manage: {}", id);
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

pub fn rings_event<X: XConn + 'static>(event: &XEvent, state: &mut State<X>, x: &X) -> Result<bool> {
    let rings = state.extension::<Rings>()?;
    let cs = &mut state.client_set;
    match event {
        XEvent::Destroy(id) => {
            let sid = rings.borrow().scratchpad;
            if let Some(sid) = sid {
                if sid == *id {
                    rings.borrow_mut().scratchpad = None;
                }
            }
            let res = rings.borrow_mut().delete(*id);
            if let Some(fid) = res {
                rebuild(rings, cs);
                println!("c: fc rings_event destroy: {}", fid);
                cs.focus_client(&fid);
                x.refresh(state)?;
            }
        },
        XEvent::ConfigureNotify(conf_event) => {
            // println!("config notify event!");
            let xid = conf_event.id;
            let net_wm_state = Atom::NetWmState.as_ref();
            let full_screen = x.intern_atom(Atom::NetWmStateFullscreen.as_ref())?;
            let wstate = match x.get_prop(xid, net_wm_state) {
                Ok(Some(Prop::Cardinal(vals))) => vals,
                _ => vec![],
            };
            let was_fullscreen = rings.borrow().fullscreen.contains(&xid);
            let is_fullscreen = wstate.contains(&full_screen);
            let update = was_fullscreen != is_fullscreen;

            // println!("{}: {}, {}, {}", xid, was_fullscreen, is_fullscreen, update);
            // TODO
            if update {
                if is_fullscreen {
                    rings.borrow_mut().fullscreen.insert(xid);
                } else {
                    rings.borrow_mut().fullscreen.remove(&xid);
                }
                println!("c: fc rings_event !!!: {}", xid);
                rebuild(rings, cs);
                cs.focus_client(&xid);
                x.refresh(state)?;
                println!("UPDATED: {}", xid);
            }
            // cs.focus_client(&xid);
        },
        _ => { },
    }
    Ok(true)
}

fn ring_rotate<X: XConn>(right: bool) -> Box<dyn KeyEventHandler<X>> {
    key_handler(move |state, x: &X| {
        let rings = state.extension::<Rings>()?;
        let sid = rings.borrow().scratchpad;
        let cs = &mut state.client_set;
        let wstag = cs.current_workspace().tag().to_string();
        let fc = cs.current_client().copied();
        if let Some(fid) = fc {
            if let Some(sid) = sid {
                if fid == sid {
                    let res = rings.borrow_mut().delete(sid);
                    if let Some(nfid) = res {
                        rebuild(rings.clone(), cs);
                        println!("c: fc rings_rotate scratchpad: {}", nfid);
                        cs.focus_client(&nfid);
                        return x.refresh(state);
                    }
                }
            }
            let nfid = rings.borrow_mut().rotate(fid, &wstag, right);
            rebuild(rings.clone(), cs);
            if let Some(nfid) = nfid {
                println!("c: fc rings_rotate fullscreen some: {}", nfid);
                cs.focus_client(&nfid);
            } else {
                println!("c: fc rings_rotate fullscreen none: {}", fid);
                cs.focus_client(&fid);
            }
            return x.refresh(state);
        }
        Ok(())
    })
}

fn swap_cols<X: XConn>() -> Box<dyn KeyEventHandler<X>> {
    key_handler(move |state, x: &X|{
        let rings = state.extension::<Rings>()?;
        let cs = &mut state.client_set;
        let wstag = cs.current_workspace().tag();
        let need_swap = rings.borrow_mut().swap_cols(wstag);
        if need_swap {
            cs.swap_down();
            let _ = x.refresh(state);
        }
        Ok(())
    })
}

fn swap_ring<X: XConn>(right: bool) -> Box<dyn KeyEventHandler<X>> {
    key_handler(move |state, _: &X|{
        let rings = state.extension::<Rings>()?;
        let cs = &mut state.client_set;
        let wstag = cs.current_workspace().tag();
        let fc = cs.current_client().copied();
        rings.borrow_mut().swap_ring(fc, wstag, right);
        Ok(())
    })
}

fn toggle_scratchpad<X: XConn>() -> Box<dyn KeyEventHandler<X>> {
    key_handler(move |state, x: &X| {
        let rings = state.extension::<Rings>()?;
        let sid = if let Some(sid) = rings.borrow().scratchpad { sid }
        else { return Ok(()); };
        let cs = &mut state.client_set;
        let on = rings.borrow().is_focused_in_a_ring(sid);
        let wstag = cs.current_workspace().tag().to_string();
        let focused = cs.current_client().copied();
        if on {
            let res = rings.borrow_mut().delete(sid);
            rebuild(rings.clone(), cs);
            let mut refresh = false;
            if let Some(ofid) = focused {
                if ofid == sid {
                    refresh = true;
                }
            }
            if let Some(nfid) = res {
                println!("c: fc toggle_scratchpad on: {}", nfid);
                cs.focus_client(&nfid);
                rings.borrow_mut().last_focus = Some(nfid);
                refresh = true;
            }
            if refresh {
                return x.refresh(state);
            }
        }
        // let _ = rings.borrow_mut().delete(sid);
        rings.borrow_mut().insert(sid, focused, &wstag);
        rebuild(rings.clone(), cs);
        println!("c: fc toggle_scratchpad end: {}", sid);
        cs.focus_client(&sid);
        rings.borrow_mut().last_focus = Some(sid);
        x.refresh(state)
    })
}

fn link_scratchpad<X: XConn>() -> Box<dyn KeyEventHandler<X>> {
    key_handler(move |state, _: &X| {
        let rings = state.extension::<Rings>()?;
        let cs = &mut state.client_set;
        let focused = cs.current_client().copied();
        if let Some(fid) = focused {
            let sp = rings.borrow().scratchpad;
            if let Some(sp) = sp {
                if sp == fid {
                    rings.borrow_mut().scratchpad = None;
                    return Ok(());
                }
            }
            rings.borrow_mut().scratchpad = Some(fid);
        }
        Ok(())
    })
}

fn log_status<X: XConn>() -> Box<dyn KeyEventHandler<X>> {
    key_handler(move |state, _: &X| {
        println!("status:");
        let rings = state.extension::<Rings>()?;
        let rings = rings.borrow();
        println!("in rings: ");
        for (i, (l, r)) in rings.tags.iter().enumerate() {
            println!(" {}:", rings.tag_names[i]);
            print!("  l: ");
            for id in &l.ring {
                print!("{}, ", id);
            }
            println!();
            print!("  r: ");
            for id in &r.ring {
                print!("{}, ", id);
            }
            println!();
        }
        println!("in tags: ");
        for tag in rings.tag_names.iter().chain([&"reikai".to_string()]) {
            let ws = state.client_set.workspace(tag).unwrap();
            print!(" {}: ", tag);
            for id in ws.clients() {
                print!("{}, ", id);
            }
            println!();
        }
        Ok(())
    })
}

fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").finish().init();

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
    config.compose_or_set_event_hook(rings_event);

    let mut wm = WindowManager::new(config, key_bindings, HashMap::new(), conn)?;
    wm.state.add_extension(OgWindowSize::default());
    wm.state.add_extension(Rings::new());
    wm.state.client_set.add_invisible_workspace("reikai")?;

    wm.run()
}
