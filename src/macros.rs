
/// Defines a reflective enum that can be iterated and also used
/// as key type for hash maps
///
/// #Examples
///
/// ```
/// #[macro_use]
/// extern crate ows;
///
/// use std::collections::HashMap;
///
/// iterable_key_enum! {
///     Direction =>
///         North,
///         East,
///         South,
///         West
/// }
///
/// fn main() {
///     let mut dirs = HashMap::new();
///     for d in Direction::variants() {
///         dirs.insert(d, format!("{:?}", d));
///     }
///
///     println!("there are {} directions:", Direction::num_variants());
///     for (_, dname) in dirs {
///         println!("  - {}", dname);
///     }
/// }
/// ```
#[macro_export]
macro_rules! iterable_key_enum {

    ( $name:ident => $( $val:ident ),* ) => {
        use std::slice::Iter;

        #[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
        enum $name {
            $( $val ),*
        }

        impl $name {
            fn variants() -> Iter<'static, $name> {
                static VARIANTS: &'static [$name] =
                        &[$($name::$val),*];
                VARIANTS.iter()
            }

            fn num_variants() -> usize {
                [$($name::$val),*].len()
            }
        }
    };

}



#[macro_export]
macro_rules! define_event {
    ($name:ident : Fn ($($pn:ident: $pt:ty),*) ) => {
        pub struct $name {
            handlers: Vec<Box<Fn($($pt),*)>>,
        }
        impl $name {
            pub fn new() -> $name {
                $name { handlers: Vec::new() }
            }
            pub fn new_with(handler: Box<Fn($($pt),*)>) -> $name {
                $name { handlers: vec![handler] }
            }
            pub fn new_with_n(handlers: Vec<Box<Fn($($pt),*)>>) -> $name {
                $name { handlers: handlers }
            }
            pub fn is_empty(&self) -> bool {
                self.handlers.is_empty()
            }
            pub fn len(&self) -> usize {
                self.handlers.len()
            }
            pub fn add(&mut self, h: Box<Fn($($pt),*)>) -> usize {
                let id = $name::h_to_id(h.as_ref());
                self.handlers.push(h);
                id
            }
            pub fn remove(&mut self, id: usize) -> bool {
                let len_bef = self.handlers.len();
                self.handlers.retain(|h| $name::h_to_id(h.as_ref()) != id);
                len_bef > self.handlers.len()
            }
            pub fn fire(&self, $($pn: $pt),*) {
                for h in &self.handlers {
                    $(let $pn = $pn.clone();)*
                    (*h)($($pn),*);
                }
            }
            fn h_to_id(h: &Fn($($pt),*)) -> usize {
                // FIXME: make something more robust
                use std::mem;
                let hash: (usize, usize) = unsafe { mem::transmute(h) };
                hash.0 ^ hash.1
            }
        }
    };
    ($name:ident : FnMut ($($pn:ident: $pt:ty),*) ) => {
        pub struct $name {
            handlers: Vec<Box<FnMut($($pt),*)>>,
        }
        impl $name {
            pub fn new() -> $name {
                $name { handlers: Vec::new() }
            }
            pub fn new_with(handler: Box<FnMut($($pt),*)>) -> $name {
                $name { handlers: vec![handler] }
            }
            pub fn new_with_n(handlers: Vec<Box<FnMut($($pt),*)>>) -> $name {
                $name { handlers: handlers }
            }
            pub fn is_empty(&self) -> bool {
                self.handlers.is_empty()
            }
            pub fn len(&self) -> usize {
                self.handlers.len()
            }
            pub fn add(&mut self, h: Box<FnMut($($pt),*)>) -> usize {
                let id = $name::h_to_id(h.as_ref());
                self.handlers.push(h);
                id
            }
            pub fn remove(&mut self, id: usize) -> bool {
                let len_bef = self.handlers.len();
                self.handlers.retain(|h| $name::h_to_id(h.as_ref()) != id);
                len_bef > self.handlers.len()
            }
            pub fn fire(&mut self, $($pn: $pt),*) {
                for h in &mut self.handlers {
                    $(let $pn = $pn.clone();)*
                    (*h)($($pn),*);
                }
            }
            fn h_to_id(h: &FnMut($($pt),*)) -> usize {
                // FIXME: make something more robust
                use std::mem;
                let hash: (usize, usize) = unsafe { mem::transmute(h) };
                hash.0 ^ hash.1
            }
        }
    };
}

macro_rules! define_handler {
    ($name:ident : Fn ($($pn:ident: $pt:ty),*) => $ret:ty) => {
        pub struct $name {
            handler: Option<Box<Fn($($pt),*) -> $ret>>,
        }
        impl $name {
            pub fn new() -> $name {
                $name { handler: None }
            }
            pub fn new_with(handler: Box<Fn($($pt),*)->$ret>) -> $name {
                $name { handler: Some(handler) }
            }
            pub fn is_set(&self) -> bool {
                self.handler.is_some()
            }
            pub fn set(&mut self, handler: Option<Box<Fn($($pt),*) -> $ret>>) {
                self.handler = handler;
            }
            pub fn fire(&self, $($pn: $pt),*) -> Option<$ret> {
                self.handler.as_ref().map(|handler| (*handler)($($pn),*))
            }
            pub fn fire_or(&self, def:$ret, $($pn: $pt),*) -> $ret {
                self.handler.as_ref().map_or(def, |handler| (*handler)($($pn),*))
            }
        }
    };


    ($name:ident : FnMut ($($pn:ident: $pt:ty),*) => $ret:ty) => {
        pub struct $name {
            handler: Option<Box<FnMut($($pt),*) -> $ret>>,
        }
        impl $name {
            pub fn new() -> $name {
                $name { handler: None }
            }
            pub fn new_with(handler: Box<FnMut($($pt),*)->$ret>) -> $name {
                $name { handler: Some(handler) }
            }
            pub fn is_set(&self) -> bool {
                self.handler.is_some()
            }
            pub fn set(&mut self, handler: Option<Box<FnMut($($pt),*) -> $ret>>) {
                self.handler = handler;
            }
            pub fn fire(&mut self, $($pn: $pt),*) -> Option<$ret> {
                self.handler.as_mut().map(|handler| (*handler)($($pn),*))
            }
            pub fn fire_or(&mut self, def:$ret, $($pn: $pt),*) -> $ret {
                self.handler.as_mut().map_or(def, |handler| (*handler)($($pn),*))
            }
        }
    };

}

#[macro_export]
macro_rules! handler_do {
    ($handler:expr, $closure:expr) => {{
        // A lifetime error occurs without this no-op let.
        let handler = $handler;
        handler.borrow_mut().set(Some(Box::new($closure)));
    }};
}

#[macro_export]
macro_rules! handler_do_nothing {
    ($handler:expr) => {{
        // A lifetime error occurs without this no-op let.
        let handler = $handler;
        handler.borrow_mut().set(None);
    }};
}

#[macro_export]
macro_rules! event_add {
    ($event:expr, $closure:expr) => {{
        // A lifetime error occurs without this no-op let.
        let event = $event;
        let id = event.borrow_mut().add(Box::new($closure));
        id
    }};
}

#[macro_export]
macro_rules! event_rem {
    ($event:expr, $id:expr) => {{
        // A lifetime error occurs without this no-op let.
        let event = $event;
        let res = event.borrow_mut().remove($id);
        res
    }};
}


#[macro_export]
macro_rules! event_fire {
    ($event:expr, $($p:expr),*) => {{
        // A lifetime error occurs without this no-op let.
        let event = $event;
        event.borrow_mut().fire($($p),*);
    }};
}

#[macro_export]
macro_rules! handler_fire {
    ($handler:expr, $($p:expr),*) => {{
        // A lifetime error occurs without this no-op let.
        let handler = $handler;
        let res = handler.borrow_mut().fire($($p),*);
        res
    }};
}

#[macro_export]
macro_rules! handler_fire_or {
    ($handler:expr, $($p:expr),+) => {{
        // A lifetime error occurs without this no-op let.
        let handler = $handler;
        let res = handler.borrow_mut().fire_or($($p),+);
        res
    }};
}


#[allow(dead_code)]
mod test {

    define_event!{ EventParam: Fn(n:i32) }

    #[test]
    fn event_param() {
        let mut ev = EventParam::new();
        assert!(h.is_empty());
        ev.fire(15);

        let sub = h.add(Box::new(|_| { }));
        let add = h.add(Box::new(|_| { }));
        assert_eq!(2, ev.len());
        assert!(!ev.is_empty());

        assert!(ev.remove(sub));
        assert!(ev.remove(add));
        assert!(ev.is_empty());
    }


    define_event!{ EventMutParam: FnMut(n:i32) }

    #[test]
    fn event_mut_param() {
        use std::rc::Rc;
        use std::cell::Cell;

        let tracer = Rc::new(Cell::new(0));

        let mut ev = EventMutParam::new();
        assert!(ev.is_empty());
        ev.fire(15);
        assert_eq!(0, tracer.get());

        let sub = {
            let tracer = tracer.clone();
            ev.add(Box::new(move |n| {
                let t = tracer.get();
                tracer.set( t - 2*n );
            }))
        };
        let add = {
            let tracer = tracer.clone();
            ev.add(Box::new(move |n| {
                let t = tracer.get();
                tracer.set( t + 3*n );
            }))
        };

        assert_eq!(2, ev.len());
        assert!(!ev.is_empty());
        ev.fire(6);
        assert_eq!(6, tracer.get());

        assert!(ev.remove(sub));
        ev.fire(7);
        assert_eq!(27, tracer.get());

        assert!(ev.remove(add));
        assert!(ev.is_empty());
    }



    define_handler!{ Handler: Fn() => i32 }

    #[test]
    fn handler() {
        let mut h = Handler::new();
        assert!(!h.is_set());
        assert_eq!(40, h.fire_or(40));
        assert!(h.fire().is_none());

        h.set(Some(Box::new(|| 37)));
        assert!(h.is_set());
        assert_eq!(37, h.fire_or(40));
        assert_eq!(37, h.fire().unwrap());
    }


    define_handler!{ HandlerParam: Fn(n:i32, s:&str) => String }

    #[test]
    fn handler_param() {
        let mut h = HandlerParam::new();
        assert_eq!("32 / a string",
            h.fire_or("32 / a string".to_string(), 54, "another string"));
        assert!(h.fire(54, "another_string").is_none());

        h.set(Some(Box::new(|n, s| format!("{} / {}", n, s))));
        assert!(h.is_set());
        assert_eq!("54 / another string",
            h.fire_or("32 / a string".to_string(), 54, "another string"));
        assert_eq!("54 / another string",
            h.fire(54, "another string").unwrap());
    }


    define_handler!{ HandlerMut: FnMut() => i32 }

    #[test]
    fn handler_mut() {
        let mut h = HandlerMut::new();
        assert!(!h.is_set());
        assert_eq!(40, h.fire_or(40));
        assert!(h.fire().is_none());

        h.set(Some(Box::new(|| 37)));
        assert!(h.is_set());
        assert_eq!(37, h.fire_or(40));
        assert_eq!(37, h.fire().unwrap());
    }


    define_handler!{ HandlerMutParam: FnMut(n:i32, s:&str) => String }

    #[test]
    fn handler_mut_param() {
        let mut h = HandlerMutParam::new();
        assert_eq!("32 / a string",
            h.fire_or("32 / a string".to_string(), 54, "another string"));
        assert!(h.fire(54, "another string").is_none());

        h.set(Some(Box::new(|n, s| format!("{} / {}", n, s))));
        assert!(h.is_set());
        assert_eq!("54 / another string",
            h.fire_or("32 / a string".to_string(), 54, "another string"));
        assert_eq!("54 / another string",
            h.fire(54, "another string").unwrap());
    }
}

