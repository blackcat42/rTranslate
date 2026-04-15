#!/usr/bin/env node


const fs = require('fs');
const readline = require('readline');

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

let lang_src = params['--src'] ? params['--src'] : 'en';
let lang_target = params['--target'] ? params['--target'] : 'ru';


lang_src = lang_src.toLowerCase();
lang_target = lang_target.toLowerCase();


//ERROR CODES:
//73 - Language error (unsupported) 
//53 - service error
//0 - success (no errors)

//process.exit(53);

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

rl.on('line', (data) => {
    data = data.toString();
    data = data.replace('<ENDOFLINE>', '\n');

    let language_autodetect = false;
    const regex_s = /<SRC_LANG=(..)>/;
    const match_s = data.match(regex_s);
    const srcLang = match_s ? match_s[1] : null;
    if (match_s) {
        data = data.replace(match_s[0], "");
    }
    const regex_a = /<SRC_LANG=auto>/;
    const match_a = data.match(regex_a);
    if (match_a) {
        language_autodetect = true;
        data = data.replace(match_a[0], "");
    }
    const regex_t = /<TARGET_LANG=(..)>/;
    const match_t = data.match(regex_t);
    const targetLang = match_t ? match_t[1] : null;
    if (match_t) {
        data = data.replace(match_t[0], "");
    }

    const regex_id = /<SRC_ID=(\d+)>/;
    const match_id = data.match(regex_id);
    const src_id = match_id ? match_id[1] : null;
    if (match_id) {
        data = data.replace(match_id[0], "");
    }
    

    //...process translation...
    let src_id_str = "";
    if (src_id !== null) {
        src_id_str = '<SRC_ID=' + src_id + '>';
    } else {
        process.exit(53);
    }

    const output = src_id_str + '<SRC_LANG_DETECTED=es>' + 'This is an example. Source text:' + data + '\n Lang (src) from args: ' + lang_src + ' Lang (target) from args: ' + lang_target + ' Lang (src) from stdin: ' + srcLang + ' Lang (target) from stdin: ' + targetLang;
    process.stdout.write(output.replace(/\r\n|\r|\n/gm, '<ENDOFLINE>') + "\n")

})
  