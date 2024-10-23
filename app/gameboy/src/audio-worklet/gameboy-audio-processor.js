// eslint-disable-next-line no-undef
class GameBoyAudioProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super(options);
    /**
     * @type {{ offset: 0; chunk: Float32Array }[]}
     */
    this.audioBuffer = [];
    this.length = 0;

    this.port.onmessage = (evt) => {
      if (evt.data.type === "chunk") {
        const chunk = evt.data.chunk;
        this.audioBuffer.push({ offset: 0, chunk });
        this.length += chunk.length;
      }
    };
  }

  process(_inputs, outputs, _parameters) {
    const audioBuffer = this.audioBuffer;
    const channels = outputs[0];
    const channelSamplesCount = channels[0].length;

    if (this.length >= channelSamplesCount * 2) {
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
    }

    return true;
  }
}

// eslint-disable-next-line no-undef
registerProcessor("gameboy-audio-processor", GameBoyAudioProcessor);
