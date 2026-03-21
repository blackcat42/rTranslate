#!/usr/bin/env node

/**
 * A note upfront: the bergamot-translator API is pretty low level, and
 * embedding it successfully requires some knowledge about the WebWorkers and
 * WebAssembly APIs. This script tries to demonstrate the bergamot-translator
 * API with as little of that boiler plate code as possible.
 * See the wasm/test_page code for a fully fleshed out demo in a web context.
 */


//TODO: source language -> english -> target language

const fs = require('fs');

// Read wasm binary into a blob
const wasmBinary = fs.readFileSync('./bergamot-translator-worker.wasm');

// Read wasm runtime code that bridges the bergmot-translator binary with JS.
const wasmRuntime = fs.readFileSync('./bergamot-translator-worker.js', {encoding: 'utf8'});

// Initialise the `Module` object. By adding methods and options to this, we can
// affect how bergamot-translator interacts with JavaScript. See 
// https://emscripten.org/docs/api_reference/module.html for all available
// options. It is important that this object is initialised in the same scope
// but before `bergamot-translation-worker.js` is executed. Once that script
// executes, it defines the exported methods as properties of this Module
// object.
global.Module = {
  wasmBinary,
  onRuntimeInitialized
};

// Execute bergamot-translation-worker.js in this scope. This will also,
// indirectly, call the onRuntimeInitialized function defined below and
// referenced in the `Module` object above.
eval.call(global, wasmRuntime);



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
let lang_pair = 'enru';
let lang_src = params['--src'] ? params['--src'] : 'en';
let lang_target = params['--target'] ? params['--target'] : 'ru';

if (lang_src !== 'en' && lang_target !== 'en') {
  lang_target = 'en'; //TODO
}

lang_src = lang_src.toLowerCase();
lang_target = lang_target.toLowerCase();
lang_pair = lang_src + lang_target;



/**
 * Called from inside the bergamot-translation-worker.js script once the wasm
 * module is initialized. At this point that `Module` object that was
 * initialised above will have all the classes defined in the
 * bergamot-translator API available on it.
 */
async function onRuntimeInitialized() {
  // Root url for our models for now.
  //const root = 'https://storage.googleapis.com/bergamot-models-sandbox/0.3.1';

  // Urls of data files necessary to create a translation model for
  // English -> German. Note: list is in order of TranslationModel's arguments.
  // The `alignment` value is used later on to load each part of the model with
  // the correct alignment.

  //console.log(lang_pair)
  let files = [
    // Neural network and weights:
    {url: './models/model.'+lang_pair+'.intgemm.alphas.bin', alignment: 256},
    
    // Lexical shortlist which is mainly a speed improvement method, not
    // strictly necessary:
    {url: './models/lex.50.50.'+lang_pair+'.s2t.bin', alignment: 64},
    
    // Vocabulary, maps the input and output nodes of the neural network to
    // strings. Note: "deen" may look the wrong way around but vocab is the same
    // between de->en and en->de models.
    {url: './models/vocab.'+lang_pair+'.spm', alignment: 64},
  ];


  // Download model data and load it into aligned memory. AlignedMemory is a
  // necessary wrapper around allocated memory inside the WASM environment.
  // The value of `alignment` is specific for which part of the model we're
  // loading. See https://en.wikipedia.org/wiki/Data_structure_alignment for a
  // more general explanation.
  const [modelMem, shortlistMem, vocabMem] = await Promise.all(files.map(async (file) => {
    try {
      const response = fs.readFileSync(file.url);
      //const blob = await response.blob();
      //const buffer = await response.arrayBuffer();
      const bytes = new Int8Array(response);
      const memory = new Module.AlignedMemory(bytes.byteLength, file.alignment);
      memory.getByteArrayView().set(bytes);
      return memory;
    } catch (error) {
      console.error('Error:', error);
      //process.stdout.write('Error reading file');
      process.exit(1);
    }
  }));

  /*const modelMem = fs.readFileSync('./model.enru.intgemm.alphas.bin');
  const shortlistMem = fs.readFileSync('./lex.50.50.enru.s2t.bin');
  const vocabMem = fs.readFileSync('./vocab.enru.spm');*/

  // Set up translation service. This service translates a batch of text per
  // call. The larger the batch, the faster the translation (in words per
  // second) happens, but the longer you have to wait for all of them to finish.
  // The constructor expects an object with options, but only one option is
  // currently supported: `cacheSize`. Setting this to `0` disables the
  // translation cache.
  // **Note**: cacheSize is the theoretical maximum number of sentences that
  // will be cached. In practise, about 1/3 of that will actually be used.
  // See https://github.com/XapaJIaMnu/translateLocally/pull/75
  const service = new Module.BlockingService({cacheSize: 0});

  // Put vocab into its own std::vector<AlignedMemory>. Most models for the
  // Bergamot project only have one vocabulary that is shared by both the input
  // and output side of the translator. But in theory, you could have one for
  // the input side and a different one for the output side. Hence: a list.
  const vocabs = new Module.AlignedMemoryList();
  vocabs.push_back(vocabMem);

  // Config yaml (split as array to allow for indentation without adding tabs
  // or spaces to the strings themselves.)
  // See https://marian-nmt.github.io/docs/cmd/marian-decoder/ for the meaning
  // of most of these options and what other options might be available.
  const config = [
    'beam-size: 1',
    'normalize: 1.0',
    'word-penalty: 0',
    'alignment: soft', // is necessary if you want to use HTML at any point
    'max-length-break: 128',
    'mini-batch-words: 1024',
    'workspace: 128',
    'max-length-factor: 2.0',
    'skip-cost: true',
    //'batchSize: 1',
    'gemm-precision: int8shiftAll', // is necessary for speed and compatibility with Mozilla's models.
  ].join('\n');

  // Setup up model with config yaml and AlignedMemory objects. Optionally a
  // quality estimation model can also be loaded but this is not demonstrated
  // here. Generally you don't need it, and many models don't include the data
  // file necessary to use it anyway.
  const model = new Module.TranslationModel(config, modelMem, shortlistMem, vocabs, /*qualityModel=*/ null);

  // Construct std::vector<std::string> inputs; This is our batch!

  let count = 0;
  process.stdin.on("data", data => {
    const input = new Module.VectorString();
    const options = new Module.VectorResponseOptions();
    options.push_back({qualityScores: false, alignment: false, html: false});
    data = data.toString();
    //if (count < 1) return;

    input.push_back(data);
    
    //input.push_back(' Translator best suited for interactive usage. Runs with a single worker thread and a batch-size of 1 to give you a response as quickly as possible. It will cancel any pending translations that arent currently being processed if you submit a new one.');

    
    //options.push_back({qualityScores: false, alignment: false, html: false});
    //console.assert(input.size() === options.size());

    const output = service.translate(model, input, options);
    //console.assert(input.size() === output.size());

    let aaa = "";
    for (let i = 0; i < output.size(); ++i) {
      // Get output from std::vector<Response>.
      const translation = output.get(i).getTranslatedText();
      aaa += translation;
      // Print raw translation for inspection.
      //console.log(output)
      //process.stdout.write(translation + "\n")
      process.stdout.write(translation.replace(/\r\n|\r|\n/gm, '<ENDOFLINE>') + "\n")
    }
    //process.stdout.write(aaa.replace(/[\r\n]/gm, '') + "\n")
    
    input.delete();
    options.delete();
    output.delete();
  })
  

  // Construct std::vector<ResponseOptions>, one entry per input. Note that
  // all these three properties of your ResponseOptions object need to be
  // specified for each entry.
  // `qualityScores`: related to quality models not explained here. Set this
  //   to `false`.
  // `alignment`: computes alignment scores that maps parts of the input text
  //   to parts of the output text. There is currently no way to get these
  //   mappings out through the JavaScript API so I suggest you set this to
  //   `false` as well.
  // `html`: is the input HTML? If so, the HTML will be parsed and the markup
  //   will be copied back into the translated output. Note: HTML has to be
  //   valid HTML5, with proper closing tags and everything since the HTML
  //   parser built into bergamot-translator does no error correction. Output
  //   of e.g. `Element.innerHTML` meets this criteria.
  
  //options.push_back({qualityScores: false, alignment: false, html: true});
  

  // Size of `input` and `options` has to match.
  

  // Translate our batch of 2 requests. Output will be another vector of type 
  // `std::vector<Response>`.
  

  //console.assert(false);

  // Number of outputs is number of inputs.
  

  

  // Clean-up: unlike the objects in JavaScript, the objects in the WASM
  // environment are not automatically cleaned up when they're no longer
  // referenced. That is why we manually have to call `delete()` on them
  // when we're done with them.
  
}
