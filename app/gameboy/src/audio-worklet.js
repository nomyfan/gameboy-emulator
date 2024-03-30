class GameBoyAudioProcessor extends AudioWorkletProcessor {
  constructor(options) {
    const processorOptions = options.processorOptions;
    const sampleRate = processorOptions.sampleRate;

    super(options);
    this.audioBuffer = [];
    this.streamActive = true;
    this.sampleRate = sampleRate;

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
    this.port.postMessage({ type: "stream", value: writableStream }, [
      writableStream,
    ]);
  }

  process(inputs, outputs, parameters) {
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

registerProcessor("GameBoyAudioProcessor", GameBoyAudioProcessor);
