# rTranslate

Yet another replacement for abandoned QuestSoft QTranslate. Lightweight (6-25 MB RAM footprint). Local offline translation and TTS supported (bergamot with firefox models and kokoro.js through Deno's runtime).
![popup_window](assets/scrnshts/s1.png) ![popup_dict](assets/scrnshts/s2.png)

## Features

- **Instant Translation:** Select text in any application and instantly translate it using a customizable shortcut.
- **Dictionaries:** Google, DSL (local)
- **Translator services:** Google Translate, DeepL, Bergamot (local)
- **Text-to-Speech (TTS):** Kokoro.js (local)
- **Pronunciations:** Google, Wiktionary
- **Extensions:** Supports extensions running as sidecar processes that communicate with the host application via stdio.
- **History & Favorites**

![main_window](assets/scrnshts/s3.png)

## Roadmap

- [ ] Offline OCR (tesseract)
- [ ] Wiktionary parser
- [ ] i18n
- [ ] Double-key shortcuts, mouse mode
- [ ] High-DPI and multi-monitor setups support
- [ ] Windows 7, XP(?) support
- [ ] Codebase refactoring

## Installation

Fully portable. Run install_extensions.bat to download the kokoro and bergamot models.

## Building

See GitHub Actions workflow.

## Some of the 3rd party content (esp. with required attribution) used in this repo:

- **bergamot-translate:** github.com/browsermt/bergamot-translator
- **kokoro-js:** github.com/hexgrad/kokoro
- **oggenc2:** rarewares.org/ogg-oggenc.php

**Icons:**
- **CyCraft Pepicons Print collection** Licensed under CC BY 4.0
- **Phosphor Icons** Licensed under MIT
- **Lets Icons** by Leonid Tsvetkov. Licensed under CC BY 4.0
- **CoreUI Free** by creativeLabs Łukasz Holeczek. Licensed under CC BY 4.0 License
- **IconPark Outline** by ByteDance. License: Apache 2.0
