//! Implementation of OAM DMA ($4014)
//! Reference: https://www.nesdev.org/wiki/DMA

pub struct DMA {
    pub page: u8,
    pub addr: u8,
    // Represents byte in transit from CPU -> OAM. 
    pub data: u8,

    // Represents if DMA is happening.
    pub dma_transfer: bool,
    // Used for synchronizing with CPU read/write cycles.
    pub dma_is_not_sync: bool,
}

impl DMA {
    pub fn new() -> Self {
        DMA {
            page: 0,
            addr: 0,
            data: 0,

            dma_transfer: false,
            dma_is_not_sync: false,
        }
    }

    pub fn write(&mut self, data: u8){
        self.dma_transfer = true;
        self.page = data;
        self.addr = 0x00;

    }
}