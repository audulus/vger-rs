use wgpu::*;

pub struct VGER {
    pub device: wgpu::Device,
}

impl VGER {

    /*
    fn new() -> Self {
        let backend = wgpu::Backends::all();
        let instance = wgpu::Instance::new(backend);
    }
    */
    
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
