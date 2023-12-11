mod bus;
mod dma;
mod hram;
mod wram;

use std::path::Path;

use crate::bus::Bus;
use anyhow::Ok;
use gb_cartridge::Cartridge;
use gb_cpu_sm83::Cpu;
use gb_ppu::PPU;
use log::debug;

pub struct GameBoy {
    cpu: Cpu<Bus>,
    ppu: Box<PPU<Bus>>,
    bus: Box<Bus>,
}

impl GameBoy {
    pub fn from_cartridge(cart: Cartridge) -> Self {
        let mut bus = Bus::new(cart);

        let cpu = Cpu::new(bus.clone());

        let ppu = Box::new(PPU::new(bus.clone()));
        bus.set_ppu(ppu.as_ref());

        Self { cpu, ppu, bus: Box::new(bus) }
    }

    pub fn try_from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let rom = std::fs::read(path.as_ref())?;
        Self::try_from_raw(rom)
    }

    pub fn try_from_raw(rom: Vec<u8>) -> anyhow::Result<Self> {
        let cart = rom.try_into()?;
        Ok(Self::from_cartridge(cart))
    }

    pub fn play(self) -> anyhow::Result<()> {
        let mut gb = self;
        // TODO: loop and accept signals to stop
        for _ in 1..200 {
            debug!("{:?}", &gb.cpu);

            if gb.cpu.stopped {
                println!("Stopping...");
                // TODO
                std::process::exit(0);
            }

            let cycles = if gb.cpu.halted {
                // TODO
                1
            } else {
                gb.cpu.step()
            };

            for _ in 0..cycles {
                for _ in 0..4 {
                    gb.ppu.step();
                }

                gb.bus.step();
            }

            if gb.cpu.ime {
                gb.cpu.handle_interrupts();
            }
        }

        Ok(())
    }
}
