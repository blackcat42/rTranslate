(
    //A polyfill for base64 functions atob and btoa.
    //The MIT License (MIT)
    //Copyright (c) 2014 MaxArt2501
    //https://github.com/MaxArt2501/base64-js/blob/master/base64.js

    function (root, factory) {
        if (typeof define === 'function' && define.amd) {
            // AMD. Register as an anonymous module.
            define([], function() {factory(root);});
        } else factory(root);
    // node.js has always supported base64 conversions, while browsers that support
    // web workers support base64 too, but you may never know.
    }
)(typeof exports !== "undefined" ? exports : this, function(root) {
    if (root.atob) {
        // Some browsers' implementation of atob doesn't support whitespaces
        // in the encoded string (notably, IE). This wraps the native atob
        // in a function that strips the whitespaces.
        // The original function can be retrieved in atob.original
        try {
            root.atob(" ");
        } catch(e) {
            root.atob = (function(atob) {
                var func = function(string) {
                    return atob(String(string).replace(/[\t\n\f\r ]+/g, ""));
                };
                func.original = atob;
                return func;
            })(root.atob);
        }
        return;
    }

        // base64 character set, plus padding character (=)
    var b64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=",
        // Regular expression to check formal correctness of base64 encoded strings
        b64re = /^(?:[A-Za-z\d+\/]{4})*?(?:[A-Za-z\d+\/]{2}(?:==)?|[A-Za-z\d+\/]{3}=?)?$/;

    root.btoa = function(string) {
        string = String(string);
        var bitmap, a, b, c,
            result = "", i = 0,
            rest = string.length % 3; // To determine the final padding

        for (; i < string.length;) {
            if ((a = string.charCodeAt(i++)) > 255
                    || (b = string.charCodeAt(i++)) > 255
                    || (c = string.charCodeAt(i++)) > 255)
                throw new TypeError("Failed to execute 'btoa' on 'Window': The string to be encoded contains characters outside of the Latin1 range.");

            bitmap = (a << 16) | (b << 8) | c;
            result += b64.charAt(bitmap >> 18 & 63) + b64.charAt(bitmap >> 12 & 63)
                    + b64.charAt(bitmap >> 6 & 63) + b64.charAt(bitmap & 63);
        }

        // If there's need of padding, replace the last 'A's with equal signs
        return rest ? result.slice(0, rest - 3) + "===".substring(rest) : result;
    };

    root.atob = function(string) {
        // atob can work with strings with whitespaces, even inside the encoded part,
        // but only \t, \n, \f, \r and ' ', which can be stripped.
        string = String(string).replace(/[\t\n\f\r ]+/g, "");
        if (!b64re.test(string))
            throw new TypeError("Failed to execute 'atob' on 'Window': The string to be decoded is not correctly encoded.");

        // Adding the padding if missing, for semplicity
        string += "==".slice(2 - (string.length & 3));
        var bitmap, result = "", r1, r2, i = 0;
        for (; i < string.length;) {
            bitmap = b64.indexOf(string.charAt(i++)) << 18 | b64.indexOf(string.charAt(i++)) << 12
                    | (r1 = b64.indexOf(string.charAt(i++))) << 6 | (r2 = b64.indexOf(string.charAt(i++)));

            result += r1 === 64 ? String.fromCharCode(bitmap >> 16 & 255)
                    : r2 === 64 ? String.fromCharCode(bitmap >> 16 & 255, bitmap >> 8 & 255)
                    : String.fromCharCode(bitmap >> 16 & 255, bitmap >> 8 & 255, bitmap & 255);
        }
        return result;
    };
});
/*function base64encode(text) {
	return btoa(unescape(encodeURIComponent(stringifyJSON(text))));
}*/
function base64decode(base64) {
	return decodeURIComponent(escape(atob(base64)));
}

function cleanIDString(str) {
    if (typeof str !== "string") {
        return "";
    }
    return str.replace(/[^a-zA-Z0-9_а-яА-ЯёЁ\-\s]/g, "");
}

function getArgValue(argName) {
  const index = s_args.findIndex(arg => arg === `--${argName}`);
  if (index === -1) return null;
  const nextArg = s_args[index + 1];
  
  if (nextArg && !nextArg.startsWith("-")) {
    return nextArg;
  }
  return true;
}

let s_args;
let qtranslate_eval = '';
/*if (typeof Deno !== "undefined") {
	eval(`
		let f1 = Deno.readTextFileSync("./extensions/qtranslate/Services/Common.js", "utf-8");
		let f2 = Deno.readTextFileSync("./extensions/qtranslate/Services/DeepL/Service.js", "utf-8");

		const globalEval = (0, eval);
	    globalEval(f1);
	    globalEval(f2);

		s_args = Deno.args;

	`);
} else*/ 
if (typeof std !== "undefined" && typeof std.loadScript === "function") {
	s_args = scriptArgs;
	let srvc_id = cleanIDString(getArgValue("service-id"));
	std.loadScript('./extensions/qtranslate/Services/Common.js');
	std.loadScript('./extensions/qtranslate/Services/' + srvc_id + '/Service.js');
	
    let qt_eval = getArgValue("qtranslate-eval");
    if (qt_eval == 'FILE') {
        std.loadScript('./qjs_tmp.js');
        qtranslate_eval = eval_data;
    } else {
        qtranslate_eval = qt_eval;
    }

	//TODO no std, make temporary file common.js + service.js
	std = null;
	os = null;
} else {
    console.log("TODO nodejs");
}



//"zh-CN","zh-TW"
addOption('PreferredDomain', 'com');
addOption('GoogleDomain', 'com'); //cn
addOption('LanguageCode', 'ru'); //TODO!!! target lang?
//Options.GoogleTkk
//Options.BingToken
//Options.BingCookie 
//Options.BingKey 
//Options.IG 

let sh = serviceHeader();

let request_type = cleanIDString(getArgValue("type"));

if (sh.name === 'Yandex' && sh.id === 11) {
	//TODO: <QT_STATE>
	if (typeof YandexModel === "function"){
		YandexModel.prototype.hasChunks = function () {
			return false;
		};
		yandexModel = new YandexModel('');
	}
	//TODO detect lng
}



//TRANSLATE:1,DETECT_LANGUAGE:2,LISTEN:4,DICTIONARY:8
if (request_type == Capability.TRANSLATE) {
    if (!!(sh.capabilities & Capability.TRANSLATE)) {
        let arg_decoded = base64decode(qtranslate_eval);
        //console.log("QT-args:", arg_decoded);
        let result = eval(arg_decoded);
        if (result.uri !== undefined && !result.uri.startsWith("https://")) {
            result.uri = serviceHost(Capability.TRANSLATE) + result.uri;
        }
        console.log('<SERVICE_RESPONSE_BEGIN>');
        console.log(stringifyJSON(result));
        console.log('<SERVICE_RESPONSE_END>');
    }
} else if (request_type == Capability.LISTEN) {
    if (!!(sh.capabilities & Capability.LISTEN)) {
        /*let qtranslate_eval = getArgValue("qtranslate-eval");
        let arg_decoded = base64decode(qtranslate_eval);
        //console.log("QT-args:", arg_decoded);
        let result = eval(arg_decoded);
        if (result.uri !== undefined) {
            result.uri = serviceHost(Capability.LISTEN) + result.uri;
        }
        console.log('<SERVICE_RESPONSE_BEGIN>');
        console.log(stringifyJSON(result));
        console.log('<SERVICE_RESPONSE_END>');*/
    }
} else if (request_type == Capability.DICTIONARY) {
    if (!!(sh.capabilities & Capability.DICTIONARY)) {
        let arg_decoded = base64decode(qtranslate_eval);
        //console.log("QT-args:", arg_decoded);
        let result = eval(arg_decoded);
        if (result.uri !== undefined && !result.uri.startsWith("https://")) {
            result.uri = serviceHost(Capability.DICTIONARY) + result.uri;
        }
        console.log('<SERVICE_RESPONSE_BEGIN>');
        console.log(stringifyJSON(result));
        console.log('<SERVICE_RESPONSE_END>');
    }
}

//https://tts.voicetech.yandex.net/tts?format=mp3&quality=hi&platform=web&application=translate&lang=en&text=Hello