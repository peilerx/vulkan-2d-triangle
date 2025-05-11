use ash::ext::debug_utils;
use ash::khr::surface;
use ash::prelude::VkResult;
use ash::{Entry, Instance, vk};
use ash_window;
use std::os::raw::c_char;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct Swapchain {} //vulkan swapchain resources 

struct Pipeline {} //vulkan pipeline resources

struct Render {} //render resources

pub struct App {
    pub entry: Entry,
    pub instance: Instance,
    pub window: Window,
    pub surface: vk::SurfaceKHR,
    pub surface_loader: surface::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub queue_family_index: u32,
} //basic init vulkan resources

impl App {
    pub fn new(width: u32, height: u32) -> VkResult<Self> {
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

        let event_loop = EventLoop::new().unwrap();

        let window = WindowBuilder::new()
            .with_title("vulkan_2d_triangle")
            .with_inner_size(winit::dpi::LogicalSize::new(
                f64::from(width),
                f64::from(height),
            ))
            .build(&event_loop)
            .unwrap();

        let mut extension_names =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .unwrap()
                .to_vec();
        extension_names.push(debug_utils::NAME.as_ptr());

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .flags(vk::InstanceCreateFlags::empty())
            .enabled_layer_names(&layers_names)
            .enabled_extension_names(&extension_names);

        let instance: Instance =
            unsafe { entry.create_instance(&create_info, None) }.expect("Instance Create Error");

        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
        }
        .unwrap();

        let surface_loader = surface::Instance::new(&entry, &instance);

        let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() };

        let (physical_device, queue_family_index) = physical_devices
            .into_iter()
            .find_map(|pdevice| unsafe {
                {
                    instance
                        .get_physical_device_queue_family_properties(pdevice)
                        .iter()
                        .enumerate()
                        .find_map(|(index, info)| {
                            let supports_graphics =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                            let supports_surface = surface_loader
                                .get_physical_device_surface_support(pdevice, index as u32, surface)
                                .unwrap_or(false);
                            if supports_graphics && supports_surface {
                                Some((pdevice, index as u32))
                            } else {
                                None
                            }
                        })
                }
            })
            .expect("No physical device found");
        Ok(Self {
            entry,
            instance,
            window,
            surface,
            surface_loader,
            physical_device,
            queue_family_index,
        })
    }
}

fn main() {
    let app = App::new(800, 600).unwrap();
    let device_properties = unsafe {
        app.instance
            .get_physical_device_properties(app.physical_device)
    };
    let device_name = unsafe { std::ffi::CStr::from_ptr(device_properties.device_name.as_ptr()) };
    println!("Device name 1: {:?} ", device_name);
}
