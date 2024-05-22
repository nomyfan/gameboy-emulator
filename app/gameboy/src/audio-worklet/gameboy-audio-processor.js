// eslint-disable-next-line no-undef
class GameBoyAudioProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super(options);
    /**
     * @type {{ offset: 0; chunk: Float32Array }[]}
     */
    this.audioBuffer = [];
    this.streamActive = true;
    this.length = 0;

    // eslint-disable-next-line no-undef
    const writableStream = new WritableStream({
      write: (chunk) => {
        this.audioBuffer.push({ offset: 0, chunk });
        this.length += chunk.length;
      },
      close: () => {
        this.streamActive = false;
      },
      abort: () => {
        this.streamActive = false;
      },
    });
    // Crash on Safari if sending streams
    this.port.postMessage(
      { type: "stream-prepared", payload: writableStream },
      [writableStream],
    );
  }

  process(_inputs, outputs, _parameters) {
    const audioBuffer = this.audioBuffer;
    const channels = outputs[0];
    const channelSamplesCount = channels[0].length;

    if (this.length < channelSamplesCount * 2) {
      return this.streamActive;
    }

    for (let i = 0; i < channelSamplesCount; i++) {
      if (audioBuffer[0].offset >= audioBuffer[0].chunk.length) {
        audioBuffer.shift();
      }

      const buffer = audioBuffer[0];
      channels[0][i] = buffer.chunk[buffer.offset];
      channels[1][i] = buffer.chunk[buffer.offset + 1];

      buffer.offset += 2;
      this.length -= 2;
    }

    return this.streamActive;
  }
}

// eslint-disable-next-line no-undef
registerProcessor("gameboy-audio-processor", GameBoyAudioProcessor);
