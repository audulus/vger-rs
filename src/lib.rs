
use wgpu::*;
use winit::*;
use futures::executor::block_on;

pub struct VGER {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue
}

impl VGER {

    async fn setup(window: &winit::window::Window) -> (wgpu::Device, wgpu::Queue) {

        let backend = wgpu::Backends::all();
        let instance = wgpu::Instance::new(backend);
        let (size, surface) = unsafe {
            let size = window.inner_size();
            let surface = instance.create_surface(&window);
            (size, surface)
        };

        let adapter =
        wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
            .await
            .expect("No suitable GPU adapters found on the system!");

        let adapter_info = adapter.get_info();
        println!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

        let trace_dir = std::env::var("WGPU_TRACE");
        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                },
                trace_dir.ok().as_ref().map(std::path::Path::new),
            )
            .await
            .expect("Unable to find a suitable GPU adapter!")
    }

    fn new(window: &winit::window::Window) -> Self {
        let (device, queue) = block_on(VGER::setup(window));

        Self { device: device, queue: queue }
    }

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
