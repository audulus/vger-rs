use wgpu::*;

pub struct VGER {
    pub device: wgpu::Device,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
