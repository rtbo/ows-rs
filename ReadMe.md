# OWS-rs
**_Open windowing system for Rust_**

This is more or less a research project to see what an IU toolkit _could_ look like with Rust.
Particularily how to make the lifetime system work with a traditional IU toolkit.

Eventually, if other people join the project, it will become an actual IU toolkit.

## Some general design specifications
 -  Application developer should not get bothered with `RefCell` interface
    -  If such interface is needed to share some state, it should be well hidden behind regular types

## Some top-level windows design specifications
 -  Platform abstraction with erased types
 -  Support for multiple non modal windows in th main event loop
 -  Event handlers with closures (observer pattern)
 -  Some closures can return value (in such case no more than one closure is connected)
 -  Possibility to create modal windows with an event handler (also non modal? TBD)
 -  access and modify ui elements within handlers
 -  provide an easy way to carry application state in windows

## Some widgets system design specifications
 -  lazy resources loading
 -  use of vector graphics primitives
 -  provide an easy way to carry application state in widgets
 -  ... TBD

## Some graphics system design specifications
 -  vector graphics
 -  hardware accelerated
 -  rendering in a dedicated thread
 -  usage of command buffers / drawing queues (no direct rendering from handlers)
