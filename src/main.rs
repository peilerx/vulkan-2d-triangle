use std::ptr::null;

use ash::{Entry, Instance, vk};

use ash_window;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct Swapchain {} //vulkan swapchain resources 

struct Pipeline {} //vulkan pipeline resources

struct Render {} //render resources

struct App {
    pub entry: Entry,
} //basic init vulkan resources

impl App {
    pub fn new() -> Self {
        Self {
            entry: Entry::linked(),
        }
    }
}

fn main() {
    let _app = App::new();
    _app.entry.static_fn();
    println!("Hello, ash!");
}
