# ows-rs ownership model

WIP

Modeling ownership of a GUI system in the rust ownership system is challenging.

It is proposed here an ownership model that tries to make some sense.


## Display

A Display represent a backend window implementation (such as Wayland or Win32)
Owned by application. Known at compile time.
Can have private state shared with windows.

## Window

Created by Display and owned by application.
Can be (should be) on the stack.

## Event loop

Is part of Display, therefore owned by application.

## Scene

Owned by application
Borrowed by algorithms that mutate it (Style, Layout, ...)
