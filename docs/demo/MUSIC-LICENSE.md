# Product tour audio provenance

## Original ambient bed

The original music-only and cinematic voiceover tours use an ambient bed titled **Ledger Light**. It was generated for Spool Ledger on 2026-08-24 with FFmpeg 8.0.1 from synthesized sine oscillators, slow amplitude modulation, filtering, and a short algorithmic echo.

No third-party song, recording, performance, sample pack, loop, or music library was used for **Ledger Light**.

To the extent copyright or related rights exist in **Ledger Light**, the project owner dedicates the composition and generated recording to the public domain under [CC0 1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/).

## Simple live tour music

The simple live voiceover tour uses **Technology - Tech Technology 90 Second** by **BombinSound**:

- [Track page](https://pixabay.com/music/electronic-technology-tech-technology-90-second-499581/)
- [Pixabay Content License summary](https://pixabay.com/service/license-summary/)

The track is used under the Pixabay Content License. That license allows use and adaptation in a produced video without required attribution. It does not allow standalone redistribution of the substantially unchanged track, so the downloaded source MP3 is not included in this repository. This project does not claim that BombinSound or Pixabay endorses Spool Ledger.

## AI voiceovers

The cinematic voiceover cut uses AI-generated speech from OpenAI's `gpt-4o-mini-tts` model with the `cedar` voice. Its narration is recorded in [VOICEOVER-SCRIPT.md](VOICEOVER-SCRIPT.md).

The simple live cut uses OpenAI's `gpt-audio-1.5` model with the `cedar` voice. OpenAI documents this as its best voice model for audio input and output through Chat Completions. Its A2/B1 narration and readability measurements are recorded in [SIMPLE-VOICEOVER-SCRIPT.md](SIMPLE-VOICEOVER-SCRIPT.md). The same narration is available separately as [spool-ledger-simple-voiceover.wav](spool-ledger-simple-voiceover.wav).

Both voiceovers are AI-generated and do not represent a human speaker recording. The OpenAI API credential was used only for generation requests. It is not stored in this repository, the narration files, or the product-tour videos.
