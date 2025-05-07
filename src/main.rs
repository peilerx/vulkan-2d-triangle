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

#[derive(Default)]
struct App {
    pub entry: Entry,
} //basic init vulkan resources

impl App {}

fn main() {
    let mut app = App::default();
    app.entry = Entry::linked();

    println!("Hello, ash!");
}
