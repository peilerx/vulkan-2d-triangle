use ash::prelude::VkResult;
use ash::{Entry, Instance, vk};
use std::os::raw::c_char;

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
    pub instance: Instance,
    pub window: Window;
} //basic init vulkan resources

impl App {
    pub fn new() -> VkResult<Self> {
        let entry = Entry::linked();
        let app_name = c"vulkan_2d_triangle";

        let app_info = vk::ApplicationInfo::default()
            .application_name(app_name)
            .application_version(0)
            .engine_name(app_name)
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let layer_name = [c"VK_LAYER_KHRONOS_validation"];
        let layers_names: Vec<*const c_char> = layer_name
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        // let create_info = vk::InstanceCreateInfo::default()
        //     .application_info(&app_info)
        //     .flags(vk::InstanceCreateFlags::empty())
        //     .enabled_layer_names(&layers_names)
        //     .enabled_extension_names();

        Ok(Self { entry })
    }
}

fn main() {
    let app = App::new();

}
