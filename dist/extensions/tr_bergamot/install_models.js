//const fs = require('fs');
//import { fs, zlib } from 'node';
import * as fs from 'node:fs';
import * as readline from 'node:readline';
import * as process from 'node:process';
import * as https from 'node:https';
import * as zlib from 'node:zlib';

const rankedArchitectures = ['base', 'base-memory', 'tiny'];

async function run() {
    let languageNames = new Intl.DisplayNames(['en'], {type: 'language'});

    const r = await fetch(
        'https://storage.googleapis.com/moz-fx-translations-data--303e-prod-translations-data/db/models.json',
    );
    const data = await r.json();
    console.log('Available supported language pairs:');
    Object.keys(data.models).forEach(key => {
        let from = '';
        let to = '';
        try {
            from = languageNames.of(data.models[key][0].sourceLanguage);
            to = languageNames.of(data.models[key][0].targetLanguage);
        } catch (e) {}
        console.log('\'' + key + '\' (from: ' + from + ', to: ' + to + ')');
    });
    //Object.keys(data.models)
    //console.log(Object.keys(data.models));

    //const config = JSON.parse(data);

    console.log('');
    console.log('preffered archicture: ' + rankedArchitectures[0]);
    console.log('');

    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
    });
    rl.question('Enter a comma-separated list of lang-pairs(e.g.: \'en-es, es-en\' to download translation models): ', (userInput) => {
      const inputArray = userInput.trim().split(',');
      const trimmedArray = inputArray.map(item => item.trim().replaceAll(/["']/g, ''));

      console.log('Original input string:', userInput);
      console.log('Parsed array:', trimmedArray);

      rl.close();
      downloadModels(data, trimmedArray);
    });
}

async function downloadModels(data, req) {

    req.forEach((item) => {
        if (!data.models[item]) {
            console.log('Lang pair \'' + userInput + '\' not found');
            return;
        }

        let langpair_in_filename = item.split('-').join('');

        let zxc = data.models[item];
        zxc.sort((a, b) => {
            const rankA = rankedArchitectures.indexOf(a.architecture);
            const rankB = rankedArchitectures.indexOf(b.architecture);
            return rankA - rankB;
        });

        let lex =
            'https://storage.googleapis.com/moz-fx-translations-data--303e-prod-translations-data/' +
            zxc[0].files.lexicalShortlist.path;
        let model =
            'https://storage.googleapis.com/moz-fx-translations-data--303e-prod-translations-data/' +
            zxc[0].files.model.path;
        let vocab =
            'https://storage.googleapis.com/moz-fx-translations-data--303e-prod-translations-data/' +
            zxc[0].files.vocab.path;

        const vocab_path = 'models/' + getFilenameFromString(vocab);
        downloadModel(vocab, vocab_path);
        const model_path = 'models/' + getFilenameFromString(model);
        downloadModel(model, model_path);
        const lex_path = 'models/' + getFilenameFromString(lex);
        downloadModel(lex, lex_path);

        //https://storage.googleapis.com/moz-fx-translations-data--303e-prod-translations-data/db/models.json
        //https://storage.googleapis.com/moz-fx-translations-data--303e-prod-translations-data/
    });
}

function getFilenameFromString(uriString) {
    let filename = uriString.split('?')[0].split('#')[0];
    filename = filename.substring(filename.lastIndexOf('/') + 1);
    return filename;
}

function downloadModel(fileUrl, path) {
    console.log(fileUrl);
    console.log(path);
    //return;
    if (!path) return;
    const file = fs.createWriteStream(path);

    const request = https
        .get(fileUrl, (response) => {
            response.pipe(file);
            file.on('finish', () => {
                file.close(() => {
                    console.log('File downloaded successfully');
                });

                const gunzip = zlib.createGunzip();

                const input = fs.createReadStream(path);
                const output = fs.createWriteStream(path.replaceAll('.gz', ''));

                input
                    .pipe(gunzip)
                    .pipe(output)
                    .on('finish', () => {
                        fs.unlink(path, () => {
                            console.log('Delete file: ' + path);
                        });
                        console.log('file execution finished');
                    });

            });
        })
        .on('error', (err) => {
            fs.unlink(path, () => {
                // Delete the partially downloaded file
                console.error('Error downloading file:', err.message);
            });
        });
    request.on('error', (err) => {
        console.error('Error making request:', err.message);
    });
}

run();
