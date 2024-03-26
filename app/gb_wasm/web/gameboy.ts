import type { GameBoy as GameBoyHandle } from "gb_wasm_bindings";
import { new_gameboy } from "gb_wasm_bindings";

class Fps {
  frameCount = 0;
  lastTime = performance.now();
  fps = 0;

  stop() {
    this.frameCount = 0;
  }

  update() {
    if (this.frameCount === 0) {
      this.frameCount = 1;
      this.lastTime = performance.now();
      return;
    }

    this.frameCount++;
    const now = performance.now();
    if (now - this.lastTime >= 1000) {
      this.fps = (this.frameCount - 1) / ((now - this.lastTime) / 1000);
      // TODO: notify properties changes to subscriber
      document.getElementById("fps")!.textContent = `FPS: ${this.fps.toFixed(
        2
      )}`;
      this.frameCount = 1;
      this.lastTime = now;
    }
  }
}

class Monitor {
  fps: Fps = new Fps();
}

class GameBoy {
  private instance_?: GameBoyHandle;
  private status_: "playing" | "paused" | "installed" | "uninstalled" =
    "uninstalled";
  private playCallbackId_?: number;
  private drawCallbackId_?: number;
  private monitor_ = new Monitor();

  // TODO: improve typing to help the caller known this.instance_ is not undefined
  private ensureInstalled() {
    if (!this.instance_) {
      throw new Error("GameBoy is not installed");
    }
  }

  get status(): string {
    return this.status_;
  }

  install(rom: Uint8ClampedArray) {
    this.instance_ = new_gameboy(rom);
    this.status_ = "installed";
  }

  uninstall() {
    if (this.instance_) {
      this.pause();
    }
    if (this.instance_) {
      this.instance_.free();
      this.instance_ = undefined;
    }
    this.status_ = "uninstalled";
  }

  play(canvasContext: CanvasRenderingContext2D) {
    this.ensureInstalled();
    if (this.status_ === "playing") {
      return;
    }

    this.status_ = "playing";

    const drawCallback = () => {
      if (this.status_ !== "playing" || !this.instance_) {
        if (this.drawCallbackId_) {
          window.cancelAnimationFrame(this.drawCallbackId_);
          this.drawCallbackId_ = undefined;
        }
        this.monitor_.fps.stop();
        return;
      }

      if (this.instance_.draw(canvasContext)) {
        this.monitor_.fps.update();
      }
      this.drawCallbackId_ = window.requestAnimationFrame(drawCallback);
    };
    drawCallback();

    const playCallback = () => {
      if (this.status_ !== "playing" || !this.instance_) {
        return;
      }

      const start = performance.now();
      this.instance_!.play_with_clocks();
      const duration = performance.now() - start;
      this.playCallbackId_ = window.setTimeout(() => {
        playCallback();
      }, Math.max(0, 16 - duration));
    };
    playCallback();
  }

  pause() {
    this.ensureInstalled();
    this.status_ = "paused";
    this.monitor_.fps.stop();

    if (this.playCallbackId_) {
      window.clearTimeout(this.playCallbackId_);
      this.playCallbackId_ = undefined;
    }
    if (this.drawCallbackId_) {
      window.cancelAnimationFrame(this.drawCallbackId_);
      this.drawCallbackId_ = undefined;
    }
  }
}

export { GameBoy };
