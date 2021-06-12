#[macro_use]
extern crate penrose;

use penrose::{
    contrib::{
        actions::create_or_switch_to_workspace,
        extensions::{Scratchpad},
        layouts::paper,
    },
    core::{
        bindings::KeyEventHandler,
        config::Config,
        helpers::index_selectors,
        hooks::Hooks,
        layout::{bottom_stack, side_stack, Layout, LayoutConf},
        xconnection::XConn,
    },
    logging_error_handler,
    xcb::new_xcb_backed_window_manager,
    Backward, Forward, Less, More, WindowManager, Selector::Focused, Result,
};

fn main() -> penrose::Result<()> {
    let sp = Scratchpad::new("st", 0.9, 0.9);

    let hooks: Hooks<_> = vec![
        sp.get_hook(),
    ];

    let key_bindings = gen_keybindings! {
        "M-u" => run_internal!(cycle_client, Forward);
        "M-e" => run_internal!(cycle_client, Backward);
        "M-S-u" => run_internal!(drag_client, Forward);
        "M-S-e" => run_internal!(drag_client, Backward);
        "M-f" => run_internal!(kill_client);
        "M-n" => run_internal!(toggle_client_fullscreen, &Focused);
        "M-Tab" => run_internal!(cycle_workspace, Forward);
        "M-a" => run_internal!(cycle_layout, Forward);
        "M-S-a" => run_internal!(cycle_layout, Backward);
        "M-space" => sp.toggle();
        "M-h" => run_external!("dmenu_run");
        "M-b" => run_external!("st");
        "M-j" => run_external!("firefox");
        "M-S-d" => run_internal!(exit);

        refmap [ 1..10 ] in {
            "M-{}" => focus_workspace [ index_selectors(9) ];
            "M-S-{}" => client_to_workspace [ index_selectors(9) ];
        };

        // map: { "q", "g", "m", "l", "w", "y" } to index_selectors(6) => {
        //     "M-{}" => focus_workspace (REF);
        //     "M-S-{}" => client_to_workspace (REF);
        // };
    };

    fn my_layouts() -> Vec<Layout> {
        let n_main = 1;
        let ratio = 0.5;
        let follow_focus_conf = LayoutConf {
            floating: false,
            gapless: false,
            follow_focus: true,
            allow_wrapping: false,
        };

        vec![
            Layout::new("[side]", LayoutConf::default(), side_stack, n_main, ratio),
            Layout::new("[papr]", follow_focus_conf, paper, n_main, ratio),
            Layout::floating("[float]"),
        ]
    }

    let config = Config::default().builder()
        .floating_classes(vec!["dmenu"])
        .gap_px(10)
        // .outer_gap(50)
        .border_px(4)
        // .focused_border(0x888888)
        // .unfocused_border(0x000000)
        .show_bar(true)
        .top_bar(true)
        .bar_height(26)
        .layouts(my_layouts())
        .build()
        .expect("failed to build config");

    let mut wm = new_xcb_backed_window_manager(
        config,
        hooks,
        logging_error_handler()
    )?;
    wm.grab_keys_and_run(key_bindings, map!{})
}
