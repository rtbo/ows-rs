use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wlc::protocol::wl_compositor::WlCompositor;
use wlc::ConnectError;
use wlp::xdg_shell::client::xdg_wm_base::{self, RequestsTrait as XdgWmReqs, XdgWmBase};

mod window;
use self::window::Window;

pub struct Display {
    shared: Rc<DisplayShared>,
}

impl Display {}

impl Drop for Display {
    fn drop(&mut self) {}
}

impl super::Display for Display {
    type Window = Window;
    type OpenError = ConnectError;

    fn open() -> Result<Display, ConnectError> {
        wlc::Display::connect_to_env().map(|(dpy, queue)| Display {
            shared: DisplayShared::new(dpy, queue),
        })
    }

    fn create_window(&self) -> Window {
        Window::new(self.shared.clone())
    }

    fn collect_events(&self) {
        let dpy = &self.shared.dpy;
        let mut queue = self.shared.queue.borrow_mut();

        dpy.flush().unwrap();
        queue
            .dispatch()
            .expect("Error occured during Wayland queue dispatch");

        // let guard = {
        //     let mut guard = queue.prepare_read();
        //     while guard.is_none() {
        //         queue.dispatch_pending().unwrap();
        //         guard = queue.prepare_read();
        //     }
        //     guard.unwrap()
        // };
        // dpy.flush().unwrap();
        // guard.read_events().unwrap();
        // queue.dispatch_pending().unwrap();
    }

    fn instance(&self) -> Arc<gfx_back::Instance> {
        self.shared.instance.clone()
    }
}

struct DisplayShared {
    dpy: wlc::Display,
    queue: RefCell<wlc::EventQueue>,
    queue_token: wlc::QueueToken,
    compositor: wlc::Proxy<WlCompositor>,
    xdg_shell: wlc::Proxy<XdgWmBase>,
    instance: Arc<gfx_back::Instance>,
}

impl DisplayShared {
    fn new(dpy: wlc::Display, mut queue: wlc::EventQueue) -> Rc<Self> {
        use wlc::GlobalManager;

        let globals = GlobalManager::new(&dpy);
        queue.sync_roundtrip().unwrap();

        let compositor = globals
            .instantiate_auto::<WlCompositor, _>(|np| np.implement(|_, _| {}, ()))
            .unwrap();
        let xdg_shell = globals
            .instantiate_auto::<XdgWmBase, _>(|np| {
                np.implement(
                    |ev, xdg_shell| match ev {
                        xdg_wm_base::Event::Ping { serial } => xdg_shell.pong(serial),
                    },
                    (),
                )
            })
            .unwrap();

        for (id, interface, version) in globals.list() {
            println!("{}: {} (version = {})", id, interface, version);
        }

        let token = queue.get_token();

        Rc::new(Self {
            dpy: dpy,
            queue: RefCell::new(queue),
            queue_token: token,
            compositor: compositor,
            xdg_shell: xdg_shell,
            instance: Arc::new(gfx_back::Instance::create("ows-rs app", 0)),
        })
    }
}
