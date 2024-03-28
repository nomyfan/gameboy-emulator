class GameBoyAudioProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.audioBuffer = [];
    this.port.onmessage = (evt) => {
      if (evt.data instanceof Float32Array) {
        this.audioBuffer.push(...evt.data);
      }
    };
  }

  process(inputs, outputs, parameters) {
    const audioBuffer = this.audioBuffer;
    if (!audioBuffer.length) return true;

    const channels = outputs[0];
    const channelSamplesCount = channels[0].length;
    const len = Math.min(audioBuffer.length, channelSamplesCount * 2);
    const slice = audioBuffer.splice(0, len);

    const end = Math.min(slice.length, channelSamplesCount);
    for (let i = 0, index = 0; i < end; i += 2, index += 1) {
      channels[0][index] = slice[i];
      channels[1][index] = slice[i + 1];
    }

    return true;
  }
}

registerProcessor("GameBoyAudioProcessor", GameBoyAudioProcessor);
