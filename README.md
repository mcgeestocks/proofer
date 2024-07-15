# Proofer

A simple project to explore rust and help my brutal spelling. Written mostly for learning and _vibz_.

## How it works

1. It captures text from the clipboard when activated by a global hotkey **(Ctrl+Shift+P).**
2. The captured text is sent to a local language model (using the Ollama API) for proofreading.
3. The corrected text extracted and returned back into the clipboard.
4. A helper system notification indicates the proofreading is complete.

## Development / Usage

[Tauri](https://github.com/tauri-apps) is a scaffolding tool for creating desktop applications. You can build with:

```bash
$ cargo tauri build
```

or if you just want to run without installing:

```bash
$ cargo tauri dev
```

The app expects a Ollama to be installed with `deepseek-coder-v2:16b-lite-instruct-q8_0` and served at `http://localhost:11434`.

# TODO:

- [ ] Add a configuration file for the Ollama API endpoint.
- [ ] Add a configuration file for the global hotkey.
- [ ] Try out [Candle](https://github.com/huggingface/candle) to remove extra dependencies.
