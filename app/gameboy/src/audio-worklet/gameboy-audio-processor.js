// eslint-disable-next-line no-undef
class GameBoyAudioProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super(options);
    this.audioBuffer = [];
    this.streamActive = true;

    // eslint-disable-next-line no-undef
    const writableStream = new WritableStream({
      write: (chunk) => {
        this.audioBuffer.push(...chunk);
      },
      close: () => {
        this.streamActive = false;
      },
      abort: () => {
        this.streamActive = false;
      },
    });
    this.port.postMessage(
      { type: "stream-prepared", payload: writableStream },
      [writableStream],
    );
  }

  process(_inputs, outputs, _parameters) {
    const audioBuffer = this.audioBuffer;
    const channels = outputs[0];
    const channelSamplesCount = channels[0].length;

    if (audioBuffer.length < channelSamplesCount * 2) {
      return this.streamActive;
    }

    const samplesCount = Math.min(audioBuffer.length, channelSamplesCount * 2);
    const samples = audioBuffer.splice(0, samplesCount);

    for (let i = 0, index = 0; i < samplesCount; i += 2, index += 1) {
      channels[0][index] = samples[i];
      channels[1][index] = samples[i + 1];
    }

    return this.streamActive;
  }
}

// eslint-disable-next-line no-undef
registerProcessor("gameboy-audio-processor", GameBoyAudioProcessor);
