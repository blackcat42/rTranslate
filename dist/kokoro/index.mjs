import { KokoroTTS } from "kokoro-js";
import { env } from '@huggingface/transformers';

import { spawn } from 'node:child_process';
import { unlinkSync } from 'node:fs';

// Specify a custom location for models
env.localModelPath = 'models/';
// Disable remote model fetching
env.allowRemoteModels = false;

const model_id = "onnx-community/Kokoro-82M-v1.0-ONNX";
const tts = await KokoroTTS.from_pretrained(model_id, {
  dtype: "fp16", // Options: "fp32", "fp16", "q8", "q4", "q4f16"
  device: "cpu", // Options: "wasm", "webgpu" (web) or "cpu" (node). If using "webgpu", we recommend using dtype="fp32".
});
//recommend fp16 or q4f16
//https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX/tree/main/onnx
//https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX/resolve/main/onnx/model_fp16.onnx?download=true

const p_args = process.argv;
let params = {};
p_args.forEach(arg => {
  const nameValue = arg.split("=");
  if (nameValue.length === 2) {
    params[nameValue[0]] = nameValue[1];
  } else {
    params[nameValue[0]] = ''; 
  }
});
let uid = params['--uid'] ? params['--uid'] : 'out';
let voice = params['--voice'] ? params['--voice'] : 'af_nicole';
if (voice == "AfHeart") voice = "af_heart";
//let uid = (params['--uid'] && params['--uid'].length > 1) ? params['--uid'] : 'out';
//let uid = (params['--kkr-voice'] && params['--kkr-voice'].length > 2) ? params['--kkr-voice'] : 'out';

const inputPath = 'tmp_audio.wav';
const outputPath = '../tts_cache/' + uid + '.ogg';



  process.stdin.on("data", async data => {
    let text = data.toString();
    //console.log(tts.list_voices())
    const audio = await tts.generate(text, {
      voice: voice,
    });
    audio.save(inputPath);

    //https://gregr.org/tts-samples/ https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX#voicessamples

    const args = [
      inputPath,
      '-Q',
      '-o', outputPath,      
    ];

    const oggencProcess = spawn('./oggenc2', args);

    oggencProcess.stdout.on('data', (data) => {
      console.log(`stdout: ${data}`);
    });

    oggencProcess.stderr.on('data', (data) => {
      console.error(`stderr: ${data}`);
      /*if (data.includes('already exists. Overwrite? [y/N]')) {
        console.log('Responding with "yes" to child process stdin...');
        process.stdin.write('y\n'); // Write 'yes' followed by a newline character
        //process.stdin.end(); // Close the stdin stream
      }*/
    });

    oggencProcess.on('close', (code) => {
      if (code === 0) {
        console.log(`Process closed with code ${code}. Conversion complete!`);
        try {
          unlinkSync(inputPath);
          console.log(inputPath +' File deleted successfully');
        } catch (err) {
          console.error('An error occurred:', err); // Handle potential errors
        }
        process.exit(0);
      } else {
        console.error(`Process closed with code ${code}. Conversion failed.`);
        process.exit(1);
      }
    });

    oggencProcess.on('error', (err) => {
        console.error('Failed to start oggenc process:', err);
        /*try {
          unlinkSync(inputPath);
          console.log(inputPath +' File deleted successfully');
        } catch (err) {
          console.error('An error occurred:', err);
        }*/
        process.exit(1);
    });
})



// The system includes 54 different voices across 8 languages:
// 🇺🇸 American English (20 voices)
// Language code: 'a'
// Female voices (af_*):
//     af_heart: ❤️ Premium quality voice (Grade A)
//     af_alloy: Clear and professional (Grade C)
//     af_aoede: Smooth and melodic (Grade C+)
//     af_bella: 🔥 Warm and friendly (Grade A-)
//     af_jessica: Natural and engaging (Grade D)
//     af_kore: Bright and energetic (Grade C+)
//     af_nicole: 🎧 Professional and articulate (Grade B-)
//     af_nova: Modern and dynamic (Grade C)
//     af_river: Soft and flowing (Grade D)
//     af_sarah: Casual and approachable (Grade C+)
//     af_sky: Light and airy (Grade C-)
// Male voices (am_*):
//     am_adam: Strong and confident (Grade F+)
//     am_echo: Resonant and clear (Grade D)
//     am_eric: Professional and authoritative (Grade D)
//     am_fenrir: Deep and powerful (Grade C+)
//     am_liam: Friendly and conversational (Grade D)
//     am_michael: Warm and trustworthy (Grade C+)
//     am_onyx: Rich and sophisticated (Grade D)
//     am_puck: Playful and energetic (Grade C+)
//     am_santa: Holiday-themed voice (Grade D-)

// 🇬🇧 British English (8 voices)
// Language code: 'b'
// Female voices (bf_*):
//     bf_alice: Refined and elegant (Grade D)
//     bf_emma: Warm and professional (Grade B-)
//     bf_isabella: Sophisticated and clear (Grade C)
//     bf_lily: Sweet and gentle (Grade D)
// Male voices (bm_*):
//     bm_daniel: Polished and professional (Grade D)
//     bm_fable: Storytelling and engaging (Grade C)
//     bm_george: Classic British accent (Grade C)
//     bm_lewis: Modern British accent (Grade D+)

// 🇯🇵 Japanese (5 voices)
// Language code: 'j'
// Female voices (jf_*):
//     jf_alpha: Standard Japanese female (Grade C+)
//     jf_gongitsune: Based on classic tale (Grade C)
//     jf_nezumi: Mouse bride tale voice (Grade C-)
//     jf_tebukuro: Glove story voice (Grade C)
// Male voices (jm_*):
//     jm_kumo: Spider thread tale voice (Grade C-)

// 🇨🇳 Mandarin Chinese (8 voices)
// Language code: 'z'
// Female voices (zf_*):
//     zf_xiaobei: Chinese female voice (Grade D)
//     zf_xiaoni: Chinese female voice (Grade D)
//     zf_xiaoxiao: Chinese female voice (Grade D)
//     zf_xiaoyi: Chinese female voice (Grade D)
// Male voices (zm_*):
//     zm_yunjian: Chinese male voice (Grade D)
//     zm_yunxi: Chinese male voice (Grade D)
//     zm_yunxia: Chinese male voice (Grade D)
//     zm_yunyang: Chinese male voice (Grade D)
// Note: For Chinese TTS setup and usage, see CHINESE_TTS_GUIDE.md or README_CHINESE_TTS.md.

// 🇪🇸 Spanish (3 voices)
// Language code: 'e'
// Female voices (ef_*):
//     ef_dora: Spanish female voice
// Male voices (em_*):
//     em_alex: Spanish male voice
//     em_santa: Spanish holiday voice

// 🇫🇷 French (1 voice)
// Language code: 'f'
// Female voices (ff_*):
//     ff_siwis: French female voice (Grade B-)

// 🇮🇳 Hindi (4 voices)
// Language code: 'h'
// Female voices (hf_*):
//     hf_alpha: Hindi female voice (Grade C)
//     hf_beta: Hindi female voice (Grade C)
// Male voices (hm_*):
//     hm_omega: Hindi male voice (Grade C)
//     hm_psi: Hindi male voice (Grade C)

// 🇮🇹 Italian (2 voices)
// Language code: 'i'
// Female voices (if_*):
//     if_sara: Italian female voice (Grade C)
// Male voices (im_*):
//     im_nicola: Italian male voice (Grade C)

// 🇧🇷 Brazilian Portuguese (3 voices)
// Language code: 'p'
// Female voices (pf_*):
//     pf_dora: Portuguese female voice
// Male voices (pm_*):
//     pm_alex: Portuguese male voice
//     pm_santa: Portuguese holiday voice

// Note: Quality grades (A to F) indicate the overall quality based on training data quality and duration. Higher grades generally produce better speech quality.



// ┌─────────────┬────────────┬──────────┬──────────┬──────────────┬──────────────┐
// │ (index)     │ name       │ language │ gender   │targetQuality │ overallGrade │
// ├─────────────┼────────────┼──────────┼──────────┼──────────────┼──────────────┤
// │ af_heart    │ 'Heart'    │ 'en-us'  │ 'Female' │ 'A'          │ 'A'          │
// │ af_alloy    │ 'Alloy'    │ 'en-us'  │ 'Female' │ 'B'          │ 'C'          │
// │ af_aoede    │ 'Aoede'    │ 'en-us'  │ 'Female' │ 'B'          │ 'C+'         │
// │ af_bella    │ 'Bella'    │ 'en-us'  │ 'Female' │ 'A'          │ 'A-'         │
// │ af_jessica  │ 'Jessica'  │ 'en-us'  │ 'Female' │ 'C'          │ 'D'          │
// │ af_kore     │ 'Kore'     │ 'en-us'  │ 'Female' │ 'B'          │ 'C+'         │
// │ af_nicole   │ 'Nicole'   │ 'en-us'  │ 'Female' │ 'B'          │ 'B-'         │
// │ af_nova     │ 'Nova'     │ 'en-us'  │ 'Female' │ 'B'          │ 'C'          │
// │ af_river    │ 'River'    │ 'en-us'  │ 'Female' │ 'C'          │ 'D'          │
// │ af_sarah    │ 'Sarah'    │ 'en-us'  │ 'Female' │ 'B'          │ 'C+'         │
// │ af_sky      │ 'Sky'      │ 'en-us'  │ 'Female' │ 'B'          │ 'C-'         │
// │ am_adam     │ 'Adam'     │ 'en-us'  │ 'Male'   │ 'D'          │ 'F+'         │
// │ am_echo     │ 'Echo'     │ 'en-us'  │ 'Male'   │ 'C'          │ 'D'          │
// │ am_eric     │ 'Eric'     │ 'en-us'  │ 'Male'   │ 'C'          │ 'D'          │
// │ am_fenrir   │ 'Fenrir'   │ 'en-us'  │ 'Male'   │ 'B'          │ 'C+'         │
// │ am_liam     │ 'Liam'     │ 'en-us'  │ 'Male'   │ 'C'          │ 'D'          │
// │ am_michael  │ 'Michael'  │ 'en-us'  │ 'Male'   │ 'B'          │ 'C+'         │
// │ am_onyx     │ 'Onyx'     │ 'en-us'  │ 'Male'   │ 'C'          │ 'D'          │
// │ am_puck     │ 'Puck'     │ 'en-us'  │ 'Male'   │ 'B'          │ 'C+'         │
// │ am_santa    │ 'Santa'    │ 'en-us'  │ 'Male'   │ 'C'          │ 'D-'         │
// │ bf_emma     │ 'Emma'     │ 'en-gb'  │ 'Female' │ 'B'          │ 'B-'         │
// │ bf_isabella │ 'Isabella' │ 'en-gb'  │ 'Female' │ 'B'          │ 'C'          │
// │ bm_george   │ 'George'   │ 'en-gb'  │ 'Male'   │ 'B'          │ 'C'          │
// │ bm_lewis    │ 'Lewis'    │ 'en-gb'  │ 'Male'   │ 'C'          │ 'D+'         │
// │ bf_alice    │ 'Alice'    │ 'en-gb'  │ 'Female' │ 'C'          │ 'D'          │
// │ bf_lily     │ 'Lily'     │ 'en-gb'  │ 'Female' │ 'C'          │ 'D'          │
// │ bm_daniel   │ 'Daniel'   │ 'en-gb'  │ 'Male'   │ 'C'          │ 'D'          │
// │ bm_fable    │ 'Fable'    │ 'en-gb'  │ 'Male'   │ 'B'          │ 'C'          │
// └─────────────┴────────────┴──────────┴──────────┴──────────────┴──────────────┘