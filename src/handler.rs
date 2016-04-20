

#[macro_export]
macro_rules! define_handler {
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
            pub fn fire(&mut self, $($pn: $pt),*) -> $ret {
                self.handler.as_mut().map(|handler| (*handler)($($pn),*)).unwrap()
            }
            pub fn fire_or(&mut self, def:$ret, $($pn: $pt),*) -> $ret {
                self.handler.as_mut().map_or(def, |handler| (*handler)($($pn),*))
            }
        }
    };
}

#[allow(dead_code)]
mod define_handler_test {

    define_handler!{ Param: FnMut(n:i32) }

    #[test]
    fn param() {
        use std::rc::Rc;
        use std::cell::Cell;

        let tracer = Rc::new(Cell::new(0));

        let mut h = Param::new();
        assert!(h.is_empty());
        h.fire(15);
        assert_eq!(0, tracer.get());

        let sub = {
            let tracer = tracer.clone();
            h.add(Box::new(move |n| {
                let t = tracer.get();
                tracer.set( t - 2*n );
            }))
        };
        let add = {
            let tracer = tracer.clone();
            h.add(Box::new(move |n| {
                let t = tracer.get();
                tracer.set( t + 3*n );
            }))
        };

        assert_eq!(2, h.len());
        assert!(!h.is_empty());
        h.fire(6);
        assert_eq!(6, tracer.get());

        assert!(h.remove(sub));
        h.fire(7);
        assert_eq!(27, tracer.get());

        assert!(h.remove(add));
        assert!(h.is_empty());
    }


    define_handler!{ NoParamRet: FnMut() => i32 }

    #[test]
    fn no_param_ret() {
        let mut h = NoParamRet::new();
        assert!(!h.is_set());
        assert_eq!(40, h.fire_or(40));

        h.set(Some(Box::new(|| 37)));
        assert!(h.is_set());
        assert_eq!(37, h.fire_or(40));
        assert_eq!(37, h.fire());
    }


    define_handler!{ ParamRet: FnMut(n:i32, s:&str) => String }

    #[test]
    fn param_ret() {
        let mut h = ParamRet::new();
        assert_eq!("32 / a string",
            h.fire_or("32 / a string".to_string(), 54, "another string"));

        h.set(Some(Box::new(|n, s| format!("{} / {}", n, s))));
        assert!(h.is_set());
        assert_eq!("54 / another string",
            h.fire_or("32 / a string".to_string(), 54, "another string"));
        assert_eq!("54 / another string",
            h.fire(54, "another string"));
    }
}
