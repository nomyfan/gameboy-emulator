mod bus;
mod dma;
mod hram;
mod joypad;
mod serial;
mod timer;
mod wram;

use std::path::Path;

use crate::bus::Bus;
use anyhow::Ok;
use gb_cartridge::Cartridge;
use gb_cpu_sm83::Cpu;
use gb_cpu_sm83::CPU_PERIOD_NANOS;
use gb_ppu::PPU;
use gb_shared::command::Command;
use gb_shared::command::CommandReceiver;
use gb_shared::event::EventSender;
use gb_shared::Component;
use gb_shared::Memory;
use log::debug;
use timer::Timer;

pub struct GameBoy {
    cpu: Cpu<Bus>,
    ppu: Box<PPU<Bus>>,
    bus: Box<Bus>,
    // We need to hold it to make it live as long as the GameBoy.
    _timer: Box<Timer<Bus>>,
    command_receiver: Option<CommandReceiver>,
}

impl GameBoy {
    pub fn from_cartridge(cart: Cartridge) -> Self {
        let mut bus = Bus::new(cart);

        let cpu = Cpu::new(bus.clone());

        let ppu = Box::new(PPU::new(bus.clone()));
        bus.set_ppu(ppu.as_ref());
        let timer = Box::new(Timer::new(bus.clone()));
        bus.set_timer(timer.as_ref());

        Self { cpu, ppu, bus: Box::new(bus), _timer: timer, command_receiver: None }
    }

    pub fn try_from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let rom = std::fs::read(path.as_ref())?;
        Self::try_from_raw(rom)
    }

    pub fn try_from_raw(rom: Vec<u8>) -> anyhow::Result<Self> {
        let cart = rom.try_into()?;
        Ok(Self::from_cartridge(cart))
    }

    pub fn play(
        mut self,
        event_sender: EventSender,
        command_receiver: CommandReceiver,
    ) -> anyhow::Result<()> {
        self.ppu.set_event_sender(event_sender);
        self.command_receiver = Some(command_receiver);

        loop {
            if self.cpu.halted {
                if self.bus.read(0xFF0F) != 0 {
                    self.cpu.halted = false;
                }
                self.bus.step(4);
            } else {
                self.cpu.step();
            };

            std::thread::sleep(std::time::Duration::from_nanos(
                (CPU_PERIOD_NANOS * self.cpu.finish_cycles() as f64) as u64,
            ));

            debug!("{:?}", &self.cpu);

            if self.cpu.ime {
                self.cpu.handle_interrupts();
                self.cpu.enabling_ime = false;

                let cycles = self.cpu.finish_cycles();
                if cycles != 0 {
                    std::thread::sleep(std::time::Duration::from_nanos(
                        (CPU_PERIOD_NANOS * cycles as f64) as u64,
                    ));
                }
            }

            if self.cpu.enabling_ime {
                self.cpu.ime = true;
            }

            // Safety: we set the command_receiver at the start of `play` function.
            if let Some(command) = self.command_receiver.as_ref().unwrap().try_recv().ok() {
                if let Command::Exit = command {
                    return Ok(());
                }
                self.bus.handle_command(command);
            }
        }
    }
}
