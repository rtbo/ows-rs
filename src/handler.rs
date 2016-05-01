

#[macro_export]
macro_rules! define_handler {
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


    //($name:ident : FnOnce ($($pn:ident: $pt:ty),*) ) => {
    //    pub struct $name {
    //        handlers: Vec<Box<FnBox($($pt),*)>>,
    //    }
    //    impl $name {
    //        pub fn new() -> $name {
    //            $name { handlers: Vec::new() }
    //        }
    //        pub fn new_with(handler: Box<FnBox($($pt),*)>) -> $name {
    //            $name { handlers: vec![handler] }
    //        }
    //        pub fn new_with_n(handlers: Vec<Box<FnBox($($pt),*)>>) -> $name {
    //            $name { handlers: handlers }
    //        }
    //        pub fn is_empty(&self) -> bool {
    //            self.handlers.is_empty()
    //        }
    //        pub fn len(&self) -> usize {
    //            self.handlers.len()
    //        }
    //        pub fn add(&mut self, h: Box<FnBox($($pt),*)>) -> usize {
    //            let id = $name::h_to_id(h.as_ref());
    //            self.handlers.push(h);
    //            id
    //        }
    //        pub fn remove(&mut self, id: usize) -> bool {
    //            let len_bef = self.handlers.len();
    //            self.handlers.retain(|h| $name::h_to_id(h.as_ref()) != id);
    //            len_bef > self.handlers.len()
    //        }
    //        pub fn fire(self, $($pn: $pt),*) {
    //            for h in self.handlers {
    //                $(let $pn = $pn.clone();)*
    //                h.call_box($($pn),*);
    //            }
    //        }
    //        fn h_to_id(h: &FnBox($($pt),*)) -> usize {
    //            // FIXME: make something more robust
    //            use std::mem;
    //            let hash: (usize, usize) = unsafe { mem::transmute(h) };
    //            hash.0 ^ hash.1
    //        }
    //    }
    //};
    //($name:ident : FnOnce ($($pn:ident: $pt:ty),*) => $ret:ty) => {
    //    pub struct $name {
    //        handler: Option<Box<FnBox($($pt),*) -> $ret>>,
    //    }
    //    impl $name {
    //        pub fn new() -> $name {
    //            $name { handler: None }
    //        }
    //        pub fn new_with(handler: Box<FnBox($($pt),*)->$ret>) -> $name {
    //            $name { handler: Some(handler) }
    //        }
    //        pub fn is_set(&self) -> bool {
    //            self.handler.is_some()
    //        }
    //        pub fn set(&mut self, handler: Option<Box<FnBox($($pt),*) -> $ret>>) {
    //            self.handler = handler;
    //        }
    //        pub fn fire(self, $($pn: $pt),*) -> $ret {
    //            self.handler.as_mut().map(|handler| (*handler)($($pn),*)).unwrap()
    //        }
    //        pub fn fire_or(self, def:$ret, $($pn: $pt),*) -> $ret {
    //            self.handler.as_mut().map_or(def, |handler| (*handler)($($pn),*))
    //        }
    //    }
    //};
    //($name:ident : FnBox ($($pn:ident: $pt:ty),*) ) => {

    //    log_syntax!(define_handler!{$name : FnBox ($($pn: $pt),*) });
    //    define_handler!{$name : FnOnce ($($pn: $pt),*) }
    //};
    //($name:ident : FnBox ($($pn:ident: $pt:ty),*) => $ret:ty ) => {
    //    define_handler!{$name : FnOnce ($($pn: $pt),*) => $ret }
    //};
}


#[allow(dead_code)]
mod define_handler_fn_test {

    define_handler!{ Param: Fn(n:i32) }

    #[test]
    fn param() {
        let mut h = Param::new();
        assert!(h.is_empty());
        h.fire(15);

        let sub = h.add(Box::new(|_| { }));
        let add = h.add(Box::new(|_| { }));
        assert_eq!(2, h.len());
        assert!(!h.is_empty());

        assert!(h.remove(sub));
        assert!(h.remove(add));
        assert!(h.is_empty());
    }


    define_handler!{ NoParamRet: Fn() => i32 }

    #[test]
    fn no_param_ret() {
        let mut h = NoParamRet::new();
        assert!(!h.is_set());
        assert_eq!(40, h.fire_or(40));
        assert!(h.fire().is_none());

        h.set(Some(Box::new(|| 37)));
        assert!(h.is_set());
        assert_eq!(37, h.fire_or(40));
        assert_eq!(37, h.fire().unwrap());
    }


    define_handler!{ ParamRet: Fn(n:i32, s:&str) => String }

    #[test]
    fn param_ret() {
        let mut h = ParamRet::new();
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
}

#[allow(dead_code)]
mod define_handler_fnmut_test {

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
        assert!(h.fire().is_none());

        h.set(Some(Box::new(|| 37)));
        assert!(h.is_set());
        assert_eq!(37, h.fire_or(40));
        assert_eq!(37, h.fire().unwrap());
    }


    define_handler!{ ParamRet: FnMut(n:i32, s:&str) => String }

    #[test]
    fn param_ret() {
        let mut h = ParamRet::new();
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

//#[allow(dead_code)]
//mod define_handler_fnonce {
//
//    use std::boxed::FnBox;
//
//    define_handler!{ Param: FnOnce(n:i32) }
//
//    #[test]
//    fn param() {
//        use std::rc::Rc;
//        use std::cell::Cell;
//
//        let tracer = Rc::new(Cell::new(0));
//
//        let mut h = Param::new();
//        assert!(h.is_empty());
//
//        let sub = {
//            let tracer = tracer.clone();
//            h.add(Box::new(move |n| {
//                let t = tracer.get();
//                tracer.set( t - 2*n );
//            }))
//        };
//        let add = {
//            let tracer = tracer.clone();
//            h.add(Box::new(move |n| {
//                let t = tracer.get();
//                tracer.set( t + 3*n );
//            }))
//        };
//
//        assert_eq!(2, h.len());
//        assert!(!h.is_empty());
//        h.fire(6);
//        assert_eq!(6, tracer.get());
//
//    }
//
//
//    define_handler!{ NoParamRet: FnOnce() => i32 }
//
//    #[test]
//    fn no_param_ret() {
//        let mut h = NoParamRet::new();
//        assert!(!h.is_set());
//        h.set(Some(Box::new(|| 37)));
//        assert!(h.is_set());
//        assert_eq!(37, h.fire());
//    }
//
//
//    define_handler!{ ParamRet: FnMut(n:i32, s:&str) => String }
//
//    #[test]
//    fn param_ret() {
//        let mut h = ParamRet::new_with(Box::new(|n, s| format!("{} / {}", n, s)));
//        assert!(h.is_set());
//        assert_eq!("54 / another string", h.fire(54, "another string"));
//    }
//}
