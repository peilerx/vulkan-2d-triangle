use ash::Device;
use ash::ext::debug_utils;
use ash::khr::surface;
use ash::prelude::VkResult;
use ash::{Entry, Instance, vk};
use ash_window;
use std::ffi::c_char;
use vk::Queue;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct Swapchain {
    pub loader: ash::khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
} //vulkan swapchain resources 

impl Swapchain {
    pub fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        surface: vk::SurfaceKHR,
        surface_loader: &ash::khr::surface::Instance,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
        window: &Window,
    ) -> VkResult<Self> {
        let surface_capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .unwrap()
        };

        print!(
            "Surface capabilities: min_images = {}, max_images = {}, extend = {:?} ",
            surface_capabilities.min_image_count,
            surface_capabilities.max_image_count,
            surface_capabilities.current_extent
        );

        Ok(Swapchain {
            loader: unimplemented!("none"),
            swapchain: unimplemented!("none"),
        })
    }
}
struct Pipeline {} //vulkan pipeline resources

struct Render {} //render resources

pub struct App {
    pub entry: Entry,
    pub instance: Instance,
    pub event_loop: EventLoop<()>,
    pub window: Window,
    pub surface: vk::SurfaceKHR,
    pub surface_loader: surface::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub queue_family_index: u32,
    pub device: Device,
    pub present_queue: Queue,
} //basic init vulkan resources

impl App {
    pub fn new(width: u32, height: u32) -> VkResult<Self> {
        let entry = Entry::linked(); //базовый ресурс Vulkan
        let app_name = c"vulkan_2d_triangle";

        let app_info = vk::ApplicationInfo::default()
            .application_name(app_name)
            .application_version(0)
            .engine_name(app_name)
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let layer_name = [c"VK_LAYER_KHRONOS_validation"]; //слой валидации для проверок ошибок Vulkan на этапе компиляции, можно добавить еще дополнительные слои например для подсчета FPS
        let layers_names: Vec<*const c_char> = layer_name //интепретация запись слоя в массив c_char, так как ash vk работает только с C
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let event_loop = EventLoop::new().unwrap(); //создаем цикл событий

        let window = WindowBuilder::new() //создаем окно
            .with_title("vulkan_2d_triangle")
            .with_inner_size(winit::dpi::LogicalSize::new(
                //логический размер окна с учетом DPI, автоматическое масштабирование
                f64::from(width),
                f64::from(height),
            ))
            .build(&event_loop) //помещаем цикл событий в окно
            .unwrap();

        let mut extension_names =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw()) //возвращаем дескриптор оконного менеджера в список расширений
                .unwrap()
                .to_vec();
        extension_names.push(debug_utils::NAME.as_ptr());

        let create_info = vk::InstanceCreateInfo::default() //передаем данные о расширениях и слоях в Instance
            .application_info(&app_info)
            .flags(vk::InstanceCreateFlags::empty())
            .enabled_layer_names(&layers_names)
            .enabled_extension_names(&extension_names);

        let instance: Instance =
            unsafe { entry.create_instance(&create_info, None) }.expect("Instance Create Error"); //Instance создается с помощью entry 

        let surface = unsafe {
            //создаем поверхность рендера
            ash_window::create_surface(
                &entry,
                &instance,
                window.display_handle().unwrap().as_raw(), //передаем дескриптор оконного менеджера wayland, x11 etc
                window.window_handle().unwrap().as_raw(), //передаем дескриптор нашего конкретного окна, экземпляр window
                None,
            )
        }
        .unwrap();

        let surface_loader = surface::Instance::new(&entry, &instance); //загрузчик поверхности рендера

        let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() }; //возвращается массив физических устройств 

        let (physical_device, queue_family_index) = physical_devices //получаем активное физическое устройство
            .into_iter() //перечисляем без сохранения всех физический устройств
            .find_map(|pdevice| unsafe {
                //получаем физическое устройство из списка
                {
                    instance
                        .get_physical_device_queue_family_properties(pdevice) //получаем свойства очереди активного устройста
                        .iter() //перечисляем их
                        .enumerate() //достаем индексы свойств
                        .find_map(|(index, info)| {
                            let supports_graphics =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS); //смотрим поддерживает ли свойство активного устройства данный флаг
                            let supports_surface = surface_loader
                                .get_physical_device_surface_support(pdevice, index as u32, surface) //смотрим поддерживает ли устройство данную поверхность рендера
                                .unwrap_or(false); //если нет верни false
                            if supports_graphics && supports_surface {
                                Some((pdevice, index as u32)) //если поддерживает верни устройство и индекс свойств семейства очереди 
                            } else {
                                None
                            }
                        })
                }
            })
            .expect("No physical device found");

        let device_extension_names_raw: Vec<*const c_char> = vec![]; //расширения устройства
        let priorities = [1.0_f32]; //приоритет очереди, первый

        let queue_info = vk::DeviceQueueCreateInfo::default() //информация для создания очереди устройства
            .queue_family_index(queue_family_index) //индекс очереди
            .queue_priorities(&priorities); //приоритет первый
        let queue_infos = [queue_info]; //вектор очередей 
        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&device_extension_names_raw);

        let device = unsafe {
            //создаем логическое устройство с помощью экземпляра instance
            instance
                .create_device(physical_device, &device_create_info, None) //передаем физическое устройство
                .unwrap()
        };

        let present_queue = unsafe { device.get_device_queue(queue_family_index, 0) }; //возвращаем очередь логического устройства,
        // очереди принимают SubmitInfo а SubmitInfo принимает массив CommandBuffer, массив комманд на выполнение на  GPU
        // первый параметр это индекс семейства очередей GRAPHICS или COMPUTE, PRESENT, TRANSFER  etc, второй параметр это индекс очереди, очередей в одном семействе может быть заданое количество

        Ok(Self {
            entry,
            instance,
            event_loop,
            window,
            surface,
            surface_loader,
            physical_device,
            queue_family_index,
            device,
            present_queue,
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

    let swapchain = Swapchain::new(
        &app.instance,
        &app.device,
        app.surface,
        &app.surface_loader,
        app.physical_device,
        app.queue_family_index,
        &app.window,
    )
    .unwrap();
}
