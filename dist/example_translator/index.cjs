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

    //...process translation...

    const output = 'This is an example. Source text:' + data + '\n Lang (src): ' + lang_src + ' Lang (target): ' + lang_target;
    process.stdout.write(output.replace(/\r\n|\r|\n/gm, '<ENDOFLINE>') + "\n")

})
  